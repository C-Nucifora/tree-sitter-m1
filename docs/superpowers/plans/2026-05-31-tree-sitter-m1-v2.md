# tree-sitter-m1 v2 — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Grammar work follows: edit `grammar.js`/`scanner.c` → `npx tree-sitter generate` → add/regenerate corpus case → `npx tree-sitter test` → commit.

**Goal:** Add a standalone `interpolation` node for operand-position `$(VAR)`, a corrected and enriched `highlights.scm`, a new `locals.scm`, two new Rust query constants, and a broadened corpus (interpolation, scanner edges, error recovery, literals) — without regressing the m1-example acceptance gate.

**Architecture:** `grammar.js` gains an `interpolation` rule and a second external token; `src/scanner.c` dispatches on the first unit so a lone `$(...)` becomes `INTERPOLATION` while a name-leading `$(...)` stays inside the `identifier` segment (segment cohesion is load-bearing for `m1-core`). `src/parser.c` and `src/node-types.json` are regenerated and committed. Queries are corrected/added; the Rust binding exposes `LOCALS_QUERY`/`INJECTIONS_QUERY` and gains a query-load test. Corpus tests lock the behaviour. The only new node kind is `interpolation`.

**Tech Stack:** tree-sitter CLI 0.25 (`tree-sitter-cli` devDependency, run via `npx`), C external scanner, Rust binding crate (`tree-sitter` 0.25 dev-dep, `cc` build-dep).

**Spec:** `docs/superpowers/specs/2026-05-31-tree-sitter-m1-v2-design.md`

> **Note:** All example identifiers below — `Foo Bar`, `Baz Qux`, `naxID Bnk $(SEG) Vlim`, `QZP MV31 R7 TKN 5X Glim Bront`, `Cralbo.QHM.Dorn`, etc. — are **synthetic placeholders**, not from any real project. v1 already carries the obfuscation disclaimer in `grammar.js`, `src/scanner.c`, and `README.md`; preserve it in every file this plan edits, and never paste real m1-example content.

**Prerequisites:** `npm install` done (the `tree-sitter` CLI resolves via `npx`); `npx tree-sitter --version` ≥ 0.25; on `main`, working tree clean; v1 corpus green (`npx tree-sitter test` = 6/6). Commit directly on `main` with plain `git commit` (signing is automatic; pass no signing flags; no AI attribution).

---

## File structure (after v2)

| File | Change |
|------|--------|
| `grammar.js` | add `interpolation` rule; `externals` gains `$.interpolation`; `_expression` gains `$.interpolation` |
| `src/scanner.c` | add `INTERPOLATION` token; first-unit dispatch |
| `src/parser.c` | REGENERATED (committed) |
| `src/node-types.json` | REGENERATED (committed; gains `"interpolation"`) |
| `queries/highlights.scm` | rewrite per spec §3.3 |
| `queries/locals.scm` | NEW |
| `bindings/rust/lib.rs` | add `LOCALS_QUERY`, `INJECTIONS_QUERY`; query-load test |
| `test/corpus/interpolation.txt` | NEW |
| `test/corpus/scanner_edges.txt` | NEW |
| `test/corpus/errors.txt` | NEW |
| `test/corpus/literals.txt` | NEW |
| `README.md` | document the interpolation node + locals query |
| `PLAN.md`, `STATUS.md` | mark v2 done; record sync follow-ups |

---

## Task 0: Baseline verification

**Files:** none (read-only).

- [ ] **Step 1:** confirm clean tree on `main`:

```bash
git -C . status --short && git -C . branch --show-current
```

Expect: empty status, `main`.

- [ ] **Step 2:** confirm v1 corpus is green and record the baseline node count:

```bash
npx tree-sitter test
grep -c '"type"' src/node-types.json
```

Expect: `Total parses: 6; successful parses: 6` and a node-type count to compare against after regeneration (v2 adds exactly one top-level `"type": "interpolation"`).

- [ ] **Step 3:** confirm the acceptance gate is green (skip gracefully if the m1-example corpus is absent):

```bash
[ -d ../m1-example/UQR-EV/01.00/Scripts ] && bash scripts/check-corpus.sh || echo "corpus absent — gate deferred to Task 6"
```

Expect: `parsed 80 scripts; 0 with ERROR/MISSING nodes` (or the absent note).

No commit (read-only task).

---

## Task 1: Add the `interpolation` grammar rule + external token

**File:** `grammar.js`.

- [ ] **Step 1:** extend `externals` to declare the second token (order matters — it must match the scanner's enum):

Replace:

```js
  // The space-joined path segment is produced by the external scanner.
  externals: ($) => [$.identifier],
```

with:

```js
  // The space-joined path segment and the standalone `$(VAR)` interpolation are
  // both produced by the external scanner. ORDER MUST MATCH the TokenType enum
  // in src/scanner.c (IDENTIFIER, INTERPOLATION).
  externals: ($) => [$.identifier, $.interpolation],
```

- [ ] **Step 2:** add `$.interpolation` to the `_expression` choice. Replace:

```js
    _expression: ($) =>
      choice(
        $.identifier,
        $.member_expression,
```

with:

```js
    _expression: ($) =>
      choice(
        $.identifier,
        $.interpolation,
        $.member_expression,
```

- [ ] **Step 3:** add **no** `rules` entry for `interpolation`. An external
  token is named entirely by its position in the `externals` array (Step 1) and
  is used by referencing `$.interpolation` in another rule (Step 2). Adding a
  `rules: { interpolation: ... }` body is an error for an external symbol —
  exactly as `identifier` has no `rules` entry today. (Verified: a grammar that
  declares `externals: [$.identifier, $.interp]` and references `$.interp` from
  a rule, with no `interp` rule body, generates cleanly.) Document the node in
  the existing `// ---- Tokens ----` comment band instead — append this comment
  line near the top of the `rules` block (it is documentation only, not a rule):

```js
    // NOTE: `interpolation` ($(VAR) as a standalone operand, e.g. `x = $(SEG)+1`)
    // is an external token produced by src/scanner.c — it has no rule body here,
    // only the `externals` declaration above and the `_expression` reference.
    // A `$(VAR)` that leads a multi-word name (`$(SEG) Vlim ...`) folds into the
    // `identifier` segment instead, preserving "one identifier = one path
    // segment". (Example names are synthetic placeholders, not from any project.)
```

- [ ] **Step 4:** generate and confirm no grammar error:

```bash
npx tree-sitter generate
```

Expect: exits 0; `src/parser.c` and `src/node-types.json` rewritten, with a new
top-level `"type": "interpolation"` in `node-types.json`. The external
declaration in Step 1 plus the `_expression` reference in Step 2 are sufficient
to name the node — no `rules` entry is needed (or allowed).

- [ ] **Step 5:** do NOT commit yet — the scanner (Task 2) must emit the token
  before the parser is usable. (Generation alone is harmless; the next task
  completes the pair.)

---

## Task 2: Emit `INTERPOLATION` from the external scanner

**File:** `src/scanner.c`.

- [ ] **Step 1:** extend the token enum (order must match `externals`):

Replace:

```c
enum TokenType { IDENTIFIER };
```

with:

```c
enum TokenType { IDENTIFIER, INTERPOLATION };
```

- [ ] **Step 2:** rewrite `tree_sitter_m1_external_scanner_scan` so the **first
  unit** chooses the token. Replace the whole function body from the
  `valid_symbols[IDENTIFIER]` guard down to the final `return true;` with:

```c
bool tree_sitter_m1_external_scanner_scan(void *payload, TSLexer *lexer,
                                          const bool *valid_symbols) {
  (void)payload;
  if (!valid_symbols[IDENTIFIER] && !valid_symbols[INTERPOLATION]) {
    return false;
  }

  /* Skip leading whitespace; it is never part of the token. */
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t' ||
         lexer->lookahead == '\n' || lexer->lookahead == '\r') {
    lexer->advance(lexer, true);
  }

  bool leads_with_interp = false;

  /* First unit: a `$(...)` interpolation or a word. */
  if (lexer->lookahead == '$') {
    if (!try_read_interp(lexer)) {
      return false;
    }
    leads_with_interp = true;
  } else if (is_word_start(lexer->lookahead)) {
    char word[64];
    read_word(lexer, word, sizeof(word));
    if (is_reserved(word)) {
      return false; /* let the grammar match the keyword */
    }
  } else {
    return false;
  }
  lexer->mark_end(lexer); /* token currently ends after the first unit */

  /* A standalone `$(...)` operand becomes INTERPOLATION unless it is the head of
   * a multi-word name (a space followed by another unit). Peek without
   * committing: if no continuation unit follows, emit INTERPOLATION. */
  if (leads_with_interp) {
    if (lexer->lookahead != ' ') {
      if (!valid_symbols[INTERPOLATION]) {
        return false;
      }
      lexer->result_symbol = INTERPOLATION;
      return true;
    }
    /* Space follows: fall through and treat as the head of an identifier
     * segment (existing continuation behaviour). */
    if (!valid_symbols[IDENTIFIER]) {
      return false;
    }
  } else if (!valid_symbols[IDENTIFIER]) {
    return false;
  }

  /* Extend with " <unit>" while the next unit is not a reserved word. A
   * continuation word may begin with a digit ("XV Glim 4", "Glonk 9"); a unit
   * may also be a `$(...)` interpolation ("naxID Bnk $(SEG) Vlim $(NODE)"). */
  for (;;) {
    if (lexer->lookahead != ' ') {
      break;
    }
    lexer->advance(lexer, false); /* tentatively consume the space */
    if (lexer->lookahead == '$') {
      if (!try_read_interp(lexer)) {
        break;
      }
    } else if (is_word_char(lexer->lookahead)) {
      char next[64];
      read_word(lexer, next, sizeof(next));
      if (is_reserved(next)) {
        break; /* leave the consumed " <kw>" as look-ahead via mark_end */
      }
    } else {
      break;
    }
    lexer->mark_end(lexer); /* commit this unit into the identifier */
  }

  lexer->result_symbol = IDENTIFIER;
  return true;
}
```

> The leading comment block at the top of `scanner.c` already explains the
> space-joined-segment design and carries the synthetic-placeholder disclaimer;
> add one sentence to it noting that a standalone `$(...)` now yields the
> separate `INTERPOLATION` token.

- [ ] **Step 3:** append to the top-of-file comment (after the existing
  disclaimer line) a note:

```c
 * v2: a standalone `$(...)` operand (e.g. `x = $(SEG) + 1`) is emitted as a
 * separate INTERPOLATION token; a `$(...)` that leads a multi-word name folds
 * into the IDENTIFIER segment as before.
```

- [ ] **Step 4:** regenerate and smoke-test the new node:

```bash
npx tree-sitter generate
printf 'x = $(SEG) + 1;\n' | npx tree-sitter parse /dev/stdin
printf 'naxID Bnk $(SEG) Vlim = 1;\n' | npx tree-sitter parse /dev/stdin
```

Expect (standalone): the `value` is a `binary_expression` whose left child is
`(interpolation ...)` and right is `(number ...)`. Expect (embedded): the
assignment `target` is a single `(identifier ...)` spanning `naxID Bnk $(SEG) Vlim`.

- [ ] **Step 5:** confirm the v1 corpus still passes (the `expand` interpolation
  case uses embedded `$(SEG)` and must stay an `identifier`):

```bash
npx tree-sitter test
```

Expect: `6 successful parses`. If the `expand loop with interpolation in names`
case now differs, STOP — the embedded-interpolation cohesion regressed; revisit
the `leads_with_interp` space-peek in Step 2.

- [ ] **Step 6: Commit** (parser.c + node-types.json are regenerated artifacts; commit them with the source):

```bash
git add grammar.js src/scanner.c src/parser.c src/node-types.json
git commit -m "feat(grammar): standalone interpolation node for operand-position \$(VAR)"
```

---

## Task 3: Corpus — interpolation cases

**File:** `test/corpus/interpolation.txt` (NEW).

- [ ] **Step 1:** create the file with these three cases (bare S-expression
  format, matching `constructs.txt`; all names synthetic):

```
================================================================================
standalone interpolation operand
================================================================================

x = $(SEG) + 1;

--------------------------------------------------------------------------------

(source_file
  (assignment_statement
    (identifier)
    (binary_expression
      (interpolation)
      (number))))

================================================================================
embedded interpolation stays one identifier
================================================================================

naxID Bnk $(SEG) Vlim = 1;

--------------------------------------------------------------------------------

(source_file
  (assignment_statement
    (identifier)
    (number)))

================================================================================
interpolation inside expand body
================================================================================

expand (SEG = 1 to 6)
{
	Klumph $(SEG).Glonk 4 = $(SEG) + 1;
}

--------------------------------------------------------------------------------

(source_file
  (expand_statement
    (identifier)
    (number)
    (number)
    (block
      (assignment_statement
        (member_expression
          (identifier)
          (identifier))
        (binary_expression
          (interpolation)
          (number))))))
```

- [ ] **Step 2:** run only this file and reconcile with the parser. If the
  generated tree differs, regenerate the expected block from the parser rather
  than hand-editing:

```bash
npx tree-sitter test --file-name interpolation.txt
# if a case fails because the expected tree is stale, update it from the parser:
npx tree-sitter test --file-name interpolation.txt --update
git diff test/corpus/interpolation.txt   # review that the update matches the spec intent
```

Expect: 3 passing cases. The `--update` is only a convenience to capture the
*exact* tree; the shapes above are the intended result and must not drift from
spec §3.1 (standalone → `(interpolation)`, embedded → one `(identifier)`).

- [ ] **Step 3: Commit**

```bash
git add test/corpus/interpolation.txt
git commit -m "test(corpus): interpolation operand vs embedded-name cases"
```

---

## Task 4: Corpus — scanner edge cases

**File:** `test/corpus/scanner_edges.txt` (NEW).

- [ ] **Step 1:** create the file (all names synthetic placeholders):

```
================================================================================
name immediately followed by eq keyword
================================================================================

Foo Bar eq Baz Qux;

--------------------------------------------------------------------------------

(source_file
  (expression_statement
    (binary_expression
      (identifier)
      (identifier))))

================================================================================
trailing space before dot separator
================================================================================

Foo Bar .Baz = 1;

--------------------------------------------------------------------------------

(source_file
  (assignment_statement
    (member_expression
      (identifier)
      (identifier))
    (number)))

================================================================================
pathological long multi-word name
================================================================================

QZP MV31 R7 TKN 5X Glim Bront = 1;

--------------------------------------------------------------------------------

(source_file
  (assignment_statement
    (identifier)
    (number)))

================================================================================
digit-led continuation word
================================================================================

Cralbo.QHM.Dorn.Plor.XV Glim 4 = Calculate.NAN();

--------------------------------------------------------------------------------

(source_file
  (assignment_statement
    (member_expression
      (member_expression
        (member_expression
          (member_expression
            (identifier)
            (identifier))
          (identifier))
        (identifier))
      (identifier))
    (call_expression
      (member_expression
        (identifier)
        (identifier))
      (argument_list))))
```

- [ ] **Step 2:** run and reconcile (use `--update` only to capture the exact
  tree if a case is stale, then verify it matches the intent above):

```bash
npx tree-sitter test --file-name scanner_edges.txt
```

Expect: 4 passing cases. The first case is the load-bearing one — the scanner
must NOT absorb `eq` into `Foo Bar`, so `eq` becomes a `binary_expression`
operator with `Foo Bar` left and `Baz Qux` right.

- [ ] **Step 3: Commit**

```bash
git add test/corpus/scanner_edges.txt
git commit -m "test(corpus): scanner edge cases (keyword boundary, dot, long names, digit-led)"
```

---

## Task 5: Corpus — error recovery + literals

**Files:** `test/corpus/errors.txt` (NEW), `test/corpus/literals.txt` (NEW).

- [ ] **Step 1:** create `test/corpus/errors.txt`. These lock the recovery shape
  that `m1-core` surfaces as syntax diagnostics. Use the `:error` header tag so
  `tree-sitter test` tolerates ERROR nodes:

```
==================
missing right-hand side
:error
==================

x = ;

---

(source_file
  (expression_statement
    (identifier)
    (ERROR)))

==================
unterminated call argument list
:error
==================

Foo(1, 2

---

(source_file
  (ERROR
    (identifier)
    (number)
    (number)))
```

> The `:error` tag (a line after the test name, before the closing `===`)
> tells `tree-sitter test` the case is expected to contain ERROR/MISSING nodes.
> If your CLI version does not support the tag, the case still passes as long as
> the pasted S-expression matches the parser output. Generate the **exact**
> error tree with `npx tree-sitter test --file-name errors.txt --update` and verify
> the shape is stable (an `(ERROR)` appears where the RHS / closing `)` is
> missing) before trusting it.

- [ ] **Step 2:** create `test/corpus/literals.txt`:

```
================================================================================
hex with unsigned suffix
================================================================================

local x = 0x1Fu;

--------------------------------------------------------------------------------

(source_file
  (local_declaration
    (identifier)
    (number)))

================================================================================
float with exponent
================================================================================

x = 1.5e-3;

--------------------------------------------------------------------------------

(source_file
  (assignment_statement
    (identifier)
    (number)))

================================================================================
string assignment
================================================================================

x = "hello";

--------------------------------------------------------------------------------

(source_file
  (assignment_statement
    (identifier)
    (string)))

================================================================================
line and block comments around a statement
================================================================================

// leading
x = 1; /* trailing */

--------------------------------------------------------------------------------

(source_file
  (line_comment)
  (assignment_statement
    (identifier)
    (number))
  (block_comment))
```

> Note: comments are `extras`, so they appear as siblings at the point they
> occur. The block comment after `x = 1;` attaches at `source_file` level. If
> the generated tree places it differently, regenerate with `--update` and keep
> the parser's actual placement — comments-as-extras placement is the parser's
> call, not ours.

- [ ] **Step 3:** run and reconcile both files:

```bash
npx tree-sitter test --file-name errors.txt
npx tree-sitter test --file-name literals.txt
```

Expect: errors = 2 passing, literals = 4 passing. For any literal case whose
extras placement differs, run `--update` for that file and re-verify.

- [ ] **Step 4: Commit**

```bash
git add test/corpus/errors.txt test/corpus/literals.txt
git commit -m "test(corpus): error-recovery and literal/comment edge cases"
```

---

## Task 6: Acceptance gate — m1-example corpus must stay clean

**Files:** none (verification).

- [ ] **Step 1:** run the full gate (the interpolation scanner change is the
  only behavioural risk to real scripts):

```bash
[ -d ../m1-example/UQR-EV/01.00/Scripts ] && bash scripts/check-corpus.sh || echo "corpus absent — gate skipped"
```

Expect: `parsed 80 scripts; 0 with ERROR/MISSING nodes` (or the absent note). If
any script now reports ERROR/MISSING, STOP and debug — the most likely cause is
a `$(...)`-leading name mis-split; re-check Task 2 Step 2's `leads_with_interp`
space-peek.

- [ ] **Step 2:** run the complete corpus suite:

```bash
npx tree-sitter test
```

Expect: all cases across `constructs`, `interpolation`, `scanner_edges`,
`errors`, `literals` pass.

No commit (verification only).

---

## Task 7: Rewrite `queries/highlights.scm`

**File:** `queries/highlights.scm`.

- [ ] **Step 1:** replace the whole file with the corrected, enriched query
  (fixes the `<`/`>` double-capture; splits keywords; adds interpolation and the
  `expand` parameter; uses nvim-standard `@function.call` names):

```scheme
; highlights.scm — syntax highlighting for M1 (.m1scr)
; (All identifiers referenced in comments are synthetic placeholders.)

; Keywords
[
  "local"
  "static"
] @keyword

[
  "if"
  "else"
  "when"
  "is"
] @keyword.conditional

[
  "expand"
  "to"
] @keyword.repeat

[
  "and"
  "or"
  "not"
  "eq"
  "neq"
] @keyword.operator

(boolean) @boolean

; Operators (note: `<` and `>` live here only; the angle brackets of a
; type_annotation are captured separately below as punctuation.bracket)
[
  "="
  "+="
  "-="
  "*="
  "/="
  "+"
  "-"
  "*"
  "/"
  "%"
  "<"
  ">"
  "<="
  ">="
  "=="
  "!="
  "&&"
  "||"
  "&"
  "|"
  "^"
  "<<"
  ">>"
  "!"
  "?"
  ":"
] @operator

; Punctuation
[ "(" ")" "{" "}" ] @punctuation.bracket
[ "." "," ";" ] @punctuation.delimiter

; Literals
(number) @number
(string) @string
(line_comment) @comment
(block_comment) @comment

; Compile-time interpolation: $(VAR)
(interpolation) @constant.macro

; Type annotation: local <Unsigned Integer> ...
; The angle brackets here are punctuation, not relational operators.
(type_annotation
  "<" @punctuation.bracket
  (identifier) @type
  ">" @punctuation.bracket)

; The expand loop variable is a parameter-like binding.
(expand_statement
  variable: (identifier) @variable.parameter)

; Calls: highlight the final property of the callee as a method, or the bare
; callee as a function.
(call_expression
  function: (member_expression property: (identifier) @function.method.call))
(call_expression
  function: (identifier) @function.call)

; A property after a `.` (channels, enum members, fields)
(member_expression property: (identifier) @property)

; Plain identifiers (channels/parameters/locals) — lowest priority catch-all.
(identifier) @variable
```

- [ ] **Step 2:** validate the query parses against the grammar (this catches
  capture-name typos and references to non-existent fields/nodes):

```bash
printf 'local <Unsigned Integer> h = 0x00;\nx = $(SEG) + 1;\nA.B(h);\n' > /tmp/m1_hl.m1scr
npx tree-sitter query queries/highlights.scm /tmp/m1_hl.m1scr
```

Expect: a list of `capture` lines including `@type` on `Unsigned Integer`,
`@punctuation.bracket` on `<`/`>`, `@constant.macro` on `$(SEG)`,
`@function.method.call` on `B`, and `@variable` on `h`/`x`/`A`. No
"invalid node type" / "invalid field" errors.

- [ ] **Step 3: Commit**

```bash
git add queries/highlights.scm
git commit -m "feat(queries): fix angle-bracket capture; add interpolation, keyword splits, function.call"
```

---

## Task 8: Add `queries/locals.scm`

**File:** `queries/locals.scm` (NEW).

- [ ] **Step 1:** create the file (CST-only scopes/definitions/references per
  spec §3.4):

```scheme
; locals.scm — scope-aware highlighting for M1 (.m1scr)
; Consumed by Neovim's nvim-treesitter locals module. CST-only: channel and
; parameter resolution that needs the .m1prj symbol model lives in m1-core.
; (All identifiers referenced in comments are synthetic placeholders.)

; Scopes
(source_file) @local.scope
(block) @local.scope

; Definitions
(local_declaration
  name: (identifier) @local.definition.var)

(expand_statement
  variable: (identifier) @local.definition.var)

; References — every other identifier; the locals engine resolves each against
; the nearest enclosing definition, falling back to "unbound" (a channel/param).
(identifier) @local.reference
```

- [ ] **Step 2:** validate it parses against the grammar:

```bash
printf 'local foo = 1;\nexpand (SEG = 1 to 2) { x = foo + $(SEG); }\n' > /tmp/m1_loc.m1scr
npx tree-sitter query queries/locals.scm /tmp/m1_loc.m1scr
```

Expect: `@local.scope` on `source_file` and the `block`,
`@local.definition.var` on `foo` and `SEG`, `@local.reference` on the other
identifiers. No query errors.

- [ ] **Step 3: Commit**

```bash
git add queries/locals.scm
git commit -m "feat(queries): add locals.scm (scopes + local/expand definitions)"
```

---

## Task 9: Expose `LOCALS_QUERY` / `INJECTIONS_QUERY` + query-load test

**File:** `bindings/rust/lib.rs`.

- [ ] **Step 1:** add the two constants after `FOLDS_QUERY`:

```rust
/// Scope/definition/reference query source (`queries/locals.scm`).
pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");
/// Language-injection query source (`queries/injections.scm`).
pub const INJECTIONS_QUERY: &str = include_str!("../../queries/injections.scm");
```

- [ ] **Step 2:** add a query-load test to the existing `mod tests` (this
  compiles every non-empty query against the real grammar, catching capture or
  node-name typos at `cargo test`):

```rust
    #[test]
    fn queries_compile_against_grammar() {
        let language: tree_sitter::Language = super::LANGUAGE.into();
        for (name, src) in [
            ("highlights", super::HIGHLIGHTS_QUERY),
            ("indents", super::INDENTS_QUERY),
            ("folds", super::FOLDS_QUERY),
            ("locals", super::LOCALS_QUERY),
        ] {
            tree_sitter::Query::new(&language, src)
                .unwrap_or_else(|e| panic!("query {name} failed to compile: {e}"));
        }
        // injections.scm is an empty placeholder; an empty query is still valid.
        tree_sitter::Query::new(&language, super::INJECTIONS_QUERY)
            .expect("injections query failed to compile");
    }
```

- [ ] **Step 3:** build and test the crate:

```bash
cargo test
```

Expect: `can_load_grammar`, `node_types_json_is_present`, and
`queries_compile_against_grammar` all pass. (This also reconfirms the
regenerated `parser.c`/`scanner.c` link and the `interpolation` node is
present in `node-types.json`.)

- [ ] **Step 4: Commit**

```bash
git add bindings/rust/lib.rs
git commit -m "feat(bindings): expose LOCALS_QUERY/INJECTIONS_QUERY; test queries compile"
```

---

## Task 10: Docs — README, PLAN, STATUS

**Files:** `README.md`, `PLAN.md`, `STATUS.md`.

- [ ] **Step 1 (`README.md`):** in the Layout table, add a `queries/locals.scm`
  mention is already covered by the `queries/*.scm` row — instead add a short
  subsection after "The hard part" documenting the interpolation node. Insert
  after the synthetic-placeholder Note block:

```markdown
## Compile-time interpolation: `$(VAR)`

A standalone `$(VAR)` used as an operand parses as its own `interpolation`
node:

```
x = $(SEG) + 1;   // value is (binary_expression (interpolation) (number))
```

When `$(VAR)` instead leads a multi-word name it stays inside the
`identifier` segment, preserving "one identifier = one path segment":

```
naxID Bnk $(SEG) Vlim = 1;   // target is a single (identifier)
```

(Example names are synthetic placeholders, not from any real project.)
```

- [ ] **Step 2 (`PLAN.md`):** move the now-done items out of TODO. Mark the
  scanner-validation TODO and the `locals.scm` editor-integration TODO as done:

  - Change `- [ ] Validate the scanner against pathological names ...` to
    `- [x] Validate the scanner against pathological names (covered by
    test/corpus/scanner_edges.txt) and standalone `$(VAR)` (now its own
    `interpolation` node; test/corpus/interpolation.txt).`
  - Change `- [ ] `locals.scm` for scope-aware highlighting ...` to
    `- [x] `locals.scm` shipped (CST-only scopes + local/expand definitions);
    channel/param resolution still pending m1-core scopes.`

- [ ] **Step 3 (`STATUS.md`):** update the `tree-sitter-m1` row to note v2 and
  the sync follow-ups. Append to the `tree-sitter-m1` cell:
  `v2: standalone `interpolation` node; enriched highlights + new locals.scm;
  corpus expanded (interpolation/scanner-edges/errors/literals). Follow-ups:
  m1-core must re-vendor parser.c/scanner.c/node-types.json and regenerate
  `Kind` (gains `Interpolation`); .nvim-test/queries/m1 must be re-copied and
  gain indents.scm.`

- [ ] **Step 4:** sanity-check no real m1-example content leaked into the docs:

```bash
grep -rniE 'm1-example|UQR' README.md PLAN.md STATUS.md docs/ | grep -viE 'synthetic|placeholder|corpus|sibling|M1_CORPUS|example project'
```

Expect: only benign references (paths to the corpus, the disclaimer). No real
channel/identifier names.

- [ ] **Step 5: Commit**

```bash
git add README.md PLAN.md STATUS.md
git commit -m "docs: document interpolation node + locals query; record v2 status and sync follow-ups"
```

---

## Task 11: Final verification

**Files:** none.

- [ ] **Step 1:** full grammar + binding verification:

```bash
npx tree-sitter generate           # idempotent: no diff expected
git status --short src/             # parser.c/node-types.json already committed; expect clean
npx tree-sitter test                # all corpus files green
cargo test                          # bindings: grammar loads, queries compile
```

Expect: `generate` produces no new diff; `tree-sitter test` all green; `cargo
test` all green.

- [ ] **Step 2:** re-run the acceptance gate (if corpus present):

```bash
[ -d ../m1-example/UQR-EV/01.00/Scripts ] && bash scripts/check-corpus.sh || echo "corpus absent"
```

Expect: `0 with ERROR/MISSING nodes`.

- [ ] **Step 3:** confirm the only new node kind is `interpolation`:

```bash
grep -o '"type": "[a-z_]*"' src/node-types.json | sort -u | grep interpolation
```

Expect: `"type": "interpolation"` present; no other unexpected new top-level
types versus the Task 0 baseline.

- [ ] **Step 4:** push:

```bash
git push origin main
```

No further commit.

---

## Downstream sync follow-ups (NOT part of this plan — do not edit sibling repos here)

After this plan lands and is pushed:

1. **`m1-core`** — re-vendor `src/parser.c`, `src/scanner.c`, and
   `src/node-types.json` from this repo, then re-run its `Kind` generator
   (`cargo run -p xtask -- gen-kinds`). Expect exactly one new variant,
   `Interpolation`. m1-core's `node-types` freshness test will flag the change
   until synced.
2. **`.nvim-test/queries/m1/`** — re-copy `queries/highlights.scm`,
   `queries/folds.scm`, `queries/injections.scm`, and the new
   `queries/locals.scm`; **add the currently-missing `indents.scm`** (copy
   `queries/indents.scm`). Re-run the Neovim `:M1TS` smoke check.

These are recorded here only so the next operator knows what to do; this plan
makes no changes outside `tree-sitter-m1`.

---

## Task summary

| # | Description | Key file(s) |
|---|-------------|-------------|
| 0 | Baseline verification | — (read-only) |
| 1 | `interpolation` grammar rule + external token | `grammar.js` |
| 2 | Emit `INTERPOLATION` from scanner; regenerate | `src/scanner.c`, `src/parser.c`, `src/node-types.json` |
| 3 | Corpus: interpolation cases | `test/corpus/interpolation.txt` |
| 4 | Corpus: scanner edge cases | `test/corpus/scanner_edges.txt` |
| 5 | Corpus: error recovery + literals | `test/corpus/errors.txt`, `test/corpus/literals.txt` |
| 6 | Acceptance gate (m1-example stays clean) | — (verification) |
| 7 | Rewrite `highlights.scm` | `queries/highlights.scm` |
| 8 | Add `locals.scm` | `queries/locals.scm` |
| 9 | `LOCALS_QUERY`/`INJECTIONS_QUERY` + query-load test | `bindings/rust/lib.rs` |
| 10 | Docs (README/PLAN/STATUS) | `README.md`, `PLAN.md`, `STATUS.md` |
| 11 | Final verification + push | — |

**Total: 12 tasks (0–11).**

---

## Deferred to v3

| Item | Reason |
|------|--------|
| `xor` / `mod` word-operators | No occurrence in the m1-example corpus; unconfirmed against the M1 Development Manual; adding reserved words risks breaking real multi-word names. |
| Function / scheduled-function definitions | The corpus is statement-body driver scripts; no function-definition syntax appears in `.m1scr` text. Confirm against the manual first. |
| Hex/number sub-typing (`@number.hex`, `@constant`) | Needs a grammar token split (`number` → `hex` \| `float` \| `int`); cosmetic-only, no consumer asking. |
| Grammar-level error-recovery rules | v2 only *locks* the current recovery shape via `test/corpus/errors.txt`; bespoke recovery adds grammar complexity without a present need. |
| Tab-aware / width-aware `indents.scm` | Tab width is an editor config choice; brace-based indents suffice today. |
| Interpolation *inside* a name as its own sub-node | Would break "one identifier = one path segment", which `m1-core` relies on. |
| `injections.scm` content | M1 strings embed no sub-language today. |
| Auto-syncing `m1-core` / `.nvim-test` from CI | A manual follow-up for now; automate once the toolchain has a release process. |
