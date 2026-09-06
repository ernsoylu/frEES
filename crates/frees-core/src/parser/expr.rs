//! Expression grammar.
//!
//! Rules `boolExpr`, `expr`, `addExpr`, `mulExpr`, `unaryExpr`, `powExpr`,
//! `atom`, `matrixRow`, `argList`, `arg`, `arrayIndexList`, `arrayIndex`,
//! `unit` from `Frees.g4` (lines 417-497).
//!
//! Port of the expression half of
//! `../frEES/backend/core/src/main/java/com/frees/backend/parser/AstBuilder.java`
//! (`visitExpr` … `visitParenAtom`, `buildBoolExpr`, `bareUnitLiteral`).
//!
//! # Shape of the grammar
//!
//! ```text
//! boolExpr : NOT boolExpr | boolExpr AND boolExpr | boolExpr OR boolExpr
//!          | LPAREN boolExpr RPAREN | expr REL expr | expr
//! addExpr  : mulExpr   ((PLUS | MINUS) mulExpr)*                 left-assoc
//! mulExpr  : unaryExpr ((TIMES | DIV | BACKSLASH
//!                        | DOTSTAR | DOTSLASH | DOTBACKSLASH) unaryExpr)*
//! unaryExpr: (MINUS | PLUS) unaryExpr | powExpr
//! powExpr  : atom ((CARET | DOTCARET) unaryExpr)? TRANSPOSE*     right-assoc
//! ```
//!
//! ANTLR orders the `boolExpr` alternatives by precedence, so `NOT` binds
//! tighter than `AND`, which binds tighter than `OR`; both connectives are
//! left-associative. `DOTCARET` lives in `powExpr` (not `mulExpr`) — that is
//! the grammar as written, and it is reproduced verbatim here.
//!
//! # Parse-time semantics carried over from `AstBuilder`
//!
//! * A unit-annotated literal is converted to SI **while parsing**:
//!   `value * factor + offset`, with [`UnitRegistry::si_display_name`] as the
//!   stored unit. An unknown unit keeps the original value and text — the unit
//!   checker surfaces it as a warning later, and a bad unit never fails a parse.
//! * `bareUnitLiteral`: a `-` directly in front of a bare unit-annotated number
//!   folds into the literal *before* conversion, so `-10 [C]` is −10 °C =
//!   263.15 K, not −(283.15 K). Only offset scales are affected; pure factors
//!   commute with negation and keep the ordinary `Neg` path.
//! * A call carrying named arguments is a fluid-property call and is encoded as
//!   `prop$<fn>$<fluid>$<indicator>…` so the fluid and indicator labels never
//!   become solver variables.

use std::cell::Cell;

use crate::ast::{BinOp, CmpOp, Expr, LogicOp};
use crate::diag::{FreesError, Result, Span};
use crate::parser::Cursor;
use crate::token::TokenKind;
use crate::units::UnitRegistry;

// ── Recursion / AST-depth budget ────────────────────────────────────────────
//
// The grammar is recursive descent and [`Expr`] is a recursive tree that every
// consumer (`eval`, `Expr::variables`, the unit walk, and `Drop` itself) walks
// recursively. Without a ceiling, `x = ((((…1…))))` overflows the *parser's*
// stack and `x = 1 + 1 + … + 1` overflows every *consumer's* stack on a tree
// the parser built happily. A stack overflow is an immediate `SIGSEGV`/abort —
// it cannot be caught, so it is not something a caller can turn into a
// diagnostic. The only defence is to never build the tree.
//
// One budget covers both hazards because both are measured in the same unit:
// the depth of the resulting `Expr`. Each link of a left-associative chain
// charges one level — that is the part a plain recursion guard misses, since
// `a + b + c + …` is parsed by a *loop* but produces a tree as deep as the
// chain is long — while each recursive descent step charges [`NESTING_COST`]
// levels, because it adds the same one level of tree *and* burns a run of
// parser frames that a chain link does not.
//
// Both numbers are measured, not chosen by taste. The binding configuration is
// a **debug** build inside a `libtest` thread, whose stack is far smaller than
// the 8 MiB main thread and whose frames are far fatter than a release build's.
// Measured there with the guard lifted, the heaviest shape — nested calls —
// survives 128 levels of nesting and aborts by 192. `MAX_EXPR_DEPTH / NESTING_COST`
// is therefore set to 64 levels of nesting, a 2–3× margin on the tightest
// configuration that exists.
//
// The *flat chain* half of the budget was originally left at ~500 terms on the
// reasoning that no hand-written equation approaches it. That bounded the
// parser but not its consumers, and it was too generous by a factor of two:
// measured by bisection on the same debug/`libtest` configuration, `check` and
// `solve` walk a flat chain of **304** links and abort above it, for every
// operator, while the parser itself accepted 519. `x + x + … + x` with 400
// terms therefore parsed and then aborted the process in `check` — the exact
// failure this budget exists to prevent, reachable from a document a user
// could type. Halving both constants keeps the nesting ceiling at 64 levels,
// byte for byte, and brings the flat-chain ceiling under the measured floor
// while still clearing the 200-term sum that
// `robustness::a_long_flat_chain_is_allowed_where_deep_nesting_is_not` pins as
// a legitimate document.
//
// Measured headroom for a flat chain, by bisection, after the change:
//
//   debug   / 2 MiB libtest thread   304  (1.19× the 256 admitted)
//   release / 512 KiB thread         374  (1.46×)
//   release / 1 MiB — the wasm stack ≥519 (≥2×, no abort at any legal depth)

/// Maximum depth of a parsed expression. See the module-level discussion.
pub const MAX_EXPR_DEPTH: u32 = 256;

/// Levels charged by one *recursive* descent step. Nesting burns a run of
/// parser frames that a chain link does not, so it is priced accordingly:
/// `MAX_EXPR_DEPTH / NESTING_COST` is the real nesting ceiling, and the rest of
/// the budget is available to left-associative chains.
const NESTING_COST: u32 = 4;

thread_local! {
    /// Depth currently held by the guards live on this thread's stack.
    static EXPR_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// An RAII claim on some number of expression-depth levels.
///
/// Levels are released when the guard drops, which happens on the `?` path too
/// — so a failed speculative parse (`try_bool_paren`) restores the budget
/// exactly, and a parse error leaves the counter at zero for the next document.
struct DepthGuard {
    held: u32,
}

impl DepthGuard {
    fn new() -> DepthGuard {
        DepthGuard { held: 0 }
    }

    /// Charge one more link of a left-associative chain.
    fn link(&mut self, c: &Cursor<'_>) -> Result<()> {
        self.charge(1, c)
    }

    /// Charge one recursive descent step.
    fn nest(&mut self, c: &Cursor<'_>) -> Result<()> {
        self.charge(NESTING_COST, c)
    }

    /// A guard that has already charged one recursive descent step.
    fn nested(c: &Cursor<'_>) -> Result<DepthGuard> {
        let mut guard = DepthGuard::new();
        guard.nest(c)?;
        Ok(guard)
    }

    fn charge(&mut self, levels: u32, c: &Cursor<'_>) -> Result<()> {
        let ok = EXPR_DEPTH.with(|d| {
            let next = d.get().saturating_add(levels);
            if next > MAX_EXPR_DEPTH {
                false
            } else {
                d.set(next);
                true
            }
        });
        if !ok {
            return Err(FreesError::parse_at(
                "expression is too deeply nested; simplify it, or split it \
                 across several equations",
                c.span(),
            ));
        }
        self.held += levels;
        Ok(())
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        let held = self.held;
        EXPR_DEPTH.with(|d| d.set(d.get().saturating_sub(held)));
    }
}

/// `UnitRegistry.FAHRENHEIT_OFFSET_K` — 0 °F expressed in kelvin.
const FAHRENHEIT_OFFSET_K: f64 = 459.67 * 5.0 / 9.0;

/// Single-token chemistry / bulk-material functions whose arguments are
/// fluid-formula *tokens*, not variables: `MolarMass(C8H18)`,
/// `HeatingValue(CH4, 'LHV')`, `k_('Aluminum')`. Mirrors `AstBuilder.CHEM_FUNCS`.
const CHEM_FUNCS: [&str; 12] = [
    "molarmass",
    "heatingvalue",
    "stoichafr",
    "t_crit",
    "p_crit",
    "t_triple",
    "v_crit",
    "k_",
    "rho_",
    "c_",
    "e_",
    "nu_",
];

// ── Public entry points ─────────────────────────────────────────────────────

/// Parse an arithmetic expression (`expr`).
pub fn parse_expr(c: &mut Cursor<'_>) -> Result<Expr> {
    parse_add(c)
}

/// Parse a boolean/relational expression (`boolExpr`) — IF / REPEAT / WHILE
/// conditions. Relational and logical results are `1.0` / `0.0`.
pub fn parse_bool_expr(c: &mut Cursor<'_>) -> Result<Expr> {
    parse_bool_or(c)
}

/// Parse a bracketed unit annotation `[ … ]`, returning its trimmed contents,
/// or `None` when the next token is not `[`.
///
/// An empty annotation (`[]`, `[   ]`) is consumed but reported as `None`,
/// exactly as `AstBuilder.unitText` maps empty text to `null`.
pub fn parse_unit_annotation(c: &mut Cursor<'_>) -> Result<Option<String>> {
    if !matches!(c.peek(), TokenKind::LBracket) {
        return Ok(None);
    }
    let open_end = c.span().end as usize;
    c.advance();
    loop {
        if matches!(c.peek(), TokenKind::RBracket) {
            break;
        }
        if !is_unit_content(c.peek()) {
            return Err(FreesError::parse_at(
                format!(
                    "expected a unit expression inside `[ … ]`, found {}",
                    c.peek().describe()
                ),
                c.span(),
            ));
        }
        c.advance();
    }
    let close_start = c.span().start as usize;
    c.advance(); // `]`

    // The Java builder slices the raw input between the brackets and trims it,
    // which keeps multi-token units such as `kJ/kg-K` in their source spelling.
    let text = c
        .source()
        .get(open_end..close_start)
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(if text.is_empty() { None } else { Some(text) })
}

/// The tokens `unitContent` admits.
fn is_unit_content(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Ident(_)
            | TokenKind::Number { .. }
            | TokenKind::Times
            | TokenKind::Div
            | TokenKind::Caret
            | TokenKind::Minus
            | TokenKind::Plus
            | TokenKind::Comma
            | TokenKind::LParen
            | TokenKind::RParen
    )
}

// ── boolExpr ────────────────────────────────────────────────────────────────

fn parse_bool_or(c: &mut Cursor<'_>) -> Result<Expr> {
    let mut depth = DepthGuard::nested(c)?;
    let mut left = parse_bool_and(c)?;
    while c.eat(&TokenKind::Or) {
        depth.link(c)?;
        let right = parse_bool_and(c)?;
        left = Expr::Logical {
            op: LogicOp::Or,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_bool_and(c: &mut Cursor<'_>) -> Result<Expr> {
    let mut depth = DepthGuard::new();
    let mut left = parse_bool_not(c)?;
    while c.eat(&TokenKind::And) {
        depth.link(c)?;
        let right = parse_bool_not(c)?;
        left = Expr::Logical {
            op: LogicOp::And,
            left: Box::new(left),
            right: Box::new(right),
        };
    }
    Ok(left)
}

fn parse_bool_not(c: &mut Cursor<'_>) -> Result<Expr> {
    if c.eat(&TokenKind::Not) {
        let _depth = DepthGuard::nested(c)?;
        return Ok(Expr::Not(Box::new(parse_bool_not(c)?)));
    }
    parse_bool_primary(c)
}

/// `LPAREN boolExpr RPAREN | expr REL expr | expr`.
///
/// The parenthesised alternative is speculative: ANTLR's adaptive prediction
/// only takes it when the closing `)` is *not* followed by something that
/// continues an arithmetic expression, so `(a > 1) AND b` is a bool-paren while
/// `(a + b) * 2 > 5` is a relation over a parenthesised atom. Both spellings
/// unwrap the parentheses in the AST, so the rewind only ever matters for the
/// genuinely ambiguous shapes.
fn parse_bool_primary(c: &mut Cursor<'_>) -> Result<Expr> {
    if matches!(c.peek(), TokenKind::LParen) {
        let save = c.pos();
        if let Some(inner) = try_bool_paren(c) {
            return Ok(inner);
        }
        c.reset_to(save);
    }
    let left = parse_expr(c)?;
    if let Some(op) = cmp_op(c.peek()) {
        c.advance();
        let right = parse_expr(c)?;
        return Ok(Expr::Compare {
            op,
            left: Box::new(left),
            right: Box::new(right),
        });
    }
    Ok(left)
}

fn try_bool_paren(c: &mut Cursor<'_>) -> Option<Expr> {
    c.advance(); // `(`
    let inner = parse_bool_expr(c).ok()?;
    if !matches!(c.peek(), TokenKind::RParen) {
        return None;
    }
    c.advance();
    if continues_expr(c.peek()) {
        return None;
    }
    Some(inner)
}

/// True for tokens that can extend an arithmetic expression or open a
/// relation — the signal that a `( … )` was an atom, not a grouped condition.
fn continues_expr(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Times
            | TokenKind::Div
            | TokenKind::Backslash
            | TokenKind::Caret
            | TokenKind::DotStar
            | TokenKind::DotSlash
            | TokenKind::DotBackslash
            | TokenKind::DotCaret
            | TokenKind::Transpose
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::Le
            | TokenKind::Ge
            | TokenKind::Ne
            | TokenKind::Eq
    )
}

fn cmp_op(kind: &TokenKind) -> Option<CmpOp> {
    match kind {
        TokenKind::Lt => Some(CmpOp::Lt),
        TokenKind::Gt => Some(CmpOp::Gt),
        TokenKind::Le => Some(CmpOp::Le),
        TokenKind::Ge => Some(CmpOp::Ge),
        TokenKind::Ne => Some(CmpOp::Ne),
        TokenKind::Eq => Some(CmpOp::Eq),
        _ => None,
    }
}

// ── addExpr / mulExpr / unaryExpr / powExpr ─────────────────────────────────

fn parse_add(c: &mut Cursor<'_>) -> Result<Expr> {
    let mut depth = DepthGuard::nested(c)?;
    let mut left = parse_mul(c)?;
    loop {
        let op = match c.peek() {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            _ => break,
        };
        c.advance();
        depth.link(c)?;
        let right = parse_mul(c)?;
        left = Expr::bin(op, left, right);
    }
    Ok(left)
}

fn parse_mul(c: &mut Cursor<'_>) -> Result<Expr> {
    let mut depth = DepthGuard::new();
    let mut left = parse_unary(c)?;
    loop {
        let op = match c.peek() {
            TokenKind::Times => BinOp::Mul,
            TokenKind::Div => BinOp::Div,
            TokenKind::Backslash => BinOp::LeftDiv,
            TokenKind::DotStar => BinOp::ElemMul,
            TokenKind::DotSlash => BinOp::ElemDiv,
            TokenKind::DotBackslash => BinOp::ElemLeftDiv,
            _ => break,
        };
        c.advance();
        depth.link(c)?;
        let right = parse_unary(c)?;
        left = Expr::bin(op, left, right);
    }
    Ok(left)
}

fn parse_unary(c: &mut Cursor<'_>) -> Result<Expr> {
    match c.peek() {
        TokenKind::Minus => {
            c.advance();
            if let Some(folded) = try_fold_signed_unit_literal(c) {
                return Ok(folded);
            }
            let _depth = DepthGuard::nested(c)?;
            Ok(Expr::Neg(Box::new(parse_unary(c)?)))
        }
        // A leading `+` is a no-op in the Java builder: it returns the operand
        // unchanged rather than wrapping it in a node.
        TokenKind::Plus => {
            c.advance();
            let _depth = DepthGuard::nested(c)?;
            parse_unary(c)
        }
        _ => parse_pow(c),
    }
}

/// `bareUnitLiteral` — the `-10 [C]` trap.
///
/// Applies only when the operand of the `-` is *exactly* `NUMBER unit`: no
/// exponent, no transpose, no nested sign, a non-empty unit that resolves, and
/// an offset scale (a pure factor commutes with negation and keeps the `Neg`
/// path). Anything else rewinds the cursor and lets `unaryExpr` proceed
/// normally, so `--10 [C]` and `-10 [C]^2` keep ordinary operator semantics.
fn try_fold_signed_unit_literal(c: &mut Cursor<'_>) -> Option<Expr> {
    let save = c.pos();
    let value = match c.peek() {
        TokenKind::Number { value, .. } => *value,
        _ => return None,
    };
    if !matches!(c.peek_at(1), TokenKind::LBracket) {
        return None;
    }
    c.advance(); // NUMBER
    let unit = match parse_unit_annotation(c) {
        Ok(Some(unit)) => unit,
        _ => {
            c.reset_to(save);
            return None;
        }
    };
    if matches!(
        c.peek(),
        TokenKind::Caret | TokenKind::DotCaret | TokenKind::Transpose
    ) {
        c.reset_to(save);
        return None;
    }
    match UnitRegistry::parse_with_offset(&unit) {
        Ok(q) if q.offset != 0.0 => Some(Expr::Num {
            value: -value * q.factor + q.offset,
            unit: Some(UnitRegistry::si_display_name(&unit, &q.dims)),
            is_imaginary: false,
        }),
        // Unknown unit, or a purely multiplicative one: ordinary `Neg` path.
        _ => {
            c.reset_to(save);
            None
        }
    }
}

fn parse_pow(c: &mut Cursor<'_>) -> Result<Expr> {
    let mut base = parse_atom(c)?;
    let op = match c.peek() {
        TokenKind::Caret => Some(BinOp::Pow),
        TokenKind::DotCaret => Some(BinOp::ElemPow),
        _ => None,
    };
    let mut depth = DepthGuard::new();
    if let Some(op) = op {
        c.advance();
        // The exponent is a `unaryExpr`, which makes `^` right-associative and
        // lets `2^-3` parse without parentheses. Right-associativity means
        // `2^2^2^…` recurses one level per `^` without ever passing through a
        // guarded rule, so this one is charged here.
        depth.nest(c)?;
        let exponent = parse_unary(c)?;
        base = Expr::bin(op, base, exponent);
    }
    // `TRANSPOSE*` — one `transpose(...)` wrapper per `'`.
    while c.eat(&TokenKind::Transpose) {
        depth.link(c)?;
        base = Expr::call("transpose", vec![base]);
    }
    Ok(base)
}

// ── atom ────────────────────────────────────────────────────────────────────

fn parse_atom(c: &mut Cursor<'_>) -> Result<Expr> {
    match c.peek().clone() {
        TokenKind::Number { value, .. } => {
            c.advance();
            let unit = parse_unit_annotation(c)?;
            Ok(number_literal(value, unit, false))
        }
        TokenKind::ImagNumber { value, .. } => {
            c.advance();
            let unit = parse_unit_annotation(c)?;
            Ok(number_literal(value, unit, true))
        }
        TokenKind::StringLiteral(text) => {
            c.advance();
            Ok(Expr::Str(text))
        }
        TokenKind::If => {
            c.advance();
            c.expect(&TokenKind::LParen)?;
            let args = parse_arg_list(c)?;
            c.expect(&TokenKind::RParen)?;
            Ok(Expr::call("if", positional_only(args, "if")?))
        }
        TokenKind::Ident(name) => match c.peek_at(1) {
            TokenKind::LParen => parse_call_atom(c, name),
            TokenKind::Dot => parse_member_atom(c, name),
            TokenKind::LBracket => parse_array_atom(c, name),
            _ => {
                c.advance();
                // `AstBuilder.visitVarAtom`: a built-in `#` constant is
                // substituted at parse time as a numeric literal carrying its
                // raw SI unit string (already SI — no conversion), so the unit
                // checker can ground downstream variables. Unknown `#` names
                // fall through as ordinary variables, exactly like the Java.
                if name.ends_with('#') {
                    if let Some(constant) = crate::eval::lookup_constant(&name) {
                        return Ok(Expr::Num {
                            value: constant.value,
                            unit: constant.unit.map(str::to_string),
                            is_imaginary: false,
                        });
                    }
                }
                // `visitVarAtom` records the spelling *after* the constant
                // lookup returns null — a substituted `pi#` never enters the
                // display-name table.
                c.record_display_name(&name);
                Ok(Expr::var(name))
            }
        },
        TokenKind::LBracket => parse_matrix_literal(c),
        TokenKind::LParen => {
            c.advance();
            let inner = parse_expr(c)?;
            c.expect(&TokenKind::RParen)?;
            Ok(inner)
        }
        other => Err(FreesError::parse_at(
            format!("expected an expression, found {}", other.describe()),
            c.span(),
        )),
    }
}

/// `NUMBER unit?` / `IMAG_NUMBER unit?` — SI conversion happens here.
fn number_literal(value: f64, unit: Option<String>, is_imaginary: bool) -> Expr {
    let Some(unit) = unit else {
        return Expr::Num {
            value,
            unit: None,
            is_imaginary,
        };
    };
    match UnitRegistry::parse_with_offset(&unit) {
        Ok(q) => Expr::Num {
            value: value * q.factor + q.offset,
            unit: Some(UnitRegistry::si_display_name(&unit, &q.dims)),
            is_imaginary,
        },
        // Unknown unit: keep the user's text and value. The unit checker turns
        // this into a warning — a bad unit never fails a parse.
        Err(_) => Expr::Num {
            value,
            unit: Some(unit),
            is_imaginary,
        },
    }
}

/// `IDENT (DOT IDENT)+` — a dotted port/instance path carried whole in an
/// `Expr::Var`; `ComponentExpander` resolves it later. `Expr::var` lowercases,
/// exactly as the Java record's compact constructor does.
fn parse_member_atom(c: &mut Cursor<'_>, first: String) -> Result<Expr> {
    c.advance(); // IDENT
    let mut path = first;
    while c.eat(&TokenKind::Dot) {
        let segment = c.expect_ident()?;
        path.push('.');
        path.push_str(&segment);
    }
    Ok(Expr::var(path))
}

/// `IDENT LBRACKET arrayIndexList RBRACKET`.
fn parse_array_atom(c: &mut Cursor<'_>, name: String) -> Result<Expr> {
    c.advance(); // IDENT
                 // `AstBuilder.visitArrayAtom` registers the *base* name, not the indexed
                 // element; the element spellings are added later by the flattener.
    c.record_display_name(&name);
    c.expect(&TokenKind::LBracket)?;
    let mut indices = vec![parse_array_index(c)?];
    while c.eat(&TokenKind::Comma) {
        indices.push(parse_array_index(c)?);
    }
    c.expect(&TokenKind::RBracket)?;
    Ok(Expr::ArrayAccess {
        name: name.to_ascii_lowercase(),
        indices,
    })
}

/// `arrayIndex : expr (COLON expr)?` — the colon form is an index range.
fn parse_array_index(c: &mut Cursor<'_>) -> Result<Expr> {
    let start = parse_expr(c)?;
    if c.eat(&TokenKind::Colon) {
        let end = parse_expr(c)?;
        return Ok(Expr::Range {
            start: Box::new(start),
            end: Box::new(end),
        });
    }
    Ok(start)
}

/// `LBRACKET matrixRow (SEMI matrixRow)* RBRACKET unit?`
///
/// The result is an `ArrayLiteral` of one `ArrayLiteral` per row, matching
/// `visitMatrixLiteralAtom` — even for a single row.
fn parse_matrix_literal(c: &mut Cursor<'_>) -> Result<Expr> {
    let open = c.span();
    c.advance(); // `[`
    let mut rows = vec![Expr::ArrayLiteral(parse_matrix_row(c)?)];
    while c.eat(&TokenKind::Semi) {
        rows.push(Expr::ArrayLiteral(parse_matrix_row(c)?));
    }
    c.expect(&TokenKind::RBracket)?;
    let mut literal = Expr::ArrayLiteral(rows);
    if let Some(unit) = parse_unit_annotation(c)? {
        literal = apply_unit_to_elements(literal, &unit, open)?;
    }
    Ok(literal)
}

/// `matrixRow : expr (COMMA? expr)*` — commas are optional, so elements may be
/// juxtaposed (`[1 2 3]`). `[1 -2]` is still a single element, because the
/// enclosing `addExpr` is greedy.
fn parse_matrix_row(c: &mut Cursor<'_>) -> Result<Vec<Expr>> {
    let mut elements = vec![parse_expr(c)?];
    loop {
        if c.eat(&TokenKind::Comma) {
            elements.push(parse_expr(c)?);
            continue;
        }
        if starts_atom(c.peek()) {
            elements.push(parse_expr(c)?);
            continue;
        }
        break;
    }
    Ok(elements)
}

fn starts_atom(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Number { .. }
            | TokenKind::ImagNumber { .. }
            | TokenKind::StringLiteral(_)
            | TokenKind::Ident(_)
            | TokenKind::If
            | TokenKind::LParen
            | TokenKind::LBracket
    )
}

/// A trailing unit on a bracket literal (`[1 2 3] [kg]`) applies to every
/// numeric leaf that does not already carry one.
///
/// Note the asymmetry with `NUMBER unit?`: the Java builder attaches the raw
/// unit text here **without** converting the elements to SI. Reproduced as-is
/// so the wasm engine and the JVM oracle agree.
fn apply_unit_to_elements(e: Expr, unit: &str, span: Span) -> Result<Expr> {
    match e {
        Expr::ArrayLiteral(elements) => {
            let mut out = Vec::with_capacity(elements.len());
            for el in elements {
                out.push(apply_unit_to_elements(el, unit, span)?);
            }
            Ok(Expr::ArrayLiteral(out))
        }
        Expr::Num {
            value,
            unit: existing,
            is_imaginary,
        } => Ok(Expr::Num {
            value,
            unit: Some(existing.unwrap_or_else(|| unit.to_string())),
            is_imaginary,
        }),
        Expr::Neg(inner) => Ok(Expr::Neg(Box::new(apply_unit_to_elements(
            *inner, unit, span,
        )?))),
        _ => Err(FreesError::parse_at(
            "A trailing unit on a bracket literal (e.g. [1 2 3] [kg]) requires numeric elements."
                .to_string(),
            span,
        )),
    }
}

// ── argList / arg and the call forms ────────────────────────────────────────

/// One `arg`: `IDENT EQ expr` (named) or `expr` (positional).
struct Arg {
    /// `Some` for `T=300`; the indicator name in its original case.
    name: Option<String>,
    value: Expr,
    /// The argument's source text with whitespace removed — the equivalent of
    /// ANTLR's `ctx.getText()`, which the fluid and chemistry paths need
    /// because fluid names, chemical formulas and unit names are case-sensitive
    /// while `Expr::Var` is not.
    raw: String,
    span: Span,
}

fn parse_arg_list(c: &mut Cursor<'_>) -> Result<Vec<Arg>> {
    let mut args = vec![parse_arg(c)?];
    while c.eat(&TokenKind::Comma) {
        args.push(parse_arg(c)?);
    }
    Ok(args)
}

fn parse_arg(c: &mut Cursor<'_>) -> Result<Arg> {
    let mut name = None;
    if let TokenKind::Ident(ident) = c.peek().clone() {
        if matches!(c.peek_at(1), TokenKind::Eq) {
            c.advance(); // IDENT
            c.advance(); // `=`
            name = Some(ident);
        }
    }
    let start = c.span().start;
    let value = parse_expr(c)?;
    let (mut raw, span) = source_text_since(c, start);
    // A lone string literal is one token, so ANTLR's `getText()` keeps its
    // interior spacing; only *inter-token* whitespace disappears. Restoring it
    // matters because `'Stainless Steel'` must stay a two-word token and be
    // rejected as a formula, rather than silently becoming `StainlessSteel`.
    if let Expr::Str(text) = &value {
        if raw.starts_with('\'') && raw.ends_with('\'') {
            raw = format!("'{text}'");
        }
    }
    Ok(Arg {
        name,
        value,
        raw,
        span,
    })
}

/// The source text consumed since byte offset `start`, rendered the way ANTLR's
/// `ctx.getText()` renders it, plus a span that excludes the gap before the next
/// token.
fn source_text_since(c: &Cursor<'_>, start: u32) -> (String, Span) {
    let end = c.span().start;
    let slice = c
        .source()
        .get(start as usize..end as usize)
        .unwrap_or("")
        .trim_end();
    let span = Span::new(start as usize, start as usize + slice.len());
    (token_text(slice), span)
}

/// `ctx.getText()` concatenates the *tokens* of the parse sub-tree, so every
/// `-> skip` rule is invisible to it: whitespace **and all three comment forms**.
/// Slicing the raw source instead would glue a comment into the text and turn
/// `Enthalpy(R134a {ref}, T=300)` into the fluid name `R134a{ref}`, which the
/// `[A-Za-z]\w*\$?` check then rejects — a document the JVM oracle accepts.
///
/// A string literal is a single token, so its interior survives verbatim; that
/// is what keeps `'Stainless Steel'` a two-word token instead of one glued word.
fn token_text(slice: &str) -> String {
    let bytes = slice.as_bytes();
    let mut out = String::with_capacity(slice.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            // BRACE_COMMENT / QUOTE_COMMENT: non-greedy, so they end at the
            // first closing delimiter; an unterminated one runs to the end.
            b'{' => i = skip_to(slice, i + 1, '}'),
            b'"' => i = skip_to(slice, i + 1, '"'),
            // LINE_COMMENT: to end of line, the break itself left in place.
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i += slice[i..].find(['\r', '\n']).unwrap_or(slice.len() - i);
            }
            // STRING_LITERAL — one token, copied as written. With no closing
            // quote this is a TRANSPOSE, which is also copied as written.
            b'\'' => {
                let close = skip_to(slice, i + 1, '\'');
                out.push_str(&slice[i..close]);
                i = close;
            }
            _ => {
                let ch = slice[i..].chars().next().unwrap_or('\u{fffd}');
                if !ch.is_whitespace() {
                    out.push(ch);
                }
                i += ch.len_utf8();
            }
        }
    }
    out
}

/// The offset just past the next `close` at or after `from`, or the end of
/// `slice` when there is none.
fn skip_to(slice: &str, from: usize, close: char) -> usize {
    match slice.get(from..).and_then(|rest| rest.find(close)) {
        Some(off) => from + off + close.len_utf8(),
        None => slice.len(),
    }
}

/// `IDENT LPAREN argList RPAREN`, dispatching exactly as `visitCallAtom` does.
fn parse_call_atom(c: &mut Cursor<'_>, name: String) -> Result<Expr> {
    let span = c.span();
    c.advance(); // IDENT
    c.expect(&TokenKind::LParen)?;
    // Which spellings were already registered *before* this call's arguments
    // were parsed. `Enthalpy(CO2, T=…)` and `MolarMass(C8H18)` reach
    // `parse_expr` for their first argument, which builds an `Expr.Var` and
    // registers `CO2` / `C8H18` as a display name — but the Java grammar has a
    // dedicated `fluidName` / chem-token rule, so `AstBuilder.visitVarAtom`
    // never runs on it and `ParseResult.displayNames` has no such entry. The
    // token is discarded a few lines below (baked into the `prop$…` name, or
    // turned into an `Expr::Str`), so its registration is undone here.
    //
    // Undone only when the argument list *introduced* it: a document that uses
    // `Steel` as a variable before writing `k_(Steel)` keeps the variable's
    // spelling, which is what the Java's first-wins `putIfAbsent` produces.
    let mark = c.display_name_mark();
    let args = parse_arg_list(c)?;
    c.expect(&TokenKind::RParen)?;

    if args.iter().any(|a| a.name.is_some()) {
        // A property call consumes only its first (positional) argument as a
        // token; the indicator values stay real expressions.
        let token = args.first().map(|a| unquote(&a.raw).to_string());
        let call = build_property_call(&name, args, span)?;
        if let Some(token) = token {
            c.forget_display_name_if_new(&token, mark);
        }
        return Ok(call);
    }
    let lower = name.to_ascii_lowercase();
    if CHEM_FUNCS.contains(&lower.as_str()) {
        // A chem call consumes *every* argument as a token.
        let tokens: Vec<String> = args.iter().map(|a| unquote(&a.raw).to_string()).collect();
        let call = build_chem_call(&lower, args)?;
        for token in &tokens {
            c.forget_display_name_if_new(token, mark);
        }
        return Ok(call);
    }
    if lower == "convert" {
        // Both arguments are unit tokens, never variables: the Java grammar
        // routes them through `unit` contexts that `visitVarAtom` never sees,
        // so `Convert(psi, Pa)` leaves no `psi`/`pa` display entries. Undo the
        // registration the expression-first parse above just did (the
        // 2026-08-22 docs harvest caught the difference).
        let tokens: Vec<String> = args.iter().map(|a| unquote(&a.raw).to_string()).collect();
        let call = build_convert(args, span);
        for token in &tokens {
            c.forget_display_name_if_new(token, mark);
        }
        return call;
    }
    if lower == "converttemp" {
        // The first two arguments are temperature-scale tokens (C, K, F, R);
        // only the third is a real expression. Same rule as `Convert` above.
        let tokens: Vec<String> = args
            .iter()
            .take(2)
            .map(|a| unquote(&a.raw).to_string())
            .collect();
        let call = build_convert_temp(args, span);
        for token in &tokens {
            c.forget_display_name_if_new(token, mark);
        }
        return call;
    }
    Ok(Expr::call(
        &name,
        args.into_iter().map(|a| a.value).collect(),
    ))
}

/// `positionalExprs` — named arguments are only legal in property calls.
fn positional_only(args: Vec<Arg>, function: &str) -> Result<Vec<Expr>> {
    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        if arg.name.is_some() {
            return Err(FreesError::parse_at(
                format!(
                    "Named arguments (name=value) are only valid in fluid property functions: {function}"
                ),
                arg.span,
            ));
        }
        out.push(arg.value);
    }
    Ok(out)
}

/// Fluid property call: `Enthalpy(R134a, T=T1, x=1)`.
///
/// Encoded as a synthetic call `prop$<output>$<fluid>$<indicators…>` over the
/// *value* expressions, so the fluid and indicator labels never become solver
/// variables.
fn build_property_call(function: &str, args: Vec<Arg>, span: Span) -> Result<Expr> {
    let mut args = args.into_iter();
    let fluid_arg = args.next().ok_or_else(|| {
        FreesError::parse_at(
            format!(
                "Property functions take the fluid name first: {function}(R134a, T=..., x=...)"
            ),
            span,
        )
    })?;
    if fluid_arg.name.is_some() {
        return Err(FreesError::parse_at(
            format!(
                "Property functions take the fluid name first: {function}(R134a, T=..., x=...)"
            ),
            fluid_arg.span,
        ));
    }
    // The fluid may be a bare name (R134a), a quoted literal ('R134a'), or a
    // string variable (R$) that `EquationParser` resolves once its assignment
    // is known.
    let fluid = unquote(&fluid_arg.raw);
    if !is_fluid_name(fluid) {
        return Err(FreesError::parse_at(
            format!("Invalid fluid name '{fluid}' in {function}(...)"),
            fluid_arg.span,
        ));
    }
    let mut encoded = String::from("prop$");
    encoded.push_str(&function.to_ascii_lowercase());
    encoded.push('$');
    encoded.push_str(&fluid.to_ascii_lowercase());
    let mut values = Vec::new();
    for arg in args {
        let Some(indicator) = arg.name else {
            return Err(FreesError::parse_at(
                format!(
                    "Property indicators must be named after the fluid, e.g. {function}(R134a, T=300, x=1)"
                ),
                arg.span,
            ));
        };
        encoded.push('$');
        encoded.push_str(&indicator.to_ascii_lowercase());
        values.push(arg.value);
    }
    Ok(Expr::call(encoded, values))
}

/// Chemistry / bulk-material call: `MolarMass(C8H18)`, `HeatingValue(CH4, 'LHV')`.
///
/// The tokens travel as `Expr::Str` rather than inside the function name,
/// because `Expr::Call` lowercases its name and that would corrupt a
/// case-sensitive formula such as `C8H18`.
fn build_chem_call(function: &str, args: Vec<Arg>) -> Result<Expr> {
    let mut tokens = Vec::with_capacity(args.len());
    for arg in args {
        if arg.name.is_some() {
            return Err(FreesError::parse_at(
                format!(
                    "{function} takes fluid/formula tokens, e.g. MolarMass(C8H18) or HeatingValue(CH4, 'LHV')."
                ),
                arg.span,
            ));
        }
        let token = unquote(&arg.raw);
        if !is_chem_token(token) {
            return Err(FreesError::parse_at(
                format!(
                    "Invalid token '{token}' in {function}(...). Use a fluid name, ideal-gas \
                     species, or chemical formula (quote formulas with parentheses, e.g. 'Ca(OH)2')."
                ),
                arg.span,
            ));
        }
        tokens.push(Expr::Str(token.to_string()));
    }
    Ok(Expr::call(format!("prop${function}"), tokens))
}

/// `Convert(From, To)` — folded to the scale factor at parse time.
fn build_convert(args: Vec<Arg>, span: Span) -> Result<Expr> {
    if args.len() != 2 {
        return Err(FreesError::parse_at(
            "Convert requires exactly two unit arguments: Convert(From, To)".to_string(),
            span,
        ));
    }
    let from = unquote(&args[0].raw).to_string();
    let to = unquote(&args[1].raw).to_string();
    let source = UnitRegistry::parse(&from)
        .map_err(|e| FreesError::parse_at(e.to_string(), args[0].span))?;
    let target =
        UnitRegistry::parse(&to).map_err(|e| FreesError::parse_at(e.to_string(), args[1].span))?;
    if !source.same_dimensions_as(&target) {
        return Err(FreesError::parse_at(
            format!(
                "Convert({from}, {to}): units have different dimensions [{}] vs [{}].",
                source.dimension_string(),
                target.dimension_string()
            ),
            span,
        ));
    }
    Ok(Expr::Num {
        value: source.factor / target.factor,
        unit: None,
        is_imaginary: false,
    })
}

/// `ConvertTemp(From, To, value)` — an affine rescaling folded at parse time.
fn build_convert_temp(args: Vec<Arg>, span: Span) -> Result<Expr> {
    if args.len() != 3 {
        return Err(FreesError::parse_at(
            "ConvertTemp requires three arguments: ConvertTemp(From, To, value)".to_string(),
            span,
        ));
    }
    let (to_scale, to_offset) = temperature_to_kelvin(unquote(&args[0].raw), args[0].span)?;
    let (from_scale, from_offset) = kelvin_to_temperature(unquote(&args[1].raw), args[1].span)?;
    let a = to_scale * from_scale;
    let b = from_scale * to_offset + from_offset;
    let target_is_kelvin = unquote(&args[1].raw).trim().eq_ignore_ascii_case("k");
    let x = args.into_iter().nth(2).expect("length checked above").value;
    if let Expr::Num {
        value,
        unit: None,
        is_imaginary: false,
    } = x
    {
        return Ok(Expr::Num {
            value: a * value + b,
            unit: if target_is_kelvin {
                Some("K".to_string())
            } else {
                None
            },
            is_imaginary: false,
        });
    }
    let scaled = if a == 1.0 {
        x
    } else {
        Expr::bin(BinOp::Mul, Expr::num(a), x)
    };
    Ok(if b == 0.0 {
        scaled
    } else {
        Expr::bin(BinOp::Add, scaled, Expr::num(b))
    })
}

/// `(scale, offset)` taking the named temperature scale to kelvin.
fn temperature_to_kelvin(scale: &str, span: Span) -> Result<(f64, f64)> {
    match scale.trim().to_ascii_lowercase().as_str() {
        "c" => Ok((1.0, 273.15)),
        "k" => Ok((1.0, 0.0)),
        "f" => Ok((5.0 / 9.0, FAHRENHEIT_OFFSET_K)),
        "r" => Ok((5.0 / 9.0, 0.0)),
        _ => Err(FreesError::parse_at(
            format!("ConvertTemp: unknown temperature scale '{scale}' (use C, K, F, or R)"),
            span,
        )),
    }
}

/// `(scale, offset)` taking kelvin to the named temperature scale.
fn kelvin_to_temperature(scale: &str, span: Span) -> Result<(f64, f64)> {
    match scale.trim().to_ascii_lowercase().as_str() {
        "c" => Ok((1.0, -273.15)),
        "k" => Ok((1.0, 0.0)),
        "f" => Ok((9.0 / 5.0, -459.67)),
        "r" => Ok((9.0 / 5.0, 0.0)),
        _ => Err(FreesError::parse_at(
            format!("ConvertTemp: unknown temperature scale '{scale}' (use C, K, F, or R)"),
            span,
        )),
    }
}

/// Strips optional surrounding single quotes: `Convert('ft^2','in^2')` ≡
/// `Convert(ft^2, in^2)`.
fn unquote(text: &str) -> &str {
    if text.len() >= 2 && text.starts_with('\'') && text.ends_with('\'') {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

/// The Java `[A-Za-z]\w*\$?` fluid-name check.
fn is_fluid_name(name: &str) -> bool {
    let body = name.strip_suffix('$').unwrap_or(name);
    let mut chars = body.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// The Java `[A-Za-z][A-Za-z0-9().]*` chemistry-token check.
fn is_chem_token(token: &str) -> bool {
    let mut chars = token.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '(' | ')' | '.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    // ── A stand-in lexer ────────────────────────────────────────────────────
    //
    // `crate::lexer::tokenize` is another module's `todo!()`, so these tests
    // carry a minimal scanner covering the token forms the expression grammar
    // uses. Spans are real byte offsets, because `parse_unit_annotation` slices
    // the source between the brackets.

    fn lex(src: &str) -> Vec<Token> {
        let b = src.as_bytes();
        let mut out: Vec<Token> = Vec::new();
        let mut i = 0usize;
        while i < b.len() {
            let start = i;
            match b[i] {
                b' ' | b'\t' => {
                    i += 1;
                    continue;
                }
                b'\r' | b'\n' => {
                    while i < b.len() && (b[i] == b'\r' || b[i] == b'\n') {
                        i += 1;
                    }
                    out.push(Token::new(TokenKind::Newline, Span::new(start, i)));
                    continue;
                }
                b'{' => {
                    while i < b.len() && b[i] != b'}' {
                        i += 1;
                    }
                    i = (i + 1).min(b.len());
                    continue;
                }
                b'"' => {
                    i += 1;
                    while i < b.len() && b[i] != b'"' {
                        i += 1;
                    }
                    i = (i + 1).min(b.len());
                    continue;
                }
                _ => {}
            }
            if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            // Numbers, including the leading-dot form `.5`.
            if b[i].is_ascii_digit()
                || (b[i] == b'.' && i + 1 < b.len() && b[i + 1].is_ascii_digit())
            {
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                if i < b.len() && b[i] == b'.' && i + 1 < b.len() && b[i + 1].is_ascii_digit() {
                    i += 1;
                    while i < b.len() && b[i].is_ascii_digit() {
                        i += 1;
                    }
                }
                if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
                    let mut j = i + 1;
                    if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
                        j += 1;
                    }
                    if j < b.len() && b[j].is_ascii_digit() {
                        i = j;
                        while i < b.len() && b[i].is_ascii_digit() {
                            i += 1;
                        }
                    }
                }
                let text = src[start..i].to_string();
                let value: f64 = text.parse().expect("test lexer: bad number");
                if i < b.len() && matches!(b[i], b'i' | b'I' | b'j' | b'J') {
                    i += 1;
                    out.push(Token::new(
                        TokenKind::ImagNumber {
                            value,
                            text: src[start..i].to_string(),
                        },
                        Span::new(start, i),
                    ));
                } else {
                    out.push(Token::new(
                        TokenKind::Number { value, text },
                        Span::new(start, i),
                    ));
                }
                continue;
            }
            if b[i].is_ascii_alphabetic() {
                i += 1;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                    i += 1;
                }
                if i < b.len() && (b[i] == b'$' || b[i] == b'#') {
                    i += 1;
                }
                out.push(Token::new(
                    TokenKind::keyword_or_ident(&src[start..i]),
                    Span::new(start, i),
                ));
                continue;
            }
            if b[i] == b'\'' {
                // `'` is TRANSPOSE right after something that can end an
                // operand, and opens a string literal otherwise. The real lexer
                // makes its own call; this keeps the fixtures unambiguous.
                let after_operand = matches!(
                    out.last().map(|t| &t.kind),
                    Some(TokenKind::Ident(_))
                        | Some(TokenKind::Number { .. })
                        | Some(TokenKind::ImagNumber { .. })
                        | Some(TokenKind::RParen)
                        | Some(TokenKind::RBracket)
                        | Some(TokenKind::Transpose)
                );
                if after_operand {
                    i += 1;
                    out.push(Token::new(TokenKind::Transpose, Span::new(start, i)));
                } else {
                    i += 1;
                    let text_start = i;
                    while i < b.len() && b[i] != b'\'' {
                        i += 1;
                    }
                    let text = src[text_start..i].to_string();
                    i = (i + 1).min(b.len());
                    out.push(Token::new(
                        TokenKind::StringLiteral(text),
                        Span::new(start, i),
                    ));
                }
                continue;
            }
            let kind2 = match src.get(i..i + 2).unwrap_or("") {
                ":=" => Some(TokenKind::Assign),
                ".." => Some(TokenKind::DotDot),
                ".*" => Some(TokenKind::DotStar),
                "./" => Some(TokenKind::DotSlash),
                ".\\" => Some(TokenKind::DotBackslash),
                ".^" => Some(TokenKind::DotCaret),
                "<=" => Some(TokenKind::Le),
                ">=" => Some(TokenKind::Ge),
                "<>" => Some(TokenKind::Ne),
                "->" => Some(TokenKind::Arrow),
                _ => None,
            };
            if let Some(kind) = kind2 {
                i += 2;
                out.push(Token::new(kind, Span::new(start, i)));
                continue;
            }
            let kind = match b[i] {
                b'=' => TokenKind::Eq,
                b'+' => TokenKind::Plus,
                b'-' => TokenKind::Minus,
                b'*' => TokenKind::Times,
                b'/' => TokenKind::Div,
                b'^' => TokenKind::Caret,
                b'\\' => TokenKind::Backslash,
                b'.' => TokenKind::Dot,
                b':' => TokenKind::Colon,
                b'<' => TokenKind::Lt,
                b'>' => TokenKind::Gt,
                b'(' => TokenKind::LParen,
                b')' => TokenKind::RParen,
                b'[' => TokenKind::LBracket,
                b']' => TokenKind::RBracket,
                b';' => TokenKind::Semi,
                b',' => TokenKind::Comma,
                b'~' => TokenKind::Tilde,
                b'|' => TokenKind::Pipe,
                other => panic!("test lexer: unexpected byte {:?}", other as char),
            };
            i += 1;
            out.push(Token::new(kind, Span::new(start, i)));
        }
        out.push(Token::new(TokenKind::Eof, Span::new(src.len(), src.len())));
        out
    }

    /// Parse a complete expression; trailing tokens are an error.
    fn parse(src: &str) -> Result<Expr> {
        let tokens = lex(src);
        let mut c = Cursor::new(&tokens, src);
        let e = parse_expr(&mut c)?;
        expect_consumed(&c)?;
        Ok(e)
    }

    fn parse_bool(src: &str) -> Result<Expr> {
        let tokens = lex(src);
        let mut c = Cursor::new(&tokens, src);
        let e = parse_bool_expr(&mut c)?;
        expect_consumed(&c)?;
        Ok(e)
    }

    fn expect_consumed(c: &Cursor<'_>) -> Result<()> {
        if c.is_eof() {
            Ok(())
        } else {
            Err(FreesError::parse_at(
                format!("unexpected {} after expression", c.peek().describe()),
                c.span(),
            ))
        }
    }

    fn ok(src: &str) -> Expr {
        parse(src).unwrap_or_else(|e| panic!("`{src}` should parse: {e}"))
    }

    fn err(src: &str) -> String {
        match parse(src) {
            Ok(e) => panic!("`{src}` should not parse, got {e:?}"),
            Err(e) => e.to_string(),
        }
    }

    fn bool_ok(src: &str) -> Expr {
        parse_bool(src).unwrap_or_else(|e| panic!("`{src}` should parse: {e}"))
    }

    fn num(v: f64) -> Expr {
        Expr::num(v)
    }

    fn var(n: &str) -> Expr {
        Expr::var(n)
    }

    fn bin(op: BinOp, l: Expr, r: Expr) -> Expr {
        Expr::bin(op, l, r)
    }

    fn neg(e: Expr) -> Expr {
        Expr::Neg(Box::new(e))
    }

    fn cmp(op: CmpOp, l: Expr, r: Expr) -> Expr {
        Expr::Compare {
            op,
            left: Box::new(l),
            right: Box::new(r),
        }
    }

    fn logic(op: LogicOp, l: Expr, r: Expr) -> Expr {
        Expr::Logical {
            op,
            left: Box::new(l),
            right: Box::new(r),
        }
    }

    fn not(e: Expr) -> Expr {
        Expr::Not(Box::new(e))
    }

    fn transpose(e: Expr) -> Expr {
        Expr::call("transpose", vec![e])
    }

    fn row(elements: Vec<Expr>) -> Expr {
        Expr::ArrayLiteral(elements)
    }

    // ── addExpr / mulExpr precedence and associativity ──────────────────────

    #[test]
    fn addition_is_left_associative() {
        assert_eq!(
            ok("a - b - c"),
            bin(BinOp::Sub, bin(BinOp::Sub, var("a"), var("b")), var("c"))
        );
        assert_eq!(
            ok("1 + 2 - 3"),
            bin(BinOp::Sub, bin(BinOp::Add, num(1.0), num(2.0)), num(3.0))
        );
    }

    #[test]
    fn multiplication_binds_tighter_than_addition() {
        assert_eq!(
            ok("a + b * c"),
            bin(BinOp::Add, var("a"), bin(BinOp::Mul, var("b"), var("c")))
        );
        assert_eq!(
            ok("a * b + c"),
            bin(BinOp::Add, bin(BinOp::Mul, var("a"), var("b")), var("c"))
        );
    }

    #[test]
    fn multiplication_is_left_associative() {
        assert_eq!(
            ok("a / b / c"),
            bin(BinOp::Div, bin(BinOp::Div, var("a"), var("b")), var("c"))
        );
        assert_eq!(
            ok("a \\ b * c"),
            bin(
                BinOp::Mul,
                bin(BinOp::LeftDiv, var("a"), var("b")),
                var("c")
            )
        );
    }

    #[test]
    fn element_wise_operators_map_to_their_binops() {
        assert_eq!(ok("a .* b"), bin(BinOp::ElemMul, var("a"), var("b")));
        assert_eq!(ok("a ./ b"), bin(BinOp::ElemDiv, var("a"), var("b")));
        assert_eq!(ok("a .\\ b"), bin(BinOp::ElemLeftDiv, var("a"), var("b")));
        assert_eq!(ok("a .^ b"), bin(BinOp::ElemPow, var("a"), var("b")));
    }

    #[test]
    fn element_wise_multiplicatives_share_the_mul_level() {
        assert_eq!(
            ok("a .* b ./ c"),
            bin(
                BinOp::ElemDiv,
                bin(BinOp::ElemMul, var("a"), var("b")),
                var("c")
            )
        );
        assert_eq!(
            ok("a + b .* c"),
            bin(
                BinOp::Add,
                var("a"),
                bin(BinOp::ElemMul, var("b"), var("c"))
            )
        );
    }

    #[test]
    fn element_wise_power_lives_at_the_pow_level() {
        // The grammar puts DOTCARET in powExpr, so `a .^ b * c` is (a.^b)*c and
        // `a * b .^ c` is a*(b.^c).
        assert_eq!(
            ok("a .^ b * c"),
            bin(
                BinOp::Mul,
                bin(BinOp::ElemPow, var("a"), var("b")),
                var("c")
            )
        );
        assert_eq!(
            ok("a * b .^ c"),
            bin(
                BinOp::Mul,
                var("a"),
                bin(BinOp::ElemPow, var("b"), var("c"))
            )
        );
    }

    // ── powExpr ─────────────────────────────────────────────────────────────

    #[test]
    fn power_is_right_associative() {
        assert_eq!(
            ok("2 ^ 3 ^ 2"),
            bin(BinOp::Pow, num(2.0), bin(BinOp::Pow, num(3.0), num(2.0)))
        );
        assert_eq!(
            ok("a ^ b ^ c ^ d"),
            bin(
                BinOp::Pow,
                var("a"),
                bin(BinOp::Pow, var("b"), bin(BinOp::Pow, var("c"), var("d")))
            )
        );
    }

    #[test]
    fn power_binds_tighter_than_multiplication_and_unary_minus() {
        assert_eq!(
            ok("a * b ^ c"),
            bin(BinOp::Mul, var("a"), bin(BinOp::Pow, var("b"), var("c")))
        );
        // -2^2 is -(2^2) = -4, never (-2)^2.
        assert_eq!(ok("-2 ^ 2"), neg(bin(BinOp::Pow, num(2.0), num(2.0))));
    }

    #[test]
    fn a_signed_exponent_needs_no_parentheses() {
        assert_eq!(ok("2 ^ -3"), bin(BinOp::Pow, num(2.0), neg(num(3.0))));
        assert_eq!(ok("2 .^ -3"), bin(BinOp::ElemPow, num(2.0), neg(num(3.0))));
    }

    // ── transpose ───────────────────────────────────────────────────────────

    #[test]
    fn transpose_becomes_a_call() {
        assert_eq!(ok("a'"), transpose(var("a")));
    }

    #[test]
    fn transpose_chains_wrap_once_per_tick() {
        assert_eq!(ok("a''"), transpose(transpose(var("a"))));
        assert_eq!(ok("a'''"), transpose(transpose(transpose(var("a")))));
    }

    #[test]
    fn transpose_binds_to_its_own_atom() {
        assert_eq!(ok("a * b'"), bin(BinOp::Mul, var("a"), transpose(var("b"))));
        // The exponent is parsed before TRANSPOSE*, so `a ^ b'` transposes b.
        assert_eq!(ok("a ^ b'"), bin(BinOp::Pow, var("a"), transpose(var("b"))));
    }

    #[test]
    fn a_power_after_a_transpose_is_a_parse_error() {
        // powExpr is `atom (CARET unaryExpr)? TRANSPOSE*`: once the transposes
        // are consumed the rule is finished, so `^ b` has nowhere to attach.
        let message = err("a' ^ b");
        assert!(message.contains("unexpected"), "{message}");
    }

    #[test]
    fn a_matrix_literal_can_be_transposed() {
        assert_eq!(
            ok("[1, 2]'"),
            transpose(row(vec![row(vec![num(1.0), num(2.0)])]))
        );
    }

    // ── unaryExpr ───────────────────────────────────────────────────────────

    #[test]
    fn unary_plus_is_a_no_op_node() {
        assert_eq!(ok("+a"), var("a"));
        assert_eq!(ok("+ -a"), neg(var("a")));
    }

    #[test]
    fn unary_minus_nests() {
        assert_eq!(ok("--a"), neg(neg(var("a"))));
    }

    #[test]
    fn unary_minus_sits_under_the_mul_level() {
        // unaryExpr is a mulExpr operand, so `-a * b` is (-a) * b.
        assert_eq!(ok("-a * b"), bin(BinOp::Mul, neg(var("a")), var("b")));
    }

    // ── atoms ───────────────────────────────────────────────────────────────

    #[test]
    fn plain_numbers_carry_no_unit() {
        assert_eq!(
            ok("140"),
            Expr::Num {
                value: 140.0,
                unit: None,
                is_imaginary: false
            }
        );
        assert_eq!(ok("1.5e3"), num(1500.0));
        assert_eq!(ok(".5"), num(0.5));
    }

    #[test]
    fn imaginary_literals_set_the_flag() {
        assert_eq!(
            ok("3i"),
            Expr::Num {
                value: 3.0,
                unit: None,
                is_imaginary: true
            }
        );
        assert_eq!(
            ok("2 + 3j"),
            bin(
                BinOp::Add,
                num(2.0),
                Expr::Num {
                    value: 3.0,
                    unit: None,
                    is_imaginary: true
                }
            )
        );
    }

    #[test]
    fn string_literals_drop_their_quotes() {
        assert_eq!(ok("'R134a'"), Expr::Str("R134a".into()));
        assert_eq!(ok("''"), Expr::Str(String::new()));
    }

    #[test]
    fn identifiers_are_lowercased() {
        assert_eq!(ok("T_in"), Expr::Var("t_in".into()));
        assert_eq!(ok("R$"), Expr::Var("r$".into()));
    }

    #[test]
    fn calls_lowercase_their_name_and_keep_argument_order() {
        assert_eq!(
            ok("Enthalpy(x, y)"),
            Expr::call("enthalpy", vec![var("x"), var("y")])
        );
        assert_eq!(ok("sin(x)"), Expr::call("sin", vec![var("x")]));
    }

    #[test]
    fn the_if_intrinsic_is_a_call() {
        // frees `If` is the five-argument EES form, not a relational condition.
        assert_eq!(
            ok("If(time, t_burn, F0, F0, 0)"),
            Expr::call(
                "if",
                vec![var("time"), var("t_burn"), var("f0"), var("f0"), num(0.0)]
            )
        );
    }

    #[test]
    fn member_access_keeps_the_dotted_path_whole() {
        assert_eq!(ok("in.P"), Expr::Var("in.p".into()));
        assert_eq!(ok("HP.out.h"), Expr::Var("hp.out.h".into()));
        // A dotted path is one variable, not an access on `hp`.
        assert_eq!(
            ok("HP.out.h").variables().into_iter().collect::<Vec<_>>(),
            vec!["hp.out.h"]
        );
    }

    #[test]
    fn array_access_lowercases_the_name() {
        assert_eq!(
            ok("Speed[N]"),
            Expr::ArrayAccess {
                name: "speed".into(),
                indices: vec![var("n")]
            }
        );
    }

    #[test]
    fn array_access_takes_several_indices() {
        assert_eq!(
            ok("A[i, j+1]"),
            Expr::ArrayAccess {
                name: "a".into(),
                indices: vec![var("i"), bin(BinOp::Add, var("j"), num(1.0))]
            }
        );
    }

    #[test]
    fn an_index_range_becomes_a_range_node() {
        assert_eq!(
            ok("A[1:3]"),
            Expr::ArrayAccess {
                name: "a".into(),
                indices: vec![Expr::Range {
                    start: Box::new(num(1.0)),
                    end: Box::new(num(3.0))
                }]
            }
        );
        assert_eq!(
            ok("A[1:N, 2]"),
            Expr::ArrayAccess {
                name: "a".into(),
                indices: vec![
                    Expr::Range {
                        start: Box::new(num(1.0)),
                        end: Box::new(var("n"))
                    },
                    num(2.0)
                ]
            }
        );
    }

    #[test]
    fn a_range_bound_may_be_an_expression() {
        assert_eq!(
            ok("A[i+1 : 2*n]"),
            Expr::ArrayAccess {
                name: "a".into(),
                indices: vec![Expr::Range {
                    start: Box::new(bin(BinOp::Add, var("i"), num(1.0))),
                    end: Box::new(bin(BinOp::Mul, num(2.0), var("n")))
                }]
            }
        );
    }

    #[test]
    fn parentheses_override_precedence_and_leave_no_node() {
        assert_eq!(
            ok("(a + b) * c"),
            bin(BinOp::Mul, bin(BinOp::Add, var("a"), var("b")), var("c"))
        );
        assert_eq!(ok("(((a)))"), var("a"));
    }

    // ── matrix literals ─────────────────────────────────────────────────────

    #[test]
    fn matrix_rows_nest_one_array_literal_per_row() {
        assert_eq!(
            ok("[1, 2; 3, 4]"),
            row(vec![
                row(vec![num(1.0), num(2.0)]),
                row(vec![num(3.0), num(4.0)]),
            ])
        );
    }

    #[test]
    fn a_single_row_is_still_wrapped() {
        assert_eq!(
            ok("[1, 2, 3]"),
            row(vec![row(vec![num(1.0), num(2.0), num(3.0)])])
        );
    }

    #[test]
    fn matrix_elements_may_be_juxtaposed() {
        assert_eq!(ok("[1 2 3]"), ok("[1, 2, 3]"));
        assert_eq!(ok("[a b]"), ok("[a, b]"));
        assert_eq!(ok("[1 2; 3 4]"), ok("[1, 2; 3, 4]"));
    }

    #[test]
    fn juxtaposition_never_splits_a_greedy_expression() {
        // `[1 -2]` is one element (1 - 2), because addExpr is greedy — exactly
        // what ANTLR does with `expr (COMMA? expr)*`.
        assert_eq!(
            ok("[1 -2]"),
            row(vec![row(vec![bin(BinOp::Sub, num(1.0), num(2.0))])])
        );
        assert_eq!(ok("[1, -2]"), row(vec![row(vec![num(1.0), neg(num(2.0))])]));
    }

    #[test]
    fn matrix_literals_nest() {
        assert_eq!(
            ok("[[1, 2], [3, 4]]"),
            row(vec![row(vec![
                row(vec![row(vec![num(1.0), num(2.0)])]),
                row(vec![row(vec![num(3.0), num(4.0)])]),
            ])])
        );
    }

    #[test]
    fn matrix_elements_may_be_arbitrary_expressions() {
        assert_eq!(
            ok("[a*2, f(b); -c, d']"),
            row(vec![
                row(vec![
                    bin(BinOp::Mul, var("a"), num(2.0)),
                    Expr::call("f", vec![var("b")])
                ]),
                row(vec![neg(var("c")), transpose(var("d"))]),
            ])
        );
    }

    #[test]
    fn a_trailing_unit_reaches_every_numeric_leaf() {
        // Matches `applyUnitToElements`: the raw unit text is attached to each
        // element and the values are *not* rescaled (unlike `NUMBER unit?`).
        assert_eq!(
            ok("[1, 2] [kg]"),
            row(vec![row(vec![
                Expr::Num {
                    value: 1.0,
                    unit: Some("kg".into()),
                    is_imaginary: false
                },
                Expr::Num {
                    value: 2.0,
                    unit: Some("kg".into()),
                    is_imaginary: false
                },
            ])])
        );
    }

    #[test]
    fn a_trailing_unit_on_a_symbolic_literal_is_rejected() {
        let message = err("[a, b] [kg]");
        assert!(message.contains("requires numeric elements"), "{message}");
    }

    // ── unit annotations ────────────────────────────────────────────────────

    fn unit_of(src: &str) -> Result<Option<String>> {
        let tokens = lex(src);
        let mut c = Cursor::new(&tokens, src);
        parse_unit_annotation(&mut c)
    }

    #[test]
    fn a_unit_annotation_is_returned_verbatim() {
        assert_eq!(unit_of("[kPa]").unwrap(), Some("kPa".into()));
        assert_eq!(unit_of("[kJ/kg-K]").unwrap(), Some("kJ/kg-K".into()));
        assert_eq!(unit_of("[m^3/s]").unwrap(), Some("m^3/s".into()));
        assert_eq!(unit_of("[lbm/ft^3]").unwrap(), Some("lbm/ft^3".into()));
        assert_eq!(unit_of("[1/s]").unwrap(), Some("1/s".into()));
    }

    #[test]
    fn a_unit_annotation_is_trimmed_but_keeps_inner_spelling() {
        assert_eq!(unit_of("[  kPa  ]").unwrap(), Some("kPa".into()));
        assert_eq!(unit_of("[kJ / kg]").unwrap(), Some("kJ / kg".into()));
    }

    #[test]
    fn an_empty_annotation_is_no_unit() {
        assert_eq!(unit_of("[]").unwrap(), None);
        assert_eq!(unit_of("[   ]").unwrap(), None);
    }

    #[test]
    fn a_missing_annotation_is_no_unit_and_consumes_nothing() {
        let src = "kPa";
        let tokens = lex(src);
        let mut c = Cursor::new(&tokens, src);
        assert_eq!(parse_unit_annotation(&mut c).unwrap(), None);
        assert_eq!(c.pos(), 0);
    }

    #[test]
    fn a_unit_annotation_rejects_foreign_tokens() {
        let message = unit_of("[kPa;]").unwrap_err().to_string();
        assert!(message.contains("unit expression"), "{message}");
        let unterminated = unit_of("[kPa").unwrap_err().to_string();
        assert!(unterminated.contains("unit expression"), "{unterminated}");
    }

    // ── SI conversion of literals (needs the unit registry) ─────────────────

    #[test]
    fn a_unit_annotated_literal_is_converted_to_si_at_parse_time() {
        assert_eq!(
            ok("140 [kPa]"),
            Expr::Num {
                value: 140_000.0,
                unit: Some("Pa".into()),
                is_imaginary: false
            }
        );
    }

    #[test]
    fn an_offset_scale_literal_uses_factor_then_offset() {
        match ok("25 [C]") {
            Expr::Num { value, unit, .. } => {
                assert!((value - 298.15).abs() < 1e-9, "{value}");
                assert_eq!(unit.as_deref(), Some("K"));
            }
            other => panic!("expected a literal, got {other:?}"),
        }
    }

    #[test]
    fn an_unknown_unit_keeps_the_original_value_and_text() {
        assert_eq!(
            ok("5 [flurbles]"),
            Expr::Num {
                value: 5.0,
                unit: Some("flurbles".into()),
                is_imaginary: false
            }
        );
    }

    #[test]
    fn imaginary_literals_convert_too() {
        match ok("2i [kPa]") {
            Expr::Num {
                value,
                unit,
                is_imaginary,
            } => {
                assert!((value - 2000.0).abs() < 1e-9, "{value}");
                assert_eq!(unit.as_deref(), Some("Pa"));
                assert!(is_imaginary);
            }
            other => panic!("expected a literal, got {other:?}"),
        }
    }

    // ── the -10 [C] trap ────────────────────────────────────────────────────

    #[test]
    fn a_sign_folds_into_a_bare_offset_unit_literal() {
        // -10 °C is 263.15 K, never -(283.15 K).
        match ok("-10 [C]") {
            Expr::Num { value, unit, .. } => {
                assert!((value - 263.15).abs() < 1e-9, "{value}");
                assert_eq!(unit.as_deref(), Some("K"));
            }
            other => panic!("the sign must fold into the literal, got {other:?}"),
        }
    }

    #[test]
    fn a_double_sign_keeps_operator_semantics() {
        // `--10 [C]` is Neg(fold(-10 [C])) = -263.15, not 283.15.
        match ok("--10 [C]") {
            Expr::Neg(inner) => match *inner {
                Expr::Num { value, .. } => assert!((value - 263.15).abs() < 1e-9, "{value}"),
                other => panic!("expected a folded literal under the Neg, got {other:?}"),
            },
            other => panic!("expected a Neg, got {other:?}"),
        }
    }

    #[test]
    fn an_exponent_defeats_the_fold() {
        // `-10 [C]^2` is -((283.15 K)^2): the literal converts normally.
        match ok("-10 [C]^2") {
            Expr::Neg(inner) => match *inner {
                Expr::BinOp { op, left, right } => {
                    assert_eq!(op, BinOp::Pow);
                    match *left {
                        Expr::Num { value, .. } => {
                            assert!((value - 283.15).abs() < 1e-9, "{value}")
                        }
                        other => panic!("expected the converted base, got {other:?}"),
                    }
                    assert_eq!(*right, num(2.0));
                }
                other => panic!("expected a power, got {other:?}"),
            },
            other => panic!("expected a Neg, got {other:?}"),
        }
    }

    #[test]
    fn a_transpose_defeats_the_fold() {
        match ok("-10 [C]'") {
            Expr::Neg(inner) => match *inner {
                Expr::Call { ref function, .. } => assert_eq!(function, "transpose"),
                ref other => panic!("expected a transpose, got {other:?}"),
            },
            other => panic!("expected a Neg, got {other:?}"),
        }
    }

    #[test]
    fn a_purely_multiplicative_unit_keeps_the_neg_path() {
        // Nothing to fold: negation commutes with a pure factor, so the Java
        // builder leaves the Neg in place.
        assert_eq!(
            ok("-10 [kPa]"),
            neg(Expr::Num {
                value: 10_000.0,
                unit: Some("Pa".into()),
                is_imaginary: false
            })
        );
    }

    #[test]
    fn an_unknown_unit_keeps_the_neg_path() {
        assert_eq!(
            ok("-10 [flurbles]"),
            neg(Expr::Num {
                value: 10.0,
                unit: Some("flurbles".into()),
                is_imaginary: false
            })
        );
    }

    #[test]
    fn the_fold_covers_exactly_the_literal() {
        // `* 2` belongs to mulExpr, not to the folded unaryExpr.
        match ok("-10 [C] * 2") {
            Expr::BinOp { op, left, right } => {
                assert_eq!(op, BinOp::Mul);
                match *left {
                    Expr::Num { value, .. } => assert!((value - 263.15).abs() < 1e-9, "{value}"),
                    other => panic!("expected the folded literal, got {other:?}"),
                }
                assert_eq!(*right, num(2.0));
            }
            other => panic!("expected a product, got {other:?}"),
        }
    }

    #[test]
    fn an_empty_annotation_after_a_sign_keeps_the_neg_path() {
        // No registry needed: `[]` is not a unit, so the fold bails out early.
        assert_eq!(ok("-10 []"), neg(num(10.0)));
    }

    #[test]
    fn a_sign_in_front_of_a_plain_number_is_never_folded() {
        assert_eq!(ok("-10"), neg(num(10.0)));
    }

    // ── named arguments / property calls ────────────────────────────────────

    #[test]
    fn named_arguments_encode_a_property_call() {
        assert_eq!(
            ok("Enthalpy(R134a, T=T1, x=1)"),
            Expr::call("prop$enthalpy$r134a$t$x", vec![var("t1"), num(1.0)])
        );
    }

    #[test]
    fn a_property_call_hides_the_fluid_from_the_variable_set() {
        let vars: Vec<_> = ok("Enthalpy(R134a, T=T1, x=1)")
            .variables()
            .into_iter()
            .collect();
        assert_eq!(vars, vec!["t1"]);
    }

    #[test]
    fn a_quoted_fluid_name_is_unquoted_before_encoding() {
        assert_eq!(
            ok("Enthalpy('R134a', T=300)"),
            Expr::call("prop$enthalpy$r134a$t", vec![num(300.0)])
        );
    }

    #[test]
    fn a_string_variable_may_name_the_fluid() {
        assert_eq!(
            ok("Enthalpy(R$, T=300)"),
            Expr::call("prop$enthalpy$r$$t", vec![num(300.0)])
        );
    }

    #[test]
    fn the_fluid_must_come_first_and_be_a_name() {
        let message = err("Enthalpy(1+2, T=300)");
        assert!(message.contains("Invalid fluid name '1+2'"), "{message}");

        let named_first = err("Enthalpy(T=300, x=1)");
        assert!(
            named_first.contains("take the fluid name first"),
            "{named_first}"
        );
    }

    #[test]
    fn property_indicators_after_the_fluid_must_be_named() {
        let message = err("Enthalpy(R134a, T=300, 5)");
        assert!(
            message.contains("Property indicators must be named"),
            "{message}"
        );
    }

    #[test]
    fn named_arguments_are_rejected_by_the_if_intrinsic() {
        let message = err("If(x=1, 2, 3)");
        assert!(
            message.contains("only valid in fluid property functions: if"),
            "{message}"
        );
    }

    // ── chemistry calls ─────────────────────────────────────────────────────

    #[test]
    fn chemistry_calls_preserve_token_case() {
        assert_eq!(
            ok("MolarMass(C8H18)"),
            Expr::call("prop$molarmass", vec![Expr::Str("C8H18".into())])
        );
        assert_eq!(
            ok("HeatingValue(CH4, 'LHV')"),
            Expr::call(
                "prop$heatingvalue",
                vec![Expr::Str("CH4".into()), Expr::Str("LHV".into())]
            )
        );
    }

    #[test]
    fn a_chemistry_token_never_becomes_a_variable() {
        assert!(ok("MolarMass(C8H18)").variables().is_empty());
    }

    #[test]
    fn a_quoted_formula_with_parentheses_is_accepted() {
        assert_eq!(
            ok("MolarMass('Ca(OH)2')"),
            Expr::call("prop$molarmass", vec![Expr::Str("Ca(OH)2".into())])
        );
    }

    #[test]
    fn an_invalid_chemistry_token_is_rejected() {
        let message = err("MolarMass('2H2O')");
        assert!(message.contains("Invalid token '2H2O'"), "{message}");
    }

    #[test]
    fn a_quoted_token_keeps_its_interior_spacing() {
        // The space survives into the token, so the formula check rejects it —
        // it must not be silently glued into `StainlessSteel`.
        let message = err("k_('Stainless Steel')");
        assert!(
            message.contains("Invalid token 'Stainless Steel'"),
            "{message}"
        );
    }

    // ── ConvertTemp / Convert ───────────────────────────────────────────────

    #[test]
    fn convert_temp_folds_a_numeric_argument() {
        assert_eq!(
            ok("ConvertTemp(C, K, 25)"),
            Expr::Num {
                value: 298.15,
                unit: Some("K".into()),
                is_imaginary: false
            }
        );
        match ok("ConvertTemp(C, F, 25)") {
            Expr::Num { value, unit, .. } => {
                assert!((value - 77.0).abs() < 1e-9, "{value}");
                assert_eq!(unit, None);
            }
            other => panic!("expected a literal, got {other:?}"),
        }
    }

    #[test]
    fn convert_temp_emits_an_affine_expression_for_a_variable() {
        // C→K is x + 273.15: the scale is 1, so no multiply node is emitted.
        assert_eq!(
            ok("ConvertTemp(C, K, T)"),
            bin(BinOp::Add, var("t"), num(273.15))
        );
        // K→K is the identity.
        assert_eq!(ok("ConvertTemp(K, K, T)"), var("t"));
        // C→F scales and shifts: 9/5 * T + 32 (the offset is accumulated in
        // floating point exactly as the Java builder does it).
        match ok("ConvertTemp(C, F, T)") {
            Expr::BinOp { op, left, right } => {
                assert_eq!(op, BinOp::Add);
                assert_eq!(*left, bin(BinOp::Mul, num(9.0 / 5.0), var("t")));
                match *right {
                    Expr::Num { value, .. } => assert!((value - 32.0).abs() < 1e-9, "{value}"),
                    other => panic!("expected the offset, got {other:?}"),
                }
            }
            other => panic!("expected an affine expression, got {other:?}"),
        }
    }

    #[test]
    fn convert_temp_rejects_unknown_scales_and_arities() {
        let scale = err("ConvertTemp(X, K, 25)");
        assert!(scale.contains("unknown temperature scale 'X'"), "{scale}");
        let arity = err("ConvertTemp(C, K)");
        assert!(arity.contains("three arguments"), "{arity}");
    }

    #[test]
    fn convert_folds_to_a_scale_factor() {
        match ok("Convert(kJ, J)") {
            Expr::Num { value, unit, .. } => {
                assert!((value - 1000.0).abs() < 1e-9, "{value}");
                assert_eq!(unit, None);
            }
            other => panic!("expected a literal, got {other:?}"),
        }
    }

    #[test]
    fn convert_rejects_bad_arity_and_dimension_mismatches() {
        let arity = err("Convert(kJ)");
        assert!(arity.contains("exactly two unit arguments"), "{arity}");
        let mismatch = err("Convert(kJ, m)");
        assert!(mismatch.contains("different dimensions"), "{mismatch}");
    }

    // ── boolExpr ────────────────────────────────────────────────────────────

    #[test]
    fn every_relational_operator_maps_to_its_cmpop() {
        for (src, op) in [
            ("a < b", CmpOp::Lt),
            ("a > b", CmpOp::Gt),
            ("a <= b", CmpOp::Le),
            ("a >= b", CmpOp::Ge),
            ("a <> b", CmpOp::Ne),
            ("a = b", CmpOp::Eq),
        ] {
            assert_eq!(bool_ok(src), cmp(op, var("a"), var("b")), "{src}");
        }
    }

    #[test]
    fn a_relation_takes_whole_arithmetic_expressions() {
        assert_eq!(
            bool_ok("a + b > c * 2"),
            cmp(
                CmpOp::Gt,
                bin(BinOp::Add, var("a"), var("b")),
                bin(BinOp::Mul, var("c"), num(2.0))
            )
        );
    }

    #[test]
    fn a_bare_arithmetic_expression_is_a_valid_condition() {
        assert_eq!(bool_ok("a + 1"), bin(BinOp::Add, var("a"), num(1.0)));
    }

    #[test]
    fn and_binds_tighter_than_or() {
        assert_eq!(
            bool_ok("a > 1 AND b < 2 OR c = 3"),
            logic(
                LogicOp::Or,
                logic(
                    LogicOp::And,
                    cmp(CmpOp::Gt, var("a"), num(1.0)),
                    cmp(CmpOp::Lt, var("b"), num(2.0))
                ),
                cmp(CmpOp::Eq, var("c"), num(3.0))
            )
        );
        assert_eq!(
            bool_ok("a OR b AND c"),
            logic(
                LogicOp::Or,
                var("a"),
                logic(LogicOp::And, var("b"), var("c"))
            )
        );
    }

    #[test]
    fn connectives_are_left_associative_and_case_insensitive() {
        assert_eq!(
            bool_ok("a and b and c"),
            logic(
                LogicOp::And,
                logic(LogicOp::And, var("a"), var("b")),
                var("c")
            )
        );
        assert_eq!(
            bool_ok("a or b or c"),
            logic(
                LogicOp::Or,
                logic(LogicOp::Or, var("a"), var("b")),
                var("c")
            )
        );
        assert_eq!(bool_ok("a OR b"), bool_ok("a or b"));
    }

    #[test]
    fn not_binds_tighter_than_and() {
        assert_eq!(
            bool_ok("NOT a AND b"),
            logic(LogicOp::And, not(var("a")), var("b"))
        );
    }

    #[test]
    fn not_takes_a_whole_relation_and_nests() {
        assert_eq!(
            bool_ok("NOT a > 1"),
            not(cmp(CmpOp::Gt, var("a"), num(1.0)))
        );
        assert_eq!(bool_ok("NOT NOT a"), not(not(var("a"))));
    }

    #[test]
    fn parenthesised_conditions_group_connectives() {
        assert_eq!(
            bool_ok("NOT (a AND b)"),
            not(logic(LogicOp::And, var("a"), var("b")))
        );
        assert_eq!(
            bool_ok("(a > 1) AND (b < 2)"),
            logic(
                LogicOp::And,
                cmp(CmpOp::Gt, var("a"), num(1.0)),
                cmp(CmpOp::Lt, var("b"), num(2.0))
            )
        );
        assert_eq!(
            bool_ok("a AND (b OR c)"),
            logic(
                LogicOp::And,
                var("a"),
                logic(LogicOp::Or, var("b"), var("c"))
            )
        );
    }

    #[test]
    fn a_parenthesised_atom_is_not_mistaken_for_a_grouped_condition() {
        // `( … )` followed by an arithmetic operator is an atom: the
        // speculative bool-paren parse has to rewind.
        assert_eq!(
            bool_ok("(a + b) * 2 > 5"),
            cmp(
                CmpOp::Gt,
                bin(BinOp::Mul, bin(BinOp::Add, var("a"), var("b")), num(2.0)),
                num(5.0)
            )
        );
        assert_eq!(bool_ok("(a) = 3"), cmp(CmpOp::Eq, var("a"), num(3.0)));
        assert_eq!(
            bool_ok("(a)' = 3"),
            cmp(CmpOp::Eq, transpose(var("a")), num(3.0))
        );
        // Parentheses vanish either way when nothing follows.
        assert_eq!(bool_ok("(a)"), var("a"));
    }

    #[test]
    fn a_relation_is_not_an_arithmetic_expression() {
        let message = err("a > b");
        assert!(message.contains("unexpected"), "{message}");
    }

    // ── error paths ─────────────────────────────────────────────────────────

    #[test]
    fn a_missing_operand_is_reported_with_the_found_token() {
        let message = err("a +");
        assert!(
            message.contains("expected an expression, found end of input"),
            "{message}"
        );
        let leading = err("* a");
        assert!(
            leading.contains("expected an expression, found `*`"),
            "{leading}"
        );
    }

    #[test]
    fn an_unclosed_group_names_the_missing_token() {
        for (src, needle) in [
            ("(a + b", "expected `)`"),
            ("f(a", "expected `)`"),
            ("A[1", "expected `]`"),
            ("[1, 2", "expected `]`"),
        ] {
            let message = err(src);
            assert!(message.contains(needle), "`{src}`: {message}");
        }
    }

    #[test]
    fn a_call_needs_at_least_one_argument() {
        // `argList : arg (COMMA arg)*` — the grammar has no empty alternative.
        let message = err("f()");
        assert!(
            message.contains("expected an expression, found `)`"),
            "{message}"
        );
    }

    #[test]
    fn a_dangling_member_segment_is_reported() {
        let message = err("in.");
        assert!(message.contains("expected an identifier"), "{message}");
    }

    #[test]
    fn errors_carry_a_span_into_the_source() {
        let src = "a + * b";
        let tokens = lex(src);
        let mut c = Cursor::new(&tokens, src);
        let error = parse_expr(&mut c).unwrap_err();
        let span = error.span().expect("parse errors are source-mapped");
        assert_eq!(span.slice(src), "*");
        assert_eq!(span.line_col(src), (1, 5));
    }

    #[test]
    fn a_bool_expr_error_survives_the_speculative_paren_parse() {
        // The rewind must not swallow the real diagnostic.
        let tokens = lex("(a + ) > 1");
        let mut c = Cursor::new(&tokens, "(a + ) > 1");
        let error = parse_bool_expr(&mut c).unwrap_err();
        assert!(
            error.to_string().contains("expected an expression"),
            "{error}"
        );
    }

    // ── mixed / regression ──────────────────────────────────────────────────

    #[test]
    fn a_realistic_equation_side_parses_with_the_expected_shape() {
        assert_eq!(
            ok("-b + sqrt(b^2 - 4*a*c) / (2*a)"),
            bin(
                BinOp::Add,
                neg(var("b")),
                bin(
                    BinOp::Div,
                    Expr::call(
                        "sqrt",
                        vec![bin(
                            BinOp::Sub,
                            bin(BinOp::Pow, var("b"), num(2.0)),
                            bin(BinOp::Mul, bin(BinOp::Mul, num(4.0), var("a")), var("c"))
                        )]
                    ),
                    bin(BinOp::Mul, num(2.0), var("a"))
                )
            )
        );
    }

    #[test]
    fn variables_are_visible_through_the_parsed_tree() {
        let vars: Vec<_> = ok("sum(i, 1, N, A[i]) + Q_dot")
            .variables()
            .into_iter()
            .collect();
        assert_eq!(vars, vec!["a", "n", "q_dot"]);
    }

    #[test]
    fn a_call_argument_may_be_any_expression() {
        assert_eq!(
            ok("f(a + b, -c, [1, 2], g(h))"),
            Expr::call(
                "f",
                vec![
                    bin(BinOp::Add, var("a"), var("b")),
                    neg(var("c")),
                    row(vec![row(vec![num(1.0), num(2.0)])]),
                    Expr::call("g", vec![var("h")]),
                ]
            )
        );
    }

    #[test]
    fn comments_and_padding_do_not_disturb_spans_or_units() {
        // The unit slice is taken straight out of the source, so surrounding
        // trivia must not leak into it.
        assert_eq!(unit_of("[ kJ/kg-K ]").unwrap(), Some("kJ/kg-K".into()));
        assert_eq!(ok("a  {note}  +  b"), bin(BinOp::Add, var("a"), var("b")));
    }
}
