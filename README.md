# tree-sitter-m1

A [tree-sitter](https://tree-sitter.github.io/) grammar for the **MoTeC M1
script language** (`.m1scr`) — the C#-like language used inside MoTeC M1 Build
to program M1-series ECUs (e.g. the M150).

This is the **foundation** of the M1 editor-tooling stack. It produces:

- a parser the Rust tools consume (`m1-core`, `m1-lint`, `m1-fmt`,
  `m1-typecheck`, `m1-lsp`), and
- highlight / indent / fold queries for Neovim.

## The hard part: identifiers contain spaces

In M1, a path segment can contain spaces and `.` separates segments:

```
Brenloft.Quassor.Vund Klee.Mosko.Trilby Glonk = CanComms.GetUnsignedInteger(h, 48, 16);
Pellow.KVB Bonquil eq Wexlar Bonquil Mosko.Vor
```

> **Note:** All example identifiers in this repo (here, in the grammar/scanner
> comments, and in the corpus tests) are synthetic placeholders. The grammar and
> tests were anonymised; the names are not drawn from any real project.

A regex token can't express "a run of words, but stop before the keyword `eq`".
So the `identifier` token is produced by an **external scanner**
([`src/scanner.c`](src/scanner.c)) that joins `word SPACE word` greedily while
refusing to absorb reserved words. The reserved set there must stay in sync with
the keyword strings in [`grammar.js`](grammar.js).

## Layout

| Path | Purpose |
|---|---|
| `grammar.js` | Grammar definition |
| `src/scanner.c` | External scanner (space-joined identifiers) |
| `src/parser.c` | Generated parser (committed so the Rust crate builds without the CLI) |
| `queries/*.scm` | highlight / indent / fold / injection queries |
| `bindings/rust/` | Rust crate exposing `LANGUAGE` + query strings |
| `test/corpus/` | tree-sitter corpus tests covering the grammar's constructs |

## Develop

```bash
npm install                 # gets the tree-sitter CLI locally
npx tree-sitter generate    # regenerate src/parser.c from grammar.js
npx tree-sitter test        # run corpus tests
# parse a real script from the sibling EV-M1 repo:
npx tree-sitter parse "../EV-M1/UQR-EV/01.00/Scripts/CAN.DBC Init.m1scr"
```

Use as a Rust dependency:

```toml
[dependencies]
tree-sitter = "0.25"
tree-sitter-m1 = { path = "../tree-sitter-m1" }
```

```rust
let mut parser = tree_sitter::Parser::new();
parser.set_language(&tree_sitter_m1::LANGUAGE.into())?;
```

## Status

First-pass grammar covering the constructs seen across the EV-M1 corpus. Known
gaps and next steps are in [`PLAN.md`](PLAN.md).

## License

Not yet chosen — decided by the repository owner. Treated as proprietary until
then.
