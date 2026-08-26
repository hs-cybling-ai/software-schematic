#!/bin/sh
set -eu

project_root="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$project_root/web-editor"
npm ci
npm run verify
test -f "$project_root/DiagramStudio/Sources/DiagramStudio/Resources/Web/index.html"
! grep -q 'type="module"' "$project_root/DiagramStudio/Sources/DiagramStudio/Resources/Web/index.html"
! grep -q 'crossorigin' "$project_root/DiagramStudio/Sources/DiagramStudio/Resources/Web/index.html"
grep -q '<script defer ' "$project_root/DiagramStudio/Sources/DiagramStudio/Resources/Web/index.html"
