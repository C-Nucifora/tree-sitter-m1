#!/usr/bin/env bash
#
# check-corpus.sh — parse every real .m1scr in the EV-M1 corpus and fail if any
# produces an ERROR or MISSING node. This is the Phase-1 acceptance gate from
# PLAN.md ("Parse all EV-M1 scripts with zero ERROR nodes").
#
# Usage: scripts/check-corpus.sh [corpus_dir]
#   corpus_dir defaults to ../EV-M1/UQR-EV/01.00/Scripts

set -u

here="$(cd "$(dirname "$0")/.." && pwd)"
corpus="${1:-$here/../EV-M1/UQR-EV/01.00/Scripts}"
tsc="npx --prefix \"$here\" tree-sitter"

cd "$here" || exit 2

total=0
failed=0
fail_list=""

while IFS= read -r f; do
  total=$((total + 1))
  out="$(npx tree-sitter parse --quiet "$f" 2>&1)"
  if printf '%s' "$out" | grep -qE 'ERROR|MISSING'; then
    failed=$((failed + 1))
    fail_list="$fail_list  $f\n"
  fi
done < <(find "$corpus" -name '*.m1scr')

echo "parsed $total scripts; $failed with ERROR/MISSING nodes"
if [ "$failed" -ne 0 ]; then
  printf 'FAILURES:\n%b' "$fail_list"
  exit 1
fi
echo "OK — corpus parses clean"
