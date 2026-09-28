## MODIFIED Requirements

### Requirement: Markdown ownership and provenance
The system SHALL read `main.md` for each loaded diagram and `docs/<element-id>.md` for each loaded node or edge when present as authored logical documentation. For a loaded edge or event, it SHALL also read generated `docs/<element-id>-contract.md` when present as implementation-contract evidence, retain its plan ID and semantic version metadata, and relate it to the same owning entity without replacing or merging the authored logical Markdown. The graph SHALL associate each document and content hash with its role and owner and SHALL retain confined file provenance only in the snapshot source map.

#### Scenario: Diagram and element documentation are present
- **WHEN** a loaded diagram has `main.md` and one of its elements has an ID-bound Markdown file
- **THEN** both documents are retrievable from their respective owners with distinct source paths and hashes

#### Scenario: Logical and implementation contracts are present
- **WHEN** a loaded edge or event has both `<element-id>.md` and `<element-id>-contract.md`
- **THEN** graph queries can retrieve both separately as logical intent and actual implementation evidence for the same entity

#### Scenario: Implemented contract differs
- **WHEN** generated contract content differs from logical documentation
- **THEN** graph loading succeeds, preserves both documents, and exposes a contract-drift indicator without treating the generated file as new design authority

#### Scenario: Optional Markdown is absent
- **WHEN** a loaded entity has neither logical nor generated Markdown
- **THEN** graph loading succeeds and reports that entity without documentation chunks

#### Scenario: Generated contract targets an unsupported owner
- **WHEN** a `-contract.md` file does not resolve to a reachable edge or event
- **THEN** the loader excludes it from implementation evidence and reports a bounded diagnostic
