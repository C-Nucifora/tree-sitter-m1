; indents.scm — indentation hints for M1 (.m1scr)
; Consumed by Neovim's nvim-treesitter indent module.

; `when_statement` is listed explicitly: the `{ ... }` wrapping its `is` clauses
; is not a `block` node (the braces are inline tokens on `when_statement`), so
; without this the opening `{` gives no `@indent.begin` while its matching `}`
; still fires `@indent.end`, leaving indentation unbalanced inside a `when`.
[
  (block)
  (when_statement)
  (argument_list)
  (parenthesized_expression)
] @indent.begin

[ "}" ")" ] @indent.branch

[ "}" ")" ] @indent.end
