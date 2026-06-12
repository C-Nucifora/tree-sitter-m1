# tree-sitter-m1

A [tree-sitter](https://tree-sitter.github.io/) grammar for the **MoTeC M1
script language** (`.m1scr`) — the C#-like language used inside MoTeC M1 Build
to program M1-series ECUs (e.g. the M150).

This is the root of the M1 editor-tooling stack
([m1-tools](https://github.com/C-Nucifora/m1-tools) is the map of the
ecosystem). It produces the parser the Rust tools consume — `m1-core`
regenerates its node-kind enums from this grammar's `node-types.json`, so a
change here ripples through the whole toolchain — plus highlight / indent /
fold queries for editors. The committed `test/corpus` suite runs in CI; the
real-world corpus gate (`scripts/check-corpus.sh` — every script in the two
real corpora parses with zero `ERROR`/`MISSING` nodes) is run locally before
a grammar release, since those corpora are not available on public runners.

## The hard part: identifiers contain spaces

In M1, a path segment can contain spaces and `.` separates segments:

```
Brenloft.Quassor.Vund Klee.Mosko.Trilby Glonk = CanComms.GetUnsignedInteger(h, 48, 16);
Pellow.KVB Bonquil eq Wexlar Bonquil Mosko.Vor
```

A regex token can't express "a run of words, but stop before the keyword
`eq`". So the `identifier` token is produced by an **external scanner**
([`src/scanner.c`](src/scanner.c)) that joins `word SPACE word` greedily while
refusing to absorb reserved words. The reserved set there must stay in sync
with the keyword strings in [`grammar.js`](grammar.js).

Compile-time interpolation is the other subtlety: a standalone `$(VAR)`
operand parses as its own `interpolation` node, but when `$(VAR)` appears
inside a multi-word name it stays part of the `identifier` segment —
preserving "one identifier = one path segment".

> All example identifiers in this repo (grammar comments, corpus tests, docs)
> are synthetic placeholders, not drawn from any real project.

## Develop

```sh
npm install                 # gets the tree-sitter CLI locally
npx tree-sitter generate    # regenerate src/parser.c from grammar.js
npx tree-sitter test        # corpus tests
scripts/check-corpus.sh DIR # parse a real corpus dir; fail on ERROR/MISSING
```

`src/parser.c` is generated but **committed**, so the Rust crate builds
without the tree-sitter CLI — regenerate and commit it together with any
`grammar.js` change.

## Use as a Rust dependency

Consumed via a versioned git tag (the whole toolchain uses this scheme). Pin
the [latest release](https://github.com/C-Nucifora/tree-sitter-m1/releases):

```toml
[dependencies]
tree-sitter = "0.25"
tree-sitter-m1 = { git = "https://github.com/C-Nucifora/tree-sitter-m1.git", tag = "vX.Y.Z" }
```

```rust
let mut parser = tree_sitter::Parser::new();
parser.set_language(&tree_sitter_m1::LANGUAGE.into())?;
```

(Most Rust users want [m1-core](https://github.com/C-Nucifora/m1-core)'s
typed API instead of the raw grammar.)

## Neovim

Use the unified [nvim-m1](https://github.com/C-Nucifora/nvim-m1) plugin,
which wires this grammar together with the language server, formatter, and
linter behind a single `setup` call. To install **only the grammar**
(highlighting/indent/folds without the rest of the toolchain), register the
parser with nvim-treesitter pointing at this repo with
`files = { "src/parser.c", "src/scanner.c" }` — the external scanner is
required.

## Fuzzing

A libFuzzer harness (`fuzz/`) drives arbitrary bytes through the parser and
scanner, and asserts incremental parses match fresh parses after random
edits, under AddressSanitizer. CI runs a smoke pass on grammar/scanner
changes plus a weekly run.

## License

GPL-3.0-or-later — see [LICENSE](LICENSE).

## Trademark

Independent, community-built open-source tooling for the MoTeC® M1 script
language. Not affiliated with, authorised, or endorsed by MoTeC Pty Ltd.
"MoTeC" and "M1" are trademarks of MoTeC Pty Ltd.
