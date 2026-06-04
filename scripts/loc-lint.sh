#!/usr/bin/env bash
# Fail the build if any source file exceeds 1,300 lines (PRD §14).
# Test files are exempt (they must be co-located but may exceed the limit).
set -euo pipefail

LIMIT=1300
fail=0

# Source files across backend, mobile, and web, excluding tests. Portable to
# bash 3.2 (macOS) — no `mapfile`.
while IFS= read -r f; do
  [ -z "$f" ] && continue
  lines=$(wc -l < "$f")
  if [ "$lines" -gt "$LIMIT" ]; then
    echo "LOC LIMIT EXCEEDED: $f has $lines lines (max $LIMIT)"
    fail=1
  fi
done < <(
  git ls-files '*.rs' '*.ts' '*.tsx' '*.js' '*.jsx' '*.astro' \
    | grep -vE '(/__tests__/|/tests/|\.test\.|\.spec\.|_test\.rs$)' || true
)

if [ "$fail" -ne 0 ]; then
  echo "::error::One or more files exceed the $LIMIT-line limit."
  exit 1
fi
echo "loc-lint: all source files within $LIMIT lines."
