use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Extension},
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
pub mod assistant;
pub mod delivery_protocol;
pub mod delivery_runtime;
pub mod embedding_document;
pub mod project_runtime;
pub mod schematic_graph;
pub mod schematic_mcp;
use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use thiserror::Error;
use tower_http::services::{ServeDir, ServeFile};
use walkdir::WalkDir;

const AGENTS_BEGIN: &str = "<!-- software-schematic:begin -->";
const AGENTS_END: &str = "<!-- software-schematic:end -->";
const AGENTS_GUIDANCE: &str = r#"<!-- software-schematic:begin -->
## Software Schematic development

Treat the Software Schematic diagram as the source of software contracts. Use the project-local `design` skill for a deep interview with frequent small visual proposals. Let the developer adjust the diagram directly, then read those edits before continuing the interview.

Do not start implementation until a versioned plan exists. The developer can click Play for the current browser selection or invoke `plan` with a natural-language node or label; both use the same planner. Then use `build` to select an open plan and implement one claimed work item at a time. Only model elements marked `new` or `modify` are buildable; `open` and `locked` items are context.

If the diagram changes during a build, stop and create or resume a plan for the newly published model. The saved diagram is the logical contract; generated edge/event implementation contracts may record justified physical differences without expanding feature scope. Do not create a second specification, orchestration script, or raw XML edit.
<!-- software-schematic:end -->"#;

pub fn validate_package_name(value: &str) -> Result<&str> {
    if value.is_empty() || !value.split('.').all(is_lower_camel) {
        return Err(Error::Message("package Name must use dot-separated lowerCamelCase segments, for example cybling.subscription".into()));
    }
    Ok(value)
}

pub fn validate_process_name(value: &str) -> Result<&str> {
    if !is_upper_camel(value) {
        return Err(Error::Message(
            "process Name must use UpperCamelCase, for example SelectAndOutfit".into(),
        ));
    }
    Ok(value)
}

pub fn validate_member_name(value: &str) -> Result<&str> {
    if !is_lower_camel(value) {
        return Err(Error::Message(
            "task or event Name must use lowerCamelCase, for example processSubscription".into(),
        ));
    }
    Ok(value)
}

fn is_lower_camel(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn is_upper_camel(value: &str) -> bool {
    value.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

pub fn validate_qualified_process_name(value: &str) -> Result<&str> {
    let (package, process) = value.rsplit_once('.').ok_or_else(|| Error::Message(
        "qualified process Name must include a package and process, for example cybling.SelectAndOutfit".into(),
    ))?;
    validate_package_name(package)?;
    validate_process_name(process)?;
    Ok(value)
}

pub fn validate_element_name(value: &str) -> Result<&str> {
    let mut parts = value.split('#');
    let process = parts.next().unwrap_or_default();
    let member = parts.next();
    if parts.next().is_some() {
        return Err(Error::Message(
            "Name may contain only one # member separator".into(),
        ));
    }
    validate_qualified_process_name(process)?;
    if let Some(member) = member {
        validate_member_name(member)?;
    }
    Ok(value)
}

pub fn composition_folder_for_name(value: &str) -> Result<PathBuf> {
    validate_qualified_process_name(value)?;
    Ok(value.split('.').collect())
}

pub fn qualified_name_for_composition_folder(value: &Path) -> Result<String> {
    if value.is_absolute()
        || value
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(Error::Message(
            "composition folder must be confined beneath schematics".into(),
        ));
    }
    let mut parts = value
        .iter()
        .map(|part| {
            part.to_str()
                .ok_or_else(|| Error::Message("composition folder must be UTF-8".into()))
        })
        .collect::<Result<Vec<_>>>()?;
    if parts.last() == Some(&"main.bpmn") {
        parts.pop();
    }
    let name = parts.join(".");
    validate_qualified_process_name(&name)?;
    Ok(name)
}

pub fn symbol_collision_key(value: &str) -> String {
    value.to_ascii_lowercase()
}

// Release builds embed the production web bundle generated by software-schematic-web.
static WEB_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/web");
static MODEL_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/models");
const STARTER_BPMN: &str = include_str!("../assets/starter.bpmn");
const STARTER_CMMN: &str = include_str!("../assets/starter.cmmn");
const STARTER_MD: &str = include_str!("../assets/starter.md");
const PROJECT_LICENSE: &str = include_str!("../../LICENSE");
const PROJECT_NOTICE: &str = include_str!("../../NOTICE");
const THIRD_PARTY_NOTICES: &str = include_str!("../../THIRD_PARTY_NOTICES.md");
const MAC_WRAPPER: &str = "#!/bin/sh\nset -eu\nSCRIPT_DIR=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\ncase \"${1:-}\" in\n  auth) shift; exec \"$SCRIPT_DIR/.ss/bin/ss\" auth --project \"$SCRIPT_DIR\" \"$@\" ;;\n  doctor) shift; exec \"$SCRIPT_DIR/.ss/bin/ss\" doctor --project \"$SCRIPT_DIR\" \"$@\" ;;\n  stop) shift; exec \"$SCRIPT_DIR/.ss/bin/ss\" stop --project \"$SCRIPT_DIR\" \"$@\" ;;\n  embeddings) shift; exec \"$SCRIPT_DIR/.ss/bin/ss\" embeddings --project \"$SCRIPT_DIR\" \"$@\" ;;\n  mcp) shift; exec \"$SCRIPT_DIR/.ss/bin/ss\" mcp --project \"$SCRIPT_DIR\" \"$@\" ;;\n  update) shift; exec \"$SCRIPT_DIR/.ss/bin/ss\" update --project \"$SCRIPT_DIR\" \"$@\" ;;\nesac\nexec \"$SCRIPT_DIR/.ss/bin/ss\" serve --project \"$SCRIPT_DIR\" \"$@\"\n";
const WINDOWS_WRAPPER: &str = "@echo off\r\nsetlocal\r\nif \"%~1\"==\"auth\" (\r\n  shift\r\n  \"%~dp0.ss\\bin\\ss.exe\" auth --project \"%~dp0\" %*\r\n) else if \"%~1\"==\"doctor\" (\r\n  shift\r\n  \"%~dp0.ss\\bin\\ss.exe\" doctor --project \"%~dp0\" %*\r\n) else if \"%~1\"==\"stop\" (\r\n  shift\r\n  \"%~dp0.ss\\bin\\ss.exe\" stop --project \"%~dp0\" %*\r\n) else if \"%~1\"==\"embeddings\" (\r\n  shift\r\n  \"%~dp0.ss\\bin\\ss.exe\" embeddings --project \"%~dp0\" %*\r\n) else if \"%~1\"==\"mcp\" (\r\n  shift\r\n  \"%~dp0.ss\\bin\\ss.exe\" mcp --project \"%~dp0\" %*\r\n) else if \"%~1\"==\"update\" (\r\n  shift\r\n  \"%~dp0.ss\\bin\\ss.exe\" update --project \"%~dp0\" %*\r\n) else (\r\n  \"%~dp0.ss\\bin\\ss.exe\" serve --project \"%~dp0\" %*\r\n)\r\n";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),
    #[error("document revision conflict for {path}: expected {expected}, current {current}")]
    RevisionConflict {
        path: String,
        expected: String,
        current: String,
    },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::RevisionConflict { .. } => StatusCode::CONFLICT,
            Error::Message(_) => StatusCode::BAD_REQUEST,
            Error::Io(ref value) if value.kind() == std::io::ErrorKind::NotFound => {
                StatusCode::NOT_FOUND
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = match &self {
            Error::RevisionConflict {
                path,
                expected,
                current,
            } => serde_json::json!({
                "error": self.to_string(),
                "code": "revisionConflict",
                "path": path,
                "expectedRevision": expected,
                "currentRevision": current,
            }),
            _ => serde_json::json!({ "error": self.to_string() }),
        };
        (status, Json(body)).into_response()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub project: PathBuf,
    pub tool: PathBuf,
    pub schematics: PathBuf,
}

pub fn init_project(project: impl AsRef<Path>) -> Result<ProjectLayout> {
    let project = absolute(project.as_ref())?;
    fs::create_dir_all(&project)?;
    let collisions: Vec<_> = [".ss", "schematics", "ssw", "ssw.cmd"]
        .into_iter()
        .map(|name| project.join(name))
        .filter(|path| path.exists())
        .collect();
    if !collisions.is_empty() {
        return Err(Error::Message(format!(
            "initialization refused; path already exists: {}",
            collisions
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    let tool = project.join(".ss");
    let web = tool.join("web");
    let schematics = project.join("schematics");
    fs::create_dir_all(tool.join("bin"))?;
    fs::create_dir_all(&web)?;
    fs::create_dir_all(&schematics)?;
    WEB_ASSETS
        .extract(&web)
        .map_err(|e| Error::Message(e.to_string()))?;
    MODEL_ASSETS
        .extract(tool.join("models"))
        .map_err(|e| Error::Message(e.to_string()))?;
    fs::write(
        tool.join("version"),
        format!("{}\n", env!("CARGO_PKG_VERSION")),
    )?;
    fs::write(
        tool.join("operation-plan.schema.json"),
        serde_json::to_vec_pretty(&assistant::operation_plan_schema()).unwrap(),
    )?;
    fs::write(tool.join("starter.cmmn"), STARTER_CMMN)?;
    fs::write(tool.join("LICENSE"), PROJECT_LICENSE)?;
    fs::write(tool.join("NOTICE"), PROJECT_NOTICE)?;
    fs::write(tool.join("THIRD_PARTY_NOTICES.md"), THIRD_PARTY_NOTICES)?;
    fs::write(schematics.join("main.cmmn"), STARTER_CMMN)?;
    fs::write(schematics.join("main.md"), STARTER_MD)?;
    fs::write(project.join("ssw"), MAC_WRAPPER)?;
    fs::write(project.join("ssw.cmd"), WINDOWS_WRAPPER)?;
    write_project_id(&tool, &project)?;
    update_managed_project_files(&project)?;
    install_runtime_binary(&tool)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(project.join("ssw"), fs::Permissions::from_mode(0o755))?;
    }
    Ok(ProjectLayout {
        project,
        tool,
        schematics,
    })
}

pub fn update_project(project: impl AsRef<Path>) -> Result<ProjectLayout> {
    let project = absolute(project.as_ref())?;
    let tool = project.join(".ss");
    let schematics = project.join("schematics");
    if !tool.is_dir() || !schematics.join("main.cmmn").is_file() {
        return Err(Error::Message(
            "not a Software Schematic project; expected .ss and schematics/main.cmmn".into(),
        ));
    }
    fs::create_dir_all(tool.join("bin"))?;
    fs::create_dir_all(tool.join("web"))?;
    WEB_ASSETS
        .extract(tool.join("web"))
        .map_err(|e| Error::Message(e.to_string()))?;
    MODEL_ASSETS
        .extract(tool.join("models"))
        .map_err(|e| Error::Message(e.to_string()))?;
    fs::write(
        tool.join("version"),
        format!("{}\n", env!("CARGO_PKG_VERSION")),
    )?;
    fs::write(
        tool.join("operation-plan.schema.json"),
        serde_json::to_vec_pretty(&assistant::operation_plan_schema()).unwrap(),
    )?;
    fs::write(tool.join("LICENSE"), PROJECT_LICENSE)?;
    fs::write(tool.join("NOTICE"), PROJECT_NOTICE)?;
    fs::write(tool.join("THIRD_PARTY_NOTICES.md"), THIRD_PARTY_NOTICES)?;
    fs::write(project.join("ssw"), MAC_WRAPPER)?;
    fs::write(project.join("ssw.cmd"), WINDOWS_WRAPPER)?;
    write_project_id(&tool, &project)?;
    update_managed_project_files(&project)?;
    install_runtime_binary(&tool)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(project.join("ssw"), fs::Permissions::from_mode(0o755))?;
    }
    Ok(ProjectLayout {
        project,
        tool,
        schematics,
    })
}

fn install_runtime_binary(tool: &Path) -> Result<PathBuf> {
    let source = std::env::current_exe()?;
    let runtime_name = if cfg!(windows) { "ss.exe" } else { "ss" };
    let destination = tool.join("bin").join(runtime_name);
    let temporary = tool
        .join("bin")
        .join(format!(".{runtime_name}.{}.tmp", std::process::id()));
    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }
    fs::copy(source, &temporary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o755))?;
        fs::rename(&temporary, &destination)?;
    }
    #[cfg(windows)]
    {
        let backup = tool.join("bin").join(format!(".{runtime_name}.previous"));
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        if destination.exists() {
            fs::rename(&destination, &backup)?;
        }
        if let Err(error) = fs::rename(&temporary, &destination) {
            if backup.exists() {
                let _ = fs::rename(&backup, &destination);
            }
            return Err(error.into());
        }
        if backup.exists() {
            fs::remove_file(backup)?;
        }
    }
    Ok(destination)
}

fn write_project_id(tool: &Path, project: &Path) -> Result<()> {
    let path = tool.join("project-id");
    if path.exists() {
        return Ok(());
    }
    use sha2::{Digest, Sha256};
    let seed = format!(
        "{}\0{}",
        project
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project"),
        project.display()
    );
    let digest = format!("{:x}", Sha256::digest(seed.as_bytes()));
    fs::write(path, format!("{}\n", &digest[..24]))?;
    Ok(())
}

fn update_managed_project_files(project: &Path) -> Result<()> {
    fs::create_dir_all(project.join(".ss/run"))?;
    fs::create_dir_all(project.join(".ss/workflows/proposals"))?;
    fs::create_dir_all(project.join(".ss/workflows/interviews"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(project.join(".ss/run"), fs::Permissions::from_mode(0o700))?;
    }
    update_agents(project)?;
    update_codex_config(project)?;
    delivery_runtime::install_skill_adapters(project)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub status: String,
    pub project_id: String,
    pub daemon: String,
    pub managed_integrations: String,
    pub repaired: bool,
}

pub fn doctor_project(project: impl AsRef<Path>, repair: bool) -> Result<DoctorReport> {
    let project = absolute(project.as_ref())?.canonicalize()?;
    if repair {
        let _ = project_runtime::DaemonOwnership::reclaim_stale(&project)?;
        update_managed_project_files(&project)?;
        fs::write(
            project.join(".ss/version"),
            format!("{}\n", env!("CARGO_PKG_VERSION")),
        )?;
    }
    let project_id = fs::read_to_string(project.join(".ss/project-id"))?
        .trim()
        .to_owned();
    if project_id.is_empty() {
        return Err(Error::Message(".ss/project-id is empty".into()));
    }
    let skills = ["design", "plan", "build"];
    let codex_config = fs::read_to_string(project.join(".codex/config.toml")).unwrap_or_default();
    let codex_managed = codex_config
        .parse::<toml_edit::DocumentMut>()
        .ok()
        .is_some_and(|document| {
            document["mcp_servers"]["software_schematic"]["command"].as_str() == Some("./ssw")
                && document["mcp_servers"]["software_schematic"]["args"]
                    .as_array()
                    .is_some_and(|args| {
                        args.len() == 1
                            && args.get(0).and_then(|value| value.as_str()) == Some("mcp")
                    })
        });
    let managed = codex_managed
        && skills.iter().all(|name| {
            project
                .join(".codex/skills")
                .join(name)
                .join("SKILL.md")
                .is_file()
        })
        && skills.iter().all(|name| {
            project
                .join(".claude/commands")
                .join(format!("{name}.md"))
                .is_file()
        });
    let daemon = match project_runtime::DaemonOwnership::healthy_discovery(&project)? {
        Some(_) => "running",
        None => "stopped",
    };
    Ok(DoctorReport {
        status: if managed {
            "ready"
        } else {
            "run with --repair"
        }
        .into(),
        project_id,
        daemon: daemon.into(),
        managed_integrations: if managed { "current" } else { "missing" }.into(),
        repaired: repair,
    })
}

fn update_agents(project: &Path) -> Result<()> {
    let path = project.join("AGENTS.md");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let eol = if existing.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let block = AGENTS_GUIDANCE.replace('\n', eol);
    let updated = match (existing.find(AGENTS_BEGIN), existing.find(AGENTS_END)) {
        (Some(start), Some(end)) if start <= end => {
            let tail = end + AGENTS_END.len();
            format!("{}{}{}", &existing[..start], block, &existing[tail..])
        }
        (None, None) if existing.is_empty() => format!("{block}{eol}"),
        (None, None) => format!(
            "{}{eol}{eol}{block}{eol}",
            existing.trim_end_matches(['\r', '\n'])
        ),
        _ => {
            return Err(Error::Message(
                "AGENTS.md contains an incomplete Software Schematic managed block".into(),
            ));
        }
    };
    fs::write(path, updated)?;
    Ok(())
}

fn update_codex_config(project: &Path) -> Result<()> {
    use toml_edit::{DocumentMut, Item, Table, Value};
    let directory = project.join(".codex");
    fs::create_dir_all(&directory)?;
    let path = directory.join("config.toml");
    let source = fs::read_to_string(&path).unwrap_or_default();
    let mut document = source
        .parse::<DocumentMut>()
        .map_err(|e| Error::Message(format!("invalid .codex/config.toml: {e}")))?;
    if !document.as_table().contains_key("mcp_servers") {
        document["mcp_servers"] = Item::Table(Table::new());
    }
    document["mcp_servers"]["software_schematic"]["command"] = Item::Value(Value::from("./ssw"));
    document["mcp_servers"]["software_schematic"]["args"] =
        toml_edit::value(toml_edit::Array::from_iter(["mcp"]));
    fs::write(path, document.to_string())?;
    Ok(())
}

fn absolute(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AssistantProjectConfig {
    provider: String,
}

fn assistant_config_path(project: &Path) -> PathBuf {
    project.join(".ss/assistant.json")
}

fn command_available(command: &str) -> bool {
    std::process::Command::new(command)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn configured_assistant_provider(project: &Path) -> Option<String> {
    fs::read(assistant_config_path(project))
        .ok()
        .and_then(|content| serde_json::from_slice::<AssistantProjectConfig>(&content).ok())
        .map(|config| config.provider)
}

fn resolved_assistant_provider(project: &Path) -> Option<String> {
    std::env::var("SSW_ASSISTANT_PROVIDER")
        .ok()
        .or_else(|| configured_assistant_provider(project))
        .or_else(|| command_available("codex").then(|| "codex".into()))
        .or_else(|| command_available("claude").then(|| "claude".into()))
}

pub fn assistant_auth_login(project: impl AsRef<Path>, requested: Option<&str>) -> Result<()> {
    let project = absolute(project.as_ref())?.canonicalize()?;
    let provider = match requested {
        Some("codex") if command_available("codex") => "codex",
        Some("claude") if command_available("claude") => "claude",
        Some("codex" | "claude") => {
            return Err(Error::Message(format!(
                "{} CLI is not installed or not available on PATH",
                requested.unwrap()
            )));
        }
        Some(other) => {
            return Err(Error::Message(format!(
                "unsupported assistant provider: {other}"
            )));
        }
        None if command_available("codex") => "codex",
        None if command_available("claude") => "claude",
        None => {
            return Err(Error::Message(
                "install Codex CLI or Claude Code before running assistant login".into(),
            ));
        }
    };
    println!(
        "Opening the official {provider} sign-in flow. SSW never receives your password or token."
    );
    let status = if provider == "codex" {
        std::process::Command::new("codex").arg("login").status()?
    } else {
        std::process::Command::new("claude")
            .args(["auth", "login"])
            .status()?
    };
    if !status.success() {
        return Err(Error::Message(format!(
            "{provider} sign-in did not complete"
        )));
    }
    let path = assistant_config_path(&project);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(
        path,
        serde_json::to_vec_pretty(&AssistantProjectConfig {
            provider: provider.into(),
        })
        .unwrap(),
    )?;
    println!("Software Schematic assistant configured to use {provider} for this project.");
    Ok(())
}

pub fn assistant_auth_status(project: impl AsRef<Path>) -> Result<()> {
    let project = absolute(project.as_ref())?.canonicalize()?;
    let Some(provider) = configured_assistant_provider(&project) else {
        println!("No local assistant is configured. Run ./ssw auth login.");
        return Ok(());
    };
    let status = if provider == "codex" {
        std::process::Command::new("codex")
            .args(["login", "status"])
            .status()?
    } else {
        std::process::Command::new("claude")
            .args(["auth", "status"])
            .status()?
    };
    if !status.success() {
        return Err(Error::Message(format!(
            "{provider} is configured but not authenticated"
        )));
    }
    println!("Project assistant provider: {provider}");
    Ok(())
}

pub fn assistant_auth_logout(project: impl AsRef<Path>) -> Result<()> {
    let project = absolute(project.as_ref())?.canonicalize()?;
    let Some(provider) = configured_assistant_provider(&project) else {
        return Err(Error::Message(
            "no project assistant provider is configured".into(),
        ));
    };
    let status = if provider == "codex" {
        std::process::Command::new("codex").arg("logout").status()?
    } else {
        std::process::Command::new("claude")
            .args(["auth", "logout"])
            .status()?
    };
    if !status.success() {
        return Err(Error::Message(format!(
            "{provider} logout did not complete"
        )));
    }
    let path = assistant_config_path(&project);
    if path.exists() {
        fs::remove_file(path)?;
    }
    println!("Signed out of {provider} and removed the project assistant selection.");
    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    project: Arc<PathBuf>,
    schematics: Arc<PathBuf>,
    assistant_slots: Arc<tokio::sync::Semaphore>,
    embedding_jobs: Arc<tokio::sync::Mutex<BTreeMap<String, u64>>>,
    embedding_generation: Arc<AtomicU64>,
    embedding_slots: Arc<tokio::sync::Semaphore>,
    embedding_status: Arc<tokio::sync::RwLock<EmbeddingOutcome>>,
    document_write_lock: Arc<tokio::sync::Mutex<()>>,
    assistant_proposals: Arc<tokio::sync::RwLock<BTreeMap<String, AssistantProposalSubmission>>>,
    graph: Option<schematic_mcp::SchematicMcp>,
}

impl AppState {
    pub fn new(project: impl AsRef<Path>) -> Result<Self> {
        let project = absolute(project.as_ref())?.canonicalize()?;
        let schematics = project.join("schematics").canonicalize()?;
        Ok(Self {
            project: Arc::new(project),
            schematics: Arc::new(schematics),
            assistant_slots: Arc::new(tokio::sync::Semaphore::new(2)),
            embedding_jobs: Arc::new(tokio::sync::Mutex::new(BTreeMap::new())),
            embedding_generation: Arc::new(AtomicU64::new(0)),
            embedding_slots: Arc::new(tokio::sync::Semaphore::new(1)),
            embedding_status: Arc::new(tokio::sync::RwLock::new(EmbeddingOutcome::default())),
            document_write_lock: Arc::new(tokio::sync::Mutex::new(())),
            assistant_proposals: Arc::new(tokio::sync::RwLock::new(BTreeMap::new())),
            graph: None,
        })
    }

    pub fn with_graph(mut self, graph: schematic_mcp::SchematicMcp) -> Self {
        self.graph = Some(graph);
        self
    }

    async fn notify_document_changed(
        &self,
        path: &str,
        kind: schematic_mcp::DocumentChangeKind,
    ) -> schematic_mcp::GraphRefreshOutcome {
        if let Some(graph) = &self.graph {
            graph.notify_document_changed(path, kind).await
        } else {
            schematic_mcp::GraphRefreshOutcome::not_running("project runtime graph is unavailable")
        }
    }

    async fn graph_refresh_status(&self) -> schematic_mcp::GraphRefreshOutcome {
        match &self.graph {
            Some(graph) => graph.refresh_status_public().await,
            None => schematic_mcp::GraphRefreshOutcome::not_running(
                "project runtime graph is unavailable",
            ),
        }
    }

    pub fn resolve(&self, relative: &str, permit_missing: bool) -> Result<PathBuf> {
        let path = Path::new(relative);
        if relative.is_empty()
            || path.is_absolute()
            || path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(Error::Message(
                "path must be a normalized path relative to schematics".into(),
            ));
        }
        let candidate = self.schematics.join(path);
        let mut existing = candidate.as_path();
        while !existing.exists() {
            existing = existing
                .parent()
                .ok_or_else(|| Error::Message("path has no confined ancestor".into()))?;
        }
        let canonical_ancestor = existing.canonicalize()?;
        if !canonical_ancestor.starts_with(self.schematics.as_path()) {
            return Err(Error::Message("path escapes schematics".into()));
        }
        if !permit_missing && !candidate.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{} not found", relative),
            )
            .into());
        }
        if candidate.exists()
            && !candidate
                .canonicalize()?
                .starts_with(self.schematics.as_path())
        {
            return Err(Error::Message("path escapes schematics".into()));
        }
        Ok(candidate)
    }

    pub(crate) async fn write_generated_contract(
        &self,
        relative: &str,
        physical: &str,
        _body: &str,
        expected_revision: &str,
    ) -> Result<delivery_protocol::DocumentRevision> {
        if !(relative.starts_with("docs/") || relative.contains("/docs/"))
            || !relative.ends_with("-contract.md")
        {
            return Err(Error::Message(
                "generated contracts require a derived docs/<element>-contract.md path".into(),
            ));
        }
        let _write_guard = self.document_write_lock.lock().await;
        let path = self.resolve(relative, true)?;
        let change_kind = if path.exists() {
            schematic_mcp::DocumentChangeKind::Replaced
        } else {
            schematic_mcp::DocumentChangeKind::Created
        };
        let current = if path.exists() {
            let existing = tokio::fs::read_to_string(&path).await?;
            content_revision(
                embedding_document::parse_markdown(&existing)
                    .body
                    .as_bytes(),
            )
        } else {
            delivery_protocol::MISSING_REVISION.into()
        };
        if current != expected_revision {
            return Err(Error::RevisionConflict {
                path: relative.into(),
                expected: expected_revision.into(),
                current,
            });
        }
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        atomic_write(&path, physical.as_bytes()).await?;
        let revision = content_revision(physical.as_bytes());
        drop(_write_guard);
        let _ = schedule_embedding(self, relative.into(), physical.into()).await;
        let _ = self.notify_document_changed(relative, change_kind).await;
        Ok(delivery_protocol::DocumentRevision {
            path: relative.into(),
            revision,
        })
    }

    pub(crate) async fn generated_publication_pending(&self, relative: &str) -> bool {
        let embedding = self.embedding_status.read().await;
        let embedding_pending = embedding.path.as_deref() == Some(relative)
            && matches!(embedding.status.as_str(), "queued" | "processing");
        drop(embedding);
        let graph = self.graph_refresh_status().await;
        embedding_pending
            || matches!(
                graph.status,
                schematic_mcp::GraphRefreshStatus::Queued
                    | schematic_mcp::GraphRefreshStatus::Processing
            )
    }

    pub fn project_name(&self) -> Result<String> {
        self.project
            .file_name()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| Error::Message("project directory has no usable name".into()))
    }
}

#[derive(Deserialize)]
struct PathQuery {
    path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WriteRequest {
    path: String,
    content: String,
    revision: u64,
    expected_revision: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteRequest {
    path: String,
    expected_revision: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WriteResponse {
    path: String,
    revision: u64,
    content_revision: String,
    graph_refresh: schematic_mcp::GraphRefreshOutcome,
    embedding: EmbeddingOutcome,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CoordinatedWrite {
    path: String,
    content: String,
    expected_revision: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CoordinatedWriteRequest {
    writes: Vec<CoordinatedWrite>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CoordinatedWriteResponse {
    documents: Vec<delivery_protocol::DocumentRevision>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadResponse {
    path: String,
    content: String,
    content_revision: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbeddingOutcome {
    status: String,
    path: Option<String>,
    body_hash: Option<String>,
    diagnostic: Option<String>,
}

impl Default for EmbeddingOutcome {
    fn default() -> Self {
        Self {
            status: "current".into(),
            path: None,
            body_hash: None,
            diagnostic: None,
        }
    }
}

#[derive(Deserialize)]
struct RenameRequest {
    diagram_path: String,
    old_id: String,
    new_id: String,
}

#[derive(Deserialize)]
struct CompositionRequest {
    #[serde(default = "default_composition_kind")]
    kind: String,
    qualified_name: Option<String>,
    package_name: Option<String>,
}

fn default_composition_kind() -> String {
    "bpmn".into()
}

#[derive(Serialize)]
struct CompositionResponse {
    name: String,
    diagram: String,
    documentation: String,
    created: bool,
}

#[derive(Deserialize)]
struct CompositionRevisionRequest {
    qualified_name: String,
}

#[derive(Serialize)]
struct CompositionRevisionResponse {
    qualified_name: String,
    revision: String,
}

#[derive(Serialize)]
struct SchematicRevisionResponse {
    revision: String,
}

#[derive(Deserialize)]
struct CompositionRevertRequest {
    qualified_name: String,
    expected_revision: String,
}

#[derive(Debug, Deserialize)]
struct ProcessRenameRequest {
    old_qualified_name: String,
    new_qualified_name: String,
    #[serde(default)]
    expected_revision: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PackageRenameRequest {
    old_package_name: String,
    new_package_name: String,
}

#[derive(Serialize)]
struct ProjectMetadata {
    name: String,
    assistant_provider: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssistantProposalSubmission {
    request: assistant::AssistantRequest,
    proposal: assistant::AssistantPlan,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AssistantInboxDismissRequest {
    request_id: String,
}

pub fn app(state: AppState) -> Router {
    let web = state.project.join(".ss/web");
    Router::new()
        .route("/api/project", get(project_metadata))
        .route("/api/diagrams", get(list_diagrams))
        .route("/api/file", get(read_file).put(write_file))
        .route("/api/file-deletes", post(delete_file))
        .route("/api/files", post(write_files))
        .route("/api/graph-refresh", get(graph_refresh_status))
        .route("/api/embedding-status", get(embedding_status))
        .route("/api/rename-documentation", post(rename_documentation))
        .route("/api/compositions", post(resolve_composition))
        .route("/api/schematic-revision", get(schematic_revision))
        .route("/api/composition-revisions", post(composition_revision))
        .route("/api/composition-reverts", post(revert_created_composition))
        .route("/api/process-renames", post(rename_process))
        .route("/api/package-renames", post(rename_package))
        .route(
            "/api/assistant/proposals",
            post(assistant_proposal).layer(DefaultBodyLimit::max(512 * 1024)),
        )
        .route(
            "/api/assistant/conversations",
            post(assistant_conversation).layer(DefaultBodyLimit::max(512 * 1024)),
        )
        .route(
            "/api/assistant/inbox",
            get(list_assistant_inbox)
                .post(submit_assistant_inbox)
                .layer(DefaultBodyLimit::max(512 * 1024)),
        )
        .route(
            "/api/assistant/inbox-dismissals",
            post(dismiss_assistant_inbox),
        )
        .route("/api/assistant/capabilities", get(assistant_capabilities))
        .fallback_service(
            ServeDir::new(&web).not_found_service(ServeFile::new(web.join("index.html"))),
        )
        .with_state(state)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RuntimeAuthQuery {
    token: String,
    project_id: String,
    generation: String,
    #[serde(default)]
    after: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PlayRequestInput {
    diagram_path: String,
    source_revision: String,
    selected_entity_ref: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BuildPlanRequestInput {
    diagram_path: String,
    selected_entity_ref: Option<String>,
    semantic_bump: Option<delivery_protocol::SemanticBump>,
    expected_prior_version: Option<delivery_protocol::SemanticVersion>,
}

fn authenticate_runtime(
    runtime: &project_runtime::ProjectRuntime,
    query: &RuntimeAuthQuery,
) -> Result<()> {
    runtime.authenticate(&query.token, &query.project_id, &query.generation)
}

fn runtime_routes() -> Router {
    Router::new()
        .route("/api/runtime/health", get(runtime_health))
        .route("/api/runtime/sessions", put(runtime_session))
        .route("/api/runtime/events", get(runtime_events))
        .route(
            "/api/runtime/proposals",
            get(runtime_proposals).post(runtime_submit_proposal),
        )
        .route("/api/runtime/proposals/{id}", get(runtime_proposal))
        .route(
            "/api/runtime/proposals/{id}/decisions",
            post(runtime_decide_proposal),
        )
        .route(
            "/api/runtime/build",
            get(runtime_build).post(runtime_request_build),
        )
        .route(
            "/api/runtime/build-plans",
            get(runtime_build_plans).post(runtime_create_build_plan),
        )
        .layer(DefaultBodyLimit::max(512 * 1024))
}

async fn runtime_build_plans(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
) -> Result<Json<Vec<delivery_protocol::BuildPlanSummary>>> {
    authenticate_runtime(&runtime, &auth)?;
    Ok(Json(runtime.build_plans().await))
}

async fn runtime_create_build_plan(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
    Json(request): Json<BuildPlanRequestInput>,
) -> Result<(StatusCode, Json<delivery_protocol::BuildPlanRecord>)> {
    authenticate_runtime(&runtime, &auth)?;
    let summary = runtime.graph.summary().await;
    let project_id = runtime.project_id.to_string();
    let record = runtime
        .create_build_plan(delivery_protocol::CreateBuildPlanRequest {
            protocol_version: delivery_protocol::BUILD_PLAN_PROTOCOL_VERSION.into(),
            project_id,
            diagram_path: request.diagram_path,
            selected_entity_ref: request.selected_entity_ref,
            expected_source_manifest_revision: summary.source_manifest_revision,
            expected_graph_revision: summary.revision,
            semantic_bump: request.semantic_bump,
            expected_prior_version: request.expected_prior_version,
        })
        .await?;
    Ok((StatusCode::ACCEPTED, Json(record)))
}

async fn runtime_build(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
) -> Result<Json<Option<delivery_protocol::BuildRequest>>> {
    authenticate_runtime(&runtime, &auth)?;
    Ok(Json(runtime.build_request().await?))
}

async fn runtime_request_build(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
    Json(request): Json<PlayRequestInput>,
) -> Result<(StatusCode, Json<delivery_protocol::BuildRequest>)> {
    authenticate_runtime(&runtime, &auth)?;
    let request = runtime
        .request_build(
            request.diagram_path,
            request.source_revision,
            request.selected_entity_ref,
        )
        .await?;
    Ok((StatusCode::ACCEPTED, Json(request)))
}

async fn runtime_health(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
) -> Result<Json<delivery_protocol::DaemonHealth>> {
    authenticate_runtime(&runtime, &auth)?;
    Ok(Json(runtime.health(String::new()).await))
}

async fn runtime_session(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
    Json(session): Json<delivery_protocol::BrowserSession>,
) -> Result<StatusCode> {
    authenticate_runtime(&runtime, &auth)?;
    runtime.register_session(session).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn runtime_proposals(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
) -> Result<Json<Vec<delivery_protocol::ProposalRecord>>> {
    authenticate_runtime(&runtime, &auth)?;
    Ok(Json(runtime.proposals().await))
}

async fn runtime_proposal(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<delivery_protocol::ProposalRecord>> {
    authenticate_runtime(&runtime, &auth)?;
    runtime
        .proposal(&id)
        .await
        .map(Json)
        .ok_or_else(|| Error::Message("proposal not found".into()))
}

async fn runtime_submit_proposal(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
    Json(proposal): Json<delivery_protocol::ProposalRecord>,
) -> Result<(StatusCode, Json<delivery_protocol::ProposalRecord>)> {
    authenticate_runtime(&runtime, &auth)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(runtime.submit_proposal(proposal).await?),
    ))
}

async fn runtime_decide_proposal(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(decision): Json<delivery_protocol::ProposalDecision>,
) -> Result<Json<delivery_protocol::ProposalRecord>> {
    authenticate_runtime(&runtime, &auth)?;
    if id != decision.proposal_id {
        return Err(Error::Message("proposal path identity mismatch".into()));
    }
    Ok(Json(runtime.decide(decision).await?))
}

async fn runtime_events(
    Extension(runtime): Extension<project_runtime::ProjectRuntime>,
    Query(auth): Query<RuntimeAuthQuery>,
    headers: HeaderMap,
) -> Result<Response> {
    authenticate_runtime(&runtime, &auth)?;
    let header_cursor = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let events = runtime.events_after(auth.after.max(header_cursor)).await;
    let mut body = String::new();
    for event in events {
        body.push_str(&format!(
            "id: {}\nevent: {}\ndata: {}\n\n",
            event.event_id,
            event.kind,
            serde_json::to_string(&event).unwrap()
        ));
    }
    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "text/event-stream"),
            (axum::http::header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
        .into_response())
}

async fn assistant_capabilities() -> Json<serde_json::Value> {
    Json(assistant::operation_registry())
}

async fn submit_assistant_inbox(
    State(state): State<AppState>,
    Json(submission): Json<AssistantProposalSubmission>,
) -> Result<StatusCode> {
    assistant::validate_request(&submission.request)?;
    assistant::validate_plan(&submission.proposal, &submission.request)?;
    state
        .assistant_proposals
        .write()
        .await
        .insert(submission.request.request_id.clone(), submission);
    Ok(StatusCode::ACCEPTED)
}

async fn list_assistant_inbox(
    State(state): State<AppState>,
) -> Json<Vec<AssistantProposalSubmission>> {
    Json(
        state
            .assistant_proposals
            .read()
            .await
            .values()
            .cloned()
            .collect(),
    )
}

async fn dismiss_assistant_inbox(
    State(state): State<AppState>,
    Json(request): Json<AssistantInboxDismissRequest>,
) -> StatusCode {
    state
        .assistant_proposals
        .write()
        .await
        .remove(&request.request_id);
    StatusCode::NO_CONTENT
}

async fn assistant_proposal(
    State(state): State<AppState>,
    Json(request): Json<assistant::AssistantRequest>,
) -> Result<Json<assistant::AssistantResult>> {
    let _permit = state
        .assistant_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            Error::Message(
                "assistant request limit reached; wait for an active request to finish".into(),
            )
        })?;
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        assistant::generate(
            &request,
            state.project.as_ref().clone(),
            resolved_assistant_provider(state.project.as_path()),
        ),
    )
    .await
    .map_err(|_| Error::Message("assistant provider timed out".into()))??;
    Ok(Json(result))
}

async fn assistant_conversation(
    State(state): State<AppState>,
    Json(request): Json<assistant::AssistantConversationRequest>,
) -> Result<Json<assistant::AssistantConversationResult>> {
    let _permit = state
        .assistant_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| {
            Error::Message(
                "assistant request limit reached; wait for an active request to finish".into(),
            )
        })?;
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        assistant::converse(
            &request,
            state.project.as_ref().clone(),
            resolved_assistant_provider(state.project.as_path()),
        ),
    )
    .await
    .map_err(|_| Error::Message("assistant provider timed out".into()))??;
    Ok(Json(result))
}

async fn project_metadata(State(state): State<AppState>) -> Result<Json<ProjectMetadata>> {
    Ok(Json(ProjectMetadata {
        name: state.project_name()?,
        assistant_provider: resolved_assistant_provider(state.project.as_path()),
    }))
}

pub async fn serve(project: PathBuf, open_browser: bool) -> Result<()> {
    let project = absolute(&project)?.canonicalize()?;
    let _ = project_runtime::DaemonOwnership::reclaim_stale(&project)?;
    if let Some(discovery) = project_runtime::DaemonOwnership::healthy_discovery(&project)? {
        if open_browser {
            let launch_url = format!(
                "{}/?daemonToken={}&projectId={}&generation={}",
                discovery.http_url, discovery.token, discovery.project_id, discovery.generation
            );
            open::that(&launch_url)
                .map_err(|e| Error::Message(format!("could not open browser: {e}")))?;
        }
        return Ok(());
    }
    let runtime =
        project_runtime::ProjectRuntime::load(&project, schematic_graph::LoadOptions::default())?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let rpc_listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let address = listener.local_addr()?;
    let rpc_port = rpc_listener.local_addr()?.port();
    let url = format!("http://{address}");
    runtime.set_browser_base_url(url.clone())?;
    runtime
        .graph
        .attach_workflow(Arc::new(runtime.clone()))
        .await;
    let state = runtime.app.clone();
    let backfill_state = state.clone();
    tokio::spawn(async move {
        let _ = schedule_project_embeddings(&backfill_state).await;
    });
    let discovery = runtime.discovery(url.clone(), rpc_port);
    let _ownership = project_runtime::DaemonOwnership::acquire(&project, &discovery)?;
    let rpc_runtime = runtime.clone();
    tokio::spawn(async move {
        let _ = project_runtime::serve_private_mcp(rpc_listener, rpc_runtime).await;
    });
    println!("Software Schematic is running at {url}");
    if open_browser {
        runtime.request_browser_open()?;
    }
    let shutdown_runtime = runtime.clone();
    axum::serve(
        listener,
        app(state).merge(runtime_routes()).layer(Extension(runtime)),
    )
    .with_graceful_shutdown(shutdown_signal(shutdown_runtime))
    .await?;
    Ok(())
}

async fn shutdown_signal(runtime: project_runtime::ProjectRuntime) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler")
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    let idle = async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            runtime
                .expire_sessions(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis(),
                )
                .await;
            if runtime.idle_for(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis(),
            ) >= std::time::Duration::from_secs(30 * 60)
            {
                break;
            }
        }
    };
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {}, _ = idle => {} }
}

async fn list_diagrams(State(state): State<AppState>) -> Result<Json<Vec<String>>> {
    let mut diagrams = Vec::new();
    for entry in WalkDir::new(state.schematics.as_path()).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry.path().extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("bpmn") || ext.eq_ignore_ascii_case("cmmn")
            })
        {
            diagrams.push(
                entry
                    .path()
                    .strip_prefix(state.schematics.as_path())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    diagrams.sort();
    Ok(Json(diagrams))
}

async fn read_file(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
) -> Result<Json<ReadResponse>> {
    let path = state.resolve(&query.path, false)?;
    if !matches!(
        path.extension().and_then(|x| x.to_str()),
        Some("bpmn" | "cmmn" | "md")
    ) {
        return Err(Error::Message(
            "only BPMN, CMMN, and Markdown files are readable".into(),
        ));
    }
    let content = tokio::fs::read_to_string(path).await?;
    let authored = if query.path.ends_with(".md") {
        embedding_document::parse_markdown(&content).body
    } else {
        content
    };
    Ok(Json(ReadResponse {
        path: query.path,
        content_revision: content_revision(authored.as_bytes()),
        content: authored,
    }))
}

async fn write_file(
    State(state): State<AppState>,
    Json(request): Json<WriteRequest>,
) -> Result<Json<WriteResponse>> {
    let _write_guard = state.document_write_lock.lock().await;
    let path = state.resolve(&request.path, true)?;
    if !matches!(
        path.extension().and_then(|x| x.to_str()),
        Some("bpmn" | "cmmn" | "md")
    ) {
        return Err(Error::Message(
            "only BPMN, CMMN, and Markdown files are writable".into(),
        ));
    }
    let change_kind = if path.exists() {
        schematic_mcp::DocumentChangeKind::Replaced
    } else {
        schematic_mcp::DocumentChangeKind::Created
    };
    let authored_content = if request.path.ends_with(".md") {
        embedding_document::parse_markdown(&request.content).body
    } else {
        request.content
    };
    let current_revision = if path.exists() {
        let physical = tokio::fs::read_to_string(&path).await?;
        let current = if request.path.ends_with(".md") {
            embedding_document::parse_markdown(&physical).body
        } else {
            physical
        };
        content_revision(current.as_bytes())
    } else {
        delivery_protocol::MISSING_REVISION.into()
    };
    if request.expected_revision != current_revision {
        return Err(Error::RevisionConflict {
            path: request.path,
            expected: request.expected_revision,
            current: current_revision,
        });
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    atomic_write(&path, authored_content.as_bytes()).await?;
    let new_content_revision = content_revision(authored_content.as_bytes());
    drop(_write_guard);
    let embedding = if request.path.ends_with(".md") {
        schedule_embedding(&state, request.path.clone(), authored_content).await
    } else {
        state.embedding_status.read().await.clone()
    };
    let graph_refresh = state
        .notify_document_changed(&request.path, change_kind)
        .await;
    Ok(Json(WriteResponse {
        path: request.path,
        revision: request.revision,
        content_revision: new_content_revision,
        graph_refresh,
        embedding,
    }))
}

async fn delete_file(
    State(state): State<AppState>,
    Json(request): Json<DeleteRequest>,
) -> Result<StatusCode> {
    let _write_guard = state.document_write_lock.lock().await;
    let path = state.resolve(&request.path, false)?;
    if !matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("bpmn" | "cmmn" | "md")
    ) {
        return Err(Error::Message(
            "only BPMN, CMMN, and Markdown files are deletable".into(),
        ));
    }
    let physical = tokio::fs::read_to_string(&path).await?;
    let authored = if request.path.ends_with(".md") {
        embedding_document::parse_markdown(&physical).body
    } else {
        physical
    };
    let current = content_revision(authored.as_bytes());
    if current != request.expected_revision {
        return Err(Error::RevisionConflict {
            path: request.path,
            expected: request.expected_revision,
            current,
        });
    }
    tokio::fs::remove_file(&path).await?;
    state.embedding_jobs.lock().await.remove(&request.path);
    drop(_write_guard);
    let _ = state
        .notify_document_changed(&request.path, schematic_mcp::DocumentChangeKind::Deleted)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn write_files(
    State(state): State<AppState>,
    Json(request): Json<CoordinatedWriteRequest>,
) -> Result<Json<CoordinatedWriteResponse>> {
    let documents = coordinated_write_documents(&state, request.writes, None).await?;
    for document in &documents {
        let _ = state
            .notify_document_changed(&document.path, schematic_mcp::DocumentChangeKind::Replaced)
            .await;
    }
    Ok(Json(CoordinatedWriteResponse { documents }))
}

async fn coordinated_write_documents(
    state: &AppState,
    writes: Vec<CoordinatedWrite>,
    fail_after: Option<usize>,
) -> Result<Vec<delivery_protocol::DocumentRevision>> {
    if writes.is_empty() {
        return Err(Error::Message(
            "coordinated write requires at least one document".into(),
        ));
    }
    let _guard = state.document_write_lock.lock().await;
    let mut prepared = Vec::with_capacity(writes.len());
    let mut seen = std::collections::BTreeSet::new();
    for write in writes {
        if !seen.insert(write.path.clone()) {
            return Err(Error::Message(format!(
                "duplicate coordinated write path: {}",
                write.path
            )));
        }
        let path = state.resolve(&write.path, true)?;
        if !matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("bpmn" | "cmmn" | "md")
        ) {
            return Err(Error::Message(
                "only BPMN, CMMN, and Markdown files are writable".into(),
            ));
        }
        let before = if path.exists() {
            Some(tokio::fs::read(&path).await?)
        } else {
            None
        };
        let authored_before = before.as_ref().map(|bytes| {
            if write.path.ends_with(".md") {
                embedding_document::parse_markdown(&String::from_utf8_lossy(bytes))
                    .body
                    .into_bytes()
            } else {
                bytes.clone()
            }
        });
        let current_revision = authored_before
            .as_deref()
            .map(content_revision)
            .unwrap_or_else(|| delivery_protocol::MISSING_REVISION.into());
        if write.expected_revision != current_revision {
            return Err(Error::RevisionConflict {
                path: write.path,
                expected: write.expected_revision,
                current: current_revision,
            });
        }
        let content = if path.extension().and_then(|value| value.to_str()) == Some("md") {
            embedding_document::parse_markdown(&write.content)
                .body
                .into_bytes()
        } else {
            write.content.into_bytes()
        };
        prepared.push((write.path, path, before, content));
    }
    prepared.sort_by(|left, right| left.0.cmp(&right.0));

    for (committed, (_, path, _, content)) in prepared.iter().enumerate() {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let result = if fail_after == Some(committed) {
            Err(Error::Message("injected coordinated write failure".into()))
        } else {
            atomic_write(path, content).await
        };
        if let Err(error) = result {
            for (_, rollback_path, before, _) in prepared[..committed].iter().rev() {
                match before {
                    Some(bytes) => atomic_write(rollback_path, bytes).await?,
                    None if rollback_path.exists() => tokio::fs::remove_file(rollback_path).await?,
                    None => {}
                }
            }
            return Err(error);
        }
    }

    Ok(prepared
        .into_iter()
        .map(
            |(path, _, _, content)| delivery_protocol::DocumentRevision {
                path,
                revision: content_revision(&content),
            },
        )
        .collect())
}

pub fn content_revision(content: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("sha256:{:x}", Sha256::digest(content))
}

async fn graph_refresh_status(
    State(state): State<AppState>,
) -> Json<schematic_mcp::GraphRefreshOutcome> {
    Json(state.graph_refresh_status().await)
}

async fn embedding_status(State(state): State<AppState>) -> Json<EmbeddingOutcome> {
    Json(state.embedding_status.read().await.clone())
}

async fn schedule_embedding(state: &AppState, relative: String, body: String) -> EmbeddingOutcome {
    if body.trim().is_empty() {
        state.embedding_jobs.lock().await.remove(&relative);
        let current = EmbeddingOutcome {
            status: "current".into(),
            path: Some(relative),
            body_hash: Some(embedding_document::body_hash(&body)),
            diagnostic: None,
        };
        *state.embedding_status.write().await = current.clone();
        return current;
    }
    let generation = state.embedding_generation.fetch_add(1, Ordering::SeqCst) + 1;
    state
        .embedding_jobs
        .lock()
        .await
        .insert(relative.clone(), generation);
    let body_hash = embedding_document::body_hash(&body);
    let queued = EmbeddingOutcome {
        status: "queued".into(),
        path: Some(relative.clone()),
        body_hash: Some(body_hash.clone()),
        diagnostic: None,
    };
    *state.embedding_status.write().await = queued.clone();
    let worker = state.clone();
    tokio::spawn(async move {
        let Ok(_permit) = worker.embedding_slots.clone().acquire_owned().await else {
            return;
        };
        if worker.embedding_jobs.lock().await.get(&relative).copied() != Some(generation) {
            return;
        }
        *worker.embedding_status.write().await = EmbeddingOutcome {
            status: "processing".into(),
            path: Some(relative.clone()),
            body_hash: Some(body_hash.clone()),
            diagnostic: None,
        };
        let project = worker.project.as_ref().clone();
        let owner = match markdown_owner(worker.schematics.as_path(), &relative) {
            Ok(value) => value,
            Err(error) => {
                *worker.embedding_status.write().await = EmbeddingOutcome {
                    status: "failed".into(),
                    path: Some(relative),
                    body_hash: Some(body_hash),
                    diagnostic: Some(error.to_string()),
                };
                return;
            }
        };
        let body_for_derivation = body.clone();
        let derived = tokio::task::spawn_blocking(move || {
            schematic_graph::derive_embedding_envelope(
                &project,
                &owner,
                &body_for_derivation,
                &schematic_graph::LoadOptions::default(),
            )
        })
        .await;
        let outcome = match derived {
            Ok(Ok(envelope)) => {
                publish_embedding_header(&worker, &relative, generation, &body, &envelope).await
            }
            Ok(Err(error)) => Err(error),
            Err(error) => Err(Error::Message(format!("embedding task failed: {error}"))),
        };
        match outcome {
            Ok(true) => {
                *worker.embedding_status.write().await = EmbeddingOutcome {
                    status: "current".into(),
                    path: Some(relative),
                    body_hash: Some(body_hash),
                    diagnostic: None,
                };
            }
            Ok(false) => {}
            Err(error) => {
                worker.embedding_jobs.lock().await.remove(&relative);
                *worker.embedding_status.write().await = EmbeddingOutcome {
                    status: "failed".into(),
                    path: Some(relative),
                    body_hash: Some(body_hash),
                    diagnostic: Some(error.to_string()),
                };
            }
        }
    });
    queued
}

async fn schedule_project_embeddings(state: &AppState) -> Result<usize> {
    let documents = WalkDir::new(state.schematics.as_path())
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|value| value.to_str()) == Some("md")
        })
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    let mut scheduled = 0;
    for path in documents {
        let physical = tokio::fs::read_to_string(&path).await?;
        let parsed = embedding_document::parse_markdown(&physical);
        if parsed.body.trim().is_empty() {
            continue;
        }
        let relative = path
            .strip_prefix(state.schematics.as_path())
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let current = parsed.envelope.as_ref().is_some_and(|envelope| {
            envelope.body_hash == embedding_document::body_hash(&parsed.body)
                && envelope.model == schematic_graph::EMBEDDING_MODEL
                && embedding_document::validate_envelope_shape(envelope).is_ok()
        });
        if !current {
            schedule_embedding(state, relative, parsed.body).await;
            scheduled += 1;
        }
    }
    Ok(scheduled)
}

pub async fn backfill_embeddings(project: impl AsRef<Path>) -> Result<usize> {
    let state = AppState::new(project)?;
    let scheduled = schedule_project_embeddings(&state).await?;
    while !state.embedding_jobs.lock().await.is_empty() {
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    let status = state.embedding_status.read().await.clone();
    if status.status == "failed" {
        return Err(Error::Message(
            status
                .diagnostic
                .unwrap_or_else(|| "embedding backfill failed".into()),
        ));
    }
    Ok(scheduled)
}

async fn publish_embedding_header(
    state: &AppState,
    relative: &str,
    generation: u64,
    expected_body: &str,
    envelope: &embedding_document::EmbeddingEnvelope,
) -> Result<bool> {
    if state.embedding_jobs.lock().await.get(relative).copied() != Some(generation) {
        return Ok(false);
    }
    let path = state.resolve(relative, false)?;
    let current = tokio::fs::read_to_string(&path).await?;
    let parsed = embedding_document::parse_markdown(&current);
    if parsed.body != expected_body
        || envelope.body_hash != embedding_document::body_hash(&parsed.body)
    {
        return Ok(false);
    }
    let physical = embedding_document::serialize_markdown(envelope, &parsed.body)?;
    if physical != current {
        atomic_write(&path, physical.as_bytes()).await?;
    }
    state.embedding_jobs.lock().await.remove(relative);
    let _ = state
        .notify_document_changed(relative, schematic_mcp::DocumentChangeKind::Replaced)
        .await;
    Ok(true)
}

fn markdown_owner(schematics: &Path, relative: &str) -> Result<String> {
    let relative_path = Path::new(relative);
    if relative_path.extension().and_then(|value| value.to_str()) != Some("md") {
        return Err(Error::Message("embedding source must be Markdown".into()));
    }
    let file = relative_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let (diagram_folder, suffix) = if file == "main.md" {
        (
            relative_path.parent().unwrap_or(Path::new("")),
            "diagram".to_owned(),
        )
    } else if relative_path
        .parent()
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        == Some("docs")
    {
        (
            relative_path
                .parent()
                .and_then(Path::parent)
                .unwrap_or(Path::new("")),
            relative_path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_owned(),
        )
    } else {
        return Err(Error::Message(
            "Markdown is not connected to a schematic owner".into(),
        ));
    };
    let diagram = ["cmmn", "bpmn"]
        .iter()
        .map(|extension| diagram_folder.join(format!("main.{extension}")))
        .find(|candidate| schematics.join(candidate).is_file())
        .ok_or_else(|| Error::Message("Markdown owner diagram does not exist".into()))?;
    Ok(format!(
        "{}#{suffix}",
        diagram.to_string_lossy().replace('\\', "/")
    ))
}

async fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| Error::Message("invalid filename".into()))?;
    let temporary = path.with_file_name(format!(".{file_name}.{}.tmp", std::process::id()));
    tokio::fs::write(&temporary, content).await?;
    #[cfg(windows)]
    if path.exists() {
        tokio::fs::remove_file(path).await?;
    }
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

async fn rename_documentation(
    State(state): State<AppState>,
    Json(request): Json<RenameRequest>,
) -> Result<StatusCode> {
    validate_id(&request.old_id)?;
    validate_id(&request.new_id)?;
    let diagram = state.resolve(&request.diagram_path, false)?;
    let folder = diagram
        .parent()
        .ok_or_else(|| Error::Message("diagram has no composition folder".into()))?;
    let old = folder.join("docs").join(format!("{}.md", request.old_id));
    let new = folder.join("docs").join(format!("{}.md", request.new_id));
    if new.exists() {
        return Err(Error::Message("documentation target already exists".into()));
    }
    if old.exists() {
        tokio::fs::rename(&old, &new).await?;
        let relative = new
            .strip_prefix(state.schematics.as_path())
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let physical = tokio::fs::read_to_string(&new).await?;
        let body = embedding_document::parse_markdown(&physical).body;
        atomic_write(&new, body.as_bytes()).await?;
        schedule_embedding(&state, relative.clone(), body).await;
        let _ = state
            .notify_document_changed(&relative, schematic_mcp::DocumentChangeKind::Renamed)
            .await;
    }
    Ok(StatusCode::NO_CONTENT)
}

fn validate_id(id: &str) -> Result<()> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
    {
        return Err(Error::Message(
            "element ID must contain only letters, digits, underscore, or hyphen".into(),
        ));
    }
    Ok(())
}

async fn resolve_composition(
    State(state): State<AppState>,
    Json(request): Json<CompositionRequest>,
) -> Result<Json<CompositionResponse>> {
    let _write_guard = state.document_write_lock.lock().await;
    let (name, relative_folder, extension, starter) =
        match request.kind.as_str() {
            "bpmn" => {
                let name = request.qualified_name.as_deref().ok_or_else(|| {
                    Error::Message("BPMN composition requires qualified_name".into())
                })?;
                let folder = composition_folder_for_name(name)?
                    .to_string_lossy()
                    .replace('\\', "/");
                (
                    name.to_owned(),
                    folder,
                    "bpmn",
                    set_definitions_process_name(STARTER_BPMN, name),
                )
            }
            "cmmn" => {
                let name = request.package_name.as_deref().ok_or_else(|| {
                    Error::Message("CMMN composition requires package_name".into())
                })?;
                validate_package_name(name)?;
                let folder = name
                    .split('.')
                    .collect::<PathBuf>()
                    .to_string_lossy()
                    .replace('\\', "/");
                (
                    name.to_owned(),
                    folder,
                    "cmmn",
                    set_definitions_package_name(STARTER_CMMN, name),
                )
            }
            _ => {
                return Err(Error::Message(
                    "composition kind must be bpmn or cmmn".into(),
                ));
            }
        };
    let folder = state.resolve(&relative_folder, true)?;
    let diagram = folder.join(format!("main.{extension}"));
    let documentation = folder.join("main.md");
    let created = !diagram.exists();
    if created {
        tokio::fs::create_dir_all(&folder).await?;
        atomic_write(&diagram, starter.as_bytes()).await?;
        if !documentation.exists() {
            atomic_write(&documentation, STARTER_MD.as_bytes()).await?;
        }
        let relative_diagram = diagram
            .strip_prefix(state.schematics.as_path())
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let _ = state
            .notify_document_changed(
                &relative_diagram,
                schematic_mcp::DocumentChangeKind::Created,
            )
            .await;
    }
    let relative = |path: &Path| {
        path.strip_prefix(state.schematics.as_path())
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/")
    };
    Ok(Json(CompositionResponse {
        name,
        diagram: relative(&diagram),
        documentation: relative(&documentation),
        created,
    }))
}

fn composition_tree_revision(folder: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};

    if !folder.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{} not found", folder.display()),
        )
        .into());
    }
    let mut files = Vec::new();
    for entry in WalkDir::new(folder).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            return Err(Error::Message(
                "composition revisions do not follow symbolic links".into(),
            ));
        }
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort_by(|left, right| {
        left.strip_prefix(folder)
            .unwrap()
            .cmp(right.strip_prefix(folder).unwrap())
    });
    let mut digest = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(folder).unwrap().to_string_lossy();
        digest.update(relative.as_bytes());
        digest.update([0]);
        let bytes = fs::read(&path)?;
        if path.extension().and_then(|value| value.to_str()) == Some("md") {
            let physical = String::from_utf8(bytes)
                .map_err(|_| Error::Message("composition Markdown must be UTF-8".into()))?;
            digest.update(embedding_document::parse_markdown(&physical).body);
        } else {
            digest.update(bytes);
        }
        digest.update([0]);
    }
    Ok(format!("sha256:{:x}", digest.finalize()))
}

async fn composition_revision(
    State(state): State<AppState>,
    Json(request): Json<CompositionRevisionRequest>,
) -> Result<Json<CompositionRevisionResponse>> {
    let relative = composition_folder_for_name(&request.qualified_name)?
        .to_string_lossy()
        .replace('\\', "/");
    let folder = state.resolve(&relative, false)?;
    Ok(Json(CompositionRevisionResponse {
        qualified_name: request.qualified_name,
        revision: composition_tree_revision(&folder)?,
    }))
}

async fn schematic_revision(
    State(state): State<AppState>,
) -> Result<Json<SchematicRevisionResponse>> {
    Ok(Json(SchematicRevisionResponse {
        revision: composition_tree_revision(state.schematics.as_path())?,
    }))
}

async fn revert_created_composition(
    State(state): State<AppState>,
    Json(request): Json<CompositionRevertRequest>,
) -> Result<StatusCode> {
    let relative = composition_folder_for_name(&request.qualified_name)?
        .to_string_lossy()
        .replace('\\', "/");
    let _write_guard = state.document_write_lock.lock().await;
    let folder = state.resolve(&relative, false)?;
    let current = composition_tree_revision(&folder)?;
    if current != request.expected_revision {
        return Err(Error::RevisionConflict {
            path: relative,
            expected: request.expected_revision,
            current,
        });
    }

    let file_name = folder
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| Error::Message("composition folder has no valid name".into()))?;
    let tombstone = folder.with_file_name(format!(
        ".{file_name}.assistant-revert-{}",
        std::process::id()
    ));
    if tombstone.exists() {
        return Err(Error::Message(
            "a previous composition revert requires recovery".into(),
        ));
    }
    tokio::fs::rename(&folder, &tombstone).await?;
    let staged_revision = composition_tree_revision(&tombstone)?;
    if staged_revision != request.expected_revision {
        tokio::fs::rename(&tombstone, &folder).await?;
        return Err(Error::RevisionConflict {
            path: relative,
            expected: request.expected_revision,
            current: staged_revision,
        });
    }
    if let Err(error) = tokio::fs::remove_dir_all(&tombstone).await {
        let _ = tokio::fs::rename(&tombstone, &folder).await;
        return Err(error.into());
    }
    let deleted = format!("{relative}/main.bpmn");
    let _ = state
        .notify_document_changed(&deleted, schematic_mcp::DocumentChangeKind::Deleted)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn rename_package(
    State(state): State<AppState>,
    Json(request): Json<PackageRenameRequest>,
) -> Result<StatusCode> {
    let _write_guard = state.document_write_lock.lock().await;
    validate_package_name(&request.old_package_name)?;
    validate_package_name(&request.new_package_name)?;
    let old_folder = request.old_package_name.replace('.', "/");
    let new_folder = request.new_package_name.replace('.', "/");
    let old = state.resolve(&old_folder, false)?;
    let new = state.resolve(&new_folder, true)?;
    if new.exists() {
        return Err(Error::Message("package Name already exists".into()));
    }
    if !old.join("main.cmmn").exists() {
        return Err(Error::Message(
            "source package has no main.cmmn business anchor".into(),
        ));
    }
    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&old, &new)?;
    rewrite_local_names(
        state.schematics.as_path(),
        &request.old_package_name,
        &request.new_package_name,
    )?;
    let renamed = format!("{new_folder}/main.cmmn");
    let _ = state
        .notify_document_changed(&renamed, schematic_mcp::DocumentChangeKind::Renamed)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

fn rewrite_local_names(root: &Path, old_name: &str, new_name: &str) -> Result<()> {
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("bpmn") || x.eq_ignore_ascii_case("cmmn"))
        {
            let content = fs::read_to_string(entry.path())?;
            let rewritten = content.replace(old_name, new_name);
            if rewritten != content {
                fs::write(entry.path(), rewritten)?;
            }
        }
    }
    Ok(())
}

async fn rename_process(
    State(state): State<AppState>,
    Json(request): Json<ProcessRenameRequest>,
) -> Result<StatusCode> {
    let _write_guard = state.document_write_lock.lock().await;
    validate_qualified_process_name(&request.old_qualified_name)?;
    validate_qualified_process_name(&request.new_qualified_name)?;
    if let Some(expected) = request.expected_revision.as_deref() {
        let current = composition_tree_revision(state.schematics.as_path())?;
        if current != expected {
            return Err(Error::RevisionConflict {
                path: ".".into(),
                expected: expected.into(),
                current,
            });
        }
    }
    let old_folder = composition_folder_for_name(&request.old_qualified_name)?
        .to_string_lossy()
        .replace('\\', "/");
    let new_folder = composition_folder_for_name(&request.new_qualified_name)?
        .to_string_lossy()
        .replace('\\', "/");
    let old = state.resolve(&old_folder, false)?;
    let new = state.resolve(&new_folder, true)?;
    if new.exists() {
        return Err(Error::Message("process Name already exists".into()));
    }
    if let Some(parent) = new.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&old, &new)?;
    rewrite_local_names(
        state.schematics.as_path(),
        &request.old_qualified_name,
        &request.new_qualified_name,
    )?;
    let renamed = format!("{new_folder}/main.bpmn");
    let _ = state
        .notify_document_changed(&renamed, schematic_mcp::DocumentChangeKind::Renamed)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

fn set_definitions_process_name(xml: &str, qualified_name: &str) -> String {
    if let Some(start) = xml.find("<bpmn:definitions")
        && let Some(end_offset) = xml[start..].find('>')
    {
        let end = start + end_offset;
        let tag = &xml[start..end];
        if tag.contains("ssw:processName=\"") {
            let marker = "ssw:processName=\"";
            let value_start = start + tag.find(marker).unwrap() + marker.len();
            if let Some(value_end) = xml[value_start..].find('"') {
                let mut result = xml.to_owned();
                result.replace_range(value_start..value_start + value_end, qualified_name);
                return result;
            }
        }
        let namespace = if tag.contains("xmlns:ssw=") {
            ""
        } else {
            " xmlns:ssw=\"https://software-schematic.dev/schema/bpmn\""
        };
        let mut result = xml.to_owned();
        result.insert_str(
            end,
            &format!("{namespace} ssw:processName=\"{qualified_name}\""),
        );
        return result;
    }
    xml.to_owned()
}

fn set_definitions_package_name(xml: &str, package_name: &str) -> String {
    if let Some(start) = xml.find("<cmmn:definitions")
        && let Some(end_offset) = xml[start..].find('>')
    {
        let end = start + end_offset;
        let tag = &xml[start..end];
        let namespace = if tag.contains("xmlns:ssw=") {
            ""
        } else {
            " xmlns:ssw=\"https://software-schematic.dev/schema/cmmn\""
        };
        let attribute = format!(" ssw:packageName=\"{package_name}\"");
        let mut result = xml.to_owned();
        result.insert_str(end, &format!("{namespace}{attribute}"));
        return result;
    }
    xml.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tempfile::tempdir;
    use tower::ServiceExt;

    fn initialized() -> (tempfile::TempDir, ProjectLayout) {
        let directory = tempdir().unwrap();
        let layout = init_project(directory.path()).unwrap();
        (directory, layout)
    }

    #[test]
    fn scoped_names_validate_resolve_and_map_to_portable_folders() {
        assert_eq!(
            validate_package_name("cybling.subscription").unwrap(),
            "cybling.subscription"
        );
        assert_eq!(
            validate_process_name("SelectAndOutfit").unwrap(),
            "SelectAndOutfit"
        );
        assert_eq!(
            validate_member_name("outfitCybling").unwrap(),
            "outfitCybling"
        );
        assert!(validate_package_name("Cybling.subscription").is_err());
        assert!(validate_process_name("selectAndOutfit").is_err());
        assert!(validate_member_name("OutfitCybling").is_err());
        assert_eq!(
            validate_element_name("cybling.subscription.SelectAndOutfit#outfitCybling").unwrap(),
            "cybling.subscription.SelectAndOutfit#outfitCybling"
        );
        assert_eq!(
            validate_element_name("cybling.subscription.SelectAndOutfit").unwrap(),
            "cybling.subscription.SelectAndOutfit"
        );
        assert!(validate_element_name("SelectAndOutfit#outfitCybling").is_err());
        let folder = composition_folder_for_name("cybling.subscription.SelectAndOutfit").unwrap();
        assert_eq!(
            folder,
            PathBuf::from("cybling/subscription/SelectAndOutfit")
        );
        assert_eq!(
            qualified_name_for_composition_folder(&folder).unwrap(),
            "cybling.subscription.SelectAndOutfit"
        );
        assert_eq!(
            symbol_collision_key("ConfigureSubscription"),
            symbol_collision_key("configuresubscription")
        );
        assert!(qualified_name_for_composition_folder(Path::new("../private/Process")).is_err());
    }

    #[test]
    fn initialization_writes_complete_versioned_layout_and_refuses_collisions() {
        let (directory, layout) = initialized();
        assert!(layout.tool.join("web/index.html").is_file());
        assert!(layout.tool.join("operation-plan.schema.json").is_file());
        assert!(layout.tool.join("starter.cmmn").is_file());
        assert_eq!(
            fs::read_to_string(layout.tool.join("LICENSE")).unwrap(),
            PROJECT_LICENSE
        );
        assert_eq!(
            fs::read_to_string(layout.tool.join("NOTICE")).unwrap(),
            PROJECT_NOTICE
        );
        assert_eq!(
            fs::read_to_string(layout.tool.join("THIRD_PARTY_NOTICES.md")).unwrap(),
            THIRD_PARTY_NOTICES
        );
        assert!(layout.tool.join("web/vendor").is_dir());
        assert!(layout.schematics.join("main.cmmn").is_file());
        assert!(!layout.schematics.join("main.bpmn").exists());
        assert!(layout.schematics.join("main.md").is_file());
        assert_eq!(
            fs::read_to_string(layout.tool.join("version")).unwrap(),
            format!("{}\n", env!("CARGO_PKG_VERSION"))
        );
        assert!(directory.path().join("ssw.cmd").is_file());
        assert!(directory.path().join(".ss/project-id").is_file());
        assert!(
            fs::read_to_string(directory.path().join("AGENTS.md"))
                .unwrap()
                .contains("deep interview with frequent small visual proposals")
        );
        for skill in ["design", "plan", "build"] {
            let codex_skill = directory
                .path()
                .join(".codex/skills")
                .join(skill)
                .join("SKILL.md");
            let claude_skill = directory
                .path()
                .join(".claude/commands")
                .join(format!("{skill}.md"));
            assert!(codex_skill.is_file());
            assert!(claude_skill.is_file());
            let contents = fs::read_to_string(codex_skill).unwrap();
            assert!(!contents.contains(directory.path().to_string_lossy().as_ref()));
            assert!(!contents.to_ascii_lowercase().contains("api_key"));
        }
        let codex = fs::read_to_string(directory.path().join(".codex/config.toml")).unwrap();
        assert!(codex.contains("software_schematic"));
        assert!(codex.contains("command = \"./ssw\""));
        assert!(
            fs::read_to_string(directory.path().join("ssw"))
                .unwrap()
                .contains("auth --project")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_ne!(
                fs::metadata(directory.path().join("ssw"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o111,
                0
            );
        }
        assert!(
            init_project(directory.path())
                .unwrap_err()
                .to_string()
                .contains("already exists")
        );
    }

    #[test]
    fn update_preserves_authored_files_and_is_idempotent_with_crlf() {
        let (directory, layout) = initialized();
        let diagram_before = fs::read(layout.schematics.join("main.cmmn")).unwrap();
        let workflow_path = layout.tool.join("workflows/interviews/authored.json");
        fs::write(&workflow_path, "{\"authored\":true}\n").unwrap();
        #[cfg(unix)]
        let runtime_inode_before = {
            use std::os::unix::fs::MetadataExt;
            fs::metadata(layout.tool.join("bin/ss")).unwrap().ino()
        };
        fs::write(layout.tool.join("NOTICE"), "stale managed notice").unwrap();
        fs::write(directory.path().join("AGENTS.md"), "authored\r\n\r\n<!-- software-schematic:begin -->\r\nstale\r\n<!-- software-schematic:end -->\r\ntail\r\n").unwrap();
        fs::write(directory.path().join(".codex/config.toml"), "theme = \"dark\"\n\n[mcp_servers.other]\ncommand = \"other\"\n\n[mcp_servers.software_schematic]\ncommand = \"stale\"\n").unwrap();
        update_project(directory.path()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_ne!(
                runtime_inode_before,
                fs::metadata(layout.tool.join("bin/ss")).unwrap().ino(),
                "update must atomically replace rather than overwrite the executable inode"
            );
        }
        let once_agents = fs::read(directory.path().join("AGENTS.md")).unwrap();
        let once_codex = fs::read(directory.path().join(".codex/config.toml")).unwrap();
        update_project(directory.path()).unwrap();
        assert_eq!(
            once_agents,
            fs::read(directory.path().join("AGENTS.md")).unwrap()
        );
        assert_eq!(
            once_codex,
            fs::read(directory.path().join(".codex/config.toml")).unwrap()
        );
        assert!(once_agents.starts_with(b"authored\r\n"));
        assert!(
            String::from_utf8(once_agents)
                .unwrap()
                .ends_with("tail\r\n")
        );
        let once_codex = String::from_utf8(once_codex).unwrap();
        assert!(once_codex.contains("theme = \"dark\""));
        assert!(once_codex.contains("[mcp_servers.other]"));
        assert!(once_codex.contains("command = \"./ssw\""));
        assert_eq!(
            diagram_before,
            fs::read(layout.schematics.join("main.cmmn")).unwrap()
        );
        assert_eq!(
            fs::read_to_string(layout.tool.join("NOTICE")).unwrap(),
            PROJECT_NOTICE
        );
        assert_eq!(
            fs::read_to_string(workflow_path).unwrap(),
            "{\"authored\":true}\n"
        );
        assert!(
            fs::read_to_string(directory.path().join(".codex/skills/design/SKILL.md"))
                .unwrap()
                .contains("open_design_workspace")
        );
    }

    #[test]
    fn doctor_repair_restores_guided_skill_without_touching_authored_state() {
        let (directory, layout) = initialized();
        let markdown = layout.schematics.join("main.md");
        fs::write(&markdown, "# Authored model\nKeep this.\n").unwrap();
        let workflow = layout.tool.join("workflows/interviews/authored.json");
        fs::write(&workflow, "{\"authored\":true}\n").unwrap();
        let unrelated = directory.path().join(".codex/unrelated.txt");
        fs::write(&unrelated, "keep\n").unwrap();
        fs::remove_file(directory.path().join(".codex/skills/design/SKILL.md")).unwrap();

        let report = doctor_project(directory.path(), true).unwrap();
        assert_eq!(report.status, "ready");
        assert!(report.repaired);
        assert!(
            fs::read_to_string(directory.path().join(".codex/skills/design/SKILL.md"))
                .unwrap()
                .contains("open_design_workspace")
        );
        assert_eq!(
            fs::read_to_string(markdown).unwrap(),
            "# Authored model\nKeep this.\n"
        );
        assert_eq!(
            fs::read_to_string(workflow).unwrap(),
            "{\"authored\":true}\n"
        );
        assert_eq!(fs::read_to_string(unrelated).unwrap(), "keep\n");
    }

    #[test]
    fn update_refuses_incomplete_managed_guidance_and_invalid_toml() {
        let (directory, _) = initialized();
        fs::write(
            directory.path().join("AGENTS.md"),
            "authored\n<!-- software-schematic:begin -->\nstale\n",
        )
        .unwrap();
        let before = fs::read(directory.path().join("AGENTS.md")).unwrap();
        assert!(
            update_project(directory.path())
                .unwrap_err()
                .to_string()
                .contains("incomplete")
        );
        assert_eq!(
            before,
            fs::read(directory.path().join("AGENTS.md")).unwrap()
        );

        fs::write(directory.path().join("AGENTS.md"), "authored\n").unwrap();
        fs::write(directory.path().join(".codex/config.toml"), "not = [valid").unwrap();
        assert!(
            update_project(directory.path())
                .unwrap_err()
                .to_string()
                .contains("invalid .codex/config.toml")
        );
        assert_eq!(
            fs::read_to_string(directory.path().join(".codex/config.toml")).unwrap(),
            "not = [valid"
        );
    }

    #[test]
    fn initialized_project_loads_the_packaged_embedding_model() {
        let (directory, layout) = initialized();
        assert!(
            layout
                .tool
                .join("models/all-MiniLM-L6-v2/all-MiniLM-L6-v2.onnx")
                .is_file()
        );
        assert!(
            layout
                .tool
                .join("models/all-MiniLM-L6-v2/tokenizer.json")
                .is_file()
        );
        let snapshot = crate::schematic_graph::load_schematic_graph(
            directory.path(),
            crate::schematic_graph::LoadOptions::default(),
        )
        .unwrap();
        assert_eq!(snapshot.summary.embedding_model, "all-MiniLM-L6-v2");
    }

    #[test]
    fn project_assistant_configuration_contains_only_the_provider_selection() {
        let (directory, _) = initialized();
        let path = directory.path().join(".ss/assistant.json");
        fs::write(&path, br#"{"provider":"codex"}"#).unwrap();
        assert_eq!(
            configured_assistant_provider(directory.path()).as_deref(),
            Some("codex")
        );
        let content = fs::read_to_string(path).unwrap();
        assert!(!content.to_ascii_lowercase().contains("token"));
        assert!(!content.to_ascii_lowercase().contains("password"));
    }

    #[test]
    fn confinement_rejects_traversal_absolute_and_symlink_escape() {
        let (directory, _) = initialized();
        let state = AppState::new(directory.path()).unwrap();
        assert!(state.resolve("../private.md", true).is_err());
        assert!(state.resolve("/private.md", true).is_err());
        assert!(state.resolve("docs/Task_1.md", true).is_ok());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                directory.path().parent().unwrap(),
                directory.path().join("schematics/link"),
            )
            .unwrap();
            assert!(state.resolve("link/secret.md", true).is_err());
        }
    }

    async fn send(
        router: Router,
        method: &str,
        uri: &str,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, Vec<u8>) {
        let mut builder = Request::builder().method(method).uri(uri);
        let request_body = if let Some(value) = body {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        } else {
            Body::empty()
        };
        let response = router
            .oneshot(builder.body(request_body).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap()
            .to_vec();
        (status, bytes)
    }

    #[tokio::test]
    async fn typed_file_endpoints_replace_complete_content_and_list_diagrams() {
        let (directory, _) = initialized();
        let router = app(AppState::new(directory.path()).unwrap());
        let (status, bytes) = send(
            router.clone(),
            "PUT",
            "/api/file",
            Some(serde_json::json!({
                "path": "docs/Task_1.md", "content": "complete documentation", "revision": 7,
                "expectedRevision": "missing"
            })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let response: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(response["path"], "docs/Task_1.md");
        assert_eq!(response["revision"], 7);
        assert_eq!(
            response["contentRevision"],
            content_revision(b"complete documentation")
        );
        assert_eq!(response["graphRefresh"]["status"], "notRunning");
        let (status, bytes) = send(
            router.clone(),
            "GET",
            "/api/file?path=docs%2FTask_1.md",
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let response: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(response["content"], "complete documentation");
        assert_eq!(
            response["contentRevision"],
            content_revision(b"complete documentation")
        );
        let (status, bytes) = send(router.clone(), "GET", "/api/diagrams", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(String::from_utf8(bytes).unwrap().contains("main.cmmn"));
        let (status, bytes) = send(router, "GET", "/api/graph-refresh", None).await;
        assert_eq!(status, StatusCode::OK);
        let response: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(response["status"], "notRunning");
    }

    #[test]
    fn content_revisions_are_strong_and_content_addressed() {
        assert_eq!(
            content_revision(b"same bytes"),
            content_revision(b"same bytes")
        );
        assert_ne!(
            content_revision(b"same bytes"),
            content_revision(b"different bytes")
        );
        assert!(content_revision(b"same bytes").starts_with("sha256:"));
    }

    #[tokio::test]
    async fn conditional_writes_reject_stale_revisions_without_losing_newer_content() {
        let (directory, layout) = initialized();
        let path = layout.schematics.join("main.md");
        let base = embedding_document::parse_markdown(&fs::read_to_string(&path).unwrap()).body;
        let base_revision = content_revision(base.as_bytes());
        let router = app(AppState::new(directory.path()).unwrap());

        let (status, bytes) = send(
            router.clone(),
            "PUT",
            "/api/file",
            Some(serde_json::json!({
                "path": "main.md", "content": "newer", "revision": 1,
                "expectedRevision": base_revision
            })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let accepted: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        let (status, bytes) = send(
            router,
            "PUT",
            "/api/file",
            Some(serde_json::json!({
                "path": "main.md", "content": "stale overwrite", "revision": 2,
                "expectedRevision": base_revision
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        let conflict: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(conflict["code"], "revisionConflict");
        assert_eq!(conflict["currentRevision"], accepted["contentRevision"]);
        assert_eq!(fs::read_to_string(path).unwrap(), "newer");
    }

    #[tokio::test]
    async fn conditional_delete_restores_missing_without_removing_newer_content() {
        let (directory, layout) = initialized();
        let path = layout.schematics.join("docs/Temporary.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "assistant annotation").unwrap();
        let original_revision = content_revision(b"assistant annotation");
        let router = app(AppState::new(directory.path()).unwrap());

        fs::write(&path, "newer user annotation").unwrap();
        let (status, _) = send(
            router.clone(),
            "POST",
            "/api/file-deletes",
            Some(serde_json::json!({
                "path": "docs/Temporary.md",
                "expectedRevision": original_revision
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(fs::read_to_string(&path).unwrap(), "newer user annotation");

        let (status, _) = send(
            router,
            "POST",
            "/api/file-deletes",
            Some(serde_json::json!({
                "path": "docs/Temporary.md",
                "expectedRevision": content_revision(b"newer user annotation")
            })),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn coordinated_writes_roll_back_every_document_after_mid_commit_failure() {
        let (directory, layout) = initialized();
        let state = AppState::new(directory.path()).unwrap();
        let diagram_path = layout.schematics.join("main.cmmn");
        let markdown_path = layout.schematics.join("main.md");
        let diagram_before = fs::read(&diagram_path).unwrap();
        let markdown_physical_before = fs::read(&markdown_path).unwrap();
        let markdown_before =
            embedding_document::parse_markdown(&String::from_utf8_lossy(&markdown_physical_before))
                .body;
        let result = coordinated_write_documents(
            &state,
            vec![
                CoordinatedWrite {
                    path: "main.md".into(),
                    content: "changed markdown".into(),
                    expected_revision: content_revision(markdown_before.as_bytes()),
                },
                CoordinatedWrite {
                    path: "main.cmmn".into(),
                    content: "changed diagram".into(),
                    expected_revision: content_revision(&diagram_before),
                },
            ],
            Some(1),
        )
        .await;
        assert!(result.unwrap_err().to_string().contains("injected"));
        assert_eq!(fs::read(diagram_path).unwrap(), diagram_before);
        assert_eq!(fs::read(markdown_path).unwrap(), markdown_physical_before);
    }

    #[tokio::test]
    async fn embedding_header_publication_is_compare_and_swap_and_hidden_from_ui() {
        let (directory, layout) = initialized();
        let state = AppState::new(directory.path()).unwrap();
        let path = layout.schematics.join("main.md");
        let old_body = "# Old body\n";
        let newer_body = "# Newer body\n";
        atomic_write(&path, newer_body.as_bytes()).await.unwrap();
        let envelope = schematic_graph::derive_embedding_envelope(
            directory.path(),
            "main.cmmn#diagram",
            old_body,
            &schematic_graph::LoadOptions::deterministic_test(),
        )
        .unwrap();
        state
            .embedding_jobs
            .lock()
            .await
            .insert("main.md".into(), 1);
        assert!(
            !publish_embedding_header(&state, "main.md", 1, old_body, &envelope)
                .await
                .unwrap()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), newer_body);

        let envelope = schematic_graph::derive_embedding_envelope(
            directory.path(),
            "main.cmmn#diagram",
            newer_body,
            &schematic_graph::LoadOptions::deterministic_test(),
        )
        .unwrap();
        state
            .embedding_jobs
            .lock()
            .await
            .insert("main.md".into(), 2);
        assert!(
            publish_embedding_header(&state, "main.md", 2, newer_body, &envelope)
                .await
                .unwrap()
        );
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .starts_with(embedding_document::HEADER_START)
        );
        let (status, bytes) = send(app(state), "GET", "/api/file?path=main.md", None).await;
        assert_eq!(status, StatusCode::OK);
        let response: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(response["content"], newer_body);
    }

    #[tokio::test]
    async fn markdown_save_publishes_text_then_hybrid_without_false_graph_warning() {
        let (directory, layout) = initialized();
        let server = schematic_mcp::SchematicMcp::load(
            directory.path(),
            schematic_graph::LoadOptions::deterministic_test(),
        )
        .unwrap();
        let state = AppState::new(directory.path())
            .unwrap()
            .with_graph(server.clone());
        let path = layout.schematics.join("main.md");
        let body = "# Two-stage publication\nThe authored body remains stable.\n";
        atomic_write(&path, body.as_bytes()).await.unwrap();

        state
            .notify_document_changed("main.md", schematic_mcp::DocumentChangeKind::Replaced)
            .await;
        let text_ready = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let status = server.refresh_status_public().await;
                if !matches!(
                    status.status,
                    schematic_mcp::GraphRefreshStatus::Queued
                        | schematic_mcp::GraphRefreshStatus::Processing
                ) {
                    break status;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(text_ready.retrieval_mode.as_deref(), Some("text"));
        assert_eq!(
            text_ready.vector_readiness,
            Some(schematic_graph::VectorReadiness::Pending)
        );
        assert!(!text_ready.diagnostic.as_deref().is_some_and(|diagnostic| {
            diagnostic.contains("embedding header")
                || diagnostic.contains("vector retrieval is pending")
        }));

        let generation = 42;
        let envelope = schematic_graph::derive_embedding_envelope(
            directory.path(),
            "main.cmmn#diagram",
            body,
            &schematic_graph::LoadOptions::deterministic_test(),
        )
        .unwrap();
        state
            .embedding_jobs
            .lock()
            .await
            .insert("main.md".into(), generation);
        assert!(
            publish_embedding_header(&state, "main.md", generation, body, &envelope)
                .await
                .unwrap()
        );
        let hybrid_ready = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let status = server.refresh_status_public().await;
                if !matches!(
                    status.status,
                    schematic_mcp::GraphRefreshStatus::Queued
                        | schematic_mcp::GraphRefreshStatus::Processing
                ) {
                    break status;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(hybrid_ready.retrieval_mode.as_deref(), Some("hybrid"));
        assert_eq!(
            hybrid_ready.vector_readiness,
            Some(schematic_graph::VectorReadiness::Current)
        );
        let physical = fs::read_to_string(path).unwrap();
        assert_eq!(embedding_document::parse_markdown(&physical).body, body);
    }

    #[tokio::test]
    async fn successful_web_save_refreshes_running_mcp_revision() {
        let (directory, _) = initialized();
        let server = schematic_mcp::SchematicMcp::load(
            directory.path(),
            schematic_graph::LoadOptions::deterministic_test(),
        )
        .unwrap();
        let initial = server.summary().await.revision;
        let router = app(AppState::new(directory.path())
            .unwrap()
            .with_graph(server.clone()));
        let (status, bytes) = send(
            router,
            "PUT",
            "/api/file",
            Some(serde_json::json!({
                "path": "main.md",
                "content": "# Evaluated contract\nA required parameter was added by the user.",
                "revision": 1,
                "expectedRevision": content_revision(fs::read_to_string(directory.path().join("schematics/main.md")).unwrap().as_bytes())
            })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let response: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(response["graphRefresh"]["status"], "queued");
        let active = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let status = server.refresh_status_public().await;
                if !matches!(
                    status.status,
                    schematic_mcp::GraphRefreshStatus::Queued
                        | schematic_mcp::GraphRefreshStatus::Processing
                ) {
                    break status.active_revision.unwrap();
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_ne!(active, initial);
        assert_eq!(server.summary().await.revision, active);
        assert_eq!(
            fs::read_to_string(directory.path().join("schematics/main.md")).unwrap(),
            "# Evaluated contract\nA required parameter was added by the user."
        );
    }

    #[tokio::test]
    async fn project_metadata_returns_only_the_unicode_directory_basename() {
        let parent = tempdir().unwrap();
        let project = parent.path().join("Café Platform");
        init_project(&project).unwrap();
        fs::write(
            project.join(".ss/assistant.json"),
            br#"{"provider":"fake"}"#,
        )
        .unwrap();
        let router = app(AppState::new(&project).unwrap());
        let (status, bytes) = send(router, "GET", "/api/project", None).await;
        assert_eq!(status, StatusCode::OK);
        let metadata: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            metadata,
            serde_json::json!({ "name": "Café Platform", "assistant_provider": "fake" })
        );
        assert!(
            !String::from_utf8(bytes)
                .unwrap()
                .contains(&parent.path().to_string_lossy().to_string())
        );
    }

    #[tokio::test]
    async fn documentation_rename_refuses_collisions() {
        let (directory, _) = initialized();
        let docs = directory.path().join("schematics/docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("Old.md"), "old").unwrap();
        fs::write(docs.join("Taken.md"), "taken").unwrap();
        let router = app(AppState::new(directory.path()).unwrap());
        let (status, _) = send(
            router,
            "POST",
            "/api/rename-documentation",
            Some(serde_json::json!({
                "diagram_path": "main.cmmn", "old_id": "Old", "new_id": "Taken"
            })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(fs::read_to_string(docs.join("Old.md")).unwrap(), "old");
        assert_eq!(fs::read_to_string(docs.join("Taken.md")).unwrap(), "taken");
    }

    #[tokio::test]
    async fn documentation_rename_follows_id_and_not_architectural_name() {
        let (directory, _) = initialized();
        let docs = directory.path().join("schematics/docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("Activity_42.md"), "call-site documentation").unwrap();
        let router = app(AppState::new(directory.path()).unwrap());
        let (status, _) = send(
            router,
            "POST",
            "/api/rename-documentation",
            Some(serde_json::json!({
                "diagram_path": "main.cmmn", "old_id": "Activity_42", "new_id": "Activity_99"
            })),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(!docs.join("Activity_42.md").exists());
        assert_eq!(
            fs::read_to_string(docs.join("Activity_99.md")).unwrap(),
            "call-site documentation"
        );
        assert!(!docs.join("validateOrder.md").exists());
    }

    #[tokio::test]
    async fn composition_creation_is_idempotent_and_reusable() {
        let (directory, _) = initialized();
        let router = app(AppState::new(directory.path()).unwrap());
        let payload = serde_json::json!({ "qualified_name": "sales.Order" });
        let (first_status, first) = send(
            router.clone(),
            "POST",
            "/api/compositions",
            Some(payload.clone()),
        )
        .await;
        let (second_status, second) =
            send(router, "POST", "/api/compositions", Some(payload)).await;
        assert_eq!(first_status, StatusCode::OK);
        assert_eq!(second_status, StatusCode::OK);
        assert!(
            String::from_utf8(first)
                .unwrap()
                .contains("\"created\":true")
        );
        assert!(
            String::from_utf8(second)
                .unwrap()
                .contains("\"created\":false")
        );
        assert!(
            directory
                .path()
                .join("schematics/sales/Order/main.bpmn")
                .is_file()
        );
        assert!(
            directory
                .path()
                .join("schematics/sales/Order/main.md")
                .is_file()
        );
        let diagram =
            fs::read_to_string(directory.path().join("schematics/sales/Order/main.bpmn")).unwrap();
        assert!(!diagram.contains("<bpmn:startEvent"));
        assert!(!diagram.contains("<bpmn:task"));
        assert!(!diagram.contains("<bpmn:sequenceFlow"));
        assert!(!diagram.contains("<bpmndi:BPMNShape"));
    }

    #[tokio::test]
    async fn created_composition_revert_requires_its_current_tree_revision() {
        let (directory, _) = initialized();
        let router = app(AppState::new(directory.path()).unwrap());
        let qualified_name = "sales.Rollback";
        let (status, _) = send(
            router.clone(),
            "POST",
            "/api/compositions",
            Some(serde_json::json!({ "qualified_name": qualified_name })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let revision_request = serde_json::json!({ "qualified_name": qualified_name });
        let (status, bytes) = send(
            router.clone(),
            "POST",
            "/api/composition-revisions",
            Some(revision_request.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let initial: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let folder = directory.path().join("schematics/sales/Rollback");
        fs::write(folder.join("main.md"), "newer user documentation").unwrap();

        let (status, bytes) = send(
            router.clone(),
            "POST",
            "/api/composition-reverts",
            Some(serde_json::json!({
                "qualified_name": qualified_name,
                "expected_revision": initial["revision"]
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        let conflict: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(conflict["code"], "revisionConflict");
        assert!(folder.is_dir());
        assert_eq!(
            fs::read_to_string(folder.join("main.md")).unwrap(),
            "newer user documentation"
        );

        let (status, bytes) = send(
            router.clone(),
            "POST",
            "/api/composition-revisions",
            Some(revision_request),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let current: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let (status, _) = send(
            router,
            "POST",
            "/api/composition-reverts",
            Some(serde_json::json!({
                "qualified_name": qualified_name,
                "expected_revision": current["revision"]
            })),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(!folder.exists());
    }

    #[tokio::test]
    async fn cmmn_business_anchor_coexists_with_descendant_bpmn_and_renames_confined_package() {
        let (directory, _) = initialized();
        let router = app(AppState::new(directory.path()).unwrap());
        let anchor = serde_json::json!({ "kind": "cmmn", "package_name": "cybling" });
        let (status, bytes) = send(router.clone(), "POST", "/api/compositions", Some(anchor)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            String::from_utf8(bytes)
                .unwrap()
                .contains("cybling/main.cmmn")
        );
        let design = serde_json::json!({ "kind": "bpmn", "qualified_name": "cybling.sdk.Birth" });
        let (status, _) = send(router.clone(), "POST", "/api/compositions", Some(design)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            directory
                .path()
                .join("schematics/cybling/main.cmmn")
                .is_file()
        );
        assert!(
            directory
                .path()
                .join("schematics/cybling/sdk/Birth/main.bpmn")
                .is_file()
        );
        assert!(
            !directory
                .path()
                .join("schematics/cybling/sdk/main.cmmn")
                .exists()
        );
        let cmmn =
            fs::read_to_string(directory.path().join("schematics/cybling/main.cmmn")).unwrap();
        assert!(cmmn.contains("ssw:packageName=\"cybling\""));
        let (status, bytes) = send(router.clone(), "GET", "/api/diagrams", None).await;
        assert_eq!(status, StatusCode::OK);
        let listed = String::from_utf8(bytes).unwrap();
        assert!(listed.contains("cybling/main.cmmn"));
        assert!(listed.contains("cybling/sdk/Birth/main.bpmn"));
        let rename =
            serde_json::json!({ "old_package_name": "cybling", "new_package_name": "platform" });
        let (status, _) = send(router, "POST", "/api/package-renames", Some(rename)).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(
            directory
                .path()
                .join("schematics/platform/main.cmmn")
                .is_file()
        );
        assert!(
            directory
                .path()
                .join("schematics/platform/sdk/Birth/main.bpmn")
                .is_file()
        );
        assert!(!directory.path().join("schematics/cybling").exists());
    }

    #[tokio::test]
    async fn process_rename_moves_folder_and_updates_shared_references() {
        let (directory, _) = initialized();
        let state = AppState::new(directory.path()).unwrap();
        let router = app(state);
        let create =
            serde_json::json!({ "qualified_name": "cybling.subscription.SelectAndOutfit" });
        let (status, _) = send(router.clone(), "POST", "/api/compositions", Some(create)).await;
        assert_eq!(status, StatusCode::OK);
        let old = directory
            .path()
            .join("schematics/cybling/subscription/SelectAndOutfit");
        fs::create_dir_all(old.join("assets")).unwrap();
        fs::write(old.join("assets/model.png"), b"portable").unwrap();
        fs::write(old.join("main.md"), "![model](./assets/model.png)").unwrap();
        let root = directory.path().join("schematics/main.cmmn");
        fs::write(&root, fs::read_to_string(&root).unwrap().replace("<cmmn:case", "<cmmn:process id=\"Process_Reference\" externalRef=\"cybling.subscription.SelectAndOutfit\" /><cmmn:case")).unwrap();
        let rename = serde_json::json!({ "old_qualified_name": "cybling.subscription.SelectAndOutfit", "new_qualified_name": "cybling.subscription.ConfigureSubscription" });
        let (rename_status, _) = send(router, "POST", "/api/process-renames", Some(rename)).await;
        assert_eq!(rename_status, StatusCode::NO_CONTENT);
        let new = directory
            .path()
            .join("schematics/cybling/subscription/ConfigureSubscription");
        assert!(!old.exists());
        assert_eq!(fs::read(new.join("assets/model.png")).unwrap(), b"portable");
        assert_eq!(
            fs::read_to_string(new.join("main.md")).unwrap(),
            "![model](./assets/model.png)"
        );
        assert!(
            fs::read_to_string(root)
                .unwrap()
                .contains("externalRef=\"cybling.subscription.ConfigureSubscription\"")
        );
        assert!(
            fs::read_to_string(new.join("main.bpmn"))
                .unwrap()
                .contains("ssw:processName=\"cybling.subscription.ConfigureSubscription\"")
        );
    }

    #[tokio::test]
    async fn process_rename_rejects_existing_name() {
        let (directory, _) = initialized();
        let router = app(AppState::new(directory.path()).unwrap());
        for name in ["sales.Order", "sales.Invoice"] {
            let (status, _) = send(
                router.clone(),
                "POST",
                "/api/compositions",
                Some(serde_json::json!({ "qualified_name": name })),
            )
            .await;
            assert_eq!(status, StatusCode::OK);
        }
        let (collision, _) = send(router, "POST", "/api/process-renames", Some(serde_json::json!({ "old_qualified_name": "sales.Order", "new_qualified_name": "sales.Invoice" }))).await;
        assert_eq!(collision, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn process_rename_rollback_rejects_a_stale_schematic_revision() {
        let (directory, _) = initialized();
        let router = app(AppState::new(directory.path()).unwrap());
        let (status, _) = send(
            router.clone(),
            "POST",
            "/api/compositions",
            Some(serde_json::json!({ "qualified_name": "sales.Order" })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let (status, bytes) = send(router.clone(), "GET", "/api/schematic-revision", None).await;
        assert_eq!(status, StatusCode::OK);
        let revision: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        fs::write(
            directory.path().join("schematics/main.md"),
            "newer user documentation",
        )
        .unwrap();

        let (status, bytes) = send(
            router,
            "POST",
            "/api/process-renames",
            Some(serde_json::json!({
                "old_qualified_name": "sales.Order",
                "new_qualified_name": "sales.RenamedOrder",
                "expected_revision": revision["revision"]
            })),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        let conflict: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(conflict["code"], "revisionConflict");
        assert!(
            directory
                .path()
                .join("schematics/sales/Order/main.bpmn")
                .is_file()
        );
        assert!(
            !directory
                .path()
                .join("schematics/sales/RenamedOrder")
                .exists()
        );
    }

    #[tokio::test]
    async fn assistant_endpoint_returns_a_correlated_non_mutating_fake_proposal() {
        let (directory, _) = initialized();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::write(
            directory.path().join(".ss/assistant.json"),
            br#"{"provider":"fake"}"#,
        )
        .unwrap();
        let before = fs::read_to_string(directory.path().join("schematics/main.cmmn")).unwrap();
        let router = app(AppState::new(directory.path()).unwrap());
        let (status, bytes) = send(router, "POST", "/api/assistant/proposals", Some(serde_json::json!({
            "requestId": "request-1", "prompt": "Improve this task", "snapshot": {
                "version": "2.0", "diagramPath": "main.cmmn", "sourceRevision": "revision-1", "primaryNodeId": "Task_1",
                "graph": {"nodes": [{"id": "Task_1", "name": "work", "label": "Work", "status": "open"}], "flows": []}
            },
            "turns": [
                {"role":"user", "text":"Explain the current task."},
                {"role":"assistant", "text":"It represents the persisted work step."},
                {"role":"user", "text":"Suggest a clearer label."}
            ]
        }))).await;
        assert_eq!(status, StatusCode::OK);
        let result: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["proposal"]["requestId"], "request-1");
        assert_eq!(result["provider"], "fake");
        assert_eq!(
            fs::read_to_string(directory.path().join("schematics/main.cmmn")).unwrap(),
            before
        );
    }

    #[tokio::test]
    async fn assistant_conversation_endpoint_returns_prose_without_a_proposal() {
        let (directory, _) = initialized();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::write(
            directory.path().join(".ss/assistant.json"),
            br#"{"provider":"fake"}"#,
        )
        .unwrap();
        let before = fs::read_to_string(directory.path().join("schematics/main.cmmn")).unwrap();
        let router = app(AppState::new(directory.path()).unwrap());
        let (status, bytes) = send(router.clone(), "POST", "/api/assistant/conversations", Some(serde_json::json!({
            "requestId": "conversation-1", "snapshot": {
                "version": "2.0", "scope": "node", "diagramPath": "main.cmmn", "sourceRevision": "revision-1", "primaryElementId": "Task_1", "primaryNodeId": "Task_1",
                "graph": {"nodes": [{"id": "Task_1", "type": "cmmn:HumanTask", "name": "work", "label": "Work", "status": "open"}], "flows": []}
            },
            "turns": [{"role":"user", "text":"What should this task do?"}]
        }))).await;
        assert_eq!(status, StatusCode::OK);
        let result: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["provider"], "fake");
        assert!(result["reply"].as_str().unwrap().contains("Task_1"));
        assert!(result.get("proposal").is_none());
        assert_eq!(
            fs::read_to_string(directory.path().join("schematics/main.cmmn")).unwrap(),
            before
        );

        let (status, _) = send(router, "POST", "/api/assistant/conversations", Some(serde_json::json!({
            "requestId": "conversation-2", "snapshot": {"version":"2.0","diagramPath":"main.cmmn"},
            "turns": [{"role":"assistant", "text":"Not a user turn"}]
        }))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn assistant_conversation_endpoint_rejects_requests_when_slots_are_full() {
        let (directory, _) = initialized();
        fs::create_dir_all(directory.path().join(".ss")).unwrap();
        fs::write(
            directory.path().join(".ss/assistant.json"),
            br#"{"provider":"fake"}"#,
        )
        .unwrap();
        let state = AppState::new(directory.path()).unwrap();
        let _first = state.assistant_slots.clone().try_acquire_owned().unwrap();
        let _second = state.assistant_slots.clone().try_acquire_owned().unwrap();
        let router = app(state);
        let (status, bytes) = send(router, "POST", "/api/assistant/conversations", Some(serde_json::json!({
            "requestId": "conversation-full", "snapshot": {"version":"2.0","diagramPath":"main.cmmn"},
            "turns": [{"role":"user", "text":"Continue"}]
        }))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            error["error"]
                .as_str()
                .unwrap()
                .contains("request limit reached")
        );
    }

    #[tokio::test]
    async fn chat_proposals_enter_the_same_validated_browser_inbox() {
        let (directory, _) = initialized();
        let router = app(AppState::new(directory.path()).unwrap());
        let submission = serde_json::json!({
            "request": {
                "requestId": "chat-request-1",
                "prompt": "Rename the task",
                "snapshot": {
                    "version": "2.0",
                    "diagramKind": "cmmn",
                    "diagramPath": "main.cmmn",
                    "sourceRevision": "snapshot-revision",
                    "graph": {
                        "nodes": [{
                            "id": "Task_1", "type": "cmmn:HumanTask",
                            "name": "sales#work", "label": "Work", "status": "open"
                        }],
                        "flows": []
                    }
                }
            },
            "proposal": {
                "version": "2.0",
                "requestId": "chat-request-1",
                "sourceRevision": "snapshot-revision",
                "summary": "Use a concise task label",
                "assumptions": [],
                "warnings": [],
                "operations": [{
                    "type": "update_node_label", "diagramPath": "main.cmmn",
                    "nodeId": "Task_1", "label": "Save data"
                }]
            }
        });
        let (status, _) = send(
            router.clone(),
            "POST",
            "/api/assistant/inbox",
            Some(submission),
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        let (status, bytes) = send(router, "GET", "/api/assistant/inbox", None).await;
        assert_eq!(status, StatusCode::OK);
        let inbox: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(inbox[0]["request"]["requestId"], "chat-request-1");
        assert_eq!(inbox[0]["proposal"]["operations"][0]["label"], "Save data");
    }
}
