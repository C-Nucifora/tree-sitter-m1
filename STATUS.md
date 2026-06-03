# STATUS — M1 tooling ecosystem

Source of truth for what is actually built, verified against the code and the
m1-example corpus on 2026-05-30.

## Repo-by-repo

| Repo            | State            | Evidence |
|-----------------|------------------|----------|
| `tree-sitter-m1`| **Phase 1 complete** | Grammar + external scanner + queries + Rust bindings. `src/parser.c` regenerated. **All 80 corpus scripts parse with zero ERROR/MISSING nodes** (`scripts/check-corpus.sh`). 6/6 construct regression tests pass (`test/corpus/constructs.txt`). v2: standalone `interpolation` node; enriched highlights + new `locals.scm`; corpus expanded (interpolation/scanner-edges/errors/literals). Follow-ups: m1-core must re-vendor `parser.c`/`scanner.c`/`node-types.json` and regenerate `Kind` (gains `Interpolation`); `.nvim-test/queries/m1` must be re-copied and gain `indents.scm`. |
| `m1-core`       | **v1 in progress** | CST helpers (wrapped tree-sitter nodes), shared `Diagnostic` type, and syntax-error diagnostics. Generated `Kind` enum (committed, freshness-tested). Corpus regression test green. Symbol model deferred. |
| `m1-fmt`        | **not started**  | Only empty `src/` and `tests/` dirs. |
| `m1-lint`       | **not started**  | Only empty `src/` and `tests/` dirs. |
| `m1-typecheck`  | **not started**  | Only empty `src/` and `tests/` dirs. |
| `m1-lsp`        | **not started**  | Only empty `src/` dir. |

Dependency order is `lint/fmt/typecheck/lsp → m1-core → tree-sitter-m1`. Phase 1
(tree-sitter-m1) is now done, so **m1-core is unblocked and is the next repo.**

## Acceptance gate

`tree-sitter-m1/scripts/check-corpus.sh` parses every `*.m1scr` under
`m1-example/UQR-EV/01.00/Scripts` (override with `$M1_CORPUS_PATH`) and fails if
any ERROR/MISSING node appears.
Current: `parsed 80 scripts; 0 with ERROR/MISSING nodes`.

## Phase-1 root causes — all resolved (corpus-driven)

The 25 failing scripts traced to six concrete language features. All are now
handled by the grammar/scanner:

- **A. Name segments containing digit-led words** (`XV Glim 4`, `5X`, `Glonk 9`,
  `Plun 6`) — scanner now continues a segment into digit-led words; only the
  *first* word of a segment must start with a letter/`_`/`$`. *~20 files.*
- **B. Bitwise operators** — added `>>` `<<` (shift) and `&` `^` `|` with C-like
  precedence. Corpus uses only `>>` (96×) and `&` (107×); siblings added as safe
  standard operators. (`~` hits were all comment ASCII-art.)
- **C. `static local`** — `optional("static")` on `local_declaration` (3×).
- **D. Unsigned integer suffix** — `number` token accepts trailing `[uU]` (`0u`).
- **E. `when`/`is` state-machine blocks** — `when_statement` + `is_clause` rules.
  11 files.
- **F. `expand` loop + `$(VAR)` interpolation** — `expand_statement` rule plus
  scanner support for `$(...)` as a unit inside identifier segments
  (`naxID Bnk $(SEG) Vlim $(NODE)`). 1 file. Keywords `expand`/`to` reserved.

`when/is/static/expand/to` added to the scanner's reserved-word set (verified no
identifier segment in the corpus contains them as words). `for/while/foreach/
return/function/choose/case/step` do **not** appear in `.m1scr` driver scripts
(only in comments), so they remain out of scope for Phase 1.

## Done (verified)

- Grammar covers: comments, number/hex/float/bool/string literals, `local`
  declarations (Hungarian + `<Type>`), assignment (`= += -= *= /=`),
  `if`/`else if`/`else`, blocks, expression statements, calls, member access,
  unary/binary/ternary, word operators (`eq neq and or not`, plus `== != && ||`).
- External scanner joins space-separated identifier words, refusing reserved words.
- highlight/indent/fold/injection queries; Rust binding crate; generated `parser.c`.
