#!/bin/sh
set -eu

project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if [ "${1:-}" = "--clean-export" ]; then
  export_root=$(mktemp -d "${TMPDIR:-/tmp}/software-schematic-release.XXXXXX")
  trap 'rm -rf "$export_root"' EXIT HUP INT TERM
  file_list="$export_root/files.txt"
  (
    cd "$project_root"
    git ls-files --cached --others --exclude-standard | while IFS= read -r path; do
      if [ -f "$path" ] || [ -L "$path" ]; then
        printf '%s\n' "$path"
      fi
    done > "$file_list"
    tar -cf "$export_root/source.tar" -T "$file_list"
  )
  mkdir -p "$export_root/source"
  tar -xf "$export_root/source.tar" -C "$export_root/source"
  SSW_CLEAN_EXPORT_ACTIVE=1 "$export_root/source/scripts/verify-release.sh"
  exit 0
fi

cd "$project_root"

allowed_top_level='.codex .github .gitignore CONTRIBUTING.md LICENSE NOTICE README.md SECURITY.md THIRD_PARTY_NOTICES.md docs openspec scripts software-schematic-cli software-schematic-web'
if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  actual_top_level=$(git ls-files --cached --others --exclude-standard | cut -d/ -f1 | sort -u)
else
  actual_top_level=$(find . -mindepth 1 -maxdepth 1 ! -name . -exec basename {} \; | sort -u)
fi
for path in $actual_top_level; do
  case " $allowed_top_level " in
    *" $path "*) ;;
    *) echo "Unexpected tracked top-level path: $path" >&2; exit 1 ;;
  esac
done

for required in \
  LICENSE NOTICE THIRD_PARTY_NOTICES.md \
  software-schematic-cli/Cargo.toml software-schematic-cli/Cargo.lock \
  software-schematic-cli/assets/starter.cmmn \
  software-schematic-cli/assets/starter.bpmn \
  software-schematic-cli/assets/starter.md \
  software-schematic-cli/assets/models/all-MiniLM-L6-v2/NOTICE.md \
  software-schematic-cli/assets/models/all-MiniLM-L6-v2/all-MiniLM-L6-v2.onnx \
  software-schematic-cli/assets/models/all-MiniLM-L6-v2/tokenizer.json \
  software-schematic-web/package.json software-schematic-web/package-lock.json; do
  test -f "$required" || { echo "Missing required release input: $required" >&2; exit 1; }
done

npm ci --prefix software-schematic-web
npm test --prefix software-schematic-web
npm run build --prefix software-schematic-web

cargo fmt --manifest-path software-schematic-cli/Cargo.toml --check
if [ "${SSW_SKIP_NATIVE_MODEL_TEST:-0}" = "1" ]; then
  cargo test --manifest-path software-schematic-cli/Cargo.toml -- --skip initialized_project_loads_the_packaged_embedding_model
else
  cargo test --manifest-path software-schematic-cli/Cargo.toml
fi
cargo build --release --manifest-path software-schematic-cli/Cargo.toml

if command -v openspec >/dev/null 2>&1; then
  openspec validate --all
else
  echo "openspec is required for release verification" >&2
  exit 1
fi

release_bin=software-schematic-cli/target/release/ss
if [ "${OS:-}" = "Windows_NT" ]; then
  release_bin=software-schematic-cli/target/release/ss.exe
fi
test -x "$release_bin" || { echo "Release executable missing: $release_bin" >&2; exit 1; }

smoke_root=$(mktemp -d "${TMPDIR:-/tmp}/software-schematic-smoke.XXXXXX")
trap 'rm -rf "$smoke_root"' EXIT HUP INT TERM
project="$smoke_root/project"
"$release_bin" init "$project"

for installed in \
  .ss/LICENSE .ss/NOTICE .ss/THIRD_PARTY_NOTICES.md \
  .ss/web/index.html .ss/models/all-MiniLM-L6-v2/all-MiniLM-L6-v2.onnx \
  .ss/models/all-MiniLM-L6-v2/tokenizer.json .ss/operation-plan.schema.json \
  schematics/main.cmmn schematics/main.md ssw ssw.cmd; do
  test -e "$project/$installed" || { echo "Missing initialized asset: $installed" >&2; exit 1; }
done

printf '\nAuthored release verification marker.\n' >> "$project/schematics/main.md"
diagram_before=$(shasum -a 256 "$project/schematics/main.cmmn" | cut -d' ' -f1)
markdown_before=$(shasum -a 256 "$project/schematics/main.md" | cut -d' ' -f1)
"$release_bin" update --project "$project"
diagram_after=$(shasum -a 256 "$project/schematics/main.cmmn" | cut -d' ' -f1)
markdown_after=$(shasum -a 256 "$project/schematics/main.md" | cut -d' ' -f1)
test "$diagram_before" = "$diagram_after"
test "$markdown_before" = "$markdown_after"
cmp LICENSE "$project/.ss/LICENSE"
cmp NOTICE "$project/.ss/NOTICE"
cmp THIRD_PARTY_NOTICES.md "$project/.ss/THIRD_PARTY_NOTICES.md"
"$project/ssw" --help >/dev/null

echo "Software Schematic release verification passed${SSW_CLEAN_EXPORT_ACTIVE:+ in a clean export}."
