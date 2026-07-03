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

mapfile -t files < <(find "$corpus" -name '*.m1scr')
total=${#files[@]}

# Zero files is a hard error, not a pass: a missing or mistyped corpus dir
# previously made the gate green while testing nothing (#51).
if [ "$total" -eq 0 ]; then
  echo "ERROR: no .m1scr files found under $corpus" >&2
  echo "set M1_CORPUS_PATH (or pass a corpus dir); refusing to pass vacuously" >&2
  exit 1
fi

# Parse every script in ONE tree-sitter invocation (xargs batches to respect
# ARG_MAX) rather than spawning `npx tree-sitter` per file — ~80x faster on a
# large corpus, so gating the second corpus (AV-M1, ~1450 files) stays cheap.
# Quiet mode prints one tab-separated line per file; a parse error carries an
# ERROR/MISSING node on that file's line, with the path at the start.
parse_out="$(printf '%s\0' "${files[@]}" | xargs -0 npx tree-sitter parse --quiet 2>/dev/null)"
fail_list="$(printf '%s\n' "$parse_out" | grep -E 'ERROR|MISSING' | sed -E 's/\t.*//; s/^/  /')"
if [ -n "$fail_list" ]; then
  failed="$(printf '%s\n' "$fail_list" | grep -c .)"
else
  failed=0
fi

echo "parsed $total scripts; $failed with ERROR/MISSING nodes"
if [ "$failed" -ne 0 ]; then
  printf 'FAILURES:\n%s\n' "$fail_list"
  exit 1
fi
echo "OK — corpus parses clean"
