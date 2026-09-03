## ADDED Requirements

### Requirement: Document and graph save status
The web application SHALL distinguish durable document persistence from derived graph refresh. After a successful diagram or connected Markdown save, it SHALL report whether the running project MCP published a new revision, was not running, or retained its prior revision because refresh failed. A refresh failure SHALL NOT be represented as a document-save failure and SHALL NOT roll back the authored file.

#### Scenario: Save and graph refresh succeed
- **WHEN** a document saves and the running MCP publishes the replacement graph
- **THEN** the web application reports that the document is saved and identifies the updated graph revision

#### Scenario: Save succeeds without MCP
- **WHEN** the document saves but no matching MCP process is running
- **THEN** the web application reports the document as saved and the graph as not refreshed

#### Scenario: Save succeeds but graph rebuild fails
- **WHEN** the document saves but the MCP retains its prior graph after a rebuild diagnostic
- **THEN** the web application reports the document as saved, the graph update as failed, and an actionable diagnostic
