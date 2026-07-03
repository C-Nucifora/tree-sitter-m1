vim.bo.commentstring = "// %s"
vim.bo.tabstop = 4
vim.bo.shiftwidth = 4
-- M1 indents with tabs (Development Manual, Code Layout & Format; the toolchain
-- defaults to tab indentation). Expanding to spaces makes m1-lint L010 flag
-- every indented line and m1-fmt rewrite the file, so keep hard tabs.
vim.bo.expandtab = false
