# Third-party notices

Software Schematic includes third-party software and assets. Those works remain
under their own licenses; the repository's Apache License 2.0 does not replace
their terms.

## Bundled browser application

- `bpmn-js` — copyright Camunda Services GmbH; bpmn.io license (MIT-style with
  the additional requirement that the bpmn.io watermark remain visible).
- `cmmn-js` — copyright camunda services GmbH; bpmn.io license (MIT-style with
  the additional requirement that the bpmn.io logo remain visible).
- `DOMPurify` — Mozilla Public License 2.0 or Apache License 2.0.
- `Lucide` — ISC License; portions copyright Cole Bemis and other portions
  copyright Lucide Contributors.
- `markdown-it` — MIT License; copyright Vitaly Puzrin and Alex Kocharin.
- `Mermaid` — MIT License; copyright Knut Sveidqvist and contributors.

The compiled bundle also contains transitive dependencies recorded exactly in
`software-schematic-web/package-lock.json`. Their package metadata and license
files are installed by `npm ci`. Build-only tools such as Vite, Vitest, and
jsdom are not shipped as standalone packages but remain governed by their
respective licenses.

## Rust executable

The executable statically or dynamically incorporates Rust crates recorded
exactly in `software-schematic-cli/Cargo.lock`. License metadata for the direct
and transitive crate graph can be inspected with `cargo metadata` and the
corresponding registry source license files. Notable direct components include
Axum, Clap, Grafeo, ONNX Runtime bindings, Quick XML, Reqwest, RMCP, Serde,
Tokio, and Tower HTTP.

ONNX Runtime is loaded from the user's system and is distributed separately by
Microsoft under the MIT License. Software Schematic does not redistribute its
native runtime library.

## Embedding model

The bundled `sentence-transformers/all-MiniLM-L6-v2` ONNX model and tokenizer
are distributed under Apache License 2.0. Its source and transformation note is
retained at
`software-schematic-cli/assets/models/all-MiniLM-L6-v2/NOTICE.md`.

## Fonts and notation assets

The bundled BPMN and CMMN font files originate from the bpmn.io toolchain and
are distributed with the applicable bpmn.io packages. The bpmn.io watermark
and logo requirements must remain intact in rendered diagrams.

## Review procedure

When a dependency or embedded asset changes:

1. Regenerate the applicable lockfile and production bundle.
2. Inspect every changed package's declared license and included license or
   notice files.
3. Update this file and preserve any required copyright, attribution,
   watermark, source-offer, or notice text.
4. Run `./scripts/verify-release.sh --clean-export` before publishing.

This notice is an engineering inventory, not legal advice. Cybling Labs should
review it before the first public release and whenever licensing terms change.
