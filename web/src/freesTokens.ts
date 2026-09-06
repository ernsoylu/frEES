// Shared lexer-shaped tokens used by the wizard, GUESS reader, and completion.
// One copy so Sonar does not count the same NUMBER/IDENT regex three times.

/** Signed/scientific numeric literal the lexer accepts as a NUMBER token. */
export const NUMERIC_LITERAL = /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$/

/** Source fragment of the same NUMBER token, for embedding in larger regexes. */
export const NUMERIC_LITERAL_SRC = String.raw`[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?`

/** `10 [W/K]`, `-2.5e-3[degC]` — a numeric literal that already carries units. */
export const UNIT_ANNOTATED_LITERAL =
  /^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?\s*\[[^\]]+]$/

/** Bare identifier, including a trailing `$` string name. */
export const IDENT = /^[A-Za-z_][A-Za-z0-9_]*\$?$/

/** Dotted member path (`HX.in.P`, `conductance`). */
export const MEMBER_PATH = /^[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*\$?$/
