# Diagram fixtures

- BPMN: `.bpmn` and `.bpmn20.xml`, BPMN 2.0 XML accepted by `bpmn-js`.
- ArchiMate: `.archimate` and `.archimate.xml`, Open Group ArchiMate Exchange XML accepted by `archimate-js` 0.0.4.
- Data Graph: `.dgraph`, Diagram Studio's normalized ontology format containing topology plus a reference to its sibling `.context.sqlite` Markdown/embedding database. The valid fixture includes a cycle-safe `sameAs`/`subclassOf` link cycle.
- Each format includes a valid smoke-test document and deliberately malformed input used to confirm non-destructive import failures.
