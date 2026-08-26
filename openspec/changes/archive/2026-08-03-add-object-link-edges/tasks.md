## 1. Data Model and Validation

- [x] 1.1 Add `objectLink` and the registered `sameAs`/`subclassOf` link modifiers to the Data Graph type registry and shared element helpers
- [x] 1.2 Extend document validation and deterministic serialization for typed object links, including endpoint, self-link, duplicate-tuple, and unknown-type checks
- [x] 1.3 Add valid and invalid fixture coverage for typed links, accepted cycles, and rejected malformed links

## 2. Relationship Authoring

- [x] 2.1 Replace the generic object Connect action with the two explicit authoring choices, Relationship and Object link
- [x] 2.2 Implement targetless Relationship completion so dropping on empty canvas creates a scalar edge node at the terminal position with one domain edge
- [x] 2.3 Implement targeted Relationship completion as one command that places a scalar edge node at the object midpoint and creates its label, domain edge, and range edge
- [x] 2.4 Verify undo and redo treat each targetless or targeted relationship creation as one atomic topology change

## 3. Object-Link Editing and Rendering

- [x] 3.1 Implement Object link connection rules that accept only distinct object endpoints and reject duplicate source-target-type tuples
- [x] 3.2 Render object links as direct outline-docked dashed connections with accessible modifier labels and no intermediate node
- [x] 3.3 Add the anchored object-link modifier chooser and command handling for undoable `sameAs`/`subclassOf` changes
- [x] 3.4 Support selection, deletion, import reconstruction, export, undo, and redo for object links

## 4. Linked Connection Resolution

- [x] 4.1 Implement a pure effective-connection resolver that follows applicable object links from source to target without mutating stored topology
- [x] 4.2 Add visited-object cycle protection, edge-node identity deduplication, and deterministic result ordering
- [x] 4.3 Integrate effective connections with the model consumer that needs inherited object relationships while keeping export limited to stored topology

## 5. Verification and Documentation

- [x] 5.1 Add modeler interaction tests for the two connection choices, terminal edge-node placement, midpoint placement, invalid endpoints, and atomic undo/redo
- [x] 5.2 Add document tests for typed link round trips, malformed-link rejection, non-materialized inheritance, cycles, and diamond deduplication
- [x] 5.3 Update the README and fixture documentation to explain Relationship versus Object link and the initial link modifiers
- [x] 5.4 Run the web-editor test, lint, and production-build checks and rebuild the bundled macOS web assets
- [x] 5.5 Run the Swift package tests to verify the rebuilt assets and native integration remain valid
