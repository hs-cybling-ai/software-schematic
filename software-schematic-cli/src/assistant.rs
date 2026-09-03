use crate::{
    Error, Result, validate_element_name, validate_member_name, validate_package_name,
    validate_qualified_process_name,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    env,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

pub const SCHEMA_VERSION: &str = "2.0";
pub const MAX_OPERATIONS: usize = 64;
pub const MAX_RESPONSE_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantRequest {
    pub request_id: String,
    pub prompt: String,
    pub snapshot: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantPlan {
    pub version: String,
    pub request_id: String,
    pub source_revision: String,
    pub summary: String,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub operations: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantResult {
    pub proposal: AssistantPlan,
    pub provider: String,
    pub model: String,
    pub usage: Value,
}

#[allow(async_fn_in_trait)]
pub trait AssistantProvider {
    async fn propose(&self, request: &AssistantRequest) -> Result<AssistantResult>;
}

pub struct FakeProvider {
    model: String,
}

impl FakeProvider {
    pub fn new(model: String) -> Self {
        Self { model }
    }
}

impl AssistantProvider for FakeProvider {
    async fn propose(&self, request: &AssistantRequest) -> Result<AssistantResult> {
        if request.prompt.contains("[timeout]") {
            tokio::time::sleep(Duration::from_secs(35)).await;
        }
        if request.prompt.contains("[auth]") {
            return Err(Error::Message(
                "assistant provider authentication failed; check host configuration".into(),
            ));
        }
        if request.prompt.contains("[invalid]") {
            return Err(Error::Message(
                "assistant provider returned invalid structured output".into(),
            ));
        }
        let primary = request
            .snapshot
            .pointer("/primaryNodeId")
            .and_then(Value::as_str);
        let diagram = request
            .snapshot
            .pointer("/diagramPath")
            .and_then(Value::as_str)
            .unwrap_or("main.bpmn");
        let operations = if let Some(node_id) =
            primary.filter(|_| request.prompt.to_ascii_lowercase().contains("subprocess"))
        {
            let child = "assistant.AssistantSubprocess";
            let child_diagram = "assistant/AssistantSubprocess/main.bpmn";
            vec![
                json!({"type":"replace_node_type","diagramPath":diagram,"nodeId":node_id,"bpmnType":"bpmn:CallActivity"}),
                json!({"type":"set_process_reference","diagramPath":diagram,"nodeId":node_id,"qualifiedName":child}),
                json!({"type":"create_process","qualifiedName":child}),
                json!({"type":"add_flow_node","diagramPath":child_diagram,"nodeId":"AssistantStep_1","bpmnType":"bpmn:Task","name":"assistant.AssistantSubprocess#firstStep","label":"First step","x":180,"y":330}),
                json!({"type":"add_flow_node","diagramPath":child_diagram,"nodeId":"AssistantStep_2","bpmnType":"bpmn:Task","name":"assistant.AssistantSubprocess#secondStep","label":"Second step","x":350,"y":330}),
                json!({"type":"add_flow_node","diagramPath":child_diagram,"nodeId":"AssistantStep_3","bpmnType":"bpmn:Task","name":"assistant.AssistantSubprocess#thirdStep","label":"Third step","x":520,"y":330}),
                json!({"type":"add_flow_node","diagramPath":child_diagram,"nodeId":"AssistantStep_4","bpmnType":"bpmn:Task","name":"assistant.AssistantSubprocess#fourthStep","label":"Fourth step","x":690,"y":330}),
                json!({"type":"connect_sequence_flow","diagramPath":child_diagram,"flowId":"AssistantFlow_1","sourceId":"AssistantStep_1","targetId":"AssistantStep_2"}),
                json!({"type":"connect_sequence_flow","diagramPath":child_diagram,"flowId":"AssistantFlow_2","sourceId":"AssistantStep_2","targetId":"AssistantStep_3"}),
                json!({"type":"connect_sequence_flow","diagramPath":child_diagram,"flowId":"AssistantFlow_3","sourceId":"AssistantStep_3","targetId":"AssistantStep_4"}),
                json!({"type":"replace_node_markdown","diagramPath":diagram,"nodeId":node_id,"markdown":"# Assistant subprocess\n\nThis activity delegates to a documented four-step composition."}),
            ]
        } else {
            primary.map(|node_id| vec![json!({
            "type": "update_node_label", "diagramPath": diagram, "nodeId": node_id,
            "label": format!("{} (assistant suggestion)", request.snapshot.pointer("/graph/nodes").and_then(Value::as_array).and_then(|nodes| nodes.iter().find(|node| node["id"] == node_id)).and_then(|node| node["label"].as_str()).unwrap_or(node_id))
        })]).unwrap_or_default()
        };
        let plan = AssistantPlan {
            version: SCHEMA_VERSION.into(),
            request_id: request.request_id.clone(),
            source_revision: request.snapshot["sourceRevision"]
                .as_str()
                .unwrap_or_default()
                .into(),
            summary: if primary.is_some() {
                "Update the selected node"
            } else {
                "Review the complete diagram"
            }
            .into(),
            assumptions: vec![
                "This deterministic proposal was generated by the local fake provider.".into(),
            ],
            warnings: vec![],
            operations,
        };
        Ok(AssistantResult {
            proposal: plan,
            provider: "fake".into(),
            model: self.model.clone(),
            usage: json!({"inputTokens": 0, "outputTokens": 0}),
        })
    }
}

pub struct OpenAiProvider {
    model: String,
    endpoint: String,
    key: String,
    client: reqwest::Client,
}

pub struct LocalCliProvider {
    kind: String,
    project: PathBuf,
}

impl LocalCliProvider {
    pub fn new(kind: &str, project: PathBuf) -> Self {
        Self {
            kind: kind.into(),
            project,
        }
    }

    fn run(&self, request: &AssistantRequest) -> Result<AssistantResult> {
        let schema = operation_plan_schema();
        let prompt = format!(
            "Return only a Software Schematic operation plan matching the supplied JSON schema. Do not inspect files, run tools, or mutate anything. Preserve requestId={} and sourceRevision={}. Preserve every existing ID, Type, Label, Name, Implementation Status, and Documentation unless an explicit operation changes it. Never emit a composition folder path: use a complete Name. BPMN Process Names use package.Process and BPMN members use package.Process#member. CMMN business-need members use package#member, while a CMMN ProcessTask link uses package.Process. A process rename must use rename_process. For diagramPath, use the path already present in context. Interpret system task as bpmn:ServiceTask. Emit each intended operation only once. User request: {}\nContext: {}",
            request.request_id,
            request.snapshot["sourceRevision"]
                .as_str()
                .unwrap_or_default(),
            request.prompt,
            request.snapshot
        );
        let output = if self.kind == "codex" {
            let schema_path = self.project.join(".ss/operation-plan.schema.json");
            let mut child = Command::new("codex")
                .args([
                    "exec",
                    "--ephemeral",
                    "--sandbox",
                    "read-only",
                    "--skip-git-repo-check",
                    "--ignore-user-config",
                    "--color",
                    "never",
                    "--output-schema",
                ])
                .arg(schema_path)
                .args([
                    "-c",
                    "features.shell_tool=false",
                    "-c",
                    "features.web_search=false",
                    "-",
                ])
                .current_dir(self.project.join(".ss"))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|_| {
                    Error::Message("Codex CLI is unavailable; run ./ssw auth login".into())
                })?;
            child.stdin.take().unwrap().write_all(prompt.as_bytes())?;
            child.wait_with_output()?
        } else {
            Command::new("claude")
                .args([
                    "-p",
                    "--output-format",
                    "json",
                    "--no-session-persistence",
                    "--permission-mode",
                    "plan",
                    "--tools",
                    "",
                    "--json-schema",
                ])
                .arg(schema.to_string())
                .arg(prompt)
                .current_dir(self.project.join(".ss"))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .map_err(|_| {
                    Error::Message(
                        "Claude Code is unavailable; run ./ssw auth login --provider claude".into(),
                    )
                })?
        };
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("invalid_json_schema") {
                return Err(Error::Message(
                    "Codex rejected the Software Schematic operation schema; update SSW to a compatible release"
                        .into(),
                ));
            }
            if stderr.contains("401")
                || stderr.to_ascii_lowercase().contains("not logged in")
                || stderr.to_ascii_lowercase().contains("authentication")
            {
                return Err(Error::Message(format!(
                    "{} is not authenticated; run ./ssw auth login --provider {}",
                    self.kind, self.kind
                )));
            }
            return Err(Error::Message(format!(
                "{} could not generate a proposal (exit status {}); run ./ssw auth status and retry",
                self.kind,
                output
                    .status
                    .code()
                    .map_or_else(|| "unknown".into(), |code| code.to_string())
            )));
        }
        if output.stdout.len() > MAX_RESPONSE_BYTES {
            return Err(Error::Message(
                "local assistant response exceeds the configured limit".into(),
            ));
        }
        let mut value: Value = serde_json::from_slice(&output.stdout).map_err(|_| {
            Error::Message(format!(
                "{} returned malformed structured output",
                self.kind
            ))
        })?;
        if self.kind == "claude" {
            value = value
                .get("structured_output")
                .cloned()
                .or_else(|| {
                    value
                        .get("result")
                        .and_then(Value::as_str)
                        .and_then(|text| serde_json::from_str(text).ok())
                })
                .ok_or_else(|| Error::Message("Claude returned no structured proposal".into()))?;
        }
        let plan: AssistantPlan = serde_json::from_value(value).map_err(|_| {
            Error::Message(format!("{} returned an invalid operation plan", self.kind))
        })?;
        Ok(AssistantResult {
            proposal: plan,
            provider: self.kind.clone(),
            model: "account-default".into(),
            usage: json!({}),
        })
    }
}

impl AssistantProvider for LocalCliProvider {
    async fn propose(&self, request: &AssistantRequest) -> Result<AssistantResult> {
        self.run(request)
    }
}

impl OpenAiProvider {
    pub fn new(model: String, endpoint: String, key: String) -> Result<Self> {
        if endpoint != "https://api.openai.com/v1/responses" {
            return Err(Error::Message(
                "assistant endpoint is not on the HTTPS provider allowlist".into(),
            ));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|_| Error::Message("could not configure assistant HTTPS client".into()))?;
        Ok(Self {
            model,
            endpoint,
            key,
            client,
        })
    }
}

impl AssistantProvider for OpenAiProvider {
    async fn propose(&self, request: &AssistantRequest) -> Result<AssistantResult> {
        let schema = operation_plan_schema();
        let response = self.client.post(&self.endpoint).bearer_auth(&self.key).json(&json!({
            "model": self.model,
            "instructions": "You propose safe Software Schematic changes. Return only the required structured operation plan. Never emit code, shell commands, patches, or raw BPMN XML.",
            "input": [{"role":"user","content":[{"type":"input_text","text": format!("Request: {}\nContext: {}", request.prompt, request.snapshot)}]}],
            "text": {"format": {"type":"json_schema","name":"software_schematic_operation_plan","strict":true,"schema":schema}}
        })).send().await.map_err(|_| Error::Message("assistant provider request failed or timed out".into()))?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::Message("assistant provider authentication failed; check OPENAI_API_KEY in the host environment".into()));
        }
        if !response.status().is_success() {
            return Err(Error::Message(format!(
                "assistant provider returned HTTP {}",
                response.status().as_u16()
            )));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|_| Error::Message("assistant provider response could not be read".into()))?;
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(Error::Message(
                "assistant provider response exceeds the configured limit".into(),
            ));
        }
        let envelope: Value = serde_json::from_slice(&bytes)
            .map_err(|_| Error::Message("assistant provider returned malformed JSON".into()))?;
        let text = envelope
            .pointer("/output/0/content/0/text")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                Error::Message("assistant provider returned no structured proposal".into())
            })?;
        let mut plan: AssistantPlan = serde_json::from_str(text).map_err(|_| {
            Error::Message("assistant provider returned invalid structured output".into())
        })?;
        plan.request_id = request.request_id.clone();
        plan.source_revision = request.snapshot["sourceRevision"]
            .as_str()
            .unwrap_or_default()
            .into();
        Ok(AssistantResult {
            proposal: plan,
            provider: "openai".into(),
            model: self.model.clone(),
            usage: envelope.get("usage").cloned().unwrap_or_else(|| json!({})),
        })
    }
}

pub async fn generate(
    request: &AssistantRequest,
    project: PathBuf,
    configured_provider: Option<String>,
) -> Result<AssistantResult> {
    validate_request(request)?;
    let provider = env::var("SSW_ASSISTANT_PROVIDER")
        .ok()
        .or(configured_provider)
        .ok_or_else(|| Error::Message("assistant is not configured; run ./ssw auth login in this project, then restart SSW".into()))?;
    let model = env::var("SSW_ASSISTANT_MODEL").unwrap_or_else(|_| {
        if provider == "openai" {
            "gpt-5.4".into()
        } else {
            "deterministic-v1".into()
        }
    });
    let mut result = match provider.as_str() {
        "fake" => FakeProvider::new(model).propose(request).await?,
        "openai" => {
            let key = env::var("OPENAI_API_KEY").map_err(|_| Error::Message("OpenAI assistance is not configured; set OPENAI_API_KEY in the host environment".into()))?;
            let endpoint = env::var("SSW_ASSISTANT_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/responses".into());
            OpenAiProvider::new(model, endpoint, key)?
                .propose(request)
                .await?
        }
        "codex" | "claude" => {
            LocalCliProvider::new(&provider, project)
                .propose(request)
                .await?
        }
        _ => {
            return Err(Error::Message(
                "assistant provider is not allowlisted; use codex, claude, fake, or openai".into(),
            ));
        }
    };
    canonicalize_plan_paths(&mut result.proposal)?;
    validate_plan(&result.proposal, request)?;
    Ok(result)
}

fn canonicalize_plan_paths(plan: &mut AssistantPlan) -> Result<()> {
    for operation in &mut plan.operations {
        for field in ["path", "diagramPath"] {
            let Some(raw) = operation[field].as_str() else {
                continue;
            };
            let trimmed = raw.trim();
            let normalized = trimmed
                .strip_prefix("/schematics/")
                .or_else(|| trimmed.strip_prefix("schematics/"))
                .unwrap_or(trimmed);
            confined_path(normalized)?;
            operation[field] = json!(normalized);
        }
    }
    Ok(())
}

pub fn validate_request(request: &AssistantRequest) -> Result<()> {
    if request.request_id.is_empty() || request.request_id.len() > 128 {
        return Err(Error::Message("assistant request ID is invalid".into()));
    }
    if request.prompt.trim().is_empty() || request.prompt.len() > 16_000 {
        return Err(Error::Message(
            "assistant prompt is empty or exceeds the configured limit".into(),
        ));
    }
    if request.snapshot["version"] != SCHEMA_VERSION {
        return Err(Error::Message(
            "unsupported assistant context version".into(),
        ));
    }
    confined_path(request.snapshot["diagramPath"].as_str().unwrap_or_default())?;
    Ok(())
}

pub fn validate_plan(plan: &AssistantPlan, request: &AssistantRequest) -> Result<()> {
    if plan.version != SCHEMA_VERSION
        || plan.request_id != request.request_id
        || plan.source_revision
            != request.snapshot["sourceRevision"]
                .as_str()
                .unwrap_or_default()
    {
        return Err(Error::Message(
            "assistant proposal correlation or version is invalid".into(),
        ));
    }
    if plan.operations.len() > MAX_OPERATIONS {
        return Err(Error::Message(
            "assistant proposal exceeds the operation limit".into(),
        ));
    }
    let allowed: HashSet<&str> = [
        "replace_node_type",
        "update_node_label",
        "update_node_name",
        "set_node_status",
        "set_process_reference",
        "create_process",
        "open_process",
        "rename_process",
        "add_flow_node",
        "connect_sequence_flow",
        "add_plan_item",
        "connect_cmmn",
        "replace_diagram_markdown",
        "replace_node_markdown",
    ]
    .into_iter()
    .collect();
    let nodes = request
        .snapshot
        .pointer("/graph/nodes")
        .and_then(Value::as_array);
    let flows = request
        .snapshot
        .pointer("/graph/flows")
        .and_then(Value::as_array);
    let elements: Vec<&Value> = nodes
        .into_iter()
        .flatten()
        .chain(flows.into_iter().flatten())
        .collect();
    let locked: HashSet<&str> = elements
        .iter()
        .filter(|element| element["status"] == "locked")
        .filter_map(|element| element["id"].as_str())
        .collect();
    let mut ids: HashSet<&str> = elements
        .iter()
        .filter_map(|element| element["id"].as_str())
        .collect();
    let mut node_types: HashMap<&str, &str> = elements
        .iter()
        .filter_map(|element| {
            Some((
                element["id"].as_str()?,
                element["type"].as_str().unwrap_or_default(),
            ))
        })
        .collect();
    let mut process_names = std::collections::HashMap::new();
    for operation in &plan.operations {
        let kind = operation["type"]
            .as_str()
            .ok_or_else(|| Error::Message("assistant operation has no type".into()))?;
        if !allowed.contains(kind) {
            return Err(Error::Message(format!(
                "unsupported assistant operation: {kind}"
            )));
        }
        if operation.get("xml").is_some() || operation.get("rawXml").is_some() {
            return Err(Error::Message(
                "assistant providers cannot supply raw diagram XML".into(),
            ));
        }
        if operation.get("path").is_some() {
            return Err(Error::Message(
                "assistant providers cannot supply composition paths; use a qualified process Name"
                    .into(),
            ));
        }
        for field in ["path", "diagramPath"] {
            if let Some(path) = operation[field].as_str() {
                confined_path(path)?;
            }
        }
        for field in ["qualifiedName", "oldQualifiedName", "newQualifiedName"] {
            if let Some(name) = operation[field].as_str() {
                validate_qualified_process_name(name)?;
                if field != "oldQualifiedName" {
                    let key = name.to_ascii_lowercase();
                    if process_names
                        .insert(key, name)
                        .is_some_and(|prior| prior != name)
                    {
                        return Err(Error::Message(format!(
                            "case-insensitive qualified Name collision involving {name}"
                        )));
                    }
                }
            }
        }
        if kind == "update_node_name" {
            let name = operation["name"].as_str().unwrap_or_default();
            if request.snapshot["diagramKind"] == "cmmn" {
                let node_id = operation["nodeId"].as_str().unwrap_or_default();
                if node_types.get(node_id) == Some(&"cmmn:ProcessTask") {
                    validate_qualified_process_name(name)?;
                } else {
                    validate_cmmn_member_name(name)?;
                }
            } else {
                validate_element_name(name)?;
            }
        }
        if kind == "set_node_status"
            && !matches!(
                operation["status"].as_str(),
                Some("open" | "new" | "modify" | "locked")
            )
        {
            return Err(Error::Message("unsupported node status".into()));
        }
        if kind == "add_flow_node" && operation["name"].is_string() {
            let name = operation["name"].as_str().unwrap();
            validate_element_name(name)?;
        }
        if let Some(id) = operation["nodeId"].as_str() {
            valid_id(id)?;
            if locked.contains(id) {
                return Err(Error::Message(format!(
                    "locked node cannot be changed: {id}"
                )));
            }
            if matches!(kind, "add_flow_node" | "add_plan_item") {
                if !ids.insert(id) {
                    return Err(Error::Message(format!("duplicate created ID: {id}")));
                }
            } else if !ids.contains(id) {
                return Err(Error::Message(format!("unknown node: {id}")));
            }
        }
        if kind == "connect_sequence_flow" {
            let id = operation["flowId"]
                .as_str()
                .ok_or_else(|| Error::Message("sequence flow has no ID".into()))?;
            valid_id(id)?;
            if !ids.insert(id) {
                return Err(Error::Message(format!("duplicate created ID: {id}")));
            }
        }
        if kind == "add_plan_item" {
            if request.snapshot["diagramKind"] != "cmmn" {
                return Err(Error::Message(
                    "CMMN plan items require a CMMN diagram".into(),
                ));
            }
            let cmmn_type = operation["cmmnType"].as_str().unwrap_or_default();
            if !matches!(
                cmmn_type,
                "cmmn:Task"
                    | "cmmn:HumanTask"
                    | "cmmn:ProcessTask"
                    | "cmmn:CaseTask"
                    | "cmmn:Stage"
                    | "cmmn:Milestone"
                    | "cmmn:EventListener"
            ) {
                return Err(Error::Message(format!(
                    "unsupported CMMN type: {cmmn_type}"
                )));
            }
            if let Some(name) = operation["name"].as_str() {
                if cmmn_type == "cmmn:ProcessTask" {
                    validate_qualified_process_name(name)?;
                } else {
                    validate_cmmn_member_name(name)?;
                }
            }
            node_types.insert(operation["nodeId"].as_str().unwrap_or_default(), cmmn_type);
        }
        if kind == "connect_cmmn" {
            if request.snapshot["diagramKind"] != "cmmn" {
                return Err(Error::Message(
                    "CMMN connections require a CMMN diagram".into(),
                ));
            }
            let id = operation["connectionId"]
                .as_str()
                .ok_or_else(|| Error::Message("CMMN connection has no ID".into()))?;
            valid_id(id)?;
            if !ids.insert(id) {
                return Err(Error::Message(format!("duplicate created ID: {id}")));
            }
            for field in ["sourceId", "targetId"] {
                let element_id = operation[field].as_str().unwrap_or_default();
                if !ids.contains(element_id) {
                    return Err(Error::Message(format!(
                        "CMMN connection references unknown node: {element_id}"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn validate_cmmn_member_name(name: &str) -> Result<()> {
    if let Some((package, member)) = name.split_once('#') {
        validate_package_name(package)?;
        validate_member_name(member)?;
        if member.contains('#') {
            return Err(Error::Message(
                "CMMN Name may contain only one # member separator".into(),
            ));
        }
        Ok(())
    } else {
        Err(Error::Message(
            "CMMN node or connection Names require #memberName".into(),
        ))
    }
}

fn valid_id(id: &str) -> Result<()> {
    if id.is_empty()
        || !id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(Error::Message(format!(
            "invalid assistant element ID: {id}"
        )));
    }
    Ok(())
}

fn confined_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(Error::Message(
            "assistant path must remain relative to schematics".into(),
        ));
    }
    Ok(())
}

pub fn operation_plan_schema() -> Value {
    fn operation(required: &[&str], properties: Value) -> Value {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": required,
            "properties": properties
        })
    }
    let diagram_path = json!({"type":"string","minLength":1,"pattern":"^[^/\\\\].*$"});
    let qualified_name = json!({"type":"string","pattern":"^[a-z][A-Za-z0-9]*(\\.[a-z][A-Za-z0-9]*)*\\.[A-Z][A-Za-z0-9]*$"});
    let node_id = json!({"type": "string"});
    json!({
        "type": "object", "additionalProperties": false,
        "required": ["version", "requestId", "sourceRevision", "summary", "assumptions", "warnings", "operations"],
        "properties": {
            "version": {"type": "string", "const": SCHEMA_VERSION},
            "requestId": {"type": "string"}, "sourceRevision": {"type": "string"}, "summary": {"type": "string"},
            "assumptions": {"type": "array", "items": {"type": "string"}},
            "warnings": {"type": "array", "items": {"type": "string"}},
            "operations": {"type": "array", "maxItems": MAX_OPERATIONS, "items": {"anyOf": [
                operation(&["type", "diagramPath", "nodeId", "bpmnType"], json!({"type":{"type":"string","const":"replace_node_type"},"diagramPath":diagram_path,"nodeId":node_id,"bpmnType":{"type":"string"}})),
                operation(&["type", "diagramPath", "nodeId", "label"], json!({"type":{"type":"string","const":"update_node_label"},"diagramPath":diagram_path,"nodeId":node_id,"label":{"type":"string"}})),
                operation(&["type", "diagramPath", "nodeId", "name"], json!({"type":{"type":"string","const":"update_node_name"},"diagramPath":diagram_path,"nodeId":node_id,"name":{"type":"string"}})),
                operation(&["type", "diagramPath", "nodeId", "status"], json!({"type":{"type":"string","const":"set_node_status"},"diagramPath":diagram_path,"nodeId":node_id,"status":{"type":"string","enum":["open","new","modify","locked"]}})),
                operation(&["type", "diagramPath", "nodeId", "qualifiedName"], json!({"type":{"type":"string","const":"set_process_reference"},"diagramPath":diagram_path,"nodeId":node_id,"qualifiedName":qualified_name})),
                operation(&["type", "qualifiedName"], json!({"type":{"type":"string","const":"create_process"},"qualifiedName":qualified_name})),
                operation(&["type", "qualifiedName"], json!({"type":{"type":"string","const":"open_process"},"qualifiedName":qualified_name})),
                operation(&["type", "oldQualifiedName", "newQualifiedName"], json!({"type":{"type":"string","const":"rename_process"},"oldQualifiedName":qualified_name,"newQualifiedName":qualified_name})),
                operation(&["type", "diagramPath", "nodeId", "bpmnType", "name", "label", "x", "y"], json!({"type":{"type":"string","const":"add_flow_node"},"diagramPath":diagram_path,"nodeId":node_id,"bpmnType":{"type":"string"},"name":{"type":"string"},"label":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"}})),
                operation(&["type", "diagramPath", "flowId", "sourceId", "targetId"], json!({"type":{"type":"string","const":"connect_sequence_flow"},"diagramPath":diagram_path,"flowId":{"type":"string"},"sourceId":{"type":"string"},"targetId":{"type":"string"}})),
                operation(&["type", "diagramPath", "nodeId", "cmmnType", "name", "label", "x", "y"], json!({"type":{"type":"string","const":"add_plan_item"},"diagramPath":diagram_path,"nodeId":node_id,"cmmnType":{"type":"string","enum":["cmmn:Task","cmmn:HumanTask","cmmn:ProcessTask","cmmn:CaseTask","cmmn:Stage","cmmn:Milestone","cmmn:EventListener"]},"name":{"type":"string"},"label":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"}})),
                operation(&["type", "diagramPath", "connectionId", "sourceId", "targetId"], json!({"type":{"type":"string","const":"connect_cmmn"},"diagramPath":diagram_path,"connectionId":{"type":"string"},"sourceId":{"type":"string"},"targetId":{"type":"string"}})),
                operation(&["type", "diagramPath", "markdown"], json!({"type":{"type":"string","const":"replace_diagram_markdown"},"diagramPath":diagram_path,"markdown":{"type":"string"}})),
                operation(&["type", "diagramPath", "nodeId", "markdown"], json!({"type":{"type":"string","const":"replace_node_markdown"},"diagramPath":diagram_path,"nodeId":node_id,"markdown":{"type":"string"}}))
            ]}}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> AssistantRequest {
        AssistantRequest {
            request_id: "request-1".into(),
            prompt: "Improve it".into(),
            snapshot: json!({"version":"2.0","diagramPath":"main.bpmn","sourceRevision":"abc","primaryNodeId":"Task_1","graph":{"nodes":[{"id":"Task_1","name":"work","label":"Work","status":"open"}]}}),
        }
    }
    #[tokio::test]
    async fn fake_provider_is_correlated_and_deterministic() {
        let result = FakeProvider::new("test".into())
            .propose(&request())
            .await
            .unwrap();
        validate_plan(&result.proposal, &request()).unwrap();
        assert_eq!(result.proposal.operations[0]["nodeId"], "Task_1");
    }
    #[test]
    fn validation_rejects_path_escape_unsupported_and_locked_nodes() {
        let mut value = request();
        value.snapshot["graph"]["nodes"][0]["status"] = json!("locked");
        let plan = AssistantPlan {
            version: SCHEMA_VERSION.into(),
            request_id: "request-1".into(),
            source_revision: "abc".into(),
            summary: "x".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![
                json!({"type":"update_node_label","diagramPath":"main.bpmn","nodeId":"Task_1","label":"x"}),
            ],
        };
        assert!(
            validate_plan(&plan, &value)
                .unwrap_err()
                .to_string()
                .contains("locked")
        );
        let mut escaped = plan.clone();
        escaped.operations[0]["diagramPath"] = json!("../outside.bpmn");
        assert!(validate_plan(&escaped, &request()).is_err());
        let mut raw_xml = plan.clone();
        raw_xml.operations[0]["rawXml"] = json!("<bpmn />");
        assert!(validate_plan(&raw_xml, &request()).is_err());
        let mut unsupported = plan;
        unsupported.operations[0]["type"] = json!("shell");
        assert!(validate_plan(&unsupported, &request()).is_err());
    }

    #[test]
    fn operation_schema_uses_closed_strict_variants() {
        let schema = operation_plan_schema();
        let variants = schema
            .pointer("/properties/operations/items/anyOf")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(variants.len(), 14);
        assert!(
            variants
                .iter()
                .all(|variant| variant["additionalProperties"] == false)
        );
        assert!(variants.iter().all(|variant| {
            let required = variant["required"].as_array().unwrap();
            variant["properties"]
                .as_object()
                .unwrap()
                .keys()
                .all(|key| required.iter().any(|value| value == key))
        }));
    }

    #[test]
    fn node_status_operations_accept_only_known_statuses() {
        let mut plan = AssistantPlan {
            version: SCHEMA_VERSION.into(),
            request_id: "request-1".into(),
            source_revision: "abc".into(),
            summary: "Mark implementation work".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![
                json!({"type":"set_node_status","diagramPath":"main.bpmn","nodeId":"Task_1","status":"new"}),
            ],
        };
        validate_plan(&plan, &request()).unwrap();
        plan.operations[0]["status"] = json!("pending");
        assert!(validate_plan(&plan, &request()).is_err());
    }

    #[test]
    fn cmmn_operations_validate_business_members_process_links_and_connections() {
        let request = AssistantRequest {
            request_id: "request-cmmn".into(),
            prompt: "Trace the need to its design".into(),
            snapshot: json!({
                "version":"2.0", "diagramKind":"cmmn", "diagramPath":"cybling/main.cmmn", "sourceRevision":"cmmn-rev",
                "graph":{"nodes":[{"id":"PlanItem_Need","type":"cmmn:HumanTask","name":"cybling#captureNeed","status":"open"}],"flows":[]}
            }),
        };
        let plan = AssistantPlan {
            version: SCHEMA_VERSION.into(),
            request_id: "request-cmmn".into(),
            source_revision: "cmmn-rev".into(),
            summary: "Trace need".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![
                json!({"type":"add_plan_item","diagramPath":"cybling/main.cmmn","nodeId":"PlanItem_Birth","cmmnType":"cmmn:ProcessTask","name":"cybling.sdk.Birth","label":"Birth design"}),
                json!({"type":"connect_cmmn","diagramPath":"cybling/main.cmmn","connectionId":"Association_Birth","sourceId":"PlanItem_Need","targetId":"PlanItem_Birth"}),
                json!({"type":"update_node_name","diagramPath":"cybling/main.cmmn","nodeId":"Association_Birth","name":"cybling#birthTrace"}),
                json!({"type":"set_process_reference","diagramPath":"cybling/main.cmmn","nodeId":"PlanItem_Birth","qualifiedName":"cybling.sdk.Birth"}),
            ],
        };
        validate_plan(&plan, &request).unwrap();

        let mut invalid_member = plan.clone();
        invalid_member.operations[0]["cmmnType"] = json!("cmmn:Stage");
        assert!(validate_plan(&invalid_member, &request).is_err());
        let mut missing_target = plan;
        missing_target.operations[1]["targetId"] = json!("Missing");
        assert!(validate_plan(&missing_target, &request).is_err());
    }

    #[test]
    fn provider_paths_are_canonicalized_before_validation() {
        let mut plan = AssistantPlan {
            version: SCHEMA_VERSION.into(),
            request_id: "request-1".into(),
            source_revision: "abc".into(),
            summary: "x".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![
                json!({"type":"create_composition","path":" /schematics/cybling-setup "}),
            ],
        };
        canonicalize_plan_paths(&mut plan).unwrap();
        assert_eq!(plan.operations[0]["path"], "cybling-setup");
        plan.operations[0]["path"] = json!("cybling-setup/main.bpmn");
        canonicalize_plan_paths(&mut plan).unwrap();
        assert_eq!(plan.operations[0]["path"], "cybling-setup/main.bpmn");
        plan.operations[0]["path"] = json!("");
        assert!(canonicalize_plan_paths(&mut plan).is_err());
    }
}
