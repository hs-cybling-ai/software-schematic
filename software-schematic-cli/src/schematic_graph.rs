use crate::{Error, Result};
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
    sync::Arc,
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
    pub level: String,
    pub message: String,
    pub source: Option<String>,
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
        "ONNX Runtime is required only for `ssw mcp` local embeddings.\nInstall it with:\n  brew install onnxruntime\n\nThe diagram editor remains available with `./ssw`."
    } else if cfg!(target_os = "windows") {
        "ONNX Runtime is required only for `ssw mcp` local embeddings.\nInstall the Microsoft.ML.OnnxRuntime native package and place onnxruntime.dll beside .ss\\bin\\ss.exe.\nSee https://onnxruntime.ai/docs/get-started/with-c.html\n\nThe diagram editor remains available with ssw.cmd."
    } else {
        "ONNX Runtime is required only for `ssw mcp` local embeddings.\nInstall a compatible libonnxruntime.so and make it available through the system library path.\nSee https://onnxruntime.ai/docs/install/\n\nThe diagram editor remains available with ./ssw."
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
    if let Some(paths) = std::env::var_os("PATH") {
        candidates.extend(std::env::split_paths(&paths).map(|path| path.join(filename)));
    }
    candidates.into_iter().find(|path| path.is_file())
}

#[derive(Debug, Clone)]
pub struct LoadOptions {
    pub limits: GraphLimits,
    pub embedding: EmbeddingProfile,
}
impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            limits: GraphLimits::default(),
            embedding: EmbeddingProfile::Preset,
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
    internal_chunks: HashMap<NodeId, String>,
    embedding_name: String,
    limits: GraphLimits,
    project_root: PathBuf,
}

impl std::fmt::Debug for GraphSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphSnapshot")
            .field("summary", &self.summary)
            .finish()
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
    let root = project.as_ref().canonicalize()?;
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
        let diagram = parse_diagram(&xml, &relative)?;
        for element in &diagram.elements {
            if let Some(name) = &element.composition {
                let target = PathBuf::from(name.replace('.', "/")).join("main.bpmn");
                if schematics.join(&target).is_file() {
                    queue.push_back(target);
                } else {
                    skipped_elements.insert((relative.clone(), element.id.clone()));
                    diagnostics.push(Diagnostic {
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
                    level: "warning".into(),
                    message: format!("skipped edge {} because {reason}", element.id),
                    source: Some(relative.to_string_lossy().replace('\\', "/")),
                });
            }
        }
    }
    let db = GrafeoDB::new_in_memory();
    let embedding_name = configure_embedding(&db, &root, &options.embedding)?;
    let readiness = db
        .embed_text(&embedding_name, &["readiness"])
        .map_err(graph_error)?
        .first()
        .cloned()
        .ok_or_else(|| Error::Message("embedding model returned no readiness vector".into()))?;
    if readiness.is_empty() || readiness.iter().any(|value| !value.is_finite()) {
        return Err(Error::Message(
            "embedding model returned invalid readiness vector".into(),
        ));
    }
    let dimensions = readiness.len();
    let mut entities = BTreeMap::new();
    let mut relations = Vec::new();
    let mut chunks = BTreeMap::new();
    let mut source_map = BTreeMap::new();
    let mut source_to_urn = HashMap::new();
    let mut diagram_urns = HashMap::new();
    let mut revision_hasher = Sha256::new();
    for (relative, diagram) in &diagrams {
        let diagram_id = urn(&project_id, "diagram", &diagram.owner, None);
        diagram_urns.insert(relative.clone(), diagram_id.clone());
        let diagram_md_path = relative.parent().unwrap_or(Path::new("")).join("main.md");
        let markdown = read_optional_document(
            &schematics,
            &diagram_md_path,
            options.limits.max_document_bytes,
        )?;
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
            dimensions,
            &mut chunks,
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
            let markdown =
                read_optional_document(&schematics, &doc_path, options.limits.max_document_bytes)?;
            revision_hasher.update(markdown.as_bytes());
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
            relations.push(GraphRelation {
                relation_type: "CONTAINS".into(),
                source: diagram_id.clone(),
                target: id.clone(),
            });
            add_chunks(
                &project_id,
                &id,
                &markdown,
                &options.limits,
                &embedding_name,
                dimensions,
                &mut chunks,
            );
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
                        source: element_urn,
                        target: target.clone(),
                    });
                }
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
    let texts: Vec<&str> = chunks.values().map(|c| c.markdown.as_str()).collect();
    let embeddings = if texts.is_empty() {
        Vec::new()
    } else {
        db.embed_text(&embedding_name, &texts)
            .map_err(graph_error)?
    };
    for (chunk, vector) in chunks.values_mut().zip(embeddings) {
        if vector.len() != dimensions || vector.iter().any(|v| !v.is_finite()) {
            return Err(Error::Message(
                "embedding model returned invalid vector".into(),
            ));
        }
        let n = db.create_node_with_props(
            &["DocumentChunk"],
            [
                ("id", Value::from(chunk.id.clone())),
                ("ownerId", Value::from(chunk.owner_id.clone())),
                ("markdown", Value::from(chunk.markdown.clone())),
                ("contentHash", Value::from(chunk.content_hash.clone())),
                ("embedding", Value::Vector(vector.into())),
            ],
        );
        internal.insert(n, chunk.id.clone());
        if let Some(owner) = urn_to_node.get(&chunk.owner_id) {
            db.create_edge(*owner, n, "DOCUMENTED_BY");
        }
        source_map.insert(
            chunk.id.clone(),
            source_map.get(&chunk.owner_id).cloned().unwrap(),
        );
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
    revision_hasher.update(embedding_name.as_bytes());
    let revision = format!("sha256:{}", hex(&revision_hasher.finalize()));
    let root_id = diagram_urns
        .get(&PathBuf::from("main.cmmn"))
        .cloned()
        .unwrap();
    let summary = SnapshotSummary {
        project_name,
        project_id,
        root_id,
        revision,
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
    Ok(GraphSnapshot {
        db,
        summary,
        entities,
        relations,
        chunks,
        source_map,
        internal_chunks: internal,
        embedding_name,
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
        let query_vec = self
            .db
            .embed_text(&self.embedding_name, &[query])
            .map_err(graph_error)?
            .remove(0);
        let matches = self
            .db
            .hybrid_search(
                "DocumentChunk",
                "markdown",
                "embedding",
                query,
                Some(&query_vec),
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
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                depth += 1;
                parse_xml_element(&e, &mut diagram_id, &mut owner, &mut raw)?;
            }
            Ok(Event::Empty(e)) => {
                parse_xml_element(&e, &mut diagram_id, &mut owner, &mut raw)?;
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
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
) -> Result<()> {
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
            return Ok(());
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
            edge,
        });
    }
    Ok(())
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
fn read_optional_document(root: &Path, relative: &Path, max: usize) -> Result<String> {
    let path = root.join(relative);
    if !path.exists() {
        return Ok(String::new());
    }
    let full = path.canonicalize()?;
    if !full.starts_with(root) {
        return Err(Error::Message("document path escapes schematics".into()));
    }
    let bytes = fs::read(full)?;
    if bytes.len() > max {
        return Err(Error::Message(format!("document exceeds {max} bytes")));
    }
    String::from_utf8(bytes).map_err(|_| Error::Message("Markdown must be UTF-8".into()))
}
fn add_chunks(
    project: &str,
    owner: &str,
    markdown: &str,
    limits: &GraphLimits,
    model: &str,
    dimensions: usize,
    out: &mut BTreeMap<String, DocumentChunk>,
) {
    for (ordinal, (headings, text)) in
        chunk_markdown(markdown, limits.chunk_chars, limits.chunk_overlap)
            .into_iter()
            .enumerate()
    {
        let hash = short_hash(text.as_bytes(), 32);
        let id = format!(
            "urn:ssw:{project}:chunk:{}#{hash}:{ordinal}",
            short_hash(owner.as_bytes(), 16)
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
        assert!(
            graph
                .relations
                .iter()
                .any(|r| r.relation_type == "COMPOSES_TO")
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

    fn project_with_root(xml: &str) -> tempfile::TempDir {
        let directory = tempdir().unwrap();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::create_dir_all(directory.path().join("schematics")).unwrap();
        fs::write(directory.path().join(".ss/project-id"), "fixture\n").unwrap();
        fs::write(directory.path().join("schematics/main.cmmn"), xml).unwrap();
        directory
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
            load_schematic_graph(project.path(), options)
                .unwrap_err()
                .to_string()
                .contains("fixture embedding failure")
        );
        let mut options = LoadOptions::deterministic_test();
        options.embedding = EmbeddingProfile::NonFiniteTest;
        assert!(
            load_schematic_graph(project.path(), options)
                .unwrap_err()
                .to_string()
                .contains("invalid readiness vector")
        );
    }

    #[test]
    fn missing_onnx_runtime_has_actionable_platform_guidance() {
        let guidance = onnx_install_guidance();
        assert!(guidance.contains("required only for `ssw mcp`"));
        assert!(guidance.contains("diagram editor remains available"));
        if cfg!(target_os = "macos") {
            assert!(guidance.contains("brew install onnxruntime"));
        }
        if cfg!(target_os = "windows") {
            assert!(guidance.contains("onnxruntime.dll"));
        }
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
