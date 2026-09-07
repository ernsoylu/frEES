//! The thirteen CAS operations, over an internal rational IR.
//!
//! Port target: `../frEES/backend/core/.../cas/{CasEngine,ExprToSymja,
//! SymjaOutputNormalizer,CasExpressions}.java` — but **not** a transcription.
//! The Java round-trips `frees Expr -> Symja string -> Symja -> string ->
//! frees Expr`. That bridge is gone. Here an expression is lowered into
//! [`crate::cas::poly`] / [`crate::cas::ratfun`] — exact rational polynomial
//! algebra over ℚ — operated on, and lifted straight back into an [`Expr`]. No
//! string is parsed at any point in an operation.
//!
//! # What the REPL actually shows
//!
//! `ReplEvaluator.casDisplay` returns **Symja's own printed output** with
//! `Log(`/`Log10(` rewritten to `ln(`/`log10(`. Those strings are the
//! observable behaviour, so [`display`] reproduces Symja's spelling rather than
//! inventing one. Every rule below was read off the real engine
//! (the golden-dumper classpath driving `CasEngine` directly), never guessed:
//!
//! * **Sum order** — ascending lexicographic on the exponent vector read from
//!   the *last* variable to the first, variables sorted ascending. In one
//!   variable that is "constant first, then rising powers" (`1+3*x+3*x^2+x^3`);
//!   in several it reproduces Symja exactly (`x^2+2*x*y+y^2`,
//!   `a*c+b*c+a*d+b*d`, `(b*c+a*d)/(b*d)`).
//! * **Negative terms print as subtraction** — `-1+x`, `1-2*x^2+x^4`,
//!   `2/(1+s)-1/(2+s)`.
//! * **Monomial coefficient `p/q`** — `q == 1` gives `p*m`; `|p| == 1` gives
//!   `m/q` (`x^3/3`, `x/2`, `-Cos(3*x)/3`); otherwise `p/q*m` (`2/3*x`,
//!   `5/6*x`). Confirmed by `Expand[(x+1)^2/3]` = `1/3+2/3*x+x^2/3`.
//! * **A factored product prints its content in front** — `2*(1+x)^2`,
//!   `1/6*(2+3*x)`, `1/4*(-2+x)*(2+x)`, `-2*(1+x)^2`. A single factor of
//!   multiplicity one with content ±1 is printed expanded instead (`1+x^2`,
//!   `-2+x^3`, `-1-x`).
//! * **Factor order** — descending-monomial term sequences compared
//!   lexicographically, shorter first: `(-1+x)*(1+x)`, `(-3+x)*(-2+x)*(-1+x)`,
//!   `(1+2*x)*(1+3*x)`, `x*y*(x+y)`.
//! * **`Apart` terms fold the coefficient denominator into the printed
//!   denominator** — `1/(2*(1+s))`, not `1/2*1/(1+s)`. Fourteen of the fifteen
//!   probed `Apart` results come back byte-identical.
//!
//! # Where this deliberately differs from Symja
//!
//! Recorded rather than papered over.
//!
//! 1. **`Integrate` refuses instead of guessing.** The table on [`integrate`]
//!    is the whole of it. Symja reaches `Erfi`, `SinIntegral`,
//!    `ExpIntegralEi`, `FresnelS`, integration by parts and `ArcTan`/`ArcTanh`
//!    forms; every one of those is refused **by name** here. A CAS that invents
//!    an antiderivative is worse than one that declines.
//! 2. **No transcendental identities.** `Simplify(sin(x)^2+cos(x)^2)` stays
//!    `Cos(x)^2+Sin(x)^2`; Symja returns `1`. `Exp(Log(x))`, `Exp(x)*Exp(y)`
//!    and `Sqrt(4)` likewise stay put. The core is rational algebra over ℚ with
//!    every other call treated as an opaque generator.
//! 3. **Sign placement inside `Factor`.** Each factor is normalised to a
//!    positive leading coefficient and the sign lives in the content, so
//!    `Factor(100*x^2-100)` is `100*(-1+x)*(1+x)` where Symja spells the same
//!    polynomial `-100*(1-x)*(1+x)`.
//! 4. **Rational inputs to `Expand`/`Cancel` stay one fraction.** Symja
//!    distributes the numerator over the denominator
//!    (`Cancel[(x+1)/(x+2)]` = `1/(2+x)+x/(2+x)`); that behaviour is
//!    syntax-driven and cannot be reproduced from a canonical IR.
//! 5. **Non-integer literals are exact.** `1.5` becomes `3/2`, so
//!    `Factor(1.5*x^2-1.5)` factors. Symja switches to machine floats and
//!    returns `-1.5*(1.0-x^2)`.
//! 6. **Multivariate `Factor` only pulls out the content and the monomial
//!    gcd.** `cas::poly` provides univariate factorisation over ℚ and says so;
//!    a genuinely multivariate polynomial is therefore left expanded, so
//!    `Factor(x^2-y^2)` stays `x^2-y^2` where Symja gives `(x-y)*(x+y)`.
//!    Likewise `Factor(x^3-y^3)`, `Factor(x*y+x+y+1)` and
//!    `Factor(x^2-2*x*y+y^2-1)`. The result is always *correct* — an
//!    un-factored polynomial equals itself — but a caller cannot tell it apart
//!    from a genuinely irreducible one.
//!    Everything whose interesting part is univariate still factors —
//!    `Factor(a*x^2+2*a*x+a)` = `a*(1+x)^2`, `Factor(x^2*y+x*y^2)` =
//!    `x*y*(x+y)`.
//!
//! # Wire contract — what this module needs from `ratfun`
//!
//! `poly` is in hand and this module is written against its real API
//! (`Rat = BigRational`, `UPoly`, `MPoly`, `Mono`, `Factorization`).
//! `ratfun` is not, so it is used through exactly these ten items and nothing
//! else:
//!
//! ```text
//! cas::ratfun::RatFun   from_poly(MPoly) -> RatFun
//!                       new(MPoly, MPoly) -> Option<RatFun>   // None iff den == 0
//!                       numer(&self) -> &MPoly
//!                       denom(&self) -> &MPoly
//!                       add/sub/mul(&self, &RatFun) -> RatFun
//!                       div(&self, &RatFun) -> Option<RatFun> // None iff rhs == 0
//!                       neg(&self) -> RatFun
//!                       pow(&self, i32) -> Option<RatFun>     // None iff 0^negative
//!                       Clone + Debug
//! ```
//!
//! **Normalisation this module relies on:** `numer`/`denom` share no factor,
//! and `denom` is a *primitive integer* polynomial with a positive leading
//! coefficient — every rational coefficient lives in `numer`. Consequently
//! `denom().as_constant() == Some(1)` exactly when the value is a polynomial,
//! which is the test `Expand`, `Collect`, `Simplify` and `Integrate` all
//! branch on.
//!
//! Partial fractions are **not** in the contract: [`apart`] is implemented here
//! on `UPoly`, which already has `factor`, `div_rem` and `gcd`.
//!
//! A generator name is minted by [`Gens`]: a frees variable contributes its own
//! lowercase name, any other atom (`Sin(x)`, `E^x`, `x^(3/2)`) contributes its
//! printed form. That keeps `poly`/`ratfun` free of any knowledge of [`Expr`].

use std::cmp::Ordering;
use std::collections::BTreeMap;

use num_bigint::BigInt;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::ast::{BinOp, Expr};
use crate::cas::engine::CasError;
use crate::cas::poly::{rat_int, MPoly, Rat};
use crate::cas::ratfun::RatFun;

/// Deepest expression the lowering will walk.
///
/// Same ceiling as [`crate::parser::expr::MAX_EXPR_DEPTH`], for the same
/// reason: a tree this module builds is later walked by `eval`,
/// `Expr::variables` and `Drop`, all of which are calibrated against it. The
/// Java bounds the equivalent risk with Symja's `MAX_RECURSION = 256` plus a
/// three-second timeout; wasm has no timeout to fall back on, so the guard has
/// to be structural.
const MAX_CAS_DEPTH: u32 = 256;

/// Largest integer exponent `^` will expand rather than treat as opaque.
///
/// `(x+1)^100000` is a legal parse and a denial-of-service otherwise.
const MAX_POW: i64 = 64;

/// Largest number of monomials any intermediate polynomial may carry.
const MAX_TERMS: usize = 4096;

/// Largest linear system [`apart`] will solve — the degree of the denominator.
const MAX_APART_DEGREE: usize = 64;

// Ledger item 25 (Wave C5): the ceilings above are right, but "declined, not
// proved" has to be *sayable*. When a ceiling turns an operation into a no-op
// — an oversized exponent interning opaquely, an over-degree Apart returning
// its input — the decline is recorded here and drained by
// `engine::apply_expr` into `CasResult::note`, which the REPL prints under
// the value. A thread-local because the lowering is deeply recursive and the
// wasm build is single-threaded anyway; the native test runner is not, which
// is why this must not be a `static Cell`.
std::thread_local! {
    static CEILING_NOTE: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

pub(crate) fn clear_ceiling_note() {
    CEILING_NOTE.with(|note| *note.borrow_mut() = None);
}

pub(crate) fn take_ceiling_note() -> Option<String> {
    CEILING_NOTE.with(|note| note.borrow_mut().take())
}

fn record_ceiling_note(message: String) {
    CEILING_NOTE.with(|note| {
        let mut slot = note.borrow_mut();
        if slot.is_none() {
            *slot = Some(message);
        }
    });
}

// ── generators ──────────────────────────────────────────────────────────────

/// The generator table: every indeterminate the polynomial core sees.
///
/// A generator is either a frees variable (name = the variable) or an atom the
/// rational core cannot open up — a function call, a symbolic power. Naming a
/// generator by its *printed* form makes the polynomial core's alphabetical
/// variable ordering agree with Symja's (`Cos(x)` before `Sin(x)`, so
/// `D[Sin(x)*Cos(x), x]` prints `Cos(x)^2-Sin(x)^2`) and keeps `poly`/`ratfun`
/// free of [`Expr`].
#[derive(Debug, Default, Clone)]
pub struct Gens {
    entries: Vec<(String, Expr)>,
}

impl Gens {
    pub fn new() -> Gens {
        Gens {
            entries: Vec::new(),
        }
    }

    /// The generator name for `e`, interning it on first sight.
    fn intern(&mut self, e: Expr) -> String {
        let name = match &e {
            Expr::Var(v) => v.clone(),
            other => display(other),
        };
        if !self.entries.iter().any(|(n, _)| *n == name) {
            self.entries.push((name.clone(), e));
        }
        name
    }

    /// The expression a generator name stands for.
    fn expr(&self, name: &str) -> Expr {
        self.entries
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, e)| e.clone())
            // Unreachable for names this table minted; a bare variable is the
            // only sane fallback and keeps the printer total.
            .unwrap_or_else(|| Expr::Var(name.to_string()))
    }

    /// True when the generator depends on `var` — the guard that stops
    /// [`integrate`] treating `Sin(x)` as a constant with respect to `x`.
    fn depends_on(&self, name: &str, var: &str) -> bool {
        if name == var {
            return true;
        }
        self.entries
            .iter()
            .find(|(n, _)| n == name)
            .is_some_and(|(_, e)| e.variables().contains(var))
    }
}

// ── a term view over MPoly ──────────────────────────────────────────────────

/// A monomial as `(generator, exponent)` pairs, ascending by name, no zero
/// exponents. `MPoly` stores a dense exponent vector aligned to its own
/// variable list; this is the sparse view the printer and the pattern matchers
/// want.
type Mono = Vec<(String, u32)>;

fn mpoly_terms(p: &MPoly) -> Vec<(Rat, Mono)> {
    let vars = p.vars();
    p.terms()
        .filter(|(_, c)| !c.is_zero())
        .map(|(m, c)| {
            let mut mono: Mono = (0..vars.len())
                .filter_map(|i| {
                    let e = m.exponent(i);
                    if e == 0 {
                        None
                    } else {
                        Some((vars[i].clone(), e))
                    }
                })
                .collect();
            mono.sort();
            (c.clone(), mono)
        })
        .collect()
}

fn mpoly_from_terms(terms: &[(Rat, Mono)]) -> MPoly {
    let mut acc = MPoly::zero();
    for (c, mono) in terms {
        if c.is_zero() {
            continue;
        }
        let mut term = MPoly::constant(c.clone());
        for (v, e) in mono {
            term = &term * &MPoly::var(v).pow(*e as usize);
        }
        acc = &acc + &term;
    }
    acc
}

fn exponent_of(m: &Mono, var: &str) -> u32 {
    m.iter()
        .find(|(v, _)| v == var)
        .map(|(_, e)| *e)
        .unwrap_or(0)
}

/// Every generator actually occurring in `p`, ascending.
fn poly_vars(p: &MPoly) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for (_, mono) in mpoly_terms(p) {
        for (v, _) in mono {
            if !seen.contains(&v) {
                seen.push(v);
            }
        }
    }
    seen.sort();
    seen
}

fn constant_value(p: &MPoly) -> Option<Rat> {
    p.as_constant()
}

// ── lowering: Expr -> RatFun ────────────────────────────────────────────────

/// Reject a node the CAS layer cannot handle, with the parent engine's exact
/// wording (`ExprToSymja.write`'s `UnsupportedExpression` messages), because
/// those strings reach the user through `"CAS: " + message`.
fn unsupported(e: &Expr) -> Option<CasError> {
    let msg = match e {
        Expr::Str(_) => "string literals are not supported in CAS expressions",
        Expr::ArrayAccess { .. } => "array indexing is not supported in CAS expressions",
        Expr::ArrayLiteral(_) => "array/matrix literals are not supported in CAS expressions",
        Expr::Range { .. } => "ranges are not supported in CAS expressions",
        Expr::Compare { .. } => "comparisons are not supported in CAS expressions",
        Expr::Logical { .. } => "logical operators are not supported in CAS expressions",
        Expr::Not(_) => "boolean negation is not supported in CAS expressions",
        _ => return None,
    };
    Some(CasError::unsupported(msg))
}

/// frees function name (lowercase) → the name Symja prints, already carrying
/// `SymjaOutputNormalizer`'s two rewrites (`Log`→`ln`, `Log10`→`log10`).
///
/// Transcribed from `ExprToSymja.FUNCTION_NAMES`; membership is also the
/// admissibility test, so an unlisted call is refused exactly as the Java
/// refuses it.
const FUNCTION_NAMES: [(&str, &str); 19] = [
    ("ln", "ln"),
    ("log10", "log10"),
    ("log2", "Log2"),
    ("sqrt", "Sqrt"),
    ("cbrt", "CubeRoot"),
    ("exp", "Exp"),
    ("abs", "Abs"),
    ("sin", "Sin"),
    ("cos", "Cos"),
    ("tan", "Tan"),
    ("arcsin", "ArcSin"),
    ("arccos", "ArcCos"),
    ("arctan", "ArcTan"),
    ("sinh", "Sinh"),
    ("cosh", "Cosh"),
    ("tanh", "Tanh"),
    ("arcsinh", "ArcSinh"),
    ("arccosh", "ArcCosh"),
    ("arctanh", "ArcTanh"),
];

fn symja_function_name(frees: &str) -> Option<&'static str> {
    FUNCTION_NAMES
        .iter()
        .find(|(f, _)| *f == frees)
        .map(|(_, s)| *s)
}

/// An `f64` literal as an exact rational.
///
/// `ExprToSymja.number` emits an integral double without a decimal point so
/// Symja factors over the integers, and any other value as a decimal string
/// that Symja reads as a machine float — at which point `Factor` gives up
/// (`Factor(1.5*x^2-1.5)` = `-1.5*(1.0-x^2)`). This port keeps the integral
/// case identical and makes the fractional case *exact* by reading the shortest
/// round-tripping decimal: `1.5` is `3/2`, not
/// `3377699720527872/2251799813685248`. Deliberate divergence, item 5 above.
fn rational_from_f64(v: f64) -> Option<Rat> {
    if !v.is_finite() {
        return None;
    }
    let text = format!("{v}");
    // Rust's `Display` for f64 emits either `-?\d+` or `-?\d+\.\d+`, or an
    // exponent form for extreme magnitudes; the exponent form is refused rather
    // than mis-parsed.
    let (mantissa, scale) = match text.split_once('.') {
        None => (text.clone(), 0u32),
        Some((int, frac)) => {
            if frac.len() > 18 {
                return None;
            }
            (format!("{int}{frac}"), frac.len() as u32)
        }
    };
    let numerator: BigInt = mantissa.parse().ok()?;
    let denominator = BigInt::from(10u32).pow(scale);
    Some(Rat::new(numerator, denominator))
}

struct Lower<'g> {
    gens: &'g mut Gens,
    depth: u32,
}

impl Lower<'_> {
    fn lower(&mut self, e: &Expr) -> Result<RatFun, CasError> {
        self.depth += 1;
        if self.depth > MAX_CAS_DEPTH {
            self.depth -= 1;
            return Err(CasError::too_deep());
        }
        let out = self.lower_inner(e);
        self.depth -= 1;
        out
    }

    fn lower_inner(&mut self, e: &Expr) -> Result<RatFun, CasError> {
        if let Some(err) = unsupported(e) {
            return Err(err);
        }
        match e {
            Expr::Num {
                value,
                is_imaginary,
                ..
            } => {
                if *is_imaginary {
                    // Symja carries `I` natively; the rational core is over ℚ,
                    // not ℚ(i). Refuse by name rather than silently drop the
                    // imaginary unit.
                    return Err(CasError::unsupported(
                        "imaginary literals are not supported in CAS expressions",
                    ));
                }
                let r = rational_from_f64(*value).ok_or_else(|| {
                    CasError::unsupported("number is not representable as an exact rational")
                })?;
                Ok(RatFun::from_poly(MPoly::constant(r)))
            }
            Expr::Var(name) => {
                let g = self.gens.intern(Expr::Var(name.clone()));
                Ok(RatFun::from_poly(MPoly::var(&g)))
            }
            Expr::Neg(inner) => Ok(self.lower(inner)?.neg()),
            Expr::BinOp { op, left, right } => self.lower_binop(*op, left, right),
            Expr::Call { function, args } => self.lower_call(function, args),
            // Every remaining variant is handled by `unsupported` above.
            other => Err(unsupported(other).unwrap_or_else(|| {
                CasError::unsupported("expression is not supported in CAS expressions")
            })),
        }
    }

    fn lower_binop(&mut self, op: BinOp, left: &Expr, right: &Expr) -> Result<RatFun, CasError> {
        match op {
            BinOp::Add => {
                let (l, r) = (self.lower(left)?, self.lower(right)?);
                checked(l.add(&r))
            }
            BinOp::Sub => {
                let (l, r) = (self.lower(left)?, self.lower(right)?);
                checked(l.sub(&r))
            }
            BinOp::Mul => {
                let (l, r) = (self.lower(left)?, self.lower(right)?);
                checked(l.mul(&r))
            }
            BinOp::Div => {
                let (l, r) = (self.lower(left)?, self.lower(right)?);
                let q = l.div(&r).ok_or_else(CasError::division_by_zero)?;
                checked(q)
            }
            BinOp::Pow => self.lower_pow(left, right),
            // `ExprToSymja` writes the operator char straight into the Symja
            // string, so `\` and the four element-wise operators reach Symja as
            // syntax it cannot parse and come back as `CAS syntax error
            // evaluating '...'`. Refusing by name is the same outcome with a
            // message that says what is actually wrong.
            BinOp::LeftDiv
            | BinOp::ElemMul
            | BinOp::ElemDiv
            | BinOp::ElemLeftDiv
            | BinOp::ElemPow => Err(CasError::unsupported(format!(
                "the `{}` operator is not supported in CAS expressions",
                op.as_str()
            ))),
        }
    }

    fn lower_pow(&mut self, base: &Expr, exponent: &Expr) -> Result<RatFun, CasError> {
        if let Some(n) = integer_literal(exponent) {
            if n.abs() <= MAX_POW {
                let b = self.lower(base)?;
                let p = b
                    .pow(n as i32)
                    .ok_or_else(|| CasError::unsupported("0 raised to a negative power"))?;
                return checked(p);
            }
            // Ledger 25: the exponent ceiling is about to make this power one
            // opaque generator — the answer will be *unfactored*, not proved
            // irreducible, and the user must be able to tell which.
            record_ceiling_note(format!(
                "an exponent of {n} exceeds the ^{MAX_POW} expansion ceiling, so the power \
                 was kept opaque — the result is returned unexpanded, not proved \
                 irreducible"
            ));
        }
        // A symbolic or oversized exponent makes the whole power one opaque
        // generator, with both halves canonicalised first so `cos(2*x+1)` and
        // `cos(1+2*x)` intern to the same generator.
        let atom = Expr::bin(
            BinOp::Pow,
            self.canon_atom(base)?,
            self.canon_atom(exponent)?,
        );
        let g = self.gens.intern(atom);
        Ok(RatFun::from_poly(MPoly::var(&g)))
    }

    fn lower_call(&mut self, function: &str, args: &[Expr]) -> Result<RatFun, CasError> {
        if symja_function_name(function).is_none() {
            return Err(CasError::unsupported(format!(
                "function not supported in CAS expressions: {function}"
            )));
        }
        let mut canon_args = Vec::with_capacity(args.len());
        for a in args {
            canon_args.push(self.canon_atom(a)?);
        }
        let g = self.gens.intern(Expr::call(function, canon_args));
        Ok(RatFun::from_poly(MPoly::var(&g)))
    }

    /// Canonicalise a sub-expression that will live inside a generator.
    /// Mirrors Symja, which prints `Integrate[Cos(2*x+1),x]` as `Sin(1+2*x)/2`.
    fn canon_atom(&mut self, e: &Expr) -> Result<Expr, CasError> {
        let rf = self.lower(e)?;
        Ok(ratfun_to_expr(&rf, self.gens, DenForm::Factored))
    }
}

fn checked(rf: RatFun) -> Result<RatFun, CasError> {
    if rf.numer().term_count() > MAX_TERMS || rf.denom().term_count() > MAX_TERMS {
        return Err(CasError::too_large());
    }
    Ok(rf)
}

/// The integer value of a literal exponent, if it is one.
///
/// `x^(-2)` parses as `Pow(x, Neg(Num(2)))` — the negation is a node, not part
/// of the literal — so the unary minus has to be seen through here or every
/// negative power would become an opaque generator.
fn integer_literal(e: &Expr) -> Option<i64> {
    match e {
        Expr::Neg(inner) => integer_literal(inner)?.checked_neg(),
        Expr::Num {
            value,
            is_imaginary: false,
            ..
        } => {
            let r = rational_from_f64(*value)?;
            if r.denom().is_one() {
                r.numer().to_i64()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Lower `e` into the rational IR, interning generators into `gens`.
pub fn lower(e: &Expr, gens: &mut Gens) -> Result<RatFun, CasError> {
    Lower { gens, depth: 0 }.lower(e)
}

// ── monomial ordering ───────────────────────────────────────────────────────

/// Symja's `Plus` order: lexicographic **ascending** on the exponent vector
/// read from the last variable to the first, variables sorted ascending.
///
/// Reproduces `1+3*x+3*x^2+x^3`, `x^2+2*x*y+y^2`, `a*c+b*c+a*d+b*d` and
/// `(b*c+a*d)/(b*d)` exactly. The constant term is the all-zero vector, so it
/// sorts first without a special case.
fn cmp_mono(a: &Mono, b: &Mono, vars: &[String]) -> Ordering {
    for v in vars.iter().rev() {
        match exponent_of(a, v).cmp(&exponent_of(b, v)) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

/// `p`'s terms in Symja's print order.
fn ordered_terms(p: &MPoly) -> Vec<(Rat, Mono)> {
    let vars = poly_vars(p);
    let mut terms = mpoly_terms(p);
    terms.sort_by(|(_, a), (_, b)| cmp_mono(a, b, &vars));
    terms
}

/// Factor order inside a product: descending-monomial term sequences compared
/// lexicographically on `(monomial, coefficient)`, shorter first.
///
/// Gives `(-1+x)*(1+x)`, `(-3+x)*(-2+x)*(-1+x)`, `(1+2*x)*(1+3*x)`,
/// `(1+s)*(2+s)`, `(x-y)*(x+y)` and `x*y*(x+y)` — every probed `Factor` result.
fn cmp_factor(a: &MPoly, b: &MPoly) -> Ordering {
    let mut vars = poly_vars(a);
    for v in poly_vars(b) {
        if !vars.contains(&v) {
            vars.push(v);
        }
    }
    vars.sort();
    let key = |p: &MPoly| {
        let mut t = mpoly_terms(p);
        t.sort_by(|(_, x), (_, y)| cmp_mono(y, x, &vars));
        t
    };
    let (ka, kb) = (key(a), key(b));
    for (ta, tb) in ka.iter().zip(kb.iter()) {
        match cmp_mono(&ta.1, &tb.1, &vars) {
            Ordering::Equal => {}
            other => return other,
        }
        match ta.0.cmp(&tb.0) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    ka.len().cmp(&kb.len())
}

// ── factoring an MPoly ──────────────────────────────────────────────────────

/// `content · Π factorᵢ^multᵢ`, the shape [`factorization_to_expr`] prints.
#[derive(Debug, Clone)]
struct Factored {
    content: Rat,
    factors: Vec<(MPoly, u32)>,
}

/// Factor over ℚ as far as `cas::poly` allows.
///
/// The rational content and the monomial gcd always come out; the remainder is
/// factored properly when it is univariate (`poly` provides Yun + Cantor–
/// Zassenhaus + Hensel there) and left alone otherwise. See divergence 6.
fn factor_mpoly(p: &MPoly) -> Factored {
    if p.is_zero() {
        return Factored {
            content: Rat::zero(),
            factors: Vec::new(),
        };
    }
    if let Some(c) = p.as_constant() {
        return Factored {
            content: c,
            factors: Vec::new(),
        };
    }

    // Split off the monomial gcd: `x^2*y + x*y^2` = `x*y*(x+y)`.
    let terms = mpoly_terms(p);
    let mut shared: Mono = terms[0].1.clone();
    for (_, mono) in &terms[1..] {
        shared.retain(|(v, _)| exponent_of(mono, v) > 0);
        for (v, e) in shared.iter_mut() {
            *e = (*e).min(exponent_of(mono, v));
        }
    }
    shared.retain(|(_, e)| *e > 0);
    let mut factors: Vec<(MPoly, u32)> = shared.iter().map(|(v, e)| (MPoly::var(v), *e)).collect();
    let reduced: Vec<(Rat, Mono)> = terms
        .iter()
        .map(|(c, mono)| {
            let mut m = mono.clone();
            for (v, e) in &shared {
                for slot in m.iter_mut() {
                    if slot.0 == *v {
                        slot.1 -= e;
                    }
                }
            }
            m.retain(|(_, e)| *e > 0);
            (c.clone(), m)
        })
        .collect();
    let rest = mpoly_from_terms(&reduced);

    let vars = poly_vars(&rest);
    if vars.is_empty() {
        return Factored {
            content: rest.as_constant().unwrap_or_else(Rat::one),
            factors,
        };
    }
    if vars.len() == 1 {
        if let Some(up) = rest.to_upoly(&vars[0]) {
            if let Ok(f) = up.factor() {
                for (fp, mult) in f.factors {
                    factors.push((MPoly::from_upoly(&fp, &vars[0]), mult as u32));
                }
                return Factored {
                    content: f.unit,
                    factors,
                };
            }
        }
    }
    // Multivariate, or the univariate factoriser declined (its recombination
    // budget). Keep the content out front and leave the rest expanded.
    let content = signed_content(&rest);
    let primitive = rest.scale(&content.recip());
    factors.push((primitive, 1));
    Factored { content, factors }
}

/// The rational content of `p`, signed by its leading coefficient in this
/// module's monomial order, so every factor ends up with a positive leading
/// coefficient and the sign lives in the content.
fn signed_content(p: &MPoly) -> Rat {
    let content = p.content();
    if content.is_zero() {
        return Rat::one();
    }
    let leading_negative = ordered_terms(p)
        .last()
        .map(|(c, _)| c.is_negative())
        .unwrap_or(false);
    if leading_negative {
        -content
    } else {
        content
    }
}

// ── lifting: IR -> Expr, in Symja's spelling ────────────────────────────────

fn big_expr(n: &BigInt) -> Expr {
    // `Expr::Num` is an `f64`; integers past 2^53 therefore print approximately.
    // The term-count and exponent caps keep real inputs well inside that.
    Expr::num(n.to_f64().unwrap_or(f64::NAN))
}

fn int_expr(n: i64) -> Expr {
    Expr::num(n as f64)
}

/// A rational as an expression: `3`, `-3`, `3/4`, `-3/4`.
fn rat_expr(r: &Rat) -> Expr {
    if r.denom().is_one() {
        big_expr(r.numer())
    } else {
        Expr::bin(BinOp::Div, big_expr(r.numer()), big_expr(r.denom()))
    }
}

/// A monomial as a product of generators, ascending by name. `None` for the
/// empty monomial.
fn mono_expr(m: &Mono, gens: &Gens) -> Option<Expr> {
    let mut acc: Option<Expr> = None;
    for (v, e) in m {
        let base = gens.expr(v);
        let factor = if *e == 1 {
            base
        } else {
            Expr::bin(BinOp::Pow, base, int_expr(i64::from(*e)))
        };
        acc = Some(match acc {
            None => factor,
            Some(prev) => Expr::bin(BinOp::Mul, prev, factor),
        });
    }
    acc
}

/// `coefficient * monomial` in Symja's spelling.
///
/// `q == 1` → `p*m`; `|p| == 1` → `m/q`; otherwise `p/q*m`. Read off
/// `Expand[(x+1)^2/3]` = `1/3+2/3*x+x^2/3` and confirmed against `x^3/3`,
/// `x/2`, `5/6*x`, `2/3*x^(3/2)`.
fn scaled(magnitude: Expr, coeff: &Rat) -> Expr {
    if coeff.is_one() {
        return magnitude;
    }
    let unit_numerator = coeff.numer().magnitude().is_one();
    if coeff.denom().is_one() {
        if coeff.numer().is_negative() && unit_numerator {
            return Expr::Neg(Box::new(magnitude));
        }
        return Expr::bin(BinOp::Mul, big_expr(coeff.numer()), magnitude);
    }
    if unit_numerator {
        let quotient = Expr::bin(BinOp::Div, magnitude, big_expr(coeff.denom()));
        return if coeff.numer().is_negative() {
            Expr::Neg(Box::new(quotient))
        } else {
            quotient
        };
    }
    Expr::bin(BinOp::Mul, rat_expr(coeff), magnitude)
}

/// One term of a sum as `(is_negative, magnitude)`.
fn term_expr(coeff: &Rat, mono: &Mono, gens: &Gens) -> (bool, Expr) {
    let negative = coeff.is_negative();
    let magnitude = coeff.abs();
    match mono_expr(mono, gens) {
        None => (negative, rat_expr(&magnitude)),
        Some(m) => (negative, scaled(m, &magnitude)),
    }
}

/// Fold signed magnitudes into a left-associated `+`/`-` chain, which is how
/// Symja prints a `Plus` (`-1+x`, `1-2*x^2+x^4`, `2/(1+s)-1/(2+s)`).
fn sum_expr(parts: Vec<(bool, Expr)>) -> Expr {
    let mut iter = parts.into_iter();
    let Some((neg, first)) = iter.next() else {
        return Expr::num(0.0);
    };
    let mut acc = if neg { negate(first) } else { first };
    for (neg, part) in iter {
        acc = Expr::bin(if neg { BinOp::Sub } else { BinOp::Add }, acc, part);
    }
    acc
}

/// `-e`, collapsing into a literal so `-1+x` prints as Symja does.
fn negate(e: Expr) -> Expr {
    match e {
        Expr::Num {
            value,
            unit,
            is_imaginary,
        } => Expr::Num {
            value: -value,
            unit,
            is_imaginary,
        },
        other => Expr::Neg(Box::new(other)),
    }
}

/// A polynomial, expanded, in Symja's term order.
fn poly_to_expr(p: &MPoly, gens: &Gens) -> Expr {
    let terms = ordered_terms(p);
    if terms.is_empty() {
        return Expr::num(0.0);
    }
    sum_expr(
        terms
            .iter()
            .map(|(c, m)| term_expr(c, m, gens))
            .collect::<Vec<_>>(),
    )
}

/// A factorisation as `content * f1^e1 * f2^e2 …`.
///
/// A single factor of multiplicity one whose content is ±1 is printed expanded
/// instead — that is what `Factor(x^2+1)` = `1+x^2`, `Factor(x^3-2)` = `-2+x^3`
/// and `Factor(-x-1)` = `-1-x` do.
fn factorization_to_expr(f: &Factored, gens: &Gens) -> Expr {
    let mut factors: Vec<(MPoly, u32)> = f
        .factors
        .iter()
        .filter(|(p, e)| *e > 0 && !p.is_zero())
        .cloned()
        .collect();
    if factors.is_empty() {
        return rat_expr(&f.content);
    }
    let unit_content = f.content.abs().is_one();
    if unit_content && factors.len() == 1 && factors[0].1 == 1 {
        return poly_to_expr(&factors[0].0.scale(&f.content), gens);
    }
    factors.sort_by(|(a, _), (b, _)| cmp_factor(a, b));

    let mut acc: Option<Expr> = None;
    for (p, e) in &factors {
        let base = poly_to_expr(p, gens);
        let piece = if *e == 1 {
            base
        } else {
            Expr::bin(BinOp::Pow, base, int_expr(i64::from(*e)))
        };
        acc = Some(match acc {
            None => piece,
            Some(prev) => Expr::bin(BinOp::Mul, prev, piece),
        });
    }
    let product = acc.unwrap_or_else(|| Expr::num(1.0));
    if f.content.is_one() {
        product
    } else if unit_content {
        negate(product)
    } else {
        Expr::bin(BinOp::Mul, rat_expr(&f.content), product)
    }
}

/// How a denominator should be spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DenForm {
    /// `(1+s)*(2+s)` — what `Together` and `Apart` show.
    Factored,
    /// `2+3*s+s^2` — what `Expand` and `Cancel` show.
    Expanded,
}

/// A rational function as `num/den`, or as a bare polynomial when `den` is 1.
///
/// A polynomial with fractional coefficients is printed with its content in
/// front (`Together[x/2+1/3]` = `1/6*(2+3*x)`), which is the same convention
/// `Factor` uses.
fn ratfun_to_expr(rf: &RatFun, gens: &Gens, den_form: DenForm) -> Expr {
    let den = rf.denom();
    if let Some(c) = constant_value(den) {
        if !c.is_zero() {
            // `RatFun` keeps `den` primitive, so this is `den == 1`; the value
            // is a polynomial over ℚ.
            let p = rf.numer().scale(&c.recip());
            return polynomial_with_content(&p, gens);
        }
    }
    let num = poly_to_expr(rf.numer(), gens);
    let den_expr = match den_form {
        DenForm::Factored => factorization_to_expr(&factor_mpoly(den), gens),
        DenForm::Expanded => poly_to_expr(den, gens),
    };
    Expr::bin(BinOp::Div, num, den_expr)
}

/// A polynomial printed with its rational content pulled in front when that
/// content is not 1 — `1/6*(2+3*x)`, `1/2*(2+x)`, `2/3*(2+x)`.
fn polynomial_with_content(p: &MPoly, gens: &Gens) -> Expr {
    let terms = ordered_terms(p);
    if terms.is_empty() {
        return Expr::num(0.0);
    }
    let content = p.content();
    if content.is_one() || terms.len() == 1 {
        return poly_to_expr(p, gens);
    }
    let primitive = p.scale(&content.recip());
    Expr::bin(
        BinOp::Mul,
        rat_expr(&content),
        poly_to_expr(&primitive, gens),
    )
}

// ── the printer ─────────────────────────────────────────────────────────────

fn precedence(e: &Expr) -> u8 {
    match e {
        Expr::Num { value, .. } if *value < 0.0 => 1,
        Expr::Num { .. } | Expr::Var(_) | Expr::Call { .. } | Expr::Str(_) => 4,
        Expr::Neg(_) => 1,
        Expr::BinOp { op, .. } => match op {
            BinOp::Add | BinOp::Sub => 1,
            BinOp::Pow => 3,
            _ => 2,
        },
        _ => 0,
    }
}

/// A number the way `ExprToSymja.number` writes one: integral values without a
/// decimal point, everything else as its shortest round-tripping decimal.
fn number_text(v: f64) -> String {
    if v.is_finite() && v.abs() < 1e15 {
        let rounded = v.round();
        // Relative tolerance rather than `v == rounded`: this module only ever
        // builds integral literals, and a bare float equality is both a
        // SonarCloud finding and the wrong predicate once the value has been
        // through a `BigInt::to_f64`.
        if (v - rounded).abs() < f64::EPSILON * v.abs().max(1.0) {
            return format!("{}", rounded as i64);
        }
    }
    format!("{v}")
}

/// The plain-text form the REPL prints — Symja's own spelling, with
/// `Log`/`Log10` already rewritten to `ln`/`log10` as `casDisplay` does.
pub fn display(e: &Expr) -> String {
    let mut out = String::new();
    write_expr(e, 0, &mut out);
    out
}

fn write_expr(e: &Expr, min_prec: u8, out: &mut String) {
    let prec = precedence(e);
    let parens = prec < min_prec;
    if parens {
        out.push('(');
    }
    match e {
        Expr::Num { value, .. } => out.push_str(&number_text(*value)),
        Expr::Var(name) => out.push_str(name),
        Expr::Str(s) => {
            out.push('\'');
            out.push_str(s);
            out.push('\'');
        }
        Expr::Neg(inner) => {
            out.push('-');
            write_expr(inner, 2, out);
        }
        Expr::Call { function, args } => write_call(function, args, out),
        Expr::BinOp { op, left, right } => match op {
            BinOp::Add => {
                write_expr(left, 1, out);
                out.push('+');
                write_expr(right, 1, out);
            }
            BinOp::Sub => {
                write_expr(left, 1, out);
                out.push('-');
                write_expr(right, 2, out);
            }
            BinOp::Mul => {
                write_expr(left, 2, out);
                out.push('*');
                write_expr(right, 2, out);
            }
            BinOp::Div => {
                write_expr(left, 2, out);
                out.push('/');
                write_expr(right, 3, out);
            }
            BinOp::Pow => {
                write_expr(left, 4, out);
                out.push('^');
                write_expr(right, 4, out);
            }
            other => {
                write_expr(left, 2, out);
                out.push_str(other.as_str());
                write_expr(right, 2, out);
            }
        },
        // Never produced by this module; printed rather than panicking so the
        // printer stays total over `Expr`.
        other => out.push_str(&format!("{other:?}")),
    }
    if parens {
        out.push(')');
    }
}

fn write_call(function: &str, args: &[Expr], out: &mut String) {
    // Symja prints `Exp(u)` as `E^u`, which is what the REPL shows
    // (`Integrate[Exp(2*x),x]` → `E^(2*x)/2`).
    if function == "exp" && args.len() == 1 {
        out.push_str("E^");
        write_expr(&args[0], 4, out);
        return;
    }
    out.push_str(symja_function_name(function).unwrap_or(function));
    out.push('(');
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_expr(a, 0, out);
    }
    out.push(')');
}

/// The Symja *input* string the Java would have built — a faithful port of
/// `ExprToSymja.convert`, kept because `CasResult.symjaInput` is part of the
/// result record and because it validates the same node set with the same
/// messages.
pub fn to_symja_input(e: &Expr) -> Result<String, CasError> {
    let mut out = String::new();
    write_symja_input(e, 0, &mut out)?;
    Ok(out)
}

fn write_symja_input(e: &Expr, depth: u32, out: &mut String) -> Result<(), CasError> {
    if depth > MAX_CAS_DEPTH {
        return Err(CasError::too_deep());
    }
    if let Some(err) = unsupported(e) {
        return Err(err);
    }
    match e {
        Expr::Num {
            value,
            is_imaginary,
            ..
        } => {
            if *is_imaginary {
                out.push('(');
                out.push_str(&number_text(*value));
                out.push_str("*I)");
            } else {
                out.push_str(&number_text(*value));
            }
        }
        Expr::Var(name) => out.push_str(name),
        Expr::Neg(inner) => {
            out.push_str("(-(");
            write_symja_input(inner, depth + 1, out)?;
            out.push_str("))");
        }
        Expr::BinOp { op, left, right } => {
            out.push('(');
            write_symja_input(left, depth + 1, out)?;
            out.push_str(op.as_str());
            write_symja_input(right, depth + 1, out)?;
            out.push(')');
        }
        Expr::Call { function, args } => {
            let name = symja_function_name(function).ok_or_else(|| {
                CasError::unsupported(format!(
                    "function not supported in CAS expressions: {function}"
                ))
            })?;
            // `ln`/`log10` are the *display* spellings; the input string keeps
            // Symja's own names.
            let name = match function.as_str() {
                "ln" => "Log",
                "log10" => "Log10",
                _ => name,
            };
            out.push_str(name);
            out.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_symja_input(a, depth + 1, out)?;
            }
            out.push(')');
        }
        other => {
            return Err(unsupported(other).unwrap_or_else(|| {
                CasError::unsupported("expression is not supported in CAS expressions")
            }))
        }
    }
    Ok(())
}

// ── the nine rational-core operations ───────────────────────────────────────

/// `Factor` — factor numerator and denominator over ℚ and cancel.
pub fn factor(e: &Expr) -> Result<Expr, CasError> {
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;
    let num = factorization_to_expr(&factor_mpoly(rf.numer()), &gens);
    if let Some(c) = constant_value(rf.denom()) {
        if c.is_one() {
            return Ok(num);
        }
    }
    let den = factorization_to_expr(&factor_mpoly(rf.denom()), &gens);
    Ok(Expr::bin(BinOp::Div, num, den))
}

/// `Expand` — multiply out. A rational input keeps one fraction (divergence 4).
pub fn expand(e: &Expr) -> Result<Expr, CasError> {
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;
    if let Some(c) = constant_value(rf.denom()) {
        if !c.is_zero() {
            return Ok(poly_to_expr(&rf.numer().scale(&c.recip()), &gens));
        }
    }
    Ok(ratfun_to_expr(&rf, &gens, DenForm::Expanded))
}

/// `Together` — one fraction, numerator expanded, denominator factored.
pub fn together(e: &Expr) -> Result<Expr, CasError> {
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;
    Ok(ratfun_to_expr(&rf, &gens, DenForm::Factored))
}

/// `Cancel` — lowest terms, both halves expanded.
pub fn cancel(e: &Expr) -> Result<Expr, CasError> {
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;
    Ok(ratfun_to_expr(&rf, &gens, DenForm::Expanded))
}

/// `Numerator` — **syntactic**, exactly as Symja's is.
///
/// `Numerator[1/(s+1)+1/(s+2)]` is the whole sum, not `3+2*s`; and
/// `Numerator[(x^2-1)/((x-1)*(x+3))]` is `-1+x^2` with nothing cancelled. Only
/// the polynomial leaves are re-ordered into Symja's canonical spelling.
pub fn numerator(e: &Expr) -> Result<Expr, CasError> {
    let (num, _) = split_fraction(e, 0)?;
    canonical_product(&num)
}

/// `Denominator` — the syntactic partner of [`numerator`].
pub fn denominator(e: &Expr) -> Result<Expr, CasError> {
    let (_, den) = split_fraction(e, 0)?;
    canonical_product(&den)
}

/// Split an expression into syntactic numerator and denominator factor lists.
fn split_fraction(e: &Expr, depth: u32) -> Result<(Vec<Expr>, Vec<Expr>), CasError> {
    if depth > MAX_CAS_DEPTH {
        return Err(CasError::too_deep());
    }
    if let Some(err) = unsupported(e) {
        return Err(err);
    }
    match e {
        Expr::BinOp {
            op: BinOp::Div,
            left,
            right,
        } => {
            let (ln, ld) = split_fraction(left, depth + 1)?;
            let (rn, rd) = split_fraction(right, depth + 1)?;
            Ok(([ln, rd].concat(), [ld, rn].concat()))
        }
        Expr::BinOp {
            op: BinOp::Mul,
            left,
            right,
        } => {
            let (ln, ld) = split_fraction(left, depth + 1)?;
            let (rn, rd) = split_fraction(right, depth + 1)?;
            Ok(([ln, rn].concat(), [ld, rd].concat()))
        }
        Expr::BinOp {
            op: BinOp::Pow,
            left,
            right,
        } => match integer_literal(right) {
            Some(n) if n < 0 && n.abs() <= MAX_POW => Ok((
                Vec::new(),
                vec![Expr::bin(BinOp::Pow, (**left).clone(), int_expr(-n))],
            )),
            _ => Ok((vec![e.clone()], Vec::new())),
        },
        _ => Ok((vec![e.clone()], Vec::new())),
    }
}

/// Print a syntactic factor list, canonicalising each factor but keeping the
/// product structure — `(-1+x)*(3+x)`, `2*(-1+x)`, `(1+x)^2`.
fn canonical_product(factors: &[Expr]) -> Result<Expr, CasError> {
    let mut acc: Option<Expr> = None;
    for f in factors {
        let piece = canonical_factor(f)?;
        acc = Some(match acc {
            None => piece,
            Some(prev) => Expr::bin(BinOp::Mul, prev, piece),
        });
    }
    Ok(acc.unwrap_or_else(|| Expr::num(1.0)))
}

fn canonical_factor(e: &Expr) -> Result<Expr, CasError> {
    if let Expr::BinOp {
        op: BinOp::Pow,
        left,
        right,
    } = e
    {
        if let Some(n) = integer_literal(right) {
            if n > 1 && n <= MAX_POW {
                return Ok(Expr::bin(BinOp::Pow, canonical_factor(left)?, int_expr(n)));
            }
        }
    }
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;
    Ok(ratfun_to_expr(&rf, &gens, DenForm::Expanded))
}

/// Exact division that declines rather than panicking on a zero divisor.
fn rat_div(a: &Rat, b: &Rat) -> Option<Rat> {
    if b.is_zero() {
        None
    } else {
        Some(a / b)
    }
}

// ── Apart ───────────────────────────────────────────────────────────────────

/// One partial-fraction decomposition: a polynomial part plus
/// `numer / base^power` terms.
struct Decomposition {
    polynomial: MPoly,
    terms: Vec<(MPoly, MPoly, u32)>,
}

/// `Apart` — partial fractions with respect to `var`.
///
/// Implemented here rather than in `ratfun` because everything it needs
/// (`factor`, `div_rem`, exact division) is already on `UPoly`, and because
/// partial fractions are univariate by definition. When the expression is not
/// a rational function *of `var` alone* Symja returns it unchanged, and so does
/// this: `Apart[1/(a*s+b), s]` = `1/(b+a*s)`, `Apart[(s+3)/(s^2+3*s+2), x]` =
/// `(3+s)/((1+s)*(2+s))`.
pub fn apart(e: &Expr, var: &str) -> Result<Expr, CasError> {
    let var = var.to_ascii_lowercase();
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;
    let Some(decomposition) = partial_fractions(&rf, &var) else {
        return Ok(ratfun_to_expr(&rf, &gens, DenForm::Factored));
    };

    let mut parts: Vec<(bool, Expr)> = Vec::new();
    if !decomposition.polynomial.is_zero() {
        for (c, m) in ordered_terms(&decomposition.polynomial) {
            parts.push(term_expr(&c, &m, &gens));
        }
    }
    let mut terms = decomposition.terms;
    // Factors in the printer's order; within one factor, descending power —
    // `1/s^2-1/s+1/(1+s)` and `-1/(2+s)^2+2/(2+s)`.
    terms.sort_by(|a, b| cmp_factor(&a.1, &b.1).then_with(|| b.2.cmp(&a.2)));
    for (numer, base, power) in &terms {
        parts.push(apart_term_expr(numer, base, *power, &gens));
    }
    if parts.is_empty() {
        return Ok(Expr::num(0.0));
    }
    Ok(sum_expr(parts))
}

/// Collapse to one generator and hand the decomposition to
/// [`crate::cas::ratfun::RatFun::partial_fractions`], which factors the
/// denominator over ℚ and splits by CRT. `None` when the expression is not a
/// rational function of `var` alone, which is the case Symja returns unchanged.
fn partial_fractions(rf: &RatFun, var: &str) -> Option<Decomposition> {
    let degree = rf.denom().to_upoly(var)?.degree_or_zero();
    if degree > MAX_APART_DEGREE {
        // Ledger 25: over the ceiling Apart returns its input unchanged —
        // say so instead of letting "declined" read as "already simplest".
        record_ceiling_note(format!(
            "the denominator degree {degree} exceeds Apart's {MAX_APART_DEGREE} \
             ceiling, so the expression was returned undecomposed"
        ));
        return None;
    }
    if degree == 0 {
        return None;
    }
    let decomposition = rf.partial_fractions(var).ok()?;
    Some(Decomposition {
        polynomial: MPoly::from_upoly(&decomposition.polynomial, var),
        terms: decomposition
            .terms
            .iter()
            .map(|t| {
                (
                    MPoly::from_upoly(&t.numerator, var),
                    MPoly::from_upoly(&t.base, var),
                    t.power as u32,
                )
            })
            .collect(),
    })
}

/// One partial-fraction term, `N/(q*B^k)`.
///
/// The coefficients' common denominator `q` is folded into the printed
/// denominator rather than left in front — `1/(2*(1+s))`, which is what the
/// engine returns for fourteen of the fifteen probed decompositions.
fn apart_term_expr(numer: &MPoly, base: &MPoly, power: u32, gens: &Gens) -> (bool, Expr) {
    let content = numer.content();
    let scale = content.denom().clone();
    let integral = numer.scale(&Rat::from_integer(scale.clone()));
    let terms = ordered_terms(&integral);
    let negative = terms.last().map(|(c, _)| c.is_negative()).unwrap_or(false);
    let signed = if negative {
        integral.scale(&-Rat::one())
    } else {
        integral
    };
    let num_expr = poly_to_expr(&signed, gens);

    let base_expr = poly_to_expr(base, gens);
    let powered = if power == 1 {
        base_expr
    } else {
        Expr::bin(BinOp::Pow, base_expr, int_expr(i64::from(power)))
    };
    let den_expr = if scale.is_one() {
        powered
    } else {
        Expr::bin(BinOp::Mul, big_expr(&scale), powered)
    };
    (negative, Expr::bin(BinOp::Div, num_expr, den_expr))
}

// ── Collect ─────────────────────────────────────────────────────────────────

/// `Collect` — gather powers of `var`.
///
/// `Collect[x*a+x*b+c, x]` = `c+(a+b)*x`; `Collect[x*y+x*z, x]` = `x*(y+z)`.
/// The order inside a term follows Symja: the collected power leads when its
/// name sorts before the coefficient's first generator, otherwise the
/// coefficient leads.
pub fn collect(e: &Expr, var: &str) -> Result<Expr, CasError> {
    let var = var.to_ascii_lowercase();
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;
    let Some(c) = constant_value(rf.denom()) else {
        // Symja leaves a genuine fraction alone.
        return Ok(ratfun_to_expr(&rf, &gens, DenForm::Factored));
    };
    if c.is_zero() {
        return Err(CasError::division_by_zero());
    }
    let p = rf.numer().scale(&c.recip());

    // power of `var` -> the coefficient's terms
    let mut buckets: BTreeMap<u32, Vec<(Rat, Mono)>> = BTreeMap::new();
    for (coeff, mut mono) in mpoly_terms(&p) {
        let k = exponent_of(&mono, &var);
        mono.retain(|(v, _)| *v != var);
        buckets.entry(k).or_default().push((coeff, mono));
    }

    let mut parts: Vec<(bool, Expr)> = Vec::new();
    for (power, terms) in &buckets {
        let coefficient = mpoly_from_terms(terms);
        if coefficient.is_zero() {
            continue;
        }
        if *power == 0 {
            for (c, m) in ordered_terms(&coefficient) {
                parts.push(term_expr(&c, &m, &gens));
            }
            continue;
        }
        let var_expr = if *power == 1 {
            gens.expr(&var)
        } else {
            Expr::bin(BinOp::Pow, gens.expr(&var), int_expr(i64::from(*power)))
        };
        parts.push(collected_term(&coefficient, var_expr, &var, &gens));
    }
    if parts.is_empty() {
        return Ok(Expr::num(0.0));
    }
    Ok(sum_expr(parts))
}

/// `coefficient * var^k`, with Symja's factor order and sign handling.
fn collected_term(coeff: &MPoly, var_expr: Expr, var: &str, gens: &Gens) -> (bool, Expr) {
    let terms = ordered_terms(coeff);
    if terms.len() == 1 {
        // A one-term coefficient merges into the monomial: `c*x`, `5*x`, `x^2`.
        let (c, m) = &terms[0];
        let negative = c.is_negative();
        let magnitude = c.abs();
        let base = match mono_expr(m, gens) {
            None => var_expr,
            Some(cm) => {
                if leads_with_variable(m, var) {
                    Expr::bin(BinOp::Mul, var_expr, cm)
                } else {
                    Expr::bin(BinOp::Mul, cm, var_expr)
                }
            }
        };
        return (negative, scaled(base, &magnitude));
    }
    // A multi-term coefficient stays a parenthesised sum.
    let sum = poly_to_expr(coeff, gens);
    let first_gen = terms
        .iter()
        .flat_map(|(_, m)| m.iter().map(|(v, _)| v.clone()))
        .min();
    let coefficient_first = first_gen.map(|g| g.as_str() < var).unwrap_or(true);
    let product = if coefficient_first {
        Expr::bin(BinOp::Mul, sum, var_expr)
    } else {
        Expr::bin(BinOp::Mul, var_expr, sum)
    };
    (false, product)
}

fn leads_with_variable(coeff_mono: &Mono, var: &str) -> bool {
    coeff_mono
        .iter()
        .map(|(v, _)| v.as_str())
        .min()
        .map(|first| var < first)
        .unwrap_or(false)
}

// ── Simplify ────────────────────────────────────────────────────────────────

/// `Simplify` — cancel, then take the shortest of a small candidate set.
///
/// Symja's `Simplify` is a heuristic search over a large rule set scored by
/// `LeafCount`; only its algebraic half is reachable from a rational core. The
/// candidates here are the cancelled single fraction, the factored form (for
/// polynomials only — Symja does not offer it for fractions, which is why
/// `Simplify[(s+3)/(s^2+3*s+2)]` stays `(3+s)/(2+3*s+s^2)`), and the
/// partial-fraction form for a single-variable fraction. A candidate must be
/// *strictly* shorter to displace the single fraction.
pub fn simplify(e: &Expr) -> Result<Expr, CasError> {
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;
    let baseline = ratfun_to_expr(&rf, &gens, DenForm::Expanded);
    let best_score = leaf_count(&baseline);
    let mut best = baseline;

    match constant_value(rf.denom()) {
        Some(c) if !c.is_zero() => {
            let p = rf.numer().scale(&c.recip());
            let factored = factorization_to_expr(&factor_mpoly(&p), &gens);
            if leaf_count(&factored) < best_score {
                best = factored;
            }
        }
        _ => {
            let mut vars = poly_vars(rf.numer());
            for x in poly_vars(rf.denom()) {
                if !vars.contains(&x) {
                    vars.push(x);
                }
            }
            if vars.len() == 1 {
                if let Ok(decomposed) = apart(e, &vars[0]) {
                    if leaf_count(&decomposed) < best_score {
                        best = decomposed;
                    }
                }
            }
        }
    }
    Ok(best)
}

/// An approximation of Symja's `LeafCount`, used only to rank [`simplify`]
/// candidates against each other.
fn leaf_count(e: &Expr) -> usize {
    match e {
        Expr::Num { .. } | Expr::Var(_) | Expr::Str(_) => 1,
        Expr::Neg(inner) => 2 + leaf_count(inner),
        Expr::Call { args, .. } => 1 + args.iter().map(leaf_count).sum::<usize>(),
        Expr::BinOp { op, left, right } => {
            let extra = match op {
                BinOp::Sub | BinOp::Div => 2,
                _ => 1,
            };
            extra + leaf_count(left) + leaf_count(right)
        }
        _ => 1,
    }
}

// ── D ───────────────────────────────────────────────────────────────────────

/// `D` — the already-ported [`crate::differentiator`], with the result put
/// through the same canonicaliser everything else uses so the spelling matches
/// (`D[x^3+2*x, x]` = `2+3*x^2`, `D[Sin(x)*Cos(x), x]` = `Cos(x)^2-Sin(x)^2`).
pub fn differentiate(e: &Expr, var: &str) -> Result<Expr, CasError> {
    let var = var.to_ascii_lowercase();
    let derivative = crate::differentiator::differentiate(e, &var)
        .ok_or_else(|| CasError::no_closed_form("D"))?;
    let mut gens = Gens::new();
    let rf = lower(&derivative, &mut gens)?;
    Ok(ratfun_to_expr(&rf, &gens, DenForm::Expanded))
}

// ── Integrate ───────────────────────────────────────────────────────────────

/// `Integrate` — a **closed** pattern table. Anything outside it is refused by
/// name; nothing is guessed.
///
/// | pattern | result |
/// |---|---|
/// | integrand free of `v` | `e*v` |
/// | `c*v^m`, `m` integer | `c*v^(m+1)/(m+1)`, or `c*ln(v)` at `m = -1` |
/// | `(a*v+b)^n`, `n` integer, `b ≠ 0`, `a` rational | `(a*v+b)^(n+1)/(a*(n+1))` |
/// | `N/(a*v+b)`, `N` and `a` constant | `N/a*ln(a*v+b)` |
/// | `N/D` with `D' = k*N` | `ln(D)/k` |
/// | `p*F(u)`, `F ∈ {exp,sin,cos}`, `p = k*u'` | `k * ∫F` at `u` |
/// | `exp/sin/cos` of `a*v+b` with symbolic `a` | `∫F(u)/a` |
/// | `Sin(v)^n*Cos(v)`, `Cos(v)^n*Sin(v)` | `Sin^(n+1)/(n+1)`, `-Cos^(n+1)/(n+1)` |
/// | polynomial in `v` over ℚ | term-by-term power rule |
///
/// Everything else — `exp(x^2)`, `sin(x)/x`, `tan(x)`, `ln(x)`, `sqrt(x)`,
/// `1/(x^2+1)`, integration by parts — returns [`CasError::NoClosedForm`].
/// Symja finds all of those; refusing is the deliberate scope decision recorded
/// in `PLAN.md` §5. The refusal reaches the REPL as
/// `"integrate: no closed form found for this input."`, the same sentence the
/// Java prints when Symja echoes an unevaluated `Integrate(...)` back.
pub fn integrate(e: &Expr, var: &str) -> Result<Expr, CasError> {
    let var = var.to_ascii_lowercase();
    let mut gens = Gens::new();
    let rf = lower(e, &mut gens)?;

    // 1. Free of the integration variable: `∫y dx = x*y`. Checked on the
    //    original expression, because a generator such as `Sin(x)` hides `x`
    //    from the polynomial's variable list.
    if !e.variables().contains(&var) {
        let v = RatFun::from_poly(MPoly::var(&gens.intern(Expr::Var(var.clone()))));
        return Ok(ratfun_to_expr(&rf.mul(&v), &gens, DenForm::Expanded));
    }

    // 2. A single power of the variable, positive or negative.
    if let Some(result) = integrate_monomial_power(&rf, &var, &gens) {
        return Ok(result);
    }

    // 3. `(a*v+b)^n`, kept unexpanded so the answer reads `(1+2*x)^4/8`.
    if let Some(result) = integrate_linear_power(e, &var)? {
        return Ok(result);
    }

    // Every generator other than `var` must be free of `var`, or treating the
    // integrand as a polynomial in `var` would be a silent lie (`∫Sin(x)dx` is
    // not `x*Sin(x)`).
    let opaque_dependency = poly_vars(rf.numer())
        .into_iter()
        .chain(poly_vars(rf.denom()))
        .any(|g| g != var && gens.depends_on(&g, &var));

    if opaque_dependency {
        // 4. exp/sin/cos, by substitution or with a linear argument.
        if let Some(result) = integrate_transcendental(&rf, &var, &gens)? {
            return Ok(result);
        }
        return Err(CasError::no_closed_form("Integrate"));
    }

    // 5-7. Log rules and the polynomial power rule.
    if let Some(result) = integrate_rational(&rf, &var, &gens) {
        return Ok(result);
    }
    Err(CasError::no_closed_form("Integrate"))
}

/// `coefficient * var^m` for an integer `m` — the power rule and, at `m = -1`,
/// the logarithm.
///
/// Handles `x^2`, `x^5`, `x/2`, `x^(-2)`, `1/x^2`, `1/x`, `3/x` and `1/(3*x)`
/// with one rule, which is why `∫dx/(3*x)` comes out `ln(x)/3` rather than
/// `ln(3*x)/3` — exactly as Symja spells it.
fn integrate_monomial_power(rf: &RatFun, var: &str, gens: &Gens) -> Option<Expr> {
    let (num_coeff, num_power) = single_monomial(rf.numer(), var)?;
    let (den_coeff, den_power) = single_monomial(rf.denom(), var)?;
    let coeff = rat_div(&num_coeff, &den_coeff)?;
    let m = i64::from(num_power) - i64::from(den_power);
    let v = gens.expr(var);
    if m == -1 {
        return Some(scaled_signed(Expr::call("ln", vec![v]), &coeff));
    }
    let scale = rat_div(&coeff, &rat_int(m + 1))?;
    Some(scaled_signed(power_expr(v, m + 1), &scale))
}

/// `base^n` as an expression, folding a negative exponent into a division so
/// the result reads `-1/x` rather than `-x^-1`.
fn power_expr(base: Expr, n: i64) -> Expr {
    match n {
        0 => Expr::num(1.0),
        1 => base,
        n if n > 1 => Expr::bin(BinOp::Pow, base, int_expr(n)),
        -1 => Expr::bin(BinOp::Div, Expr::num(1.0), base),
        n => Expr::bin(
            BinOp::Div,
            Expr::num(1.0),
            Expr::bin(BinOp::Pow, base, int_expr(-n)),
        ),
    }
}

/// `(coefficient, exponent of var)` when `p` is one monomial whose only
/// generator is `var` itself.
///
/// A monomial carrying any other symbol is refused here so the polynomial rule
/// picks it up instead — `∫a*x dx` is `a*x^2/2`, which the power rule below
/// would have to special-case.
fn single_monomial(p: &MPoly, var: &str) -> Option<(Rat, u32)> {
    let terms = mpoly_terms(p);
    if terms.len() != 1 {
        return None;
    }
    let (coeff, mono) = &terms[0];
    let mut power = 0;
    for (g, e) in mono {
        if g != var {
            return None;
        }
        power = *e;
    }
    Some((coeff.clone(), power))
}

/// `(a*v+b)^n` with `b ≠ 0` and a rational slope, left unexpanded.
fn integrate_linear_power(e: &Expr, var: &str) -> Result<Option<Expr>, CasError> {
    let Expr::BinOp {
        op: BinOp::Pow,
        left,
        right,
    } = e
    else {
        return Ok(None);
    };
    let Some(n) = integer_literal(right) else {
        return Ok(None);
    };
    if n.abs() < 2 || n.abs() >= MAX_POW || n == -1 {
        return Ok(None);
    }
    let mut gens = Gens::new();
    let rf = lower(left, &mut gens)?;
    let Some((a, b)) = linear_in(&rf, var, &gens) else {
        return Ok(None);
    };
    if b.is_zero() {
        // A bare `c*v` is the monomial rule's job, and expanding it there
        // matches Symja (`∫(2*x)^3 dx` = `2*x^4`, not `(2*x)^4/8`).
        return Ok(None);
    }
    let Some(slope) = constant_value(&a) else {
        return Ok(None);
    };
    let Some(scale) = rat_div(&Rat::one(), &(slope * rat_int(n + 1))) else {
        return Ok(None);
    };
    let inner = poly_to_expr(rf.numer(), &gens);
    Ok(Some(scaled_signed(power_expr(inner, n + 1), &scale)))
}

/// [`scaled`], but routing a negative coefficient through the same sign
/// handling the sum builder uses.
fn scaled_signed(magnitude: Expr, coeff: &Rat) -> Expr {
    let negative = coeff.is_negative();
    let e = scaled(magnitude, &coeff.abs());
    if negative {
        negate(e)
    } else {
        e
    }
}

/// `a*v + b` with `a` and `b` free of `v`, if that is the shape.
fn linear_in(rf: &RatFun, var: &str, gens: &Gens) -> Option<(MPoly, MPoly)> {
    constant_value(rf.denom())?;
    let mut a: Vec<(Rat, Mono)> = Vec::new();
    let mut b: Vec<(Rat, Mono)> = Vec::new();
    for (c, mut m) in mpoly_terms(rf.numer()) {
        if m.iter().any(|(g, _)| g != var && gens.depends_on(g, var)) {
            return None;
        }
        match exponent_of(&m, var) {
            0 => b.push((c, m)),
            1 => {
                m.retain(|(g, _)| g != var);
                a.push((c, m));
            }
            _ => return None,
        }
    }
    let a = mpoly_from_terms(&a);
    if a.is_zero() {
        return None;
    }
    Some((a, mpoly_from_terms(&b)))
}

/// The rational-function half of the table: the two logarithm rules and the
/// term-by-term power rule.
fn integrate_rational(rf: &RatFun, var: &str, gens: &Gens) -> Option<Expr> {
    let Some(denominator_constant) = constant_value(rf.denom()) else {
        // N/(a*v+b) with N and a constant -> a logarithm.
        let den_rf = RatFun::from_poly(rf.denom().clone());
        if let Some((a, _)) = linear_in(&den_rf, var, gens) {
            if let (Some(num), Some(slope)) = (constant_value(rf.numer()), constant_value(&a)) {
                if let Some(scale) = rat_div(&num, &slope) {
                    let log = Expr::call("ln", vec![poly_to_expr(rf.denom(), gens)]);
                    return Some(scaled_signed(log, &scale));
                }
            }
        }
        // N/D with D' = k*N -> ln(D)/k.
        let derivative = derivative_poly(rf.denom(), var);
        let k = proportionality(&derivative, rf.numer())?;
        let scale = rat_div(&Rat::one(), &k)?;
        let log = Expr::call("ln", vec![poly_to_expr(rf.denom(), gens)]);
        return Some(scaled_signed(log, &scale));
    };

    // A polynomial in `var` over ℚ — every other generator is a genuine
    // constant, which the caller has already checked.
    let inverse = rat_div(&Rat::one(), &denominator_constant)?;
    let p = rf.numer().scale(&inverse);
    let mut out: Vec<(Rat, Mono)> = Vec::new();
    for (coeff, mut m) in mpoly_terms(&p) {
        let k = exponent_of(&m, var);
        let scale = rat_div(&coeff, &rat_int(i64::from(k) + 1))?;
        m.retain(|(g, _)| g != var);
        m.push((var.to_string(), k + 1));
        m.sort();
        out.push((scale, m));
    }
    Some(poly_to_expr(&mpoly_from_terms(&out), gens))
}

/// `d p / d var`, valid because every generator other than `var` is a constant
/// on the paths that call this.
fn derivative_poly(p: &MPoly, var: &str) -> MPoly {
    let mut out: Vec<(Rat, Mono)> = Vec::new();
    for (c, mut m) in mpoly_terms(p) {
        let k = exponent_of(&m, var);
        if k == 0 {
            continue;
        }
        m.retain(|(g, _)| g != var);
        if k > 1 {
            m.push((var.to_string(), k - 1));
            m.sort();
        }
        out.push((c * rat_int(i64::from(k)), m));
    }
    mpoly_from_terms(&out)
}

/// `k` such that `a == k * b`, if such a rational exists.
fn proportionality(a: &MPoly, b: &MPoly) -> Option<Rat> {
    if a.is_zero() || b.is_zero() {
        return None;
    }
    let at = ordered_terms(a);
    let bt = ordered_terms(b);
    if at.len() != bt.len() {
        return None;
    }
    let k = rat_div(&at.last()?.0, &bt.last()?.0)?;
    for ((ca, ma), (cb, mb)) in at.iter().zip(bt.iter()) {
        if ma != mb || &(cb * &k) != ca {
            return None;
        }
    }
    Some(k)
}

/// The transcendental half of the table: `u'·F(u)` substitution, `F` of a
/// linear argument, and the `Sin^n·Cos` pair.
fn integrate_transcendental(rf: &RatFun, var: &str, gens: &Gens) -> Result<Option<Expr>, CasError> {
    if constant_value(rf.denom()).is_none() {
        return Ok(None);
    }
    // `Sin(v)^n*Cos(v)` and its partner: a substitution in the generator
    // itself, so it is tried before the single-generator split below (which
    // would refuse a term carrying two var-dependent generators).
    if let Some(result) = trig_power_pair(rf, var, gens) {
        return Ok(Some(result));
    }

    let Some((coeff, generator, power)) = single_generator_factor(rf.numer(), var, gens) else {
        return Ok(None);
    };
    if power != 1 {
        return Ok(None);
    }
    let call = gens.expr(&generator);
    let Expr::Call { function, args } = &call else {
        return Ok(None);
    };
    if args.len() != 1 || !matches!(function.as_str(), "exp" | "sin" | "cos") {
        return Ok(None);
    }
    let argument = &args[0];
    let (negated, antiderivative) = match function.as_str() {
        "exp" => (false, Expr::call("exp", vec![argument.clone()])),
        "sin" => (true, Expr::call("cos", vec![argument.clone()])),
        _ => (false, Expr::call("sin", vec![argument.clone()])),
    };

    let mut arg_gens = Gens::new();
    let arg_rf = lower(argument, &mut arg_gens)?;
    if constant_value(arg_rf.denom()).is_none() {
        return Ok(None);
    }
    let du = derivative_poly(arg_rf.numer(), var);
    if du.is_zero() {
        return Ok(None);
    }

    // The leftover coefficient must be a rational multiple of u'.
    if let Some(k) = proportionality(&coeff, &du) {
        let scale = if negated { -k } else { k };
        return Ok(Some(scaled_signed(antiderivative, &scale)));
    }

    // Otherwise accept a *linear* argument whose slope may itself be symbolic
    // (`∫exp(-a*x)dx = -E^(-a*x)/a`), provided the outer coefficient is free of
    // `var`.
    let Some((slope, _)) = linear_in(&arg_rf, var, &arg_gens) else {
        return Ok(None);
    };
    if !derivative_poly(&coeff, var).is_zero() {
        return Ok(None);
    }
    let mut merged = gens.clone();
    let numerator = Expr::bin(BinOp::Mul, poly_to_expr(&coeff, &merged), antiderivative);
    let quotient = Expr::bin(BinOp::Div, numerator, poly_to_expr(&slope, &arg_gens));
    let result = if negated { negate(quotient) } else { quotient };
    let out = lower(&result, &mut merged)?;
    Ok(Some(ratfun_to_expr(&out, &merged, DenForm::Expanded)))
}

/// `∫Sin(v)^m*Cos(v)dv` and `∫Cos(v)^m*Sin(v)dv`.
fn trig_power_pair(rf: &RatFun, var: &str, gens: &Gens) -> Option<Expr> {
    let terms = ordered_terms(rf.numer());
    if terms.len() != 1 {
        return None;
    }
    let denominator = constant_value(rf.denom())?;
    let (coeff, mono) = &terms[0];
    let coeff = rat_div(coeff, &denominator)?;
    let sin_name = display(&Expr::call("sin", vec![Expr::Var(var.to_string())]));
    let cos_name = display(&Expr::call("cos", vec![Expr::Var(var.to_string())]));
    if mono.iter().any(|(g, _)| *g != sin_name && *g != cos_name) {
        return None;
    }
    let s = exponent_of(mono, &sin_name);
    let c = exponent_of(mono, &cos_name);
    // u = Sin needs exactly one Cos factor; u = Cos needs exactly one Sin.
    let (base_name, exponent, flip) = if c == 1 && s >= 1 {
        (sin_name, s, false)
    } else if s == 1 && c >= 1 {
        (cos_name, c, true)
    } else {
        return None;
    };
    let scale = rat_div(&coeff, &rat_int(i64::from(exponent) + 1))?;
    let scale = if flip { -scale } else { scale };
    let powered = Expr::bin(
        BinOp::Pow,
        gens.expr(&base_name),
        int_expr(i64::from(exponent) + 1),
    );
    Some(scaled_signed(powered, &scale))
}

/// Split a polynomial into `coefficient * g^power` when exactly one generator
/// depends on `var`, and it does so identically in every term.
fn single_generator_factor(p: &MPoly, var: &str, gens: &Gens) -> Option<(MPoly, String, u32)> {
    let mut generator: Option<(String, u32)> = None;
    let mut coeff: Vec<(Rat, Mono)> = Vec::new();
    for (c, mut m) in mpoly_terms(p) {
        let dependents: Vec<(String, u32)> = m
            .iter()
            .filter(|(g, _)| g != var && gens.depends_on(g, var))
            .cloned()
            .collect();
        if dependents.len() != 1 {
            return None;
        }
        let (name, power) = dependents[0].clone();
        match &generator {
            None => generator = Some((name.clone(), power)),
            Some((n, e)) if *n == name && *e == power => {}
            Some(_) => return None,
        }
        m.retain(|(g, _)| *g != name);
        coeff.push((c, m));
    }
    let (name, power) = generator?;
    Some((mpoly_from_terms(&coeff), name, power))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse one frees expression, the way `CasExpressions.parse` does.
    fn parse(source: &str) -> Expr {
        crate::cas::engine::parse_expression(source).expect("parses")
    }

    fn show(op: fn(&Expr) -> Result<Expr, CasError>, source: &str) -> String {
        display(&op(&parse(source)).expect("ok"))
    }

    fn show_var(op: fn(&Expr, &str) -> Result<Expr, CasError>, source: &str, var: &str) -> String {
        display(&op(&parse(source), var).expect("ok"))
    }

    // ── ground truth: every string below came from the real Java engine ────

    #[test]
    fn factor_matches_the_oracle() {
        assert_eq!(show(factor, "x^2 - 1"), "(-1+x)*(1+x)");
        assert_eq!(show(factor, "x^2 + 2*x + 1"), "(1+x)^2");
        assert_eq!(show(factor, "2*x^2 + 4*x + 2"), "2*(1+x)^2");
        assert_eq!(show(factor, "x^4 - 1"), "(-1+x)*(1+x)*(1+x^2)");
        assert_eq!(show(factor, "x^2 + 1"), "1+x^2");
        assert_eq!(show(factor, "x^3 - 2"), "-2+x^3");
        assert_eq!(show(factor, "6*x^2 + 5*x + 1"), "(1+2*x)*(1+3*x)");
        assert_eq!(show(factor, "x^3-6*x^2+11*x-6"), "(-3+x)*(-2+x)*(-1+x)");
        assert_eq!(show(factor, "x/2 + 1/3"), "1/6*(2+3*x)");
        assert_eq!(show(factor, "(x^2-1)/(x^2+2*x+1)"), "(-1+x)/(1+x)");
        assert_eq!(show(factor, "3*x + 3*y"), "3*(x+y)");
        assert_eq!(show(factor, "s^2 + 3*s + 2"), "(1+s)*(2+s)");
        assert_eq!(show(factor, "x^2/4 - 1"), "1/4*(-2+x)*(2+x)");
        assert_eq!(show(factor, "2*x + 4"), "2*(2+x)");
        assert_eq!(show(factor, "-x - 1"), "-1-x");
        assert_eq!(show(factor, "2/3*x + 4/3"), "2/3*(2+x)");
        assert_eq!(show(factor, "x^3 + 3*x^2 + 3*x + 1"), "(1+x)^3");
        assert_eq!(show(factor, "x"), "x");
        assert_eq!(show(factor, "5"), "5");
    }

    #[test]
    fn expand_matches_the_oracle() {
        assert_eq!(show(expand, "(x+1)^3"), "1+3*x+3*x^2+x^3");
        assert_eq!(show(expand, "(x+1)*(x-1)"), "-1+x^2");
        assert_eq!(show(expand, "(x+y)^2"), "x^2+2*x*y+y^2");
        assert_eq!(show(expand, "2*(x+3)"), "6+2*x");
        assert_eq!(show(expand, "(x+1)*(x+2)*(x+3)"), "6+11*x+6*x^2+x^3");
        assert_eq!(show(expand, "(s+1)*(s+2)"), "2+3*s+s^2");
        assert_eq!(show(expand, "(2*x+3)^2"), "9+12*x+4*x^2");
        assert_eq!(show(expand, "-(x+1)"), "-1-x");
        assert_eq!(show(expand, "(x+1)^2*(x-1)^2"), "1-2*x^2+x^4");
        assert_eq!(show(expand, "(x+1)^2/2"), "1/2+x+x^2/2");
        assert_eq!(show(expand, "(x+1)^2/3"), "1/3+2/3*x+x^2/3");
        assert_eq!(show(expand, "(x+1)/3"), "1/3+x/3");
        assert_eq!(show(expand, "(a+b)*(c+d)"), "a*c+b*c+a*d+b*d");
        assert_eq!(show(expand, "1/(x+1)"), "1/(1+x)");
    }

    #[test]
    fn together_matches_the_oracle() {
        assert_eq!(show(together, "1/(s+1) + 1/(s+2)"), "(3+2*s)/((1+s)*(2+s))");
        assert_eq!(show(together, "1/x + 1/y"), "(x+y)/(x*y)");
        assert_eq!(show(together, "x + 1/x"), "(1+x^2)/x");
        assert_eq!(show(together, "a/b + c/d"), "(b*c+a*d)/(b*d)");
        assert_eq!(show(together, "x + 1"), "1+x");
        assert_eq!(show(together, "1/(s^2) + 1/s"), "(1+s)/s^2");
        assert_eq!(show(together, "x/2 + 1/3"), "1/6*(2+3*x)");
        assert_eq!(show(together, "1/2 + 1/3"), "5/6");
    }

    #[test]
    fn cancel_matches_the_oracle() {
        assert_eq!(show(cancel, "(x^2-1)/(x-1)"), "1+x");
        assert_eq!(show(cancel, "(2*x^2+4*x)/(4*x)"), "1/2*(2+x)");
        assert_eq!(show(cancel, "x/x"), "1");
        assert_eq!(show(cancel, "(x^2-1)/(x^2+2*x+1)"), "(-1+x)/(1+x)");
        assert_eq!(show(cancel, "(s^2+3*s+2)/(s+1)"), "2+s");
        assert_eq!(show(cancel, "(x^3-1)/(x^2-1)"), "(1+x+x^2)/(1+x)");
    }

    #[test]
    fn numerator_and_denominator_are_syntactic_like_symja() {
        assert_eq!(show(numerator, "(x+1)/(x-1)"), "1+x");
        assert_eq!(show(denominator, "(x+1)/(x-1)"), "-1+x");
        assert_eq!(show(numerator, "x+1"), "1+x");
        assert_eq!(show(denominator, "x+1"), "1");
        // Not combined over a common denominator — Symja's is syntactic too.
        assert_eq!(show(denominator, "1/(s+1) + 1/(s+2)"), "1");
        assert_eq!(show(numerator, "3/4"), "3");
        assert_eq!(show(denominator, "3/4"), "4");
        assert_eq!(show(numerator, "x/2"), "x");
        assert_eq!(show(denominator, "x/2"), "2");
        assert_eq!(show(numerator, "(x^2-1)/((x-1)*(x+3))"), "-1+x^2");
        assert_eq!(show(denominator, "(x^2-1)/((x-1)*(x+3))"), "(-1+x)*(3+x)");
        assert_eq!(show(numerator, "(x+1)^2/(x-1)"), "(1+x)^2");
        assert_eq!(show(denominator, "(x+1)/(2*(x-1))"), "2*(-1+x)");
        assert_eq!(show(denominator, "1/x^2"), "x^2");
        assert_eq!(show(numerator, "2/(3*x)"), "2");
        assert_eq!(show(denominator, "2/(3*x)"), "3*x");
    }

    #[test]
    fn apart_matches_the_oracle() {
        assert_eq!(
            show_var(apart, "(s + 3)/(s^2 + 3*s + 2)", "s"),
            "2/(1+s)-1/(2+s)"
        );
        assert_eq!(show_var(apart, "1/(s^2*(s+1))", "s"), "1/s^2-1/s+1/(1+s)");
        assert_eq!(show_var(apart, "(s^3+1)/(s^2-1)", "s"), "s+1/(-1+s)");
        assert_eq!(
            show_var(apart, "1/(s*(s+1)^2)", "s"),
            "1/s-1/(1+s)^2-1/(1+s)"
        );
        assert_eq!(show_var(apart, "(2*s+3)/(s^2+4)", "s"), "(3+2*s)/(4+s^2)");
        assert_eq!(
            show_var(apart, "1/(s^2-1)", "s"),
            "1/(2*(-1+s))-1/(2*(1+s))"
        );
        assert_eq!(show_var(apart, "(s^2+1)/(s+1)", "s"), "-1+s+2/(1+s)");
        assert_eq!(show_var(apart, "1/(s+1)", "s"), "1/(1+s)");
        assert_eq!(show_var(apart, "1/(s^2+2*s+1)", "s"), "1/(1+s)^2");
        assert_eq!(
            show_var(apart, "(3*s+5)/(s^2+3*s+2)", "s"),
            "2/(1+s)+1/(2+s)"
        );
        assert_eq!(
            show_var(apart, "1/((s+1)*(s+2)*(s+3))", "s"),
            "1/(2*(1+s))-1/(2+s)+1/(2*(3+s))"
        );
        assert_eq!(show_var(apart, "1/(s^3+s)", "s"), "1/s-s/(1+s^2)");
        assert_eq!(show_var(apart, "(s+2)/(s*(s+1))", "s"), "2/s-1/(1+s)");
        assert_eq!(
            show_var(apart, "(2*s+3)/(s^2+4*s+4)", "s"),
            "-1/(2+s)^2+2/(2+s)"
        );
    }

    #[test]
    fn collect_matches_the_oracle() {
        assert_eq!(show_var(collect, "x*a + x*b + c", "x"), "c+(a+b)*x");
        assert_eq!(
            show_var(collect, "a*x^2 + b*x^2 + c*x + d", "x"),
            "d+c*x+(a+b)*x^2"
        );
        assert_eq!(show_var(collect, "x^2 + x + 1", "x"), "1+x+x^2");
        assert_eq!(show_var(collect, "a*x + b", "x"), "b+a*x");
        assert_eq!(show_var(collect, "(x+1)*(x+2)", "x"), "2+3*x+x^2");
        assert_eq!(show_var(collect, "x*y + x*z", "x"), "x*(y+z)");
        assert_eq!(show_var(collect, "a*x + a*y", "a"), "a*(x+y)");
        assert_eq!(show_var(collect, "2*x + 3*x", "x"), "5*x");
        assert_eq!(show_var(collect, "x/2 + x/3", "x"), "5/6*x");
        assert_eq!(show_var(collect, "x + y", "z"), "x+y");
        assert_eq!(show_var(collect, "a*x^2 + x", "x"), "x+a*x^2");
    }

    #[test]
    fn differentiate_matches_the_oracle() {
        assert_eq!(show_var(differentiate, "x^2", "x"), "2*x");
        assert_eq!(show_var(differentiate, "sin(x)", "x"), "Cos(x)");
        assert_eq!(show_var(differentiate, "x*y", "x"), "y");
        assert_eq!(show_var(differentiate, "x^3 + 2*x", "x"), "2+3*x^2");
        assert_eq!(show_var(differentiate, "x^2*y + y^3", "y"), "x^2+3*y^2");
        assert_eq!(
            show_var(differentiate, "sin(x)*cos(x)", "x"),
            "Cos(x)^2-Sin(x)^2"
        );
        assert_eq!(show_var(differentiate, "x^2", "y"), "0");
    }

    #[test]
    fn integrate_matches_the_oracle_inside_the_table() {
        assert_eq!(show_var(integrate, "x^2", "x"), "x^3/3");
        assert_eq!(show_var(integrate, "x", "x"), "x^2/2");
        assert_eq!(show_var(integrate, "1/x", "x"), "ln(x)");
        assert_eq!(show_var(integrate, "exp(x)", "x"), "E^x");
        assert_eq!(show_var(integrate, "sin(x)", "x"), "-Cos(x)");
        assert_eq!(show_var(integrate, "cos(x)", "x"), "Sin(x)");
        assert_eq!(show_var(integrate, "exp(2*x)", "x"), "E^(2*x)/2");
        assert_eq!(show_var(integrate, "sin(3*x)", "x"), "-Cos(3*x)/3");
        assert_eq!(show_var(integrate, "1/(x+1)", "x"), "ln(1+x)");
        assert_eq!(show_var(integrate, "x^2 + 2*x + 1", "x"), "x+x^2+x^3/3");
        assert_eq!(show_var(integrate, "1", "x"), "x");
        assert_eq!(show_var(integrate, "5", "x"), "5*x");
        assert_eq!(show_var(integrate, "x^5", "x"), "x^6/6");
        assert_eq!(show_var(integrate, "y", "x"), "x*y");
        assert_eq!(show_var(integrate, "x/2", "x"), "x^2/4");
        assert_eq!(show_var(integrate, "3/x", "x"), "3*ln(x)");
        assert_eq!(show_var(integrate, "1/(3*x)", "x"), "ln(x)/3");
        assert_eq!(show_var(integrate, "1/(2*x+3)", "x"), "ln(3+2*x)/2");
        assert_eq!(show_var(integrate, "(2*x+1)^3", "x"), "(1+2*x)^4/8");
        assert_eq!(show_var(integrate, "(x+1)^2", "x"), "(1+x)^3/3");
        assert_eq!(show_var(integrate, "cos(2*x+1)", "x"), "Sin(1+2*x)/2");
        assert_eq!(show_var(integrate, "exp(3*x+2)", "x"), "E^(2+3*x)/3");
        assert_eq!(show_var(integrate, "x/(x^2+1)", "x"), "ln(1+x^2)/2");
        assert_eq!(show_var(integrate, "2*x/(x^2+1)", "x"), "ln(1+x^2)");
        assert_eq!(
            show_var(integrate, "(x+1)/(x^2+2*x+3)", "x"),
            "ln(3+2*x+x^2)/2"
        );
        assert_eq!(show_var(integrate, "(3*x^2)/(x^3+1)", "x"), "ln(1+x^3)");
        assert_eq!(show_var(integrate, "x*exp(x^2)", "x"), "E^(x^2)/2");
        assert_eq!(show_var(integrate, "x*sin(x^2)", "x"), "-Cos(x^2)/2");
        assert_eq!(show_var(integrate, "x^3*exp(x^4)", "x"), "E^(x^4)/4");
        assert_eq!(show_var(integrate, "sin(x)*cos(x)", "x"), "Sin(x)^2/2");
        assert_eq!(show_var(integrate, "cos(x)*sin(x)^2", "x"), "Sin(x)^3/3");
        assert_eq!(show_var(integrate, "cos(x)^3*sin(x)", "x"), "-Cos(x)^4/4");
        assert_eq!(show_var(integrate, "x^(-2)", "x"), "-1/x");
    }

    /// The whole point of the table: everything outside it is **refused**,
    /// never guessed. Symja answers all of these.
    #[test]
    fn integrate_refuses_everything_outside_the_table() {
        for source in [
            "exp(x^2)",
            "sin(x)/x",
            "exp(x)/x",
            "1/ln(x)",
            "sin(x^2)",
            "tan(x)",
            "ln(x)",
            "sqrt(x)",
            "1/sqrt(x)",
            "1/(x^2+1)",
            "1/(x^2-1)",
            "x*exp(x)",
            "x^2*exp(x)",
            "exp(x)*sin(x)",
            "cos(x)^2",
            "sqrt(1-x^2)",
            "abs(x)",
            "x^n",
        ] {
            let e = parse(source);
            assert!(
                matches!(integrate(&e, "x"), Err(CasError::NoClosedForm { .. })),
                "integrate({source}) must refuse by name, got {:?}",
                integrate(&e, "x").map(|r| display(&r))
            );
        }
    }

    #[test]
    fn simplify_matches_the_oracle_for_rational_algebra() {
        assert_eq!(show(simplify, "(x^2-1)/(x-1)"), "1+x");
        assert_eq!(show(simplify, "x + x"), "2*x");
        assert_eq!(show(simplify, "x/x"), "1");
        assert_eq!(show(simplify, "(x+1)^2 - (x^2+2*x+1)"), "0");
        assert_eq!(show(simplify, "2*x/4"), "x/2");
        assert_eq!(show(simplify, "1/(1/x)"), "x");
        assert_eq!(show(simplify, "x*0"), "0");
        assert_eq!(show(simplify, "(x^3-1)/(x-1)"), "1+x+x^2");
        assert_eq!(show(simplify, "x^2/x"), "x");
        assert_eq!(show(simplify, "(x+1)*(x-1)"), "-1+x^2");
        assert_eq!(show(simplify, "a*x + b*x"), "(a+b)*x");
        assert_eq!(show(simplify, "(x+y)^2 - (x-y)^2"), "4*x*y");
        assert_eq!(show(simplify, "x^2 + 2*x + 1"), "(1+x)^2");
        assert_eq!(show(simplify, "(2*x+2)/2"), "1+x");
        assert_eq!(show(simplify, "x - x"), "0");
        assert_eq!(show(simplify, "2^10"), "1024");
        assert_eq!(show(simplify, "1/3 + 1/6"), "1/2");
        assert_eq!(show(simplify, "(s+3)/(s^2+3*s+2)"), "(3+s)/(2+3*s+s^2)");
        assert_eq!(show(simplify, "1/(s^2*(s+1))"), "1/(s^2+s^3)");
        assert_eq!(show(simplify, "1/(s+1) + 1/(s+2)"), "1/(1+s)+1/(2+s)");
    }

    /// The documented divergence: no transcendental identities.
    #[test]
    fn simplify_does_not_know_trig_identities() {
        assert_eq!(show(simplify, "sin(x)^2 + cos(x)^2"), "Cos(x)^2+Sin(x)^2");
        assert_eq!(show(simplify, "sqrt(x^2)"), "Sqrt(x^2)");
        assert_eq!(show(simplify, "ln(x) + ln(y)"), "ln(x)+ln(y)");
    }

    /// Non-integer literals are exact here where Symja falls back to floats.
    #[test]
    fn fractional_literals_stay_exact() {
        assert_eq!(show(factor, "1.5*x^2 - 1.5"), "3/2*(-1+x)*(1+x)");
        assert_eq!(show(expand, "0.5*(x+1)"), "1/2+x/2");
        assert_eq!(show(cancel, "(0.5*x^2 - 0.5)/(x-1)"), "1/2*(1+x)");
    }

    /// The monomial gcd plus the univariate factoriser reaches most of what a
    /// user actually types, even when more than one symbol is present.
    #[test]
    fn multivariate_factor_pulls_out_what_it_can() {
        assert_eq!(show(factor, "x^3 + x"), "x*(1+x^2)");
        assert_eq!(show(factor, "x^2*y + x*y^2"), "x*y*(x+y)");
        assert_eq!(show(factor, "a*x^2 + 2*a*x + a"), "a*(1+x)^2");
    }

    /// `Apart` leaves alone exactly what Symja leaves alone: an expression that
    /// is not a rational function of the requested variable.
    #[test]
    fn apart_returns_the_input_when_there_is_nothing_to_decompose() {
        assert_eq!(show_var(apart, "1/(a*s + b)", "s"), "1/(b+a*s)");
        assert_eq!(
            show_var(apart, "(s + 3)/(s^2 + 3*s + 2)", "x"),
            "(3+s)/((1+s)*(2+s))"
        );
        assert_eq!(show_var(apart, "5", "s"), "5");
        assert_eq!(show_var(apart, "s", "s"), "s");
        assert_eq!(show_var(apart, "(s+1)/(s+1)", "s"), "1");
    }

    /// The divergences the module header lists, pinned so a future change to
    /// any of them is a deliberate decision rather than a silent drift. Each
    /// `oracle` string is what the real Java engine returned.
    #[test]
    fn documented_divergences_from_symja_are_pinned() {
        // 3 — sign placement inside `Factor`: same polynomial, other spelling.
        assert_eq!(show(factor, "100*x^2 - 100"), "100*(-1+x)*(1+x)");
        //   oracle: -100*(1-x)*(1+x)
        assert_eq!(show(factor, "1 - x^2"), "-(-1+x)*(1+x)");
        //   oracle: (1-x)*(1+x)

        // 4 — a rational input stays one fraction instead of being distributed.
        assert_eq!(show(cancel, "(x+1)/(x+2)"), "(1+x)/(2+x)");
        //   oracle: 1/(2+x)+x/(2+x)
        assert_eq!(show(expand, "(x+1)^2/(x-1)"), "(1+2*x+x^2)/(-1+x)");
        //   oracle: (1+x)^2/(-1+x)

        // 6 — genuinely multivariate polynomials are not factored.
        assert_eq!(show(factor, "x^2 - y^2"), "x^2-y^2");
        //   oracle: (x-y)*(x+y)

        // `Together` normalises the sign; the oracle carries it in both halves.
        assert_eq!(show(together, "2/(s+1) - 1/(s+2)"), "(3+s)/((1+s)*(2+s))");
        //   oracle: (-3-s)/((-2-s)*(1+s))

        // `Apart` puts the polynomial part first; the oracle's `Plus` ordering
        // sometimes places it last.
        assert_eq!(show_var(apart, "(s^3+1)/(s^2-1)", "s"), "s+1/(-1+s)");
        //   oracle: 1/(-1+s)+s

        // `Integrate` picks the antiderivative that is real for large x; the
        // oracle picks the other branch. They differ by a constant.
        assert_eq!(show_var(integrate, "1/(x-2)", "x"), "ln(-2+x)");
        //   oracle: ln(2-x)
    }

    #[test]
    fn unsupported_nodes_carry_the_parent_engines_wording() {
        let cases: [(Expr, &str); 5] = [
            (
                Expr::Str("hi".into()),
                "string literals are not supported in CAS expressions",
            ),
            (
                Expr::ArrayAccess {
                    name: "a".into(),
                    indices: vec![Expr::num(1.0)],
                },
                "array indexing is not supported in CAS expressions",
            ),
            (
                Expr::ArrayLiteral(vec![Expr::num(1.0)]),
                "array/matrix literals are not supported in CAS expressions",
            ),
            (
                Expr::Not(Box::new(Expr::var("x"))),
                "boolean negation is not supported in CAS expressions",
            ),
            (
                Expr::call("mysteryfn", vec![Expr::var("x")]),
                "function not supported in CAS expressions: mysteryfn",
            ),
        ];
        for (e, message) in cases {
            let err = factor(&e).expect_err("must refuse");
            assert_eq!(err.to_string(), message);
        }
    }

    #[test]
    fn symja_input_is_transcribed_exactly() {
        assert_eq!(to_symja_input(&Expr::num(5.0)).unwrap(), "5");
        assert_eq!(to_symja_input(&Expr::num(2.5)).unwrap(), "2.5");
        assert_eq!(
            to_symja_input(&Expr::Num {
                value: 1.0,
                unit: None,
                is_imaginary: true
            })
            .unwrap(),
            "(1*I)"
        );
        assert_eq!(to_symja_input(&parse("x + 1")).unwrap(), "(x+1)");
        assert_eq!(to_symja_input(&parse("x^2")).unwrap(), "(x^2)");
        assert_eq!(to_symja_input(&parse("-x")).unwrap(), "(-(x))");
        assert_eq!(to_symja_input(&parse("ln(x)")).unwrap(), "Log(x)");
        assert_eq!(to_symja_input(&parse("sqrt(x)")).unwrap(), "Sqrt(x)");
        assert_eq!(
            to_symja_input(&parse("(s + 3)/(s^2 + 3*s + 2)")).unwrap(),
            "((s+3)/(((s^2)+(3*s))+2))"
        );
    }

    #[test]
    fn deeply_nested_input_is_refused_not_overflowed() {
        let mut e = Expr::var("x");
        for _ in 0..400 {
            e = Expr::bin(BinOp::Add, e, Expr::var("x"));
        }
        assert!(matches!(factor(&e), Err(CasError::TooDeep)));
    }

    #[test]
    fn huge_powers_become_opaque_rather_than_expanding() {
        // `(x+1)^100000` must not be multiplied out.
        let e = Expr::bin(BinOp::Pow, parse("x + 1"), Expr::num(100_000.0));
        let out = display(&factor(&e).expect("ok"));
        assert!(out.contains("^100000"), "{out}");
    }

    #[test]
    fn imaginary_literals_are_refused_by_name() {
        let e = Expr::Num {
            value: 1.0,
            unit: None,
            is_imaginary: true,
        };
        assert_eq!(
            factor(&e).expect_err("refused").to_string(),
            "imaginary literals are not supported in CAS expressions"
        );
    }
}
