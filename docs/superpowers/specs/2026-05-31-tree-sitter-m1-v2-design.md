# tree-sitter-m1 v2 — Design Specification

**Date:** 2026-05-31
**Status:** Approved for implementation
**Scope:** v2 — a standalone `interpolation` node, a richer `highlights.scm`, a
new `locals.scm`, a broadened corpus (error-recovery + scanner edge cases +
literals), and a downstream-sync note for `m1-core` and `.nvim-test`.
**Spec (v1):** none — v1 was tracked in `PLAN.md` / `STATUS.md`.

> **Note:** Every example identifier in this document, in the grammar/scanner
> comments, in the queries, and in the corpus tests is a **synthetic
> placeholder**. None is drawn from any real MoTeC EV-M1 project. The repo
> already carries this obfuscation disclaimer in `grammar.js`, `src/scanner.c`,
> and `README.md`; v2 preserves the practice in every file it adds or edits.

---

## 1. Purpose

v1 (declared "Phase 1 complete" in `STATUS.md`) shipped a tree-sitter grammar
for the MoTeC M1 script language (`.m1scr`): an external scanner for
space-joined identifier segments, statement/expression coverage driven by the
EV-M1 corpus (all 80 scripts parse with zero ERROR/MISSING nodes), a generated
`src/parser.c`, a Rust binding crate, four queries (`highlights`, `indents`,
`folds`, `injections`), and a 6-case construct corpus
(`test/corpus/constructs.txt`).

v2 is the **next useful increment** for the two consumers that actually exist:

- **`m1-core`** wraps the grammar (committed `parser.c` / `scanner.c`, Rust
  bindings, a generated `Kind` enum derived from `src/node-types.json`).
- **`.nvim-test`** loads the compiled parser + the `queries/` for Neovim
  treesitter highlighting/folding.

The chosen v2 deliverables, each grounded in a gap found by reading the
grammar, scanner, queries, and corpus:

1. **A standalone `interpolation` node** for `$(VAR)` used as an *operand*
   (e.g. `x = $(SEG) + 1`), which v1 silently collapses into `identifier`.
   Embedded interpolation inside a multi-word name stays part of the
   `identifier` segment (segment cohesion is load-bearing for `m1-core`).
2. **A richer, more correct `highlights.scm`**: fix the `<` / `>` double-capture
   collision (currently captured as both `@operator` and `@punctuation.bracket`),
   add `@keyword.conditional` / `@keyword.repeat` splits, `@constant.builtin`
   for booleans, `@number` precision, `@variable.parameter` for the `expand`
   loop variable, `@function.call`/`@function.method` consistency, and an
   `interpolation` capture.
3. **A new `locals.scm`** defining scopes and definition/reference bindings for
   `local` declarations and `expand` loop variables — schedulable now without
   the `m1-core` symbol model (it only needs CST structure).
4. **A broadened `test/corpus/`**: error-recovery cases (locking in ERROR-node
   behaviour `m1-core` surfaces as syntax diagnostics), scanner edge cases
   (name immediately followed by a keyword, trailing space before `.`,
   pathological long names, standalone vs embedded interpolation), and
   literal/comment edge cases.
5. **A downstream-sync follow-up note** (NOT executed here): regenerate
   `src/parser.c`, then `m1-core` must re-vendor `parser.c` / `scanner.c` /
   `node-types.json` and re-run its `Kind` generator; `.nvim-test/queries/m1/`
   must be re-copied and gain `indents.scm`.

Out of scope (YAGNI; see §8): `xor` / `mod` word-operators, function /
scheduled-function definitions, grammar-level error-recovery rules, and
tab-aware indentation — none has corpus evidence or a present consumer need.

---

## 2. What v1 Already Provides (build on this, do not re-invent)

The actual v1 surface, read from the repo:

- **`grammar.js`** — `name: "m1"`; `extras` = whitespace + `line_comment` +
  `block_comment`; `externals: ($) => [$.identifier]`; one conflict
  (`assignment_statement` vs `expression_statement`). Rules: `source_file`,
  `_statement` (`local_declaration`, `assignment_statement`, `if_statement`,
  `when_statement`, `expand_statement`, `expression_statement`, `block`,
  `empty_statement`), `type_annotation`, `else_clause`, `is_clause`,
  expression rules (`member_expression`, `call_expression`, `argument_list`,
  `parenthesized_expression`, `unary_expression`, `ternary_expression`,
  `binary_expression`), and tokens (`number`, `boolean`, `string`,
  `line_comment`, `block_comment`).
- **`src/scanner.c`** — external scanner emitting a single token `IDENTIFIER`.
  It greedily joins `word SPACE word`, refuses to absorb reserved words
  (`RESERVED[]` must mirror the grammar's keyword strings), supports digit-led
  continuation words (`XV Glim 4`), and consumes `$(...)` interpolation units
  via `try_read_interp` — both as the first unit and as continuation units of a
  segment. `mark_end` keeps a trailing ` eq`/` and` as look-ahead.
- **`queries/highlights.scm`** — keywords, operators, punctuation, literals,
  `type_annotation` → `@type`, call-callee → `@function`/`@function.method`,
  member property → `@property`, plain `identifier` → `@variable`.
- **`queries/indents.scm`** — `(block)`/`(argument_list)`/
  `(parenthesized_expression)` as `@indent.begin`; `}`/`)` as branch + end.
- **`queries/folds.scm`** — `(block)` + `(block_comment)`.
- **`queries/injections.scm`** — placeholder (no injections).
- **`test/corpus/constructs.txt`** — 6 cases; bare S-expression form (no field
  names, no positions), generated via `tree-sitter test --update`.
- **`bindings/rust/lib.rs`** — `LANGUAGE`, `HIGHLIGHTS_QUERY`, `INDENTS_QUERY`,
  `FOLDS_QUERY`, `NODE_TYPES_JSON` (each `include_str!`'d). No `LOCALS_QUERY`,
  no `INJECTIONS_QUERY` yet.
- **`scripts/check-corpus.sh`** — acceptance gate: parses every `*.m1scr` under
  the EV-M1 Scripts dir, fails on any ERROR/MISSING node.

v2 adds new grammar rules + an external token, edits queries, adds `locals.scm`,
adds corpus files, and exposes two new query constants — without breaking the
existing node kinds that `m1-core`'s generated `Kind` enum depends on.

---

## 3. Key Decisions

### 3.1 `interpolation` as a standalone external token (not a restructure)

v1's scanner consumes `$(...)` two ways: as the **first unit** of a segment and
as a **continuation unit**. As a result, an operand-position `$(SEG)` currently
parses as a bare `identifier`:

```
x = $(SEG) + 1;
      └────── parsed today as: value (binary_expression left: (identifier) right: (number))
```

That hides a compile-time substitution behind the same node a channel name
uses. v2 adds a **second external token**, `interpolation`, that the scanner
emits **only** when a `$(...)` stands alone (i.e. it is not immediately
preceded, within the same word run, by an identifier word). The grammar adds
`$.interpolation` to the `_expression` choice.

**Segment cohesion is preserved.** Embedded interpolation inside a multi-word
name (`naxID Bnk $(SEG) Vlim`) must remain part of the single `identifier`
segment, because `m1-core`'s CST helpers treat one identifier node as one path
segment. The scanner therefore keeps absorbing `$(...)` as a *continuation*
unit; it only emits the new `interpolation` token when `$(...)` is the **first
and only** unit and no identifier word precedes it. The disambiguation is driven
by `valid_symbols`: in operand position both `IDENTIFIER` and `INTERPOLATION`
are valid, and a leading `$(` with no following ` word` resolves to
`INTERPOLATION`; a leading identifier word resolves to `IDENTIFIER` (and may
then absorb embedded `$(...)`).

**Externals ordering.** `externals: ($) => [$.identifier, $.interpolation]`.
The scanner's `TokenType` enum gains `INTERPOLATION` as the second variant
(order must match the `externals` array exactly).

### 3.2 Scanner algorithm change (precise)

`tree_sitter_m1_external_scanner_scan` is restructured so the **first unit**
decides which token is produced:

1. Skip leading whitespace.
2. If `lookahead == '$'`:
   - Try `try_read_interp`. If it fails, `return false`.
   - `mark_end`. Then peek: if the next char is a space *followed by* a word
     start or another `$(` (i.e. the interpolation is the head of a multi-word
     segment), continue as an **identifier** segment (existing continuation
     loop) and set `result_symbol = IDENTIFIER`. Otherwise (the common
     operand case) set `result_symbol = INTERPOLATION` and return.
   - Guard each branch on `valid_symbols`: only produce `IDENTIFIER` when
     `valid_symbols[IDENTIFIER]`, only `INTERPOLATION` when
     `valid_symbols[INTERPOLATION]`.
3. Else if `is_word_start(lookahead)`: read the first word; if reserved,
   `return false`; otherwise this is an `IDENTIFIER` segment (existing loop,
   including embedded `$(...)` continuation units).
4. Else `return false`.

The serialize/deserialize functions remain no-ops (the scanner is still
stateless — it carries no cross-token state).

### 3.3 `highlights.scm` corrections and enrichment

Concrete problems in v1 and their fixes:

| Problem (v1) | Fix (v2) |
|---|---|
| `<` and `>` appear in **both** the `@operator` list and the `@punctuation.bracket` list, so the *last* matching pattern wins arbitrarily and type-annotation angle brackets are styled as relational operators. | Drop `<`/`>` from `@punctuation.bracket`; keep them in `@operator`. Capture the angle brackets of a `type_annotation` explicitly as `@punctuation.bracket` so context decides. |
| All keywords share one `@keyword`. | Split: `if`/`else` → `@keyword.conditional`; `when`/`is` → `@keyword.conditional`; `expand`/`to` → `@keyword.repeat`; `local`/`static` → `@keyword`. |
| `(boolean) @boolean` only. | Keep `@boolean`; additionally capture the boolean *words* `true`/`false` are already one node, so no change needed beyond confirming the node-level capture. Add `@constant.builtin` is **not** added (no builtin constants exist). |
| Hex/number undifferentiated. | Keep `(number) @number` (a single node; no sub-typing in the grammar — splitting hex needs a grammar token split, deferred to v3). |
| `expand` loop variable styled as a plain `@variable`. | Capture `(expand_statement variable: (identifier) @variable.parameter)`. |
| No interpolation highlight. | `(interpolation) @constant.macro` and, where it can be matched, the inner text. |
| `@function` vs `@function.method` is fine but inconsistent with nvim's preferred `@function.call`. | Use `@function.call` and `@function.method.call` (the nvim-treesitter standard capture names) so highlighting maps to real highlight groups. |

`(identifier) @variable` stays as the final, lowest-priority catch-all.

### 3.4 New `locals.scm`

Neovim's locals module (and editors generally) use `@local.scope`,
`@local.definition.*`, and `@local.reference` to drive scope-aware
highlighting and rename. v2 ships a `locals.scm` that needs only CST structure:

- `(block) @local.scope` and `(source_file) @local.scope` — scope nodes.
- `(local_declaration name: (identifier) @local.definition.var)` — a `local`
  binding.
- `(expand_statement variable: (identifier) @local.definition.var)` — the
  compile-time loop variable.
- `(identifier) @local.reference` — every other identifier is a reference;
  the locals engine resolves it against the nearest enclosing definition.

This is intentionally minimal and correct; channel/parameter resolution that
needs the `.m1prj` symbol model stays in `m1-core`/`m1-typecheck` and is **not**
attempted here.

### 3.5 Corpus expansion

`test/corpus/` grows from one file to four, all in the existing bare
S-expression format (no positions, no field names) so they run under
`tree-sitter test` and are regenerable with `--update`:

- `test/corpus/constructs.txt` — **unchanged** (the v1 6 cases).
- `test/corpus/interpolation.txt` — standalone interpolation operand, embedded
  interpolation in a name (stays one `identifier`), interpolation in an
  `expand` body.
- `test/corpus/scanner_edges.txt` — name immediately followed by `eq`/`and`,
  trailing space before `.`, pathological long multi-word name, digit-led
  continuation word.
- `test/corpus/errors.txt` — malformed inputs that must produce a stable
  `(ERROR)` shape: missing right-hand side (`x = ;`), unterminated call. These
  lock in the recovery behaviour `m1-core` surfaces as syntax diagnostics.
- `test/corpus/literals.txt` — hex with `u` suffix, float with exponent,
  string, line + block comment placement.

The exact expected S-expressions are produced by `tree-sitter test --update`
against the regenerated parser and pasted verbatim into the plan tasks — no
hand-authored trees.

### 3.6 Rust binding constants

`bindings/rust/lib.rs` gains:

```rust
/// Scope/definition/reference query source (`queries/locals.scm`).
pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");
/// Language-injection query source (`queries/injections.scm`).
pub const INJECTIONS_QUERY: &str = include_str!("../../queries/injections.scm");
```

plus a unit test asserting each loads as a valid `tree_sitter::Query` against
`LANGUAGE` (this also guards the new highlight captures from typos).

### 3.7 Build reality (must be in the plan)

Any change to `grammar.js` or `src/scanner.c` requires:

```bash
npx tree-sitter generate    # regenerates src/parser.c AND src/node-types.json
npx tree-sitter test        # runs the corpus
```

`src/parser.c` and `src/node-types.json` are **committed** (the Rust crate
builds without the CLI). The new `interpolation` node changes
`node-types.json`, so `m1-core`'s generated `Kind` enum will gain a
`Interpolation` variant. **This is a follow-up sync in `m1-core`, performed in
that repo — never edited from here.** Likewise `.nvim-test/queries/m1/` holds a
*copy* of the queries and must be re-synced (and gain `indents.scm`, which it is
currently missing). Both are recorded as follow-up notes in the plan, not as
tasks that touch sibling repos.

---

## 4. Architecture

### 4.1 Files changed / added

```
grammar.js                       (add interpolation rule + external; add to _expression)
src/scanner.c                    (add INTERPOLATION token + first-unit dispatch)
src/parser.c                     (REGENERATED — committed)
src/node-types.json              (REGENERATED — committed; gains "interpolation")
queries/highlights.scm           (rewrite: fixes + new captures)
queries/locals.scm               (NEW)
queries/injections.scm           (unchanged; now also exposed as a Rust const)
bindings/rust/lib.rs             (add LOCALS_QUERY, INJECTIONS_QUERY + query-loads test)
test/corpus/interpolation.txt    (NEW)
test/corpus/scanner_edges.txt    (NEW)
test/corpus/errors.txt           (NEW)
test/corpus/literals.txt         (NEW)
README.md                        (document interpolation node + locals query)
PLAN.md / STATUS.md              (mark v2 items done; record sync follow-ups)
```

### 4.2 Node-type contract (what `m1-core` sees change)

The only **new** node kind is `interpolation`. No existing kind is renamed or
removed, so `m1-core`'s generated `Kind` enum gains exactly one variant
(`Interpolation`) and its freshness test will flag the regeneration — that is
the intended sync signal.

---

## 5. Testing Strategy

- **Corpus tests** (`tree-sitter test`): the four new files plus the unchanged
  `constructs.txt`. Each case is `input` + the verbatim regenerated
  S-expression. The plan shows the exact expected trees.
- **Acceptance gate** (`scripts/check-corpus.sh`): must still report
  `0 with ERROR/MISSING nodes` over the EV-M1 corpus after the scanner change
  — the interpolation split must not regress real-script parsing.
- **Rust query-load test** (`bindings/rust/lib.rs`): `LANGUAGE` loads;
  `HIGHLIGHTS_QUERY`, `LOCALS_QUERY`, `INJECTIONS_QUERY` each compile as a
  `tree_sitter::Query` (catches capture/syntax typos at `cargo test`).
- **Manual smoke** for the new node:
  `printf 'x = $(SEG) + 1;\n' | npx tree-sitter parse /dev/stdin` must show
  `(interpolation)` in operand position while
  `printf 'naxID Bnk $(SEG) Vlim = 1;\n' | npx tree-sitter parse /dev/stdin`
  keeps a single `(identifier)` target.

---

## 6. Compatibility & Migration

- The grammar gains one node and one external token; the conflict set and
  precedence table are unchanged.
- `m1-core` sync (separate repo): re-vendor `src/parser.c`, `src/scanner.c`,
  `src/node-types.json`; re-run the `Kind` generator; expect one new
  `Interpolation` variant.
- `.nvim-test` sync (separate repo/dir): copy `queries/*.scm` (now including
  `locals.scm`) into `.nvim-test/queries/m1/` and add the missing
  `indents.scm`.

---

## 7. Non-Goals

- No semantic analysis (types, channel resolution, `.m1prj`): that is
  `m1-core` / `m1-typecheck`.
- No grammar-level error-recovery rules — v2 only *tests* the existing recovery
  shape so downstream diagnostics stay stable.
- No editor plugin changes beyond the documented query-sync note.

---

## 8. Deferred to v3

| Item | Reason |
|------|--------|
| `xor` / `mod` word-operators | No occurrence in the EV-M1 corpus; unconfirmed against the M1 Development Manual. Adding reserved words now risks breaking real names. |
| Function / scheduled-function definitions | The corpus is statement-body driver scripts; no function-definition syntax appears in `.m1scr` text. Confirm against the manual before grammaring. |
| Hex/number sub-typing (`@number.hex` etc.) | Needs a grammar token split (`number` → `hex_number` \| `float` \| `int`); cosmetic-only benefit, no consumer asking. |
| Grammar-level error recovery rules | v2 locks the current recovery shape via corpus tests; bespoke recovery rules add grammar complexity without a present need. |
| Tab-aware / width-aware `indents.scm` | Tab width is an editor config choice; current brace-based indents suffice. |
| Interpolation *inside* a name as its own sub-node | Would break the "one identifier = one segment" contract `m1-core` relies on; revisit only if a consumer needs the substructure. |
| `injections.scm` content (e.g. regex in strings) | M1 strings have no embedded sub-language today. |
