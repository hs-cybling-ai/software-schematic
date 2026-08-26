## 1. Connection Classification and Projection

- [x] 1.1 Add a shared target-type classifier for property, mutation, and ordinary relationship connections in the Data Graph modeler.
- [x] 1.2 Use the classifier in interactive connection creation so property targets do not create edge-label shapes while other supported targets retain applicable labels.
- [x] 1.3 Use the same classifier during document import so persisted connections reconstruct with identical target-driven label behavior.
- [x] 1.4 Update export handling to serialize property-targeting connections safely without a projected edge-label element while retaining compatible semantic label data.

## 2. Edge Rendering

- [x] 2.1 Add distinct rendered connection classes for property-targeting, mutation-targeting, and ordinary relationship edges.
- [x] 2.2 Style property-targeting connections as dashed lines with no target marker and retain the property semantic label below its node.
- [x] 2.3 Define a filled black circular SVG target marker and apply it to every mutation-targeting connection while keeping its line solid.
- [x] 2.4 Verify ordinary relationship edges preserve their existing solid line, arrow marker, control, and label presentation.

## 3. Verification and Documentation

- [x] 3.1 Add modeler tests covering property-targeting edges from each supported source type, including absent edge labels and target markers.
- [x] 3.2 Add modeler tests covering mutation-targeting edges from each supported source type, including solid lines and black circular target markers.
- [x] 3.3 Add save-and-reopen regression coverage proving imported and newly created connections use the same target-driven notation and property-source edges follow their actual targets.
- [x] 3.4 Update Data Graph notation documentation to describe dependent property edges, below-node property labels, and mutation target circles.
- [x] 3.5 Run the focused web-editor test suite and the broader project checks affected by Data Graph rendering and persistence.
