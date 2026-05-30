/**
 * grammar.js — Tree-sitter grammar for the MoTeC M1 script language (.m1scr)
 *
 * The M1 language is the C#-like source language used inside MoTeC M1 Build to
 * program M1-series ECUs (e.g. the M150). Source files (.m1scr) are plain text.
 *
 * The single hardest feature of the language for a lexer is that identifiers
 * may contain spaces, and `.` is the path separator, e.g.
 *
 *     Brenloft.Quassor.Vund Klee.Mosko.Trilby Glonk
 *     QZP MV31 R7 TKN 5X Glim Bront.Plarq Volim
 *     Wexlar Bonquil Mosko.Vor
 *
 * (Example identifiers shown here are synthetic placeholders, not from any
 * real project; the grammar and corpus tests were anonymised.)
 *
 * A single path *segment* ("Vund Klee", "Trilby Glonk", "QZP MV31 R7 TKN 5X Glim
 * Bront") is therefore a run of space-separated words. We cannot express that
 * with a regex token without also swallowing the spaces that separate a name
 * from a following keyword/operator (e.g. `Pellow.KVB Bonquil eq ...`). So the
 * segment token is produced by an external scanner (see src/scanner.c) that
 * greedily joins words but refuses to absorb reserved words.
 *
 * This is a first-pass grammar covering the constructs observed across the
 * m1-example corpus. Gaps are tracked in PLAN.md.
 */

const PREC = {
  ternary: 1,
  or: 2,
  and: 3,
  bitwise_or: 4,
  bitwise_xor: 5,
  bitwise_and: 6,
  equality: 7,
  relational: 8,
  shift: 9,
  additive: 10,
  multiplicative: 11,
  unary: 12,
  call: 13,
  member: 14,
};

module.exports = grammar({
  name: "m1",

  // Whitespace and comments are insignificant between tokens.
  extras: ($) => [/\s/, $.line_comment, $.block_comment],

  // The space-joined path segment and the standalone `$(VAR)` interpolation are
  // both produced by the external scanner. ORDER MUST MATCH the TokenType enum
  // in src/scanner.c (IDENTIFIER, INTERPOLATION).
  externals: ($) => [$.identifier, $.interpolation],

  conflicts: ($) => [
    // `a.b` may be the target of an assignment or a standalone expression
    // statement; both share the expression prefix until the operator/`;` is
    // seen, so the parser must explore both reductions.
    [$.assignment_statement, $.expression_statement],
  ],

  rules: {
    // NOTE: `interpolation` ($(VAR) as a standalone operand, e.g. `x = $(SEG)+1`)
    // is an external token produced by src/scanner.c — it has no rule body here,
    // only the `externals` declaration above and the `_expression` reference.
    // A `$(VAR)` that leads a multi-word name (`$(SEG) Vlim ...`) folds into the
    // `identifier` segment instead, preserving "one identifier = one path
    // segment". (Example names are synthetic placeholders, not from any project.)
    source_file: ($) => repeat($._statement),

    // ---- Statements ---------------------------------------------------------
    _statement: ($) =>
      choice(
        $.local_declaration,
        $.assignment_statement,
        $.if_statement,
        $.when_statement,
        $.expand_statement,
        $.expression_statement,
        $.block,
        $.empty_statement,
      ),

    block: ($) => seq("{", repeat($._statement), "}"),

    empty_statement: (_) => ";",

    // local foo = expr;            (Hungarian-typed, type inferred from prefix)
    // local <Unsigned Integer> h = 0x00;   (explicitly typed)
    local_declaration: ($) =>
      seq(
        optional("static"),
        "local",
        optional($.type_annotation),
        field("name", $.identifier),
        optional(seq("=", field("value", $._expression))),
        ";",
      ),

    type_annotation: ($) => seq("<", field("type", $.identifier), ">"),

    // `target` is any expression syntactically; the linter/type-checker is
    // responsible for rejecting non-lvalue targets (e.g. `(a + b) = c`).
    assignment_statement: ($) =>
      seq(
        field("target", $._expression),
        field("operator", $._assignment_operator),
        field("value", $._expression),
        ";",
      ),

    _assignment_operator: (_) => choice("=", "+=", "-=", "*=", "/="),

    expression_statement: ($) => seq($._expression, ";"),

    if_statement: ($) =>
      seq(
        "if",
        "(",
        field("condition", $._expression),
        ")",
        field("consequence", $.block),
        optional($.else_clause),
      ),

    else_clause: ($) => seq("else", choice($.if_statement, $.block)),

    // State-machine block: `when (<channel>) { is (<state>) { ... } ... }`.
    // The `when` subject is the channel whose enumeration is being matched; each
    // `is` clause guards a block on one of that channel's state values.
    when_statement: ($) =>
      seq(
        "when",
        "(",
        field("subject", $._expression),
        ")",
        "{",
        repeat($.is_clause),
        "}",
      ),

    is_clause: ($) =>
      seq("is", "(", field("state", $._expression), ")", field("body", $.block)),

    // Compile-time loop: `expand (VAR = <start> to <end>) { ... }`. The body is
    // text-substituted for each value; $(VAR) interpolations live in the body.
    expand_statement: ($) =>
      seq(
        "expand",
        "(",
        field("variable", $.identifier),
        "=",
        field("start", $._expression),
        "to",
        field("end", $._expression),
        ")",
        $.block,
      ),

    // ---- Expressions --------------------------------------------------------
    _expression: ($) =>
      choice(
        $.identifier,
        $.interpolation,
        $.member_expression,
        $.call_expression,
        $.unary_expression,
        $.binary_expression,
        $.ternary_expression,
        $.parenthesized_expression,
        $.number,
        $.boolean,
        $.string,
      ),

    member_expression: ($) =>
      prec.left(
        PREC.member,
        seq(field("object", $._expression), ".", field("property", $.identifier)),
      ),

    call_expression: ($) =>
      prec(
        PREC.call,
        seq(field("function", $._expression), field("arguments", $.argument_list)),
      ),

    argument_list: ($) =>
      seq("(", optional(seq($._expression, repeat(seq(",", $._expression)))), ")"),

    parenthesized_expression: ($) => seq("(", $._expression, ")"),

    unary_expression: ($) =>
      prec(PREC.unary, seq(field("operator", choice("-", "not", "!")), $._expression)),

    ternary_expression: ($) =>
      prec.right(
        PREC.ternary,
        seq(
          field("condition", $._expression),
          "?",
          field("consequence", $._expression),
          ":",
          field("alternative", $._expression),
        ),
      ),

    binary_expression: ($) => {
      const table = [
        [PREC.or, choice("or", "||")],
        [PREC.and, choice("and", "&&")],
        [PREC.bitwise_or, "|"],
        [PREC.bitwise_xor, "^"],
        [PREC.bitwise_and, "&"],
        [PREC.equality, choice("==", "!=", "eq", "neq")],
        [PREC.relational, choice("<", ">", "<=", ">=")],
        [PREC.shift, choice("<<", ">>")],
        [PREC.additive, choice("+", "-")],
        [PREC.multiplicative, choice("*", "/", "%")],
      ];
      return choice(
        ...table.map(([precedence, operator]) =>
          prec.left(
            precedence,
            seq(
              field("left", $._expression),
              field("operator", operator),
              field("right", $._expression),
            ),
          ),
        ),
      );
    },

    // ---- Tokens -------------------------------------------------------------
    number: (_) =>
      token(
        choice(
          /0[xX][0-9a-fA-F]+[uU]?/,
          /\d+\.\d+([eE][+-]?\d+)?/,
          /\d+[eE][+-]?\d+/,
          /\d+[uU]?/,
        ),
      ),

    boolean: (_) => choice("true", "false"),

    string: (_) => token(seq('"', /[^"]*/, '"')),

    line_comment: (_) => token(seq("//", /[^\n]*/)),

    block_comment: (_) => token(seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/")),
  },
});
