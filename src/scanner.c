/*
 * scanner.c — external scanner for the M1 tree-sitter grammar.
 *
 * Produces a single external token: `identifier`. An identifier is one path
 * *segment*: a run of words ([A-Za-z_][A-Za-z0-9_]*) joined by single spaces,
 * where no word is a reserved keyword.
 *
 * The tricky case this exists to handle:
 *
 *     Pellow.KVB Bonquil eq Wexlar Bonquil Mosko.Vor
 *             ^^^^^^^^^^                              segment "KVB Bonquil"
 *                        ^^                           keyword `eq` (NOT absorbed)
 *
 * We greedily join "word SPACE word" but stop before a word that is reserved,
 * using mark_end() so the trailing " eq" is returned to the lexer as
 * look-ahead rather than being swallowed into the identifier.
 */

#include "tree_sitter/parser.h"

#include <stdbool.h>
#include <string.h>

enum TokenType { IDENTIFIER };

/* Mirrors the keyword tokens in grammar.js. Keep the two in sync. */
static const char *const RESERVED[] = {
    "local", "if", "else", "and", "or", "not", "eq", "neq", "true", "false",
    "static", "when", "is", "expand", "to",
};
static const unsigned RESERVED_COUNT = sizeof(RESERVED) / sizeof(RESERVED[0]);

static bool is_word_start(int32_t c) {
  return c == '_' || (c >= 'A' && c <= 'Z') || (c >= 'a' && c <= 'z');
}

static bool is_word_char(int32_t c) {
  return is_word_start(c) || (c >= '0' && c <= '9');
}

/* Read a maximal word into buf (NUL-terminated, truncated to cap). */
static void read_word(TSLexer *lexer, char *buf, unsigned cap) {
  unsigned n = 0;
  while (is_word_char(lexer->lookahead)) {
    if (n + 1 < cap) {
      buf[n] = (char)lexer->lookahead;
    }
    n++;
    lexer->advance(lexer, false);
  }
  buf[n < cap ? n : cap - 1] = '\0';
}

static bool is_reserved(const char *word) {
  for (unsigned i = 0; i < RESERVED_COUNT; i++) {
    if (strcmp(word, RESERVED[i]) == 0) {
      return true;
    }
  }
  return false;
}

/* Consume a `$(...)` compile-time interpolation, e.g. `$(SEG)`. These appear
 * both as standalone operands (`( $(SEG) - 1)`) and as units inside a name
 * segment (`naxID Bnk $(SEG) Vlim $(NODE)`). Returns true if a complete
 * interpolation was consumed. */
static bool try_read_interp(TSLexer *lexer) {
  if (lexer->lookahead != '$') {
    return false;
  }
  lexer->advance(lexer, false);
  if (lexer->lookahead != '(') {
    return false;
  }
  lexer->advance(lexer, false);
  while (lexer->lookahead != ')' && lexer->lookahead != 0) {
    lexer->advance(lexer, false);
  }
  if (lexer->lookahead != ')') {
    return false;
  }
  lexer->advance(lexer, false);
  return true;
}

void *tree_sitter_m1_external_scanner_create(void) { return NULL; }
void tree_sitter_m1_external_scanner_destroy(void *payload) { (void)payload; }
void tree_sitter_m1_external_scanner_reset(void *payload) { (void)payload; }

unsigned tree_sitter_m1_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}

void tree_sitter_m1_external_scanner_deserialize(void *payload, const char *buffer,
                                                 unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

bool tree_sitter_m1_external_scanner_scan(void *payload, TSLexer *lexer,
                                          const bool *valid_symbols) {
  (void)payload;
  if (!valid_symbols[IDENTIFIER]) {
    return false;
  }

  /* Skip leading whitespace; it is never part of the token. */
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t' ||
         lexer->lookahead == '\n' || lexer->lookahead == '\r') {
    lexer->advance(lexer, true);
  }

  /* First unit of the segment: a word or a `$(...)` interpolation. Requiring a
   * letter/`_`/`$` start keeps bare numeric literals (e.g. `, 16)`) from being
   * lexed as identifiers. */
  if (lexer->lookahead == '$') {
    if (!try_read_interp(lexer)) {
      return false;
    }
  } else if (is_word_start(lexer->lookahead)) {
    char word[64];
    read_word(lexer, word, sizeof(word));
    if (is_reserved(word)) {
      return false; /* let the grammar match the keyword */
    }
  } else {
    return false;
  }
  lexer->mark_end(lexer); /* identifier currently ends after the first unit */

  /* Extend with " <unit>" while the next unit is not a reserved word. A
   * continuation word may begin with a digit ("XV Glim 4", "Glonk 9",
   * "...FSE 5X Vund Klee"); a unit may also be a `$(...)` interpolation
   * ("naxID Bnk $(SEG) Vlim $(NODE)"). */
  for (;;) {
    if (lexer->lookahead != ' ') {
      break;
    }
    lexer->advance(lexer, false); /* tentatively consume the space */
    if (lexer->lookahead == '$') {
      if (!try_read_interp(lexer)) {
        break;
      }
    } else if (is_word_char(lexer->lookahead)) {
      char next[64];
      read_word(lexer, next, sizeof(next));
      if (is_reserved(next)) {
        break; /* leave the consumed " <kw>" as look-ahead via mark_end */
      }
    } else {
      break;
    }
    lexer->mark_end(lexer); /* commit this unit into the identifier */
  }

  lexer->result_symbol = IDENTIFIER;
  return true;
}
