## ADDED Requirements

### Requirement: Plan-declared implementation-contract publication
Generated edge and event implementation contracts accepted through an active build plan SHALL enter the same durable conditional-save, embedding, refresh, and atomic-publication pipeline as connected authored Markdown. The plan SHALL retain its immutable base graph revision and record the graph revision produced by its own declared generated-contract outputs as its completion revision; only unrelated authoritative changes SHALL stale the plan during this transition.

#### Scenario: Build publishes one declared edge contract
- **WHEN** a building plan conditionally saves an authorized `docs/<edge-id>-contract.md`
- **THEN** the coordinator publishes it as implementation evidence and the plan records the resulting graph revision without becoming stale from that save

#### Scenario: Unrelated diagram changes during contract publication
- **WHEN** another save changes authoritative content outside the plan's declared contract-output set before completion
- **THEN** the plan becomes stale and cannot complete against the substituted graph revision

#### Scenario: Generated contract save conflicts
- **WHEN** an existing generated contract changed after the build captured its expected revision
- **THEN** the conditional save is rejected, no caller content is published, and the plan remains resumable with a conflict diagnostic

