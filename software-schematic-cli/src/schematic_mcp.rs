use crate::{
    Error, Result as SswResult,
    schematic_graph::{
        EntityKind, EntityResult, GraphSnapshot, LoadOptions, ScopeResult, SearchResult,
        SnapshotSummary, load_schematic_graph,
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
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{Mutex, RwLock},
};

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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GraphRefreshOutcome {
    pub status: GraphRefreshStatus,
    pub previous_revision: Option<String>,
    pub active_revision: Option<String>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GraphRefreshStatus {
    Updated,
    Unchanged,
    NotRunning,
    Failed,
}

impl GraphRefreshOutcome {
    pub fn not_running(diagnostic: impl Into<String>) -> Self {
        Self {
            status: GraphRefreshStatus::NotRunning,
            previous_revision: None,
            active_revision: None,
            diagnostic: Some(diagnostic.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ControlMetadata {
    project: String,
    project_id: String,
    pid: u32,
    port: u16,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ControlRequest {
    operation: String,
    project_id: String,
    token: String,
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

#[derive(Clone)]
pub struct SchematicMcp {
    tool_router: ToolRouter<Self>,
    state: Arc<RwLock<Arc<GraphSnapshot>>>,
    project: PathBuf,
    options: LoadOptions,
    reload_lock: Arc<Mutex<()>>,
}

impl SchematicMcp {
    pub fn load(project: impl AsRef<Path>, options: LoadOptions) -> SswResult<Self> {
        let project = project.as_ref().canonicalize()?;
        let snapshot = Arc::new(load_schematic_graph(&project, options.clone())?);
        Ok(Self {
            tool_router: Self::tool_router(),
            state: Arc::new(RwLock::new(snapshot)),
            project,
            options,
            reload_lock: Arc::new(Mutex::new(())),
        })
    }
    pub async fn summary(&self) -> SnapshotSummary {
        self.state.read().await.summary.clone()
    }

    async fn refresh_from_documents(&self) -> GraphRefreshOutcome {
        let _reload = self.reload_lock.lock().await;
        let previous = self.summary().await.revision;
        let project = self.project.clone();
        let options = self.options.clone();
        let replacement =
            tokio::task::spawn_blocking(move || load_schematic_graph(project, options))
                .await
                .map_err(|error| Error::Message(format!("graph refresh task failed: {error}")))
                .and_then(|result| result);
        match replacement {
            Ok(replacement) => {
                let active = replacement.summary.revision.clone();
                let warning = replacement
                    .summary
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.level == "warning")
                    .map(|diagnostic| match &diagnostic.source {
                        Some(source) => format!("{source}: {}", diagnostic.message),
                        None => diagnostic.message.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                let status = if previous == active {
                    GraphRefreshStatus::Unchanged
                } else {
                    GraphRefreshStatus::Updated
                };
                *self.state.write().await = Arc::new(replacement);
                GraphRefreshOutcome {
                    status,
                    previous_revision: Some(previous),
                    active_revision: Some(active),
                    diagnostic: (!warning.is_empty()).then_some(warning),
                }
            }
            Err(error) => GraphRefreshOutcome {
                status: GraphRefreshStatus::Failed,
                previous_revision: Some(previous.clone()),
                active_revision: Some(previous),
                diagnostic: Some(error.to_string()),
            },
        }
    }
}

#[tool_router]
impl SchematicMcp {
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
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions("Query-only project-local compiled Software Schematic graph. Verify get_project_model, then resolve development scope. Only new/modify entities authorize implementation. If a contract is missing or incorrect, stop and ask the user to evaluate and save it in the Software Schematic web application.")
    }
}

fn control_path(project: &Path) -> PathBuf {
    project.join(".ss/run/mcp-control.json")
}

fn project_id(project: &Path) -> SswResult<String> {
    let value = fs::read_to_string(project.join(".ss/project-id"))?;
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::Message(".ss/project-id is empty".into()));
    }
    Ok(value.into())
}

fn control_token(project: &Path) -> String {
    let random: [u8; 32] = rand::random();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{:x}",
        Sha256::digest(format!("{}\0{}\0{:?}", project.display(), now, random).as_bytes())
    )
}

pub(crate) struct RefreshListener {
    task: tokio::task::JoinHandle<()>,
    path: PathBuf,
    pid: u32,
}

impl Drop for RefreshListener {
    fn drop(&mut self) {
        self.task.abort();
        if let Ok(bytes) = fs::read(&self.path)
            && let Ok(metadata) = serde_json::from_slice::<ControlMetadata>(&bytes)
            && metadata.pid == self.pid
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(crate) async fn start_refresh_listener(server: SchematicMcp) -> SswResult<RefreshListener> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let path = control_path(&server.project);
    let directory = path.parent().unwrap();
    fs::create_dir_all(directory)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        set_owner_permissions(directory, fs::Permissions::from_mode(0o700))?;
    }
    let metadata = ControlMetadata {
        project: server.project.to_string_lossy().into_owned(),
        project_id: project_id(&server.project)?,
        pid: std::process::id(),
        port: listener.local_addr()?.port(),
        token: control_token(&server.project),
    };
    let temporary = path.with_extension(format!("{}.tmp", metadata.pid));
    fs::write(&temporary, serde_json::to_vec(&metadata).unwrap())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        set_owner_permissions(&temporary, fs::Permissions::from_mode(0o600))?;
    }
    fs::rename(temporary, &path)?;
    let expected = Arc::new(metadata);
    let task = tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let server = server.clone();
            let expected = expected.clone();
            tokio::spawn(async move {
                let _ = handle_control_connection(stream, server, expected).await;
            });
        }
    });
    Ok(RefreshListener {
        task,
        path,
        pid: std::process::id(),
    })
}

#[cfg(unix)]
fn set_owner_permissions(path: &Path, permissions: fs::Permissions) -> SswResult<()> {
    match fs::set_permissions(path, permissions) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(error) => Err(error.into()),
    }
}

async fn handle_control_connection(
    stream: TcpStream,
    server: SchematicMcp,
    expected: Arc<ControlMetadata>,
) -> SswResult<()> {
    let (read, mut write) = stream.into_split();
    let mut line = String::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(2),
        BufReader::new(read).read_line(&mut line),
    )
    .await
    .map_err(|_| Error::Message("graph refresh notification timed out".into()))??;
    let request: ControlRequest = serde_json::from_str(&line)
        .map_err(|error| Error::Message(format!("invalid graph refresh notification: {error}")))?;
    let response = if request.operation == "documents_changed"
        && request.project_id == expected.project_id
        && request.token == expected.token
    {
        server.refresh_from_documents().await
    } else {
        GraphRefreshOutcome {
            status: GraphRefreshStatus::Failed,
            previous_revision: None,
            active_revision: Some(server.summary().await.revision),
            diagnostic: Some("graph refresh notification was rejected".into()),
        }
    };
    write
        .write_all(&serde_json::to_vec(&response).unwrap())
        .await?;
    write.write_all(b"\n").await?;
    Ok(())
}

pub async fn notify_documents_changed(project: &Path) -> GraphRefreshOutcome {
    let project = match project.canonicalize() {
        Ok(project) => project,
        Err(error) => return GraphRefreshOutcome::not_running(error.to_string()),
    };
    let bytes = match tokio::fs::read(control_path(&project)).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return GraphRefreshOutcome::not_running("project MCP is not running");
        }
        Err(error) => return GraphRefreshOutcome::not_running(error.to_string()),
    };
    let metadata: ControlMetadata = match serde_json::from_slice(&bytes) {
        Ok(metadata) => metadata,
        Err(error) => {
            return GraphRefreshOutcome::not_running(format!(
                "stale MCP control metadata: {error}"
            ));
        }
    };
    let expected_id = match project_id(&project) {
        Ok(value) => value,
        Err(error) => return GraphRefreshOutcome::not_running(error.to_string()),
    };
    if metadata.project != project.to_string_lossy() || metadata.project_id != expected_id {
        return GraphRefreshOutcome {
            status: GraphRefreshStatus::Failed,
            previous_revision: None,
            active_revision: None,
            diagnostic: Some("MCP control metadata belongs to another project".into()),
        };
    }
    let response = async {
        let mut stream = TcpStream::connect((std::net::Ipv4Addr::LOCALHOST, metadata.port)).await?;
        let request = ControlRequest {
            operation: "documents_changed".into(),
            project_id: expected_id,
            token: metadata.token,
        };
        stream
            .write_all(&serde_json::to_vec(&request).unwrap())
            .await?;
        stream.write_all(b"\n").await?;
        let mut line = String::new();
        BufReader::new(stream).read_line(&mut line).await?;
        serde_json::from_str::<GraphRefreshOutcome>(&line)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    };
    match tokio::time::timeout(std::time::Duration::from_secs(30), response).await {
        Ok(Ok(outcome)) => outcome,
        Ok(Err(error)) => {
            GraphRefreshOutcome::not_running(format!("project MCP did not answer: {error}"))
        }
        Err(_) => GraphRefreshOutcome {
            status: GraphRefreshStatus::Failed,
            previous_revision: None,
            active_revision: None,
            diagnostic: Some("project MCP graph refresh timed out".into()),
        },
    }
}

pub async fn serve_mcp(project: PathBuf) -> SswResult<()> {
    let server = SchematicMcp::load(project, LoadOptions::default())?;
    let _refresh_listener = start_refresh_listener(server.clone()).await?;
    eprintln!(
        "Software Schematic MCP project={} revision={}",
        server.summary().await.project_name,
        server.summary().await.revision
    );
    let service = server
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| Error::Message(format!("MCP initialization: {e}")))?;
    service
        .waiting()
        .await
        .map_err(|e| Error::Message(format!("MCP service: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
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
        fs::write(directory.path().join("schematics/main.cmmn"), r#"<cmmn:definitions xmlns:cmmn="x" xmlns:ssw="y" id="Root" ssw:packageName="shop"><cmmn:task id="Checkout" name="Checkout" ssw:implementationStatus="modify"/><cmmn:task id="Payment" name="Payment" ssw:implementationStatus="new"/><cmmn:task id="Locked" name="Archive" ssw:implementationStatus="locked"/></cmmn:definitions>"#).unwrap();
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
    async fn document_notification_refreshes_and_failed_build_keeps_last_revision() {
        let directory = fixture();
        let server =
            SchematicMcp::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let listener = start_refresh_listener(server.clone()).await.unwrap();
        let initial = server.summary().await.revision;
        fs::write(
            directory.path().join("schematics/docs/Checkout.md"),
            "# Checkout\nValidate a newly required contract parameter.",
        )
        .unwrap();
        let refreshed = notify_documents_changed(directory.path()).await;
        assert_eq!(refreshed.status, GraphRefreshStatus::Updated);
        assert_eq!(
            refreshed.previous_revision.as_deref(),
            Some(initial.as_str())
        );
        let updated = server.summary().await.revision;
        assert_ne!(updated, initial);

        fs::write(
            directory.path().join("schematics/main.cmmn"),
            r#"<cmmn:definitions xmlns:cmmn="x" id="D"><cmmn:task id="Keep"/><cmmn:association id="Stale" sourceRef="Deleted" targetRef="Keep"/></cmmn:definitions>"#,
        )
        .unwrap();
        let warned = notify_documents_changed(directory.path()).await;
        assert_eq!(warned.status, GraphRefreshStatus::Updated);
        assert!(
            warned
                .diagnostic
                .as_deref()
                .is_some_and(|message| message.contains("skipped edge Stale"))
        );
        assert_eq!(server.summary().await.entities, 2);

        fs::write(
            directory.path().join("schematics/docs/Checkout.md"),
            "# Checkout\nThe newest durable contract wins overlapping refreshes.",
        )
        .unwrap();
        let (first, second) = tokio::join!(
            notify_documents_changed(directory.path()),
            notify_documents_changed(directory.path())
        );
        assert_ne!(first.status, GraphRefreshStatus::Failed);
        assert_ne!(second.status, GraphRefreshStatus::Failed);
        let updated = server.summary().await.revision;
        assert_eq!(
            first.active_revision.as_deref(),
            second.active_revision.as_deref()
        );

        fs::write(directory.path().join("schematics/main.cmmn"), "<broken>").unwrap();
        let failed = notify_documents_changed(directory.path()).await;
        assert_eq!(failed.status, GraphRefreshStatus::Failed);
        assert_eq!(failed.active_revision.as_deref(), Some(updated.as_str()));
        assert_eq!(server.summary().await.revision, updated);
        drop(listener);
        assert!(!control_path(directory.path()).exists());
    }

    #[tokio::test]
    async fn notification_is_project_bound_and_missing_mcp_is_non_fatal() {
        let directory = fixture();
        let missing = notify_documents_changed(directory.path()).await;
        assert_eq!(missing.status, GraphRefreshStatus::NotRunning);

        fs::create_dir_all(directory.path().join(".ss/run")).unwrap();
        fs::write(control_path(directory.path()), b"stale metadata").unwrap();
        let stale = notify_documents_changed(directory.path()).await;
        assert_eq!(stale.status, GraphRefreshStatus::NotRunning);
        assert!(
            stale
                .diagnostic
                .unwrap()
                .contains("stale MCP control metadata")
        );

        let server =
            SchematicMcp::load(directory.path(), LoadOptions::deterministic_test()).unwrap();
        let _listener = start_refresh_listener(server.clone()).await.unwrap();
        let path = control_path(directory.path());
        let mut metadata: ControlMetadata =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        metadata.project_id = "another-project".into();
        fs::write(&path, serde_json::to_vec(&metadata).unwrap()).unwrap();
        let rejected = notify_documents_changed(directory.path()).await;
        assert_eq!(rejected.status, GraphRefreshStatus::Failed);
        assert!(rejected.diagnostic.unwrap().contains("another project"));
    }
}
