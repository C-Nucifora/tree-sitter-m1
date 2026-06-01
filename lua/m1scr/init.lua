-- Registers the m1scr filetype and the nvim-treesitter parser so that
-- TSInstall m1 (or TSInstall! from m1-build.lua) can compile the grammar.
local M = {}

function M.setup()
  -- 1. Filetype detection: *.m1scr -> "m1scr"
  vim.filetype.add({ extension = { m1scr = "m1scr" } })

  -- 1b. Map the "m1scr" filetype to the "m1" tree-sitter language. nvim-treesitter
  --     resolves parsers by language name ("m1"), not the `filetype` metadata
  --     field, so without this the parser never auto-attaches to m1scr buffers
  --     and `:TSBufEnable highlight` silently fails. Guarded for older Neovim
  --     that lacks the register API.
  if
    vim.treesitter
    and vim.treesitter.language
    and vim.treesitter.language.register
  then
    pcall(vim.treesitter.language.register, "m1", "m1scr")
  end

  -- 2. nvim-treesitter parser registration.
  --    The grammar name is "m1" (matches grammar.js `name: "m1"`).
  --    We point install_info at the GitHub repo so TSInstall fetches and
  --    compiles parser.c + scanner.c.
  local ok, parsers = pcall(require, "nvim-treesitter.parsers")
  if not ok then
    return
  end

  -- nvim-treesitter ≥ rewrite: require("nvim-treesitter.parsers") IS the
  -- config table. Legacy builds expose get_parser_configs() instead.
  local parser_config
  if type(parsers.get_parser_configs) == "function" then
    parser_config = parsers.get_parser_configs()
  else
    parser_config = parsers
  end
  if not parser_config or parser_config.m1 then
    return -- already registered or API unavailable (idempotent)
  end

  -- Locate this plugin's own directory so we can point at the local source
  -- when developing; fall back to the GitHub URL for end users.
  local plugin_dir = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":h:h:h")
  local local_src = plugin_dir .. "/src/parser.c"
  local use_local = vim.fn.filereadable(local_src) == 1

  parser_config.m1 = {
    install_info = use_local and {
      url = plugin_dir,
      files = { "src/parser.c", "src/scanner.c" },
      generate = false, -- grammar.js already compiled; don't re-run tree-sitter
    } or {
      url = "https://github.com/C-Nucifora/tree-sitter-m1",
      files = { "src/parser.c", "src/scanner.c" },
      branch = "main",
    },
    filetype = "m1scr", -- the Neovim filetype that uses this parser
  }
end

return M
