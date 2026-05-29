; highlights.scm — syntax highlighting for M1 (.m1scr)

; Keywords
[
  "local"
  "static"
  "if"
  "else"
  "when"
  "is"
  "expand"
  "to"
] @keyword

[
  "and"
  "or"
  "not"
  "eq"
  "neq"
] @keyword.operator

(boolean) @boolean

; Operators
[
  "="
  "+="
  "-="
  "*="
  "/="
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
[ "(" ")" "{" "}" "<" ">" ] @punctuation.bracket
[ "." "," ";" ] @punctuation.delimiter

; Literals
(number) @number
(string) @string
(line_comment) @comment
(block_comment) @comment

; Type annotation: local <Unsigned Integer> ...
(type_annotation (identifier) @type)

; Calls: highlight the final property of the callee as a function
(call_expression
  function: (member_expression property: (identifier) @function.method))
(call_expression
  function: (identifier) @function)

; A property after a `.` (channels, enum members, fields)
(member_expression property: (identifier) @property)

; Plain identifiers (channels/parameters/locals)
(identifier) @variable
