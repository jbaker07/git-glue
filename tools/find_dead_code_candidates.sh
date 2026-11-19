#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
cd "$ROOT"

echo "== Dead-code CANDIDATES (review list; NOT deleted) =="

# Find all .rs files (not in attic)
find src -type f -name '*.rs' -not -path '*/attic/*' | sort | while read -r f; do
  base="$(basename "$f" .rs)"
  
  # Skip special files
  if [[ "$f" == src/bin/* ]] || [[ "$base" == "lib" ]] || [[ "$base" == "main" ]] || [[ "$base" == "mod" ]]; then
    continue
  fi
  
  # Skip if declared as a module
  if grep -rq "pub mod $base" src --include="*.rs"; then
    continue
  fi
  
  # Skip if referenced: look for "use ...base" or "::base" patterns
  if grep -rq "use.*$base\|::$base\|mod $base" src --include="*.rs"; then
    continue
  fi
  
  echo "$f"
done


