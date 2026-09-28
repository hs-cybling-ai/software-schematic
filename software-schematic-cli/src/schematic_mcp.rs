use crate::{
    Error, Result as SswResult,
    delivery_protocol::{
        BuildPlanClaimRequest, BuildPlanCompletion, BuildPlanLookup, BuildPlanRecord,
        BuildPlanSummary, BuildPlanWait, BuildRequest, BuildState, BuildUpdate,
        BuildWorkItemContext, CapabilityManifest, CompleteBuildPlanRequest,
        CreateBuildPlanFromIntentRequest, CreateBuildPlanRequest, DELIVERY_PROTOCOL_VERSION,
        DesignWorkspace, DocumentRevision, GeneratedContractSubmission, InterviewRecord,
        ProjectIdentity, ProposalRecord, PublicationOutcome, PublicationReceipt,
        RuntimeWorkspaceSnapshot, SynchronizationRequest, validate_design_workspace,
    },
    schematic_graph::{
        Diagnostic, EntityKind, EntityResult, GraphSnapshot, LoadOptions, ScopeResult,
        SearchResult, SnapshotSummary, VectorReadiness, load_schematic_graph,
        refresh_schematic_graph,
    },
};
use rmcp::{
    Json, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};
use std::{future::Future, pin::Pin};
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntityRequest {
    pub id: Option<String>,
    pub owner_name: Option<String>,
    pub source_id: Option<String>,
    pub name: Option<String>,
}
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct NeighborRequest {
    pub id: String,
    #[serde(default = "default_hops")]
    pub hops: usize,
    #[serde(default)]
    pub relation_types: Vec<String>,
    #[serde(default = "default_direction")]
    pub direction: String,
    #[serde(default)]
    pub entity_kinds: Vec<EntityKind>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: String,
    pub owner_name: Option<String>,
    #[serde(default)]
    pub entity_kinds: Vec<EntityKind>,
    pub neighborhood_root: Option<String>,
    #[serde(default = "default_hops")]
    pub max_distance: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScopeRequest {
    pub proposal: String,
    pub root_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImplementationContextRequest {
    pub protocol_version: String,
    pub project_id: String,
    pub graph_revision: String,
    pub entity_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalLookupRequest {
    pub proposal_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalWaitRequest {
    pub proposal_id: String,
    pub after_state_revision: u64,
    #[serde(default = "default_wait_ms")]
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildWaitRequest {
    #[serde(default)]
    pub after_request_revision: u64,
    #[serde(default = "default_wait_ms")]
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildStartRequest {
    pub request_id: String,
    pub expected_request_revision: u64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildItemResultRequest {
    pub completion: BuildPlanCompletion,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanIntentOutcome {
    pub candidates: Vec<crate::schematic_graph::PlanScopeCandidate>,
    pub plan: Option<BuildPlanRecord>,
    pub diagnostic: Option<String>,
}

fn default_wait_ms() -> u64 {
    15_000
}

pub trait WorkflowBackend: Send + Sync {
    fn open_design_workspace(
        &self,
    ) -> Pin<
        Box<dyn Future<Output = std::result::Result<RuntimeWorkspaceSnapshot, String>> + Send + '_>,
    >;
    fn save_interview(
        &self,
        record: InterviewRecord,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<InterviewRecord, String>> + Send + '_>>;
    fn submit_proposal(
        &self,
        record: ProposalRecord,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<ProposalRecord, String>> + Send + '_>>;
    fn get_proposal(
        &self,
        id: String,
    ) -> Pin<Box<dyn Future<Output = Option<ProposalRecord>> + Send + '_>>;
    fn cancel_proposal(
        &self,
        id: String,
        expected: u64,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<ProposalRecord, String>> + Send + '_>>;
    fn get_build(
        &self,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<Option<BuildRequest>, String>> + Send + '_>>;
    fn update_build(
        &self,
        update: BuildUpdate,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<BuildRequest, String>> + Send + '_>>;
    fn create_build_plan(
        &self,
        request: CreateBuildPlanRequest,
        scope_ref: Option<String>,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<BuildPlanRecord, String>> + Send + '_>>;
    fn list_build_plans(&self) -> Pin<Box<dyn Future<Output = Vec<BuildPlanSummary>> + Send + '_>>;
    fn get_build_plan(
        &self,
        selector: String,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<BuildPlanRecord, String>> + Send + '_>>;
    fn claim_next_build_item(
        &self,
        request: BuildPlanClaimRequest,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<BuildPlanRecord, String>> + Send + '_>>;
    fn build_item_context(
        &self,
        plan_id: String,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<BuildWorkItemContext, String>> + Send + '_>>;
    fn record_build_item(
        &self,
        completion: BuildPlanCompletion,
        success: bool,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<BuildPlanRecord, String>> + Send + '_>>;
    fn complete_build_plan(
        &self,
        request: CompleteBuildPlanRequest,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<BuildPlanRecord, String>> + Send + '_>>;
    fn publish_generated_contract(
        &self,
        submission: GeneratedContractSubmission,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<DocumentRevision, String>> + Send + '_>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImplementationContext {
    pub grant: crate::delivery_protocol::ImplementationContextGrant,
    pub entities: Vec<crate::schematic_graph::GraphEntity>,
    pub relationships: Vec<crate::schematic_graph::GraphRelation>,
    pub citations: BTreeMap<String, crate::schematic_graph::SourceCitation>,
    pub excluded_entity_refs: Vec<String>,
    pub contract_revision: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GraphRefreshOutcome {
    pub status: GraphRefreshStatus,
    #[serde(default)]
    pub notification_id: Option<u64>,
    #[serde(default)]
    pub affected_paths: Vec<String>,
    pub previous_revision: Option<String>,
    pub active_revision: Option<String>,
    pub diagnostic: Option<String>,
    #[serde(default)]
    pub retrieval_mode: Option<String>,
    #[serde(default)]
    pub embedding_revision: Option<String>,
    #[serde(default)]
    pub vector_readiness: Option<VectorReadiness>,
    #[serde(default)]
    pub vector_diagnostics: Vec<Diagnostic>,
    #[serde(default)]
    pub queued_at_epoch_ms: Option<u128>,
    #[serde(default)]
    pub started_at_epoch_ms: Option<u128>,
    #[serde(default)]
    pub completed_at_epoch_ms: Option<u128>,
    #[serde(default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub receipt: Option<PublicationReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GraphRefreshStatus {
    Queued,
    Processing,
    Updated,
    Unchanged,
    NotRunning,
    Failed,
    Rejected,
}

impl GraphRefreshOutcome {
    pub fn not_running(diagnostic: impl Into<String>) -> Self {
        Self {
            status: GraphRefreshStatus::NotRunning,
            notification_id: None,
            affected_paths: Vec::new(),
            previous_revision: None,
            active_revision: None,
            diagnostic: Some(diagnostic.into()),
            retrieval_mode: None,
            embedding_revision: None,
            vector_readiness: None,
            vector_diagnostics: Vec::new(),
            queued_at_epoch_ms: None,
            started_at_epoch_ms: None,
            completed_at_epoch_ms: Some(now_epoch_ms()),
            operation_id: None,
            receipt: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DocumentChangeKind {
    Replaced,
    Created,
    Renamed,
    Deleted,
}

#[derive(Default)]
struct RefreshCoordinatorState {
    pending: BTreeMap<String, (u64, DocumentChangeKind)>,
    running: bool,
}

struct RefreshCoordinator {
    queue: Mutex<RefreshCoordinatorState>,
    status: RwLock<GraphRefreshOutcome>,
}
fn default_hops() -> usize {
    1
}
fn default_limit() -> usize {
    20
}
fn default_direction() -> String {
    "both".into()
}

fn now_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[derive(Clone)]
pub struct SchematicMcp {
    tool_router: ToolRouter<Self>,
    state: Arc<RwLock<Arc<GraphSnapshot>>>,
    project: PathBuf,
    options: LoadOptions,
    refresh: Arc<RefreshCoordinator>,
    workflow: Arc<RwLock<Option<Arc<dyn WorkflowBackend>>>>,
}

impl SchematicMcp {
    pub fn load(project: impl AsRef<Path>, options: LoadOptions) -> SswResult<Self> {
        let project = project.as_ref().canonicalize()?;
        let snapshot = Arc::new(load_schematic_graph(&project, options.clone())?);
        let revision = snapshot.summary.revision.clone();
        let retrieval_mode = snapshot.summary.retrieval_mode.clone();
        let embedding_revision = snapshot.summary.embedding_revision.clone();
        let vector_readiness = snapshot.summary.vector_readiness;
        let vector_diagnostics = snapshot
            .summary
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code.is_vector())
            .cloned()
            .collect();
        Ok(Self {
            tool_router: Self::tool_router(),
            state: Arc::new(RwLock::new(snapshot)),
            project,
            options,
            refresh: Arc::new(RefreshCoordinator {
                queue: Mutex::new(RefreshCoordinatorState::default()),
                status: RwLock::new(GraphRefreshOutcome {
                    status: GraphRefreshStatus::Unchanged,
                    notification_id: None,
                    affected_paths: Vec::new(),
                    previous_revision: Some(revision.clone()),
                    active_revision: Some(revision),
                    diagnostic: None,
                    retrieval_mode: Some(retrieval_mode),
                    embedding_revision: Some(embedding_revision),
                    vector_readiness: Some(vector_readiness),
                    vector_diagnostics,
                    queued_at_epoch_ms: None,
                    started_at_epoch_ms: None,
                    completed_at_epoch_ms: Some(now_epoch_ms()),
                    operation_id: None,
                    receipt: None,
                }),
            }),
            workflow: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn attach_workflow(&self, backend: Arc<dyn WorkflowBackend>) {
        *self.workflow.write().await = Some(backend);
    }

    async fn workflow_backend(&self) -> std::result::Result<Arc<dyn WorkflowBackend>, String> {
        self.workflow
            .read()
            .await
            .clone()
            .ok_or_else(|| "proposal workflow is unavailable outside the project daemon".into())
    }
    pub async fn summary(&self) -> SnapshotSummary {
        self.state.read().await.summary.clone()
    }

    pub async fn resolve_plan_selection(
        &self,
        diagram_path: &str,
        source_id: Option<&str>,
    ) -> crate::Result<crate::schematic_graph::GraphEntity> {
        self.state
            .read()
            .await
            .resolve_plan_selection(diagram_path, source_id)
    }

    pub async fn resolve_plan_intent(
        &self,
        language: &str,
        limit: usize,
    ) -> crate::Result<Vec<crate::schematic_graph::PlanScopeCandidate>> {
        self.state.read().await.resolve_plan_intent(language, limit)
    }

    pub async fn plan_scope(
        &self,
        root_id: &str,
    ) -> crate::Result<crate::schematic_graph::PlanScopeEnvelope> {
        self.state.read().await.plan_scope(root_id)
    }

    pub async fn plan_context(
        &self,
        entity_refs: &std::collections::BTreeSet<String>,
    ) -> (
        Vec<crate::schematic_graph::GraphEntity>,
        Vec<crate::schematic_graph::GraphRelation>,
        BTreeMap<String, crate::schematic_graph::SourceCitation>,
    ) {
        let snapshot = self.state.read().await;
        let entities = entity_refs
            .iter()
            .filter_map(|id| snapshot.entities.get(id).cloned())
            .collect::<Vec<_>>();
        let relationships = snapshot
            .relations
            .iter()
            .filter(|relation| {
                entity_refs.contains(&relation.source) && entity_refs.contains(&relation.target)
            })
            .cloned()
            .collect::<Vec<_>>();
        let citations = entity_refs
            .iter()
            .filter_map(|id| {
                snapshot
                    .source_map
                    .get(id)
                    .cloned()
                    .map(|citation| (id.clone(), citation))
            })
            .collect();
        (entities, relationships, citations)
    }

    async fn enqueue_refresh(
        &self,
        notification_id: u64,
        path: String,
        change_kind: DocumentChangeKind,
    ) -> GraphRefreshOutcome {
        let queued_at = now_epoch_ms();
        let summary = self.summary().await;
        let previous = summary.revision.clone();
        let vector_diagnostics = summary
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code.is_vector())
            .cloned()
            .collect();
        let mut queue = self.refresh.queue.lock().await;
        queue
            .pending
            .insert(path.clone(), (notification_id, change_kind));
        let outcome = GraphRefreshOutcome {
            status: GraphRefreshStatus::Queued,
            notification_id: Some(notification_id),
            affected_paths: vec![path],
            previous_revision: Some(previous.clone()),
            active_revision: Some(previous),
            diagnostic: None,
            retrieval_mode: Some(summary.retrieval_mode),
            embedding_revision: Some(summary.embedding_revision),
            vector_readiness: Some(summary.vector_readiness),
            vector_diagnostics,
            queued_at_epoch_ms: Some(queued_at),
            started_at_epoch_ms: None,
            completed_at_epoch_ms: None,
            operation_id: Some(format!("sync-{notification_id}")),
            receipt: None,
        };
        *self.refresh.status.write().await = outcome.clone();
        if !queue.running {
            queue.running = true;
            let server = self.clone();
            tokio::spawn(async move { server.run_refresh_worker().await });
        }
        outcome
    }

    /// Queue a durable document change on this runtime's sole refresh
    /// coordinator. The daemon calls this directly; no MCP client is needed.
    pub async fn notify_document_changed(
        &self,
        path: impl Into<String>,
        change_kind: DocumentChangeKind,
    ) -> GraphRefreshOutcome {
        self.enqueue_refresh(
            NEXT_NOTIFICATION_ID.fetch_add(1, Ordering::Relaxed),
            path.into(),
            change_kind,
        )
        .await
    }

    async fn refresh_status(&self) -> GraphRefreshOutcome {
        self.refresh.status.read().await.clone()
    }

    /// Exposes the daemon-owned coordinator state without creating another
    /// graph authority. Browser health and private proxy diagnostics use this
    /// same snapshot.
    pub async fn refresh_status_public(&self) -> GraphRefreshOutcome {
        self.refresh_status().await
    }

    async fn run_refresh_worker(self) {
        loop {
            let batch = {
                let mut queue = self.refresh.queue.lock().await;
                if queue.pending.is_empty() {
                    queue.running = false;
                    return;
                }
                std::mem::take(&mut queue.pending)
            };
            let notification_id = batch.values().map(|(id, _)| *id).max();
            let paths = batch.keys().cloned().collect::<Vec<_>>();
            let fallback_reason = batch.values().find_map(|(_, kind)| match kind {
                DocumentChangeKind::Created => {
                    Some("created document requires reachability validation")
                }
                DocumentChangeKind::Renamed => {
                    Some("renamed document requires complete dependency validation")
                }
                DocumentChangeKind::Deleted => {
                    Some("deleted document requires complete dependency validation")
                }
                DocumentChangeKind::Replaced => None,
            });
            let queued_at = self.refresh.status.read().await.queued_at_epoch_ms;
            let summary = self.summary().await;
            let previous = summary.revision.clone();
            let vector_diagnostics = summary
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code.is_vector())
                .cloned()
                .collect();
            *self.refresh.status.write().await = GraphRefreshOutcome {
                status: GraphRefreshStatus::Processing,
                notification_id,
                affected_paths: paths.clone(),
                previous_revision: Some(previous.clone()),
                active_revision: Some(previous.clone()),
                diagnostic: None,
                retrieval_mode: Some(summary.retrieval_mode),
                embedding_revision: Some(summary.embedding_revision),
                vector_readiness: Some(summary.vector_readiness),
                vector_diagnostics,
                queued_at_epoch_ms: queued_at,
                started_at_epoch_ms: Some(now_epoch_ms()),
                completed_at_epoch_ms: None,
                operation_id: notification_id.map(|id| format!("sync-{id}")),
                receipt: None,
            };
            let outcome = self
                .refresh_from_documents(
                    previous,
                    notification_id,
                    paths,
                    queued_at,
                    fallback_reason,
                )
                .await;
            let has_pending = !self.refresh.queue.lock().await.pending.is_empty();
            if !has_pending {
                *self.refresh.status.write().await = outcome;
            }
        }
    }

    async fn refresh_from_documents(
        &self,
        previous: String,
        notification_id: Option<u64>,
        affected_paths: Vec<String>,
        queued_at_epoch_ms: Option<u128>,
        fallback_reason: Option<&'static str>,
    ) -> GraphRefreshOutcome {
        let project = self.project.clone();
        let options = self.options.clone();
        let prior_snapshot = self.state.read().await.clone();
        let prior_summary = prior_snapshot.summary.clone();
        let replacement = tokio::task::spawn_blocking(move || {
            #[cfg(test)]
            std::thread::sleep(options.refresh_delay);
            refresh_schematic_graph(project, options, &prior_snapshot)
        })
        .await
        .map_err(|error| Error::Message(format!("graph refresh task failed: {error}")))
        .and_then(|result| result);
        match replacement {
            Ok(replacement) => {
                let active = replacement.summary.revision.clone();
                let retrieval_mode = replacement.summary.retrieval_mode.clone();
                let embedding_revision = replacement.summary.embedding_revision.clone();
                let vector_readiness = replacement.summary.vector_readiness;
                let vector_diagnostics = replacement
                    .summary
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.code.is_vector())
                    .cloned()
                    .collect::<Vec<_>>();
                let warning = replacement
                    .summary
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.level == "warning" && !diagnostic.code.is_vector()
                    })
                    .map(|diagnostic| match &diagnostic.source {
                        Some(source) => format!("{source}: {}", diagnostic.message),
                        None => diagnostic.message.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let diagnostic = match (fallback_reason, warning.is_empty()) {
                    (Some(reason), true) => Some(reason.into()),
                    (Some(reason), false) => Some(format!("{reason}; {warning}")),
                    (None, false) => Some(warning),
                    (None, true) => None,
                };
                let status = if previous == active {
                    GraphRefreshStatus::Unchanged
                } else {
                    GraphRefreshStatus::Updated
                };
                let operation_id = notification_id
                    .map(|id| format!("sync-{id}"))
                    .unwrap_or_else(|| "sync-editor".into());
                let receipt = PublicationReceipt {
                    protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
                    operation_id: operation_id.clone(),
                    source_manifest_revision: replacement.source_manifest.revision.clone(),
                    graph_revision: active.clone(),
                    outcome: if status == GraphRefreshStatus::Unchanged {
                        PublicationOutcome::Unchanged
                    } else if fallback_reason.is_some() {
                        PublicationOutcome::FullFallback
                    } else {
                        PublicationOutcome::Incremental
                    },
                    processed_paths: affected_paths.clone(),
                };
                *self.state.write().await = Arc::new(replacement);
                GraphRefreshOutcome {
                    status,
                    notification_id,
                    affected_paths,
                    previous_revision: Some(previous),
                    active_revision: Some(active),
                    diagnostic,
                    retrieval_mode: Some(retrieval_mode),
                    embedding_revision: Some(embedding_revision),
                    vector_readiness: Some(vector_readiness),
                    vector_diagnostics,
                    queued_at_epoch_ms,
                    started_at_epoch_ms: Some(now_epoch_ms()),
                    completed_at_epoch_ms: Some(now_epoch_ms()),
                    operation_id: Some(operation_id),
                    receipt: Some(receipt),
                }
            }
            Err(error) => {
                let vector_diagnostics = prior_summary
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.code.is_vector())
                    .cloned()
                    .collect();
                GraphRefreshOutcome {
                    status: GraphRefreshStatus::Failed,
                    notification_id,
                    affected_paths,
                    previous_revision: Some(previous.clone()),
                    active_revision: Some(previous),
                    diagnostic: Some(error.to_string()),
                    retrieval_mode: Some(prior_summary.retrieval_mode),
                    embedding_revision: Some(prior_summary.embedding_revision),
                    vector_readiness: Some(prior_summary.vector_readiness),
                    vector_diagnostics,
                    queued_at_epoch_ms,
                    started_at_epoch_ms: Some(now_epoch_ms()),
                    completed_at_epoch_ms: Some(now_epoch_ms()),
                    operation_id: notification_id.map(|id| format!("sync-{id}")),
                    receipt: None,
                }
            }
        }
    }
}

#[tool_router]
impl SchematicMcp {
    #[tool(
        description = "Discover versioned diagram-delivery capabilities before a design, plan, or build workflow writes anything."
    )]
    async fn get_delivery_capabilities(&self) -> Json<CapabilityManifest> {
        Json(CapabilityManifest {
            delivery_protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
            document_protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
            mcp_protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
            features: vec![
                "changedDocumentSynchronization".into(),
                "publicationReceipts".into(),
                "implementationContext".into(),
                "conditionalDocumentWrites".into(),
                "durableInterviews".into(),
                "liveDiagramProposals".into(),
                "proposalDecisionWaits".into(),
                "playToBuild".into(),
                "versionedBuildPlans".into(),
                "naturalLanguagePlanScope".into(),
                "sequentialBuildClaims".into(),
                "generatedImplementationContracts".into(),
                "guidedDesignWorkspace".into(),
            ],
        })
    }

    #[tool(
        description = "Open or resume this project's browser modeling workspace and return bounded process, selection, interview, and proposal orientation without exposing launch credentials."
    )]
    async fn open_design_workspace(&self) -> std::result::Result<Json<DesignWorkspace>, String> {
        let runtime = self
            .workflow_backend()
            .await?
            .open_design_workspace()
            .await?;
        let snapshot = self.state.read().await;
        if runtime.project_id != snapshot.summary.project_id {
            return Err("workspace runtime belongs to another project".into());
        }
        let (root_anchor, process_candidates) = snapshot
            .design_targets()
            .map_err(|error| error.to_string())?;
        let workspace = DesignWorkspace {
            protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
            project: ProjectIdentity {
                protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
                project_id: snapshot.summary.project_id.clone(),
                project_name: snapshot.summary.project_name.clone(),
            },
            graph_revision: snapshot.summary.revision.clone(),
            source_manifest_revision: snapshot.summary.source_manifest_revision.clone(),
            root_anchor,
            process_candidates,
            launch_state: runtime.launch_state,
            active_diagram: runtime.active_diagram,
            selected_entity_refs: runtime.selected_entity_refs,
            interview: runtime.interview,
            proposal: runtime.proposal,
            diagnostic: runtime.diagnostic,
        };
        validate_design_workspace(&workspace)?;
        Ok(Json(workspace))
    }

    #[tool(
        description = "Start or resume a bounded durable design interview shared by Codex, Claude, and the browser."
    )]
    async fn save_design_interview(
        &self,
        Parameters(record): Parameters<InterviewRecord>,
    ) -> std::result::Result<Json<InterviewRecord>, String> {
        Ok(Json(
            self.workflow_backend()
                .await?
                .save_interview(record)
                .await?,
        ))
    }

    #[tool(
        description = "Submit a typed revision-bound diagram proposal to the live browser review inbox. This never approves or applies the proposal."
    )]
    async fn submit_diagram_proposal(
        &self,
        Parameters(record): Parameters<ProposalRecord>,
    ) -> std::result::Result<Json<ProposalRecord>, String> {
        Ok(Json(
            self.workflow_backend()
                .await?
                .submit_proposal(record)
                .await?,
        ))
    }

    #[tool(description = "Get current durable proposal state and any publication receipt.")]
    async fn get_diagram_proposal(
        &self,
        Parameters(request): Parameters<ProposalLookupRequest>,
    ) -> std::result::Result<Json<ProposalRecord>, String> {
        self.workflow_backend()
            .await?
            .get_proposal(request.proposal_id)
            .await
            .map(Json)
            .ok_or_else(|| "proposal not found".into())
    }

    #[tool(
        description = "Wait a bounded time for a proposal state revision to change. Returns the current resumable state on timeout."
    )]
    async fn wait_for_diagram_proposal(
        &self,
        Parameters(request): Parameters<ProposalWaitRequest>,
    ) -> std::result::Result<Json<ProposalRecord>, String> {
        if request.timeout_ms > 30_000 {
            return Err("proposal wait timeout exceeds 30000 ms".into());
        }
        let backend = self.workflow_backend().await?;
        let deadline =
            tokio::time::Instant::now() + std::time::Duration::from_millis(request.timeout_ms);
        loop {
            let record = backend
                .get_proposal(request.proposal_id.clone())
                .await
                .ok_or_else(|| "proposal not found".to_string())?;
            if record.state_revision > request.after_state_revision
                || tokio::time::Instant::now() >= deadline
            {
                return Ok(Json(record));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    #[tool(
        description = "Cancel an awaiting-review proposal using compare-and-swap state revision."
    )]
    async fn cancel_diagram_proposal(
        &self,
        Parameters(request): Parameters<ProposalWaitRequest>,
    ) -> std::result::Result<Json<ProposalRecord>, String> {
        Ok(Json(
            self.workflow_backend()
                .await?
                .cancel_proposal(request.proposal_id, request.after_state_revision)
                .await?,
        ))
    }

    #[tool(
        description = "Get the current Play-to-build request, if the developer has clicked Play."
    )]
    async fn get_build_request(&self) -> std::result::Result<Json<Option<BuildRequest>>, String> {
        Ok(Json(self.workflow_backend().await?.get_build().await?))
    }

    #[tool(
        description = "Wait briefly for the developer to click Play or for the current build state to advance."
    )]
    async fn wait_for_build_request(
        &self,
        Parameters(request): Parameters<BuildWaitRequest>,
    ) -> std::result::Result<Json<Option<BuildRequest>>, String> {
        if request.timeout_ms > 30_000 {
            return Err("build wait timeout exceeds 30000 ms".into());
        }
        let backend = self.workflow_backend().await?;
        let deadline =
            tokio::time::Instant::now() + std::time::Duration::from_millis(request.timeout_ms);
        loop {
            let current = backend.get_build().await?;
            if current
                .as_ref()
                .is_some_and(|value| value.request_revision > request.after_request_revision)
                || tokio::time::Instant::now() >= deadline
            {
                return Ok(Json(current));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    #[tool(
        description = "Mark the current Play request as building against its diagram and graph revisions."
    )]
    async fn start_build(
        &self,
        Parameters(request): Parameters<BuildStartRequest>,
    ) -> std::result::Result<Json<BuildRequest>, String> {
        let update = BuildUpdate {
            request_id: request.request_id,
            expected_request_revision: request.expected_request_revision,
            state: BuildState::Building,
            changed_paths: vec![],
            checks: BTreeMap::new(),
            diagnostic: None,
        };
        Ok(Json(
            self.workflow_backend().await?.update_build(update).await?,
        ))
    }

    #[tool(
        description = "Complete or fail the current Play request with bounded changed-path and check evidence."
    )]
    async fn complete_build(
        &self,
        Parameters(update): Parameters<BuildUpdate>,
    ) -> std::result::Result<Json<BuildRequest>, String> {
        if !matches!(update.state, BuildState::Complete | BuildState::Failed) {
            return Err("complete_build state must be complete or failed".into());
        }
        if update.changed_paths.len() > 100 || update.checks.len() > 50 {
            return Err("build evidence exceeds configured bounds".into());
        }
        Ok(Json(
            self.workflow_backend().await?.update_build(update).await?,
        ))
    }

    #[tool(
        description = "Resolve a natural-language diagram scope and create the same immutable versioned build plan used by browser Play. Ambiguous matches return a short deterministic candidate list without creating a plan."
    )]
    async fn create_build_plan(
        &self,
        Parameters(request): Parameters<CreateBuildPlanFromIntentRequest>,
    ) -> std::result::Result<Json<PlanIntentOutcome>, String> {
        if request.description.len() > 4_096 {
            return Err("plan description exceeds 4096 characters".into());
        }
        let candidates = self
            .resolve_plan_intent(&request.description, 8)
            .await
            .map_err(|error| error.to_string())?;
        let selected = if let Some(scope_ref) = request.selected_scope_ref.as_deref() {
            candidates
                .iter()
                .find(|candidate| candidate.entity.id == scope_ref)
        } else {
            match candidates.as_slice() {
                [candidate] => Some(candidate),
                [first, second, ..] if first.score > second.score + 100 => Some(first),
                _ => None,
            }
        };
        let Some(selected) = selected else {
            return Ok(Json(PlanIntentOutcome {
                diagnostic: Some(if candidates.is_empty() {
                    "No reachable diagram item matched that description.".into()
                } else {
                    "Choose one of these numbered diagram items and call create_build_plan again with selectedScopeRef.".into()
                }),
                candidates,
                plan: None,
            }));
        };
        let create = CreateBuildPlanRequest {
            protocol_version: request.protocol_version,
            project_id: request.project_id,
            diagram_path: selected.citation.diagram.clone(),
            selected_entity_ref: selected.entity.source_id.clone(),
            expected_source_manifest_revision: request.expected_source_manifest_revision,
            expected_graph_revision: request.expected_graph_revision,
            semantic_bump: request.semantic_bump,
            expected_prior_version: request.expected_prior_version,
        };
        let plan = self
            .workflow_backend()
            .await?
            .create_build_plan(create, Some(selected.entity.id.clone()))
            .await?;
        Ok(Json(PlanIntentOutcome {
            candidates: vec![],
            plan: Some(plan),
            diagnostic: None,
        }))
    }

    #[tool(description = "List build plans in deterministic order with open plans first.")]
    async fn list_build_plans(&self) -> std::result::Result<Json<Vec<BuildPlanSummary>>, String> {
        Ok(Json(
            self.workflow_backend().await?.list_build_plans().await,
        ))
    }

    #[tool(
        description = "Retrieve one build plan by opaque plan ID or exact scope-URN@semantic-version selector."
    )]
    async fn get_build_plan(
        &self,
        Parameters(request): Parameters<BuildPlanLookup>,
    ) -> std::result::Result<Json<BuildPlanRecord>, String> {
        Ok(Json(
            self.workflow_backend()
                .await?
                .get_build_plan(request.selector)
                .await?,
        ))
    }

    #[tool(
        description = "Wait a bounded time for an exact build plan progress revision to advance."
    )]
    async fn wait_for_build_plan(
        &self,
        Parameters(request): Parameters<BuildPlanWait>,
    ) -> std::result::Result<Json<BuildPlanRecord>, String> {
        if request.timeout_ms > 30_000 {
            return Err("build-plan wait timeout exceeds 30000 ms".into());
        }
        let backend = self.workflow_backend().await?;
        let deadline =
            tokio::time::Instant::now() + std::time::Duration::from_millis(request.timeout_ms);
        loop {
            let current = backend.get_build_plan(request.selector.clone()).await?;
            if current.progress.progress_revision > request.after_progress_revision
                || tokio::time::Instant::now() >= deadline
            {
                return Ok(Json(current));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }

    #[tool(
        description = "Compare-and-swap claim the next dependency-ready work item. Only one item may be active per plan."
    )]
    async fn claim_next_build_item(
        &self,
        Parameters(request): Parameters<BuildPlanClaimRequest>,
    ) -> std::result::Result<Json<BuildPlanRecord>, String> {
        Ok(Json(
            self.workflow_backend()
                .await?
                .claim_next_build_item(request)
                .await?,
        ))
    }

    #[tool(
        description = "Load claim-bound focused implementation context, exclusions, and completed evidence for the active work item."
    )]
    async fn get_build_item_context(
        &self,
        Parameters(request): Parameters<BuildPlanLookup>,
    ) -> std::result::Result<Json<BuildWorkItemContext>, String> {
        Ok(Json(
            self.workflow_backend()
                .await?
                .build_item_context(request.selector)
                .await?,
        ))
    }

    #[tool(
        description = "Record bounded evidence and complete or fail the currently claimed work item."
    )]
    async fn record_build_item_result(
        &self,
        Parameters(request): Parameters<BuildItemResultRequest>,
    ) -> std::result::Result<Json<BuildPlanRecord>, String> {
        if request.completion.evidence.changed_paths.len() > 512
            || request.completion.evidence.checks.len() > 100
            || request.completion.evidence.physical_effects.len() > 256
        {
            return Err("build work-item evidence exceeds configured bounds".into());
        }
        Ok(Json(
            self.workflow_backend()
                .await?
                .record_build_item(request.completion, request.success)
                .await?,
        ))
    }

    #[tool(
        description = "Complete a build plan after every work item is complete and record the published graph revision plus integration checks."
    )]
    async fn complete_build_plan(
        &self,
        Parameters(request): Parameters<CompleteBuildPlanRequest>,
    ) -> std::result::Result<Json<BuildPlanRecord>, String> {
        if request.checks.len() > 100 {
            return Err("build-plan checks exceed configured bounds".into());
        }
        Ok(Json(
            self.workflow_backend()
                .await?
                .complete_build_plan(request)
                .await?,
        ))
    }

    #[tool(
        description = "Publish a plan- and claim-bound edge/event implementation contract to its server-derived document path."
    )]
    async fn publish_generated_contract(
        &self,
        Parameters(submission): Parameters<GeneratedContractSubmission>,
    ) -> std::result::Result<Json<DocumentRevision>, String> {
        Ok(Json(
            self.workflow_backend()
                .await?
                .publish_generated_contract(submission)
                .await?,
        ))
    }

    #[tool(
        description = "Submit a confined, revision-validated set of durable diagram or Markdown documents for graph synchronization. The server reads all content itself."
    )]
    async fn synchronize_documents(
        &self,
        Parameters(request): Parameters<SynchronizationRequest>,
    ) -> std::result::Result<Json<GraphRefreshOutcome>, String> {
        crate::delivery_protocol::require_current_version(&request.protocol_version)?;
        let expected_project = project_id(&self.project).map_err(|error| error.to_string())?;
        if request.project_id != expected_project {
            return Err("synchronization request belongs to another project".into());
        }
        let snapshot = self.state.read().await;
        if let Some(base) = &request.base_manifest_revision
            && base != &snapshot.source_manifest.revision
        {
            return Err(format!(
                "source manifest is stale: expected {}, current {}",
                base, snapshot.source_manifest.revision
            ));
        }
        let mut validated = Vec::new();
        for document in request.documents {
            let path = validate_changed_path(&self.project, &document.path)
                .map_err(|error| error.to_string())?;
            let durable = durable_document_revision(&self.project, &path)
                .map_err(|error| error.to_string())?;
            if durable != document.revision {
                return Err(format!(
                    "durable revision is stale for {path}: submitted {}, current {durable}",
                    document.revision
                ));
            }
            let kind = match document.change_kind.as_str() {
                "created" => DocumentChangeKind::Created,
                "replaced" => DocumentChangeKind::Replaced,
                "renamed" => DocumentChangeKind::Renamed,
                "deleted" => DocumentChangeKind::Deleted,
                other => return Err(format!("unsupported change kind: {other}")),
            };
            validated.push((path, kind));
        }
        drop(snapshot);
        if validated.is_empty() {
            return Ok(Json(self.refresh_status().await));
        }
        let notification_id = NEXT_NOTIFICATION_ID.fetch_add(1, Ordering::Relaxed);
        let mut outcome = None;
        for (path, kind) in validated {
            outcome = Some(self.enqueue_refresh(notification_id, path, kind).await);
        }
        Ok(Json(outcome.expect("validated change set is non-empty")))
    }

    #[tool(
        description = "Return deterministic, revision-bound implementation context for an explicitly authorized new/modify entity set."
    )]
    async fn get_implementation_context(
        &self,
        Parameters(request): Parameters<ImplementationContextRequest>,
    ) -> std::result::Result<Json<ImplementationContext>, String> {
        crate::delivery_protocol::require_current_version(&request.protocol_version)?;
        let snapshot = self.state.read().await;
        if request.project_id != snapshot.summary.project_id {
            return Err("implementation context request belongs to another project".into());
        }
        if request.graph_revision != snapshot.summary.revision {
            return Err(format!(
                "revalidation required: requested graph revision {}, active {}",
                request.graph_revision, snapshot.summary.revision
            ));
        }
        if request.entity_refs.is_empty() || request.entity_refs.len() > 100 {
            return Err(
                "implementation context requires between 1 and 100 entity references".into(),
            );
        }
        let mut entities = Vec::new();
        let mut citations = BTreeMap::new();
        for id in &request.entity_refs {
            let entity = snapshot
                .entities
                .get(id)
                .ok_or_else(|| format!("unknown implementation entity: {id}"))?;
            if !entity.development_scope_eligible {
                return Err(format!(
                    "entity is not eligible for implementation (expected new/modify): {id}"
                ));
            }
            entities.push(entity.clone());
            if let Some(citation) = snapshot.source_map.get(id) {
                citations.insert(id.clone(), citation.clone());
            }
        }
        let selected = request
            .entity_refs
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let relationships = snapshot
            .relations
            .iter()
            .filter(|relation| {
                selected.contains(&relation.source) || selected.contains(&relation.target)
            })
            .cloned()
            .collect::<Vec<_>>();
        let excluded_entity_refs = snapshot
            .entities
            .values()
            .filter(|entity| !entity.development_scope_eligible)
            .map(|entity| entity.id.clone())
            .collect::<Vec<_>>();
        let encoded = serde_json::to_vec(&(&entities, &relationships, &citations)).unwrap();
        let contract_revision = format!("sha256:{:x}", Sha256::digest(encoded));
        Ok(Json(ImplementationContext {
            grant: crate::delivery_protocol::ImplementationContextGrant {
                protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
                project_id: snapshot.summary.project_id.clone(),
                graph_revision: snapshot.summary.revision.clone(),
                source_manifest_revision: snapshot.summary.source_manifest_revision.clone(),
                entity_refs: request.entity_refs,
            },
            entities,
            relationships,
            citations,
            excluded_entity_refs,
            contract_revision,
        }))
    }

    #[tool(
        description = "Return and verify the project identity, root schematic, snapshot revision, model, counts, and diagnostics. Call this before other Software Schematic tools."
    )]
    async fn get_project_model(&self) -> Json<SnapshotSummary> {
        Json(self.state.read().await.summary.clone())
    }

    #[tool(
        description = "Get one compiled schematic diagram, node, or edge by universal URN, semantic owner plus source ID, canonical Name, or unambiguous source ID."
    )]
    async fn get_entity(
        &self,
        Parameters(p): Parameters<EntityRequest>,
    ) -> std::result::Result<Json<EntityResult>, String> {
        self.state
            .read()
            .await
            .get_entity(
                p.id.as_deref(),
                p.owner_name.as_deref(),
                p.source_id.as_deref(),
                p.name.as_deref(),
            )
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        description = "Resolve natural proposal language to a confident new/modify root and authorized development scope. Returns candidates instead of guessing when ambiguous."
    )]
    async fn resolve_development_scope(
        &self,
        Parameters(p): Parameters<ScopeRequest>,
    ) -> std::result::Result<Json<ScopeResult>, String> {
        self.state
            .read()
            .await
            .resolve_scope(&p.proposal, p.root_id.as_deref(), p.limit)
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        description = "Traverse a bounded compiled schematic neighborhood from a universal entity URN."
    )]
    async fn get_neighbors(
        &self,
        Parameters(p): Parameters<NeighborRequest>,
    ) -> std::result::Result<Json<Vec<EntityResult>>, String> {
        self.state
            .read()
            .await
            .neighbors(
                &p.id,
                p.hops,
                &p.relation_types,
                &p.direction,
                &p.entity_kinds,
                p.limit,
            )
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        description = "Search compiled schematic metadata and Markdown using Grafeo native text/vector hybrid retrieval."
    )]
    async fn search_model(
        &self,
        Parameters(p): Parameters<SearchRequest>,
    ) -> std::result::Result<Json<Vec<SearchResult>>, String> {
        self.state
            .read()
            .await
            .search(
                &p.query,
                p.owner_name.as_deref(),
                &p.entity_kinds,
                p.neighborhood_root.as_deref(),
                p.max_distance,
                p.limit,
            )
            .map(Json)
            .map_err(|e| e.to_string())
    }
}

#[tool_handler(router=self.tool_router)]
impl ServerHandler for SchematicMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions("Project-local Software Schematic delivery service. Verify project identity and get_delivery_capabilities first. Planning resolves only reachable diagram entities and creates immutable versioned manifests; build work must be claimed and completed one item at a time. Graph queries are read-only; synchronize_documents accepts only confined durable document identities and revisions, and the server reads source content itself. Only new/modify entities are implementation targets; open/locked entities are context only. Plan tools never grant raw-path, graph-mutation, executable-payload, or proposal-approval authority.")
    }
}

fn project_id(project: &Path) -> SswResult<String> {
    let value = fs::read_to_string(project.join(".ss/project-id"))?;
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::Message(".ss/project-id is empty".into()));
    }
    Ok(value.into())
}

fn validate_changed_path(project: &Path, path: &str) -> SswResult<String> {
    let candidate = Path::new(path);
    if candidate.as_os_str().is_empty()
        || candidate.is_absolute()
        || candidate
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        || !matches!(
            candidate.extension().and_then(|value| value.to_str()),
            Some("bpmn" | "cmmn" | "md")
        )
    {
        return Err(Error::Message(
            "changed document path is not a confined BPMN, CMMN, or Markdown path".into(),
        ));
    }
    let schematics = project.join("schematics").canonicalize()?;
    let target = schematics.join(candidate);
    let confined = if target.exists() {
        target.canonicalize()?.starts_with(&schematics)
    } else {
        target
            .parent()
            .and_then(|parent| parent.canonicalize().ok())
            .is_some_and(|parent| parent.starts_with(&schematics))
    };
    if !confined {
        return Err(Error::Message(
            "changed document path escapes schematics".into(),
        ));
    }
    Ok(candidate.to_string_lossy().replace('\\', "/"))
}

fn durable_document_revision(project: &Path, path: &str) -> SswResult<String> {
    let bytes = fs::read(project.join("schematics").join(path))?;
    let authored = if path.ends_with(".md") {
        let text = String::from_utf8(bytes)
            .map_err(|_| Error::Message(format!("{path}: Markdown must be UTF-8")))?;
        crate::embedding_document::parse_markdown(&text)
            .body
            .into_bytes()
    } else {
        bytes
    };
    Ok(format!("sha256:{:x}", Sha256::digest(&authored)))
}

static NEXT_NOTIFICATION_ID: AtomicU64 = AtomicU64::new(1);

pub async fn serve_mcp(project: PathBuf) -> SswResult<()> {
    crate::project_runtime::serve_mcp_proxy(project).await
}

pub async fn serve_mcp_stream(
    server: SchematicMcp,
    stream: tokio::net::TcpStream,
) -> SswResult<()> {
    let service = server
        .serve(stream)
        .await
        .map_err(|e| Error::Message(format!("MCP initialization: {e}")))?;
    service
        .waiting()
        .await
        .map(|_| ())
        .map_err(|e| Error::Message(format!("MCP service: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        delivery_protocol::{BrowserSession, DAEMON_PROTOCOL_VERSION, ProposalState},
        project_runtime::{BrowserLauncher, ProjectRuntime},
    };
    use std::fs;
    use tempfile::tempdir;

    #[derive(Default)]
    struct UnexpectedLauncher;

    impl BrowserLauncher for UnexpectedLauncher {
        fn open(&self, _target: &str) -> std::result::Result<(), String> {
            Err("browser should not open when a session is connected".into())
        }
    }

    #[derive(Default)]
    struct RecordingLauncher(std::sync::atomic::AtomicUsize);

    impl BrowserLauncher for RecordingLauncher {
        fn open(&self, _target: &str) -> std::result::Result<(), String> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }
    #[test]
    fn defaults_are_bounded() {
        assert_eq!(default_hops(), 1);
        assert_eq!(default_limit(), 20);
    }

    fn fixture() -> tempfile::TempDir {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::create_dir_all(directory.path().join("schematics/docs")).unwrap();
        fs::write(directory.path().join(".ss/project-id"), "mcp-fixture\n").unwrap();
        fs::write(directory.path().join("schematics/main.cmmn"), r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:task id="Checkout" name="Checkout" ssw:implementationStatus="modify"/><cmmn:task id="Payment" name="Payment" ssw:implementationStatus="new"/><cmmn:task id="Locked" name="Archive" ssw:implementationStatus="locked"/><cmmn:processTask id="OrderDef" name="Order" ssw:architecturalName="shop.Order"/><cmmn:planItem id="Order" definitionRef="OrderDef"/></cmmn:definitions>"#).unwrap();
        fs::write(
            directory.path().join("schematics/docs/Checkout.md"),
            "# Checkout\nValidate and submit customer orders.",
        )
        .unwrap();
        fs::write(
            directory.path().join("schematics/docs/Payment.md"),
            "# Payment\nCollect customer payment.",
        )
        .unwrap();
        directory
    }

    #[tokio::test]
    async fn advertises_query_only_tools_and_resolves_scope() {
        let directory = fixture();
        let server =
            SchematicMcp::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let names: Vec<_> = server
            .tool_router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();
        for expected in [
            "get_project_model",
            "get_entity",
            "resolve_development_scope",
            "get_neighbors",
            "search_model",
            "get_delivery_capabilities",
            "synchronize_documents",
            "get_implementation_context",
            "open_design_workspace",
            "create_build_plan",
            "list_build_plans",
            "get_build_plan",
            "wait_for_build_plan",
            "claim_next_build_item",
            "get_build_item_context",
            "record_build_item_result",
            "complete_build_plan",
            "publish_generated_contract",
        ] {
            assert!(names.contains(&expected.into()));
        }
        assert!(!names.contains(&"reload_model".into()));
        let summary = server.get_project_model().await.0;
        assert_eq!(summary.project_id, "mcp-fixture");
        let scope = server
            .resolve_development_scope(Parameters(ScopeRequest {
                proposal: "improve checkout order submission".into(),
                root_id: None,
                limit: 20,
            }))
            .await
            .unwrap()
            .0;
        assert!(
            scope
                .candidates
                .iter()
                .all(|candidate| candidate.root.development_scope_eligible)
        );

        let capabilities = server.get_delivery_capabilities().await.0;
        assert_eq!(
            capabilities.delivery_protocol_version,
            DELIVERY_PROTOCOL_VERSION
        );
        assert!(
            capabilities
                .features
                .contains(&"guidedDesignWorkspace".into())
        );
        let open_tool = server
            .tool_router
            .list_all()
            .into_iter()
            .find(|tool| tool.name == "open_design_workspace")
            .unwrap();
        let open_schema = serde_json::to_string(&open_tool.input_schema).unwrap();
        for forbidden in ["token", "url", "path", "xml", "approve"] {
            assert!(!open_schema.to_ascii_lowercase().contains(forbidden));
        }
        for tool_name in [
            "create_build_plan",
            "claim_next_build_item",
            "record_build_item_result",
            "publish_generated_contract",
        ] {
            let tool = server
                .tool_router
                .list_all()
                .into_iter()
                .find(|tool| tool.name == tool_name)
                .unwrap();
            let schema = serde_json::to_string(&tool.input_schema)
                .unwrap()
                .to_ascii_lowercase();
            for forbidden in ["command", "rawpath", "xml", "approveproposal"] {
                assert!(
                    !schema.contains(forbidden),
                    "{tool_name} exposed {forbidden}"
                );
            }
        }
        let selected = scope.selected_root.clone().unwrap();
        let context = server
            .get_implementation_context(Parameters(ImplementationContextRequest {
                protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
                project_id: "mcp-fixture".into(),
                graph_revision: server.summary().await.revision,
                entity_refs: vec![selected.id.clone()],
            }))
            .await
            .unwrap()
            .0;
        assert_eq!(context.grant.entity_refs, vec![selected.id]);
        assert!(context.contract_revision.starts_with("sha256:"));
        assert!(
            scope
                .candidates
                .iter()
                .any(|candidate| candidate.root.source_id.as_deref() == Some("Checkout"))
        );
        assert_eq!(
            scope
                .selected_root
                .as_ref()
                .and_then(|root| root.source_id.as_deref()),
            Some("Checkout")
        );
        let ambiguous = server
            .resolve_development_scope(Parameters(ScopeRequest {
                proposal: "customer".into(),
                root_id: None,
                limit: 1_000,
            }))
            .await
            .unwrap()
            .0;
        assert!(ambiguous.candidates.len() <= 50);
        assert!(ambiguous.selected_root.is_none());
        assert!(
            server
                .resolve_development_scope(Parameters(ScopeRequest {
                    proposal: "archive".into(),
                    root_id: Some("urn:ssw:mcp-fixture:shop#Locked".into()),
                    limit: 20
                }))
                .await
                .is_err()
        );
        let no_scope = fixture();
        let path = no_scope.path().join("schematics/main.cmmn");
        let xml = fs::read_to_string(&path)
            .unwrap()
            .replace("modify", "open")
            .replace("new", "open");
        fs::write(path, xml).unwrap();
        let server =
            SchematicMcp::load(no_scope.path(), LoadOptions::deterministic_test()).unwrap();
        let result = server
            .resolve_development_scope(Parameters(ScopeRequest {
                proposal: "checkout".into(),
                root_id: None,
                limit: 20,
            }))
            .await
            .unwrap()
            .0;
        assert!(result.selected_root.is_none());
        assert!(
            result
                .diagnostic
                .unwrap()
                .contains("No new or modify nodes")
        );
    }

    #[tokio::test]
    async fn open_workspace_merges_current_project_model_and_resumable_state() {
        let directory = fixture();
        let runtime = ProjectRuntime::load_with_browser_launcher(
            directory.path(),
            LoadOptions::deterministic_test(),
            Arc::new(UnexpectedLauncher),
        )
        .unwrap();
        runtime
            .set_browser_base_url("http://127.0.0.1:4444".into())
            .unwrap();
        runtime
            .register_session(BrowserSession {
                protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                session_id: "browser".into(),
                daemon_generation: runtime.generation.to_string(),
                active_diagram: Some("main.cmmn".into()),
                selected_entity_refs: vec!["urn:ssw:mcp-fixture:shop#Checkout".into()],
                documents: BTreeMap::new(),
                dirty_documents: vec![],
                last_event_id: 0,
                heartbeat_epoch_ms: now_epoch_ms(),
            })
            .await
            .unwrap();
        runtime
            .save_interview(InterviewRecord {
                protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                interview_id: "design".into(),
                project_id: "mcp-fixture".into(),
                active_diagram: "main.cmmn".into(),
                source_revisions: BTreeMap::new(),
                decisions: BTreeMap::from([("goal".into(), "Order goods".into())]),
                unresolved: vec!["recovery".into()],
                updated_at_epoch_ms: now_epoch_ms(),
            })
            .await
            .unwrap();
        let revision = durable_document_revision(directory.path(), "main.cmmn").unwrap();
        runtime
            .submit_proposal(ProposalRecord {
                protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                proposal_id: "proposal".into(),
                interview_id: "design".into(),
                project_id: "mcp-fixture".into(),
                origin: "codex".into(),
                operation_registry_version: "2.0".into(),
                source_revisions: BTreeMap::from([("main.cmmn".into(), revision)]),
                diagram_path: "main.cmmn".into(),
                summary: "Refine ordering".into(),
                assumptions: vec![],
                warnings: vec![],
                operations: vec![],
                state: ProposalState::AwaitingReview,
                state_revision: 1,
                receipt: None,
                diagnostic: None,
                updated_at_epoch_ms: now_epoch_ms(),
            })
            .await
            .unwrap();
        runtime
            .graph
            .attach_workflow(Arc::new(runtime.clone()))
            .await;

        let workspace = runtime.graph.open_design_workspace().await.unwrap().0;
        assert_eq!(workspace.project.project_id, "mcp-fixture");
        assert_eq!(workspace.active_diagram.as_deref(), Some("main.cmmn"));
        assert_eq!(workspace.selected_entity_refs.len(), 1);
        assert_eq!(workspace.interview.as_ref().unwrap().interview_id, "design");
        assert_eq!(workspace.proposal.as_ref().unwrap().proposal_id, "proposal");
        assert!(
            workspace
                .process_candidates
                .iter()
                .any(|candidate| candidate.name.as_deref() == Some("shop.Order"))
        );
        let encoded = serde_json::to_string(&workspace).unwrap();
        assert!(!encoded.contains(runtime.token.as_str()));
        assert!(!encoded.contains("daemonToken"));
    }

    #[tokio::test]
    async fn open_workspace_rejects_a_foreign_runtime_backend() {
        let first = fixture();
        let second = fixture();
        fs::write(second.path().join(".ss/project-id"), "other-project\n").unwrap();
        let server = SchematicMcp::load(first.path(), LoadOptions::deterministic_test()).unwrap();
        let foreign = ProjectRuntime::load_with_browser_launcher(
            second.path(),
            LoadOptions::deterministic_test(),
            Arc::new(UnexpectedLauncher),
        )
        .unwrap();
        foreign
            .set_browser_base_url("http://127.0.0.1:5555".into())
            .unwrap();
        foreign
            .register_session(BrowserSession {
                protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                session_id: "foreign".into(),
                daemon_generation: foreign.generation.to_string(),
                active_diagram: Some("main.cmmn".into()),
                selected_entity_refs: vec![],
                documents: BTreeMap::new(),
                dirty_documents: vec![],
                last_event_id: 0,
                heartbeat_epoch_ms: now_epoch_ms(),
            })
            .await
            .unwrap();
        server.attach_workflow(Arc::new(foreign)).await;
        let error = match server.open_design_workspace().await {
            Ok(_) => panic!("foreign runtime should be rejected"),
            Err(error) => error,
        };
        assert!(error.contains("another project"));
    }

    #[tokio::test]
    async fn open_workspace_requests_the_headless_runtime_browser_once() {
        let directory = fixture();
        let launcher = Arc::new(RecordingLauncher::default());
        let runtime = ProjectRuntime::load_with_browser_launcher(
            directory.path(),
            LoadOptions::deterministic_test(),
            launcher.clone(),
        )
        .unwrap();
        runtime
            .set_browser_base_url("http://127.0.0.1:6666".into())
            .unwrap();
        runtime
            .graph
            .attach_workflow(Arc::new(runtime.clone()))
            .await;
        let registering = runtime.clone();
        let observed = launcher.clone();
        tokio::spawn(async move {
            while observed.0.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
            registering
                .register_session(BrowserSession {
                    protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                    session_id: "opened".into(),
                    daemon_generation: registering.generation.to_string(),
                    active_diagram: Some("main.cmmn".into()),
                    selected_entity_refs: vec![],
                    documents: BTreeMap::new(),
                    dirty_documents: vec![],
                    last_event_id: 0,
                    heartbeat_epoch_ms: now_epoch_ms(),
                })
                .await
                .unwrap();
        });

        let workspace = runtime.graph.open_design_workspace().await.unwrap().0;
        assert_eq!(
            workspace.launch_state,
            crate::delivery_protocol::WorkspaceLaunchState::OpenedAndConnected
        );
        assert_eq!(launcher.0.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(workspace.project.project_id, "mcp-fixture");
    }

    #[tokio::test]
    async fn initialized_project_workspace_smoke_is_orienting_and_non_mutating() {
        let directory = tempdir().unwrap();
        crate::init_project(directory.path()).unwrap();
        let schematics = directory.path().join("schematics");
        fs::create_dir_all(schematics.join("shop/Order")).unwrap();
        fs::write(
            schematics.join("main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:processTask id="OrderDef" name="Order" ssw:architecturalName="shop.Order"/><cmmn:planItem id="Order" definitionRef="OrderDef"/><cmmn:processTask id="RefundDef" name="Refund" ssw:architecturalName="shop.Refund"/><cmmn:planItem id="Refund" definitionRef="RefundDef"/></cmmn:definitions>"#,
        )
        .unwrap();
        fs::write(
            schematics.join("shop/Order/main.bpmn"),
            r#"<bpmn:definitions xmlns:bpmn="x" xmlns:ssw="y" id="Order" ssw:processName="shop.Order"><bpmn:process id="Process_1"><bpmn:startEvent id="Start_1"/></bpmn:process></bpmn:definitions>"#,
        )
        .unwrap();
        let root_before = fs::read(schematics.join("main.cmmn")).unwrap();
        let process_before = fs::read(schematics.join("shop/Order/main.bpmn")).unwrap();

        let launcher = Arc::new(RecordingLauncher::default());
        let runtime = ProjectRuntime::load_with_browser_launcher(
            directory.path(),
            LoadOptions::deterministic_test(),
            launcher.clone(),
        )
        .unwrap();
        runtime
            .set_browser_base_url("http://127.0.0.1:7777".into())
            .unwrap();
        runtime
            .save_interview(InterviewRecord {
                protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                interview_id: "persisted".into(),
                project_id: runtime.project_id.to_string(),
                active_diagram: "main.cmmn".into(),
                source_revisions: BTreeMap::new(),
                decisions: BTreeMap::from([("goal".into(), "Sell products".into())]),
                unresolved: vec!["refund recovery".into()],
                updated_at_epoch_ms: now_epoch_ms(),
            })
            .await
            .unwrap();
        runtime
            .graph
            .attach_workflow(Arc::new(runtime.clone()))
            .await;
        let registering = runtime.clone();
        let observed = launcher.clone();
        tokio::spawn(async move {
            while observed.0.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
            registering
                .register_session(BrowserSession {
                    protocol_version: DAEMON_PROTOCOL_VERSION.into(),
                    session_id: "smoke".into(),
                    daemon_generation: registering.generation.to_string(),
                    active_diagram: Some("main.cmmn".into()),
                    selected_entity_refs: vec![],
                    documents: BTreeMap::new(),
                    dirty_documents: vec![],
                    last_event_id: 0,
                    heartbeat_epoch_ms: now_epoch_ms(),
                })
                .await
                .unwrap();
        });

        let workspace = runtime.graph.open_design_workspace().await.unwrap().0;
        assert_eq!(workspace.project.project_id, runtime.project_id.as_str());
        assert_eq!(
            workspace.root_anchor.diagram_path.as_deref(),
            Some("main.cmmn")
        );
        assert!(workspace.process_candidates.iter().any(|candidate| {
            candidate.name.as_deref() == Some("shop.Order")
                && candidate.composition_state
                    == crate::delivery_protocol::CompositionState::Existing
        }));
        assert!(workspace.process_candidates.iter().any(|candidate| {
            candidate.name.as_deref() == Some("shop.Refund")
                && candidate.composition_state
                    == crate::delivery_protocol::CompositionState::NotCreated
        }));
        assert_eq!(
            workspace.interview.as_ref().unwrap().interview_id,
            "persisted"
        );
        let encoded = serde_json::to_string(&workspace).unwrap();
        assert!(!encoded.contains(runtime.token.as_str()));
        assert!(!encoded.contains("daemonToken"));
        assert_eq!(fs::read(schematics.join("main.cmmn")).unwrap(), root_before);
        assert_eq!(
            fs::read(schematics.join("shop/Order/main.bpmn")).unwrap(),
            process_before
        );
    }

    #[tokio::test]
    async fn synchronization_rejects_foreign_paths_projects_and_stale_revisions() {
        let directory = fixture();
        let server =
            SchematicMcp::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let current = durable_document_revision(directory.path(), "docs/Checkout.md").unwrap();
        let request = |project: &str, path: &str, revision: &str| SynchronizationRequest {
            protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
            project_id: project.into(),
            base_manifest_revision: None,
            documents: vec![crate::delivery_protocol::SynchronizationDocument {
                path: path.into(),
                revision: revision.into(),
                change_kind: "replaced".into(),
            }],
        };
        assert!(
            server
                .synchronize_documents(Parameters(request("other", "docs/Checkout.md", &current)))
                .await
                .is_err()
        );
        assert!(
            server
                .synchronize_documents(Parameters(request("mcp-fixture", "../secret.md", &current)))
                .await
                .is_err()
        );
        assert!(
            server
                .synchronize_documents(Parameters(request(
                    "mcp-fixture",
                    "docs/Checkout.md",
                    "sha256:stale"
                )))
                .await
                .is_err()
        );
        let accepted = server
            .synchronize_documents(Parameters(request(
                "mcp-fixture",
                "docs/Checkout.md",
                &current,
            )))
            .await
            .unwrap()
            .0;
        assert_eq!(accepted.status, GraphRefreshStatus::Queued);
        assert!(accepted.operation_id.is_some());
    }

    #[tokio::test]
    async fn document_notification_refreshes_and_failed_build_keeps_last_revision() {
        let directory = fixture();
        let server =
            SchematicMcp::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let initial = server.summary().await.revision;
        fs::write(
            directory.path().join("schematics/docs/Checkout.md"),
            "# Checkout\nValidate a newly required contract parameter.",
        )
        .unwrap();
        let refreshed = server
            .notify_document_changed("docs/Checkout.md", DocumentChangeKind::Replaced)
            .await;
        assert_eq!(refreshed.status, GraphRefreshStatus::Queued);
        let refreshed = wait_for_refresh(&server).await;
        assert_eq!(refreshed.status, GraphRefreshStatus::Updated);
        assert_eq!(refreshed.retrieval_mode.as_deref(), Some("text"));
        assert_eq!(refreshed.vector_readiness, Some(VectorReadiness::Pending));
        assert!(!refreshed.diagnostic.as_deref().is_some_and(|diagnostic| {
            diagnostic.contains("embedding header")
                || diagnostic.contains("vector retrieval is pending")
        }));
        assert!(refreshed.vector_diagnostics.iter().any(|diagnostic| {
            diagnostic.code == crate::schematic_graph::DiagnosticCode::EmbeddingHeaderMissing
        }));
        let updated = server.summary().await.revision;
        assert_ne!(updated, initial);

        fs::write(
            directory.path().join("schematics/main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" id="D"><cmmn:task id="Keep"/><cmmn:association id="Stale" sourceRef="Deleted" targetRef="Keep"/></cmmn:definitions>"#,
        )
        .unwrap();
        let warned = server
            .notify_document_changed("main.cmmn", DocumentChangeKind::Replaced)
            .await;
        assert_eq!(warned.status, GraphRefreshStatus::Queued);
        let warned = wait_for_refresh(&server).await;
        assert_eq!(warned.status, GraphRefreshStatus::Updated);
        assert!(
            warned
                .diagnostic
                .as_deref()
                .is_some_and(|message| message.contains("skipped edge Stale"))
        );
        assert_eq!(server.summary().await.entities, 2);
        let before_coalesced = server.summary().await.revision;

        fs::write(
            directory.path().join("schematics/main.md"),
            "# Checkout\nThe newest durable contract wins overlapping refreshes.",
        )
        .unwrap();
        let (first, second) = tokio::join!(
            server.notify_document_changed("main.md", DocumentChangeKind::Replaced),
            server.notify_document_changed("main.md", DocumentChangeKind::Replaced)
        );
        assert_eq!(first.status, GraphRefreshStatus::Queued);
        assert_eq!(second.status, GraphRefreshStatus::Queued);
        let coalesced = wait_for_refresh(&server).await;
        assert!(matches!(
            coalesced.status,
            GraphRefreshStatus::Updated | GraphRefreshStatus::Unchanged
        ));
        let updated = server.summary().await.revision;
        assert_ne!(updated, before_coalesced);

        fs::write(directory.path().join("schematics/main.cmmn"), "<broken>").unwrap();
        let failed = server
            .notify_document_changed("main.cmmn", DocumentChangeKind::Replaced)
            .await;
        assert_eq!(failed.status, GraphRefreshStatus::Queued);
        let failed = wait_for_refresh(&server).await;
        assert_eq!(failed.status, GraphRefreshStatus::Failed);
        assert_eq!(failed.active_revision.as_deref(), Some(updated.as_str()));
        assert_eq!(server.summary().await.revision, updated);
    }

    #[test]
    fn changed_paths_are_confined_and_supported() {
        let directory = fixture();
        assert_eq!(
            validate_changed_path(directory.path(), "docs/Checkout.md").unwrap(),
            "docs/Checkout.md"
        );
        assert!(validate_changed_path(directory.path(), "../secret.md").is_err());
        assert!(validate_changed_path(directory.path(), "/tmp/secret.md").is_err());
        assert!(validate_changed_path(directory.path(), "main.txt").is_err());
        #[cfg(unix)]
        {
            let outside = tempfile::tempdir().unwrap();
            fs::write(outside.path().join("secret.md"), "secret").unwrap();
            std::os::unix::fs::symlink(outside.path(), directory.path().join("schematics/escape"))
                .unwrap();
            assert!(validate_changed_path(directory.path(), "escape/secret.md").is_err());
        }
    }

    #[tokio::test]
    async fn enqueue_does_not_wait_for_a_slow_graph_build() {
        let directory = fixture();
        let mut options = LoadOptions::deterministic_test();
        options.refresh_delay = std::time::Duration::from_millis(250);
        let server = SchematicMcp::load(directory.path(), options).unwrap();
        fs::write(
            directory.path().join("schematics/main.md"),
            "# Slow build\nThe save acknowledgement stays fast.",
        )
        .unwrap();

        let started = std::time::Instant::now();
        let outcome = server
            .enqueue_refresh(42, "main.md".into(), DocumentChangeKind::Replaced)
            .await;
        assert_eq!(outcome.status, GraphRefreshStatus::Queued);
        assert!(started.elapsed() < std::time::Duration::from_millis(100));
        assert!(matches!(
            server.refresh_status().await.status,
            GraphRefreshStatus::Queued | GraphRefreshStatus::Processing
        ));
        assert_eq!(
            wait_for_refresh(&server).await.status,
            GraphRefreshStatus::Updated
        );
    }

    async fn wait_for_refresh(server: &SchematicMcp) -> GraphRefreshOutcome {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let status = server.refresh_status().await;
                if !matches!(
                    status.status,
                    GraphRefreshStatus::Queued | GraphRefreshStatus::Processing
                ) {
                    return status;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("background graph refresh timed out")
    }
}
