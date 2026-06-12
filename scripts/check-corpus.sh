#!/usr/bin/env bash
#
# check-corpus.sh — parse every real .m1scr in the m1-example corpus and fail if any
# produces an ERROR or MISSING node — the grammar's corpus acceptance gate
# ("parse all m1-example scripts with zero ERROR nodes").
#
# Usage: scripts/check-corpus.sh <corpus_dir>
#   corpus_dir may also be supplied via $M1_CORPUS_PATH. There is no default:
#   real-world corpora live outside this repo, so point this at your own
#   directory of .m1scr scripts.

set -u

here="$(cd "$(dirname "$0")/.." && pwd)"
corpus="${1:-${M1_CORPUS_PATH:-}}"
if [ -z "$corpus" ]; then
  echo "usage: scripts/check-corpus.sh <corpus_dir>   (or set M1_CORPUS_PATH)" >&2
  exit 2
fi

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

# Zero files is a hard error, not a pass: a missing or mistyped corpus dir
# previously made the gate green while testing nothing (#51).
if [ "$total" -eq 0 ]; then
  echo "ERROR: no .m1scr files found under $corpus" >&2
  echo "set M1_CORPUS_PATH (or pass a corpus dir); refusing to pass vacuously" >&2
  exit 1
fi

echo "parsed $total scripts; $failed with ERROR/MISSING nodes"
if [ "$failed" -ne 0 ]; then
  printf 'FAILURES:\n%b' "$fail_list"
  exit 1
fi
echo "OK — corpus parses clean"
