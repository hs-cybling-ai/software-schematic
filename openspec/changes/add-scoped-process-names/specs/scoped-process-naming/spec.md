## ADDED Requirements

### Requirement: Distinct ID, Name, and Label
The system SHALL model BPMN instance ID, architectural Name, and display Label as distinct values. ID SHALL identify the diagram instance, Name SHALL identify its architectural symbol within a scope, and Label SHALL remain unrestricted presentation text. Editing a Label SHALL NOT change a Name, qualified symbol, reference, or composition path.

#### Scenario: Label changes independently
- **WHEN** a user changes `Select and outfit subscription` to another display Label
- **THEN** the node retains its ID, Name, qualified symbol, and composition reference

### Requirement: Java-style scoped symbols
The system SHALL validate pool package Names as dot-separated lowerCamelCase segments, reusable process Names as UpperCamelCase identifiers, and ordinary task or event Names as lowerCamelCase identifiers. It SHALL resolve a process as `package.Process` and an ordinary member as `package.Process#member`.

#### Scenario: Same member function exists in different processes
- **WHEN** two processes each contain a member named `init`
- **THEN** they resolve independently, such as `cybling.CyblingLifecycle#init` and `subscription.SubscriptionLifecycle#init`

#### Scenario: Invalid naming convention is entered
- **WHEN** a package, process, or member Name violates its required identifier form
- **THEN** the editor rejects it with an example of the expected Java-style form

### Requirement: Pool-provided package scope
A named pool SHALL provide package scope for contained reusable process nodes. A child process diagram SHALL retain its qualified process Name as owning scope for its ordinary tasks and events, so those members need only local Names. Moving a named node into a scope that would change its resolved symbol SHALL require an explicit decision to retain the old target or rebind to the new scope.

#### Scenario: Identically named processes occupy different pools
- **WHEN** pools named `cybling` and `subscription` each contain a process named `Init`
- **THEN** the processes resolve as distinct symbols `cybling.Init` and `subscription.Init`

#### Scenario: Ordinary task inherits process scope
- **WHEN** task `processSubscription` is inside process `cybling.subscription.SelectAndOutfit`
- **THEN** its derived qualified symbol is `cybling.subscription.SelectAndOutfit#processSubscription`

#### Scenario: Named node moves across scopes
- **WHEN** a user moves a named node to a pool or process with a different owning scope
- **THEN** SSW does not silently change its target and asks whether to retain or rebind the qualified symbol

### Requirement: Shared process resolution
Multiple nodes SHALL resolve to the same reusable composition when their qualified process Names are equal, regardless of their BPMN IDs or Labels. Ordinary task and event members SHALL remain owned by their parent process and SHALL NOT create composition folders.

#### Scenario: Two nodes call one process
- **WHEN** two nodes have different IDs and Labels but resolve to `cybling.subscription.SelectAndOutfit`
- **THEN** both open the same process composition

#### Scenario: Ordinary member is named
- **WHEN** a task receives local Name `outfitCybling`
- **THEN** SSW derives its member symbol but creates no child diagram or folder

### Requirement: Explicit scoped rename semantics
Changing an ordinary member Name SHALL update only that member's architectural symbol. Changing a reusable process Name or its package SHALL be an explicit refactor that updates the definition, all references, and the deterministic composition location. The root navigation diagram SHALL NOT be renamed or moved.

#### Scenario: Member is renamed
- **WHEN** `#processSubscription` becomes `#activateSubscription`
- **THEN** its parent process folder remains unchanged and the member receives the new derived symbol

#### Scenario: Process is renamed
- **WHEN** `cybling.subscription.SelectAndOutfit` becomes `cybling.subscription.ConfigureSubscription`
- **THEN** SSW performs one reviewed process refactor rather than a local text edit
