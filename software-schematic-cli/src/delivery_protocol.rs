use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

pub const DELIVERY_PROTOCOL_VERSION: &str = "1.0";
pub const MISSING_REVISION: &str = "missing";
pub const DAEMON_PROTOCOL_VERSION: &str = "1.0";
pub const MAX_PROPOSAL_OPERATIONS: usize = 64;
pub const MAX_INTERVIEW_DECISIONS: usize = 128;
pub const MAX_SESSION_DOCUMENTS: usize = 128;
pub const MAX_DESIGN_TARGETS: usize = 50;
pub const BUILD_PLAN_PROTOCOL_VERSION: &str = "1.0";
pub const GENERATED_CONTRACT_SCHEMA_VERSION: &str = "1.0";
pub const MAX_BUILD_WORK_ITEMS: usize = 256;
pub const MAX_BUILD_CONTEXT_REFS: usize = 1_024;
pub const MAX_BUILD_EVIDENCE_PATHS: usize = 512;
pub const MAX_GENERATED_CONTRACT_BYTES: usize = 256 * 1_024;
pub const GENERATED_CONTRACT_HEADER_START: &str = "<!-- software-schematic-contract\n";
pub const GENERATED_CONTRACT_HEADER_END: &str = "\n-->\n\n";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectIdentity {
    pub protocol_version: String,
    pub project_id: String,
    pub project_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DocumentRevision {
    pub path: String,
    pub revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConditionalDocumentWrite {
    pub protocol_version: String,
    pub path: String,
    pub content: String,
    pub expected_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceManifest {
    pub protocol_version: String,
    pub revision: String,
    pub documents: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynchronizationOperation {
    pub protocol_version: String,
    pub operation_id: String,
    pub project_id: String,
    pub base_manifest_revision: Option<String>,
    pub documents: Vec<DocumentRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynchronizationRequest {
    pub protocol_version: String,
    pub project_id: String,
    pub base_manifest_revision: Option<String>,
    pub documents: Vec<SynchronizationDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynchronizationDocument {
    pub path: String,
    pub revision: String,
    pub change_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PublicationOutcome {
    Unchanged,
    Incremental,
    FullFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PublicationReceipt {
    pub protocol_version: String,
    pub operation_id: String,
    pub source_manifest_revision: String,
    pub graph_revision: String,
    pub outcome: PublicationOutcome,
    pub processed_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImplementationContextGrant {
    pub protocol_version: String,
    pub project_id: String,
    pub graph_revision: String,
    pub source_manifest_revision: String,
    pub entity_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityManifest {
    pub delivery_protocol_version: String,
    pub document_protocol_version: String,
    pub mcp_protocol_version: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WorkflowPhase {
    Design,
    Graph,
    Build,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillHandoff {
    pub protocol_version: String,
    pub project_id: String,
    pub phase: WorkflowPhase,
    pub source_manifest_revision: Option<String>,
    pub graph_revision: Option<String>,
    pub entity_refs: Vec<String>,
    pub operation_id: Option<String>,
    pub verification: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DaemonDiscovery {
    pub protocol_version: String,
    pub project_id: String,
    pub project_root_hash: String,
    pub generation: String,
    pub pid: u32,
    pub http_url: String,
    pub rpc_port: u16,
    pub token: String,
    pub started_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrivateDaemonHello {
    pub protocol_version: String,
    pub project_id: String,
    pub generation: String,
    pub token: String,
    pub client_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrowserSession {
    pub protocol_version: String,
    pub session_id: String,
    pub daemon_generation: String,
    pub active_diagram: Option<String>,
    pub selected_entity_refs: Vec<String>,
    pub documents: BTreeMap<String, String>,
    pub dirty_documents: Vec<String>,
    pub last_event_id: u64,
    pub heartbeat_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceLaunchState {
    AlreadyConnected,
    OpenedAndConnected,
    OpenRequested,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CompositionState {
    Existing,
    NotCreated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesignTarget {
    pub entity_ref: String,
    pub source_id: Option<String>,
    pub owner_name: String,
    pub name: Option<String>,
    pub label: String,
    pub diagram_path: Option<String>,
    pub composition_state: CompositionState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalResumeSummary {
    pub proposal_id: String,
    pub diagram_path: String,
    pub summary: String,
    pub state: ProposalState,
    pub state_revision: u64,
}

impl From<&ProposalRecord> for ProposalResumeSummary {
    fn from(value: &ProposalRecord) -> Self {
        Self {
            proposal_id: value.proposal_id.clone(),
            diagram_path: value.diagram_path.clone(),
            summary: value.summary.clone(),
            state: value.state.clone(),
            state_revision: value.state_revision,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeWorkspaceSnapshot {
    pub protocol_version: String,
    pub project_id: String,
    pub launch_state: WorkspaceLaunchState,
    pub active_diagram: Option<String>,
    pub selected_entity_refs: Vec<String>,
    pub interview: Option<InterviewRecord>,
    pub proposal: Option<ProposalResumeSummary>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesignWorkspace {
    pub protocol_version: String,
    pub project: ProjectIdentity,
    pub graph_revision: String,
    pub source_manifest_revision: String,
    pub root_anchor: DesignTarget,
    pub process_candidates: Vec<DesignTarget>,
    pub launch_state: WorkspaceLaunchState,
    pub active_diagram: Option<String>,
    pub selected_entity_refs: Vec<String>,
    pub interview: Option<InterviewRecord>,
    pub proposal: Option<ProposalResumeSummary>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeEvent {
    pub protocol_version: String,
    pub event_id: u64,
    pub daemon_generation: String,
    pub kind: String,
    pub proposal_id: Option<String>,
    pub state_revision: Option<u64>,
    pub diagram_path: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InterviewRecord {
    pub protocol_version: String,
    pub interview_id: String,
    pub project_id: String,
    pub active_diagram: String,
    pub source_revisions: BTreeMap<String, String>,
    pub decisions: BTreeMap<String, String>,
    pub unresolved: Vec<String>,
    pub updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ProposalState {
    AwaitingReview,
    Approved,
    Applying,
    Persisted,
    Publishing,
    Published,
    Rejected,
    Cancelled,
    Conflicted,
    Failed,
    RolledBack,
}

impl ProposalState {
    pub fn terminal(&self) -> bool {
        matches!(
            self,
            Self::Published
                | Self::Rejected
                | Self::Cancelled
                | Self::Conflicted
                | Self::Failed
                | Self::RolledBack
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalRecord {
    pub protocol_version: String,
    pub proposal_id: String,
    pub interview_id: String,
    pub project_id: String,
    pub origin: String,
    pub operation_registry_version: String,
    pub source_revisions: BTreeMap<String, String>,
    pub diagram_path: String,
    pub summary: String,
    pub assumptions: Vec<String>,
    pub warnings: Vec<String>,
    pub operations: Vec<serde_json::Value>,
    pub state: ProposalState,
    pub state_revision: u64,
    pub receipt: Option<PublicationReceipt>,
    pub diagnostic: Option<String>,
    pub updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplicationLease {
    pub protocol_version: String,
    pub proposal_id: String,
    pub session_id: String,
    pub daemon_generation: String,
    pub proposal_revision: u64,
    pub expires_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ProposalDecisionKind {
    Approve,
    Reject,
    Cancel,
    Conflict,
    Applied,
    Failed,
    Reverted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProposalDecision {
    pub protocol_version: String,
    pub proposal_id: String,
    pub expected_state_revision: u64,
    pub session_id: Option<String>,
    pub kind: ProposalDecisionKind,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BuildState {
    Ready,
    Building,
    Complete,
    Failed,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildRequest {
    pub protocol_version: String,
    pub request_id: String,
    pub project_id: String,
    pub diagram_path: String,
    pub source_revision: String,
    pub graph_revision: String,
    pub selected_entity_ref: Option<String>,
    pub request_revision: u64,
    pub state: BuildState,
    pub changed_paths: Vec<String>,
    pub checks: BTreeMap<String, String>,
    pub diagnostic: Option<String>,
    pub updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildUpdate {
    pub request_id: String,
    pub expected_request_revision: u64,
    pub state: BuildState,
    pub changed_paths: Vec<String>,
    pub checks: BTreeMap<String, String>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SemanticBump {
    Major,
    Minor,
    Fix,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct SemanticVersion(pub String);

impl SemanticVersion {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self(format!("{major}.{minor}.{patch}"))
    }

    pub fn components(&self) -> Option<(u64, u64, u64)> {
        let mut parts = self.0.split('.');
        let parsed = (
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        );
        (parts.next().is_none() && Self::new(parsed.0, parsed.1, parsed.2).0 == self.0)
            .then_some(parsed)
    }

    pub fn bumped(&self, bump: SemanticBump) -> Option<Self> {
        let (major, minor, patch) = self.components()?;
        Some(match bump {
            SemanticBump::Major => Self::new(major.checked_add(1)?, 0, 0),
            SemanticBump::Minor => Self::new(major, minor.checked_add(1)?, 0),
            SemanticBump::Fix => Self::new(major, minor, patch.checked_add(1)?),
        })
    }
}

impl std::fmt::Display for SemanticVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BuildPlanState {
    Ready,
    Building,
    Complete,
    Failed,
    Stale,
}

impl BuildPlanState {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Complete | Self::Failed | Self::Stale)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BuildWorkItemState {
    Pending,
    Active,
    Complete,
    Failed,
    Stale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildWorkItem {
    pub work_item_id: String,
    pub entity_ref: String,
    pub label: String,
    pub element_type: String,
    pub phase: u32,
    pub depends_on: Vec<String>,
    pub context_refs: Vec<String>,
    pub generated_contract: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedContractOutput {
    pub entity_ref: String,
    pub document_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanManifest {
    pub protocol_version: String,
    pub plan_id: String,
    pub project_id: String,
    pub scope_ref: String,
    pub scope_label: String,
    pub version: SemanticVersion,
    pub scope_contract_revision: String,
    pub source_manifest_revision: String,
    pub graph_revision: String,
    pub diagram_path: String,
    pub selected_entity_ref: Option<String>,
    pub work_items: Vec<BuildWorkItem>,
    pub context_refs: Vec<String>,
    pub excluded_refs: Vec<String>,
    pub contract_outputs: Vec<GeneratedContractOutput>,
    pub created_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildClaim {
    pub work_item_id: String,
    pub claim_token: String,
    pub claimed_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanProgress {
    pub protocol_version: String,
    pub plan_id: String,
    pub progress_revision: u64,
    pub state: BuildPlanState,
    pub work_item_states: BTreeMap<String, BuildWorkItemState>,
    pub active_claim: Option<BuildClaim>,
    pub diagnostic: Option<String>,
    pub updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildWorkItemEvidence {
    pub work_item_id: String,
    pub changed_paths: Vec<String>,
    pub physical_effects: Vec<String>,
    pub checks: BTreeMap<String, String>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanEvidence {
    pub protocol_version: String,
    pub plan_id: String,
    pub evidence_revision: u64,
    pub items: BTreeMap<String, BuildWorkItemEvidence>,
    pub changed_paths: Vec<String>,
    pub checks: BTreeMap<String, String>,
    pub generated_contracts: Vec<String>,
    pub completion_graph_revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanRecord {
    pub manifest: BuildPlanManifest,
    pub progress: BuildPlanProgress,
    pub evidence: BuildPlanEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanSummary {
    pub plan_id: String,
    pub scope_ref: String,
    pub scope_label: String,
    pub version: SemanticVersion,
    pub state: BuildPlanState,
    pub progress_revision: u64,
    pub updated_at_epoch_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateBuildPlanRequest {
    pub protocol_version: String,
    pub project_id: String,
    pub diagram_path: String,
    pub selected_entity_ref: Option<String>,
    pub expected_source_manifest_revision: String,
    pub expected_graph_revision: String,
    pub semantic_bump: Option<SemanticBump>,
    pub expected_prior_version: Option<SemanticVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateBuildPlanFromIntentRequest {
    pub protocol_version: String,
    pub project_id: String,
    pub description: String,
    pub selected_scope_ref: Option<String>,
    pub expected_source_manifest_revision: String,
    pub expected_graph_revision: String,
    pub semantic_bump: Option<SemanticBump>,
    pub expected_prior_version: Option<SemanticVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanLookup {
    pub selector: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanWait {
    pub selector: String,
    pub after_progress_revision: u64,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanClaimRequest {
    pub plan_id: String,
    pub expected_progress_revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompleteBuildPlanRequest {
    pub plan_id: String,
    pub expected_progress_revision: u64,
    pub checks: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildWorkItemContext {
    pub plan_id: String,
    pub version: SemanticVersion,
    pub progress_revision: u64,
    pub claim: BuildClaim,
    pub focused_item: BuildWorkItem,
    pub supporting_items: Vec<BuildWorkItem>,
    pub context_refs: Vec<String>,
    pub excluded_refs: Vec<String>,
    pub completed_evidence: Vec<BuildWorkItemEvidence>,
    pub entities: Vec<BuildContextEntity>,
    pub relationships: Vec<BuildContextRelation>,
    pub citations: BTreeMap<String, BuildContextCitation>,
    pub graph_revision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildContextEntity {
    pub entity_ref: String,
    pub label: String,
    pub element_type: String,
    pub implementation_status: Option<String>,
    pub logical_markdown: String,
    pub implementation_contract_markdown: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildContextRelation {
    pub relation_type: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildContextCitation {
    pub diagram: String,
    pub document: Option<String>,
    pub source_id: Option<String>,
}

impl From<&BuildPlanRecord> for BuildPlanSummary {
    fn from(record: &BuildPlanRecord) -> Self {
        Self {
            plan_id: record.manifest.plan_id.clone(),
            scope_ref: record.manifest.scope_ref.clone(),
            scope_label: record.manifest.scope_label.clone(),
            version: record.manifest.version.clone(),
            state: record.progress.state,
            progress_revision: record.progress.progress_revision,
            updated_at_epoch_ms: record.progress.updated_at_epoch_ms,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BuildPlanCompletion {
    pub plan_id: String,
    pub expected_progress_revision: u64,
    pub claim_token: String,
    pub evidence: BuildWorkItemEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedContractMetadata {
    pub schema_version: String,
    pub project_id: String,
    pub plan_id: String,
    pub scope_ref: String,
    pub semantic_version: SemanticVersion,
    pub entity_ref: String,
    pub base_graph_revision: String,
    pub body_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedContractSubmission {
    pub plan_id: String,
    pub expected_progress_revision: u64,
    pub claim_token: String,
    pub entity_ref: String,
    pub expected_document_revision: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DaemonHealth {
    pub protocol_version: String,
    pub project_id: String,
    pub generation: String,
    pub status: String,
    pub browser_url: String,
    pub active_graph_revision: String,
    pub pending_proposals: usize,
    pub browser_sessions: usize,
    pub refresh_status: String,
    pub integration_versions: BTreeMap<String, String>,
}

pub fn validate_browser_session(session: &BrowserSession) -> Result<(), String> {
    require_daemon_version(&session.protocol_version)?;
    if session.documents.len() > MAX_SESSION_DOCUMENTS
        || session.dirty_documents.len() > MAX_SESSION_DOCUMENTS
        || session.selected_entity_refs.len() > MAX_SESSION_DOCUMENTS
    {
        return Err("browser session exceeds configured bounds".into());
    }
    Ok(())
}

fn safe_project_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("://")
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn credential_free_text(value: &str) -> bool {
    value.len() <= 4_096
        && !value.contains("daemonToken=")
        && !value.contains("http://")
        && !value.contains("https://")
}

fn bounded_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 2_048
        && !value.chars().any(char::is_control)
        && !value.contains("../")
}

pub fn validate_build_plan(record: &BuildPlanRecord, project_id: &str) -> Result<(), String> {
    let manifest = &record.manifest;
    let progress = &record.progress;
    let evidence = &record.evidence;
    if manifest.protocol_version != BUILD_PLAN_PROTOCOL_VERSION
        || progress.protocol_version != BUILD_PLAN_PROTOCOL_VERSION
        || evidence.protocol_version != BUILD_PLAN_PROTOCOL_VERSION
    {
        return Err("unsupported build-plan protocol version".into());
    }
    if manifest.project_id != project_id {
        return Err("build plan belongs to another project".into());
    }
    if manifest.plan_id != progress.plan_id
        || manifest.plan_id != evidence.plan_id
        || !bounded_identifier(&manifest.plan_id)
        || !bounded_identifier(&manifest.scope_ref)
        || manifest.version.components().is_none()
        || !safe_project_relative_path(&manifest.diagram_path)
        || manifest.work_items.is_empty()
        || manifest.work_items.len() > MAX_BUILD_WORK_ITEMS
        || manifest.context_refs.len() > MAX_BUILD_CONTEXT_REFS
        || manifest.excluded_refs.len() > MAX_BUILD_CONTEXT_REFS
        || evidence.changed_paths.len() > MAX_BUILD_EVIDENCE_PATHS
        || evidence.generated_contracts.len() > MAX_BUILD_EVIDENCE_PATHS
    {
        return Err("build plan contains invalid or oversized fields".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    for item in &manifest.work_items {
        if !bounded_identifier(&item.work_item_id)
            || !bounded_identifier(&item.entity_ref)
            || !ids.insert(item.work_item_id.as_str())
            || item.context_refs.len() > MAX_BUILD_CONTEXT_REFS
            || item.depends_on.len() > MAX_BUILD_WORK_ITEMS
        {
            return Err("build plan contains an invalid work item".into());
        }
    }
    if progress.work_item_states.len() != manifest.work_items.len()
        || manifest
            .work_items
            .iter()
            .any(|item| !progress.work_item_states.contains_key(&item.work_item_id))
        || progress.active_claim.as_ref().is_some_and(|claim| {
            !ids.contains(claim.work_item_id.as_str()) || !bounded_identifier(&claim.claim_token)
        })
    {
        return Err("build progress does not match the immutable manifest".into());
    }
    for path in evidence
        .changed_paths
        .iter()
        .chain(evidence.generated_contracts.iter())
        .chain(
            evidence
                .items
                .values()
                .flat_map(|item| item.changed_paths.iter()),
        )
    {
        if !safe_project_relative_path(path) {
            return Err("build evidence contains an unsafe path".into());
        }
    }
    for output in &manifest.contract_outputs {
        if !output.entity_ref.starts_with("urn:ssw:")
            || !safe_project_relative_path(&output.document_path)
            || !(output.document_path.starts_with("docs/")
                || output.document_path.contains("/docs/"))
            || !output.document_path.ends_with("-contract.md")
        {
            return Err("generated contract output is invalid".into());
        }
    }
    Ok(())
}

pub fn validate_generated_contract_metadata(
    metadata: &GeneratedContractMetadata,
    project_id: &str,
) -> Result<(), String> {
    if metadata.schema_version != GENERATED_CONTRACT_SCHEMA_VERSION
        || metadata.project_id != project_id
        || metadata.semantic_version.components().is_none()
        || !bounded_identifier(&metadata.plan_id)
        || !bounded_identifier(&metadata.scope_ref)
        || !metadata.entity_ref.starts_with("urn:ssw:")
        || !bounded_identifier(&metadata.base_graph_revision)
        || metadata.body_hash.len() != 64
        || !metadata
            .body_hash
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err("generated contract metadata is invalid".into());
    }
    Ok(())
}

pub fn validate_generated_contract_submission(
    submission: &GeneratedContractSubmission,
) -> Result<(), String> {
    if !bounded_identifier(&submission.plan_id)
        || !bounded_identifier(&submission.claim_token)
        || !submission.entity_ref.starts_with("urn:ssw:")
        || submission.body.len() > MAX_GENERATED_CONTRACT_BYTES
        || submission.body.contains("<!-- executable:")
    {
        return Err("generated contract submission is invalid or oversized".into());
    }
    Ok(())
}

pub fn serialize_generated_contract(
    metadata: &GeneratedContractMetadata,
    project_id: &str,
    body: &str,
) -> Result<String, String> {
    validate_generated_contract_metadata(metadata, project_id)?;
    if body.len() > MAX_GENERATED_CONTRACT_BYTES {
        return Err("generated contract body exceeds size limit".into());
    }
    let payload = serde_json::to_string(metadata).map_err(|error| error.to_string())?;
    Ok(format!(
        "{GENERATED_CONTRACT_HEADER_START}{payload}{GENERATED_CONTRACT_HEADER_END}{body}"
    ))
}

pub fn parse_generated_contract(
    physical: &str,
    project_id: &str,
) -> Result<(GeneratedContractMetadata, String), String> {
    if !physical.starts_with(GENERATED_CONTRACT_HEADER_START) {
        return Err("generated contract metadata header is missing".into());
    }
    let payload_start = GENERATED_CONTRACT_HEADER_START.len();
    let relative_end = physical[payload_start..]
        .find(GENERATED_CONTRACT_HEADER_END)
        .ok_or_else(|| "generated contract metadata header is unterminated".to_string())?;
    let payload_end = payload_start + relative_end;
    let metadata: GeneratedContractMetadata =
        serde_json::from_str(&physical[payload_start..payload_end])
            .map_err(|error| format!("invalid generated contract metadata: {error}"))?;
    validate_generated_contract_metadata(&metadata, project_id)?;
    let body = physical[payload_end + GENERATED_CONTRACT_HEADER_END.len()..].to_string();
    let digest = format!("{:x}", sha2::Sha256::digest(body.as_bytes()));
    if metadata.body_hash != digest {
        return Err("generated contract body hash does not match metadata".into());
    }
    Ok((metadata, body))
}

pub fn validate_runtime_workspace(snapshot: &RuntimeWorkspaceSnapshot) -> Result<(), String> {
    require_daemon_version(&snapshot.protocol_version)?;
    if snapshot.selected_entity_refs.len() > MAX_SESSION_DOCUMENTS
        || snapshot
            .active_diagram
            .as_deref()
            .is_some_and(|path| !safe_project_relative_path(path))
        || snapshot.proposal.as_ref().is_some_and(|proposal| {
            !safe_project_relative_path(&proposal.diagram_path)
                || !credential_free_text(&proposal.summary)
        })
        || snapshot
            .diagnostic
            .as_deref()
            .is_some_and(|value| !credential_free_text(value))
    {
        return Err("workspace snapshot contains invalid, sensitive, or oversized fields".into());
    }
    if let Some(interview) = &snapshot.interview {
        validate_interview(interview)?;
    }
    Ok(())
}

pub fn validate_design_workspace(workspace: &DesignWorkspace) -> Result<(), String> {
    require_current_version(&workspace.protocol_version)?;
    if workspace.project.protocol_version != workspace.protocol_version
        || workspace.process_candidates.len() > MAX_DESIGN_TARGETS
        || workspace.selected_entity_refs.len() > MAX_SESSION_DOCUMENTS
        || workspace
            .active_diagram
            .as_deref()
            .is_some_and(|path| !safe_project_relative_path(path))
        || workspace
            .diagnostic
            .as_deref()
            .is_some_and(|value| !credential_free_text(value))
    {
        return Err("design workspace contains invalid, sensitive, or oversized fields".into());
    }
    for target in std::iter::once(&workspace.root_anchor).chain(&workspace.process_candidates) {
        if target.entity_ref.len() > 2_048
            || target.owner_name.len() > 1_024
            || target.label.len() > 4_096
            || target
                .name
                .as_ref()
                .is_some_and(|value| value.len() > 1_024)
            || target
                .source_id
                .as_ref()
                .is_some_and(|value| value.len() > 1_024)
            || target
                .diagram_path
                .as_deref()
                .is_some_and(|path| !safe_project_relative_path(path))
        {
            return Err("design target contains invalid or oversized fields".into());
        }
    }
    let runtime = RuntimeWorkspaceSnapshot {
        protocol_version: DAEMON_PROTOCOL_VERSION.into(),
        project_id: workspace.project.project_id.clone(),
        launch_state: workspace.launch_state.clone(),
        active_diagram: workspace.active_diagram.clone(),
        selected_entity_refs: workspace.selected_entity_refs.clone(),
        interview: workspace.interview.clone(),
        proposal: workspace.proposal.clone(),
        diagnostic: workspace.diagnostic.clone(),
    };
    validate_runtime_workspace(&runtime)
}

pub fn validate_interview(record: &InterviewRecord) -> Result<(), String> {
    require_daemon_version(&record.protocol_version)?;
    if record.decisions.len() > MAX_INTERVIEW_DECISIONS
        || record.unresolved.len() > MAX_INTERVIEW_DECISIONS
    {
        return Err("interview exceeds configured bounds".into());
    }
    Ok(())
}

pub fn validate_proposal(record: &ProposalRecord) -> Result<(), String> {
    require_daemon_version(&record.protocol_version)?;
    if record.operations.len() > MAX_PROPOSAL_OPERATIONS {
        return Err("proposal exceeds configured operation limit".into());
    }
    if record.summary.len() > 16_000 || record.diagram_path.contains("..") {
        return Err("proposal contains invalid or oversized fields".into());
    }
    for operation in &record.operations {
        let encoded = serde_json::to_vec(operation).map_err(|error| error.to_string())?;
        if encoded.len() > 64 * 1024
            || operation.get("xml").is_some()
            || operation.get("command").is_some()
            || operation
                .get("path")
                .and_then(|value| value.as_str())
                .is_some_and(|path| {
                    path.starts_with('/') || path.split('/').any(|part| part == "..")
                })
        {
            return Err("proposal operation contains unsupported authority".into());
        }
    }
    Ok(())
}

pub fn require_daemon_version(version: &str) -> Result<(), String> {
    if version == DAEMON_PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(format!(
            "unsupported daemon protocol version {version}; expected {DAEMON_PROTOCOL_VERSION}"
        ))
    }
}

pub fn require_current_version(version: &str) -> Result<(), String> {
    if version == DELIVERY_PROTOCOL_VERSION {
        Ok(())
    } else {
        Err(format!(
            "unsupported delivery protocol version {version}; expected {DELIVERY_PROTOCOL_VERSION}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_contracts_round_trip() {
        let handoff = SkillHandoff {
            protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
            project_id: "project-1".into(),
            phase: WorkflowPhase::Build,
            source_manifest_revision: Some("sha256:manifest".into()),
            graph_revision: Some("sha256:graph".into()),
            entity_refs: vec!["urn:ssw:node:one".into()],
            operation_id: Some("operation-1".into()),
            verification: BTreeMap::from([("tests".into(), "passed".into())]),
        };
        let encoded = serde_json::to_vec(&handoff).unwrap();
        let decoded: SkillHandoff = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, handoff);
        require_current_version(&decoded.protocol_version).unwrap();
    }

    fn build_plan_fixture() -> BuildPlanRecord {
        let work_item = BuildWorkItem {
            work_item_id: "work-1".into(),
            entity_ref: "urn:ssw:project:owner#one".into(),
            label: "One".into(),
            element_type: "bpmn:userTask".into(),
            phase: 1,
            depends_on: vec![],
            context_refs: vec![],
            generated_contract: false,
        };
        BuildPlanRecord {
            manifest: BuildPlanManifest {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: "plan-1".into(),
                project_id: "project-1".into(),
                scope_ref: "urn:ssw:project:owner#one".into(),
                scope_label: "One".into(),
                version: SemanticVersion::new(1, 0, 0),
                scope_contract_revision: "sha256:scope".into(),
                source_manifest_revision: "sha256:source".into(),
                graph_revision: "sha256:graph".into(),
                diagram_path: "main.bpmn".into(),
                selected_entity_ref: Some("Task_1".into()),
                work_items: vec![work_item],
                context_refs: vec![],
                excluded_refs: vec![],
                contract_outputs: vec![],
                created_at_epoch_ms: 1,
            },
            progress: BuildPlanProgress {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: "plan-1".into(),
                progress_revision: 1,
                state: BuildPlanState::Ready,
                work_item_states: BTreeMap::from([("work-1".into(), BuildWorkItemState::Pending)]),
                active_claim: None,
                diagnostic: None,
                updated_at_epoch_ms: 1,
            },
            evidence: BuildPlanEvidence {
                protocol_version: BUILD_PLAN_PROTOCOL_VERSION.into(),
                plan_id: "plan-1".into(),
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
    fn build_plan_contract_round_trips_and_fails_closed() {
        let plan = build_plan_fixture();
        validate_build_plan(&plan, "project-1").unwrap();
        let encoded = serde_json::to_value(&plan).unwrap();
        let decoded: BuildPlanRecord = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(decoded, plan);
        let schema = schemars::schema_for!(BuildPlanRecord);
        let schema_json = serde_json::to_value(schema).unwrap();
        assert!(schema_json.get("$schema").is_some());
        assert!(validate_build_plan(&plan, "foreign-project").is_err());

        let mut incompatible = plan.clone();
        incompatible.manifest.protocol_version = "2.0".into();
        assert!(validate_build_plan(&incompatible, "project-1").is_err());
        let mut oversized = plan.clone();
        oversized.manifest.work_items = (0..=MAX_BUILD_WORK_ITEMS)
            .map(|index| BuildWorkItem {
                work_item_id: format!("work-{index}"),
                ..oversized.manifest.work_items[0].clone()
            })
            .collect();
        assert!(validate_build_plan(&oversized, "project-1").is_err());

        let mut executable = encoded;
        executable
            .as_object_mut()
            .unwrap()
            .insert("command".into(), serde_json::json!("rm -rf ."));
        assert!(serde_json::from_value::<BuildPlanRecord>(executable).is_err());
        assert_eq!(
            SemanticVersion::new(1, 2, 3)
                .bumped(SemanticBump::Minor)
                .unwrap(),
            SemanticVersion::new(1, 3, 0)
        );
    }

    #[test]
    fn generated_contract_metadata_round_trips_and_detects_tampering() {
        let body = "# Implemented event\nPublishes one durable message.";
        let metadata = GeneratedContractMetadata {
            schema_version: GENERATED_CONTRACT_SCHEMA_VERSION.into(),
            project_id: "project-1".into(),
            plan_id: "plan-1".into(),
            scope_ref: "urn:ssw:project-1:owner#scope".into(),
            semantic_version: SemanticVersion::new(1, 2, 3),
            entity_ref: "urn:ssw:project-1:owner#Event_1".into(),
            base_graph_revision: "sha256:graph".into(),
            body_hash: format!("{:x}", sha2::Sha256::digest(body.as_bytes())),
        };
        let physical = serialize_generated_contract(&metadata, "project-1", body).unwrap();
        assert_eq!(
            parse_generated_contract(&physical, "project-1").unwrap(),
            (metadata.clone(), body.into())
        );
        assert!(parse_generated_contract(&physical, "foreign").is_err());
        assert!(
            parse_generated_contract(&physical.replace("durable", "volatile"), "project-1")
                .is_err()
        );
    }

    #[test]
    fn incompatible_versions_and_unknown_fields_fail_closed() {
        assert!(require_current_version("2.0").is_err());
        let invalid = serde_json::json!({
            "protocolVersion": DELIVERY_PROTOCOL_VERSION,
            "path": "main.md",
            "content": "body",
            "expectedRevision": MISSING_REVISION,
            "unexpected": true
        });
        assert!(serde_json::from_value::<ConditionalDocumentWrite>(invalid).is_err());
    }

    #[test]
    fn daemon_contracts_round_trip_and_reject_unknown_or_oversized_input() {
        let proposal = ProposalRecord {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            proposal_id: "proposal-1".into(),
            interview_id: "interview-1".into(),
            project_id: "project-1".into(),
            origin: "codex".into(),
            operation_registry_version: "2.0".into(),
            source_revisions: BTreeMap::from([("main.bpmn".into(), "sha256:one".into())]),
            diagram_path: "main.bpmn".into(),
            summary: "Add one task".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![serde_json::json!({"type":"add_flow_node","nodeId":"Task_1"})],
            state: ProposalState::AwaitingReview,
            state_revision: 1,
            receipt: None,
            diagnostic: None,
            updated_at_epoch_ms: 1,
        };
        validate_proposal(&proposal).unwrap();
        let decoded: ProposalRecord =
            serde_json::from_slice(&serde_json::to_vec(&proposal).unwrap()).unwrap();
        assert_eq!(decoded, proposal);
        let build = BuildRequest {
            protocol_version: DAEMON_PROTOCOL_VERSION.into(),
            request_id: "build-1".into(),
            project_id: "project-1".into(),
            diagram_path: "main.bpmn".into(),
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
        let decoded: BuildRequest =
            serde_json::from_slice(&serde_json::to_vec(&build).unwrap()).unwrap();
        assert_eq!(decoded, build);
        let mut oversized = proposal.clone();
        oversized.operations = (0..=MAX_PROPOSAL_OPERATIONS)
            .map(|index| serde_json::json!({"type":"op","index":index}))
            .collect();
        assert!(validate_proposal(&oversized).is_err());
        let invalid = serde_json::json!({"protocolVersion":DAEMON_PROTOCOL_VERSION,"sessionId":"one","daemonGeneration":"g","activeDiagram":null,"selectedEntityRefs":[],"documents":{},"dirtyDocuments":[],"lastEventId":0,"heartbeatEpochMs":1,"unknown":true});
        assert!(serde_json::from_value::<BrowserSession>(invalid).is_err());
        assert!(require_daemon_version("2.0").is_err());
    }

    #[test]
    fn design_workspace_is_bounded_and_has_no_launch_credentials() {
        let workspace = DesignWorkspace {
            protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
            project: ProjectIdentity {
                protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
                project_id: "project-1".into(),
                project_name: "orders".into(),
            },
            graph_revision: "sha256:graph".into(),
            source_manifest_revision: "sha256:manifest".into(),
            root_anchor: DesignTarget {
                entity_ref: "urn:ssw:project-1:diagram:orders".into(),
                source_id: Some("Root".into()),
                owner_name: "orders".into(),
                name: Some("orders".into()),
                label: "orders".into(),
                diagram_path: Some("main.cmmn".into()),
                composition_state: CompositionState::Existing,
            },
            process_candidates: vec![],
            launch_state: WorkspaceLaunchState::OpenRequested,
            active_diagram: None,
            selected_entity_refs: vec![],
            interview: None,
            proposal: None,
            diagnostic: Some("Open the project with ./ssw if the browser does not appear.".into()),
        };
        validate_design_workspace(&workspace).unwrap();
        let encoded = serde_json::to_string(&workspace).unwrap();
        for forbidden in [
            "daemonToken",
            "sessionId",
            "http://",
            "/Users/example/project",
        ] {
            assert!(!encoded.contains(forbidden));
        }
        let decoded: DesignWorkspace = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, workspace);

        let mut invalid = workspace.clone();
        invalid.diagnostic = Some("http://127.0.0.1:1234/?daemonToken=secret".into());
        assert!(validate_design_workspace(&invalid).is_err());
        let mut absolute = workspace;
        absolute.active_diagram = Some("/private/project/main.cmmn".into());
        assert!(validate_design_workspace(&absolute).is_err());
    }
}
