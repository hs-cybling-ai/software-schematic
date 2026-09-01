## MODIFIED Requirements

### Requirement: Name-based composition folders
The system SHALL map process Name `package.Process` to `schematics/package/Process/`, containing `main.bpmn` and `main.md`. Users and providers SHALL supply the Name and SHALL NOT supply a separate folder path.

#### Scenario: Named process opens
- **WHEN** an element has Name `sales.checkout.PlaceOrder`
- **THEN** SSW opens or creates `schematics/sales/checkout/PlaceOrder/main.bpmn`

### Requirement: ID-bound element documentation
The system SHALL store node and edge Documentation at `docs/<element-id>.md` within the composition. It SHALL use `main.md` for process Documentation and SHALL NOT expose a Documentation-path metadata field.

#### Scenario: Node documentation is created
- **WHEN** node `Activity_42` has Name `sales.checkout.PlaceOrder#validateCart`
- **THEN** its Documentation is stored as `schematics/sales/checkout/PlaceOrder/docs/Activity_42.md`

#### Scenario: Edge documentation is created
- **WHEN** edge `Flow_7` has Name `sales.checkout.PlaceOrder#paymentApproved`
- **THEN** its Documentation is stored as `schematics/sales/checkout/PlaceOrder/docs/Flow_7.md`

#### Scenario: Two call sites share one implementation
- **WHEN** two call activities use the same process Name from different owning diagrams
- **THEN** each caller keeps separate ID-bound Documentation in its owning composition and both open the same subprocess `main.md`
