use crate::{
    Error, Result,
    delivery_protocol::{
        CompositionState, DELIVERY_PROTOCOL_VERSION, DesignTarget, GeneratedContractMetadata,
        GeneratedContractOutput, MAX_DESIGN_TARGETS, SourceManifest, parse_generated_contract,
    },
    embedding_document::{
        CHUNKER_VERSION, EmbeddingChunk, EmbeddingEnvelope, ParsedMarkdown, body_hash,
        embedding_revision, parse_markdown, validate_envelope_shape,
    },
};
use grafeo::{GrafeoDB, NodeId, Value};
use grafeo_engine::embedding::{EmbeddingModel, EmbeddingModelConfig};
use quick_xml::{Reader, events::Event};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;

pub const EMBEDDING_MODEL: &str = "all-MiniLM-L6-v2";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImplementationStatus {
    New,
    Modify,
    Locked,
    Open,
}

impl ImplementationStatus {
    fn parse(value: Option<&str>) -> Self {
        match value {
            Some("new") => Self::New,
            Some("modify") => Self::Modify,
            Some("locked") => Self::Locked,
            _ => Self::Open,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Modify => "modify",
            Self::Locked => "locked",
            Self::Open => "open",
        }
    }
    pub fn eligible(self) -> bool {
        matches!(self, Self::New | Self::Modify)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EntityKind {
    Diagram,
    DiagramNode,
    DiagramEdge,
    DocumentChunk,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GraphEntity {
    pub id: String,
    pub source_id: Option<String>,
    pub kind: EntityKind,
    pub element_type: String,
    pub owner_name: String,
    pub name: Option<String>,
    pub label: String,
    pub implementation_status: Option<ImplementationStatus>,
    pub development_scope_eligible: bool,
    pub markdown: String,
    pub implementation_contract_markdown: Option<String>,
    pub implementation_contract: Option<GeneratedContractMetadata>,
    pub contract_drift: Option<ContractDriftState>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContractDriftState {
    Missing,
    Matching,
    Differing,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GraphRelation {
    pub relation_type: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChunk {
    pub id: String,
    pub owner_id: String,
    pub heading_path: Vec<String>,
    pub ordinal: usize,
    pub markdown: String,
    pub content_hash: String,
    pub embedding_model: String,
    pub dimensions: usize,
    pub document_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceCitation {
    pub diagram: String,
    pub document: Option<String>,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub level: String,
    pub message: String,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticCode {
    GraphStructure,
    ImplementationContract,
    EmbeddingHeaderMissing,
    EmbeddingHeaderMalformed,
    EmbeddingHeaderStale,
    EmbeddingHeaderChunkMismatch,
}

impl DiagnosticCode {
    pub fn is_vector(self) -> bool {
        matches!(
            self,
            Self::EmbeddingHeaderMissing
                | Self::EmbeddingHeaderMalformed
                | Self::EmbeddingHeaderStale
                | Self::EmbeddingHeaderChunkMismatch
        )
    }

    pub fn is_expected_pending(self) -> bool {
        matches!(
            self,
            Self::EmbeddingHeaderMissing | Self::EmbeddingHeaderStale
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum VectorReadiness {
    NotApplicable,
    Current,
    Pending,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GraphLimits {
    pub max_diagrams: usize,
    pub max_entities: usize,
    pub max_document_bytes: usize,
    pub chunk_chars: usize,
    pub chunk_overlap: usize,
    pub max_results: usize,
    pub max_hops: usize,
}
impl Default for GraphLimits {
    fn default() -> Self {
        Self {
            max_diagrams: 512,
            max_entities: 100_000,
            max_document_bytes: 2_000_000,
            chunk_chars: 1800,
            chunk_overlap: 180,
            max_results: 50,
            max_hops: 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummary {
    pub project_name: String,
    pub project_id: String,
    pub root_id: String,
    pub revision: String,
    pub source_manifest_revision: String,
    pub embedding_revision: String,
    pub retrieval_mode: String,
    pub vector_ready: bool,
    pub vector_readiness: VectorReadiness,
    pub loaded_at_epoch_ms: u128,
    pub diagrams: usize,
    pub entities: usize,
    pub chunks: usize,
    pub embedding_model: String,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EntityResult {
    pub entity: GraphEntity,
    pub citation: Option<SourceCitation>,
    pub relations: Vec<GraphRelation>,
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub owner: GraphEntity,
    pub chunk: Option<DocumentChunk>,
    pub score: f64,
    pub graph_distance: Option<usize>,
    pub citation: Option<SourceCitation>,
    pub revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScopeCandidate {
    pub root: GraphEntity,
    pub score: f64,
    pub evidence: String,
    pub citation: Option<SourceCitation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ScopeResult {
    pub selected_root: Option<GraphEntity>,
    pub candidates: Vec<ScopeCandidate>,
    pub authorized: Vec<GraphEntity>,
    pub context_only: Vec<GraphEntity>,
    pub revision: String,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanScopeCandidate {
    pub entity: GraphEntity,
    pub citation: SourceCitation,
    pub breadcrumb: String,
    pub score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlanScopeEnvelope {
    pub root: GraphEntity,
    pub targets: Vec<GraphEntity>,
    pub context_only: Vec<GraphEntity>,
    pub excluded: Vec<GraphEntity>,
    pub relations: Vec<GraphRelation>,
    pub contract_outputs: Vec<GeneratedContractOutput>,
    pub scope_contract_revision: String,
    pub revision: String,
    pub source_manifest_revision: String,
}

#[derive(Debug, Clone)]
pub enum EmbeddingProfile {
    LocalOnnx { model: PathBuf, tokenizer: PathBuf },
    Preset,
    DeterministicTest,
    NonFiniteTest,
    FailingTest,
}

pub fn onnx_install_guidance() -> &'static str {
    if cfg!(target_os = "macos") {
        "ONNX Runtime is required only to generate Markdown embeddings.\nInstall it with:\n  brew install onnxruntime\n\nThe diagram editor and MCP text retrieval remain available without it."
    } else if cfg!(target_os = "windows") {
        "ONNX Runtime is required only to generate Markdown embeddings.\nInstall the Microsoft.ML.OnnxRuntime native package and place onnxruntime.dll beside .ss\\bin\\ss.exe.\nSee https://onnxruntime.ai/docs/get-started/with-c.html\n\nThe diagram editor and MCP text retrieval remain available without it."
    } else {
        "ONNX Runtime is required only to generate Markdown embeddings.\nInstall a compatible libonnxruntime.so and make it available through the system library path.\nSee https://onnxruntime.ai/docs/install/\n\nThe diagram editor and MCP text retrieval remain available without it."
    }
}

fn prepare_onnx_runtime() -> Result<PathBuf> {
    let library =
        locate_onnx_runtime().ok_or_else(|| Error::Message(onnx_install_guidance().into()))?;
    ort::init_from(&library)
        .map_err(|error| {
            Error::Message(format!(
                "Could not load ONNX Runtime from {}: {error}\n\n{}",
                library.display(),
                onnx_install_guidance()
            ))
        })?
        .commit();
    Ok(library)
}

fn locate_onnx_runtime() -> Option<PathBuf> {
    let filename = if cfg!(target_os = "windows") {
        "onnxruntime.dll"
    } else if cfg!(target_os = "macos") {
        "libonnxruntime.dylib"
    } else {
        "libonnxruntime.so"
    };
    let mut candidates = Vec::new();
    if let Ok(executable) = std::env::current_exe()
        && let Some(parent) = executable.parent()
    {
        candidates.push(parent.join(filename));
    }
    if cfg!(target_os = "macos") {
        candidates.extend([
            PathBuf::from("/opt/homebrew/opt/onnxruntime/lib").join(filename),
            PathBuf::from("/usr/local/opt/onnxruntime/lib").join(filename),
        ]);
    }
    // Windows installations deliberately keep ONNX Runtime beside ss.exe. Do
    // not load an arbitrary DLL from PATH: hosted runners and developer tools
    // may expose an incompatible copy whose failed initialization poisons
    // ONNX Runtime's process-global state.
    if !cfg!(target_os = "windows")
        && let Some(paths) = std::env::var_os("PATH")
    {
        candidates.extend(std::env::split_paths(&paths).map(|path| path.join(filename)));
    }
    candidates.into_iter().find(|path| path.is_file())
}

#[derive(Debug, Clone)]
pub struct LoadOptions {
    pub limits: GraphLimits,
    pub embedding: EmbeddingProfile,
    #[cfg(test)]
    pub refresh_delay: std::time::Duration,
}
impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            limits: GraphLimits::default(),
            embedding: EmbeddingProfile::Preset,
            #[cfg(test)]
            refresh_delay: std::time::Duration::ZERO,
        }
    }
}
impl LoadOptions {
    pub fn deterministic_test() -> Self {
        Self {
            embedding: EmbeddingProfile::DeterministicTest,
            ..Self::default()
        }
    }
}

#[derive(Debug)]
struct HashEmbedding;
impl EmbeddingModel for HashEmbedding {
    fn embed(&self, texts: &[&str]) -> grafeo::Result<Vec<Vec<f32>>> {
        Ok(texts
            .iter()
            .map(|text| {
                let mut vector = vec![0.0f32; 64];
                for token in tokenize(text) {
                    let hash = Sha256::digest(token.as_bytes());
                    let index = usize::from(hash[0]) % vector.len();
                    vector[index] += if hash[1] & 1 == 0 { 1.0 } else { -1.0 };
                }
                let norm = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
                if norm > 0.0 {
                    vector.iter_mut().for_each(|v| *v /= norm);
                }
                vector
            })
            .collect())
    }
    fn dimensions(&self) -> usize {
        64
    }
    fn name(&self) -> &str {
        "ssw-deterministic-test"
    }
}

#[derive(Debug)]
struct NonFiniteEmbedding;
impl EmbeddingModel for NonFiniteEmbedding {
    fn embed(&self, texts: &[&str]) -> grafeo::Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| vec![f32::NAN; 4]).collect())
    }
    fn dimensions(&self) -> usize {
        4
    }
    fn name(&self) -> &str {
        "ssw-non-finite-test"
    }
}

#[derive(Debug)]
struct FailingEmbedding;
impl EmbeddingModel for FailingEmbedding {
    fn embed(&self, _texts: &[&str]) -> grafeo::Result<Vec<Vec<f32>>> {
        Err(grafeo::Error::InvalidValue(
            "fixture embedding failure".into(),
        ))
    }
    fn dimensions(&self) -> usize {
        4
    }
    fn name(&self) -> &str {
        "ssw-failing-test"
    }
}

pub struct GraphSnapshot {
    pub db: GrafeoDB,
    pub summary: SnapshotSummary,
    pub entities: BTreeMap<String, GraphEntity>,
    pub relations: Vec<GraphRelation>,
    pub chunks: BTreeMap<String, DocumentChunk>,
    pub source_map: BTreeMap<String, SourceCitation>,
    pub source_hashes: BTreeMap<String, String>,
    pub source_manifest: SourceManifest,
    pub dependencies: BTreeMap<String, BTreeSet<String>>,
    pub reverse_dependencies: BTreeMap<String, BTreeSet<String>>,
    pub reuse: BuildReuseStats,
    parsed_diagrams: BTreeMap<String, ParsedDiagram>,
    internal_chunks: HashMap<NodeId, String>,
    embedding_cache: BTreeMap<(String, String), Vec<f32>>,
    embedding_name: String,
    embedding_profile: EmbeddingProfile,
    query_embedding_ready: OnceLock<std::result::Result<(), String>>,
    limits: GraphLimits,
    project_root: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BuildReuseStats {
    pub parsed_diagrams: usize,
    pub embeddings: usize,
}

impl std::fmt::Debug for GraphSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphSnapshot")
            .field("summary", &self.summary)
            .finish()
    }
}

impl GraphSnapshot {
    pub fn design_targets(&self) -> Result<(DesignTarget, Vec<DesignTarget>)> {
        let root_entity = self
            .entities
            .get(&self.summary.root_id)
            .ok_or_else(|| Error::Message("root schematic entity is unavailable".into()))?;
        let root_source = self.source_map.get(&root_entity.id);
        let root = DesignTarget {
            entity_ref: root_entity.id.clone(),
            source_id: root_entity.source_id.clone(),
            owner_name: root_entity.owner_name.clone(),
            name: root_entity.name.clone(),
            label: root_entity.label.clone(),
            diagram_path: root_source.map(|source| source.diagram.clone()),
            composition_state: CompositionState::Existing,
        };

        let mut diagrams_by_owner = BTreeMap::new();
        for entity in self
            .entities
            .values()
            .filter(|entity| entity.kind == EntityKind::Diagram && entity.element_type == "bpmn")
        {
            diagrams_by_owner.insert(entity.owner_name.clone(), entity);
        }

        let mut candidates = BTreeMap::<String, (bool, DesignTarget)>::new();
        for (path, diagram) in &self.parsed_diagrams {
            if diagram.kind != "cmmn" {
                continue;
            }
            for element in &diagram.elements {
                if !element
                    .element_type
                    .to_ascii_lowercase()
                    .ends_with("processtask")
                {
                    continue;
                }
                let Some(name) = element.name.as_ref().or(element.composition.as_ref()) else {
                    continue;
                };
                let existing = diagrams_by_owner.get(name).copied();
                let anchor = DesignTarget {
                    entity_ref: urn(
                        &self.summary.project_id,
                        "node",
                        &diagram.owner,
                        Some(&element.id),
                    ),
                    source_id: Some(element.id.clone()),
                    owner_name: diagram.owner.clone(),
                    name: Some(name.clone()),
                    label: if element.label.trim().is_empty() {
                        name.clone()
                    } else {
                        element.label.clone()
                    },
                    diagram_path: existing
                        .and_then(|entity| self.source_map.get(&entity.id))
                        .map(|source| source.diagram.clone())
                        .or_else(|| Some(path.clone())),
                    composition_state: if existing.is_some() {
                        CompositionState::Existing
                    } else {
                        CompositionState::NotCreated
                    },
                };
                let is_occurrence = element.definition_ref.is_some();
                let key = name.to_ascii_lowercase();
                if candidates
                    .get(&key)
                    .is_none_or(|(prior_occurrence, _)| is_occurrence && !prior_occurrence)
                {
                    candidates.insert(key, (is_occurrence, anchor));
                }
            }
        }

        for (owner, entity) in diagrams_by_owner {
            let key = owner.to_ascii_lowercase();
            if candidates.contains_key(&key) {
                continue;
            }
            let source = self.source_map.get(&entity.id);
            candidates.insert(
                key,
                (
                    false,
                    DesignTarget {
                        entity_ref: entity.id.clone(),
                        source_id: entity.source_id.clone(),
                        owner_name: entity.owner_name.clone(),
                        name: entity.name.clone(),
                        label: entity.label.clone(),
                        diagram_path: source.map(|source| source.diagram.clone()),
                        composition_state: CompositionState::Existing,
                    },
                ),
            );
        }

        let process_candidates = candidates
            .into_values()
            .map(|(_, target)| target)
            .take(MAX_DESIGN_TARGETS)
            .collect();
        Ok((root, process_candidates))
    }
}

#[derive(Debug, Clone)]
struct ParsedElement {
    id: String,
    element_type: String,
    label: String,
    name: Option<String>,
    status: ImplementationStatus,
    source: Option<String>,
    target: Option<String>,
    definition_ref: Option<String>,
    composition: Option<String>,
    attached_to: Option<String>,
    process_ref: Option<String>,
    parent: Option<String>,
    edge: bool,
}
#[derive(Debug, Clone)]
struct ParsedDiagram {
    owner: String,
    kind: String,
    source_id: String,
    elements: Vec<ParsedElement>,
}

pub fn load_schematic_graph(
    project: impl AsRef<Path>,
    options: LoadOptions,
) -> Result<GraphSnapshot> {
    load_schematic_graph_with_previous(project.as_ref(), options, None)
}

pub fn refresh_schematic_graph(
    project: impl AsRef<Path>,
    options: LoadOptions,
    previous: &GraphSnapshot,
) -> Result<GraphSnapshot> {
    load_schematic_graph_with_previous(project.as_ref(), options, Some(previous))
}

fn load_schematic_graph_with_previous(
    project: &Path,
    options: LoadOptions,
    previous: Option<&GraphSnapshot>,
) -> Result<GraphSnapshot> {
    let root = project.canonicalize()?;
    let schematics = root
        .join("schematics")
        .canonicalize()
        .map_err(|_| Error::Message("project has no confined schematics directory".into()))?;
    let anchor = schematics.join("main.cmmn");
    if !anchor.is_file() || !root.join(".ss").is_dir() {
        return Err(Error::Message(
            "project must contain .ss and schematics/main.cmmn".into(),
        ));
    }
    let project_name = root
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("project")
        .to_string();
    let project_id = fs::read_to_string(root.join(".ss/project-id"))
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| short_hash(project_name.as_bytes(), 24));
    let mut queue = VecDeque::from([PathBuf::from("main.cmmn")]);
    let mut visited = BTreeSet::new();
    let mut diagrams = Vec::new();
    let mut diagnostics = Vec::new();
    let mut skipped_elements = BTreeSet::new();
    let mut source_hashes = BTreeMap::new();
    let mut dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut parsed_diagrams = BTreeMap::new();
    let mut reuse = BuildReuseStats::default();
    while let Some(relative) = queue.pop_front() {
        let relative = confined_relative(&relative)?;
        if !visited.insert(relative.clone()) {
            continue;
        }
        if visited.len() > options.limits.max_diagrams {
            return Err(Error::Message("schematic diagram limit exceeded".into()));
        }
        let full = confined_existing(&schematics, &relative)?;
        let xml = fs::read_to_string(&full)
            .map_err(|e| Error::Message(format!("{}: {e}", relative.display())))?;
        let relative_key = relative.to_string_lossy().replace('\\', "/");
        let xml_hash = short_hash(xml.as_bytes(), 64);
        source_hashes.insert(relative_key.clone(), xml_hash.clone());
        let cached = previous
            .filter(|snapshot| snapshot.source_hashes.get(&relative_key) == Some(&xml_hash))
            .and_then(|snapshot| snapshot.parsed_diagrams.get(&relative_key))
            .cloned();
        let diagram = match cached {
            Some(diagram) => {
                reuse.parsed_diagrams += 1;
                diagram
            }
            None => parse_diagram(&xml, &relative)?,
        };
        parsed_diagrams.insert(relative_key.clone(), diagram.clone());
        for element in &diagram.elements {
            if let Some(name) = &element.composition {
                let target = PathBuf::from(name.replace('.', "/")).join("main.bpmn");
                if schematics.join(&target).is_file() {
                    dependencies
                        .entry(relative_key.clone())
                        .or_default()
                        .insert(target.to_string_lossy().replace('\\', "/"));
                    queue.push_back(target);
                } else {
                    skipped_elements.insert((relative.clone(), element.id.clone()));
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::GraphStructure,
                        level: "warning".into(),
                        message: format!(
                            "skipped node {} because referenced composition {name} does not exist",
                            element.id
                        ),
                        source: Some(relative.to_string_lossy().replace('\\', "/")),
                    });
                }
            }
        }
        diagrams.push((relative, diagram));
    }
    let discovered: BTreeSet<PathBuf> = WalkDir::new(&schematics)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let p = e.path();
            matches!(
                p.extension().and_then(|x| x.to_str()),
                Some("cmmn" | "bpmn")
            )
            .then(|| p.strip_prefix(&schematics).ok().map(Path::to_path_buf))
            .flatten()
        })
        .collect();
    for orphan in discovered.difference(&visited) {
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::GraphStructure,
            level: "info".into(),
            message: "diagram is not reachable from root".into(),
            source: Some(orphan.to_string_lossy().replace('\\', "/")),
        });
    }
    for (relative, diagram) in &diagrams {
        let element_ids: BTreeSet<_> = diagram
            .elements
            .iter()
            .map(|item| item.id.as_str())
            .collect();
        for element in &diagram.elements {
            if !element.edge || skipped_elements.contains(&(relative.clone(), element.id.clone())) {
                continue;
            }
            let missing_source = element
                .source
                .as_deref()
                .filter(|id| !element_ids.contains(id));
            let missing_target = element
                .target
                .as_deref()
                .filter(|id| !element_ids.contains(id));
            let skipped_source = element
                .source
                .as_ref()
                .is_some_and(|id| skipped_elements.contains(&(relative.clone(), id.clone())));
            let skipped_target = element
                .target
                .as_ref()
                .is_some_and(|id| skipped_elements.contains(&(relative.clone(), id.clone())));
            if missing_source.is_some()
                || missing_target.is_some()
                || skipped_source
                || skipped_target
            {
                skipped_elements.insert((relative.clone(), element.id.clone()));
                let reason = match (
                    missing_source,
                    missing_target,
                    skipped_source,
                    skipped_target,
                ) {
                    (Some(id), _, _, _) => format!("source endpoint {id} does not exist"),
                    (_, Some(id), _, _) => format!("target endpoint {id} does not exist"),
                    (_, _, true, _) => "source endpoint was skipped".into(),
                    _ => "target endpoint was skipped".into(),
                };
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::GraphStructure,
                    level: "warning".into(),
                    message: format!("skipped edge {} because {reason}", element.id),
                    source: Some(relative.to_string_lossy().replace('\\', "/")),
                });
            }
        }
    }
    let db = GrafeoDB::new_in_memory();
    let embedding_name = embedding_profile_name(&options.embedding);
    let default_dimensions = embedding_profile_dimensions(&options.embedding).unwrap_or(0);
    let mut entities = BTreeMap::new();
    let mut relations = Vec::new();
    let mut chunks = BTreeMap::new();
    let mut source_map = BTreeMap::new();
    let mut contract_chunk_sources = BTreeMap::<String, String>::new();
    let mut contract_documents_seen = BTreeSet::<String>::new();
    let mut source_to_urn = HashMap::new();
    let mut diagram_urns = HashMap::new();
    let mut revision_hasher = Sha256::new();
    let mut artifact_vectors = BTreeMap::<String, Vec<f32>>::new();
    let mut accepted_envelopes = Vec::<EmbeddingEnvelope>::new();
    for (relative, diagram) in &diagrams {
        let diagram_id = urn(&project_id, "diagram", &diagram.owner, None);
        diagram_urns.insert(relative.clone(), diagram_id.clone());
        let diagram_md_path = relative.parent().unwrap_or(Path::new("")).join("main.md");
        let parsed_markdown = read_optional_document(
            &schematics,
            &diagram_md_path,
            options.limits.max_document_bytes,
        )?;
        let markdown = parsed_markdown.body;
        if !markdown.is_empty() {
            source_hashes.insert(
                diagram_md_path.to_string_lossy().replace('\\', "/"),
                short_hash(markdown.as_bytes(), 64),
            );
        }
        revision_hasher.update(fs::read(schematics.join(relative))?);
        revision_hasher.update(markdown.as_bytes());
        let entity = GraphEntity {
            id: diagram_id.clone(),
            source_id: Some(diagram.source_id.clone()),
            kind: EntityKind::Diagram,
            element_type: diagram.kind.clone(),
            owner_name: diagram.owner.clone(),
            name: Some(diagram.owner.clone()),
            label: diagram.owner.clone(),
            implementation_status: None,
            development_scope_eligible: false,
            markdown: markdown.clone(),
            implementation_contract_markdown: None,
            implementation_contract: None,
            contract_drift: None,
        };
        if entities.contains_key(&diagram_id) {
            return Err(Error::Message(format!(
                "conflicting diagram identity {diagram_id}"
            )));
        }
        entities.insert(diagram_id.clone(), entity);
        source_map.insert(
            diagram_id.clone(),
            SourceCitation {
                diagram: relative.to_string_lossy().replace('\\', "/"),
                document: (!markdown.is_empty())
                    .then(|| diagram_md_path.to_string_lossy().replace('\\', "/")),
                source_id: Some(diagram.source_id.clone()),
            },
        );
        add_chunks(
            &project_id,
            &diagram_id,
            &markdown,
            &options.limits,
            &embedding_name,
            default_dimensions,
            "logical",
            &mut chunks,
        );
        collect_artifact_vectors(
            parsed_markdown.envelope,
            parsed_markdown.diagnostic,
            &markdown,
            &format!("{}#diagram", relative.to_string_lossy().replace('\\', "/")),
            &diagram_id,
            "logical",
            &embedding_name,
            embedding_profile_dimensions(&options.embedding),
            &chunks,
            &mut artifact_vectors,
            &mut accepted_envelopes,
            &mut diagnostics,
            &diagram_md_path,
        );
        for element in &diagram.elements {
            if skipped_elements.contains(&(relative.clone(), element.id.clone())) {
                continue;
            }
            let id = urn(
                &project_id,
                if element.edge { "edge" } else { "node" },
                &diagram.owner,
                Some(&element.id),
            );
            if entities.contains_key(&id) {
                return Err(Error::Message(format!("duplicate compiled identity {id}")));
            }
            let doc_path = relative
                .parent()
                .unwrap_or(Path::new(""))
                .join("docs")
                .join(format!("{}.md", element.id));
            let parsed_markdown =
                read_optional_document(&schematics, &doc_path, options.limits.max_document_bytes)?;
            let markdown = parsed_markdown.body;
            if !markdown.is_empty() {
                source_hashes.insert(
                    doc_path.to_string_lossy().replace('\\', "/"),
                    short_hash(markdown.as_bytes(), 64),
                );
            }
            revision_hasher.update(markdown.as_bytes());
            let contract_supported =
                element.edge || element.element_type.to_ascii_lowercase().contains("event");
            let contract_path = relative
                .parent()
                .unwrap_or(Path::new(""))
                .join("docs")
                .join(format!("{}-contract.md", element.id));
            let mut implementation_contract_markdown = None;
            let mut implementation_contract = None;
            let mut contract_authored_markdown = None;
            let mut contract_embedding_envelope = None;
            let mut contract_embedding_diagnostic = None;
            let mut contract_drift = contract_supported.then_some(ContractDriftState::Missing);
            if schematics.join(&contract_path).is_file() {
                contract_documents_seen.insert(contract_path.to_string_lossy().replace('\\', "/"));
                if !contract_supported {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::ImplementationContract,
                        level: "warning".into(),
                        message: format!(
                            "ignored implementation contract for unsupported owner {}",
                            element.id
                        ),
                        source: Some(contract_path.to_string_lossy().replace('\\', "/")),
                    });
                } else {
                    let physical = fs::read_to_string(schematics.join(&contract_path))?;
                    let parsed_contract = parse_markdown(&physical);
                    let authored = parsed_contract.body;
                    contract_embedding_envelope = parsed_contract.envelope;
                    contract_embedding_diagnostic = parsed_contract.diagnostic;
                    match parse_generated_contract(&authored, &project_id) {
                        Ok((metadata, body)) if metadata.entity_ref == id => {
                            source_hashes.insert(
                                contract_path.to_string_lossy().replace('\\', "/"),
                                short_hash(physical.as_bytes(), 64),
                            );
                            revision_hasher.update(physical.as_bytes());
                            contract_drift = Some(if body.trim() == markdown.trim() {
                                ContractDriftState::Matching
                            } else {
                                ContractDriftState::Differing
                            });
                            implementation_contract_markdown = Some(body);
                            implementation_contract = Some(metadata);
                            contract_authored_markdown = Some(authored);
                        }
                        Ok(_) => diagnostics.push(Diagnostic {
                            code: DiagnosticCode::ImplementationContract,
                            level: "warning".into(),
                            message: format!(
                                "ignored implementation contract whose entity identity does not match {}",
                                element.id
                            ),
                            source: Some(contract_path.to_string_lossy().replace('\\', "/")),
                        }),
                        Err(error) => diagnostics.push(Diagnostic {
                            code: DiagnosticCode::ImplementationContract,
                            level: "warning".into(),
                            message: error,
                            source: Some(contract_path.to_string_lossy().replace('\\', "/")),
                        }),
                    }
                }
            }
            let entity = GraphEntity {
                id: id.clone(),
                source_id: Some(element.id.clone()),
                kind: if element.edge {
                    EntityKind::DiagramEdge
                } else {
                    EntityKind::DiagramNode
                },
                element_type: element.element_type.clone(),
                owner_name: diagram.owner.clone(),
                name: element.name.clone(),
                label: element.label.clone(),
                implementation_status: Some(element.status),
                development_scope_eligible: element.status.eligible(),
                markdown: markdown.clone(),
                implementation_contract_markdown: implementation_contract_markdown.clone(),
                implementation_contract,
                contract_drift,
            };
            entities.insert(id.clone(), entity);
            source_to_urn.insert((relative.clone(), element.id.clone()), id.clone());
            source_map.insert(
                id.clone(),
                SourceCitation {
                    diagram: relative.to_string_lossy().replace('\\', "/"),
                    document: (!markdown.is_empty())
                        .then(|| doc_path.to_string_lossy().replace('\\', "/")),
                    source_id: Some(element.id.clone()),
                },
            );
            let containment_source = element
                .parent
                .as_ref()
                .and_then(|parent| source_to_urn.get(&(relative.clone(), parent.clone())))
                .cloned()
                .unwrap_or_else(|| diagram_id.clone());
            relations.push(GraphRelation {
                relation_type: "CONTAINS".into(),
                source: containment_source,
                target: id.clone(),
            });
            add_chunks(
                &project_id,
                &id,
                &markdown,
                &options.limits,
                &embedding_name,
                default_dimensions,
                "logical",
                &mut chunks,
            );
            collect_artifact_vectors(
                parsed_markdown.envelope,
                parsed_markdown.diagnostic,
                &markdown,
                &format!(
                    "{}#{}",
                    relative.to_string_lossy().replace('\\', "/"),
                    element.id
                ),
                &id,
                "logical",
                &embedding_name,
                embedding_profile_dimensions(&options.embedding),
                &chunks,
                &mut artifact_vectors,
                &mut accepted_envelopes,
                &mut diagnostics,
                &doc_path,
            );
            if let Some(contract_markdown) = contract_authored_markdown.as_deref() {
                let prior_chunks = chunks.keys().cloned().collect::<BTreeSet<_>>();
                add_chunks(
                    &project_id,
                    &id,
                    contract_markdown,
                    &options.limits,
                    &embedding_name,
                    default_dimensions,
                    "implementationContract",
                    &mut chunks,
                );
                for chunk_id in chunks.keys().filter(|id| !prior_chunks.contains(*id)) {
                    contract_chunk_sources.insert(
                        chunk_id.clone(),
                        contract_path.to_string_lossy().replace('\\', "/"),
                    );
                }
                collect_artifact_vectors(
                    contract_embedding_envelope,
                    contract_embedding_diagnostic,
                    contract_markdown,
                    &format!(
                        "{}#{}-contract",
                        relative.to_string_lossy().replace('\\', "/"),
                        element.id
                    ),
                    &id,
                    "implementationContract",
                    &embedding_name,
                    embedding_profile_dimensions(&options.embedding),
                    &chunks,
                    &mut artifact_vectors,
                    &mut accepted_envelopes,
                    &mut diagnostics,
                    &contract_path,
                );
            }
        }
    }
    for entry in WalkDir::new(&schematics)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
    {
        let Ok(relative) = entry.path().strip_prefix(&schematics) else {
            continue;
        };
        let relative = relative.to_string_lossy().replace('\\', "/");
        if relative.ends_with("-contract.md") && !contract_documents_seen.contains(&relative) {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::ImplementationContract,
                level: "warning".into(),
                message:
                    "ignored orphan implementation contract with no reachable edge/event owner"
                        .into(),
                source: Some(relative),
            });
        }
    }
    if entities.len() > options.limits.max_entities {
        return Err(Error::Message("schematic entity limit exceeded".into()));
    }
    for (relative, diagram) in &diagrams {
        for element in &diagram.elements {
            let Some(element_urn) = source_to_urn
                .get(&(relative.clone(), element.id.clone()))
                .cloned()
            else {
                continue;
            };
            if let Some(source) = &element.source {
                let Some(endpoint) = source_to_urn.get(&(relative.clone(), source.clone())) else {
                    continue;
                };
                relations.push(GraphRelation {
                    relation_type: "SOURCE".into(),
                    source: element_urn.clone(),
                    target: endpoint.clone(),
                });
            }
            if let Some(target) = &element.target {
                let Some(endpoint) = source_to_urn.get(&(relative.clone(), target.clone())) else {
                    continue;
                };
                relations.push(GraphRelation {
                    relation_type: "TARGET".into(),
                    source: element_urn.clone(),
                    target: endpoint.clone(),
                });
            }
            if let Some(name) = &element.composition {
                let target_path = PathBuf::from(name.replace('.', "/")).join("main.bpmn");
                if let Some(target) = diagram_urns.get(&target_path) {
                    relations.push(GraphRelation {
                        relation_type: "COMPOSES_TO".into(),
                        source: element_urn.clone(),
                        target: target.clone(),
                    });
                }
            }
            if let Some(attached_to) = &element.attached_to
                && let Some(target) = source_to_urn.get(&(relative.clone(), attached_to.clone()))
            {
                relations.push(GraphRelation {
                    relation_type: "ATTACHED_TO".into(),
                    source: element_urn.clone(),
                    target: target.clone(),
                });
            }
            if let Some(process_ref) = &element.process_ref
                && let Some(target) = source_to_urn.get(&(relative.clone(), process_ref.clone()))
            {
                relations.push(GraphRelation {
                    relation_type: "CONTAINS".into(),
                    source: element_urn,
                    target: target.clone(),
                });
            }
        }
    }
    let mut internal = HashMap::new();
    let mut urn_to_node = HashMap::new();
    for entity in entities.values() {
        let label = match entity.kind {
            EntityKind::Diagram => "Diagram",
            EntityKind::DiagramNode => "DiagramNode",
            EntityKind::DiagramEdge => "DiagramEdge",
            EntityKind::DocumentChunk => "DocumentChunk",
        };
        let n = db.create_node_with_props(&["SchematicEntity", label], entity_properties(entity));
        urn_to_node.insert(entity.id.clone(), n);
    }
    let mut embedding_cache = BTreeMap::new();
    let vector_dimensions = artifact_vectors.values().next().map(Vec::len);
    for chunk in chunks.values_mut() {
        let vector = artifact_vectors.get(&chunk.id).cloned();
        if let Some(value) = &vector {
            chunk.dimensions = value.len();
            let key = (embedding_name.clone(), chunk.content_hash.clone());
            if previous
                .and_then(|snapshot| snapshot.embedding_cache.get(&key))
                .is_some_and(|prior| prior == value)
            {
                reuse.embeddings += 1;
            }
            embedding_cache.insert(key, value.clone());
        }
        let mut properties = vec![
            ("id", Value::from(chunk.id.clone())),
            ("ownerId", Value::from(chunk.owner_id.clone())),
            ("markdown", Value::from(chunk.markdown.clone())),
            ("contentHash", Value::from(chunk.content_hash.clone())),
        ];
        if let Some(vector) = vector {
            properties.push(("embedding", Value::Vector(vector.into())));
        }
        let n = db.create_node_with_props(&["DocumentChunk"], properties);
        internal.insert(n, chunk.id.clone());
        if let Some(owner) = urn_to_node.get(&chunk.owner_id) {
            db.create_edge(*owner, n, "DOCUMENTED_BY");
        }
        let mut citation = source_map.get(&chunk.owner_id).cloned().unwrap();
        if let Some(document) = contract_chunk_sources.get(&chunk.id) {
            citation.document = Some(document.clone());
        }
        source_map.insert(chunk.id.clone(), citation);
    }
    for relation in &relations {
        if let (Some(a), Some(b)) = (
            urn_to_node.get(&relation.source),
            urn_to_node.get(&relation.target),
        ) {
            db.create_edge(*a, *b, &relation.relation_type);
        }
    }
    if !chunks.is_empty() {
        db.create_text_index("DocumentChunk", "markdown")
            .map_err(graph_error)?;
        if let Some(dimensions) = vector_dimensions {
            db.create_vector_index(
                "DocumentChunk",
                "embedding",
                Some(dimensions),
                Some("cosine"),
                None,
                None,
                None,
            )
            .map_err(graph_error)?;
        }
    }
    let revision = format!("sha256:{}", hex(&revision_hasher.finalize()));
    let source_manifest = build_source_manifest(&schematics)?;
    let envelope_refs = accepted_envelopes.iter().collect::<Vec<_>>();
    let embedding_revision = embedding_revision(&envelope_refs);
    let vector_ready = !chunks.is_empty() && artifact_vectors.len() == chunks.len();
    let vector_readiness = if chunks.is_empty() {
        VectorReadiness::NotApplicable
    } else if vector_ready {
        VectorReadiness::Current
    } else if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.is_vector() && !diagnostic.code.is_expected_pending())
    {
        VectorReadiness::Degraded
    } else {
        VectorReadiness::Pending
    };
    let root_id = diagram_urns
        .get(&PathBuf::from("main.cmmn"))
        .cloned()
        .unwrap();
    let summary = SnapshotSummary {
        project_name,
        project_id,
        root_id,
        revision,
        source_manifest_revision: source_manifest.revision.clone(),
        embedding_revision,
        retrieval_mode: if vector_ready { "hybrid" } else { "text" }.into(),
        vector_ready,
        vector_readiness,
        loaded_at_epoch_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        diagrams: diagrams.len(),
        entities: entities.len(),
        chunks: chunks.len(),
        embedding_model: embedding_name.clone(),
        diagnostics,
    };
    let mut reverse_dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (source, targets) in &dependencies {
        for target in targets {
            reverse_dependencies
                .entry(target.clone())
                .or_default()
                .insert(source.clone());
        }
    }
    Ok(GraphSnapshot {
        db,
        summary,
        entities,
        relations,
        chunks,
        source_map,
        source_hashes,
        source_manifest,
        dependencies,
        reverse_dependencies,
        reuse,
        parsed_diagrams,
        internal_chunks: internal,
        embedding_cache,
        embedding_name,
        embedding_profile: options.embedding,
        query_embedding_ready: OnceLock::new(),
        limits: options.limits,
        project_root: root,
    })
}

impl GraphSnapshot {
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }
    pub fn get_entity(
        &self,
        id: Option<&str>,
        owner: Option<&str>,
        source_id: Option<&str>,
        name: Option<&str>,
    ) -> Result<EntityResult> {
        let candidates: Vec<_> = self
            .entities
            .values()
            .filter(|e| {
                id.is_some_and(|v| e.id == v)
                    || name.is_some_and(|v| e.name.as_deref() == Some(v))
                    || source_id.is_some_and(|v| e.source_id.as_deref() == Some(v))
                        && owner.is_none_or(|o| e.owner_name == o)
            })
            .cloned()
            .collect();
        if candidates.is_empty() {
            return Err(Error::Message("entity not found".into()));
        }
        if candidates.len() > 1 {
            return Err(Error::Message(format!(
                "entity identity is ambiguous: {}",
                candidates
                    .iter()
                    .map(|e| e.id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        let entity = candidates[0].clone();
        Ok(EntityResult {
            relations: self
                .relations
                .iter()
                .filter(|r| r.source == entity.id || r.target == entity.id)
                .cloned()
                .collect(),
            citation: self.source_map.get(&entity.id).cloned(),
            revision: self.summary.revision.clone(),
            entity,
        })
    }
    pub fn neighbors(
        &self,
        id: &str,
        hops: usize,
        relation_types: &[String],
        direction: &str,
        kinds: &[EntityKind],
        limit: usize,
    ) -> Result<Vec<EntityResult>> {
        if !self.entities.contains_key(id) {
            return Err(Error::Message("entity not found".into()));
        }
        let hops = hops.min(self.limits.max_hops);
        let limit = limit.min(self.limits.max_results);
        let mut seen = BTreeSet::from([id.to_string()]);
        let mut frontier = vec![id.to_string()];
        for _ in 0..hops {
            let mut next = Vec::new();
            for current in frontier {
                for r in &self.relations {
                    if !relation_types.is_empty() && !relation_types.contains(&r.relation_type) {
                        continue;
                    }
                    let candidate = match direction {
                        "outgoing" if r.source == current => Some(&r.target),
                        "incoming" if r.target == current => Some(&r.source),
                        "both" | "" if r.source == current => Some(&r.target),
                        "both" | "" if r.target == current => Some(&r.source),
                        _ => None,
                    };
                    if let Some(v) = candidate
                        && seen.insert(v.clone())
                    {
                        next.push(v.clone());
                    }
                }
            }
            frontier = next;
        }
        seen.remove(id);
        Ok(seen
            .into_iter()
            .filter_map(|v| self.get_entity(Some(&v), None, None, None).ok())
            .filter(|result| kinds.is_empty() || kinds.contains(&result.entity.kind))
            .take(limit)
            .collect())
    }
    pub fn search(
        &self,
        query: &str,
        owner: Option<&str>,
        kinds: &[EntityKind],
        neighborhood: Option<&str>,
        max_distance: usize,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        let limit = limit.min(self.limits.max_results);
        if self.chunks.is_empty() {
            return Ok(Vec::new());
        }
        let query_vec = if self.summary.vector_ready {
            let ready = self.query_embedding_ready.get_or_init(|| {
                configure_embedding(&self.db, &self.project_root, &self.embedding_profile)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
            match ready {
                Ok(()) => Some(
                    self.db
                        .embed_text(&self.embedding_name, &[query])
                        .map_err(graph_error)?
                        .remove(0),
                ),
                Err(_) => None,
            }
        } else {
            None
        };
        let matches = self
            .db
            .hybrid_search(
                "DocumentChunk",
                "markdown",
                "embedding",
                query,
                query_vec.as_deref(),
                limit * 2,
                None,
            )
            .map_err(graph_error)?;
        let mut out = Vec::new();
        for (node, score) in matches {
            if let Some(chunk_id) = self.internal_chunks.get(&node)
                && let Some(chunk) = self.chunks.get(chunk_id)
                && let Some(entity) = self.entities.get(&chunk.owner_id)
                && owner.is_none_or(|o| entity.owner_name == o)
                && (kinds.is_empty() || kinds.contains(&entity.kind))
            {
                let graph_distance = neighborhood
                    .and_then(|root| self.graph_distance(root, &entity.id, max_distance));
                if neighborhood.is_some() && graph_distance.is_none() {
                    continue;
                }
                out.push(SearchResult {
                    owner: entity.clone(),
                    chunk: Some(chunk.clone()),
                    score,
                    graph_distance,
                    citation: self.source_map.get(chunk_id).cloned(),
                    revision: self.summary.revision.clone(),
                });
                if out.len() == limit {
                    break;
                }
            }
        }
        Ok(out)
    }

    fn graph_distance(&self, start: &str, target: &str, max: usize) -> Option<usize> {
        if start == target {
            return Some(0);
        }
        if !self.entities.contains_key(start) {
            return None;
        }
        let mut seen = BTreeSet::from([start.to_string()]);
        let mut frontier = vec![start.to_string()];
        for distance in 1..=max.min(self.limits.max_hops) {
            let mut next = Vec::new();
            for current in frontier {
                for relation in &self.relations {
                    let candidate = if relation.source == current {
                        Some(&relation.target)
                    } else if relation.target == current {
                        Some(&relation.source)
                    } else {
                        None
                    };
                    if let Some(candidate) = candidate {
                        if candidate == target {
                            return Some(distance);
                        }
                        if seen.insert(candidate.clone()) {
                            next.push(candidate.clone());
                        }
                    }
                }
            }
            frontier = next;
        }
        None
    }

    pub fn resolve_plan_selection(
        &self,
        diagram_path: &str,
        source_id: Option<&str>,
    ) -> Result<GraphEntity> {
        let mut matches = self
            .entities
            .values()
            .filter(|entity| {
                self.source_map
                    .get(&entity.id)
                    .is_some_and(|citation| citation.diagram == diagram_path)
                    && source_id.map_or(entity.kind == EntityKind::Diagram, |source_id| {
                        entity.source_id.as_deref() == Some(source_id)
                    })
            })
            .cloned()
            .collect::<Vec<_>>();
        matches.sort_by(|left, right| left.id.cmp(&right.id));
        match matches.as_slice() {
            [entity] => Ok(entity.clone()),
            [] => Err(Error::Message(
                "selected diagram entity is absent from the published graph".into(),
            )),
            _ => Err(Error::Message(
                "selected diagram identity is ambiguous in its semantic owner".into(),
            )),
        }
    }

    pub fn resolve_plan_intent(
        &self,
        language: &str,
        limit: usize,
    ) -> Result<Vec<PlanScopeCandidate>> {
        let query = language.trim().to_ascii_lowercase();
        if query.is_empty() {
            return Err(Error::Message("describe the diagram scope to plan".into()));
        }
        let terms = tokenize(&query);
        let mut candidates = self
            .entities
            .values()
            .filter(|entity| entity.kind != EntityKind::DocumentChunk)
            .filter_map(|entity| {
                let citation = self.source_map.get(&entity.id)?.clone();
                let haystack = format!(
                    "{} {} {} {} {}",
                    entity.label,
                    entity.name.as_deref().unwrap_or(""),
                    entity.owner_name,
                    entity.element_type,
                    entity.markdown
                )
                .to_ascii_lowercase();
                let exact = entity.label.eq_ignore_ascii_case(language)
                    || entity
                        .name
                        .as_deref()
                        .is_some_and(|name| name.eq_ignore_ascii_case(language));
                let term_hits = terms
                    .iter()
                    .filter(|term| haystack.contains(term.as_str()))
                    .count();
                if !exact && term_hits == 0 {
                    return None;
                }
                let score = if exact {
                    10_000
                } else {
                    (term_hits * 100) as u32
                } + u32::from(entity.development_scope_eligible) * 10
                    + u32::from(haystack.contains(&query));
                Some(PlanScopeCandidate {
                    entity: entity.clone(),
                    breadcrumb: format!("{} › {}", entity.owner_name, citation.diagram),
                    citation,
                    score,
                })
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.entity.id.cmp(&right.entity.id))
        });
        candidates.truncate(limit.min(self.limits.max_results));
        Ok(candidates)
    }

    pub fn plan_scope(&self, root_id: &str) -> Result<PlanScopeEnvelope> {
        let root = self.entities.get(root_id).cloned().ok_or_else(|| {
            Error::Message("plan scope root is not in the published graph".into())
        })?;
        let normalized_type = root.element_type.to_ascii_lowercase();
        let service_boundary = root.kind == EntityKind::Diagram
            || normalized_type.contains("participant")
            || normalized_type.contains("pool");
        let boundary = root.id.clone();

        let mut included = BTreeSet::from([root.id.clone(), boundary.clone()]);
        let mut frontier = vec![boundary.clone()];
        while let Some(current) = frontier.pop() {
            for relation in self.relations.iter().filter(|relation| {
                relation.source == current
                    && matches!(relation.relation_type.as_str(), "CONTAINS" | "COMPOSES_TO")
            }) {
                if included.insert(relation.target.clone()) {
                    frontier.push(relation.target.clone());
                }
            }
        }

        if !service_boundary {
            let focus_nodes = included.clone();
            for relation in &self.relations {
                let incident = matches!(relation.relation_type.as_str(), "SOURCE" | "TARGET")
                    && focus_nodes.contains(&relation.target);
                let attached = relation.relation_type == "ATTACHED_TO"
                    && focus_nodes.contains(&relation.target);
                if incident || attached {
                    included.insert(relation.source.clone());
                    for endpoint in self.relations.iter().filter(|endpoint| {
                        endpoint.source == relation.source
                            && matches!(endpoint.relation_type.as_str(), "SOURCE" | "TARGET")
                    }) {
                        included.insert(endpoint.target.clone());
                    }
                }
            }
            if root.kind == EntityKind::DiagramEdge {
                for relation in self.relations.iter().filter(|relation| {
                    relation.source == root.id
                        && matches!(relation.relation_type.as_str(), "SOURCE" | "TARGET")
                }) {
                    included.insert(relation.target.clone());
                }
            }
        }

        let mut targets = Vec::new();
        let mut context_only = Vec::new();
        let mut excluded = Vec::new();
        for id in &included {
            let Some(entity) = self.entities.get(id).cloned() else {
                continue;
            };
            if entity.development_scope_eligible {
                targets.push(entity);
            } else if entity.implementation_status.is_some() {
                excluded.push(entity.clone());
                context_only.push(entity);
            } else {
                context_only.push(entity);
            }
        }
        if targets.is_empty() {
            return Err(Error::Message(
                "selected scope contains no new or modify implementation targets".into(),
            ));
        }
        if targets.len() >= crate::delivery_protocol::MAX_BUILD_WORK_ITEMS {
            return Err(Error::Message(
                "selected scope exceeds the build-plan work-item limit".into(),
            ));
        }
        targets.sort_by(|left, right| left.id.cmp(&right.id));
        context_only.sort_by(|left, right| left.id.cmp(&right.id));
        excluded.sort_by(|left, right| left.id.cmp(&right.id));
        let mut relations = self
            .relations
            .iter()
            .filter(|relation| {
                included.contains(&relation.source) && included.contains(&relation.target)
            })
            .cloned()
            .collect::<Vec<_>>();
        relations.sort_by(|left, right| {
            (&left.source, &left.relation_type, &left.target).cmp(&(
                &right.source,
                &right.relation_type,
                &right.target,
            ))
        });
        let contract_outputs = targets
            .iter()
            .filter(|entity| {
                entity.kind == EntityKind::DiagramEdge
                    || entity.element_type.to_ascii_lowercase().contains("event")
            })
            .filter_map(|entity| {
                let source_id = entity.source_id.as_ref()?;
                let citation = self.source_map.get(&entity.id)?;
                let parent = Path::new(&citation.diagram)
                    .parent()
                    .unwrap_or(Path::new(""));
                Some(GeneratedContractOutput {
                    entity_ref: entity.id.clone(),
                    document_path: parent
                        .join("docs")
                        .join(format!("{source_id}-contract.md"))
                        .to_string_lossy()
                        .replace('\\', "/"),
                })
            })
            .collect::<Vec<_>>();
        let fingerprint = serde_json::to_vec(&(
            &root.id,
            targets
                .iter()
                .map(|entity| (&entity.id, entity.implementation_status, &entity.markdown))
                .collect::<Vec<_>>(),
            context_only
                .iter()
                .map(|entity| (&entity.id, entity.implementation_status, &entity.markdown))
                .collect::<Vec<_>>(),
            &relations,
            &contract_outputs,
        ))
        .map_err(|error| Error::Message(error.to_string()))?;
        Ok(PlanScopeEnvelope {
            root,
            targets,
            context_only,
            excluded,
            relations,
            contract_outputs,
            scope_contract_revision: format!("sha256:{}", hex(&Sha256::digest(fingerprint))),
            revision: self.summary.revision.clone(),
            source_manifest_revision: self.summary.source_manifest_revision.clone(),
        })
    }

    pub fn resolve_scope(
        &self,
        language: &str,
        explicit: Option<&str>,
        limit: usize,
    ) -> Result<ScopeResult> {
        let eligible: Vec<_> = self
            .entities
            .values()
            .filter(|e| e.development_scope_eligible)
            .cloned()
            .collect();
        if eligible.is_empty() {
            return Ok(ScopeResult {
                selected_root: None,
                candidates: vec![],
                authorized: vec![],
                context_only: vec![],
                revision: self.summary.revision.clone(),
                diagnostic: Some(
                    "No new or modify nodes are available; mark the intended diagram node first."
                        .into(),
                ),
            });
        }
        let mut candidates: Vec<ScopeCandidate> = if let Some(id) = explicit {
            eligible
                .iter()
                .filter(|e| e.id == id)
                .map(|e| ScopeCandidate {
                    root: e.clone(),
                    score: 1.0,
                    evidence: "explicit root".into(),
                    citation: self.source_map.get(&e.id).cloned(),
                })
                .collect()
        } else {
            let tokens = tokenize(language);
            let semantic: HashMap<String, f64> = self
                .search(language, None, &[], None, 0, self.limits.max_results)?
                .into_iter()
                .filter(|result| result.owner.development_scope_eligible)
                .fold(HashMap::new(), |mut scores, result| {
                    scores
                        .entry(result.owner.id)
                        .and_modify(|score| *score = score.max(result.score))
                        .or_insert(result.score);
                    scores
                });
            eligible
                .iter()
                .map(|e| {
                    let text = format!(
                        "{} {} {} {}",
                        e.owner_name,
                        e.name.as_deref().unwrap_or(""),
                        e.label,
                        e.markdown
                    );
                    let overlap = tokenize(&text).intersection(&tokens).count() as f64;
                    let lexical = overlap / (tokens.len().max(1) as f64);
                    let hybrid = semantic.get(&e.id).copied().unwrap_or(0.0);
                    // Grafeo's hybrid score is an RRF-style ranking signal rather than a
                    // probability, so use it as a bounded boost instead of rescaling exact terms.
                    let score = (lexical + hybrid.clamp(0.0, 1.0) * 0.25).min(1.0);
                    ScopeCandidate {
                        root: e.clone(),
                        score,
                        evidence: format!(
                            "{} matching terms; Grafeo hybrid {:.3}",
                            overlap as usize, hybrid
                        ),
                        citation: self.source_map.get(&e.id).cloned(),
                    }
                })
                .filter(|c| c.score > 0.0)
                .collect()
        };
        candidates.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.root.id.cmp(&b.root.id))
        });
        candidates.truncate(limit.min(self.limits.max_results));
        let selected = if explicit.is_some() {
            candidates.first().map(|c| c.root.clone())
        } else if candidates.first().is_some_and(|c| c.score >= 0.25)
            && (candidates.len() == 1 || candidates[0].score - candidates[1].score >= 0.15)
        {
            Some(candidates[0].root.clone())
        } else {
            None
        };
        if explicit.is_some() && selected.is_none() {
            return Err(Error::Message(
                "explicit root is absent or not new/modify".into(),
            ));
        }
        let mut authorized = Vec::new();
        let mut context = Vec::new();
        if let Some(root) = &selected {
            authorized.push(root.clone());
            for n in self.neighbors(
                &root.id,
                self.limits.max_hops,
                &[],
                "both",
                &[],
                self.limits.max_results,
            )? {
                if n.entity.development_scope_eligible {
                    authorized.push(n.entity)
                } else {
                    context.push(n.entity)
                }
            }
            authorized.sort_by(|a, b| a.id.cmp(&b.id));
            authorized.dedup_by(|a, b| a.id == b.id);
        }
        Ok(ScopeResult {
            selected_root: selected,
            candidates,
            authorized,
            context_only: context,
            revision: self.summary.revision.clone(),
            diagnostic: None,
        })
    }
}

fn configure_embedding(db: &GrafeoDB, root: &Path, profile: &EmbeddingProfile) -> Result<String> {
    match profile {
        EmbeddingProfile::DeterministicTest => {
            db.register_embedding_model("ssw-deterministic-test", Arc::new(HashEmbedding));
            Ok("ssw-deterministic-test".into())
        }
        EmbeddingProfile::NonFiniteTest => {
            db.register_embedding_model("ssw-non-finite-test", Arc::new(NonFiniteEmbedding));
            Ok("ssw-non-finite-test".into())
        }
        EmbeddingProfile::FailingTest => {
            db.register_embedding_model("ssw-failing-test", Arc::new(FailingEmbedding));
            Ok("ssw-failing-test".into())
        }
        EmbeddingProfile::LocalOnnx { model, tokenizer } => {
            db.load_embedding_model(EmbeddingModelConfig::Local {
                model_path: model.clone(),
                tokenizer_path: tokenizer.clone(),
            })
            .map_err(graph_error)?;
            Ok(model
                .file_stem()
                .and_then(|v| v.to_str())
                .unwrap_or("local-model")
                .into())
        }
        EmbeddingProfile::Preset => {
            prepare_onnx_runtime()?;
            let model = root.join(".ss/models/all-MiniLM-L6-v2/all-MiniLM-L6-v2.onnx");
            let tokenizer = root.join(".ss/models/all-MiniLM-L6-v2/tokenizer.json");
            if model.is_file() && tokenizer.is_file() {
                db.load_embedding_model(EmbeddingModelConfig::Local {
                    model_path: model,
                    tokenizer_path: tokenizer,
                })
                .map_err(graph_error)?;
            } else {
                db.load_embedding_model(EmbeddingModelConfig::MiniLmL6v2).map_err(|e|Error::Message(format!("local embedding model unavailable; install .ss/models/all-MiniLM-L6-v2: {e}")))?;
            }
            Ok(EMBEDDING_MODEL.into())
        }
    }
}

pub fn embedding_profile_name(profile: &EmbeddingProfile) -> String {
    match profile {
        EmbeddingProfile::DeterministicTest => "ssw-deterministic-test".into(),
        EmbeddingProfile::NonFiniteTest => "ssw-non-finite-test".into(),
        EmbeddingProfile::FailingTest => "ssw-failing-test".into(),
        EmbeddingProfile::LocalOnnx { model, .. } => model
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("local-model")
            .into(),
        EmbeddingProfile::Preset => EMBEDDING_MODEL.into(),
    }
}

fn embedding_profile_dimensions(profile: &EmbeddingProfile) -> Option<usize> {
    match profile {
        EmbeddingProfile::DeterministicTest => Some(64),
        EmbeddingProfile::NonFiniteTest | EmbeddingProfile::FailingTest => Some(4),
        EmbeddingProfile::Preset => Some(384),
        EmbeddingProfile::LocalOnnx { .. } => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_artifact_vectors(
    envelope: Option<EmbeddingEnvelope>,
    parse_diagnostic: Option<String>,
    body: &str,
    expected_owner: &str,
    graph_owner_id: &str,
    document_role: &str,
    expected_model: &str,
    expected_dimensions: Option<usize>,
    chunks: &BTreeMap<String, DocumentChunk>,
    vectors: &mut BTreeMap<String, Vec<f32>>,
    accepted: &mut Vec<EmbeddingEnvelope>,
    diagnostics: &mut Vec<Diagnostic>,
    source: &Path,
) {
    let source = source.to_string_lossy().replace('\\', "/");
    let reject = |code: DiagnosticCode, message: String, diagnostics: &mut Vec<Diagnostic>| {
        diagnostics.push(Diagnostic {
            code,
            level: "warning".into(),
            message,
            source: Some(source.clone()),
        });
    };
    if let Some(message) = parse_diagnostic {
        reject(
            DiagnosticCode::EmbeddingHeaderMalformed,
            message,
            diagnostics,
        );
        return;
    }
    if body.trim().is_empty() {
        return;
    }
    let Some(envelope) = envelope else {
        reject(
            DiagnosticCode::EmbeddingHeaderMissing,
            "embedding header is missing; vector retrieval is pending".into(),
            diagnostics,
        );
        return;
    };
    if let Err(error) = validate_envelope_shape(&envelope) {
        reject(
            DiagnosticCode::EmbeddingHeaderMalformed,
            error.to_string(),
            diagnostics,
        );
        return;
    }
    if envelope.owner != expected_owner
        || envelope.body_hash != body_hash(body)
        || envelope.model != expected_model
        || expected_dimensions.is_some_and(|value| value != envelope.dimensions)
    {
        reject(
            DiagnosticCode::EmbeddingHeaderStale,
            "embedding header is stale or belongs to another owner/model".into(),
            diagnostics,
        );
        return;
    }
    let mut owner_chunks = chunks
        .values()
        .filter(|chunk| chunk.owner_id == graph_owner_id && chunk.document_role == document_role)
        .collect::<Vec<_>>();
    owner_chunks.sort_by_key(|chunk| chunk.ordinal);
    if owner_chunks.len() != envelope.chunks.len() {
        reject(
            DiagnosticCode::EmbeddingHeaderChunkMismatch,
            "embedding header chunk count does not match authored Markdown".into(),
            diagnostics,
        );
        return;
    }
    for (chunk, artifact) in owner_chunks.iter().zip(&envelope.chunks) {
        if artifact.id != artifact_chunk_id(&chunk.content_hash, chunk.ordinal)
            || artifact.ordinal != chunk.ordinal
            || artifact.heading_path != chunk.heading_path
            || artifact.content_hash != chunk.content_hash
        {
            reject(
                DiagnosticCode::EmbeddingHeaderChunkMismatch,
                "embedding header chunks do not match authored Markdown".into(),
                diagnostics,
            );
            return;
        }
    }
    for artifact in &envelope.chunks {
        vectors.insert(artifact.id.clone(), artifact.vector.clone());
    }
    accepted.push(envelope);
}

fn artifact_chunk_id(content_hash: &str, ordinal: usize) -> String {
    format!("{content_hash}:{ordinal}")
}

pub fn derive_embedding_envelope(
    project: &Path,
    owner: &str,
    body: &str,
    options: &LoadOptions,
) -> Result<EmbeddingEnvelope> {
    let db = GrafeoDB::new_in_memory();
    let model = configure_embedding(&db, project, &options.embedding)?;
    let pieces = chunk_markdown(
        body,
        options.limits.chunk_chars,
        options.limits.chunk_overlap,
    );
    let texts = pieces
        .iter()
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>();
    let vectors = if texts.is_empty() {
        Vec::new()
    } else {
        db.embed_text(&model, &texts).map_err(graph_error)?
    };
    let dimensions = vectors
        .first()
        .map(Vec::len)
        .unwrap_or_else(|| embedding_profile_dimensions(&options.embedding).unwrap_or(0));
    if dimensions == 0 {
        return Err(Error::Message(
            "embedding model returned zero dimensions".into(),
        ));
    }
    let mut chunks = Vec::with_capacity(pieces.len());
    for (ordinal, ((heading_path, text), vector)) in pieces.into_iter().zip(vectors).enumerate() {
        if vector.len() != dimensions || vector.iter().any(|value| !value.is_finite()) {
            return Err(Error::Message(
                "embedding model returned invalid vector".into(),
            ));
        }
        let content_hash = short_hash(text.as_bytes(), 32);
        chunks.push(EmbeddingChunk {
            id: artifact_chunk_id(&content_hash, ordinal),
            content_hash,
            ordinal,
            heading_path,
            vector,
        });
    }
    let envelope = EmbeddingEnvelope {
        schema: crate::embedding_document::EMBEDDING_SCHEMA.into(),
        owner: owner.into(),
        body_hash: body_hash(body),
        chunker: CHUNKER_VERSION.into(),
        model,
        dimensions,
        chunks,
    };
    validate_envelope_shape(&envelope)?;
    Ok(envelope)
}
fn graph_error(error: impl std::fmt::Display) -> Error {
    Error::Message(format!("Grafeo: {error}"))
}
fn entity_properties(entity: &GraphEntity) -> Vec<(&'static str, Value)> {
    vec![
        ("id", entity.id.clone().into()),
        (
            "sourceId",
            entity.source_id.clone().unwrap_or_default().into(),
        ),
        ("kind", format!("{:?}", entity.kind).into()),
        ("type", entity.element_type.clone().into()),
        ("ownerName", entity.owner_name.clone().into()),
        ("name", entity.name.clone().unwrap_or_default().into()),
        ("label", entity.label.clone().into()),
        (
            "implementationStatus",
            entity
                .implementation_status
                .map(ImplementationStatus::as_str)
                .unwrap_or("")
                .into(),
        ),
        (
            "developmentScopeEligible",
            entity.development_scope_eligible.into(),
        ),
        ("markdown", entity.markdown.clone().into()),
        (
            "implementationContractMarkdown",
            entity
                .implementation_contract_markdown
                .clone()
                .unwrap_or_default()
                .into(),
        ),
        (
            "contractDrift",
            entity
                .contract_drift
                .map(|state| format!("{state:?}").to_ascii_lowercase())
                .unwrap_or_default()
                .into(),
        ),
    ]
}
fn parse_diagram(xml: &str, path: &Path) -> Result<ParsedDiagram> {
    let kind = path
        .extension()
        .and_then(|v| v.to_str())
        .unwrap_or("")
        .to_string();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut raw = Vec::new();
    let mut diagram_id = None;
    let mut owner = None;
    let mut depth = 0usize;
    let mut element_stack: Vec<Option<String>> = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                depth += 1;
                let parent = element_stack.iter().rev().flatten().next().cloned();
                let parsed = parse_xml_element(&e, &mut diagram_id, &mut owner, &mut raw, parent)?;
                element_stack.push(parsed);
            }
            Ok(Event::Empty(e)) => {
                let parent = element_stack.iter().rev().flatten().next().cloned();
                parse_xml_element(&e, &mut diagram_id, &mut owner, &mut raw, parent)?;
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
                element_stack.pop();
            }
            Ok(Event::Eof) if depth != 0 => {
                return Err(Error::Message(format!(
                    "{}: malformed XML: unclosed element",
                    path.display()
                )));
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(Error::Message(format!(
                    "{}: malformed XML: {e}",
                    path.display()
                )));
            }
            _ => {}
        }
    }
    let lookup: HashMap<_, _> = raw.iter().map(|e| (e.id.clone(), e.clone())).collect();
    for element in &mut raw {
        if let Some(reference) = &element.definition_ref
            && let Some(def) = lookup.get(reference)
        {
            element.element_type = def.element_type.clone();
            if element.label.is_empty() {
                element.label = def.label.clone()
            }
            if element.name.is_none() {
                element.name = def.name.clone()
            }
            element.status = if element.status == ImplementationStatus::Open {
                def.status
            } else {
                element.status
            };
        }
        let normalized_type = element.element_type.to_ascii_lowercase();
        let reusable = normalized_type.ends_with("processtask")
            || normalized_type.ends_with("callactivity")
            || normalized_type.ends_with("subprocess");
        if reusable && element.composition.is_none() {
            element.composition = element
                .name
                .clone()
                .map(|v| v.split('#').next().unwrap_or(&v).to_string());
        }
    }
    let owner = owner.unwrap_or_else(|| owner_from_path(path));
    Ok(ParsedDiagram {
        owner,
        kind,
        source_id: diagram_id.unwrap_or_else(|| "definitions".into()),
        elements: raw,
    })
}
fn parse_xml_element(
    e: &quick_xml::events::BytesStart<'_>,
    diagram_id: &mut Option<String>,
    owner: &mut Option<String>,
    raw: &mut Vec<ParsedElement>,
    parent: Option<String>,
) -> Result<Option<String>> {
    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
    let attributes = attrs(e)?;
    if tag.ends_with("definitions") {
        *diagram_id = attributes.get("id").cloned();
        *owner = attributes
            .get("processName")
            .or_else(|| attributes.get("packageName"))
            .cloned();
    }
    if let Some(id) = attributes.get("id") {
        if tag.ends_with("definitions")
            || tag.ends_with("Bounds")
            || tag.ends_with("waypoint")
            || tag.ends_with("CMMNShape")
            || tag.ends_with("CMMNEdge")
            || tag.ends_with("BPMNShape")
            || tag.ends_with("BPMNEdge")
        {
            return Ok(None);
        }
        let edge = attributes.contains_key("sourceRef")
            || attributes.contains_key("targetRef")
            || tag.contains("Flow")
            || tag.ends_with("Association");
        raw.push(ParsedElement {
            id: id.clone(),
            element_type: tag,
            label: attributes.get("name").cloned().unwrap_or_default(),
            name: attributes.get("architecturalName").cloned(),
            status: ImplementationStatus::parse(
                attributes.get("implementationStatus").map(String::as_str),
            ),
            source: attributes.get("sourceRef").cloned(),
            target: attributes.get("targetRef").cloned(),
            definition_ref: attributes.get("definitionRef").cloned(),
            composition: attributes.get("calledElement").cloned(),
            attached_to: attributes.get("attachedToRef").cloned(),
            process_ref: attributes.get("processRef").cloned(),
            parent,
            edge,
        });
        return Ok(Some(id.clone()));
    }
    Ok(None)
}
fn attrs(e: &quick_xml::events::BytesStart<'_>) -> Result<HashMap<String, String>> {
    let mut out = HashMap::new();
    for a in e.attributes() {
        let a = a.map_err(|v| Error::Message(format!("invalid XML attribute: {v}")))?;
        let key = String::from_utf8_lossy(a.key.as_ref())
            .split(':')
            .next_back()
            .unwrap_or("")
            .to_string();
        let value = a
            .unescape_value()
            .map_err(|v| Error::Message(format!("invalid XML attribute: {v}")))?
            .into_owned();
        out.insert(key, value);
    }
    Ok(out)
}
fn owner_from_path(path: &Path) -> String {
    if path == Path::new("main.cmmn") || path == Path::new("main.bpmn") {
        return "root".into();
    }
    path.parent()
        .unwrap_or(Path::new(""))
        .iter()
        .filter_map(|v| v.to_str())
        .collect::<Vec<_>>()
        .join(".")
}
fn confined_relative(path: &Path) -> Result<PathBuf> {
    if path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(Error::Message(
            "schematic path must remain relative and confined".into(),
        ));
    }
    Ok(path.to_path_buf())
}
fn confined_existing(root: &Path, relative: &Path) -> Result<PathBuf> {
    let full = root.join(confined_relative(relative)?).canonicalize()?;
    if !full.starts_with(root) {
        return Err(Error::Message("schematic path escapes project".into()));
    }
    Ok(full)
}
fn read_optional_document(root: &Path, relative: &Path, max: usize) -> Result<ParsedMarkdown> {
    let path = root.join(relative);
    if !path.exists() {
        return Ok(ParsedMarkdown {
            body: String::new(),
            envelope: None,
            diagnostic: None,
        });
    }
    let full = path.canonicalize()?;
    if !full.starts_with(root) {
        return Err(Error::Message("document path escapes schematics".into()));
    }
    let bytes = fs::read(full)?;
    if bytes.len() > max {
        return Err(Error::Message(format!("document exceeds {max} bytes")));
    }
    let physical =
        String::from_utf8(bytes).map_err(|_| Error::Message("Markdown must be UTF-8".into()))?;
    Ok(parse_markdown(&physical))
}
#[allow(clippy::too_many_arguments)]
fn add_chunks(
    project: &str,
    owner: &str,
    markdown: &str,
    limits: &GraphLimits,
    model: &str,
    dimensions: usize,
    document_role: &str,
    out: &mut BTreeMap<String, DocumentChunk>,
) {
    for (ordinal, (headings, text)) in
        chunk_markdown(markdown, limits.chunk_chars, limits.chunk_overlap)
            .into_iter()
            .enumerate()
    {
        let hash = short_hash(text.as_bytes(), 32);
        let chunk_namespace = if document_role == "logical" {
            owner.to_string()
        } else {
            format!("{owner}:{document_role}")
        };
        let id = format!(
            "urn:ssw:{project}:chunk:{}#{hash}:{ordinal}",
            short_hash(chunk_namespace.as_bytes(), 16)
        );
        out.insert(
            id.clone(),
            DocumentChunk {
                id,
                owner_id: owner.into(),
                heading_path: headings,
                ordinal,
                markdown: text,
                content_hash: hash,
                embedding_model: model.into(),
                dimensions,
                document_role: document_role.into(),
            },
        );
    }
}
pub fn chunk_markdown(markdown: &str, max: usize, overlap: usize) -> Vec<(Vec<String>, String)> {
    if markdown.trim().is_empty() {
        return vec![];
    }
    let mut sections = Vec::new();
    let mut headings = Vec::new();
    let mut current = String::new();
    let mut current_headings = Vec::new();
    for line in markdown.lines() {
        if let Some(level) = line
            .chars()
            .take_while(|c| *c == '#')
            .count()
            .checked_sub(0)
            .filter(|n| *n > 0 && *n <= 6)
            && line.chars().nth(level) == Some(' ')
        {
            if !current.trim().is_empty() {
                sections.push((current_headings.clone(), current.trim().to_string()));
                current.clear()
            }
            headings.truncate(level - 1);
            headings.push(line[level + 1..].trim().into());
            current_headings = headings.clone();
        }
        current.push_str(line);
        current.push('\n');
    }
    if !current.trim().is_empty() {
        sections.push((current_headings, current.trim().to_string()));
    }
    let mut out = Vec::new();
    for (h, text) in sections {
        let chars: Vec<char> = text.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            let end = (start + max).min(chars.len());
            out.push((h.clone(), chars[start..end].iter().collect()));
            if end == chars.len() {
                break;
            }
            start = end.saturating_sub(overlap.min(max.saturating_sub(1)));
        }
    }
    out
}

fn build_source_manifest(schematics: &Path) -> Result<SourceManifest> {
    let mut documents = BTreeMap::new();
    for entry in WalkDir::new(schematics).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            return Err(Error::Message(
                "source manifest does not follow symbolic links".into(),
            ));
        }
        if !entry.file_type().is_file()
            || !matches!(
                entry.path().extension().and_then(|value| value.to_str()),
                Some("bpmn" | "cmmn" | "md")
            )
        {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(schematics)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let physical = fs::read(entry.path())?;
        let authored = if relative.ends_with(".md") {
            let text = String::from_utf8(physical)
                .map_err(|_| Error::Message(format!("{relative}: Markdown must be UTF-8")))?;
            parse_markdown(&text).body.into_bytes()
        } else {
            physical
        };
        documents.insert(
            relative,
            format!("sha256:{}", hex(&Sha256::digest(&authored))),
        );
    }
    let mut digest = Sha256::new();
    for (path, revision) in &documents {
        digest.update(path.as_bytes());
        digest.update([0]);
        digest.update(revision.as_bytes());
        digest.update([0]);
    }
    Ok(SourceManifest {
        protocol_version: DELIVERY_PROTOCOL_VERSION.into(),
        revision: format!("sha256:{}", hex(&digest.finalize())),
        documents,
    })
}

fn urn(project: &str, kind: &str, owner: &str, source: Option<&str>) -> String {
    match source {
        Some(id) => format!("urn:ssw:{project}:{owner}#{id}"),
        None => format!("urn:ssw:{project}:{kind}:{owner}"),
    }
}
fn short_hash(bytes: &[u8], length: usize) -> String {
    let digest = Sha256::digest(bytes);
    hex(&digest)[..length].to_string()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn tokenize(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|v| v.len() > 1)
        .map(str::to_ascii_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn chunks_are_stable_and_bounded() {
        let v = chunk_markdown("# A\n\nhello world\n\n## B\nmore text", 12, 2);
        assert!(v.iter().all(|(_, t)| t.chars().count() <= 12));
        assert_eq!(v[0].0, vec!["A"]);
    }
    #[test]
    fn urn_uses_owner_and_source_id() {
        assert_eq!(
            urn("p", "node", "sales.Order", Some("Task_1")),
            "urn:ssw:p:sales.Order#Task_1"
        );
    }

    #[test]
    fn compiles_reachable_compositions_documents_edges_and_scope() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::write(directory.path().join(".ss/project-id"), "project-123\n").unwrap();
        let schematics = directory.path().join("schematics");
        fs::create_dir_all(schematics.join("acme/Build/docs")).unwrap();
        fs::create_dir_all(schematics.join("docs")).unwrap();
        fs::write(
            schematics.join("main.md"),
            "# Project\nBuild ordering tools.",
        )
        .unwrap();
        fs::write(
            schematics.join("docs/Plan_1.md"),
            "# Build orders\nCreate the order workflow.",
        )
        .unwrap();
        fs::write(schematics.join("main.cmmn"), r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="RootDefs" ssw:packageName="acme"><cmmn:processTask id="Def_1" ssw:architecturalName="acme.Build"/><cmmn:planItem id="Plan_1" definitionRef="Def_1" ssw:implementationStatus="new"/></cmmn:definitions>"#).unwrap();
        fs::write(
            schematics.join("acme/Build/main.md"),
            "# Ordering\nModify checkout ordering.",
        )
        .unwrap();
        fs::write(
            schematics.join("acme/Build/docs/Task_1.md"),
            "# Checkout\nValidate and submit an order.",
        )
        .unwrap();
        fs::write(schematics.join("acme/Build/main.bpmn"), r#"<bpmn:definitions xmlns:bpmn="x" xmlns:ssw="y" id="BuildDefs" ssw:processName="acme.Build"><bpmn:process id="Process_1"><bpmn:startEvent id="Start_1"/><bpmn:task id="Task_1" name="Checkout" ssw:architecturalName="acme.Build#checkout" ssw:implementationStatus="modify"/><bpmn:sequenceFlow id="Flow_1" sourceRef="Start_1" targetRef="Task_1"/></bpmn:process></bpmn:definitions>"#).unwrap();
        let contract_body = "# Implemented flow\nUses the durable order message.";
        let contract = crate::delivery_protocol::serialize_generated_contract(
            &crate::delivery_protocol::GeneratedContractMetadata {
                schema_version: crate::delivery_protocol::GENERATED_CONTRACT_SCHEMA_VERSION.into(),
                project_id: "project-123".into(),
                plan_id: "plan-one".into(),
                scope_ref: "urn:ssw:project-123:acme.Build#Task_1".into(),
                semantic_version: crate::delivery_protocol::SemanticVersion::new(1, 0, 0),
                entity_ref: "urn:ssw:project-123:acme.Build#Flow_1".into(),
                base_graph_revision: "sha256:base".into(),
                body_hash: format!("{:x}", Sha256::digest(contract_body.as_bytes())),
            },
            "project-123",
            contract_body,
        )
        .unwrap();
        fs::write(
            schematics.join("acme/Build/docs/Flow_1-contract.md"),
            contract,
        )
        .unwrap();
        fs::write(
            schematics.join("orphan.bpmn"),
            "<definitions id=\"orphan\"/>",
        )
        .unwrap();

        let graph =
            load_schematic_graph(directory.path(), LoadOptions::deterministic_test()).unwrap();
        assert_eq!(graph.summary.project_id, "project-123");
        assert_eq!(graph.summary.diagrams, 2);
        assert!(
            graph
                .summary
                .diagnostics
                .iter()
                .any(|d| d.message.contains("not reachable"))
        );
        let task = graph
            .get_entity(None, Some("acme.Build"), Some("Task_1"), None)
            .unwrap();
        assert_eq!(task.entity.id, "urn:ssw:project-123:acme.Build#Task_1");
        assert_eq!(
            task.entity.implementation_status,
            Some(ImplementationStatus::Modify)
        );
        assert_eq!(
            task.citation.unwrap().document.as_deref(),
            Some("acme/Build/docs/Task_1.md")
        );
        let exact = graph
            .resolve_plan_selection("acme/Build/main.bpmn", Some("Task_1"))
            .unwrap();
        assert_eq!(exact.id, task.entity.id);
        assert!(
            graph
                .resolve_plan_selection("main.cmmn", Some("Task_1"))
                .is_err()
        );
        let plan_scope = graph.plan_scope(&task.entity.id).unwrap();
        assert_eq!(
            plan_scope
                .targets
                .iter()
                .map(|entity| entity.id.as_str())
                .collect::<Vec<_>>(),
            vec![task.entity.id.as_str()]
        );
        assert!(
            plan_scope
                .context_only
                .iter()
                .any(|entity| entity.source_id.as_deref() == Some("Flow_1"))
        );
        let flow = graph
            .get_entity(None, Some("acme.Build"), Some("Flow_1"), None)
            .unwrap();
        assert_eq!(
            flow.entity.contract_drift,
            Some(ContractDriftState::Differing)
        );
        assert_eq!(
            flow.entity.implementation_contract_markdown.as_deref(),
            Some(contract_body)
        );
        assert!(
            graph
                .relations
                .iter()
                .any(|r| r.relation_type == "COMPOSES_TO")
        );
        let (root, processes) = graph.design_targets().unwrap();
        assert_eq!(root.diagram_path.as_deref(), Some("main.cmmn"));
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].name.as_deref(), Some("acme.Build"));
        assert_eq!(processes[0].source_id.as_deref(), Some("Plan_1"));
        assert_eq!(processes[0].composition_state, CompositionState::Existing);
        assert_eq!(
            processes[0].diagram_path.as_deref(),
            Some("acme/Build/main.bpmn")
        );
        assert!(
            processes
                .iter()
                .all(|target| { target.diagram_path.as_deref() != Some("orphan.bpmn") })
        );
        let composed_scope = graph.plan_scope(&processes[0].entity_ref).unwrap();
        assert!(
            composed_scope
                .targets
                .iter()
                .any(|entity| entity.source_id.as_deref() == Some("Plan_1"))
        );
        assert!(
            composed_scope
                .targets
                .iter()
                .any(|entity| entity.source_id.as_deref() == Some("Task_1"))
        );
        assert!(graph.relations.iter().any(|r| r.relation_type == "TARGET"));
        assert!(
            !graph
                .search("checkout order", None, &[], None, 0, 5)
                .unwrap()
                .is_empty()
        );
        let scope = graph
            .resolve_scope("improve checkout order submission", None, 5)
            .unwrap();
        assert!(
            scope
                .candidates
                .iter()
                .all(|c| c.root.development_scope_eligible)
        );
        assert!(
            scope
                .candidates
                .iter()
                .any(|c| c.root.source_id.as_deref() == Some("Task_1"))
        );
        assert!(
            graph
                .resolve_scope("anything", Some(&task.entity.id), 5)
                .unwrap()
                .selected_root
                .is_some()
        );
        let incoming = graph
            .neighbors(&task.entity.id, 1, &["TARGET".into()], "incoming", &[], 10)
            .unwrap();
        assert_eq!(incoming.len(), 1);
        let filtered = graph
            .search(
                "checkout order",
                Some("acme.Build"),
                &[EntityKind::DiagramNode],
                Some(&task.entity.id),
                1,
                5,
            )
            .unwrap();
        assert!(
            filtered
                .iter()
                .all(|result| result.owner.kind == EntityKind::DiagramNode)
        );
        assert!(
            filtered
                .iter()
                .all(|result| result.graph_distance == Some(0))
        );
    }

    #[test]
    fn design_targets_include_named_uncreated_processes() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::write(directory.path().join(".ss/project-id"), "project-123\n").unwrap();
        let schematics = directory.path().join("schematics");
        fs::create_dir_all(&schematics).unwrap();
        fs::write(
            schematics.join("main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="RootDefs" ssw:packageName="acme"><cmmn:processTask id="Def_1" name="Build" ssw:architecturalName="acme.Build"/><cmmn:planItem id="Plan_1" definitionRef="Def_1"/></cmmn:definitions>"#,
        )
        .unwrap();

        let graph =
            load_schematic_graph(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let (_, processes) = graph.design_targets().unwrap();
        assert_eq!(processes.len(), 1);
        assert_eq!(processes[0].source_id.as_deref(), Some("Plan_1"));
        assert_eq!(processes[0].name.as_deref(), Some("acme.Build"));
        assert_eq!(processes[0].composition_state, CompositionState::NotCreated);
        assert_eq!(processes[0].diagram_path.as_deref(), Some("main.cmmn"));
        assert_eq!(processes[0].entity_ref, "urn:ssw:project-123:acme#Plan_1");
    }

    #[test]
    fn participant_scope_does_not_include_a_sibling_service() {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::write(directory.path().join(".ss/project-id"), "project-123\n").unwrap();
        let schematics = directory.path().join("schematics");
        fs::create_dir_all(schematics.join("shop/Pools")).unwrap();
        fs::write(
            schematics.join("main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:processTask id="Pools" ssw:architecturalName="shop.Pools" ssw:implementationStatus="new"/></cmmn:definitions>"#,
        )
        .unwrap();
        fs::write(
            schematics.join("shop/Pools/main.bpmn"),
            r#"<bpmn:definitions xmlns:bpmn="x" xmlns:ssw="y" id="Defs" ssw:processName="shop.Pools"><bpmn:process id="Process_A"><bpmn:task id="Task_A" ssw:implementationStatus="new"/></bpmn:process><bpmn:process id="Process_B"><bpmn:task id="Task_B" ssw:implementationStatus="new"/></bpmn:process><bpmn:collaboration id="Collab"><bpmn:participant id="Pool_A" processRef="Process_A"/><bpmn:participant id="Pool_B" processRef="Process_B"/></bpmn:collaboration></bpmn:definitions>"#,
        )
        .unwrap();
        let graph =
            load_schematic_graph(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let pool = graph
            .resolve_plan_selection("shop/Pools/main.bpmn", Some("Pool_A"))
            .unwrap();
        let scope = graph.plan_scope(&pool.id).unwrap();
        assert!(
            scope
                .targets
                .iter()
                .any(|entity| entity.source_id.as_deref() == Some("Task_A"))
        );
        assert!(
            !scope
                .targets
                .iter()
                .any(|entity| entity.source_id.as_deref() == Some("Task_B"))
        );
    }

    #[test]
    fn natural_plan_resolution_returns_duplicate_labels_deterministically() {
        let directory = project_with_root(
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:task id="Checkout_A" name="Checkout" ssw:implementationStatus="new"/><cmmn:task id="Checkout_B" name="Checkout" ssw:implementationStatus="modify"/></cmmn:definitions>"#,
        );
        let graph =
            load_schematic_graph(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let candidates = graph.resolve_plan_intent("Checkout", 8).unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].score, candidates[1].score);
        assert!(candidates[0].entity.id < candidates[1].entity.id);
    }

    fn project_with_root(xml: &str) -> tempfile::TempDir {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::create_dir_all(directory.path().join("schematics")).unwrap();
        fs::write(directory.path().join(".ss/project-id"), "fixture\n").unwrap();
        fs::write(directory.path().join("schematics/main.cmmn"), xml).unwrap();
        directory
    }

    #[test]
    fn complete_load_produces_a_deterministic_content_addressed_source_manifest() {
        let project = project_with_root("<definitions id=\"D\"/>");
        let schematics = project.path().join("schematics");
        fs::write(schematics.join("main.md"), "# Contract\nStable bytes.\n").unwrap();
        fs::create_dir_all(schematics.join("docs")).unwrap();
        fs::write(schematics.join("docs/Empty.md"), "").unwrap();

        let first =
            load_schematic_graph(project.path(), LoadOptions::deterministic_test()).unwrap();
        let same = fs::read(schematics.join("main.md")).unwrap();
        fs::write(schematics.join("main.md"), same).unwrap();
        let second =
            load_schematic_graph(project.path(), LoadOptions::deterministic_test()).unwrap();

        assert_eq!(first.source_manifest, second.source_manifest);
        assert_eq!(first.summary.revision, second.summary.revision);
        assert_eq!(
            first.summary.source_manifest_revision,
            first.source_manifest.revision
        );
        assert_eq!(
            first.source_manifest.protocol_version,
            DELIVERY_PROTOCOL_VERSION
        );
        assert_eq!(
            first
                .source_manifest
                .documents
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            vec!["docs/Empty.md", "main.cmmn", "main.md"]
        );
        assert!(
            first
                .source_manifest
                .documents
                .values()
                .all(|revision| revision.starts_with("sha256:"))
        );
    }

    #[test]
    fn rejects_malformed_but_skips_stale_references() {
        let malformed = project_with_root("<definitions>");
        assert!(
            load_schematic_graph(malformed.path(), LoadOptions::deterministic_test())
                .unwrap_err()
                .to_string()
                .contains("malformed XML")
        );
        let missing = project_with_root(
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="D" ssw:packageName="acme"><cmmn:processTask id="Call" ssw:architecturalName="acme.Missing"/></cmmn:definitions>"#,
        );
        let graph =
            load_schematic_graph(missing.path(), LoadOptions::deterministic_test()).unwrap();
        assert_eq!(graph.summary.entities, 1);
        assert!(graph.summary.diagnostics.iter().any(|diagnostic| {
            diagnostic.level == "warning"
                && diagnostic
                    .message
                    .contains("referenced composition acme.Missing does not exist")
        }));
        let endpoint = project_with_root(
            r#"<cmmn:definitions xmlns:cmmn="x" id="D"><cmmn:task id="Keep"/><cmmn:association id="A" sourceRef="Absent" targetRef="Keep"/></cmmn:definitions>"#,
        );
        let graph =
            load_schematic_graph(endpoint.path(), LoadOptions::deterministic_test()).unwrap();
        assert_eq!(graph.summary.entities, 2);
        assert!(
            graph
                .get_entity(None, Some("root"), Some("Keep"), None)
                .is_ok()
        );
        assert!(
            graph
                .get_entity(None, Some("root"), Some("A"), None)
                .is_err()
        );
        assert!(graph.summary.diagnostics.iter().any(|diagnostic| {
            diagnostic.level == "warning"
                && diagnostic.message.contains("skipped edge A")
                && diagnostic
                    .message
                    .contains("source endpoint Absent does not exist")
        }));
    }

    #[test]
    fn embedding_profile_propagates_failures_and_rejects_non_finite_vectors() {
        let project = project_with_root("<definitions id=\"D\"/>");
        let mut options = LoadOptions::deterministic_test();
        options.embedding = EmbeddingProfile::FailingTest;
        assert!(
            derive_embedding_envelope(project.path(), "main.cmmn#diagram", "content", &options)
                .unwrap_err()
                .to_string()
                .contains("fixture embedding failure")
        );
        let mut options = LoadOptions::deterministic_test();
        options.embedding = EmbeddingProfile::NonFiniteTest;
        assert!(
            derive_embedding_envelope(project.path(), "main.cmmn#diagram", "content", &options)
                .unwrap_err()
                .to_string()
                .contains("invalid vector")
        );
        let graph =
            load_schematic_graph(project.path(), LoadOptions::deterministic_test()).unwrap();
        assert!(!graph.summary.vector_ready);
    }

    #[test]
    fn graph_startup_imports_generator_header_without_touching_diagram() {
        let project = project_with_root(
            r#"<cmmn:definitions xmlns:cmmn="x" id="D"><cmmn:task id="Task_1"/></cmmn:definitions>"#,
        );
        let diagram_path = project.path().join("schematics/main.cmmn");
        let diagram_before = fs::read(&diagram_path).unwrap();
        let body = "# Generated contract\nFast vector startup.\n";
        let options = LoadOptions::deterministic_test();
        let envelope =
            derive_embedding_envelope(project.path(), "main.cmmn#diagram", body, &options).unwrap();
        let physical = crate::embedding_document::serialize_markdown(&envelope, body).unwrap();
        fs::write(project.path().join("schematics/main.md"), physical).unwrap();

        let graph = load_schematic_graph(project.path(), options).unwrap();
        assert!(graph.summary.vector_ready);
        assert_eq!(graph.summary.retrieval_mode, "hybrid");
        assert_eq!(graph.summary.vector_readiness, VectorReadiness::Current);
        assert_ne!(graph.summary.embedding_revision, embedding_revision(&[]));
        assert_eq!(fs::read(diagram_path).unwrap(), diagram_before);
    }

    #[test]
    fn imports_multichunk_header_for_markdown_without_headings() {
        let project = project_with_root("<definitions id=\"D\"/>");
        let body = "This headingless document is deliberately long enough to span several chunks without relying on Markdown section headers.\n";
        let mut options = LoadOptions::deterministic_test();
        options.limits.chunk_chars = 24;
        options.limits.chunk_overlap = 4;
        let envelope =
            derive_embedding_envelope(project.path(), "main.cmmn#diagram", body, &options).unwrap();
        assert!(envelope.chunks.len() > 1);
        let physical = crate::embedding_document::serialize_markdown(&envelope, body).unwrap();
        fs::write(project.path().join("schematics/main.md"), physical).unwrap();

        let graph = load_schematic_graph(project.path(), options).unwrap();

        assert!(graph.summary.vector_ready);
        assert_eq!(graph.summary.retrieval_mode, "hybrid");
        assert!(!graph.summary.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("embedding header chunks do not match authored Markdown")
        }));
    }

    #[test]
    fn headerless_cold_start_never_invokes_failing_model() {
        let project = project_with_root("<definitions id=\"D\"/>");
        fs::write(
            project.path().join("schematics/main.md"),
            "# Headerless legacy project\n",
        )
        .unwrap();
        let mut options = LoadOptions::deterministic_test();
        options.embedding = EmbeddingProfile::FailingTest;
        let started = std::time::Instant::now();
        let graph = load_schematic_graph(project.path(), options).unwrap();
        assert!(started.elapsed() < std::time::Duration::from_secs(2));
        assert_eq!(graph.summary.retrieval_mode, "text");
        assert!(!graph.summary.vector_ready);
        assert_eq!(graph.summary.vector_readiness, VectorReadiness::Pending);
        assert!(
            graph
                .summary
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == DiagnosticCode::EmbeddingHeaderMissing })
        );
    }

    #[test]
    fn stale_and_wrong_model_headers_degrade_without_blocking_graph() {
        let project = project_with_root("<definitions id=\"D\"/>");
        let original = "# Original\n";
        let options = LoadOptions::deterministic_test();
        let mut envelope =
            derive_embedding_envelope(project.path(), "main.cmmn#diagram", original, &options)
                .unwrap();
        envelope.model = "another-model".into();
        let physical =
            crate::embedding_document::serialize_markdown(&envelope, "# New body\n").unwrap();
        fs::write(project.path().join("schematics/main.md"), physical).unwrap();
        let graph = load_schematic_graph(project.path(), options).unwrap();
        assert!(!graph.summary.vector_ready);
        assert_eq!(graph.summary.retrieval_mode, "text");
        assert_eq!(graph.summary.vector_readiness, VectorReadiness::Pending);
        assert!(graph.summary.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::EmbeddingHeaderStale
                && (diagnostic.message.contains("stale")
                    || diagnostic.message.contains("another owner/model"))
        }));
    }

    #[test]
    fn malformed_embedding_header_is_structured_as_degraded_vector_readiness() {
        let project = project_with_root("<definitions id=\"D\"/>");
        let physical = format!(
            "{}not-json{}# Recoverable body\n",
            crate::embedding_document::HEADER_START,
            crate::embedding_document::HEADER_END
        );
        fs::write(project.path().join("schematics/main.md"), physical).unwrap();

        let graph =
            load_schematic_graph(project.path(), LoadOptions::deterministic_test()).unwrap();

        assert_eq!(graph.summary.retrieval_mode, "text");
        assert_eq!(graph.summary.vector_readiness, VectorReadiness::Degraded);
        assert!(graph.summary.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::EmbeddingHeaderMalformed
                && diagnostic
                    .message
                    .contains("invalid software-schematic embedding header")
        }));
    }

    #[test]
    fn missing_onnx_runtime_has_actionable_platform_guidance() {
        let guidance = onnx_install_guidance();
        assert!(guidance.contains("required only to generate Markdown embeddings"));
        assert!(guidance.contains("MCP text retrieval remain available"));
        if cfg!(target_os = "macos") {
            assert!(guidance.contains("brew install onnxruntime"));
        }
        if cfg!(target_os = "windows") {
            assert!(guidance.contains("onnxruntime.dll"));
        }
    }

    #[test]
    fn incremental_refresh_matches_a_clean_complete_build() {
        let directory = project_with_root(
            r#"<cmmn:definitions xmlns:cmmn="x" id="D"><cmmn:task id="Task_1" name="Checkout"/></cmmn:definitions>"#,
        );
        fs::write(
            directory.path().join("schematics/main.md"),
            "# Checkout\nOriginal contract.",
        )
        .unwrap();
        fs::create_dir_all(directory.path().join("schematics/docs")).unwrap();
        fs::write(
            directory.path().join("schematics/docs/Task_1.md"),
            "# Task\nStable task documentation.",
        )
        .unwrap();
        let initial =
            load_schematic_graph(directory.path(), LoadOptions::deterministic_test()).unwrap();
        fs::write(
            directory.path().join("schematics/main.md"),
            "# Checkout\nUpdated contract with validation.",
        )
        .unwrap();
        let incremental = refresh_schematic_graph(
            directory.path(),
            LoadOptions::deterministic_test(),
            &initial,
        )
        .unwrap();
        let complete =
            load_schematic_graph(directory.path(), LoadOptions::deterministic_test()).unwrap();

        assert_eq!(incremental.summary.revision, complete.summary.revision);
        assert_eq!(
            serde_json::to_value(&incremental.entities).unwrap(),
            serde_json::to_value(&complete.entities).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&incremental.relations).unwrap(),
            serde_json::to_value(&complete.relations).unwrap()
        );
        assert_eq!(
            serde_json::to_value(&incremental.chunks).unwrap(),
            serde_json::to_value(&complete.chunks).unwrap()
        );
        assert_eq!(incremental.source_hashes, complete.source_hashes);
        assert_eq!(incremental.dependencies, complete.dependencies);
        assert_eq!(
            incremental.reverse_dependencies,
            complete.reverse_dependencies
        );
        assert_eq!(incremental.reuse.parsed_diagrams, 1);
        assert_eq!(incremental.reuse.embeddings, 0);
        let incremental_results = incremental
            .search("validation", None, &[], None, 1, 10)
            .unwrap();
        let complete_results = complete
            .search("validation", None, &[], None, 1, 10)
            .unwrap();
        assert_eq!(
            incremental_results
                .iter()
                .map(|result| &result.owner.id)
                .collect::<Vec<_>>(),
            complete_results
                .iter()
                .map(|result| &result.owner.id)
                .collect::<Vec<_>>()
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_composition_symlink_escape() {
        let directory = project_with_root(
            r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="D" ssw:packageName="acme"><cmmn:processTask id="Call" ssw:architecturalName="acme.Escape"/></cmmn:definitions>"#,
        );
        let outside = tempdir().unwrap();
        fs::write(outside.path().join("main.bpmn"), "<definitions id=\"D\"/>").unwrap();
        fs::create_dir_all(directory.path().join("schematics/acme")).unwrap();
        std::os::unix::fs::symlink(
            outside.path(),
            directory.path().join("schematics/acme/Escape"),
        )
        .unwrap();
        let error = load_schematic_graph(directory.path(), LoadOptions::deterministic_test())
            .unwrap_err()
            .to_string();
        assert!(error.contains("escapes project"));
    }
}
