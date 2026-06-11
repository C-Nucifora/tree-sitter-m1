; textobjects.scm — nvim-treesitter-textobjects captures for M1 (.m1scr)
;
; M1 has no in-script function declarations (functions are declared in the
; .m1prj), so there are deliberately no @function captures. The `when … is`
; state machine is M1's multi-way conditional: the `when` is @conditional, each
; `is` arm is @block (its body is the natural "inner").

; ---- Conditionals -----------------------------------------------------------

(if_statement) @conditional.outer

(if_statement
  condition: (_) @conditional.inner)

(if_statement
  consequence: (block) @conditional.inner)

(else_clause
  (block) @conditional.inner)

(ternary_expression) @conditional.outer

(ternary_expression
  condition: (_) @conditional.inner)

(when_statement) @conditional.outer

(when_statement
  subject: (_) @conditional.inner)

; ---- Loops ------------------------------------------------------------------

; `expand (V = a to b) { … }` is M1's only loop-like construct (compile-time).
(expand_statement) @loop.outer

(expand_statement
  (block) @loop.inner)

; ---- Blocks -----------------------------------------------------------------

(block) @block.outer

(block
  (_) @block.inner)

; An `is` arm of a state machine: the whole arm is the outer object, its body
; block the inner.
(is_clause) @block.outer

(is_clause
  body: (block) @block.inner)

; ---- Assignments ------------------------------------------------------------

(assignment_statement) @assignment.outer

(assignment_statement
  target: (_) @assignment.lhs)

(assignment_statement
  value: (_) @assignment.inner @assignment.rhs)

(local_declaration) @assignment.outer

(local_declaration
  name: (identifier) @assignment.lhs)

(local_declaration
  value: (_) @assignment.inner @assignment.rhs)

; ---- Calls + arguments ------------------------------------------------------

(call_expression) @call.outer

(call_expression
  arguments: (argument_list) @call.inner)

(argument_list
  (_) @parameter.inner @parameter.outer)

; ---- Comments ---------------------------------------------------------------

[
  (line_comment)
  (block_comment)
] @comment.outer

; ---- Statements -------------------------------------------------------------

[
  (local_declaration)
  (assignment_statement)
  (expression_statement)
  (if_statement)
  (when_statement)
  (expand_statement)
  (empty_statement)
] @statement.outer
