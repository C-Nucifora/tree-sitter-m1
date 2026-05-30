# Neovim Setup

## lazy.nvim

Add the following spec to your lazy.nvim config:

```lua
{
  'C-Nucifora/tree-sitter-m1',
  dependencies = { 'nvim-treesitter/nvim-treesitter' },
  config = function()
    -- parser will be compiled on first :TSInstall m1scr or :TSUpdate
    vim.cmd('TSInstall! m1scr')
  end,
}
```

nvim-treesitter must be installed. It handles compiling the parser from the C sources included in this repo — no separate build step is required in the lazy.nvim spec.

On first load (or after running `:TSUpdate`), nvim-treesitter will compile the parser. After that, `.m1scr` files will have syntax highlighting, indentation, folding, and locals support automatically.

## Manual install (no plugin manager)

Clone the repo and add it to your `runtimepath`:

```lua
vim.opt.rtp:prepend('/path/to/tree-sitter-m1')
```

Then run `:TSInstall m1scr` once to compile the parser.
