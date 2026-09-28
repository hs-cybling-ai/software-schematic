## MODIFIED Requirements

### Requirement: Play-to-build transition
The diagram UI SHALL expose a prominent Play action. Play SHALL flush all pending relevant diagram and Markdown edits, wait for their graph publication, resolve the active diagram selection to a bounded build scope, and create or resume an immutable semantic-versioned plan for the exact source-manifest and graph revisions. The UI SHALL allow an explicit major, minor, or fix/patch advance when the selected scope has a materially changed prior plan, SHALL publish the created scope label and version after success, and SHALL NOT itself edit source code.

#### Scenario: Developer clicks Play
- **WHEN** the current selected scope is saved and published
- **THEN** the runtime creates or resumes its versioned build plan and the waiting AI can discover the authorized work

#### Scenario: Developer clicks Play for a new scope
- **WHEN** current relevant edits are saved and published and the selected scope has no prior plan
- **THEN** the runtime creates version `1.0.0`, the waiting AI can discover it, and the UI displays the scope and version

#### Scenario: Developer versions a changed scope
- **WHEN** a prior version exists and the selected graph contract has materially changed
- **THEN** Play requires a major, minor, or fix/patch choice and publishes the resulting version only after the new plan is durably created

#### Scenario: Diagram changes after Play
- **WHEN** an unrelated authoritative model revision advances before the build completes
- **THEN** the prior plan becomes stale and the UI directs the developer to review the design and click Play again

#### Scenario: Generated completion contracts publish
- **WHEN** the build publishes only the implementation-contract outputs declared by its plan
- **THEN** the UI records the resulting completion graph revision without treating the plan as stale from its own expected output

## ADDED Requirements

### Requirement: Discoverable plan status in the editor
The diagram UI SHALL show the selected scope's current plan version and lifecycle state after Play and after subsequent runtime updates. The display SHALL distinguish ready, building, complete, failed, and stale plans and SHALL remain a status surface rather than an implementation executor.

#### Scenario: Developer returns while another host builds
- **WHEN** Codex or Claude advances a selected plan
- **THEN** the editor displays that plan's current semantic version and lifecycle state

### Requirement: Visible implementation-contract drift
When an edge or event has both logical Markdown and generated implementation-contract Markdown with different normalized content, the diagram UI SHALL indicate that implementation drift exists and SHALL let the developer inspect both documents without overwriting either one. Drift alone SHALL NOT make an otherwise authorized build fail or silently change design intent.

#### Scenario: Built contract differs from logical design
- **WHEN** the current graph reports different logical and implementation contracts for a selected edge or event
- **THEN** the editor shows a drift warning and distinct logical and implemented contract views

#### Scenario: Divergence was required for working code
- **WHEN** the generated contract remains inside plan scope but records a workable implementation detail absent from or different from logical design
- **THEN** the completed plan remains complete while the UI preserves the drift warning for later design review
