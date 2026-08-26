use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashSet,
    env,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

pub const SCHEMA_VERSION: &str = "1.0";
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
        let operations = if request.prompt.to_ascii_lowercase().contains("subprocess")
            && primary.is_some()
        {
            let node_id = primary.unwrap();
            let child = "assistant-subprocess";
            vec![
                json!({"type":"replace_node_type","diagramPath":diagram,"nodeId":node_id,"bpmnType":"bpmn:CallActivity"}),
                json!({"type":"set_composition_link","diagramPath":diagram,"nodeId":node_id,"path":child}),
                json!({"type":"create_composition","path":child}),
                json!({"type":"add_flow_node","diagramPath":format!("{child}/main.bpmn"),"nodeId":"AssistantStep_1","bpmnType":"bpmn:Task","label":"First step","x":180,"y":330}),
                json!({"type":"add_flow_node","diagramPath":format!("{child}/main.bpmn"),"nodeId":"AssistantStep_2","bpmnType":"bpmn:Task","label":"Second step","x":350,"y":330}),
                json!({"type":"add_flow_node","diagramPath":format!("{child}/main.bpmn"),"nodeId":"AssistantStep_3","bpmnType":"bpmn:Task","label":"Third step","x":520,"y":330}),
                json!({"type":"add_flow_node","diagramPath":format!("{child}/main.bpmn"),"nodeId":"AssistantStep_4","bpmnType":"bpmn:Task","label":"Fourth step","x":690,"y":330}),
                json!({"type":"connect_sequence_flow","diagramPath":format!("{child}/main.bpmn"),"flowId":"AssistantFlow_1","sourceId":"AssistantStep_1","targetId":"AssistantStep_2"}),
                json!({"type":"connect_sequence_flow","diagramPath":format!("{child}/main.bpmn"),"flowId":"AssistantFlow_2","sourceId":"AssistantStep_2","targetId":"AssistantStep_3"}),
                json!({"type":"connect_sequence_flow","diagramPath":format!("{child}/main.bpmn"),"flowId":"AssistantFlow_3","sourceId":"AssistantStep_3","targetId":"AssistantStep_4"}),
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
            "Return only a Software Schematic operation plan matching the supplied JSON schema. Do not inspect files, run tools, or mutate anything. Preserve requestId={} and sourceRevision={}. Every path must be non-empty and relative to the schematics directory, never absolute and never prefixed with schematics/. For create_composition, open_composition, and set_composition_link, use only a composition folder such as checkout. For diagramPath, use a BPMN file such as checkout/main.bpmn. When a requested child flow needs a new composition and none is named, choose a short folder slug from the selected node label. Interpret system task as bpmn:ServiceTask. Emit each intended operation only once. User request: {}\nContext: {}",
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
        .unwrap_or_else(|| "fake".into());
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
        let operation_type = operation["type"].as_str().unwrap_or_default().to_owned();
        for field in ["path", "diagramPath"] {
            let Some(raw) = operation[field].as_str() else {
                continue;
            };
            let trimmed = raw.trim();
            let mut normalized = trimmed
                .strip_prefix("/schematics/")
                .or_else(|| trimmed.strip_prefix("schematics/"))
                .unwrap_or(trimmed);
            if field == "path"
                && matches!(
                    operation_type.as_str(),
                    "create_composition" | "open_composition" | "set_composition_link"
                )
            {
                normalized = normalized.strip_suffix("/main.bpmn").unwrap_or(normalized);
            }
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
        "set_composition_link",
        "create_composition",
        "open_composition",
        "add_flow_node",
        "connect_sequence_flow",
        "replace_diagram_markdown",
        "replace_node_markdown",
    ]
    .into_iter()
    .collect();
    let locked: HashSet<&str> = request
        .snapshot
        .pointer("/graph/nodes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|node| node["status"] == "locked")
        .filter_map(|node| node["id"].as_str())
        .collect();
    let mut ids = HashSet::new();
    for operation in &plan.operations {
        let kind = operation["type"]
            .as_str()
            .ok_or_else(|| Error::Message("assistant operation has no type".into()))?;
        if !allowed.contains(kind) {
            return Err(Error::Message(format!(
                "unsupported assistant operation: {kind}"
            )));
        }
        for field in ["path", "diagramPath"] {
            if let Some(path) = operation[field].as_str() {
                confined_path(path)?;
            }
        }
        if let Some(id) = operation["nodeId"].as_str() {
            valid_id(id)?;
            if locked.contains(id) {
                return Err(Error::Message(format!(
                    "locked node cannot be changed: {id}"
                )));
            }
            if kind == "add_flow_node" && !ids.insert(id) {
                return Err(Error::Message(format!("duplicate created ID: {id}")));
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
    }
    Ok(())
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
    let assistant_path = json!({
        "type": "string",
        "minLength": 1,
        "pattern": "^[^/\\\\].*$",
        "description": "A non-empty path relative to the schematics directory, without a schematics/ prefix."
    });
    let diagram_path = assistant_path.clone();
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
                operation(&["type", "diagramPath", "nodeId", "path"], json!({"type":{"type":"string","const":"set_composition_link"},"diagramPath":diagram_path,"nodeId":node_id,"path":assistant_path})),
                operation(&["type", "path"], json!({"type":{"type":"string","const":"create_composition"},"path":assistant_path})),
                operation(&["type", "path"], json!({"type":{"type":"string","const":"open_composition"},"path":assistant_path})),
                operation(&["type", "diagramPath", "nodeId", "bpmnType", "label", "x", "y"], json!({"type":{"type":"string","const":"add_flow_node"},"diagramPath":diagram_path,"nodeId":node_id,"bpmnType":{"type":"string"},"label":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"}})),
                operation(&["type", "diagramPath", "flowId", "sourceId", "targetId"], json!({"type":{"type":"string","const":"connect_sequence_flow"},"diagramPath":diagram_path,"flowId":{"type":"string"},"sourceId":{"type":"string"},"targetId":{"type":"string"}})),
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
            snapshot: json!({"version":"1.0","diagramPath":"main.bpmn","sourceRevision":"abc","primaryNodeId":"Task_1","graph":{"nodes":[{"id":"Task_1","label":"Work","status":"open"}]}}),
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
            version: "1.0".into(),
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
        assert_eq!(variants.len(), 9);
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
    fn provider_paths_are_canonicalized_before_validation() {
        let mut plan = AssistantPlan {
            version: "1.0".into(),
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
        assert_eq!(plan.operations[0]["path"], "cybling-setup");
        plan.operations[0]["path"] = json!("");
        assert!(canonicalize_plan_paths(&mut plan).is_err());
    }
}
