use crate::{
    Error, Result, composition_folder_for_name, validate_element_name, validate_member_name,
    validate_package_name, validate_qualified_process_name,
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
pub const MAX_TURNS: usize = 40;
pub const MAX_TURN_BYTES: usize = 16 * 1024;
pub const MAX_TRANSCRIPT_BYTES: usize = 128 * 1024;
pub const OPERATION_TYPES: &[&str] = &[
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
    "add_participant",
    "connect_message_flow",
    "add_plan_item",
    "connect_cmmn",
    "replace_diagram_markdown",
    "replace_node_markdown",
    "move_element",
    "remove_element",
    "disconnect_flow",
    "replace_edge_markdown",
];

pub fn operation_registry() -> Value {
    let plan_schema = operation_plan_schema();
    let variants = plan_schema
        .pointer("/properties/operations/items/anyOf")
        .and_then(Value::as_array)
        .expect("operation plan schema must expose variants");
    let mut operations = serde_json::Map::new();
    for operation in OPERATION_TYPES {
        let diagrams = if matches!(*operation, "add_plan_item" | "connect_cmmn") {
            json!(["cmmn"])
        } else if matches!(
            *operation,
            "replace_node_type"
                | "create_process"
                | "open_process"
                | "rename_process"
                | "add_flow_node"
                | "connect_sequence_flow"
                | "add_participant"
                | "connect_message_flow"
        ) {
            json!(["bpmn"])
        } else {
            json!(["bpmn", "cmmn"])
        };
        let schema = variants
            .iter()
            .find(|variant| {
                variant
                    .pointer("/properties/type/const")
                    .and_then(Value::as_str)
                    == Some(operation)
            })
            .expect("every registered operation must have a schema");
        let executor = if matches!(
            *operation,
            "create_process" | "open_process" | "rename_process"
        ) {
            "compositionApi"
        } else if matches!(
            *operation,
            "replace_diagram_markdown" | "replace_node_markdown" | "replace_edge_markdown"
        ) {
            "documentApi"
        } else {
            "modeler"
        };
        let reversal = match *operation {
            "create_process" => "revisionCheckedDelete",
            "open_process" => "tabState",
            "rename_process" => "revisionCheckedRename",
            "replace_diagram_markdown" | "replace_node_markdown" | "replace_edge_markdown" => {
                "conditionalDocumentRestore"
            }
            _ => "diagramSnapshot",
        };
        operations.insert(
            (*operation).into(),
            json!({
                "diagrams": diagrams,
                "schema": schema,
                "executor": executor,
                "reversal": reversal,
                "preview": true,
                "apply": true,
                "undo": true,
                "rollback": true
            }),
        );
    }
    json!({ "version": SCHEMA_VERSION, "operations": operations })
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AssistantTurnRole {
    User,
    Assistant,
    ProposalSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssistantTurn {
    pub role: AssistantTurnRole,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssistantRequest {
    pub request_id: String,
    pub prompt: String,
    pub snapshot: Value,
    #[serde(default)]
    pub turns: Vec<AssistantTurn>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AssistantConversationRequest {
    pub request_id: String,
    pub snapshot: Value,
    pub turns: Vec<AssistantTurn>,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantConversationResult {
    pub reply: String,
    pub provider: String,
    pub model: String,
    pub usage: Value,
}

#[allow(async_fn_in_trait)]
pub trait AssistantProvider {
    async fn converse(
        &self,
        request: &AssistantConversationRequest,
    ) -> Result<AssistantConversationResult>;
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
    async fn converse(
        &self,
        request: &AssistantConversationRequest,
    ) -> Result<AssistantConversationResult> {
        let message = request
            .turns
            .last()
            .map(|turn| turn.text.as_str())
            .unwrap_or_default();
        if message.contains("[timeout]") {
            tokio::time::sleep(Duration::from_secs(35)).await;
        }
        if message.contains("[auth]") {
            return Err(Error::Message(
                "assistant provider authentication failed; check host configuration".into(),
            ));
        }
        if message.contains("[invalid]") {
            return Err(Error::Message(
                "assistant provider returned an invalid conversational response".into(),
            ));
        }
        let target = request
            .snapshot
            .pointer("/primaryElementId")
            .or_else(|| request.snapshot.pointer("/primaryNodeId"))
            .and_then(Value::as_str)
            .unwrap_or("the active diagram");
        Ok(AssistantConversationResult {
            reply: format!("Let's refine {target}. I understand: {}", message.trim()),
            provider: "fake".into(),
            model: self.model.clone(),
            usage: json!({"inputTokens": 0, "outputTokens": 0}),
        })
    }

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
            .pointer("/primaryElementId")
            .or_else(|| request.snapshot.pointer("/primaryNodeId"))
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
                json!({"type":"replace_node_markdown","diagramPath":child_diagram,"nodeId":"AssistantStep_1","markdown":"# First step\n\nPerform the first operation in the assistant subprocess."}),
                json!({"type":"replace_node_markdown","diagramPath":child_diagram,"nodeId":"AssistantStep_2","markdown":"# Second step\n\nPerform the second operation in the assistant subprocess."}),
                json!({"type":"replace_node_markdown","diagramPath":child_diagram,"nodeId":"AssistantStep_3","markdown":"# Third step\n\nPerform the third operation in the assistant subprocess."}),
                json!({"type":"replace_node_markdown","diagramPath":child_diagram,"nodeId":"AssistantStep_4","markdown":"# Fourth step\n\nPerform the fourth operation in the assistant subprocess."}),
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

fn conversation_prompt(request: &AssistantConversationRequest) -> Result<String> {
    let turns = serde_json::to_string(&request.turns)
        .map_err(|_| Error::Message("assistant transcript could not be encoded".into()))?;
    Ok(format!(
        "Continue a Software Schematic design interview using only the supplied context and current-invocation turns. Answer with concise prose or ask one focused follow-up question. Do not return an operation plan, JSON mutations, code, shell commands, patches, raw diagram XML, or claim to have changed anything. Do not inspect files or use tools. Current-invocation turns: {turns}\nScoped context: {}",
        request.snapshot
    ))
}

fn proposal_prompt(request: &AssistantRequest) -> Result<String> {
    let turns = serde_json::to_string(&request.turns)
        .map_err(|_| Error::Message("assistant transcript could not be encoded".into()))?;
    Ok(format!(
        "Return only a Software Schematic operation plan matching the supplied JSON schema. Do not inspect files, run tools, or mutate anything. Preserve requestId={} and sourceRevision={}. The current persisted snapshot is authoritative if earlier conversation differs. Preserve every existing ID, Type, Label, Name, Implementation Status, and Documentation unless an explicit operation changes it. For element scope, change only the primary element in the parent diagram; unrelated multi-element changes require diagram scope. A directly owned subprocess composition may be created or populated. Never emit a composition folder path: use a complete Name. BPMN Process Names use package.Process and BPMN members use package.Process#member. CMMN business-need members use package#member, while a CMMN ProcessTask link uses package.Process. A process rename must use rename_process. For diagramPath, use the path already present in context. Interpret system task as bpmn:ServiceTask. Emit each intended operation only once. Explicit request: {}\nCurrent-invocation turns: {}\nCurrent persisted context: {}",
        request.request_id,
        request.snapshot["sourceRevision"]
            .as_str()
            .unwrap_or_default(),
        request.prompt,
        turns,
        request.snapshot
    ))
}

fn check_local_output(kind: &str, output: &std::process::Output, action: &str) -> Result<()> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("401")
            || stderr.to_ascii_lowercase().contains("not logged in")
            || stderr.to_ascii_lowercase().contains("authentication")
        {
            return Err(Error::Message(format!(
                "{kind} is not authenticated; run ./ssw auth login --provider {kind}"
            )));
        }
        return Err(Error::Message(format!(
            "{kind} could not {action} (exit status {}); run ./ssw auth status and retry",
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
    Ok(())
}

fn validate_conversation_reply(reply: &str) -> Result<()> {
    let reply = reply.trim();
    if reply.is_empty() || reply.len() > MAX_RESPONSE_BYTES {
        return Err(Error::Message(
            "assistant conversational response is empty or exceeds the configured limit".into(),
        ));
    }
    if serde_json::from_str::<Value>(reply)
        .ok()
        .is_some_and(|value| value.get("operations").is_some())
    {
        return Err(Error::Message(
            "assistant returned structured mutations during interview mode".into(),
        ));
    }
    Ok(())
}

impl LocalCliProvider {
    pub fn new(kind: &str, project: PathBuf) -> Self {
        Self {
            kind: kind.into(),
            project,
        }
    }

    fn run_conversation(
        &self,
        request: &AssistantConversationRequest,
    ) -> Result<AssistantConversationResult> {
        let prompt = conversation_prompt(request)?;
        let output = if self.kind == "codex" {
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
                ])
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
        check_local_output(&self.kind, &output, "continue the interview")?;
        let reply = if self.kind == "claude" {
            let envelope: Value = serde_json::from_slice(&output.stdout)
                .map_err(|_| Error::Message("Claude returned malformed JSON".into()))?;
            envelope["result"]
                .as_str()
                .ok_or_else(|| Error::Message("Claude returned no conversational reply".into()))?
                .to_owned()
        } else {
            String::from_utf8(output.stdout)
                .map_err(|_| Error::Message("Codex returned non-text output".into()))?
        };
        validate_conversation_reply(&reply)?;
        Ok(AssistantConversationResult {
            reply: reply.trim().to_owned(),
            provider: self.kind.clone(),
            model: "account-default".into(),
            usage: json!({}),
        })
    }

    fn run(&self, request: &AssistantRequest) -> Result<AssistantResult> {
        let schema = operation_plan_schema();
        let prompt = proposal_prompt(request)?;
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
    async fn converse(
        &self,
        request: &AssistantConversationRequest,
    ) -> Result<AssistantConversationResult> {
        self.run_conversation(request)
    }

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
    async fn converse(
        &self,
        request: &AssistantConversationRequest,
    ) -> Result<AssistantConversationResult> {
        let response = self.client.post(&self.endpoint).bearer_auth(&self.key).json(&json!({
            "model": self.model,
            "instructions": "Continue a Software Schematic design interview. Answer with concise prose or one focused follow-up question. Do not return operation plans, JSON mutations, code, shell commands, patches, or raw diagram XML, and do not claim to have changed the project.",
            "input": [{"role":"user","content":[{"type":"input_text","text": conversation_prompt(request)?}]}]
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
        let reply = envelope
            .pointer("/output/0/content/0/text")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                Error::Message("assistant provider returned no conversational reply".into())
            })?;
        validate_conversation_reply(reply)?;
        Ok(AssistantConversationResult {
            reply: reply.trim().to_owned(),
            provider: "openai".into(),
            model: self.model.clone(),
            usage: envelope.get("usage").cloned().unwrap_or_else(|| json!({})),
        })
    }

    async fn propose(&self, request: &AssistantRequest) -> Result<AssistantResult> {
        let schema = operation_plan_schema();
        let response = self.client.post(&self.endpoint).bearer_auth(&self.key).json(&json!({
            "model": self.model,
            "instructions": "You propose safe Software Schematic changes. Return only the required structured operation plan. The current persisted snapshot is authoritative. For element scope, change only the primary element in the parent diagram; unrelated multi-element changes require diagram scope. Never emit code, shell commands, patches, or raw BPMN XML.",
            "input": [{"role":"user","content":[{"type":"input_text","text": proposal_prompt(request)?}]}],
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

pub async fn converse(
    request: &AssistantConversationRequest,
    project: PathBuf,
    configured_provider: Option<String>,
) -> Result<AssistantConversationResult> {
    validate_conversation_request(request)?;
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
    match provider.as_str() {
        "fake" => FakeProvider::new(model).converse(request).await,
        "openai" => {
            let key = env::var("OPENAI_API_KEY").map_err(|_| Error::Message("OpenAI assistance is not configured; set OPENAI_API_KEY in the host environment".into()))?;
            let endpoint = env::var("SSW_ASSISTANT_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1/responses".into());
            OpenAiProvider::new(model, endpoint, key)?
                .converse(request)
                .await
        }
        "codex" | "claude" => {
            LocalCliProvider::new(&provider, project)
                .converse(request)
                .await
        }
        _ => Err(Error::Message(
            "assistant provider is not allowlisted; use codex, claude, fake, or openai".into(),
        )),
    }
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

fn validate_request_identity_and_snapshot(request_id: &str, snapshot: &Value) -> Result<()> {
    if request_id.is_empty() || request_id.len() > 128 {
        return Err(Error::Message("assistant request ID is invalid".into()));
    }
    if snapshot["version"] != SCHEMA_VERSION {
        return Err(Error::Message(
            "unsupported assistant context version".into(),
        ));
    }
    confined_path(snapshot["diagramPath"].as_str().unwrap_or_default())?;
    Ok(())
}

fn validate_turns(turns: &[AssistantTurn], require_last_user: bool) -> Result<()> {
    if turns.is_empty() || turns.len() > MAX_TURNS {
        return Err(Error::Message(format!(
            "assistant transcript must contain 1 to {MAX_TURNS} turns"
        )));
    }
    let mut total = 0usize;
    let mut has_user = false;
    for turn in turns {
        let bytes = turn.text.len();
        if turn.text.trim().is_empty() || bytes > MAX_TURN_BYTES {
            return Err(Error::Message(format!(
                "assistant transcript turn is empty or exceeds {MAX_TURN_BYTES} bytes"
            )));
        }
        total = total.saturating_add(bytes);
        has_user |= turn.role == AssistantTurnRole::User;
    }
    if !has_user || total > MAX_TRANSCRIPT_BYTES {
        return Err(Error::Message(format!(
            "assistant transcript has no user turn or exceeds {MAX_TRANSCRIPT_BYTES} bytes"
        )));
    }
    if require_last_user && turns.last().map(|turn| turn.role) != Some(AssistantTurnRole::User) {
        return Err(Error::Message(
            "assistant interview request must end with a user turn".into(),
        ));
    }
    Ok(())
}

pub fn validate_conversation_request(request: &AssistantConversationRequest) -> Result<()> {
    validate_request_identity_and_snapshot(&request.request_id, &request.snapshot)?;
    validate_turns(&request.turns, true)
}

pub fn validate_request(request: &AssistantRequest) -> Result<()> {
    validate_request_identity_and_snapshot(&request.request_id, &request.snapshot)?;
    if request.prompt.trim().is_empty() || request.prompt.len() > MAX_TURN_BYTES {
        return Err(Error::Message(
            "assistant prompt is empty or exceeds the configured limit".into(),
        ));
    }
    if !request.turns.is_empty() {
        validate_turns(&request.turns, false)?;
    }
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
    let allowed: HashSet<&str> = OPERATION_TYPES.iter().copied().collect();
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
    let mut created_message_flows = HashSet::new();
    let mut documented_edges = HashSet::new();
    let mut created_documented_nodes = HashSet::new();
    let mut documented_nodes = HashSet::new();
    let active_diagram = request.snapshot["diagramPath"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    let primary_element_id = request
        .snapshot
        .pointer("/primaryElementId")
        .or_else(|| request.snapshot.pointer("/primaryNodeId"))
        .and_then(Value::as_str);
    let element_scoped = primary_element_id.is_some()
        && request.snapshot["scope"].as_str().unwrap_or("node") != "diagram";
    let mut anchored_process_names: HashSet<&str> = plan
        .operations
        .iter()
        .filter(|operation| operation["type"] == "set_process_reference")
        .filter(|operation| operation["nodeId"].as_str() == primary_element_id)
        .filter_map(|operation| operation["qualifiedName"].as_str())
        .collect();
    if let Some(primary) = primary_element_id
        && let Some(existing_name) = elements
            .iter()
            .find(|element| element["id"] == primary)
            .and_then(|element| element["name"].as_str())
        && !existing_name.contains('#')
    {
        anchored_process_names.insert(existing_name);
    }
    let mut allowed_diagrams = HashSet::from([active_diagram.clone()]);
    for operation in &plan.operations {
        let kind = operation["type"].as_str().unwrap_or_default();
        if matches!(kind, "create_process" | "open_process")
            && let Some(name) = operation["qualifiedName"].as_str()
        {
            if element_scoped && !anchored_process_names.contains(name) {
                return Err(Error::Message(
                    "element-scoped proposals may only create or open a process directly referenced by the primary element; use diagram-level assistance for multi-node changes".into(),
                ));
            }
            let folder = composition_folder_for_name(name)?;
            allowed_diagrams.insert(format!(
                "{}/main.bpmn",
                folder.to_string_lossy().replace('\\', "/")
            ));
        }
        if kind == "rename_process"
            && let Some(name) = operation["newQualifiedName"].as_str()
        {
            if element_scoped {
                return Err(Error::Message(
                    "element-scoped proposals cannot rename the parent process; use diagram-level assistance for multi-node changes".into(),
                ));
            }
            let folder = composition_folder_for_name(name)?;
            allowed_diagrams.insert(format!(
                "{}/main.bpmn",
                folder.to_string_lossy().replace('\\', "/")
            ));
        }
    }
    for operation in &plan.operations {
        let kind = operation["type"]
            .as_str()
            .ok_or_else(|| Error::Message("assistant operation has no type".into()))?;
        if !allowed.contains(kind) {
            return Err(Error::Message(format!(
                "unsupported assistant operation: {kind}"
            )));
        }
        if element_scoped {
            let operation_diagram = operation["diagramPath"]
                .as_str()
                .unwrap_or(active_diagram.as_str());
            if operation_diagram == active_diagram {
                let primary = primary_element_id.unwrap_or_default();
                let targets_primary = match kind {
                    "replace_node_type"
                    | "update_node_label"
                    | "update_node_name"
                    | "set_node_status"
                    | "set_process_reference"
                    | "replace_node_markdown" => operation["nodeId"].as_str() == Some(primary),
                    "move_element" | "remove_element" => {
                        operation["elementId"].as_str() == Some(primary)
                    }
                    "replace_edge_markdown" => operation["edgeId"].as_str() == Some(primary),
                    "create_process" | "open_process" => operation["qualifiedName"]
                        .as_str()
                        .is_some_and(|name| anchored_process_names.contains(name)),
                    _ => false,
                };
                if !targets_primary {
                    return Err(Error::Message(
                        "element-scoped proposal targets an unrelated parent-diagram element; use diagram-level assistance for multi-node changes".into(),
                    ));
                }
            }
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
                if field == "diagramPath" && !allowed_diagrams.contains(path) {
                    return Err(Error::Message(format!(
                        "diagram path is outside this proposal scope: {path}"
                    )));
                }
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
        if matches!(
            kind,
            "update_node_label" | "add_flow_node" | "add_plan_item"
        ) && operation["label"].is_string()
            && !concise_label(operation["label"].as_str().unwrap())
        {
            return Err(Error::Message(
                "activity Label must remain concise; enumerate procedural steps in Markdown".into(),
            ));
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
                created_documented_nodes.insert(id);
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
        if kind == "add_participant" {
            if request.snapshot["diagramKind"] == "cmmn" {
                return Err(Error::Message(
                    "BPMN participants require a BPMN diagram".into(),
                ));
            }
            let id = operation["participantId"]
                .as_str()
                .ok_or_else(|| Error::Message("participant has no ID".into()))?;
            valid_id(id)?;
            if !ids.insert(id) {
                return Err(Error::Message(format!("duplicate created ID: {id}")));
            }
            node_types.insert(id, "bpmn:Participant");
        }
        if kind == "connect_message_flow" {
            if request.snapshot["diagramKind"] == "cmmn" {
                return Err(Error::Message(
                    "BPMN message flows require a BPMN diagram".into(),
                ));
            }
            let id = operation["flowId"]
                .as_str()
                .ok_or_else(|| Error::Message("message flow has no ID".into()))?;
            valid_id(id)?;
            if !ids.insert(id) {
                return Err(Error::Message(format!("duplicate created ID: {id}")));
            }
            let source = operation["sourceId"].as_str().unwrap_or_default();
            let target = operation["targetId"].as_str().unwrap_or_default();
            if !ids.contains(source) || !ids.contains(target) {
                return Err(Error::Message(
                    "message flow references an unknown activity or participant".into(),
                ));
            }
            if node_types.get(source) != Some(&"bpmn:Participant")
                && node_types.get(target) != Some(&"bpmn:Participant")
            {
                return Err(Error::Message(
                    "message flow must connect an activity to an external participant".into(),
                ));
            }
            if !concise_label(operation["label"].as_str().unwrap_or_default()) {
                return Err(Error::Message(
                    "message flow Label must be a concise function call or data name".into(),
                ));
            }
            node_types.insert(id, "bpmn:MessageFlow");
            created_message_flows.insert(id);
        }
        if kind == "move_element" {
            let id = operation["elementId"].as_str().unwrap_or_default();
            if !ids.contains(id) {
                return Err(Error::Message(format!("unknown element: {id}")));
            }
        }
        if kind == "remove_element" {
            let id = operation["elementId"].as_str().unwrap_or_default();
            if locked.contains(id) {
                return Err(Error::Message(format!(
                    "locked element cannot be removed: {id}"
                )));
            }
            if !ids.remove(id) {
                return Err(Error::Message(format!("unknown element: {id}")));
            }
        }
        if kind == "disconnect_flow" {
            let id = operation["flowId"].as_str().unwrap_or_default();
            if locked.contains(id) {
                return Err(Error::Message(format!(
                    "locked connection cannot be removed: {id}"
                )));
            }
            let connection_type = node_types.get(id).copied().unwrap_or_default();
            if !matches!(
                connection_type,
                "bpmn:SequenceFlow" | "bpmn:MessageFlow" | "cmmn:Association"
            ) || !ids.remove(id)
            {
                return Err(Error::Message(format!("unknown connection: {id}")));
            }
        }
        if kind == "replace_edge_markdown" {
            let id = operation["edgeId"].as_str().unwrap_or_default();
            if locked.contains(id) {
                return Err(Error::Message(format!(
                    "locked edge cannot be documented: {id}"
                )));
            }
            let is_edge = created_message_flows.contains(id)
                || elements.iter().any(|element| {
                    element["id"] == id
                        && element.get("source").is_some()
                        && element.get("target").is_some()
                });
            if !is_edge {
                return Err(Error::Message(format!("unknown edge: {id}")));
            }
            if operation["markdown"]
                .as_str()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                return Err(Error::Message(format!(
                    "edge Markdown cannot be empty: {id}"
                )));
            }
            documented_edges.insert(id);
        }
        if kind == "replace_node_markdown" {
            let id = operation["nodeId"].as_str().unwrap_or_default();
            if operation["markdown"]
                .as_str()
                .unwrap_or_default()
                .trim()
                .is_empty()
            {
                return Err(Error::Message(format!(
                    "node Markdown cannot be empty: {id}"
                )));
            }
            documented_nodes.insert(id);
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
    for id in created_message_flows {
        if !documented_edges.contains(id) {
            return Err(Error::Message(format!(
                "message flow {id} requires an edge Markdown contract in the same proposal"
            )));
        }
    }
    for id in created_documented_nodes {
        if !documented_nodes.contains(id) {
            return Err(Error::Message(format!(
                "created activity or event {id} requires owned Markdown in the same proposal"
            )));
        }
    }
    Ok(())
}

fn concise_label(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value.len() <= 80
        && value.split_whitespace().count() <= 12
        && !value.contains('\n')
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
                operation(&["type", "diagramPath", "participantId", "label", "x", "y"], json!({"type":{"type":"string","const":"add_participant"},"diagramPath":diagram_path,"participantId":{"type":"string"},"label":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"}})),
                operation(&["type", "diagramPath", "flowId", "sourceId", "targetId", "label"], json!({"type":{"type":"string","const":"connect_message_flow"},"diagramPath":diagram_path,"flowId":{"type":"string"},"sourceId":{"type":"string"},"targetId":{"type":"string"},"label":{"type":"string","minLength":1,"maxLength":80}})),
                operation(&["type", "diagramPath", "elementId", "x", "y"], json!({"type":{"type":"string","const":"move_element"},"diagramPath":diagram_path,"elementId":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"}})),
                operation(&["type", "diagramPath", "elementId"], json!({"type":{"type":"string","const":"remove_element"},"diagramPath":diagram_path,"elementId":{"type":"string"}})),
                operation(&["type", "diagramPath", "flowId"], json!({"type":{"type":"string","const":"disconnect_flow"},"diagramPath":diagram_path,"flowId":{"type":"string"}})),
                operation(&["type", "diagramPath", "nodeId", "cmmnType", "name", "label", "x", "y"], json!({"type":{"type":"string","const":"add_plan_item"},"diagramPath":diagram_path,"nodeId":node_id,"cmmnType":{"type":"string","enum":["cmmn:Task","cmmn:HumanTask","cmmn:ProcessTask","cmmn:CaseTask","cmmn:Stage","cmmn:Milestone","cmmn:EventListener"]},"name":{"type":"string"},"label":{"type":"string"},"x":{"type":"number"},"y":{"type":"number"}})),
                operation(&["type", "diagramPath", "connectionId", "sourceId", "targetId"], json!({"type":{"type":"string","const":"connect_cmmn"},"diagramPath":diagram_path,"connectionId":{"type":"string"},"sourceId":{"type":"string"},"targetId":{"type":"string"}})),
                operation(&["type", "diagramPath", "markdown"], json!({"type":{"type":"string","const":"replace_diagram_markdown"},"diagramPath":diagram_path,"markdown":{"type":"string"}})),
                operation(&["type", "diagramPath", "nodeId", "markdown"], json!({"type":{"type":"string","const":"replace_node_markdown"},"diagramPath":diagram_path,"nodeId":node_id,"markdown":{"type":"string"}})),
                operation(&["type", "diagramPath", "edgeId", "markdown"], json!({"type":{"type":"string","const":"replace_edge_markdown"},"diagramPath":diagram_path,"edgeId":{"type":"string"},"markdown":{"type":"string"}}))
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
            turns: vec![],
        }
    }

    fn conversation_request() -> AssistantConversationRequest {
        AssistantConversationRequest {
            request_id: "conversation-1".into(),
            snapshot: json!({"version":"2.0","scope":"node","diagramPath":"main.bpmn","sourceRevision":"abc","primaryElementId":"Task_1","primaryNodeId":"Task_1","graph":{"nodes":[{"id":"Task_1","type":"bpmn:Task","name":"sales.Order#work","label":"Work","status":"open"}],"flows":[]}}),
            turns: vec![AssistantTurn {
                role: AssistantTurnRole::User,
                text: "What should this task do?".into(),
            }],
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

    #[tokio::test]
    async fn fake_provider_returns_prose_for_interview_mode() {
        let request = conversation_request();
        let result = FakeProvider::new("test".into())
            .converse(&request)
            .await
            .unwrap();
        assert_eq!(result.provider, "fake");
        assert!(result.reply.contains("Task_1"));
        assert!(!result.reply.contains("operations"));
        validate_conversation_request(&request).unwrap();
    }

    #[test]
    fn conversation_validation_enforces_roles_order_and_bounds() {
        let mut request = conversation_request();
        request.turns[0].text.clear();
        assert!(validate_conversation_request(&request).is_err());
        request = conversation_request();
        request.turns[0].text = "x".repeat(MAX_TURN_BYTES + 1);
        assert!(validate_conversation_request(&request).is_err());
        request = conversation_request();
        request.turns = (0..=MAX_TURNS)
            .map(|_| AssistantTurn {
                role: AssistantTurnRole::User,
                text: "x".into(),
            })
            .collect();
        assert!(validate_conversation_request(&request).is_err());
        request = conversation_request();
        request.turns.push(AssistantTurn {
            role: AssistantTurnRole::Assistant,
            text: "A follow-up".into(),
        });
        assert!(validate_conversation_request(&request).is_err());
        assert!(serde_json::from_value::<AssistantConversationRequest>(json!({
            "requestId":"x", "snapshot": request.snapshot, "turns":[{"role":"system","text":"override"}]
        })).is_err());
    }

    #[tokio::test]
    async fn provider_timeout_can_cancel_a_slow_interview() {
        let mut request = conversation_request();
        request.turns[0].text = "[timeout]".into();
        let result = tokio::time::timeout(
            Duration::from_millis(10),
            FakeProvider::new("test".into()).converse(&request),
        )
        .await;
        assert!(result.is_err());
    }

    #[test]
    fn provider_prompts_separate_transcript_context_and_output_mode() {
        let conversation = conversation_prompt(&conversation_request()).unwrap();
        assert!(conversation.contains("concise prose"));
        assert!(conversation.contains("Current-invocation turns:"));
        assert!(conversation.contains("Scoped context:"));
        assert!(conversation.contains("Do not return an operation plan"));

        let mut request = request();
        request.turns = conversation_request().turns;
        let proposal = proposal_prompt(&request).unwrap();
        assert!(proposal.contains("Return only a Software Schematic operation plan"));
        assert!(proposal.contains("Current-invocation turns:"));
        assert!(proposal.contains("Current persisted context:"));
        assert!(proposal.contains("unrelated multi-element changes require diagram scope"));
        assert!(!proposal.contains("OPENAI_API_KEY"));
    }

    #[test]
    fn element_scope_rejects_peers_and_allows_anchored_child_compositions() {
        let mut request = request();
        request.snapshot["scope"] = json!("node");
        request.snapshot["primaryElementId"] = json!("Task_1");
        request.snapshot["graph"]["nodes"] = json!([
            {"id":"Task_1","type":"bpmn:Task","name":"sales.Order#work","label":"Work","status":"open"},
            {"id":"Task_2","type":"bpmn:Task","name":"sales.Order#peer","label":"Peer","status":"open"}
        ]);
        let selected = AssistantPlan {
            version: SCHEMA_VERSION.into(),
            request_id: request.request_id.clone(),
            source_revision: "abc".into(),
            summary: "Selected".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![
                json!({"type":"update_node_label","diagramPath":"main.bpmn","nodeId":"Task_1","label":"Focused work"}),
            ],
        };
        validate_plan(&selected, &request).unwrap();
        let mut peer = selected.clone();
        peer.operations[0]["nodeId"] = json!("Task_2");
        assert!(
            validate_plan(&peer, &request)
                .unwrap_err()
                .to_string()
                .contains("diagram-level assistance")
        );

        let child = AssistantPlan {
            version: SCHEMA_VERSION.into(),
            request_id: request.request_id.clone(),
            source_revision: "abc".into(),
            summary: "Create child".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![
                json!({"type":"set_process_reference","diagramPath":"main.bpmn","nodeId":"Task_1","qualifiedName":"sales.FocusedWork"}),
                json!({"type":"create_process","qualifiedName":"sales.FocusedWork"}),
                json!({"type":"add_flow_node","diagramPath":"sales/FocusedWork/main.bpmn","nodeId":"Child_1","bpmnType":"bpmn:Task","name":"sales.FocusedWork#step","label":"Do step"}),
                json!({"type":"replace_node_markdown","diagramPath":"sales/FocusedWork/main.bpmn","nodeId":"Child_1","markdown":"# Do step\n\nComplete the focused step."}),
            ],
        };
        validate_plan(&child, &request).unwrap();
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
        let mut out_of_scope = plan.clone();
        out_of_scope.operations[0]["diagramPath"] = json!("other/main.bpmn");
        assert!(
            validate_plan(&out_of_scope, &request())
                .unwrap_err()
                .to_string()
                .contains("outside this proposal scope")
        );
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
        assert_eq!(variants.len(), 20);
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
        let registry = operation_registry();
        assert_eq!(registry["version"], SCHEMA_VERSION);
        assert_eq!(
            registry["operations"].as_object().unwrap().len(),
            variants.len()
        );
        assert!(
            registry["operations"].as_object().unwrap().values().all(
                |capability| capability["preview"] == true
                    && capability["apply"] == true
                    && capability["undo"] == true
                    && capability["rollback"] == true
                    && capability["schema"].is_object()
                    && capability["executor"].is_string()
                    && capability["reversal"].is_string()
            )
        );
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
    fn bpmn_external_participant_and_message_flow_are_validated() {
        let request = AssistantRequest {
            request_id: "request-s3".into(),
            prompt: "Add an S3 pool".into(),
            snapshot: json!({
                "version":"2.0", "diagramKind":"bpmn", "diagramPath":"main.bpmn", "sourceRevision":"abc",
                "graph":{"nodes":[{"id":"Task_1","type":"bpmn:ServiceTask","name":"sales.Order#saveData","status":"open"}],"flows":[]}
            }),
            turns: vec![],
        };
        let plan = AssistantPlan {
            version: SCHEMA_VERSION.into(),
            request_id: "request-s3".into(),
            source_revision: "abc".into(),
            summary: "Model S3 collaboration".into(),
            assumptions: vec![],
            warnings: vec![],
            operations: vec![
                json!({"type":"add_participant","diagramPath":"main.bpmn","participantId":"Participant_S3","label":"Amazon S3","x":500,"y":420}),
                json!({"type":"connect_message_flow","diagramPath":"main.bpmn","flowId":"MessageFlow_Save","sourceId":"Task_1","targetId":"Participant_S3","label":"putObject"}),
                json!({"type":"replace_edge_markdown","diagramPath":"main.bpmn","edgeId":"MessageFlow_Save","markdown":"# putObject\n\nProducer: save data\nConsumer: S3\nPayload: object bytes and key."}),
            ],
        };
        validate_plan(&plan, &request).unwrap();
        let mut invalid = plan;
        invalid.operations[1]["targetId"] = json!("Missing");
        assert!(validate_plan(&invalid, &request).is_err());
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
            turns: vec![],
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
                json!({"type":"replace_node_markdown","diagramPath":"cybling/main.cmmn","nodeId":"PlanItem_Birth","markdown":"# Birth design\n\nTrace the business need into its BPMN design."}),
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
        missing_target.operations[2]["targetId"] = json!("Missing");
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
