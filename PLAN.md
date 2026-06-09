# PLAN — tree-sitter-m1

## Ecosystem (6 sibling repos, laid out via vcstool)

```
tree-sitter-m1  → grammar + queries + Rust parser crate         (foundation)
m1-core         → CST helpers, .m1prj/.m1cfg symbol model, diags (shared lib)
m1-fmt          → autoformatter           (CLI + lib)
m1-lint         → linter (CONTRIBUTING.md rules)  (CLI + lib)
m1-typecheck    → typehinter (Hungarian + .m1prj types)  (CLI + lib)
m1-lsp          → tower-lsp server tying the above together
```

Dependency direction: `lint / fmt / typecheck / lsp → m1-core → tree-sitter-m1`.
Test corpus: real scripts in `../m1-example/UQR-EV/01.00/Scripts/*.m1scr` and the
`../m1-example/UQR-EV/01.00/Project.m1prj` symbol table (override the location
with `$M1_CORPUS_PATH`). Language reference: the two PDFs in
`../m1-example/M1-docs/`.

This repo is **Phase 1** and blocks everything else.

## Scope of this repo

Parse `.m1scr` into a concrete syntax tree. No semantics — that lives in
`m1-core` and above.

## Done

- Grammar for: comments, number/hex/float/bool/string literals (incl. `u`
  integer suffix), `local` declarations (Hungarian + `<Type>` forms, optional
  `static`), assignment (`= += -= *= /=`), `if`/`else if`/`else`, `when`/`is`
  state-machine blocks, `expand` compile-time loops with `$(VAR)` interpolation,
  blocks, expression statements, calls, member access, unary/binary/ternary
  expressions, word operators (`eq neq and or not`), bitwise/shift (`>> << & ^ |`).
- External scanner for space-joined identifier segments, including digit-led
  words (`XV Glim 4`, `5X`) and `$(...)` interpolation units.
- highlight / indent / fold / injection queries.
- Rust binding crate (`LANGUAGE`, query strings).
- Corpus acceptance gate (`scripts/check-corpus.sh`) + construct regression tests
  (`test/corpus/constructs.txt`). **All 80 m1-example scripts parse with zero errors.**

## TODO — grammar correctness (drive from the corpus)

- [x] `npm install` + `tree-sitter generate`; commit `src/parser.c`.
- [x] Parse **all** `m1-example` scripts with zero ERROR nodes; add a script that
      greps the parse output for `ERROR`/`MISSING` across the corpus.
      (`scripts/check-corpus.sh` — currently `80 parsed, 0 with errors`.)
- [x] Write corpus tests from verified output (don't hand-write trees).
      (`test/corpus/constructs.txt`, generated via `tree-sitter test --update`.)
- [x] Confirm the reserved-word set against the M1 Development Manual. Added from
      the corpus: `static`, `when`, `is`, `expand`, `to`, plus bitwise/shift
      operators (`>> << & ^ |`) and the `u` integer suffix. **`xor`/`mod` are
      ABSENT from the manual's operator tables (pp.36–38)** — only `eq`, `neq`,
      `and`, `or`, `not` exist as word operators. No grammar change needed.
- [x] Function/scheduled-function definitions: **functions are declared in
      `.m1prj`, never in `.m1scr`** (confirmed against manual and both real
      corpora — no function-declaration syntax appears in any `.m1scr` file).
      No grammar addition needed.
- [x] Validate the scanner against pathological names (covered by
      `test/corpus/scanner_edges.txt`) and standalone `$(VAR)` (now its own
      `interpolation` node; `test/corpus/interpolation.txt`).
- [ ] Decide handling of multi-word names whose first word is a keyword (should
      be impossible by MoTeC naming rules — assert via lint, not grammar).

## TODO — editor integration (handled in the Neovim plugin, tracked here)

- [x] `locals.scm` shipped (CST-only scopes + local/expand definitions);
      channel/param resolution still pending `m1-core` scopes.
- [x] Verify indents.scm against the tab / Allman-brace style mandated by the M1
      Development Manual (pp.26–27). Analysis: `@indent.begin` fires on
      `(block)`, `(when_statement)`, `(argument_list)`, and
      `(parenthesized_expression)` — all correct for Allman style (contents
      inside `{}` are indented; the Allman opening `{` on its own line is
      handled by the `@indent.branch` / `@indent.end` pairing on `}`). The new
      `is_pattern_list` lives inside `is ( ... )` which is already covered by
      the `(parenthesized_expression)` or is single-line; no indent/fold query
      change needed. `folds.scm` correctly folds `(block)` and
      `(when_statement)`. No defects found.

## Open questions for the owner

- License choice (currently treated as proprietary).
- Grammar versioning / whether to publish to crates.io for the team, or keep as
  a git dependency.
