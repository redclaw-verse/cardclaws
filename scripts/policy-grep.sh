#!/usr/bin/env bash
# "No stubs / no placeholders" enforcement (PRD §15.4), adjusted per plan A3:
# scope to genuine stub markers in comments/macros rather than a blunt word
# match, so legitimate `placeholder=` input attributes don't false-positive.
# Test files are excluded. Portable to bash 3.2 (macOS) — no `mapfile`.
set -euo pipefail

PATTERNS='(//[[:space:]]*TODO|//[[:space:]]*FIXME|/\*[[:space:]]*TODO|unimplemented!\(|todo!\(|coming soon|501 Not Implemented)'

hits=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  if grep -nEi "$PATTERNS" "$f"; then
    echo "  ^ in $f"
    hits=1
  fi
done < <(
  git ls-files '*.rs' '*.ts' '*.tsx' '*.js' '*.jsx' '*.astro' \
    | grep -vE '(/__tests__/|/tests/|\.test\.|\.spec\.|_test\.rs$)' || true
)

if [ "$hits" -ne 0 ]; then
  echo "::error::Stub/placeholder markers found in source. Remove before merge."
  exit 1
fi
echo "policy-grep: no stub/placeholder markers in source."
