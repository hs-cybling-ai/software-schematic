## MODIFIED Requirements

### Requirement: Interview-driven design skill
The `design` skill SHALL begin by opening or resuming the matching project design workspace, verifying project identity and delivery capabilities, and orienting the developer to the current visual model. It SHALL prefer a fresh browser selection or resumable interview target; otherwise it SHALL present bounded choices to refine the root business model, resume an existing named process, or design a new process. For a new process it SHALL begin with the desired outcome, actors, and boundary rather than requiring the developer to supply BPMN/CMMN terminology. It SHALL deeply interview the developer, maintain a bounded adaptive assessment of material design gaps, turn each settled slice into a small typed proposal in the live browser, observe approved and manual diagram changes, and continue until the developer confirms that the model is complete and clicks Play. It SHALL NOT expose workspace credentials, approve its own proposal, or begin implementation before Play.

#### Scenario: Design starts without an open browser
- **WHEN** the developer invokes the `design` skill for an initialized project whose runtime is headless
- **THEN** the skill invokes `open_design_workspace`, directs the developer to the securely opened modeling workspace, and performs no proposal mutation until current workspace context is available

#### Scenario: Active visual context is available
- **WHEN** a fresh browser session identifies an active diagram or selected model element
- **THEN** the skill offers to continue from that context before presenting unrelated process choices

#### Scenario: Existing model needs orientation
- **WHEN** no active selection or resumable interview determines the target and the model contains named processes
- **THEN** the skill presents a bounded, human-readable choice among the root business model, existing process candidates, and a new process without requiring diagram paths or entity IDs

#### Scenario: Developer begins a new process
- **WHEN** the developer chooses to design a new process
- **THEN** the skill first establishes the intended outcome, actors, and system boundary and then proposes the business-need anchor and BPMN composition incrementally through supported operations

#### Scenario: Interview produces a diagram increment
- **WHEN** the developer's answers settle one coherent portion of a process
- **THEN** the skill saves the decisions, submits a focused visual proposal, waits for the browser decision, and continues from the resulting current saved diagram

#### Scenario: Design contract remains ambiguous
- **WHEN** two materially different process interpretations remain plausible
- **THEN** the skill asks a focused follow-up question instead of inventing diagram structure

#### Scenario: Approved design becomes stale
- **WHEN** an affected document changes after proposal generation and before application
- **THEN** the skill refuses the stale proposal and regenerates it from the current diagram

#### Scenario: Material design areas remain unresolved
- **WHEN** the current model still lacks consequential decisions about paths, recovery, data or state, integrations, trust, operations, exclusions, documentation, or implementation status
- **THEN** the skill explains the relevant gaps and continues with focused questions instead of declaring the model complete

#### Scenario: Developer confirms the model is ready
- **WHEN** no material known gap remains and the developer confirms that the model feels complete
- **THEN** the skill summarizes the modeled scope, asks the developer to click Play in the diagram UI, and waits for the resulting build request without starting implementation
