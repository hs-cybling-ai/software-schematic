## MODIFIED Requirements

### Requirement: Current workflow persistence
The runtime SHALL retain the current bounded interview summary, current proposal, latest publication receipt, and a bounded history of versioned build plans with separate progress and evidence records needed to resume work. Build plans SHALL be stored beneath `.ss/workflows`, validated against the current project and protocol on load, and excluded from the authoritative schematic source manifest. The runtime SHALL NOT retain credentials, full transcripts, copied requirement corpora, or executable orchestration scripts.

#### Scenario: UI is opened after an AI proposal
- **WHEN** a current proposal is waiting and the developer opens `./ssw`
- **THEN** the UI resumes that proposal on its affected diagram

#### Scenario: Build host starts after Play
- **WHEN** one or more open plans were created before Codex or Claude starts
- **THEN** the runtime restores their scope labels, semantic versions, lifecycle states, work progress, and bounded evidence

#### Scenario: Persisted plan belongs to another project
- **WHEN** a plan's project identity does not match the initialized project
- **THEN** the runtime rejects that plan without exposing it as executable work

#### Scenario: Legacy Play request exists
- **WHEN** an initialized project contains a compatible legacy `.ss/workflows/play.json`
- **THEN** the runtime migrates or presents it through a bounded compatibility path without modifying authored diagrams or Markdown

