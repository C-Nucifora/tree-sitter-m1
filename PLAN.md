# PLAN — tree-sitter-m1

## Ecosystem (6 repos under `temp/`)

```
tree-sitter-m1  → grammar + queries + Rust parser crate         (foundation)
m1-core         → CST helpers, .m1prj/.m1cfg symbol model, diags (shared lib)
m1-fmt          → autoformatter           (CLI + lib)
m1-lint         → linter (CONTRIBUTING.md rules)  (CLI + lib)
m1-typecheck    → typehinter (Hungarian + .m1prj types)  (CLI + lib)
m1-lsp          → tower-lsp server tying the above together
```

Dependency direction: `lint / fmt / typecheck / lsp → m1-core → tree-sitter-m1`.
Test corpus: real scripts in `../EV-M1/UQR-EV/01.00/Scripts/*.m1scr` and the
`../EV-M1/UQR-EV/01.00/Project.m1prj` symbol table. Language reference: the two
PDFs in `../EV-M1/M1-docs/`.

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
  (`test/corpus/constructs.txt`). **All 80 EV-M1 scripts parse with zero errors.**

## TODO — grammar correctness (drive from the corpus)

- [x] `npm install` + `tree-sitter generate`; commit `src/parser.c`.
- [x] Parse **all** `EV-M1` scripts with zero ERROR nodes; add a script that
      greps the parse output for `ERROR`/`MISSING` across the corpus.
      (`scripts/check-corpus.sh` — currently `80 parsed, 0 with errors`.)
- [x] Write corpus tests from verified output (don't hand-write trees).
      (`test/corpus/constructs.txt`, generated via `tree-sitter test --update`.)
- [~] Confirm the reserved-word set against the M1 Development Manual. Added from
      the corpus: `static`, `when`, `is`, `expand`, `to`, plus bitwise/shift
      operators (`>> << & ^ |`) and the `u` integer suffix. Still TODO: confirm
      against the manual whether `xor`/`mod` word-operators or function /
      scheduled-function declaration keywords exist (none appear in the corpus).
- [ ] Function/scheduled-function definitions: the corpus is mostly statement
      bodies, but the language has function objects with typed inputs/outputs —
      confirm whether they ever appear in `.m1scr` text and grammar them if so.
- [ ] Validate the scanner against pathological names
      (`QZP MV31 R7 TKN 5X Glim Bront`, trailing-space-before-`.`,
      name immediately followed by `eq`/`and`).
- [ ] Decide handling of multi-word names whose first word is a keyword (should
      be impossible by MoTeC naming rules — assert via lint, not grammar).

## TODO — editor integration (handled in the Neovim plugin, tracked here)

- [ ] `locals.scm` for scope-aware highlighting once `m1-core` defines scopes.
- [ ] Verify indents.scm against the 4-space / brace style in CONTRIBUTING.md.

## Open questions for the owner

- License choice (currently treated as proprietary).
- Grammar versioning / whether to publish to crates.io for the team, or keep as
  a git dependency.
