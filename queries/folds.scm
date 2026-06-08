; folds.scm — folding regions for M1 (.m1scr)

; `when_statement` is folded explicitly: its `{ ... }` body (the run of `is`
; clauses) is NOT a `block` node — the braces are inline tokens on
; `when_statement` itself — so without this the whole state machine could not be
; folded, only the individual `is` arm bodies.
[
  (block)
  (when_statement)
  (block_comment)
] @fold
