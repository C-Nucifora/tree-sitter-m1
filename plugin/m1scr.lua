vim.filetype.add({ extension = { m1scr = 'm1scr' } })

local ok, parsers = pcall(require, 'nvim-treesitter.parsers')
if not ok then return end

local parser_config = parsers.get_parser_configs()
parser_config.m1scr = {
  install_info = {
    url = vim.fn.fnamemodify(debug.getinfo(1, 'S').source:sub(2), ':h:h'),
    files = { 'src/parser.c', 'src/scanner.c' },
    branch = 'main',
    generate_requires_npm = false,
    requires_generate_from_grammar = false,
  },
  filetype = 'm1scr',
}
