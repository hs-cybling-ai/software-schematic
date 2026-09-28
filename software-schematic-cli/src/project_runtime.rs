use crate::{
    AppState, Error, Result,
    delivery_protocol::{
        ApplicationLease, BUILD_PLAN_PROTOCOL_VERSION, BrowserSession, BuildClaim,
        BuildContextCitation, BuildContextEntity, BuildContextRelation, BuildPlanEvidence,
        BuildPlanManifest, BuildPlanProgress, BuildPlanRecord, BuildPlanState, BuildPlanSummary,
        BuildRequest, BuildState, BuildUpdate, BuildWorkItem, BuildWorkItemContext,
        BuildWorkItemState, CompleteBuildPlanRequest, CreateBuildPlanRequest,
        DAEMON_PROTOCOL_VERSION, DaemonDiscovery, DaemonHealth, GENERATED_CONTRACT_SCHEMA_VERSION,
        GeneratedContractMetadata, GeneratedContractSubmission, InterviewRecord,
        PrivateDaemonHello, ProposalDecision, ProposalDecisionKind, ProposalRecord,
        ProposalResumeSummary, ProposalState, RuntimeEvent, RuntimeWorkspaceSnapshot, SemanticBump,
        SemanticVersion, WorkspaceLaunchState, serialize_generated_contract,
        validate_browser_session, validate_build_plan, validate_generated_contract_submission,
        validate_interview, validate_proposal, validate_runtime_workspace,
    },
    schematic_graph::LoadOptions,
    schematic_mcp::SchematicMcp,
};
use rand::RngCore;
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{Mutex, RwLock};

const MAX_EVENTS: usize = 512;
const SESSION_TIMEOUT_MS: u128 = 90_000;
const WORKSPACE_READY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

pub trait BrowserLauncher: Send + Sync {
    fn open(&self, target: &str) -> std::result::Result<(), String>;
}

#[derive(Default)]
pub struct SystemBrowserLauncher;

impl BrowserLauncher for SystemBrowserLauncher {
    fn open(&self, target: &str) -> std::result::Result<(), String> {
        open::that(target)
            .map(|_| ())
            .map_err(|_| "the system browser could not be opened".into())
    }
}

fn now_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn random_hex(bytes: usize) -> String {
    let mut value = vec![0u8; bytes];
    rand::rng().fill_bytes(&mut value);
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn project_id(project: &Path) -> Result<String> {
    let value = fs::read_to_string(project.join(".ss/project-id"))?;
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::Message(".ss/project-id is empty".into()));
    }
    Ok(value.into())
}

fn root_hash(project: &Path) -> String {
    format!("{:x}", Sha256::digest(project.to_string_lossy().as_bytes()))
}

fn workflow_root(project: &Path) -> PathBuf {
    project.join(".ss/workflows")
}

fn atomic_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(&serde_json::to_vec_pretty(value).unwrap())?;
        file.sync_all()?;
    }
    fs::rename(temporary, path)?;
    Ok(())
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| Error::Message(format!("{}: {error}", path.display()))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

enum VersionDecision {
    Resume(Box<BuildPlanRecord>),
    Create(SemanticVersion),
}

fn deterministic_work_item_id(entity_ref: &str) -> String {
    format!(
        "work-{}",
        &format!("{:x}", Sha256::digest(entity_ref.as_bytes()))[..16]
    )
}

fn build_work_items(envelope: &crate::schematic_graph::PlanScopeEnvelope) -> Vec<BuildWorkItem> {
    let target_ids = envelope
        .targets
        .iter()
        .map(|entity| (entity.id.as_str(), deterministic_work_item_id(&entity.id)))
        .collect::<BTreeMap<_, _>>();
    let contract_refs = envelope
        .contract_outputs
        .iter()
        .map(|output| output.entity_ref.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let global_context = envelope
        .context_only
        .iter()
        .map(|entity| entity.id.clone())
        .collect::<Vec<_>>();
    let mut items = envelope
        .targets
        .iter()
        .map(|entity| {
            let contract = contract_refs.contains(entity.id.as_str());
            let mut dependencies = envelope
                .relations
                .iter()
                .filter(|relation| {
                    relation.source == entity.id
                        && matches!(
                            relation.relation_type.as_str(),
                            "SOURCE" | "TARGET" | "ATTACHED_TO"
                        )
                })
                .filter_map(|relation| target_ids.get(relation.target.as_str()).cloned())
                .collect::<Vec<_>>();
            if let Some(composed_diagram) = envelope
                .relations
                .iter()
                .find(|relation| {
                    relation.source == entity.id && relation.relation_type == "COMPOSES_TO"
                })
                .map(|relation| relation.target.clone())
            {
                let mut descendants = std::collections::BTreeSet::from([composed_diagram]);
                loop {
                    let next = envelope
                        .relations
                        .iter()
                        .filter(|relation| {
                            relation.relation_type == "CONTAINS"
                                && descendants.contains(&relation.source)
                                && !descendants.contains(&relation.target)
                        })
                        .map(|relation| relation.target.clone())
                        .collect::<Vec<_>>();
                    if next.is_empty() {
                        break;
                    }
                    descendants.extend(next);
                }
                dependencies.extend(
                    descendants
                        .iter()
                        .filter_map(|id| target_ids.get(id.as_str()).cloned()),
                );
            }
            dependencies.sort();
            dependencies.dedup();
            BuildWorkItem {
                work_item_id: deterministic_work_item_id(&entity.id),
                entity_ref: entity.id.clone(),
                label: if entity.label.trim().is_empty() {
                    entity
                        .source_id
                        .clone()
                        .unwrap_or_else(|| entity.id.clone())
                } else {
                    entity.label.clone()
                },
                element_type: entity.element_type.clone(),
                phase: if contract { 2 } else { 1 },
                depends_on: dependencies,
                context_refs: global_context.clone(),
                generated_contract: contract,
            }
        })
        .collect::<Vec<_>>();
    let mut ordered = Vec::with_capacity(items.len());
    let mut resolved = std::collections::BTreeSet::new();
    while !items.is_empty() {
        let index = items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                item.depends_on
                    .iter()
                    .all(|dependency| resolved.contains(dependency))
            })
            .min_by_key(|(_, item)| (item.phase, item.work_item_id.clone()))
            .map(|(index, _)| index)
            .unwrap_or_else(|| {
                items
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, item)| (item.phase, item.work_item_id.clone()))
                    .map(|(index, _)| index)
                    .expect("items is non-empty")
            });
        let mut item = items.remove(index);
        item.depends_on
            .retain(|dependency| resolved.contains(dependency));
        resolved.insert(item.work_item_id.clone());
        ordered.push(item);
    }
    items = ordered;
    let dependencies = items
        .iter()
        .map(|item| item.work_item_id.clone())
        .collect::<Vec<_>>();
    items.push(BuildWorkItem {
        work_item_id: "work-integration".into(),
        entity_ref: envelope.root.id.clone(),
        label: format!("Verify {} integration", envelope.root.label),
        element_type: "integration-verification".into(),
        phase: 3,
        depends_on: dependencies,
        context_refs: global_context,
        generated_contract: false,
    });
    items
}

#[derive(Default)]
pub struct WorkflowStore {
    root: PathBuf,
    project_id: String,
    proposals: BTreeMap<String, ProposalRecord>,
    interviews: BTreeMap<String, InterviewRecord>,
    build: Option<BuildRequest>,
    builds: BTreeMap<String, BuildPlanRecord>,
}

impl WorkflowStore {
    pub fn open(project: &Path) -> Result<Self> {
        let root = workflow_root(project);
        fs::create_dir_all(root.join("proposals"))?;
        fs::create_dir_all(root.join("interviews"))?;
        fs::create_dir_all(root.join("builds"))?;
        let mut store = Self {
            root,
            project_id: project_id(project)?,
            proposals: BTreeMap::new(),
            interviews: BTreeMap::new(),
            build: None,
            builds: BTreeMap::new(),
        };
        store.load_and_recover()?;
        Ok(store)
    }

    fn load_and_recover(&mut self) -> Result<()> {
        for entry in fs::read_dir(self.root.join("proposals"))? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            if let Some(mut proposal) = read_json::<ProposalRecord>(&path)? {
                validate_proposal(&proposal).map_err(Error::Message)?;
                if proposal.project_id != self.project_id {
                    return Err(Error::Message(
                        "workflow proposal belongs to another project".into(),
                    ));
                }
                if matches!(proposal.state, ProposalState::Applying) {
                    proposal.state = ProposalState::RolledBack;
                    proposal.state_revision += 1;
                    proposal.diagnostic = Some("daemon restarted during live application; durable success was not recorded".into());
                    proposal.updated_at_epoch_ms = now_epoch_ms();
                    atomic_json(&path, &proposal)?;
                }
                self.proposals
                    .insert(proposal.proposal_id.clone(), proposal);
            }
        }
        for entry in fs::read_dir(self.root.join("interviews"))? {
            let path = entry?.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            if let Some(interview) = read_json::<InterviewRecord>(&path)? {
                validate_interview(&interview).map_err(Error::Message)?;
                if interview.project_id != self.project_id {
                    return Err(Error::Message(
                        "workflow interview belongs to another project".into(),
                    ));
                }
                self.interviews
                    .insert(interview.interview_id.clone(), interview);
            }
        }
        self.build = read_json(&self.root.join("play.json"))?;
        for entry in fs::read_dir(self.root.join("builds"))? {
            let directory = entry?.path();
            if !directory.is_dir() {
                continue;
            }
            let (Some(manifest), Some(mut progress), Some(evidence)) = (
                read_json::<BuildPlanManifest>(&directory.join("manifest.json"))?,
                read_json::<BuildPlanProgress>(&directory.join("progress.json"))?,
                read_json::<BuildPlanEvidence>(&directory.join("evidence.json"))?,
            ) else {
                continue;
            };
            if let Some(claim) = progress.active_claim.take() {
                progress
                    .work_item_states
                    .insert(claim.work_item_id, BuildWorkItemState::Pending);
                progress.state = BuildPlanState::Ready;
                progress.progress_revision += 1;
                progress.updated_at_epoch_ms = now_epoch_ms();
                progress.diagnostic =
                    Some("recovered an interrupted work-item claim after daemon restart".into());
                atomic_json(&directory.join("progress.json"), &progress)?;
            }
            let record = BuildPlanRecord {
                manifest,
                progress,
                evidence,
            };
            validate_build_plan(&record, &self.project_id).map_err(Error::Message)?;
            self.builds.insert(record.manifest.plan_id.clone(), record);
        }
        if self.builds.is_empty()
            && let Some(legacy) = self.build.clone()
        {
            let migrated = self.migrate_legacy_build(&legacy)?;
            self.save_plan(migrated)?;
        }
        self.prune()?;
        Ok(())
    }

    fn migrate_legacy_build(&self, legacy: &BuildRequest) -> Result<BuildPlanRecord> {
        let exact = legacy.project_id == self.project_id
            && legacy.protocol_version == DAEMON_PROTOCOL_VERSION
            && !legacy.diagram_path.is_empty()
            && !Path::new(&legacy.diagram_path).is_absolute()
            && !legacy.diagram_path.contains("..")
            && legacy.source_revision.starts_with("sha256:")
            && legacy.graph_revision.starts_with("sha256:")
            && matches!(legacy.state, BuildState::Ready | BuildState::Building);
        let identity = format!(
            "{}:{}",
            legacy.diagram_path,
            legacy.selected_entity_ref.as_deref().unwrap_or("diagram")
        );
        let hash = format!("{:x}", Sha256::digest(identity.as_bytes()));
        let plan_id = format!("legacy-{}", &hash[..24]);
        let scope_ref = format!("urn:ssw:{}:legacy#{}", self.project_id, &hash[..16]);
        let work_item = BuildWorkItem {
            work_item_id: "work-legacy".into(),
            entity_ref: scope_ref.clone(),
            label: legacy
                .selected_entity_ref
                .clone()
                .unwrap_or_else(|| legacy.diagram_path.clone()),
            element_type: "legacy-play-selection".into(),
            phase: 1,
            depends_on: vec![],
            context_refs: vec![],
            generated_contract: false,
        };
        let state = if exact {
            BuildPlanState::Ready
        } else {
            BuildPlanState::Stale
        };
        Ok(BuildPlanRecord {
            manifest: BuildPlanManifest {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: plan_id.clone(),
                project_id: self.project_id.clone(),
                scope_ref,
                scope_label: work_item.label.clone(),
                version: SemanticVersion::new(1, 0, 0),
                scope_contract_revision: format!(
                    "sha256:{:x}",
                    Sha256::digest(serde_json::to_vec(legacy).unwrap_or_default())
                ),
                source_manifest_revision: legacy.source_revision.clone(),
                graph_revision: legacy.graph_revision.clone(),
                diagram_path: legacy.diagram_path.clone(),
                selected_entity_ref: legacy.selected_entity_ref.clone(),
                work_items: vec![work_item],
                context_refs: vec![],
                excluded_refs: vec![],
                contract_outputs: vec![],
                created_at_epoch_ms: legacy.updated_at_epoch_ms,
            },
            progress: BuildPlanProgress {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: plan_id.clone(),
                progress_revision: 1,
                state,
                work_item_states: BTreeMap::from([(
                    "work-legacy".into(),
                    if exact {
                        BuildWorkItemState::Pending
                    } else {
                        BuildWorkItemState::Stale
                    },
                )]),
                active_claim: None,
                diagnostic: (!exact).then(|| {
                    "legacy Play request could not be tied safely to the current published graph"
                        .into()
                }),
                updated_at_epoch_ms: now_epoch_ms(),
            },
            evidence: BuildPlanEvidence {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id,
                evidence_revision: 1,
                items: BTreeMap::new(),
                changed_paths: legacy
                    .changed_paths
                    .iter()
                    .filter(|path| {
                        !Path::new(path).is_absolute()
                            && !path.contains("..")
                            && !path.contains("://")
                    })
                    .cloned()
                    .collect(),
                checks: legacy.checks.clone(),
                generated_contracts: vec![],
                completion_graph_revision: None,
            },
        })
    }

    pub fn save_interview(&mut self, record: InterviewRecord) -> Result<()> {
        validate_interview(&record).map_err(Error::Message)?;
        if record.project_id != self.project_id {
            return Err(Error::Message(
                "interview belongs to another project".into(),
            ));
        }
        for prior in self.interviews.keys().cloned().collect::<Vec<_>>() {
            let path = self.root.join("interviews").join(format!("{prior}.json"));
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        self.interviews.clear();
        atomic_json(
            &self
                .root
                .join("interviews")
                .join(format!("{}.json", record.interview_id)),
            &record,
        )?;
        self.interviews.insert(record.interview_id.clone(), record);
        self.prune()
    }

    pub fn interview(&self, id: &str) -> Option<InterviewRecord> {
        self.interviews.get(id).cloned()
    }

    pub fn current_interview(&self) -> Option<InterviewRecord> {
        self.interviews.values().next_back().cloned()
    }

    pub fn current_proposal_summary(&self) -> Option<ProposalResumeSummary> {
        self.proposals
            .values()
            .find(|proposal| !proposal.state.terminal())
            .map(ProposalResumeSummary::from)
    }

    pub fn save_build(&mut self, request: BuildRequest) -> Result<BuildRequest> {
        if request.project_id != self.project_id
            || request.protocol_version != DAEMON_PROTOCOL_VERSION
        {
            return Err(Error::Message(
                "build request belongs to another project or protocol".into(),
            ));
        }
        atomic_json(&self.root.join("play.json"), &request)?;
        self.build = Some(request.clone());
        Ok(request)
    }

    pub fn build(&self) -> Option<BuildRequest> {
        self.build.clone()
    }

    pub fn projected_build(&self) -> Option<BuildRequest> {
        let request = self.build.clone()?;
        let Some(plan) = self.builds.get(&request.request_id) else {
            return Some(request);
        };
        Some(BuildRequest {
            request_id: plan.manifest.plan_id.clone(),
            diagram_path: plan.manifest.diagram_path.clone(),
            source_revision: plan.manifest.source_manifest_revision.clone(),
            graph_revision: plan.manifest.graph_revision.clone(),
            selected_entity_ref: plan.manifest.selected_entity_ref.clone(),
            request_revision: plan.progress.progress_revision,
            state: match plan.progress.state {
                BuildPlanState::Ready => BuildState::Ready,
                BuildPlanState::Building => BuildState::Building,
                BuildPlanState::Complete => BuildState::Complete,
                BuildPlanState::Failed => BuildState::Failed,
                BuildPlanState::Stale => BuildState::Stale,
            },
            changed_paths: plan.evidence.changed_paths.clone(),
            checks: plan.evidence.checks.clone(),
            diagnostic: plan.progress.diagnostic.clone().or_else(|| {
                Some(format!(
                    "{} version {}",
                    plan.manifest.scope_label, plan.manifest.version
                ))
            }),
            updated_at_epoch_ms: plan.progress.updated_at_epoch_ms,
            ..request
        })
    }

    pub fn update_build(&mut self, update: BuildUpdate) -> Result<BuildRequest> {
        let request = self
            .build
            .as_mut()
            .ok_or_else(|| Error::Message("no Play request is active".into()))?;
        if request.request_id != update.request_id
            || request.request_revision != update.expected_request_revision
        {
            return Err(Error::Message("Play request revision conflict".into()));
        }
        let allowed = matches!(
            (&request.state, &update.state),
            (BuildState::Ready, BuildState::Building)
                | (BuildState::Building, BuildState::Complete)
                | (BuildState::Building, BuildState::Failed)
        );
        if !allowed {
            return Err(Error::Message("invalid Play request transition".into()));
        }
        request.state = update.state;
        request.request_revision += 1;
        request.changed_paths = update.changed_paths;
        request.checks = update.checks;
        request.diagnostic = update.diagnostic;
        request.updated_at_epoch_ms = now_epoch_ms();
        let request = request.clone();
        atomic_json(&self.root.join("play.json"), &request)?;
        Ok(request)
    }

    fn plan_directory(&self, plan_id: &str) -> PathBuf {
        self.root.join("builds").join(plan_id)
    }

    fn version_decision(
        &self,
        scope_ref: &str,
        scope_contract_revision: &str,
        bump: Option<SemanticBump>,
        expected_prior_version: Option<&SemanticVersion>,
    ) -> Result<VersionDecision> {
        let mut prior = self
            .builds
            .values()
            .filter(|record| record.manifest.scope_ref == scope_ref)
            .collect::<Vec<_>>();
        prior.sort_by_key(|record| record.manifest.version.components().unwrap_or((0, 0, 0)));
        let Some(latest) = prior.last() else {
            if bump.is_some() {
                return Err(Error::Message(
                    "a new scope starts at version 1.0.0 without a version bump".into(),
                ));
            }
            return Ok(VersionDecision::Create(SemanticVersion::new(1, 0, 0)));
        };
        if latest.manifest.scope_contract_revision == scope_contract_revision
            && matches!(
                latest.progress.state,
                BuildPlanState::Ready | BuildPlanState::Building | BuildPlanState::Failed
            )
        {
            if bump.is_some() {
                return Err(Error::Message(
                    "the unchanged incomplete scope must resume its existing version".into(),
                ));
            }
            return Ok(VersionDecision::Resume(Box::new((*latest).clone())));
        }
        let bump = bump.ok_or_else(|| {
            Error::Message(
                format!(
                    "this scope has a prior version; choose major, minor, or fix based on latest version {}",
                    latest.manifest.version
                ),
            )
        })?;
        if expected_prior_version != Some(&latest.manifest.version) {
            return Err(Error::Message(format!(
                "semantic version base is stale; latest version is {}",
                latest.manifest.version
            )));
        }
        Ok(VersionDecision::Create(
            latest
                .manifest
                .version
                .bumped(bump)
                .ok_or_else(|| Error::Message("semantic version overflow".into()))?,
        ))
    }

    pub fn save_plan(&mut self, record: BuildPlanRecord) -> Result<BuildPlanRecord> {
        validate_build_plan(&record, &self.project_id).map_err(Error::Message)?;
        if let Some(existing) = self.builds.get(&record.manifest.plan_id) {
            if existing.manifest != record.manifest {
                return Err(Error::Message("build-plan manifests are immutable".into()));
            }
            return Ok(existing.clone());
        }
        if self.builds.values().any(|existing| {
            existing.manifest.scope_ref == record.manifest.scope_ref
                && existing.manifest.version == record.manifest.version
        }) {
            return Err(Error::Message(
                "build-plan version already exists for this scope".into(),
            ));
        }
        let directory = self.plan_directory(&record.manifest.plan_id);
        fs::create_dir_all(&directory)?;
        atomic_json(&directory.join("manifest.json"), &record.manifest)?;
        atomic_json(&directory.join("progress.json"), &record.progress)?;
        atomic_json(&directory.join("evidence.json"), &record.evidence)?;
        atomic_json(
            &self.root.join("current-build.json"),
            &BuildPlanSummary::from(&record),
        )?;
        self.builds
            .insert(record.manifest.plan_id.clone(), record.clone());
        self.prune()?;
        Ok(record)
    }

    pub fn plans(&self) -> Vec<BuildPlanSummary> {
        let mut plans = self
            .builds
            .values()
            .map(BuildPlanSummary::from)
            .collect::<Vec<_>>();
        plans.sort_by(|left, right| {
            left.state
                .terminal()
                .cmp(&right.state.terminal())
                .then_with(|| right.updated_at_epoch_ms.cmp(&left.updated_at_epoch_ms))
                .then_with(|| left.plan_id.cmp(&right.plan_id))
        });
        plans
    }

    pub fn plan(&self, selector: &str) -> Result<BuildPlanRecord> {
        if let Some(record) = self.builds.get(selector) {
            return Ok(record.clone());
        }
        let Some((scope, version)) = selector.rsplit_once('@') else {
            return Err(Error::Message(
                "build plan not found; use plan ID or scope@version".into(),
            ));
        };
        let matches = self
            .builds
            .values()
            .filter(|record| {
                record.manifest.scope_ref == scope && record.manifest.version.0 == version
            })
            .cloned()
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [record] => Ok(record.clone()),
            [] => Err(Error::Message("build plan not found".into())),
            _ => Err(Error::Message("build plan selector is ambiguous".into())),
        }
    }

    fn persist_mutable_plan(&self, record: &BuildPlanRecord) -> Result<()> {
        let directory = self.plan_directory(&record.manifest.plan_id);
        atomic_json(&directory.join("progress.json"), &record.progress)?;
        atomic_json(&directory.join("evidence.json"), &record.evidence)?;
        atomic_json(
            &self.root.join("current-build.json"),
            &BuildPlanSummary::from(record),
        )
    }

    pub fn claim_next(&mut self, plan_id: &str, expected_revision: u64) -> Result<BuildPlanRecord> {
        let mut record = self
            .builds
            .get(plan_id)
            .cloned()
            .ok_or_else(|| Error::Message("build plan not found".into()))?;
        if record.progress.progress_revision != expected_revision {
            return Err(Error::Message(
                "build plan progress revision conflict".into(),
            ));
        }
        if record.progress.active_claim.is_some() {
            return Err(Error::Message("a work item is already claimed".into()));
        }
        if record.progress.state.terminal() && record.progress.state != BuildPlanState::Failed {
            return Err(Error::Message("build plan is terminal".into()));
        }
        let completed = record
            .progress
            .work_item_states
            .iter()
            .filter_map(|(id, state)| {
                (*state == BuildWorkItemState::Complete).then_some(id.as_str())
            })
            .collect::<std::collections::BTreeSet<_>>();
        let item = record
            .manifest
            .work_items
            .iter()
            .filter(|item| {
                matches!(
                    record.progress.work_item_states.get(&item.work_item_id),
                    Some(BuildWorkItemState::Pending | BuildWorkItemState::Failed)
                ) && item
                    .depends_on
                    .iter()
                    .all(|dependency| completed.contains(dependency.as_str()))
            })
            .min_by_key(|item| (item.phase, item.work_item_id.clone()))
            .ok_or_else(|| Error::Message("no dependency-ready work item is available".into()))?;
        let claim = BuildClaim {
            work_item_id: item.work_item_id.clone(),
            claim_token: random_hex(24),
            claimed_at_epoch_ms: now_epoch_ms(),
        };
        record
            .progress
            .work_item_states
            .insert(item.work_item_id.clone(), BuildWorkItemState::Active);
        record.progress.active_claim = Some(claim);
        record.progress.state = BuildPlanState::Building;
        record.progress.progress_revision += 1;
        record.progress.diagnostic = None;
        record.progress.updated_at_epoch_ms = now_epoch_ms();
        self.persist_mutable_plan(&record)?;
        self.builds.insert(plan_id.into(), record.clone());
        Ok(record)
    }

    pub fn record_result(
        &mut self,
        completion: crate::delivery_protocol::BuildPlanCompletion,
        success: bool,
    ) -> Result<BuildPlanRecord> {
        let mut record = self
            .builds
            .get(&completion.plan_id)
            .cloned()
            .ok_or_else(|| Error::Message("build plan not found".into()))?;
        if record.progress.progress_revision != completion.expected_progress_revision {
            return Err(Error::Message(
                "build plan progress revision conflict".into(),
            ));
        }
        let claim = record
            .progress
            .active_claim
            .as_ref()
            .ok_or_else(|| Error::Message("build plan has no active claim".into()))?;
        if claim.claim_token != completion.claim_token
            || claim.work_item_id != completion.evidence.work_item_id
        {
            return Err(Error::Message("work-item claim token mismatch".into()));
        }
        if completion.evidence.physical_effects.iter().any(|effect| {
            let normalized = effect.to_ascii_lowercase();
            effect.len() > 2_048
                || normalized.contains("out-of-scope")
                || normalized.contains("unrelated feature")
                || normalized.contains("alternate flow")
                || normalized.contains("duplicate service")
                || normalized.contains("stub service")
        }) {
            return Err(Error::Message(
                "physical-effect evidence claims work outside the selected scope".into(),
            ));
        }
        let next = if success {
            BuildWorkItemState::Complete
        } else {
            BuildWorkItemState::Failed
        };
        record
            .progress
            .work_item_states
            .insert(claim.work_item_id.clone(), next);
        record.progress.active_claim = None;
        record.progress.progress_revision += 1;
        record.progress.state = if success {
            BuildPlanState::Ready
        } else {
            BuildPlanState::Failed
        };
        record.progress.diagnostic = completion.evidence.diagnostic.clone();
        record.progress.updated_at_epoch_ms = now_epoch_ms();
        record.evidence.evidence_revision += 1;
        record
            .evidence
            .changed_paths
            .extend(completion.evidence.changed_paths.iter().cloned());
        record.evidence.changed_paths.sort();
        record.evidence.changed_paths.dedup();
        record.evidence.items.insert(
            completion.evidence.work_item_id.clone(),
            completion.evidence,
        );
        validate_build_plan(&record, &self.project_id).map_err(Error::Message)?;
        self.persist_mutable_plan(&record)?;
        self.builds
            .insert(record.manifest.plan_id.clone(), record.clone());
        Ok(record)
    }

    pub fn complete_plan(
        &mut self,
        plan_id: &str,
        expected_revision: u64,
        graph_revision: String,
        checks: BTreeMap<String, String>,
    ) -> Result<BuildPlanRecord> {
        let mut record = self
            .builds
            .get(plan_id)
            .cloned()
            .ok_or_else(|| Error::Message("build plan not found".into()))?;
        if record.progress.progress_revision != expected_revision
            || record.progress.active_claim.is_some()
            || record
                .progress
                .work_item_states
                .values()
                .any(|state| *state != BuildWorkItemState::Complete)
            || record.manifest.contract_outputs.iter().any(|output| {
                !record
                    .evidence
                    .generated_contracts
                    .contains(&output.document_path)
            })
        {
            return Err(Error::Message(
                "build plan cannot complete at this progress revision".into(),
            ));
        }
        record.progress.progress_revision += 1;
        record.progress.state = BuildPlanState::Complete;
        record.progress.updated_at_epoch_ms = now_epoch_ms();
        record.evidence.evidence_revision += 1;
        record.evidence.checks = checks;
        record.evidence.completion_graph_revision = Some(graph_revision);
        self.persist_mutable_plan(&record)?;
        self.builds.insert(plan_id.into(), record.clone());
        self.prune()?;
        Ok(record)
    }

    pub fn record_contract_publication(
        &mut self,
        plan_id: &str,
        expected_progress_revision: u64,
        path: String,
        graph_revision: String,
    ) -> Result<BuildPlanRecord> {
        let mut record = self
            .builds
            .get(plan_id)
            .cloned()
            .ok_or_else(|| Error::Message("build plan not found".into()))?;
        if record.progress.progress_revision != expected_progress_revision {
            return Err(Error::Message(
                "build plan progress revision conflict".into(),
            ));
        }
        record.evidence.evidence_revision += 1;
        record.evidence.generated_contracts.push(path);
        record.evidence.generated_contracts.sort();
        record.evidence.generated_contracts.dedup();
        record.evidence.completion_graph_revision = Some(graph_revision);
        self.persist_mutable_plan(&record)?;
        self.builds.insert(plan_id.into(), record.clone());
        Ok(record)
    }

    pub fn mark_plan_stale(&mut self, plan_id: &str, diagnostic: &str) -> Result<()> {
        let Some(mut record) = self.builds.get(plan_id).cloned() else {
            return Ok(());
        };
        if matches!(
            record.progress.state,
            BuildPlanState::Complete | BuildPlanState::Stale
        ) {
            return Ok(());
        }
        record.progress.state = BuildPlanState::Stale;
        record.progress.active_claim = None;
        record.progress.progress_revision += 1;
        record.progress.updated_at_epoch_ms = now_epoch_ms();
        record.progress.diagnostic = Some(diagnostic.into());
        self.persist_mutable_plan(&record)?;
        self.builds.insert(plan_id.into(), record);
        Ok(())
    }

    pub fn submit(&mut self, record: ProposalRecord) -> Result<ProposalRecord> {
        validate_proposal(&record).map_err(Error::Message)?;
        if record.project_id != self.project_id || record.state != ProposalState::AwaitingReview {
            return Err(Error::Message(
                "proposal project or initial state is invalid".into(),
            ));
        }
        if self
            .proposals
            .values()
            .any(|proposal| !proposal.state.terminal())
        {
            return Err(Error::Message(
                "finish or cancel the current proposal before submitting another".into(),
            ));
        }
        for prior in self.proposals.keys().cloned().collect::<Vec<_>>() {
            let path = self.proposal_path(&prior);
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        self.proposals.clear();
        atomic_json(&self.proposal_path(&record.proposal_id), &record)?;
        self.proposals
            .insert(record.proposal_id.clone(), record.clone());
        Ok(record)
    }

    pub fn proposal(&self, id: &str) -> Option<ProposalRecord> {
        self.proposals.get(id).cloned()
    }

    pub fn proposals(&self) -> Vec<ProposalRecord> {
        self.proposals.values().cloned().collect()
    }

    pub fn transition(&mut self, decision: &ProposalDecision) -> Result<ProposalRecord> {
        let proposal = self
            .proposals
            .get_mut(&decision.proposal_id)
            .ok_or_else(|| Error::Message("proposal not found".into()))?;
        if decision.protocol_version != DAEMON_PROTOCOL_VERSION
            || proposal.state_revision != decision.expected_state_revision
        {
            return Err(Error::Message("proposal state revision conflict".into()));
        }
        let next = match (&proposal.state, &decision.kind) {
            (ProposalState::AwaitingReview, ProposalDecisionKind::Approve) => {
                ProposalState::Approved
            }
            (ProposalState::AwaitingReview, ProposalDecisionKind::Reject) => {
                ProposalState::Rejected
            }
            (ProposalState::AwaitingReview, ProposalDecisionKind::Cancel) => {
                ProposalState::Cancelled
            }
            (
                ProposalState::AwaitingReview | ProposalState::Approved,
                ProposalDecisionKind::Conflict,
            ) => ProposalState::Conflicted,
            (ProposalState::Approved, ProposalDecisionKind::Applied) => ProposalState::Applying,
            (ProposalState::Applying, ProposalDecisionKind::Applied) => ProposalState::Persisted,
            (ProposalState::Persisted, ProposalDecisionKind::Applied) => ProposalState::Publishing,
            (ProposalState::Publishing, ProposalDecisionKind::Applied) => ProposalState::Published,
            (
                ProposalState::Approved
                | ProposalState::Applying
                | ProposalState::Persisted
                | ProposalState::Publishing,
                ProposalDecisionKind::Failed,
            ) => ProposalState::Failed,
            (ProposalState::Published, ProposalDecisionKind::Reverted) => ProposalState::RolledBack,
            _ => return Err(Error::Message("illegal proposal state transition".into())),
        };
        proposal.state = next;
        proposal.state_revision += 1;
        proposal.diagnostic = decision.diagnostic.clone();
        proposal.updated_at_epoch_ms = now_epoch_ms();
        let record = proposal.clone();
        atomic_json(
            &self
                .root
                .join("proposals")
                .join(format!("{}.json", record.proposal_id)),
            &record,
        )?;
        Ok(record)
    }

    pub fn cancel(&mut self, id: &str, expected: u64) -> Result<ProposalRecord> {
        self.transition(&ProposalDecision {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            proposal_id: id.into(),
            expected_state_revision: expected,
            session_id: None,
            kind: ProposalDecisionKind::Cancel,
            diagnostic: None,
        })
    }

    pub fn attach_receipt(
        &mut self,
        id: &str,
        receipt: crate::delivery_protocol::PublicationReceipt,
    ) -> Result<ProposalRecord> {
        let proposal = self
            .proposals
            .get_mut(id)
            .ok_or_else(|| Error::Message("proposal not found".into()))?;
        if proposal.state != ProposalState::Published {
            return Err(Error::Message(
                "receipt requires a published proposal".into(),
            ));
        }
        proposal.receipt = Some(receipt);
        proposal.updated_at_epoch_ms = now_epoch_ms();
        let record = proposal.clone();
        atomic_json(&self.proposal_path(id), &record)?;
        Ok(record)
    }

    fn proposal_path(&self, id: &str) -> PathBuf {
        self.root.join("proposals").join(format!("{id}.json"))
    }

    fn prune(&mut self) -> Result<()> {
        let mut terminal = self
            .builds
            .values()
            .filter(|record| record.progress.state.terminal())
            .map(|record| {
                (
                    record.progress.updated_at_epoch_ms,
                    record.manifest.plan_id.clone(),
                )
            })
            .collect::<Vec<_>>();
        terminal.sort_by(|left, right| right.cmp(left));
        for (_, plan_id) in terminal.into_iter().skip(50) {
            let directory = self.plan_directory(&plan_id);
            if directory.is_dir() {
                fs::remove_dir_all(directory)?;
            }
            self.builds.remove(&plan_id);
        }
        Ok(())
    }
}

#[derive(Default)]
struct EventState {
    next: u64,
    retained: VecDeque<RuntimeEvent>,
}

#[derive(Clone)]
pub struct ProjectRuntime {
    pub project: Arc<PathBuf>,
    pub project_id: Arc<String>,
    pub generation: Arc<String>,
    pub token: Arc<String>,
    pub app: AppState,
    pub graph: SchematicMcp,
    workflows: Arc<Mutex<WorkflowStore>>,
    session: Arc<RwLock<Option<BrowserSession>>>,
    lease: Arc<Mutex<Option<ApplicationLease>>>,
    events: Arc<Mutex<EventState>>,
    last_activity: Arc<AtomicU64>,
    browser_base_url: Arc<OnceLock<String>>,
    browser_launcher: Arc<dyn BrowserLauncher>,
}

impl ProjectRuntime {
    pub fn load(project: impl AsRef<Path>, options: LoadOptions) -> Result<Self> {
        Self::load_with_browser_launcher(project, options, Arc::new(SystemBrowserLauncher))
    }

    pub fn load_with_browser_launcher(
        project: impl AsRef<Path>,
        options: LoadOptions,
        browser_launcher: Arc<dyn BrowserLauncher>,
    ) -> Result<Self> {
        let project = project.as_ref().canonicalize()?;
        let project_id = project_id(&project)?;
        let graph = SchematicMcp::load(&project, options)?;
        let app = AppState::new(&project)?.with_graph(graph.clone());
        let workflows = WorkflowStore::open(&project)?;
        Ok(Self {
            project: Arc::new(project),
            project_id: Arc::new(project_id),
            generation: Arc::new(random_hex(16)),
            token: Arc::new(random_hex(32)),
            app,
            graph,
            workflows: Arc::new(Mutex::new(workflows)),
            session: Arc::new(RwLock::new(None)),
            lease: Arc::new(Mutex::new(None)),
            events: Arc::new(Mutex::new(EventState::default())),
            last_activity: Arc::new(AtomicU64::new(now_epoch_ms() as u64)),
            browser_base_url: Arc::new(OnceLock::new()),
            browser_launcher,
        })
    }

    pub fn set_browser_base_url(&self, value: String) -> Result<()> {
        let valid = value.starts_with("http://127.0.0.1:")
            || value.starts_with("http://localhost:")
            || value.starts_with("http://[::1]:");
        if !valid || value.contains('?') || value.contains('#') {
            return Err(Error::Message(
                "browser base URL must be a credential-free loopback HTTP origin".into(),
            ));
        }
        self.browser_base_url
            .set(value.trim_end_matches('/').to_string())
            .map_err(|_| Error::Message("browser base URL is already configured".into()))
    }

    fn authenticated_browser_url(&self) -> Result<String> {
        let base = self
            .browser_base_url
            .get()
            .ok_or_else(|| Error::Message("browser workspace is not ready".into()))?;
        Ok(format!(
            "{base}/?daemonToken={}&projectId={}&generation={}",
            self.token, self.project_id, self.generation
        ))
    }

    pub fn request_browser_open(&self) -> Result<()> {
        let target = self.authenticated_browser_url()?;
        self.browser_launcher.open(&target).map_err(Error::Message)
    }

    pub fn authenticate(&self, token: &str, project_id: &str, generation: &str) -> Result<()> {
        if token != self.token.as_str()
            || project_id != self.project_id.as_str()
            || generation != self.generation.as_str()
        {
            return Err(Error::Message("daemon authentication rejected".into()));
        }
        self.touch();
        Ok(())
    }

    pub async fn register_session(&self, session: BrowserSession) -> Result<()> {
        validate_browser_session(&session).map_err(Error::Message)?;
        if session.daemon_generation != self.generation.as_str() {
            return Err(Error::Message(
                "browser session belongs to a stale daemon generation".into(),
            ));
        }
        *self.session.write().await = Some(session);
        self.touch();
        Ok(())
    }

    pub async fn expire_sessions(&self, now: u128) -> usize {
        let mut session = self.session.write().await;
        if session.as_ref().is_some_and(|session| {
            now.saturating_sub(session.heartbeat_epoch_ms) > SESSION_TIMEOUT_MS
        }) {
            *session = None;
            1
        } else {
            0
        }
    }

    async fn workspace_snapshot(
        &self,
        timeout: std::time::Duration,
    ) -> Result<RuntimeWorkspaceSnapshot> {
        self.expire_sessions(now_epoch_ms()).await;
        if let Some(session) = self.session.read().await.clone() {
            return self
                .finish_workspace_snapshot(
                    WorkspaceLaunchState::AlreadyConnected,
                    Some(session),
                    None,
                )
                .await;
        }

        if self.browser_base_url.get().is_none() {
            return self
                .finish_workspace_snapshot(
                    WorkspaceLaunchState::Failed,
                    None,
                    Some("The browser workspace is not ready. Retry, then run ./ssw (macOS) or ssw.cmd (Windows) if needed.".into()),
                )
                .await;
        }
        if self.request_browser_open().is_err() {
            return self
                .finish_workspace_snapshot(
                    WorkspaceLaunchState::Failed,
                    None,
                    Some("The system browser could not be opened. Run ./ssw (macOS) or ssw.cmd (Windows) in this project.".into()),
                )
                .await;
        }

        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(session) = self.session.read().await.clone() {
                return self
                    .finish_workspace_snapshot(
                        WorkspaceLaunchState::OpenedAndConnected,
                        Some(session),
                        None,
                    )
                    .await;
            }
            if tokio::time::Instant::now() >= deadline {
                return self
                    .finish_workspace_snapshot(
                        WorkspaceLaunchState::OpenRequested,
                        None,
                        Some("The browser was asked to open. If it does not appear, run ./ssw (macOS) or ssw.cmd (Windows) in this project.".into()),
                    )
                    .await;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    async fn finish_workspace_snapshot(
        &self,
        launch_state: WorkspaceLaunchState,
        session: Option<BrowserSession>,
        diagnostic: Option<String>,
    ) -> Result<RuntimeWorkspaceSnapshot> {
        let workflows = self.workflows.lock().await;
        let snapshot = RuntimeWorkspaceSnapshot {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            project_id: self.project_id.to_string(),
            launch_state,
            active_diagram: session
                .as_ref()
                .and_then(|value| value.active_diagram.clone()),
            selected_entity_refs: session
                .map(|value| value.selected_entity_refs)
                .unwrap_or_default(),
            interview: workflows.current_interview(),
            proposal: workflows.current_proposal_summary(),
            diagnostic,
        };
        validate_runtime_workspace(&snapshot).map_err(Error::Message)?;
        self.touch();
        Ok(snapshot)
    }

    pub async fn open_design_workspace(&self) -> Result<RuntimeWorkspaceSnapshot> {
        self.workspace_snapshot(WORKSPACE_READY_TIMEOUT).await
    }

    pub async fn has_dirty_path(&self, path: &str) -> bool {
        self.session
            .read()
            .await
            .as_ref()
            .is_some_and(|session| session.dirty_documents.iter().any(|value| value == path))
    }

    pub async fn emit(
        &self,
        kind: &str,
        proposal: Option<&ProposalRecord>,
        message: Option<String>,
    ) -> RuntimeEvent {
        let mut state = self.events.lock().await;
        state.next += 1;
        let event = RuntimeEvent {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            event_id: state.next,
            daemon_generation: self.generation.to_string(),
            kind: kind.into(),
            proposal_id: proposal.map(|value| value.proposal_id.clone()),
            state_revision: proposal.map(|value| value.state_revision),
            diagram_path: proposal.map(|value| value.diagram_path.clone()),
            message,
        };
        state.retained.push_back(event.clone());
        while state.retained.len() > MAX_EVENTS {
            state.retained.pop_front();
        }
        self.touch();
        event
    }

    pub async fn events_after(&self, event_id: u64) -> Vec<RuntimeEvent> {
        self.events
            .lock()
            .await
            .retained
            .iter()
            .filter(|event| event.event_id > event_id)
            .cloned()
            .collect()
    }

    pub async fn submit_proposal(&self, proposal: ProposalRecord) -> Result<ProposalRecord> {
        if proposal.operation_registry_version != "2.0" {
            return Err(Error::Message(
                "unsupported diagram operation registry version".into(),
            ));
        }
        if !proposal
            .source_revisions
            .contains_key(&proposal.diagram_path)
        {
            return Err(Error::Message(
                "proposal must bind its target diagram revision".into(),
            ));
        }
        for path in proposal.source_revisions.keys() {
            if self.has_dirty_path(path).await {
                return Err(Error::Message(format!(
                    "proposal target is dirty in a browser session: {path}"
                )));
            }
            let confined = self.app.resolve(path, false)?;
            let bytes = fs::read(&confined)?;
            let authored = if path.ends_with(".md") {
                crate::embedding_document::parse_markdown(
                    std::str::from_utf8(&bytes)
                        .map_err(|_| Error::Message("proposal Markdown must be UTF-8".into()))?,
                )
                .body
                .into_bytes()
            } else {
                bytes
            };
            let current = format!("sha256:{:x}", Sha256::digest(authored));
            if proposal.source_revisions.get(path) != Some(&current) {
                return Err(Error::Message(format!(
                    "proposal source revision is stale for {path}"
                )));
            }
        }
        for operation in &proposal.operations {
            let object = operation
                .as_object()
                .ok_or_else(|| Error::Message("proposal operations must be objects".into()))?;
            if object.contains_key("xml")
                || object.contains_key("rawXml")
                || object.contains_key("path")
                || object.contains_key("command")
            {
                return Err(Error::Message(
                    "proposal contains an unrestricted or executable field".into(),
                ));
            }
        }
        let proposal = self.workflows.lock().await.submit(proposal)?;
        self.emit("proposalSubmitted", Some(&proposal), None).await;
        Ok(proposal)
    }

    pub async fn proposal(&self, id: &str) -> Option<ProposalRecord> {
        self.workflows.lock().await.proposal(id)
    }
    pub async fn save_interview(&self, record: InterviewRecord) -> Result<InterviewRecord> {
        self.workflows.lock().await.save_interview(record.clone())?;
        self.emit("interviewUpdated", None, Some(record.interview_id.clone()))
            .await;
        Ok(record)
    }
    pub async fn proposals(&self) -> Vec<ProposalRecord> {
        self.workflows.lock().await.proposals()
    }

    pub async fn decide(&self, decision: ProposalDecision) -> Result<ProposalRecord> {
        if matches!(decision.kind, ProposalDecisionKind::Approve) && decision.session_id.is_none() {
            return Err(Error::Message(
                "only a browser session may approve a proposal".into(),
            ));
        }
        if let Some(session) = &decision.session_id
            && self
                .session
                .read()
                .await
                .as_ref()
                .map(|active| &active.session_id)
                != Some(session)
        {
            return Err(Error::Message(
                "proposal decision browser session is not active".into(),
            ));
        }
        if matches!(
            decision.kind,
            ProposalDecisionKind::Applied
                | ProposalDecisionKind::Failed
                | ProposalDecisionKind::Reverted
        ) {
            let lease_guard = self.lease.lock().await;
            let lease = lease_guard
                .as_ref()
                .filter(|lease| lease.proposal_id == decision.proposal_id)
                .ok_or_else(|| {
                    Error::Message("proposal has no active browser application lease".into())
                })?;
            if decision.session_id.as_deref() != Some(&lease.session_id)
                || lease.daemon_generation != self.generation.as_str()
                || lease.expires_at_epoch_ms < now_epoch_ms()
            {
                return Err(Error::Message(
                    "proposal application lease is stale or belongs to another browser".into(),
                ));
            }
        }
        let mut proposal = self.workflows.lock().await.transition(&decision)?;
        if matches!(decision.kind, ProposalDecisionKind::Approve) {
            *self.lease.lock().await = Some(ApplicationLease {
                protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                proposal_id: proposal.proposal_id.clone(),
                session_id: decision
                    .session_id
                    .clone()
                    .expect("approval session validated"),
                daemon_generation: self.generation.to_string(),
                proposal_revision: proposal.state_revision,
                expires_at_epoch_ms: now_epoch_ms() + 60_000,
            });
        } else if proposal.state.terminal() {
            *self.lease.lock().await = None;
        }
        if proposal.state == ProposalState::Published
            && let Some(receipt) = self.graph.refresh_status_public().await.receipt
        {
            proposal = self
                .workflows
                .lock()
                .await
                .attach_receipt(&proposal.proposal_id, receipt)?;
        }
        self.emit(
            "proposalStateChanged",
            Some(&proposal),
            proposal.diagnostic.clone(),
        )
        .await;
        Ok(proposal)
    }

    pub async fn cancel_proposal(&self, id: &str, expected: u64) -> Result<ProposalRecord> {
        let proposal = self.workflows.lock().await.cancel(id, expected)?;
        self.emit("proposalStateChanged", Some(&proposal), None)
            .await;
        Ok(proposal)
    }

    pub async fn create_build_plan(
        &self,
        request: CreateBuildPlanRequest,
    ) -> Result<BuildPlanRecord> {
        if request.protocol_version != BUILD_PLAN_PROTOCOL_VERSION
            || request.project_id != self.project_id.as_str()
        {
            return Err(Error::Message(
                "build-plan request belongs to another project or protocol".into(),
            ));
        }
        if self
            .session
            .read()
            .await
            .as_ref()
            .is_some_and(|session| !session.dirty_documents.is_empty())
        {
            return Err(Error::Message(
                "save every open diagram and document before creating a build plan".into(),
            ));
        }
        let summary = self.graph.summary().await;
        if summary.revision != request.expected_graph_revision
            || summary.source_manifest_revision != request.expected_source_manifest_revision
        {
            return Err(Error::Message(
                "published graph or source manifest changed; resolve scope again".into(),
            ));
        }
        let root = self
            .graph
            .resolve_plan_selection(
                &request.diagram_path,
                request.selected_entity_ref.as_deref(),
            )
            .await?;
        self.create_build_plan_for_scope(request, root.id).await
    }

    pub async fn create_build_plan_for_scope(
        &self,
        request: CreateBuildPlanRequest,
        scope_ref: String,
    ) -> Result<BuildPlanRecord> {
        if self
            .session
            .read()
            .await
            .as_ref()
            .is_some_and(|session| !session.dirty_documents.is_empty())
        {
            return Err(Error::Message(
                "save every open diagram and document before creating a build plan".into(),
            ));
        }
        let summary = self.graph.summary().await;
        if request.protocol_version != BUILD_PLAN_PROTOCOL_VERSION
            || request.project_id != self.project_id.as_str()
            || summary.revision != request.expected_graph_revision
            || summary.source_manifest_revision != request.expected_source_manifest_revision
        {
            return Err(Error::Message(
                "build-plan authority is stale or belongs to another project".into(),
            ));
        }
        let envelope = self.graph.plan_scope(&scope_ref).await?;
        let mut workflows = self.workflows.lock().await;
        let version = match workflows.version_decision(
            &scope_ref,
            &envelope.scope_contract_revision,
            request.semantic_bump,
            request.expected_prior_version.as_ref(),
        )? {
            VersionDecision::Resume(record) => return Ok(*record),
            VersionDecision::Create(version) => version,
        };
        let plan_id = format!("plan-{}", random_hex(12));
        let work_items = build_work_items(&envelope);
        let work_item_states = work_items
            .iter()
            .map(|item| (item.work_item_id.clone(), BuildWorkItemState::Pending))
            .collect();
        let now = now_epoch_ms();
        let record = BuildPlanRecord {
            manifest: BuildPlanManifest {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: plan_id.clone(),
                project_id: self.project_id.to_string(),
                scope_ref: envelope.root.id.clone(),
                scope_label: if envelope.root.label.trim().is_empty() {
                    envelope.root.owner_name.clone()
                } else {
                    envelope.root.label.clone()
                },
                version,
                scope_contract_revision: envelope.scope_contract_revision,
                source_manifest_revision: envelope.source_manifest_revision,
                graph_revision: envelope.revision,
                diagram_path: request.diagram_path.clone(),
                selected_entity_ref: request.selected_entity_ref.clone(),
                work_items,
                context_refs: envelope
                    .context_only
                    .iter()
                    .map(|entity| entity.id.clone())
                    .collect(),
                excluded_refs: envelope
                    .excluded
                    .iter()
                    .map(|entity| entity.id.clone())
                    .collect(),
                contract_outputs: envelope.contract_outputs,
                created_at_epoch_ms: now,
            },
            progress: BuildPlanProgress {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: plan_id.clone(),
                progress_revision: 1,
                state: BuildPlanState::Ready,
                work_item_states,
                active_claim: None,
                diagnostic: None,
                updated_at_epoch_ms: now,
            },
            evidence: BuildPlanEvidence {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: plan_id.clone(),
                evidence_revision: 1,
                items: BTreeMap::new(),
                changed_paths: vec![],
                checks: BTreeMap::new(),
                generated_contracts: vec![],
                completion_graph_revision: None,
            },
        };
        let record = workflows.save_plan(record)?;
        let source_revision = summary.source_manifest_revision.clone();
        let legacy = BuildRequest {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            request_id: record.manifest.plan_id.clone(),
            project_id: self.project_id.to_string(),
            diagram_path: record.manifest.diagram_path.clone(),
            source_revision,
            graph_revision: record.manifest.graph_revision.clone(),
            selected_entity_ref: record.manifest.selected_entity_ref.clone(),
            request_revision: record.progress.progress_revision,
            state: BuildState::Ready,
            changed_paths: vec![],
            checks: BTreeMap::new(),
            diagnostic: Some(format!(
                "{} version {}",
                record.manifest.scope_label, record.manifest.version
            )),
            updated_at_epoch_ms: now,
        };
        workflows.save_build(legacy)?;
        drop(workflows);
        self.emit(
            "buildPlanReady",
            None,
            Some(record.manifest.plan_id.clone()),
        )
        .await;
        Ok(record)
    }

    pub async fn build_plans(&self) -> Vec<BuildPlanSummary> {
        self.workflows.lock().await.plans()
    }

    pub async fn build_plan(&self, selector: &str) -> Result<BuildPlanRecord> {
        self.workflows.lock().await.plan(selector)
    }

    async fn validate_plan_scope_authority(
        &self,
        record: &BuildPlanRecord,
        action: &str,
    ) -> Result<crate::schematic_graph::PlanScopeEnvelope> {
        let current_scope = self.graph.plan_scope(&record.manifest.scope_ref).await;
        if let Ok(scope) = &current_scope
            && scope.scope_contract_revision == record.manifest.scope_contract_revision
        {
            return current_scope;
        }
        self.workflows
            .lock()
            .await
            .mark_plan_stale(&record.manifest.plan_id, "selected build scope changed")?;
        Err(Error::Message(format!(
            "selected build scope changed; {action} authority is stale"
        )))
    }

    pub async fn claim_next_build_item(
        &self,
        plan_id: &str,
        expected_revision: u64,
    ) -> Result<BuildPlanRecord> {
        let record = self.workflows.lock().await.plan(plan_id)?;
        self.validate_plan_scope_authority(&record, "claim").await?;
        self.workflows
            .lock()
            .await
            .claim_next(plan_id, expected_revision)
    }

    pub async fn build_item_context(&self, plan_id: &str) -> Result<BuildWorkItemContext> {
        let record = self.workflows.lock().await.plan(plan_id)?;
        let claim = record.progress.active_claim.clone().ok_or_else(|| {
            Error::Message("claim the next work item before loading context".into())
        })?;
        let focused_item = record
            .manifest
            .work_items
            .iter()
            .find(|item| item.work_item_id == claim.work_item_id)
            .cloned()
            .ok_or_else(|| Error::Message("claimed work item is absent from manifest".into()))?;
        let current_scope = self
            .validate_plan_scope_authority(&record, "context")
            .await?;
        let all_refs = record
            .manifest
            .work_items
            .iter()
            .map(|item| item.entity_ref.clone())
            .chain(record.manifest.context_refs.iter().cloned())
            .chain(record.manifest.excluded_refs.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>();
        let (entities, relationships, citations) = self.graph.plan_context(&all_refs).await;
        Ok(BuildWorkItemContext {
            plan_id: record.manifest.plan_id,
            version: record.manifest.version,
            progress_revision: record.progress.progress_revision,
            claim,
            focused_item: focused_item.clone(),
            supporting_items: record
                .manifest
                .work_items
                .into_iter()
                .filter(|item| item.work_item_id != focused_item.work_item_id)
                .collect(),
            context_refs: record.manifest.context_refs,
            excluded_refs: record.manifest.excluded_refs,
            completed_evidence: record.evidence.items.into_values().collect(),
            entities: entities
                .into_iter()
                .map(|entity| BuildContextEntity {
                    entity_ref: entity.id,
                    label: entity.label,
                    element_type: entity.element_type,
                    implementation_status: entity
                        .implementation_status
                        .map(|status| status.as_str().into()),
                    logical_markdown: entity.markdown,
                    implementation_contract_markdown: entity.implementation_contract_markdown,
                })
                .collect(),
            relationships: relationships
                .into_iter()
                .map(|relation| BuildContextRelation {
                    relation_type: relation.relation_type,
                    source: relation.source,
                    target: relation.target,
                })
                .collect(),
            citations: citations
                .into_iter()
                .map(|(id, citation)| {
                    (
                        id,
                        BuildContextCitation {
                            diagram: citation.diagram,
                            document: citation.document,
                            source_id: citation.source_id,
                        },
                    )
                })
                .collect(),
            graph_revision: current_scope.revision,
        })
    }

    pub async fn record_build_item(
        &self,
        completion: crate::delivery_protocol::BuildPlanCompletion,
        success: bool,
    ) -> Result<BuildPlanRecord> {
        self.workflows
            .lock()
            .await
            .record_result(completion, success)
    }

    pub async fn publish_generated_contract(
        &self,
        submission: GeneratedContractSubmission,
    ) -> Result<crate::delivery_protocol::DocumentRevision> {
        validate_generated_contract_submission(&submission).map_err(Error::Message)?;
        let record = self.workflows.lock().await.plan(&submission.plan_id)?;
        if record.progress.progress_revision != submission.expected_progress_revision {
            return Err(Error::Message(
                "build plan progress revision conflict".into(),
            ));
        }
        let claim = record
            .progress
            .active_claim
            .as_ref()
            .ok_or_else(|| Error::Message("build plan has no active work-item claim".into()))?;
        if claim.claim_token != submission.claim_token {
            return Err(Error::Message("work-item claim token mismatch".into()));
        }
        let item = record
            .manifest
            .work_items
            .iter()
            .find(|item| item.work_item_id == claim.work_item_id)
            .ok_or_else(|| Error::Message("claimed work item is absent from manifest".into()))?;
        if !item.generated_contract || item.entity_ref != submission.entity_ref {
            return Err(Error::Message(
                "the active claim does not authorize an edge/event contract".into(),
            ));
        }
        let output = record
            .manifest
            .contract_outputs
            .iter()
            .find(|output| output.entity_ref == submission.entity_ref)
            .ok_or_else(|| {
                Error::Message("plan has no generated contract output for this item".into())
            })?;
        let allowed_refs = record
            .manifest
            .work_items
            .iter()
            .map(|item| item.entity_ref.as_str())
            .chain(record.manifest.context_refs.iter().map(String::as_str))
            .chain(record.manifest.excluded_refs.iter().map(String::as_str))
            .collect::<std::collections::BTreeSet<_>>();
        for token in submission.body.split_whitespace() {
            let reference = token.trim_matches(|character: char| {
                matches!(character, '`' | ',' | '.' | ')' | '(' | '[' | ']' | ';')
            });
            if reference.starts_with("urn:ssw:") && !allowed_refs.contains(reference) {
                return Err(Error::Message(
                    "generated contract references an entity outside the selected plan".into(),
                ));
            }
        }
        self.validate_plan_scope_authority(&record, "contract")
            .await?;
        let metadata = GeneratedContractMetadata {
            schema_version: GENERATED_CONTRACT_SCHEMA_VERSION.into(),
            project_id: self.project_id.to_string(),
            plan_id: record.manifest.plan_id.clone(),
            scope_ref: record.manifest.scope_ref.clone(),
            semantic_version: record.manifest.version.clone(),
            entity_ref: submission.entity_ref,
            base_graph_revision: record.manifest.graph_revision.clone(),
            body_hash: format!("{:x}", Sha256::digest(submission.body.as_bytes())),
        };
        let physical =
            serialize_generated_contract(&metadata, self.project_id.as_str(), &submission.body)
                .map_err(Error::Message)?;
        let receipt = self
            .app
            .write_generated_contract(
                &output.document_path,
                &physical,
                &submission.body,
                &submission.expected_document_revision,
            )
            .await?;
        for _ in 0..50 {
            if !self
                .app
                .generated_publication_pending(&output.document_path)
                .await
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let published_revision = self.graph.summary().await.revision;
        self.workflows.lock().await.record_contract_publication(
            &record.manifest.plan_id,
            record.progress.progress_revision,
            receipt.path.clone(),
            published_revision,
        )?;
        Ok(receipt)
    }

    pub async fn complete_build_plan(
        &self,
        request: CompleteBuildPlanRequest,
    ) -> Result<BuildPlanRecord> {
        let record = self.workflows.lock().await.plan(&request.plan_id)?;
        let graph_revision = self
            .validate_plan_scope_authority(&record, "completion")
            .await?
            .revision;
        self.workflows.lock().await.complete_plan(
            &request.plan_id,
            request.expected_progress_revision,
            graph_revision,
            request.checks,
        )
    }

    pub async fn request_build(
        &self,
        diagram_path: String,
        source_revision: String,
        selected_entity_ref: Option<String>,
    ) -> Result<BuildRequest> {
        let path = self.app.resolve(&diagram_path, false)?;
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("bpmn" | "cmmn")
        ) {
            return Err(Error::Message(
                "Play requires an active BPMN or CMMN diagram".into(),
            ));
        }
        let current = format!("sha256:{:x}", Sha256::digest(fs::read(path)?));
        if current != source_revision {
            return Err(Error::RevisionConflict {
                path: diagram_path,
                expected: source_revision,
                current,
            });
        }
        let graph_revision = self.graph.summary().await.revision;
        let previous_revision = self
            .workflows
            .lock()
            .await
            .build()
            .map(|request| request.request_revision)
            .unwrap_or(0);
        let request = BuildRequest {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            request_id: random_hex(12),
            project_id: self.project_id.to_string(),
            diagram_path,
            source_revision: current,
            graph_revision,
            selected_entity_ref,
            request_revision: previous_revision + 1,
            state: BuildState::Ready,
            changed_paths: vec![],
            checks: BTreeMap::new(),
            diagnostic: None,
            updated_at_epoch_ms: now_epoch_ms(),
        };
        let request = self.workflows.lock().await.save_build(request)?;
        self.emit("buildReady", None, Some(request.request_id.clone()))
            .await;
        Ok(request)
    }

    pub async fn build_request(&self) -> Result<Option<BuildRequest>> {
        let Some(mut request) = self.workflows.lock().await.projected_build() else {
            return Ok(None);
        };
        if request.request_id.starts_with("plan-") || request.request_id.starts_with("legacy-") {
            return Ok(Some(request));
        }
        if matches!(request.state, BuildState::Ready | BuildState::Building) {
            let path = self.app.resolve(&request.diagram_path, false)?;
            let current = format!("sha256:{:x}", Sha256::digest(fs::read(path)?));
            if current != request.source_revision {
                request.state = BuildState::Stale;
                request.request_revision += 1;
                request.diagnostic =
                    Some("diagram changed; click Play again to build the current model".into());
                request.updated_at_epoch_ms = now_epoch_ms();
                self.workflows.lock().await.save_build(request.clone())?;
                self.emit("buildStale", None, Some(request.request_id.clone()))
                    .await;
            }
        }
        Ok(Some(request))
    }

    pub async fn update_build(&self, update: BuildUpdate) -> Result<BuildRequest> {
        let request = self.workflows.lock().await.update_build(update)?;
        let kind = match request.state {
            BuildState::Building => "buildStarted",
            BuildState::Complete => "buildComplete",
            BuildState::Failed => "buildFailed",
            _ => "buildUpdated",
        };
        self.emit(kind, None, Some(request.request_id.clone()))
            .await;
        Ok(request)
    }

    pub async fn health(&self, browser_url: String) -> DaemonHealth {
        let graph = self.graph.summary().await;
        let pending = self
            .workflows
            .lock()
            .await
            .proposals
            .values()
            .filter(|value| !value.state.terminal())
            .count();
        let refresh = self.graph.refresh_status_public().await;
        DaemonHealth {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            project_id: self.project_id.to_string(),
            generation: self.generation.to_string(),
            status: "healthy".into(),
            browser_url,
            active_graph_revision: graph.revision,
            pending_proposals: pending,
            browser_sessions: usize::from(self.session.read().await.is_some()),
            refresh_status: format!("{:?}", refresh.status),
            integration_versions: BTreeMap::from([
                ("daemon".into(), DAEMON_PROTOCOL_VERSION.into()),
                (
                    "delivery".into(),
                    crate::delivery_protocol::DELIVERY_PROTOCOL_VERSION.into(),
                ),
            ]),
        }
    }

    pub fn discovery(&self, http_url: String, rpc_port: u16) -> DaemonDiscovery {
        DaemonDiscovery {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            project_id: self.project_id.to_string(),
            project_root_hash: root_hash(&self.project),
            generation: self.generation.to_string(),
            pid: std::process::id(),
            http_url,
            rpc_port,
            token: self.token.to_string(),
            started_at_epoch_ms: now_epoch_ms(),
        }
    }

    fn touch(&self) {
        self.last_activity
            .store(now_epoch_ms() as u64, Ordering::Relaxed);
    }
    pub fn idle_for(&self, now: u128) -> std::time::Duration {
        std::time::Duration::from_millis(
            now.saturating_sub(self.last_activity.load(Ordering::Relaxed) as u128) as u64,
        )
    }
}

impl crate::schematic_mcp::WorkflowBackend for ProjectRuntime {
    fn open_design_workspace(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<RuntimeWorkspaceSnapshot, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.open_design_workspace()
                .await
                .map_err(|error| error.to_string())
        })
    }

    fn save_interview(
        &self,
        record: InterviewRecord,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<InterviewRecord, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.save_interview(record)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn submit_proposal(
        &self,
        record: ProposalRecord,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<ProposalRecord, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.submit_proposal(record)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn get_proposal(
        &self,
        id: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<ProposalRecord>> + Send + '_>>
    {
        Box::pin(async move { self.proposal(&id).await })
    }
    fn cancel_proposal(
        &self,
        id: String,
        expected: u64,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<ProposalRecord, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.cancel_proposal(&id, expected)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn get_build(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<Option<BuildRequest>, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.build_request()
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn update_build(
        &self,
        update: BuildUpdate,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<BuildRequest, String>> + Send + '_,
        >,
    > {
        Box::pin(async move {
            self.update_build(update)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn create_build_plan(
        &self,
        request: CreateBuildPlanRequest,
        scope_ref: Option<String>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<BuildPlanRecord, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let result = if let Some(scope_ref) = scope_ref {
                self.create_build_plan_for_scope(request, scope_ref).await
            } else {
                self.create_build_plan(request).await
            };
            result.map_err(|error| error.to_string())
        })
    }
    fn list_build_plans(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BuildPlanSummary>> + Send + '_>>
    {
        Box::pin(async move { self.build_plans().await })
    }
    fn get_build_plan(
        &self,
        selector: String,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<BuildPlanRecord, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.build_plan(&selector)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn claim_next_build_item(
        &self,
        request: crate::delivery_protocol::BuildPlanClaimRequest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<BuildPlanRecord, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.claim_next_build_item(&request.plan_id, request.expected_progress_revision)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn build_item_context(
        &self,
        plan_id: String,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<BuildWorkItemContext, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.build_item_context(&plan_id)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn record_build_item(
        &self,
        completion: crate::delivery_protocol::BuildPlanCompletion,
        success: bool,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<BuildPlanRecord, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.record_build_item(completion, success)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn complete_build_plan(
        &self,
        request: CompleteBuildPlanRequest,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = std::result::Result<BuildPlanRecord, String>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.complete_build_plan(request)
                .await
                .map_err(|error| error.to_string())
        })
    }
    fn publish_generated_contract(
        &self,
        submission: GeneratedContractSubmission,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = std::result::Result<
                        crate::delivery_protocol::DocumentRevision,
                        String,
                    >,
                > + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            self.publish_generated_contract(submission)
                .await
                .map_err(|error| error.to_string())
        })
    }
}

pub struct DaemonOwnership {
    project: PathBuf,
    generation: String,
}

impl DaemonOwnership {
    pub fn acquire(project: &Path, discovery: &DaemonDiscovery) -> Result<Self> {
        let run = project.join(".ss/run");
        fs::create_dir_all(&run)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&run, fs::Permissions::from_mode(0o700))?;
        }
        let lock = run.join("daemon.lock");
        match OpenOptions::new().create_new(true).write(true).open(&lock) {
            Ok(mut file) => {
                file.write_all(discovery.generation.as_bytes())?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&lock, fs::Permissions::from_mode(0o600))?;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(Error::Message("project daemon is already owned".into()));
            }
            Err(error) => return Err(error.into()),
        }
        let path = run.join("daemon.json");
        atomic_json(&path, discovery)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(Self {
            project: project.to_path_buf(),
            generation: discovery.generation.clone(),
        })
    }

    pub fn discovery(project: &Path) -> Result<Option<DaemonDiscovery>> {
        read_json(&project.join(".ss/run/daemon.json"))
    }

    pub fn reclaim_stale(project: &Path) -> Result<bool> {
        let Some(discovery) = Self::discovery(project)? else {
            return Ok(false);
        };
        let alive = process_alive(discovery.pid);
        if alive {
            return Ok(false);
        }
        let run = project.join(".ss/run");
        for path in [run.join("daemon.json"), run.join("daemon.lock")] {
            if path.exists() {
                fs::remove_file(path)?;
            }
        }
        Ok(true)
    }

    pub fn healthy_discovery(project: &Path) -> Result<Option<DaemonDiscovery>> {
        let Some(discovery) = Self::discovery(project)? else {
            return Ok(None);
        };
        let local = discovery.http_url.starts_with("http://127.0.0.1:")
            || discovery.http_url.starts_with("http://localhost:")
            || discovery.http_url.starts_with("http://[::1]:");
        if discovery.protocol_version == DAEMON_PROTOCOL_VERSION
            && discovery.project_id == project_id(project)?
            && discovery.rpc_port != 0
            && local
            && process_alive(discovery.pid)
        {
            Ok(Some(discovery))
        } else {
            Ok(None)
        }
    }
}

pub fn stop_daemon(project: &Path) -> Result<bool> {
    let project = canonical_project(project)?;
    let Some(discovery) = DaemonOwnership::healthy_discovery(&project)? else {
        let _ = DaemonOwnership::reclaim_stale(&project)?;
        return Ok(false);
    };
    #[cfg(unix)]
    let status = std::process::Command::new("kill")
        .arg(discovery.pid.to_string())
        .status()?;
    #[cfg(windows)]
    let status = std::process::Command::new("taskkill")
        .args(["/PID", &discovery.pid.to_string()])
        .status()?;
    Ok(status.success())
}

impl Drop for DaemonOwnership {
    fn drop(&mut self) {
        let run = self.project.join(".ss/run");
        if Self::discovery(&self.project)
            .ok()
            .flatten()
            .is_some_and(|value| value.generation == self.generation)
        {
            let _ = fs::remove_file(run.join("daemon.json"));
            let _ = fs::remove_file(run.join("daemon.lock"));
        }
    }
}

fn process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Path::new(&format!("/proc/{pid}")).exists()
            || std::process::Command::new("kill")
                .args(["-0", &pid.to_string()])
                .status()
                .is_ok_and(|status| status.success())
    }
    #[cfg(not(unix))]
    {
        pid == std::process::id()
    }
}

pub async fn serve_private_mcp(
    listener: tokio::net::TcpListener,
    runtime: ProjectRuntime,
) -> Result<()> {
    loop {
        let (stream, peer) = listener.accept().await?;
        if !peer.ip().is_loopback() {
            continue;
        }
        let runtime = runtime.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            let read = tokio::time::timeout(
                std::time::Duration::from_secs(3),
                reader.read_line(&mut line),
            )
            .await;
            let Ok(Ok(size)) = read else { return };
            if size == 0 || size > 4096 {
                return;
            }
            let Ok(hello) = serde_json::from_str::<PrivateDaemonHello>(&line) else {
                return;
            };
            if hello.protocol_version != DAEMON_PROTOCOL_VERSION
                || hello.client_kind != "mcp-proxy"
                || runtime
                    .authenticate(&hello.token, &hello.project_id, &hello.generation)
                    .is_err()
            {
                return;
            }
            let mut stream = reader.into_inner();
            if stream.write_all(b"ok\n").await.is_err() {
                return;
            }
            let _ = crate::schematic_mcp::serve_mcp_stream(runtime.graph.clone(), stream).await;
        });
    }
}

fn canonical_project(project: &Path) -> Result<PathBuf> {
    Ok(project.canonicalize()?)
}

async fn wait_for_discovery(project: &Path) -> Result<DaemonDiscovery> {
    for _ in 0..50 {
        if let Some(discovery) = DaemonOwnership::discovery(project)?
            && discovery.protocol_version == DAEMON_PROTOCOL_VERSION
            && process_alive(discovery.pid)
        {
            return Ok(discovery);
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Err(Error::Message(
        "project daemon did not become ready within 5 seconds".into(),
    ))
}

pub async fn serve_mcp_proxy(project: PathBuf) -> Result<()> {
    let project = canonical_project(&project)?;
    if !project.join("schematics").is_dir() {
        return Err(Error::Message(
            "project has no confined schematics directory".into(),
        ));
    }
    let _ = DaemonOwnership::reclaim_stale(&project)?;
    let discovery = match DaemonOwnership::discovery(&project)? {
        Some(value)
            if process_alive(value.pid) && value.protocol_version == DAEMON_PROTOCOL_VERSION =>
        {
            value
        }
        _ => {
            let executable = std::env::current_exe()?;
            std::process::Command::new(executable)
                .args(["serve", "--project"])
                .arg(&project)
                .arg("--no-open")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;
            wait_for_discovery(&project).await?
        }
    };
    if discovery.project_id != project_id(&project)? || discovery.rpc_port == 0 {
        return Err(Error::Message(
            "daemon discovery does not match this project or lacks MCP transport".into(),
        ));
    }
    eprintln!(
        "Software Schematic MCP proxy project={}",
        project
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("project")
    );
    let address = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, discovery.rpc_port));
    let stream = tokio::net::TcpStream::connect(address).await?;
    let (read, mut write) = stream.into_split();
    let hello = PrivateDaemonHello {
        protocol_version: DAEMON_PROTOCOL_VERSION.into(),
        project_id: discovery.project_id,
        generation: discovery.generation,
        token: discovery.token,
        client_kind: "mcp-proxy".into(),
    };
    write
        .write_all(&serde_json::to_vec(&hello).unwrap())
        .await?;
    write.write_all(b"\n").await?;
    let mut reader = BufReader::new(read);
    let mut acknowledgement = String::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(3),
        reader.read_line(&mut acknowledgement),
    )
    .await
    .map_err(|_| Error::Message("daemon proxy authentication timed out".into()))??;
    if acknowledgement.trim() != "ok" {
        return Err(Error::Message(
            "daemon proxy authentication rejected".into(),
        ));
    }
    let mut read = reader.into_inner();
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let client_to_daemon = tokio::io::copy(&mut stdin, &mut write);
    let daemon_to_client = tokio::io::copy(&mut read, &mut stdout);
    tokio::select! {
        result = client_to_daemon => { result?; }
        result = daemon_to_client => { result?; }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery_protocol::{ProposalDecisionKind, ProposalState};

    #[derive(Default)]
    struct RecordingLauncher {
        targets: std::sync::Mutex<Vec<String>>,
        fail: bool,
    }

    impl BrowserLauncher for RecordingLauncher {
        fn open(&self, target: &str) -> std::result::Result<(), String> {
            self.targets.lock().unwrap().push(target.into());
            if self.fail {
                Err("sensitive launcher failure".into())
            } else {
                Ok(())
            }
        }
    }

    fn fixture() -> tempfile::TempDir {
        let directory = tempfile::tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::create_dir_all(directory.path().join("schematics")).unwrap();
        fs::write(directory.path().join(".ss/project-id"), "runtime-project\n").unwrap();
        fs::write(
            directory.path().join("schematics/main.cmmn"),
            "<definitions/>",
        )
        .unwrap();
        directory
    }

    fn planning_fixture() -> tempfile::TempDir {
        let directory = fixture();
        fs::write(
            directory.path().join("schematics/main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:task id="Checkout" name="Checkout" ssw:implementationStatus="modify"/><cmmn:task id="Archive" name="Archive" ssw:implementationStatus="locked"/></cmmn:definitions>"#,
        )
        .unwrap();
        directory
    }

    async fn wait_for_graph_refresh(runtime: &ProjectRuntime) {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let status = runtime.graph.refresh_status_public().await.status;
                if !matches!(
                    status,
                    crate::schematic_mcp::GraphRefreshStatus::Queued
                        | crate::schematic_mcp::GraphRefreshStatus::Processing
                ) {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }

    fn proposal(id: &str) -> ProposalRecord {
        ProposalRecord {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            proposal_id: id.into(),
            interview_id: "interview".into(),
            project_id: "runtime-project".into(),
            origin: "codex".into(),
            operation_registry_version: "2.0".into(),
            source_revisions: BTreeMap::from([("main.cmmn".into(), "sha256:one".into())]),
            diagram_path: "main.cmmn".into(),
            summary: "Add work".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![serde_json::json!({"type":"add_plan_item","nodeId":"Task_1"})],
            state: ProposalState::AwaitingReview,
            state_revision: 1,
            receipt: None,
            diagnostic: None,
            updated_at_epoch_ms: now_epoch_ms(),
        }
    }

    fn build_plan(
        id: &str,
        scope: &str,
        fingerprint: &str,
        state: BuildPlanState,
    ) -> BuildPlanRecord {
        let item = BuildWorkItem {
            work_item_id: "work-one".into(),
            entity_ref: scope.into(),
            label: "One".into(),
            element_type: "bpmn:task".into(),
            phase: 1,
            depends_on: vec![],
            context_refs: vec![],
            generated_contract: false,
        };
        BuildPlanRecord {
            manifest: BuildPlanManifest {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: id.into(),
                project_id: "runtime-project".into(),
                scope_ref: scope.into(),
                scope_label: "One".into(),
                version: SemanticVersion::new(1, 0, 0),
                scope_contract_revision: fingerprint.into(),
                source_manifest_revision: "sha256:source".into(),
                graph_revision: "sha256:graph".into(),
                diagram_path: "main.cmmn".into(),
                selected_entity_ref: Some("One".into()),
                work_items: vec![item],
                context_refs: vec![],
                excluded_refs: vec![],
                contract_outputs: vec![],
                created_at_epoch_ms: 1,
            },
            progress: BuildPlanProgress {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: id.into(),
                progress_revision: 1,
                state,
                work_item_states: BTreeMap::from([(
                    "work-one".into(),
                    if state == BuildPlanState::Complete {
                        BuildWorkItemState::Complete
                    } else {
                        BuildWorkItemState::Pending
                    },
                )]),
                active_claim: None,
                diagnostic: None,
                updated_at_epoch_ms: 1,
            },
            evidence: BuildPlanEvidence {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: id.into(),
                evidence_revision: 1,
                items: BTreeMap::new(),
                changed_paths: vec![],
                checks: BTreeMap::new(),
                generated_contracts: vec![],
                completion_graph_revision: None,
            },
        }
    }

    #[test]
    fn versioned_build_store_is_atomic_immutable_resumable_and_recovers_claims() {
        let directory = fixture();
        let mut store = WorkflowStore::open(directory.path()).unwrap();
        assert!(matches!(
            store.version_decision("urn:ssw:p:o#one", "sha256:a", None, None).unwrap(),
            VersionDecision::Create(version) if version == SemanticVersion::new(1, 0, 0)
        ));
        let record = build_plan(
            "plan-one",
            "urn:ssw:p:o#one",
            "sha256:a",
            BuildPlanState::Ready,
        );
        store.save_plan(record.clone()).unwrap();
        assert!(
            store
                .plan_directory("plan-one")
                .join("manifest.json")
                .is_file()
        );
        assert!(store.root.join("current-build.json").is_file());
        assert!(matches!(
            store
                .version_decision("urn:ssw:p:o#one", "sha256:a", None, None)
                .unwrap(),
            VersionDecision::Resume(_)
        ));
        assert!(
            store
                .version_decision("urn:ssw:p:o#one", "sha256:b", None, None)
                .is_err()
        );
        assert!(matches!(
            store
                .version_decision(
                    "urn:ssw:p:o#one",
                    "sha256:b",
                    Some(SemanticBump::Fix),
                    Some(&SemanticVersion::new(1, 0, 0)),
                )
                .unwrap(),
            VersionDecision::Create(version) if version == SemanticVersion::new(1, 0, 1)
        ));
        assert!(
            store
                .version_decision(
                    "urn:ssw:p:o#one",
                    "sha256:b",
                    Some(SemanticBump::Fix),
                    Some(&SemanticVersion::new(0, 9, 0)),
                )
                .is_err()
        );
        let mut changed_manifest = record.clone();
        changed_manifest.manifest.scope_label = "Changed".into();
        assert!(store.save_plan(changed_manifest).is_err());

        let claimed = store.claim_next("plan-one", 1).unwrap();
        assert!(claimed.progress.active_claim.is_some());
        assert!(store.claim_next("plan-one", 1).is_err());
        drop(store);
        let recovered = WorkflowStore::open(directory.path()).unwrap();
        let recovered = recovered.plan("plan-one").unwrap();
        assert!(recovered.progress.active_claim.is_none());
        assert_eq!(
            recovered.progress.work_item_states["work-one"],
            BuildWorkItemState::Pending
        );
    }

    #[test]
    fn migrates_legacy_play_without_touching_schematics() {
        let directory = fixture();
        let before = fs::read(directory.path().join("schematics/main.cmmn")).unwrap();
        let legacy = BuildRequest {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            request_id: "old".into(),
            project_id: "runtime-project".into(),
            diagram_path: "main.cmmn".into(),
            source_revision: "sha256:source".into(),
            graph_revision: "sha256:graph".into(),
            selected_entity_ref: Some("Task_1".into()),
            request_revision: 1,
            state: BuildState::Ready,
            changed_paths: vec![],
            checks: BTreeMap::new(),
            diagnostic: None,
            updated_at_epoch_ms: 1,
        };
        fs::create_dir_all(directory.path().join(".ss/workflows")).unwrap();
        atomic_json(&directory.path().join(".ss/workflows/play.json"), &legacy).unwrap();
        let store = WorkflowStore::open(directory.path()).unwrap();
        assert_eq!(store.plans().len(), 1);
        assert_eq!(store.plans()[0].version, SemanticVersion::new(1, 0, 0));
        assert_eq!(store.plans()[0].state, BuildPlanState::Ready);
        assert_eq!(
            fs::read(directory.path().join("schematics/main.cmmn")).unwrap(),
            before
        );
    }

    #[test]
    fn retains_every_open_plan_and_only_fifty_terminal_plans() {
        let directory = fixture();
        let mut store = WorkflowStore::open(directory.path()).unwrap();
        store
            .save_plan(build_plan(
                "plan-open",
                "urn:ssw:p:o#open",
                "sha256:open",
                BuildPlanState::Ready,
            ))
            .unwrap();
        for index in 0..51 {
            store
                .save_plan(build_plan(
                    &format!("plan-terminal-{index:02}"),
                    &format!("urn:ssw:p:o#terminal-{index}"),
                    &format!("sha256:{index}"),
                    BuildPlanState::Complete,
                ))
                .unwrap();
        }
        assert_eq!(
            store
                .builds
                .values()
                .filter(|record| record.progress.state.terminal())
                .count(),
            50
        );
        assert!(store.builds.contains_key("plan-open"));
    }

    #[tokio::test]
    async fn play_and_natural_scope_share_one_plan_and_build_sequentially() {
        let directory = planning_fixture();
        let runtime =
            ProjectRuntime::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let summary = runtime.graph.summary().await;
        let request = CreateBuildPlanRequest {
            protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
            project_id: "runtime-project".into(),
            diagram_path: "main.cmmn".into(),
            selected_entity_ref: Some("Checkout".into()),
            expected_source_manifest_revision: summary.source_manifest_revision.clone(),
            expected_graph_revision: summary.revision.clone(),
            semantic_bump: None,
            expected_prior_version: None,
        };
        let browser = runtime.create_build_plan(request.clone()).await.unwrap();
        assert_eq!(browser.manifest.version, SemanticVersion::new(1, 0, 0));
        assert_eq!(browser.manifest.work_items.len(), 2);
        assert!(
            !browser
                .manifest
                .excluded_refs
                .iter()
                .any(|id| id.ends_with("#Archive"))
        );
        let resolved = runtime
            .graph
            .resolve_plan_intent("Checkout", 5)
            .await
            .unwrap();
        let conversational = runtime
            .create_build_plan_for_scope(request, resolved[0].entity.id.clone())
            .await
            .unwrap();
        assert_eq!(browser.manifest.plan_id, conversational.manifest.plan_id);

        let mut record = browser;
        while record
            .progress
            .work_item_states
            .values()
            .any(|state| *state != BuildWorkItemState::Complete)
        {
            record = runtime
                .claim_next_build_item(&record.manifest.plan_id, record.progress.progress_revision)
                .await
                .unwrap();
            let context = runtime
                .build_item_context(&record.manifest.plan_id)
                .await
                .unwrap();
            assert!(!context.entities.is_empty());
            record = runtime
                .record_build_item(
                    crate::delivery_protocol::BuildPlanCompletion {
                        plan_id: record.manifest.plan_id.clone(),
                        expected_progress_revision: record.progress.progress_revision,
                        claim_token: context.claim.claim_token,
                        evidence: crate::delivery_protocol::BuildWorkItemEvidence {
                            work_item_id: context.focused_item.work_item_id,
                            changed_paths: vec!["src/checkout.rs".into()],
                            physical_effects: vec!["implementation: checkout".into()],
                            checks: BTreeMap::from([("test".into(), "passed".into())]),
                            diagnostic: None,
                        },
                    },
                    true,
                )
                .await
                .unwrap();
        }
        let complete = runtime
            .complete_build_plan(CompleteBuildPlanRequest {
                plan_id: record.manifest.plan_id.clone(),
                expected_progress_revision: record.progress.progress_revision,
                checks: BTreeMap::from([("integration".into(), "passed".into())]),
            })
            .await
            .unwrap();
        assert_eq!(complete.progress.state, BuildPlanState::Complete);
    }

    #[tokio::test]
    async fn build_authority_tracks_the_selected_function_instead_of_the_whole_graph() {
        let directory = planning_fixture();
        let runtime =
            ProjectRuntime::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let summary = runtime.graph.summary().await;
        let record = runtime
            .create_build_plan(CreateBuildPlanRequest {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                project_id: "runtime-project".into(),
                diagram_path: "main.cmmn".into(),
                selected_entity_ref: Some("Checkout".into()),
                expected_source_manifest_revision: summary.source_manifest_revision,
                expected_graph_revision: summary.revision,
                semantic_bump: None,
                expected_prior_version: None,
            })
            .await
            .unwrap();
        assert!(
            record
                .manifest
                .work_items
                .iter()
                .map(|item| item.entity_ref.as_str())
                .chain(record.manifest.context_refs.iter().map(String::as_str))
                .chain(record.manifest.excluded_refs.iter().map(String::as_str))
                .all(|entity_ref| !entity_ref.ends_with("#Archive"))
        );

        fs::write(
            directory.path().join("schematics/main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:task id="Checkout" name="Checkout" ssw:implementationStatus="modify"/><cmmn:task id="Archive" name="Document the next function" ssw:implementationStatus="locked"/></cmmn:definitions>"#,
        )
        .unwrap();
        runtime
            .graph
            .notify_document_changed(
                "main.cmmn",
                crate::schematic_mcp::DocumentChangeKind::Replaced,
            )
            .await;
        wait_for_graph_refresh(&runtime).await;
        assert_ne!(
            runtime.graph.summary().await.revision,
            record.manifest.graph_revision
        );

        let active = runtime
            .claim_next_build_item(&record.manifest.plan_id, record.progress.progress_revision)
            .await
            .unwrap();
        runtime
            .build_item_context(&active.manifest.plan_id)
            .await
            .unwrap();

        fs::write(
            directory.path().join("schematics/main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:task id="Checkout" name="Checkout" ssw:implementationStatus="new"/><cmmn:task id="Archive" name="Document the next function" ssw:implementationStatus="locked"/></cmmn:definitions>"#,
        )
        .unwrap();
        runtime
            .graph
            .notify_document_changed(
                "main.cmmn",
                crate::schematic_mcp::DocumentChangeKind::Replaced,
            )
            .await;
        wait_for_graph_refresh(&runtime).await;

        let error = runtime
            .build_item_context(&active.manifest.plan_id)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("selected build scope changed"));
        assert_eq!(
            runtime
                .build_plan(&active.manifest.plan_id)
                .await
                .unwrap()
                .progress
                .state,
            BuildPlanState::Stale
        );
    }

    #[tokio::test]
    async fn edge_work_item_publishes_separate_claim_bound_contract() {
        let directory = fixture();
        fs::create_dir_all(directory.path().join("schematics/docs")).unwrap();
        fs::write(
            directory.path().join("schematics/main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:task id="Source"/><cmmn:task id="Target"/><cmmn:association id="Flow" sourceRef="Source" targetRef="Target" ssw:implementationStatus="modify"/></cmmn:definitions>"#,
        )
        .unwrap();
        let runtime =
            ProjectRuntime::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let summary = runtime.graph.summary().await;
        let mut record = runtime
            .create_build_plan(CreateBuildPlanRequest {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                project_id: "runtime-project".into(),
                diagram_path: "main.cmmn".into(),
                selected_entity_ref: Some("Flow".into()),
                expected_source_manifest_revision: summary.source_manifest_revision,
                expected_graph_revision: summary.revision,
                semantic_bump: None,
                expected_prior_version: None,
            })
            .await
            .unwrap();
        assert_eq!(record.manifest.contract_outputs.len(), 1);
        record = runtime
            .claim_next_build_item(&record.manifest.plan_id, record.progress.progress_revision)
            .await
            .unwrap();
        let context = runtime
            .build_item_context(&record.manifest.plan_id)
            .await
            .unwrap();
        assert!(context.focused_item.generated_contract);
        let receipt = runtime
            .publish_generated_contract(GeneratedContractSubmission {
                plan_id: record.manifest.plan_id.clone(),
                expected_progress_revision: record.progress.progress_revision,
                claim_token: context.claim.claim_token.clone(),
                entity_ref: context.focused_item.entity_ref.clone(),
                expected_document_revision: crate::delivery_protocol::MISSING_REVISION.into(),
                body: "# Implemented flow\nCarries the durable checkout message.".into(),
            })
            .await
            .unwrap();
        assert_eq!(receipt.path, "docs/Flow-contract.md");
        let physical =
            fs::read_to_string(directory.path().join("schematics/docs/Flow-contract.md")).unwrap();
        let authored = crate::embedding_document::parse_markdown(&physical).body;
        let (metadata, _) =
            crate::delivery_protocol::parse_generated_contract(&authored, "runtime-project")
                .unwrap();
        assert_eq!(metadata.plan_id, record.manifest.plan_id);

        record = runtime
            .record_build_item(
                crate::delivery_protocol::BuildPlanCompletion {
                    plan_id: record.manifest.plan_id.clone(),
                    expected_progress_revision: record.progress.progress_revision,
                    claim_token: context.claim.claim_token,
                    evidence: crate::delivery_protocol::BuildWorkItemEvidence {
                        work_item_id: context.focused_item.work_item_id,
                        changed_paths: vec!["src/flow.rs".into(), receipt.path],
                        physical_effects: vec!["implementation: checkout flow".into()],
                        checks: BTreeMap::from([("contract".into(), "passed".into())]),
                        diagnostic: None,
                    },
                },
                true,
            )
            .await
            .unwrap();
        record = runtime
            .claim_next_build_item(&record.manifest.plan_id, record.progress.progress_revision)
            .await
            .unwrap();
        let integration = runtime
            .build_item_context(&record.manifest.plan_id)
            .await
            .unwrap();
        record = runtime
            .record_build_item(
                crate::delivery_protocol::BuildPlanCompletion {
                    plan_id: record.manifest.plan_id.clone(),
                    expected_progress_revision: record.progress.progress_revision,
                    claim_token: integration.claim.claim_token,
                    evidence: crate::delivery_protocol::BuildWorkItemEvidence {
                        work_item_id: integration.focused_item.work_item_id,
                        changed_paths: vec![],
                        physical_effects: vec!["test: flow integration".into()],
                        checks: BTreeMap::from([("integration".into(), "passed".into())]),
                        diagnostic: None,
                    },
                },
                true,
            )
            .await
            .unwrap();
        let complete = runtime
            .complete_build_plan(CompleteBuildPlanRequest {
                plan_id: record.manifest.plan_id,
                expected_progress_revision: record.progress.progress_revision,
                checks: BTreeMap::from([("integration".into(), "passed".into())]),
            })
            .await
            .unwrap();
        assert_eq!(complete.progress.state, BuildPlanState::Complete);
        assert!(complete.evidence.completion_graph_revision.is_some());
    }

    fn session(runtime: &ProjectRuntime, id: &str) -> BrowserSession {
        BrowserSession {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            session_id: id.into(),
            daemon_generation: runtime.generation.to_string(),
            active_diagram: Some("main.cmmn".into()),
            selected_entity_refs: vec![],
            documents: BTreeMap::new(),
            dirty_documents: vec![],
            last_event_id: 0,
            heartbeat_epoch_ms: now_epoch_ms(),
        }
    }

    #[test]
    fn durable_workflows_recover_and_exclude_sensitive_fields() {
        let directory = fixture();
        let mut store = WorkflowStore::open(directory.path()).unwrap();
        let mut record = proposal("one");
        store.submit(record.clone()).unwrap();
        record.state = ProposalState::Applying;
        record.state_revision = 3;
        atomic_json(&store.proposal_path("one"), &record).unwrap();
        drop(store);
        let store = WorkflowStore::open(directory.path()).unwrap();
        assert_eq!(
            store.proposal("one").unwrap().state,
            ProposalState::RolledBack
        );
        let persisted =
            fs::read_to_string(workflow_root(directory.path()).join("proposals/one.json")).unwrap();
        for forbidden in [
            "apiKey",
            "accessToken",
            "chatTranscript",
            "requirementCorpus",
        ] {
            assert!(!persisted.contains(forbidden));
        }
        assert!(store.current_interview().is_none());
        assert!(store.current_proposal_summary().is_none());
    }

    #[test]
    fn current_workflow_accessors_return_bounded_resumable_state() {
        let directory = fixture();
        let mut store = WorkflowStore::open(directory.path()).unwrap();
        let interview = InterviewRecord {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            interview_id: "interview".into(),
            project_id: "runtime-project".into(),
            active_diagram: "main.cmmn".into(),
            source_revisions: BTreeMap::new(),
            decisions: BTreeMap::from([("goal".into(), "Model orders".into())]),
            unresolved: vec!["recovery".into()],
            updated_at_epoch_ms: now_epoch_ms(),
        };
        store.save_interview(interview.clone()).unwrap();
        store.submit(proposal("one")).unwrap();
        assert_eq!(store.current_interview(), Some(interview));
        let summary = store.current_proposal_summary().unwrap();
        assert_eq!(summary.proposal_id, "one");
        assert_eq!(summary.summary, "Add work");
        let encoded = serde_json::to_string(&summary).unwrap();
        assert!(!encoded.contains("operations"));
        assert!(!encoded.contains("add_plan_item"));
    }

    #[test]
    fn proposal_transitions_are_compare_and_swap_and_browser_approval_only() {
        let directory = fixture();
        let mut store = WorkflowStore::open(directory.path()).unwrap();
        store.submit(proposal("one")).unwrap();
        assert!(
            store
                .transition(&ProposalDecision {
                    protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                    proposal_id: "one".into(),
                    expected_state_revision: 0,
                    session_id: Some("browser".into()),
                    kind: ProposalDecisionKind::Approve,
                    diagnostic: None
                })
                .is_err()
        );
        let approved = store
            .transition(&ProposalDecision {
                protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                proposal_id: "one".into(),
                expected_state_revision: 1,
                session_id: Some("browser".into()),
                kind: ProposalDecisionKind::Approve,
                diagnostic: None,
            })
            .unwrap();
        assert_eq!(approved.state, ProposalState::Approved);
        assert!(
            store
                .transition(&ProposalDecision {
                    protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                    proposal_id: "one".into(),
                    expected_state_revision: 2,
                    session_id: Some("browser".into()),
                    kind: ProposalDecisionKind::Reject,
                    diagnostic: None
                })
                .is_err()
        );
    }

    #[test]
    fn ownership_is_single_instance_and_stale_metadata_is_reclaimed() {
        let directory = fixture();
        let runtime =
            ProjectRuntime::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let discovery = runtime.discovery("http://127.0.0.1:1".into(), 2);
        let owner = DaemonOwnership::acquire(directory.path(), &discovery).unwrap();
        assert!(DaemonOwnership::acquire(directory.path(), &discovery).is_err());
        drop(owner);
        assert!(
            DaemonOwnership::discovery(directory.path())
                .unwrap()
                .is_none()
        );
        let run = directory.path().join(".ss/run");
        fs::create_dir_all(&run).unwrap();
        let mut stale = discovery;
        stale.pid = u32::MAX;
        atomic_json(&run.join("daemon.json"), &stale).unwrap();
        fs::write(run.join("daemon.lock"), "stale").unwrap();
        assert!(DaemonOwnership::reclaim_stale(directory.path()).unwrap());
    }

    #[test]
    fn browser_base_url_is_loopback_and_set_once() {
        let directory = fixture();
        let launcher = Arc::new(RecordingLauncher::default());
        let runtime = ProjectRuntime::load_with_browser_launcher(
            directory.path(),
            LoadOptions::deterministic_test(),
            launcher,
        )
        .unwrap();
        assert!(runtime.request_browser_open().is_err());
        assert!(
            runtime
                .set_browser_base_url("https://example.com".into())
                .is_err()
        );
        runtime
            .set_browser_base_url("http://127.0.0.1:4321/".into())
            .unwrap();
        assert!(
            runtime
                .set_browser_base_url("http://127.0.0.1:4322".into())
                .is_err()
        );
    }

    #[tokio::test]
    async fn workspace_entry_reuses_opens_expires_and_redacts_browser_state() {
        let directory = fixture();
        let launcher = Arc::new(RecordingLauncher::default());
        let runtime = ProjectRuntime::load_with_browser_launcher(
            directory.path(),
            LoadOptions::deterministic_test(),
            launcher.clone(),
        )
        .unwrap();
        runtime
            .set_browser_base_url("http://127.0.0.1:4321".into())
            .unwrap();
        runtime
            .register_session(session(&runtime, "connected"))
            .await
            .unwrap();
        let connected = runtime
            .workspace_snapshot(std::time::Duration::from_millis(1))
            .await
            .unwrap();
        assert_eq!(
            connected.launch_state,
            WorkspaceLaunchState::AlreadyConnected
        );
        assert_eq!(connected.active_diagram.as_deref(), Some("main.cmmn"));
        assert!(launcher.targets.lock().unwrap().is_empty());

        let mut stale = session(&runtime, "stale");
        stale.heartbeat_epoch_ms = 0;
        runtime.register_session(stale).await.unwrap();
        let opening = runtime
            .workspace_snapshot(std::time::Duration::from_millis(1))
            .await
            .unwrap();
        assert_eq!(opening.launch_state, WorkspaceLaunchState::OpenRequested);
        assert!(opening.active_diagram.is_none());
        let targets = launcher.targets.lock().unwrap();
        assert_eq!(targets.len(), 1);
        assert!(targets[0].contains("daemonToken="));
        let encoded = serde_json::to_string(&opening).unwrap();
        assert!(!encoded.contains(runtime.token.as_str()));
        assert!(!encoded.contains("http://"));
        assert!(
            runtime
                .authenticate(
                    "wrong",
                    runtime.project_id.as_str(),
                    runtime.generation.as_str()
                )
                .is_err()
        );
    }

    #[tokio::test]
    async fn workspace_entry_reports_connected_after_open_and_safe_failure() {
        let directory = fixture();
        let launcher = Arc::new(RecordingLauncher::default());
        let runtime = ProjectRuntime::load_with_browser_launcher(
            directory.path(),
            LoadOptions::deterministic_test(),
            launcher.clone(),
        )
        .unwrap();
        runtime
            .set_browser_base_url("http://127.0.0.1:4321".into())
            .unwrap();
        let registering = runtime.clone();
        let observed = launcher.clone();
        tokio::spawn(async move {
            while observed.targets.lock().unwrap().is_empty() {
                tokio::task::yield_now().await;
            }
            registering
                .register_session(session(&registering, "opened"))
                .await
                .unwrap();
        });
        let opened = runtime
            .workspace_snapshot(std::time::Duration::from_millis(500))
            .await
            .unwrap();
        assert_eq!(
            opened.launch_state,
            WorkspaceLaunchState::OpenedAndConnected
        );

        let failed_launcher = Arc::new(RecordingLauncher {
            targets: std::sync::Mutex::new(vec![]),
            fail: true,
        });
        let failed_runtime = ProjectRuntime::load_with_browser_launcher(
            directory.path(),
            LoadOptions::deterministic_test(),
            failed_launcher,
        )
        .unwrap();
        failed_runtime
            .set_browser_base_url("http://127.0.0.1:9876".into())
            .unwrap();
        let failed = failed_runtime
            .workspace_snapshot(std::time::Duration::from_millis(1))
            .await
            .unwrap();
        assert_eq!(failed.launch_state, WorkspaceLaunchState::Failed);
        let encoded = serde_json::to_string(&failed).unwrap();
        assert!(!encoded.contains("sensitive launcher failure"));
        assert!(!encoded.contains(failed_runtime.token.as_str()));
    }

    #[tokio::test]
    async fn runtime_validates_sources_and_uses_the_current_browser() {
        let directory = fixture();
        let runtime =
            ProjectRuntime::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        runtime
            .register_session(session(&runtime, "one"))
            .await
            .unwrap();
        let mut record = proposal("leased");
        record.source_revisions.insert(
            "main.cmmn".into(),
            format!("sha256:{:x}", Sha256::digest(b"<definitions/>")),
        );
        let submitted = runtime.submit_proposal(record).await.unwrap();
        let approved = runtime
            .decide(ProposalDecision {
                protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                proposal_id: submitted.proposal_id.clone(),
                expected_state_revision: submitted.state_revision,
                session_id: Some("one".into()),
                kind: ProposalDecisionKind::Approve,
                diagnostic: None,
            })
            .await
            .unwrap();
        assert!(
            runtime
                .decide(ProposalDecision {
                    protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                    proposal_id: approved.proposal_id.clone(),
                    expected_state_revision: approved.state_revision,
                    session_id: Some("other".into()),
                    kind: ProposalDecisionKind::Applied,
                    diagnostic: None
                })
                .await
                .is_err()
        );
        assert_eq!(
            runtime
                .decide(ProposalDecision {
                    protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                    proposal_id: approved.proposal_id,
                    expected_state_revision: approved.state_revision,
                    session_id: Some("one".into()),
                    kind: ProposalDecisionKind::Applied,
                    diagnostic: None
                })
                .await
                .unwrap()
                .state,
            ProposalState::Applying
        );
    }

    #[tokio::test]
    async fn play_request_is_revision_bound_and_drives_one_build() {
        let directory = fixture();
        let runtime =
            ProjectRuntime::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let revision = format!("sha256:{:x}", Sha256::digest(b"<definitions/>"));
        let ready = runtime
            .request_build("main.cmmn".into(), revision, Some("Task_1".into()))
            .await
            .unwrap();
        assert_eq!(ready.state, BuildState::Ready);
        let building = runtime
            .update_build(BuildUpdate {
                request_id: ready.request_id.clone(),
                expected_request_revision: ready.request_revision,
                state: BuildState::Building,
                changed_paths: vec![],
                checks: BTreeMap::new(),
                diagnostic: None,
            })
            .await
            .unwrap();
        assert_eq!(building.state, BuildState::Building);
        fs::write(
            directory.path().join("schematics/main.cmmn"),
            "<definitions id=\"changed\"/>",
        )
        .unwrap();
        let stale = runtime.build_request().await.unwrap().unwrap();
        assert_eq!(stale.state, BuildState::Stale);
        assert!(
            runtime
                .update_build(BuildUpdate {
                    request_id: stale.request_id,
                    expected_request_revision: stale.request_revision,
                    state: BuildState::Complete,
                    changed_paths: vec!["src/lib.rs".into()],
                    checks: BTreeMap::new(),
                    diagnostic: None
                })
                .await
                .is_err()
        );
        drop(runtime);
        assert_eq!(
            WorkflowStore::open(directory.path())
                .unwrap()
                .build()
                .unwrap()
                .state,
            BuildState::Stale
        );
    }
}
