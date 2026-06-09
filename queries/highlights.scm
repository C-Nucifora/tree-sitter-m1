; highlights.scm — syntax highlighting for M1 (.m1scr)
; (All identifiers referenced in comments are synthetic placeholders.)

; Keywords
[
  "local"
  "static"
] @keyword

[
  "if"
  "else"
  "when"
] @keyword.conditional

; `is` lives inside `is_clause`; bare anonymous-string matching for a token
; nested under a named rule is unreliable across nvim-treesitter versions, so
; anchor it explicitly (#14).
(is_clause
  "is" @keyword.conditional)

[
  "expand"
  "to"
] @keyword.repeat

[
  "and"
  "or"
  "not"
  "eq"
  "neq"
] @keyword.operator

(boolean) @boolean

; Operators (note: `<` and `>` live here only; the angle brackets of a
; type_annotation are captured separately below as punctuation.bracket)
[
  "="
  "+="
  "-="
  "*="
  "/="
  "%="
  "&="
  "|="
  "^="
  "<<="
  ">>="
  "~"
  "+"
  "-"
  "*"
  "/"
  "%"
  "<"
  ">"
  "<="
  ">="
  "=="
  "!="
  "&&"
  "||"
  "&"
  "|"
  "^"
  "<<"
  ">>"
  "!"
  "?"
  ":"
] @operator

; Punctuation
[ "(" ")" "{" "}" ] @punctuation.bracket
[ "." "," ";" ] @punctuation.delimiter

; Literals
(number) @number
(string) @string
(line_comment) @comment
(block_comment) @comment

; Compile-time interpolation: $(VAR)
(interpolation) @constant.macro

; Type annotation: local <Unsigned Integer> ...
; The angle brackets here are punctuation, not relational operators.
(type_annotation
  "<" @punctuation.bracket
  (identifier) @type
  ">" @punctuation.bracket)

; The expand loop variable is a parameter-like binding.
(expand_statement
  variable: (identifier) @variable.parameter)

; Calls: highlight the final property of the callee as a method, or the bare
; callee as a function.
(call_expression
  function: (member_expression property: (identifier) @function.method.call))
(call_expression
  function: (identifier) @function.call)

; A property after a `.` (channels, enum members, fields)
(member_expression property: (identifier) @property)

; The left-most root of a member/channel path (e.g. `Vehicle` in
; `Vehicle.SBG.IMU.Gyro.Z`) is the object of the innermost member_expression;
; highlight it like the path's properties rather than a plain @variable.
(member_expression object: (identifier) @property)

; Reference keywords (manual pp.39–41): In/Out/Parent/Root/Library/This.
; Captured as @variable.builtin regardless of position (head of a member chain
; or bare use). These remain `identifier` nodes in the CST; the capture is
; query-only. The explicit priority (default is 100) makes this win over the
; same-node @property/@variable captures in every consumer, independent of
; pattern order — editors disagree on whether earlier or later patterns win.
((identifier) @variable.builtin
 (#any-of? @variable.builtin "In" "Out" "Parent" "Root" "Library" "This")
 (#set! priority 105))

; Plain identifiers (channels/parameters/locals) — lowest priority catch-all.
(identifier) @variable
