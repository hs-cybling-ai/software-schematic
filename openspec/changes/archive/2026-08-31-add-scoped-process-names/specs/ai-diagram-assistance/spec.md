## MODIFIED Requirements

### Requirement: Naming-aware assistant context
Every normalized diagram element in assistant context SHALL include ID, Type, Label, the single complete Name, Implementation Status, and Documentation when included by scope. Context SHALL NOT contain separate local Name, qualified symbol, composition path, or Documentation-path fields.

#### Scenario: Node request is submitted
- **WHEN** the user invokes the assistant for a node
- **THEN** the provider can distinguish its ID, Type, Label, Name, Implementation Status, and Documentation

### Requirement: Name-based assistant operations
Assistant operations SHALL accept complete process or member Names and SHALL NOT accept a separate composition path or derived-qualified-name mutation. Existing ID, Label, and Name values SHALL be preserved unless an explicit operation changes them.

#### Scenario: Provider creates a reusable process
- **WHEN** a provider proposes process Name `sales.checkout.PlaceOrder`
- **THEN** SSW derives the composition folder from that Name

#### Scenario: Provider supplies a path
- **WHEN** a provider attempts to choose a composition or Documentation path
- **THEN** validation rejects the proposal
