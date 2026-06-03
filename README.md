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

## Layout

| Path | Purpose |
|---|---|
| `grammar.js` | Grammar definition |
| `src/scanner.c` | External scanner (space-joined identifiers) |
| `src/parser.c` | Generated parser (committed so the Rust crate builds without the CLI) |
| `queries/*.scm` | highlight / indent / fold / injection queries |
| `bindings/rust/` | Rust crate exposing `LANGUAGE` + query strings |
| `test/corpus/` | tree-sitter corpus tests covering the grammar's constructs |

## The toolchain workspace

The M1 toolchain lives in **six separate repositories** that depend on each other
through **versioned git-tag** Cargo dependencies. They are not published to
crates.io, but each crate still builds from a standalone single-repo clone —
Cargo fetches its upstreams from their tagged releases. Cloning the whole set as
siblings under one parent directory is handy for cross-repo work, not required to
build:

```
<parent>/
├── tree-sitter-m1/   # grammar (root of the dependency graph) — this repo
├── m1-core/          # parse / CST / diagnostics; depends on tree-sitter-m1
├── m1-lint/          # linter;          depends on m1-core
├── m1-fmt/           # formatter;       depends on m1-core
├── m1-typecheck/     # type checker;    depends on m1-core
└── m1-lsp/           # language server; depends on the four above
```

`tree-sitter-m1` is the **root**: it has no sibling dependencies and builds on its
own. All five Rust crates depend on it (directly or transitively) via a
**versioned git-tag** Cargo dep, e.g.
`tree-sitter-m1 = { git = "https://github.com/C-Nucifora/tree-sitter-m1.git", tag = "v0.3.0" }`.
In particular `m1-core` regenerates its `Kind` enum from this crate's
`node-types.json`, so a grammar change here ripples downstream.

Because every consumer pins this crate by tag, the coupling **is** visible on
GitHub — each `Cargo.toml` names the tag it depends on, and Dependabot opens bump
PRs as new tags ship. Cutting a new release here and bumping `tag = "vX.Y.Z"` in
the consumers is what propagates a grammar change across the stack. The
`m1-example` example project (used by some corpus tests) is an optional sibling
checkout.

## Develop

```bash
npm install                 # gets the tree-sitter CLI locally
npx tree-sitter generate    # regenerate src/parser.c from grammar.js
npx tree-sitter test        # run corpus tests
# parse a real script from the sibling m1-example repo:
npx tree-sitter parse "../m1-example/UQR-EV/01.00/Scripts/CAN.DBC Init.m1scr"
```

Use as a Rust dependency:

```toml
[dependencies]
tree-sitter = "0.25"
tree-sitter-m1 = { git = "https://github.com/C-Nucifora/tree-sitter-m1.git", tag = "v0.3.0" }
```

```rust
let mut parser = tree_sitter::Parser::new();
parser.set_language(&tree_sitter_m1::LANGUAGE.into())?;
```

## Neovim setup

Register the parser config before installing so nvim-treesitter recognises the language (avoids the `skipping unsupported language: m1` warning):

```lua
{
    "C-Nucifora/tree-sitter-m1",
    dependencies = { "nvim-treesitter/nvim-treesitter" },
    config = function()
        -- Map the `.m1scr` filetype to the `m1` tree-sitter language.
        vim.filetype.add({ extension = { m1scr = "m1scr" } })
        vim.treesitter.language.register("m1", "m1scr")

        -- Register the parser. The nvim-treesitter rewrite returns the config
        -- table directly; legacy builds expose get_parser_configs().
        local parsers = require("nvim-treesitter.parsers")
        local parser_config = type(parsers.get_parser_configs) == "function"
                and parsers.get_parser_configs()
            or parsers
        parser_config.m1 = {
            install_info = {
                url = "https://github.com/C-Nucifora/tree-sitter-m1",
                -- The grammar has an external scanner, so scanner.c is required.
                files = { "src/parser.c", "src/scanner.c" },
                branch = "main",
            },
            filetype = "m1scr",
        }
        vim.cmd("TSInstall! m1")
    end,
}
```

## Status

First-pass grammar covering the constructs seen across the m1-example corpus. Known
gaps and next steps are in [`PLAN.md`](PLAN.md).

## License

Licensed under the GNU General Public License v3.0 or later (GPL-3.0-or-later) — see [LICENSE](LICENSE).

Copyright (C) 2026 The M1 Tools authors.

## Trademark

Independent, community-built open-source tooling for the MoTeC® M1 script
language. Not affiliated with, authorised, or endorsed by MoTeC Pty Ltd.
"MoTeC" and "M1" are trademarks of MoTeC Pty Ltd.
