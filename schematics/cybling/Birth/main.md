# Cybling Birth

This folder is the shared implementation of process `cybling.Birth`. Every call activity or subprocess with that exact Name opens this same `main.bpmn` and `main.md`.

The wallet app creates the Cybling screen, requests birth through the SDK, creates and signs the programmable transaction block, and sends the transaction to the sponsor service.

Element documentation in this composition remains bound to BPMN IDs under `docs/`. For example, `docs/Activity_04ay32x.md` documents the `cybling.Birth#requestBirth` occurrence.
