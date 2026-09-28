use crate::Result;
use std::{fs, path::Path};

const LEGACY_DESIGN_SKILL: &str = r#"---
name: design
description: Deeply interview the developer and iteratively shape the live software model before Play.
---
Treat the diagram as the product contract and the conversation as a way to discover it. Verify the project, resume the current interview, and inspect the current diagram before asking questions.

Begin every invocation with `get_delivery_capabilities`; require `guidedDesignWorkspace`, then call `open_design_workspace` and verify its project identity. The project MCP adapter starts or reuses SSW, so never make the developer manage the daemon. If workspace opening fails, give the returned `./ssw` or `ssw.cmd` fallback. Do not submit a proposal until the current workspace context is available.

Orient before interviewing. Resume a non-terminal proposal first. Otherwise prefer the fresh browser selection, briefly reconcile it with a different durable interview target, then resume the interview when appropriate. If no target is established, offer human-readable choices to refine the root business model, resume a returned process, or design a new process. Narrow a long candidate list from the developer's natural-language goal. Never ask the developer for diagram paths, raw IDs, raw XML, or prior BPMN/CMMN expertise.

For a new process, first establish the desired outcome, actors, and system boundary. Then use supported typed proposals to add or refine the business-need anchor and its process composition incrementally. Do not invent a file path, diagram notation, or fallback Name for the developer.

Interview deeply. Establish the user goal, actors, system boundary, happy path, meaningful alternatives, failure and recovery behavior, important data, states and invariants, external integrations, trust boundaries, operational constraints, and what is explicitly out of scope. Ask focused questions when an answer changes structure or implementation. Do not silently invent material behavior.

Use the existing interview decisions and unresolved items as an adaptive completeness record. Track consequential coverage of goal/outcome, actors, boundary, happy path, alternatives, failure/recovery, data/state, integrations, trust, operations, exclusions, documentation, and implementation statuses. A topic may be not applicable when the developer confirms it. This is guidance, not a fixed questionnaire: ask one focused structural question at a time and explain material gaps before calling the model complete.

Visualize early and often. As soon as one coherent slice is settled, save the interview decisions and submit a small typed proposal to the live browser. Never batch the whole design into one giant proposal. Wait for approval or rejection, then re-read the current diagram so direct visual edits become input to the next interview turn. Keep Labels concise; put behavior and contracts in the narrowest diagram, activity, event, or edge Markdown owner.

The browser alone approves and applies diagram operations. Never approve your own proposal or emit raw XML. Continue until no known material gap remains and the developer explicitly confirms the model feels complete. Summarize the modeled scope, ask them to click Play in the diagram UI, and wait with `wait_for_build_request`; do not begin implementation before Play.
"#;

const LEGACY_GRAPH_SKILL: &str = r#"---
name: graph
description: Publish the current saved diagram so the AI can build from exactly what the developer sees.
---
Read the current source manifest and submit only saved diagram or Markdown paths whose content revision changed. Wait for graph publication before Play can hand the model to build. If publication fails, report the source problem plainly and preserve the prior graph. Never create a second specification.
"#;

const LEGACY_BUILD_SKILL: &str = r#"---
name: build
description: Respond to Play by building code from the current software model.
---
Read or wait for the current Play request. Verify its diagram and graph revisions, then call `start_build`. Resolve the selected element or active diagram to eligible `new` or `modify` model elements and obtain implementation context for that exact graph revision. Ask only when the requested scope is genuinely ambiguous.

Implement the code directly, run repository-relevant checks, and keep evidence concise. Before material edits and completion, verify that the model has not changed. If it has, stop and ask the developer to click Play again. Finish with `complete_build`, reporting changed paths and check results; report `failed` with a useful diagnostic when work cannot complete.
"#;

pub const DESIGN_SKILL: &str = r#"---
name: design
description: Deeply interview the developer and iteratively shape the live software model before planning implementation.
---
Treat the diagram as the product contract and the conversation as a way to discover it. Begin with `get_delivery_capabilities`, require `guidedDesignWorkspace`, then call `open_design_workspace` and verify project identity.

Resume a non-terminal proposal first. Otherwise prefer the fresh browser selection, reconcile it briefly with any durable interview target, and resume the interview when appropriate. If no target is established, offer human-readable choices to refine the root business model, resume a returned process, or design a new process. Never ask the developer for diagram paths, raw IDs, raw XML, or prior BPMN/CMMN expertise.

For a new process, establish the desired outcome, actors, and system boundary before proposing structure. Interview deeply about the happy path, meaningful alternatives, failure/recovery, data and invariants, integrations, trust, operations, exclusions, documentation, and implementation statuses. Ask one focused structural question at a time and do not silently invent material behavior.

Visualize settled slices with small typed proposals. Wait for approval or rejection, then re-read the current diagram so direct edits inform the next turn. The browser alone approves and applies diagram operations. Never approve your own proposal or emit raw XML.

Continue until no material gap remains and the developer explicitly confirms the model feels complete. Then summarize it and offer two equivalent next steps: click Play for the current browser selection, or invoke `plan` and describe the diagram node or label naturally. Both paths create the same versioned build plan; design itself never starts a build.
"#;

pub const PLAN_SKILL: &str = r#"---
name: plan
description: Create a versioned, selected-boundary build plan from a natural-language diagram node or label.
---
Begin with `get_delivery_capabilities`; require `versionedBuildPlans` and `naturalLanguagePlanScope`, verify project identity, and use the current published graph and source-manifest revisions. Refuse to plan while the browser reports unsaved documents.

Pass the developer's natural-language target to `create_build_plan`. If it returns candidates, show only the short numbered labels and composition breadcrumbs, ask for a number, then repeat with that candidate's `selectedScopeRef`. Never ask for a URN, path, or XML ID.

The selected boundary is authoritative. A pool means its service descendants and downward compositions. A node means that node, its contained items, attached events, incident edges, one connected layer, and eligible downward compositions. Never expand upward. Only `new` and `modify` are work; `open` and `locked` remain context or exclusions.

If the server says the selected scope changed, retain the returned latest version as `expectedPriorVersion`, ask whether this is a Major, Minor, or Fix version, and retry with that semantic bump. An unchanged incomplete or failed plan resumes its existing version. Report only the concise scope label, semantic version, and state. Do not generate a task-launching script or begin implementation.
"#;

pub const BUILD_SKILL: &str = r#"---
name: build
description: Build one versioned schematic plan sequentially, using every other in-scope item only as context.
---
Begin with `get_delivery_capabilities` and require `versionedBuildPlans` and `sequentialBuildClaims`. With no selector, call `list_build_plans`, show the open plans as a short numbered list, and ask for a number. A number is valid only for that displayed list. Also accept an exact plan ID or exact `scope@version`; never guess from a bare version.

Retrieve the selected plan, then loop strictly one item at a time: `claim_next_build_item`, `get_build_item_context`, implement only the focused item and necessary in-scope physical effects, run relevant checks, and call `record_build_item_result` with changed paths, physical effects, checks, and a concise diagnostic. Other plan items, connected edges/events, and completed evidence are context—not permission to implement unrelated features.

It is acceptable to add necessary shared wiring, configuration, schemas, migrations, and tests, or to make a logical edge/event contract workable. Do not add out-of-scope replay, alternate flows, speculative stubs, or duplicate services. For plan-declared edge/event outputs, publish the implementation contract through the claim-bound contract tool; never choose a filesystem path yourself.

Before each material edit, verify the claim context still grants the same graph revision. Stop on stale authority. After all items complete, run integration checks and call `complete_build_plan`. Never create an orchestration script or start separate tasks for nodes: the durable compare-and-swap queue is the orchestrator.
"#;

fn write_managed(path: &Path, content: &str, legacy: &[&str]) -> Result<()> {
    if path.is_file() {
        let current = fs::read_to_string(path)?;
        if current != content && !legacy.contains(&current.as_str()) {
            eprintln!(
                "preserved locally modified managed adapter {}",
                path.display()
            );
            return Ok(());
        }
    }
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, content)?;
    Ok(())
}

fn remove_legacy_graph(path: &Path) -> Result<()> {
    if path.is_file() && fs::read_to_string(path)? == LEGACY_GRAPH_SKILL {
        fs::remove_file(path)?;
        if let Some(parent) = path.parent()
            && parent.file_name().and_then(|value| value.to_str()) == Some("graph")
        {
            let _ = fs::remove_dir(parent);
        }
    } else if path.exists() {
        eprintln!(
            "preserved locally modified legacy graph adapter {}",
            path.display()
        );
    }
    Ok(())
}

pub fn install_skill_adapters(project: &Path) -> Result<()> {
    for (root, filename) in [
        (project.join(".codex/skills"), "SKILL.md"),
        (project.join(".claude/commands"), "md"),
    ] {
        for (name, content, legacy) in [
            ("design", DESIGN_SKILL, &[LEGACY_DESIGN_SKILL][..]),
            ("plan", PLAN_SKILL, &[][..]),
            ("build", BUILD_SKILL, &[LEGACY_BUILD_SKILL][..]),
        ] {
            let path = if filename == "SKILL.md" {
                root.join(name).join(filename)
            } else {
                root.join(format!("{name}.{filename}"))
            };
            write_managed(&path, content, legacy)?;
        }
        let graph_path = if filename == "SKILL.md" {
            root.join("graph").join(filename)
        } else {
            root.join(format!("graph.{filename}"))
        };
        remove_legacy_graph(&graph_path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_the_same_small_workflow_for_codex_and_claude() {
        let directory = tempfile::tempdir().unwrap();
        install_skill_adapters(directory.path()).unwrap();

        for (name, expected) in [
            ("design", DESIGN_SKILL),
            ("plan", PLAN_SKILL),
            ("build", BUILD_SKILL),
        ] {
            assert_eq!(
                fs::read_to_string(
                    directory
                        .path()
                        .join(".codex/skills")
                        .join(name)
                        .join("SKILL.md")
                )
                .unwrap(),
                expected
            );
            assert_eq!(
                fs::read_to_string(
                    directory
                        .path()
                        .join(".claude/commands")
                        .join(format!("{name}.md"))
                )
                .unwrap(),
                expected
            );
        }
    }

    #[test]
    fn design_skill_is_guided_visual_and_completion_aware() {
        for required in [
            "guidedDesignWorkspace",
            "open_design_workspace",
            "Resume a non-terminal proposal first",
            "fresh browser selection",
            "refine the root business model",
            "design a new process",
            "desired outcome, actors, and system boundary",
            "failure/recovery",
            "implementation statuses",
            "re-read the current diagram",
            "explicitly confirms",
            "click Play",
            "invoke `plan`",
        ] {
            assert!(DESIGN_SKILL.contains(required), "missing {required}");
        }
        assert!(DESIGN_SKILL.contains(
            "Never ask the developer for diagram paths, raw IDs, raw XML, or prior BPMN/CMMN expertise"
        ));
        assert!(DESIGN_SKILL.contains("Never approve your own proposal"));
        assert!(DESIGN_SKILL.contains("design itself never starts a build"));
    }

    #[test]
    fn plan_and_build_share_the_durable_queue_without_scripts() {
        for required in [
            "create_build_plan",
            "selectedScopeRef",
            "Major, Minor, or Fix",
            "new` and `modify",
        ] {
            assert!(PLAN_SKILL.contains(required), "missing {required}");
        }
        for required in [
            "list_build_plans",
            "claim_next_build_item",
            "get_build_item_context",
            "record_build_item_result",
            "complete_build_plan",
            "one item at a time",
        ] {
            assert!(BUILD_SKILL.contains(required), "missing {required}");
        }
        assert!(PLAN_SKILL.contains("Do not generate a task-launching script"));
        assert!(BUILD_SKILL.contains("Never create an orchestration script"));
    }

    #[test]
    fn upgrade_removes_only_an_unmodified_legacy_graph_adapter() {
        let directory = tempfile::tempdir().unwrap();
        let legacy = directory.path().join(".codex/skills/graph/SKILL.md");
        fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        fs::write(&legacy, LEGACY_GRAPH_SKILL).unwrap();
        let modified = directory.path().join(".claude/commands/graph.md");
        fs::create_dir_all(modified.parent().unwrap()).unwrap();
        fs::write(&modified, "local graph instructions").unwrap();
        install_skill_adapters(directory.path()).unwrap();
        assert!(!legacy.exists());
        assert_eq!(
            fs::read_to_string(modified).unwrap(),
            "local graph instructions"
        );
    }
}
