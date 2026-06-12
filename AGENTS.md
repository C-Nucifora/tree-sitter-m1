# AGENTS.md — tree-sitter-m1

Guidance for coding agents working in this repository.

## Purpose

The grammar at the root of the M1 toolchain's dependency graph. Everything
else (`m1-core`, and through it the formatter, linter, type checker, language
server, and editor plugins) parses M1 with this grammar — a change here is
never local. The M1 Development Manual's operator/construct tables are the
spec; the two real-world corpora are the acceptance test.

## Things that are deliberate (don't "fix" them)

- **The external scanner is load-bearing.** M1 identifiers contain spaces
  (`Front Axle Torque`), so tokenisation can't be regex-only: `src/scanner.c`
  joins words greedily while refusing reserved words. Its reserved set must
  stay in sync with the keyword strings in `grammar.js` — adding a keyword
  means touching both.
- **`src/parser.c` is generated but committed** (so Rust consumers build
  without the tree-sitter CLI). Any `grammar.js` change requires regenerating
  (`npx tree-sitter generate`) and committing the result in the same change.
- **`$(VAR)` interpolation is positional**: standalone operand → its own
  node; inside a multi-word name → part of the identifier segment. Keep "one
  identifier = one path segment" intact.

## Testing a grammar change

1. `npx tree-sitter test` — corpus tests in `test/corpus/` (add cases for new
   constructs; use synthetic identifiers, never real project names).
2. `scripts/check-corpus.sh` — parse every real-corpus script with zero
   `ERROR`/`MISSING` nodes. The corpus is an optional sibling checkout; a run
   without it proves much less.
3. The fuzz harness (`fuzz/`) covers parser+scanner crashes and
   incremental-parse consistency; CI runs a smoke pass on grammar changes.

## Release ripple

A release here is only step one: `m1-core` must bump its tag and regenerate
its `Kind`/`Field` enums from the new `node-types.json`, then release, then
the four consumers bump. A new construct typically isn't usable downstream
until that whole cascade lands — open the `m1-core` bump PR immediately after
tagging. Deps are **versioned git tags** everywhere; never `branch`/`path`/
`[patch]`.

## CI gate

`npm test` (corpus), the real-corpus parse gate, the fuzz smoke, and the Rust
binding build. The grammar repo also follows the toolchain MSRV convention:
the CI toolchain pin must stay in sync with the binding crate's
`rust-version`.
