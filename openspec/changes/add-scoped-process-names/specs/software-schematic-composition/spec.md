## MODIFIED Requirements

### Requirement: Folder-based base diagrams
The system SHALL represent each named process composition as a deterministic folder derived from its qualified Name, with package segments as nested directories and the UpperCamelCase process Name as the final directory. Each composition folder SHALL contain blank `main.bpmn` as its base diagram and `main.md` as its diagram documentation. `schematics/main.bpmn` and `schematics/main.md` SHALL remain the immutable root navigation composition.

#### Scenario: Root composition opens
- **WHEN** the initialized application starts
- **THEN** it opens `schematics/main.bpmn` and associates `schematics/main.md` with the diagram

#### Scenario: Named composition is created
- **WHEN** SSW creates process `cybling.subscription.SelectAndOutfit`
- **THEN** it creates `schematics/cybling/subscription/SelectAndOutfit/main.bpmn` as a blank diagram and the adjacent `main.md`

### Requirement: Element documentation convention
The system SHALL store documentation for a BPMN element at `docs/<element-id>.md` within the composition folder containing that element's base diagram. Relative Markdown links in both diagram and element documentation SHALL resolve from that owning composition folder.

#### Scenario: Document an element for the first time
- **WHEN** the user edits documentation for an element that has no Markdown file
- **THEN** the server creates the composition folder's `docs/` directory and writes `<element-id>.md`

#### Scenario: Composition documentation references a local asset
- **WHEN** diagram or node Markdown links to `./assets/model.png`
- **THEN** SSW resolves the link from the folder containing the owning `main.bpmn`

## ADDED Requirements

### Requirement: Name-based composition navigation and reuse
The application SHALL derive composition locations from qualified process Names and SHALL open or create the derived blank composition when a reusable process node is activated. Users and assistant providers SHALL NOT supply an independent composition path. Multiple nodes resolving to the same qualified process Name SHALL share one canonical diagram and documentation unit.

#### Scenario: User opens a named process
- **WHEN** a node resolves to `cybling.subscription.SelectAndOutfit`
- **THEN** SSW opens or creates `cybling/subscription/SelectAndOutfit/main.bpmn` in one canonical tab

#### Scenario: Two flows reuse a process
- **WHEN** two nodes resolve to the same qualified process Name
- **THEN** both focus the same composition tab and documentation files

### Requirement: Atomic composition rename and move
The application SHALL preflight and atomically rename a process definition, move its complete composition folder, update every stored reference, and rekey affected tabs, breadcrumbs, documentation targets, save queues, and revisions. It SHALL reject collisions including case-insensitive collisions and SHALL restore the complete prior state on any failure. Relative links within the moved composition SHALL remain unchanged.

#### Scenario: Process rename succeeds
- **WHEN** the user renames `cybling.subscription.SelectAndOutfit` to `cybling.subscription.ConfigureSubscription`
- **THEN** the full folder moves to the derived destination, every reference resolves to the new Name, open UI state follows it, and internal Markdown links remain valid without rewriting

#### Scenario: Rename target collides
- **WHEN** the derived destination or qualified Name already belongs to another process
- **THEN** SSW rejects the rename before moving content or changing references

#### Scenario: Rename fails after staging
- **WHEN** any folder, reference, or open-state update cannot be committed
- **THEN** SSW restores the original folder, references, and browser state and reports no partial success

## REMOVED Requirements

### Requirement: External subprocess reference
**Reason**: A user-managed external folder duplicates architectural Name and allows identity/path drift.
**Migration**: Resolve legacy `calledElement` values into qualified Names and retain them only as compatibility serialization where required.

### Requirement: External subprocess navigation and creation
**Reason**: Navigation and creation now derive from qualified process Name instead of an external path.
**Migration**: Existing confined folders remain accessible during migration and are moved only through an explicit reviewed refactor.

### Requirement: Reusable process references
**Reason**: Reuse is now specified by equal qualified process Names rather than equal folder strings.
**Migration**: Nodes that shared a legacy `calledElement` are assigned the same migrated qualified Name.

### Requirement: Pool and lane composition folders
**Reason**: Pool IDs and lane IDs are instance identifiers and no longer define architectural composition paths.
**Migration**: Pool Names provide package scope; legacy ID-based folders are preserved until assigned an unambiguous process Name.
