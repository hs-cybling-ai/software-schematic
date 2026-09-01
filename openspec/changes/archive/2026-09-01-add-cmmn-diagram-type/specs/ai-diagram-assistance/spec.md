## ADDED Requirements

### Requirement: CMMN assistant context
For a CMMN diagram- or node-scoped magic request, the application SHALL construct the existing bounded context shape using normalized CMMN elements and SHALL identify the CMMN content as business-need context. Each included element SHALL provide ID, CMMN Type, Label, complete Name, Implementation Status, and Documentation when in scope, and the request SHALL identify the owning package and `.cmmn` diagram without exposing an absolute project path.

#### Scenario: CMMN diagram request is prepared
- **WHEN** a user submits a prompt from the magic action on `cybling/sdk/main.cmmn`
- **THEN** the provider receives a bounded normalized CMMN snapshot and package identity rather than raw filesystem authority

### Requirement: Bounded CMMN operations
The assistant operation schema SHALL support only registered CMMN documentation-tool operations: changing an eligible Label or complete Name, replacing diagram or element Markdown, adding a supported CMMN plan item, connecting supported elements, and assigning or opening a Process Task BPMN-process link. It SHALL reject raw XML replacement, arbitrary paths, project-structure inference, executable content, runtime case behavior, and unsupported CMMN semantics.

#### Scenario: Assistant links a Process Task
- **WHEN** a valid proposal assigns BPMN process Name `cybling.sdk.Birth` to a CMMN Process Task
- **THEN** the preview identifies the derived BPMN composition and the provider supplies no filesystem path

#### Scenario: One need anchors a descendant package design
- **WHEN** a proposal links a Process Task in CMMN package `cybling` to BPMN process `cybling.sdk.Birth`
- **THEN** validation accepts the complete BPMN Name without requiring `cybling.sdk/main.cmmn`

#### Scenario: Assistant proposes enterprise behavior
- **WHEN** a proposal requests runtime case execution, compensation, deployment, permissions, or arbitrary expressions
- **THEN** validation rejects the operation before preview or mutation

### Requirement: CMMN proposal application
Approved CMMN proposals SHALL use supported `cmmn-js` modeling services and the existing confined file operations, automatic persistence, before-state capture, rollback, and assistant-level revert behavior. A failed mixed CMMN, BPMN, and Markdown proposal SHALL NOT report partial success.

#### Scenario: Approved CMMN documentation proposal succeeds
- **WHEN** an approved proposal adds a supported CMMN plan item and updates its Documentation successfully
- **THEN** the CMMN XML and Markdown save through their existing paths and the complete change can be reverted
