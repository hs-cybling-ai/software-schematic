## ADDED Requirements

### Requirement: Content-addressed incremental refresh
Graph refresh SHALL compare strong durable content revisions against the active source manifest and rebuild only affected parsed fragments, annotations, embeddings, relationships, and indexes when the dependency impact is safely bounded. Modification times SHALL NOT determine correctness. Unsupported changes, rename or delete ambiguity, reachability changes, or failed incremental validation SHALL use the complete rebuild path before publication.

#### Scenario: Markdown body changes for one owner
- **WHEN** one connected Markdown document has a new content revision and its ownership remains valid
- **THEN** refresh replaces that owner's annotation fragments and dependent search entries while reusing unaffected graph material

#### Scenario: Timestamp changes without content change
- **WHEN** a document modification time changes but its strong content revision is unchanged
- **THEN** refresh treats the document as unchanged and does not rebuild its graph fragment

#### Scenario: Composition reachability changes
- **WHEN** a diagram edit changes composition reachability and bounded dependency invalidation cannot be proven complete
- **THEN** refresh uses the complete builder and atomically publishes only the validated result

### Requirement: Unified change ingestion
Durable changes produced by the web editor or an approved agent proposal SHALL enter the same project-scoped coalescing refresh coordinator and SHALL produce the same status and publication semantics. The coordinator SHALL guarantee eventual processing of the newest accepted content revision for every changed path while allowing superseded intermediate revisions to be skipped.

#### Scenario: UI and skill save the same document in sequence
- **WHEN** two conditionally accepted saves produce successive durable revisions before graph processing completes
- **THEN** the coordinator may coalesce the first but eventually publishes a snapshot containing the second

#### Scenario: A conditional write conflicts
- **WHEN** an editor or skill write is rejected because its expected revision is stale
- **THEN** no refresh is scheduled for the rejected content

