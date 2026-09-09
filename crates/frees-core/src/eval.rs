//! Expression evaluation.
//!
//! Port of `../frEES/backend/core/src/main/java/com/frees/backend/ast/Evaluator.java`
//! (2,053 LOC, 226 dispatch arms).
//!
//! # What this pass covers
//!
//! Every [`Expr`] variant, every [`BinOp`] / [`CmpOp`] / [`LogicOp`], the full
//! `ConstantsRegistry` table, and the *scalar elementary* half of the Java
//! dispatch: arithmetic, rounding, trig/hyperbolic (and their inverses), logs,
//! the special functions `libm` provides, bit/number theory, descriptive
//! statistics, the orthogonal-polynomial recurrences and the two binding forms
//! `sum` / `product`.
//!
//! On top of that, [`eval_with`] dispatches the document-level families through
//! [`EvalContext`]: user `FUNCTION`s and `TABLE`s by name, the synthetic
//! `proc$<name>$<k>` procedure-output calls, the `fft$`/`conv$`/`linfit$`/
//! `polyfit$`/`interp2$` kernel synthetics, and the two quadrature intrinsics
//! (`Integral` / `GaussIntegral`, whose bodies live in [`crate::integral`]).
//!
//! The once-missing families have since been wired in place: fluid properties
//! (`prop$…`, Phase 5), the matrix decompositions incl. eigen (`crate::linalg`),
//! and the control systems (`crate::control::eval`, Phase 9). What still has no
//! kernel — e.g. the Euler decompose/rotate pair — is rejected with an explicit
//! `not yet supported: <name>` evaluation error rather than a wrong answer.
//!
//! # Design: a data-driven registry
//!
//! The Java file is a 226-arm `switch`. Here the dispatch is a table
//! ([`INTRINSICS`]) of `name -> (arity, body)`. Adding the remaining ~186 arms
//! is purely additive: append a row. Two body shapes exist:
//!
//! * [`Body::Strict`] — arguments are evaluated to `f64` first (the common case).
//! * [`Body::Lazy`] — the intrinsic sees the raw AST plus the environment, which
//!   is what `if` (evaluate only the taken branch), `sum`/`product` (bind the
//!   index variable) and the string intrinsics need.
//!
//! # Angles
//!
//! The Java engine is **radian-only**. `Math.sin/cos/tan/asin/acos/atan/atan2`
//! are used unwrapped, there is no degree/radian mode anywhere in
//! `Evaluator.java`, and `FunctionRegistry` documents every inverse-trig result
//! as `[rad]`. `angledeg` is the single degree-producing intrinsic and it does
//! the `* 180 / pi` conversion itself. This port matches: radians everywhere.
//!
//! # Determinism
//!
//! All transcendentals go through [`libm`] so a native run and a
//! `wasm32-unknown-unknown` run agree bit for bit; the host `f64` intrinsics
//! are deliberately not used.

// Float guards here are written `!(x > 0.0)` on purpose: the negation makes
// NaN take the reject branch, which `x <= 0.0` would not. Clippy's
// `neg_cmp_op_on_partial_ord` exists to catch the *accidental* form; here the
// NaN behaviour is the point, and it matches the Java guards being ported.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
// Numerical kernels index several parallel arrays (and 2-D `a[i][j]` slices)
// by the same loop variable, mirroring the Java/Fortran sources being
// transcribed. Iterator rewrites obscure that correspondence, so the indexed
// form stays.
#![allow(clippy::needless_range_loop)]
// Truncated constants such as `0.636619772` (2/pi) are transcribed verbatim
// from the Java `Evaluator.java` / Numerical Recipes coefficient tables.
// Substituting `std::f64::consts::*` would change the value in the last digits
// and break bit-parity with the oracle these tests compare against.
#![allow(clippy::approx_constant)]
// `mut_range_bound`: the Bessel `rjbesl` transcription reassigns `nstart`/`nend`
// inside the scan to drive the *inner* loop and then breaks out of the outer
// one — the captured outer bounds are intentionally stale, as in Apache's
// implementation.
#![allow(clippy::mut_range_bound)]

use crate::ast::{BinOp, CmpOp, Expr, LogicOp};
use crate::diag::{FreesError, Result};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::sync::OnceLock;

/// Variable bindings visible to an evaluation.
///
/// Deliberately **not** the default `RandomState` hasher. Every variable read
/// in every residual, in every Newton iteration, in every block, at every
/// integrator stage goes through this map, and a callgrind profile of a
/// transient solve put **22% of all instructions in SipHash** against 4.6% in
/// the actual numerics. `FxHasher` below is the same hash rustc uses for its
/// own identifier-keyed maps: far cheaper on short ASCII keys, which is all a
/// variable name ever is.
///
/// Two properties make the swap safe here rather than merely fast:
///
/// * *HashDoS resistance is irrelevant.* The keys are variable names from the
///   document being solved, the engine runs client-side in the user's own tab,
///   and a `DYNAMIC` with an absurd span is a far easier self-inflicted DoS
///   than engineered hash collisions.
/// * *Iteration order was never load-bearing.* `RandomState` reseeds per
///   process, so `Scope` order already differed on every run; anything relying
///   on it would have been flaky against the parity corpus long ago. Every
///   iteration site in the engine collects into a `HashSet` or walks an
///   ordered `BTreeMap`.
///
/// A fixed seed also makes iteration order stable run-to-run, which is
/// strictly better for debugging than what `RandomState` gave.
pub type Scope = HashMap<String, f64, BuildHasherDefault<FxHasher>>;

/// `rustc-hash`'s FxHash, inlined rather than taken as a dependency: it is
/// twenty lines, and the wasm budget has ~30 KiB of headroom that a new crate
/// would spend for identical code.
#[derive(Default, Clone, Copy)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

    #[inline]
    fn add(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(Self::SEED);
    }
}

impl std::hash::Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while rest.len() >= 8 {
            let (word, tail) = rest.split_at(8);
            self.add(u64::from_ne_bytes(word.try_into().unwrap()));
            rest = tail;
        }
        if rest.len() >= 4 {
            let (word, tail) = rest.split_at(4);
            self.add(u64::from(u32::from_ne_bytes(word.try_into().unwrap())));
            rest = tail;
        }
        if rest.len() >= 2 {
            let (word, tail) = rest.split_at(2);
            self.add(u64::from(u16::from_ne_bytes(word.try_into().unwrap())));
            rest = tail;
        }
        if let Some(&byte) = rest.first() {
            self.add(u64::from(byte));
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(u64::from(i));
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

/// The variable environment an expression is evaluated against.
///
/// The Java evaluator mutates the caller's `Map` to bind a `sum`/`product`
/// index and restores it in a `finally` block. Taking `&Scope` instead, this
/// port models the binding as a cons-cell chain: an inner binding shadows the
/// outer scope, costs no allocation, and cannot leak on an error path.
#[derive(Debug)]
pub enum Env<'a> {
    /// The caller's scope — the solver's current iterate.
    Root(&'a Scope),
    /// The caller's scope plus the document context (definitions), the
    /// counterpart of the Java `eval(expr, values, defs)` three-argument form.
    Doc {
        scope: &'a Scope,
        ctx: EvalContext<'a>,
    },
    /// A single shadowing binding (a `sum`/`product` index variable).
    Bind {
        name: &'a str,
        value: f64,
        parent: &'a Env<'a>,
    },
}

impl<'a> Env<'a> {
    /// Innermost binding of `name`, if any. `name` is expected lowercase.
    pub fn get(&self, name: &str) -> Option<f64> {
        let mut cursor = self;
        loop {
            match cursor {
                Env::Root(scope) => return scope.get(name).copied(),
                Env::Doc { scope, .. } => return scope.get(name).copied(),
                Env::Bind {
                    name: bound,
                    value,
                    parent,
                } => {
                    if *bound == name {
                        return Some(*value);
                    }
                    cursor = parent;
                }
            }
        }
    }

    /// The document context this environment roots in (empty for [`Env::Root`]).
    pub fn ctx(&self) -> EvalContext<'a> {
        let mut cursor = self;
        loop {
            match cursor {
                Env::Root(_) => return EvalContext::default(),
                Env::Doc { ctx, .. } => return *ctx,
                Env::Bind { parent, .. } => cursor = parent,
            }
        }
    }

    /// Materialize the full binding chain into an owned [`Scope`] — what the
    /// frozen kernel contracts ([`crate::integral`], [`crate::procedures`])
    /// take, mirroring the mutable `values` map the Java passes along.
    /// Inner bindings shadow outer ones.
    pub fn to_scope(&self) -> Scope {
        let mut binds: Vec<(&str, f64)> = Vec::new();
        let mut cursor = self;
        loop {
            match cursor {
                Env::Root(scope) | Env::Doc { scope, .. } => {
                    let mut out: Scope = (*scope).clone();
                    // Applied outermost-first so the innermost binding wins.
                    for (name, value) in binds.into_iter().rev() {
                        out.insert(name.to_string(), value);
                    }
                    return out;
                }
                Env::Bind {
                    name,
                    value,
                    parent,
                } => {
                    binds.push((name, *value));
                    cursor = parent;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Built-in constants (`ConstantsRegistry.java`)
// ---------------------------------------------------------------------------

/// A `#`-suffixed built-in constant.
///
/// Port of `ConstantsRegistry.Constant`. The Java parser substitutes these as
/// numeric literals at parse time; the evaluator resolves them as a fallback so
/// a hand-built AST behaves the same.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuiltinConstant {
    /// Canonical display name, e.g. `R#`.
    pub name: &'static str,
    /// Value in SI.
    pub value: f64,
    /// SI unit string; `None` for dimensionless.
    pub unit: Option<&'static str>,
    pub description: &'static str,
}

/// The full `ConstantsRegistry` table, in declaration order.
pub const CONSTANTS: &[BuiltinConstant] = &[
    // Mathematical
    BuiltinConstant {
        name: "pi#",
        value: std::f64::consts::PI,
        unit: None,
        description: "Ratio of a circle's circumference to its diameter",
    },
    BuiltinConstant {
        name: "e#",
        value: std::f64::consts::E,
        unit: None,
        description: "Euler's number (base of the natural logarithm)",
    },
    // Universal physical constants (CODATA / SI exact where applicable)
    BuiltinConstant {
        name: "R#",
        value: 8.314_462_618,
        unit: Some("J/mol-K"),
        description: "Universal (molar) gas constant",
    },
    BuiltinConstant {
        name: "g#",
        value: 9.806_65,
        unit: Some("m/s^2"),
        description: "Standard acceleration of gravity",
    },
    BuiltinConstant {
        name: "Na#",
        value: 6.022_140_76e23,
        unit: Some("1/mol"),
        description: "Avogadro constant",
    },
    BuiltinConstant {
        name: "k#",
        value: 1.380_649e-23,
        unit: Some("J/K"),
        description: "Boltzmann constant",
    },
    BuiltinConstant {
        name: "h#",
        value: 6.626_070_15e-34,
        unit: Some("J-s"),
        description: "Planck constant",
    },
    BuiltinConstant {
        name: "c#",
        value: 299_792_458.0,
        unit: Some("m/s"),
        description: "Speed of light in vacuum",
    },
    BuiltinConstant {
        name: "sigma#",
        value: 5.670_374_419e-8,
        unit: Some("W/m^2-K^4"),
        description: "Stefan-Boltzmann constant",
    },
    BuiltinConstant {
        name: "Gc#",
        value: 6.674_30e-11,
        unit: Some("m^3/kg-s^2"),
        description: "Newtonian constant of gravitation",
    },
    BuiltinConstant {
        name: "qe#",
        value: 1.602_176_634e-19,
        unit: Some("Coulomb"),
        description: "Elementary charge",
    },
];

/// The built-in constant for `name`, matched case-insensitively.
///
/// Port of `ConstantsRegistry.lookup`.
pub fn lookup_constant(name: &str) -> Option<&'static BuiltinConstant> {
    CONSTANTS.iter().find(|c| c.name.eq_ignore_ascii_case(name))
}

// ---------------------------------------------------------------------------
// Intrinsic registry
// ---------------------------------------------------------------------------

/// How many arguments an intrinsic accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    Exact(usize),
    /// Inclusive `[min, max]`.
    Range(usize, usize),
    /// One of a fixed set (`if` takes 3 or 5).
    OneOf(&'static [usize]),
    AtLeast(usize),
    /// Variadic, zero allowed.
    Any,
}

impl Arity {
    fn accepts(self, n: usize) -> bool {
        match self {
            Arity::Exact(k) => n == k,
            Arity::Range(lo, hi) => n >= lo && n <= hi,
            Arity::OneOf(set) => set.contains(&n),
            Arity::AtLeast(lo) => n >= lo,
            Arity::Any => true,
        }
    }

    fn describe(self) -> String {
        match self {
            Arity::Exact(1) => "1 argument".to_string(),
            Arity::Exact(k) => format!("{k} arguments"),
            Arity::Range(lo, hi) => format!("{lo} to {hi} arguments"),
            Arity::OneOf(set) => {
                let names: Vec<String> = set.iter().map(|k| k.to_string()).collect();
                format!("{} arguments", names.join(" or "))
            }
            Arity::AtLeast(lo) => format!("at least {lo} argument(s)"),
            Arity::Any => "any number of arguments".to_string(),
        }
    }
}

/// An intrinsic whose arguments are all evaluated to scalars first.
pub type StrictFn = fn(&str, &[f64]) -> Result<f64>;

/// An intrinsic that needs the unevaluated AST — lazy branches, bound indices,
/// string-literal arguments.
pub type LazyFn = for<'a> fn(&str, &'a [Expr], &'a Env<'a>) -> Result<f64>;

/// The two shapes an intrinsic body can take.
#[derive(Clone, Copy)]
pub enum Body {
    Strict(StrictFn),
    Lazy(LazyFn),
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Body::Strict(_) => f.write_str("Strict(..)"),
            Body::Lazy(_) => f.write_str("Lazy(..)"),
        }
    }
}

/// One row of the dispatch table.
#[derive(Debug, Clone, Copy)]
pub struct Intrinsic {
    /// Lowercase name, exactly as `Expr::Call` stores it.
    pub name: &'static str,
    pub arity: Arity,
    pub body: Body,
}

macro_rules! strict {
    ($name:literal, $arity:expr, $f:expr) => {
        Intrinsic {
            name: $name,
            arity: $arity,
            body: Body::Strict($f),
        }
    };
}

macro_rules! lazy {
    ($name:literal, $arity:expr, $f:expr) => {
        Intrinsic {
            name: $name,
            arity: $arity,
            body: Body::Lazy($f),
        }
    };
}

/// The dispatch table. Append rows to grow it; order is irrelevant.
pub const INTRINSICS: &[Intrinsic] = &[
    // ----- elementary algebra --------------------------------------------
    strict!("abs", Arity::Exact(1), |_, a| Ok(libm::fabs(a[0]))),
    strict!("sign", Arity::Exact(1), |_, a| Ok(java_signum(a[0]))),
    strict!("sqrt", Arity::Exact(1), |n, a| {
        if a[0] < 0.0 {
            return Err(domain(
                n,
                format_args!("square root of a negative number ({})", a[0]),
            ));
        }
        Ok(libm::sqrt(a[0]))
    }),
    strict!("cbrt", Arity::Exact(1), |_, a| Ok(libm::cbrt(a[0]))),
    strict!("sqr", Arity::Exact(1), |_, a| Ok(a[0] * a[0])),
    strict!("cube", Arity::Exact(1), |_, a| Ok(a[0] * a[0] * a[0])),
    strict!("hypot", Arity::Exact(2), |_, a| Ok(libm::hypot(a[0], a[1]))),
    strict!("exp", Arity::Exact(1), |_, a| Ok(libm::exp(a[0]))),
    strict!("ln", Arity::Exact(1), |n, a| check_log(n, a[0])
        .map(libm::log)),
    strict!("log10", Arity::Exact(1), |n, a| check_log(n, a[0])
        .map(libm::log10)),
    // The Java arm is `Math.log(x) / Math.log(2.0)`, not a dedicated log2.
    strict!("log2", Arity::Exact(1), |n, a| {
        check_log(n, a[0]).map(|x| libm::log(x) / libm::log(2.0))
    }),
    // `log` is not a Java arm; frees/EES spell base-10 as `log10`. Kept as an
    // alias so the classic spelling resolves instead of "unknown function".
    strict!("log", Arity::Exact(1), |n, a| check_log(n, a[0])
        .map(libm::log10)),
    // ----- rounding & integer parts ---------------------------------------
    strict!("floor", Arity::Exact(1), |_, a| Ok(libm::floor(a[0]))),
    strict!("ceil", Arity::Exact(1), |_, a| Ok(libm::ceil(a[0]))),
    strict!("trunc", Arity::Exact(1), |_, a| Ok(libm::trunc(a[0]))),
    strict!("int", Arity::Exact(1), |_, a| Ok(libm::trunc(a[0]))),
    strict!("round", Arity::Range(1, 2), |_, a| {
        if a.len() == 2 {
            let factor = libm::pow(10.0, java_round(a[1]));
            return Ok(java_round(a[0] * factor) / factor);
        }
        Ok(java_round(a[0]))
    }),
    // ----- trigonometry (radians) -----------------------------------------
    strict!("sin", Arity::Exact(1), |_, a| Ok(libm::sin(a[0]))),
    strict!("cos", Arity::Exact(1), |_, a| Ok(libm::cos(a[0]))),
    strict!("tan", Arity::Exact(1), |_, a| Ok(libm::tan(a[0]))),
    strict!("asin", Arity::Exact(1), |n, a| check_unit_interval(n, a[0])
        .map(libm::asin)),
    strict!("arcsin", Arity::Exact(1), |n, a| check_unit_interval(
        n, a[0]
    )
    .map(libm::asin)),
    strict!("acos", Arity::Exact(1), |n, a| check_unit_interval(n, a[0])
        .map(libm::acos)),
    strict!("arccos", Arity::Exact(1), |n, a| check_unit_interval(
        n, a[0]
    )
    .map(libm::acos)),
    strict!("atan", Arity::Exact(1), |_, a| Ok(libm::atan(a[0]))),
    strict!("arctan", Arity::Exact(1), |_, a| Ok(libm::atan(a[0]))),
    strict!("atan2", Arity::Exact(2), |_, a| Ok(libm::atan2(a[0], a[1]))),
    // ----- hyperbolic ------------------------------------------------------
    strict!("sinh", Arity::Exact(1), |_, a| Ok(libm::sinh(a[0]))),
    strict!("cosh", Arity::Exact(1), |_, a| Ok(libm::cosh(a[0]))),
    strict!("tanh", Arity::Exact(1), |_, a| Ok(libm::tanh(a[0]))),
    // Java spells the inverses out as logarithms rather than calling Math, so
    // the same closed forms are used here for bit-level agreement.
    strict!("arcsinh", Arity::Exact(1), |_, a| {
        Ok(libm::log(a[0] + libm::sqrt(a[0] * a[0] + 1.0)))
    }),
    strict!("arccosh", Arity::Exact(1), |n, a| {
        if a[0] < 1.0 {
            return Err(domain(
                n,
                format_args!("argument must be >= 1.0, got {}", a[0]),
            ));
        }
        Ok(libm::log(a[0] + libm::sqrt(a[0] * a[0] - 1.0)))
    }),
    strict!("arctanh", Arity::Exact(1), |n, a| {
        if libm::fabs(a[0]) >= 1.0 {
            return Err(domain(
                n,
                format_args!("argument must be in (-1, 1), got {}", a[0]),
            ));
        }
        Ok(0.5 * libm::log((1.0 + a[0]) / (1.0 - a[0])))
    }),
    // ----- piecewise -------------------------------------------------------
    strict!("step", Arity::Exact(1), |_, a| Ok(if a[0] >= 0.0 {
        1.0
    } else {
        0.0
    })),
    // `ramp` has no Java arm; defined as the integral of `step` so the two
    // agree at the origin (`step(0) = 1`, `ramp(0) = 0`).
    strict!("ramp", Arity::Exact(1), |_, a| Ok(if a[0] >= 0.0 {
        a[0]
    } else {
        0.0
    })),
    // ----- reductions over the argument list --------------------------------
    strict!("min", Arity::AtLeast(1), |_, a| {
        Ok(a.iter().copied().fold(f64::INFINITY, java_min))
    }),
    strict!("max", Arity::AtLeast(1), |_, a| {
        Ok(a.iter().copied().fold(f64::NEG_INFINITY, java_max))
    }),
    // ----- number theory & bitwise -----------------------------------------
    strict!("mod", Arity::Exact(2), |n, a| java_mod(n, a[0], a[1])),
    strict!("rem", Arity::Exact(2), |n, a| java_mod(n, a[0], a[1])),
    strict!("gcd", Arity::Exact(2), |_, a| Ok(
        gcd_i64(a[0] as i64, a[1] as i64) as f64
    )),
    strict!("lcm", Arity::Exact(2), |n, a| {
        let (x, y) = (a[0] as i64, a[1] as i64);
        let g = gcd_i64(x, y);
        if g == 0 {
            return Ok(0.0);
        }
        match (x / g).checked_mul(y) {
            Some(v) => Ok(v.unsigned_abs() as f64),
            None => Err(domain(n, format_args!("overflow computing lcm({x}, {y})"))),
        }
    }),
    strict!("bitand", Arity::Exact(2), |_, a| Ok(
        ((a[0] as i64) & (a[1] as i64)) as f64
    )),
    strict!("bitor", Arity::Exact(2), |_, a| Ok(
        ((a[0] as i64) | (a[1] as i64)) as f64
    )),
    strict!("bitxor", Arity::Exact(2), |_, a| Ok(
        ((a[0] as i64) ^ (a[1] as i64)) as f64
    )),
    strict!("bitnot", Arity::Exact(1), |_, a| Ok(!(a[0] as i64) as f64)),
    strict!("bitshiftl", Arity::Exact(2), |_, a| Ok(
        (a[0] as i64).wrapping_shl(a[1] as i32 as u32) as f64
    )),
    strict!("bitshiftr", Arity::Exact(2), |_, a| Ok(
        (a[0] as i64).wrapping_shr(a[1] as i32 as u32) as f64
    )),
    // ----- special functions ------------------------------------------------
    strict!("erf", Arity::Exact(1), |_, a| Ok(libm::erf(a[0]))),
    strict!("erfc", Arity::Exact(1), |_, a| Ok(libm::erfc(a[0]))),
    strict!("gamma", Arity::Exact(1), |n, a| gamma_checked(n, a[0])),
    // Apache's `Gamma.logGamma` — the Java arm — returns NaN for *every*
    // `x <= 0`, not just the poles: it is `log Γ(x)`, undefined wherever Γ is
    // negative. `libm::lgamma` is the different function `log |Γ(x)|`, which
    // happily returns 1.2655… for `loggamma(-0.5)`. Refusing the whole
    // non-positive half-line keeps the two in step (this port raises where Java
    // yields NaN, as elsewhere) instead of inventing a value Java never returns.
    strict!("loggamma", Arity::Exact(1), |n, a| {
        if a[0] <= 0.0 {
            return Err(domain(
                n,
                format_args!("argument must be > 0, got {}", a[0]),
            ));
        }
        Ok(libm::lgamma(a[0]))
    }),
    strict!("factorial", Arity::Exact(1), |n, a| {
        if a[0] <= -1.0 {
            return Err(domain(
                n,
                format_args!("argument must be > -1, got {}", a[0]),
            ));
        }
        gamma_checked(n, a[0] + 1.0)
    }),
    strict!("beta", Arity::Exact(2), |n, a| {
        if a[0] <= 0.0 || a[1] <= 0.0 {
            return Err(domain(
                n,
                format_args!("both arguments must be > 0, got ({}, {})", a[0], a[1]),
            ));
        }
        Ok(libm::exp(
            libm::lgamma(a[0]) + libm::lgamma(a[1]) - libm::lgamma(a[0] + a[1]),
        ))
    }),
    // ----- orthogonal polynomials (three-term recurrences) ------------------
    strict!("legendrep", Arity::Exact(2), |n, a| poly_order(n, a[0])
        .map(|k| legendre_p(k, a[1]))),
    strict!("chebyshevt", Arity::Exact(2), |n, a| poly_order(n, a[0])
        .map(|k| chebyshev_t(k, a[1]))),
    strict!("chebyshevu", Arity::Exact(2), |n, a| poly_order(n, a[0])
        .map(|k| chebyshev_u(k, a[1]))),
    strict!("hermiteh", Arity::Exact(2), |n, a| poly_order(n, a[0])
        .map(|k| hermite_h(k, a[1]))),
    strict!("laguerrel", Arity::Exact(2), |n, a| poly_order(n, a[0])
        .map(|k| laguerre_l(k, a[1]))),
    // ----- descriptive statistics -------------------------------------------
    strict!("mean", Arity::AtLeast(1), |_, a| Ok(mean(a))),
    strict!("median", Arity::AtLeast(1), |_, a| Ok(median(a))),
    strict!("variance", Arity::AtLeast(1), |_, a| Ok(sample_variance(a))),
    strict!("var", Arity::AtLeast(1), |_, a| Ok(sample_variance(a))),
    strict!("stdev", Arity::AtLeast(1), |_, a| Ok(libm::sqrt(
        sample_variance(a)
    ))),
    strict!("stddev", Arity::AtLeast(1), |_, a| Ok(libm::sqrt(
        sample_variance(a)
    ))),
    strict!("std", Arity::AtLeast(1), |_, a| Ok(libm::sqrt(
        sample_variance(a)
    ))),
    strict!("rms", Arity::AtLeast(1), |_, a| {
        Ok(libm::sqrt(
            a.iter().map(|x| x * x).sum::<f64>() / a.len() as f64,
        ))
    }),
    // Java's `average` yields 0.0 for an empty list (`.average().orElse(0.0)`).
    strict!("average", Arity::Any, |_, a| Ok(if a.is_empty() {
        0.0
    } else {
        mean(a)
    })),
    strict!("avg", Arity::Any, |_, a| Ok(if a.is_empty() {
        0.0
    } else {
        mean(a)
    })),
    strict!("percentile", Arity::AtLeast(2), |n, a| percentile(
        n,
        a[0],
        &a[1..]
    )),
    // ----- robust descriptive statistics (Phase 4.1) -------------------------
    strict!("skewness", Arity::AtLeast(3), |_, a| {
        crate::descriptive::skewness(a)
    }),
    strict!("kurtosis", Arity::AtLeast(4), |_, a| {
        crate::descriptive::kurtosis(a)
    }),
    strict!("mad", Arity::AtLeast(1), |_, a| {
        crate::descriptive::median_abs_deviation(a)
    }),
    // ----- Student-t distribution (Phase 4.1) --------------------------------
    strict!("tcdf", Arity::Exact(2), |_, a| student_t_cdf(a[0], a[1])),
    strict!("tpdf", Arity::Exact(2), |_, a| student_t_pdf(a[0], a[1])),
    strict!("tinv", Arity::Exact(2), |_, a| student_t_inv(a[0], a[1])),
    // ----- F distribution (Phase 4.1) ----------------------------------------
    strict!("fcdf", Arity::Exact(3), |_, a| f_dist_cdf(a[0], a[1], a[2])),
    strict!("fpdf", Arity::Exact(3), |_, a| f_dist_pdf(a[0], a[1], a[2])),
    strict!("finv", Arity::Exact(3), |_, a| f_dist_inv(a[0], a[1], a[2])),
    // ----- regularized incomplete beta (Phase 4.1) ---------------------------
    strict!("betainc", Arity::Exact(3), |_, a| regularized_beta(
        a[0], a[1], a[2]
    )),
    // ----- normal distribution (erf-based, matching Apache's formulas) -------
    strict!("normalcdf", Arity::Range(1, 3), |n, a| {
        let (mu, sigma) = normal_params(n, a)?;
        Ok(0.5 * libm::erfc(-(a[0] - mu) / (sigma * std::f64::consts::SQRT_2)))
    }),
    strict!("normalpdf", Arity::Range(1, 3), |n, a| {
        let (mu, sigma) = normal_params(n, a)?;
        let z = (a[0] - mu) / sigma;
        Ok(libm::exp(-0.5 * z * z) / (sigma * libm::sqrt(2.0 * std::f64::consts::PI)))
    }),
    strict!("probability", Arity::Exact(4), |n, a| {
        let (x1, x2, mu, sigma) = (a[0], a[1], a[2], a[3]);
        if sigma <= 0.0 {
            return Err(domain(
                n,
                format_args!("standard deviation must be > 0, got {sigma}"),
            ));
        }
        let d = sigma * std::f64::consts::SQRT_2;
        Ok(0.5 * (libm::erf((x2 - mu) / d) - libm::erf((x1 - mu) / d)))
    }),
    // ----- degenerate complex helpers (a real scalar is z with Im(z) = 0) ----
    strict!("real", Arity::Exact(1), |_, a| Ok(a[0])),
    strict!("imag", Arity::Exact(1), |_, _a| Ok(0.0)),
    strict!("conj", Arity::Exact(1), |_, a| Ok(a[0])),
    strict!("magnitude", Arity::Exact(1), |_, a| Ok(libm::fabs(a[0]))),
    strict!("angle", Arity::Exact(1), |_, a| Ok(libm::atan2(0.0, a[0]))),
    strict!("anglerad", Arity::Exact(1), |_, a| Ok(libm::atan2(
        0.0, a[0]
    ))),
    strict!("angledeg", Arity::Exact(1), |_, a| {
        Ok(libm::atan2(0.0, a[0]) * 180.0 / std::f64::consts::PI)
    }),
    strict!("cis", Arity::Exact(1), |_, a| Ok(libm::cos(a[0]))),
    // `pi()` is not a Java arm — the constant is spelled `pi#`. Kept as a
    // zero-argument convenience so both spellings resolve.
    strict!("pi", Arity::Exact(0), |_, _a| Ok(std::f64::consts::PI)),
    // ----- lazy forms --------------------------------------------------------
    lazy!("if", Arity::OneOf(&[3, 5]), eval_if),
    lazy!("sum", Arity::Any, |n, args, env| eval_reduction(
        n,
        args,
        env,
        0.0,
        |a, b| a + b
    )),
    lazy!("product", Arity::Any, |n, args, env| eval_reduction(
        n,
        args,
        env,
        1.0,
        |a, b| a * b
    )),
    lazy!("stringlen", Arity::Exact(1), |n, args, _| Ok(
        string_arg(n, &args[0])?.chars().count() as f64
    )),
    lazy!("stringval", Arity::Exact(1), |n, args, _| {
        let s = string_arg(n, &args[0])?;
        s.trim()
            .parse::<f64>()
            .map_err(|_| domain(n, format_args!("'{s}' is not a number")))
    }),
    lazy!("stringpos", Arity::Exact(2), |n, args, _| {
        let hay = string_arg(n, &args[0])?;
        let needle = string_arg(n, &args[1])?;
        // Java returns a 1-based index, 0 when absent.
        Ok(match hay.find(&needle) {
            Some(byte_index) => hay[..byte_index].chars().count() as f64 + 1.0,
            None => 0.0,
        })
    }),
    // ----- calculus (dispatch into the quadrature kernels) -------------------
    lazy!("integral", Arity::Range(4, 5), eval_integral_call),
    lazy!(
        "gaussintegral",
        Arity::Range(4, 5),
        eval_gauss_integral_call
    ),
    // ----- uncertainty propagation ------------------------------------------
    // Reads the `uncertaintyof$<var>` entry the uncertainty pass injects into
    // the scope; absent (no uncertainty solve ran) it is 0.0, as in Java.
    lazy!("uncertaintyof", Arity::Exact(1), |n, args, env| {
        let var = string_arg(n, &args[0])?.to_lowercase();
        Ok(env.get(&format!("uncertaintyof${var}")).unwrap_or(0.0))
    }),
    // ----- classic-solver TABLE lookup / interpolation -----------------------
    // The table name is the first (string) argument; remaining arguments are
    // lookup coordinates or 1-based column indices (`Evaluator.evalTableFunction`).
    lazy!("interpolate", Arity::Exact(2), |n, args, env| {
        let table = table_def(n, &args[0], env)?;
        let x = eval_in(&args[1], env)?;
        crate::curvetable::lookup(table, x, None)
    }),
    lazy!("interpolate1", Arity::Exact(2), |n, args, env| {
        let table = table_def(n, &args[0], env)?;
        let x = eval_in(&args[1], env)?;
        crate::curvetable::cubic_lookup(table, x)
    }),
    lazy!("interpolate2d", Arity::Exact(3), |n, args, env| {
        let table = table_def(n, &args[0], env)?;
        let x = eval_in(&args[1], env)?;
        let param = eval_in(&args[2], env)?;
        crate::curvetable::lookup(table, x, Some(param))
    }),
    lazy!("nlookuprows", Arity::Exact(1), |n, args, env| {
        let table = table_def(n, &args[0], env)?;
        Ok(crate::curvetable::row_count(table) as f64)
    }),
    lazy!("lookup", Arity::Exact(3), |n, args, env| {
        let table = table_def(n, &args[0], env)?;
        let row = java_int(eval_in(&args[1], env)?);
        let col = java_int(eval_in(&args[2], env)?);
        crate::curvetable::cell(table, row, col)
    }),
    lazy!("lookuprow", Arity::Exact(3), |n, args, env| {
        let table = table_def(n, &args[0], env)?;
        let col = java_int(eval_in(&args[1], env)?);
        let val = eval_in(&args[2], env)?;
        crate::curvetable::lookup_row(table, col, val)
    }),
    lazy!("differentiate", Arity::Exact(4), |n, args, env| {
        table_derivative(n, args, env, false)
    }),
    lazy!("differentiate1", Arity::Exact(4), |n, args, env| {
        table_derivative(n, args, env, true)
    }),
    // Derivative of the interpolant a bare `t(x)` call evaluates: the exact
    // segment slope (dtable) or the cubic-spline derivative (dtable1), first
    // y curve vs the x column — the 1-D map-call convention.
    lazy!("dtable", Arity::Exact(2), |n, args, env| {
        let table = table_def(n, &args[0], env)?;
        let x = eval_in(&args[1], env)?;
        crate::curvetable::differentiate(table, 2, 1, x, false)
    }),
    lazy!("dtable1", Arity::Exact(2), |n, args, env| {
        let table = table_def(n, &args[0], env)?;
        let x = eval_in(&args[1], env)?;
        crate::curvetable::differentiate(table, 2, 1, x, true)
    }),
    // ----- vector-argument kernels (statistics / 2-D interpolation) ----------
    // `Statistics.linFit` reached by its three `FunctionRegistry` names. The
    // Java registry documents `slope(xvals, yvals)` / `intercept` / `r2`
    // (`FunctionRegistry.java:236-238`) but `Evaluator.evalCall` never grew the
    // arms — only the `linfit$…` synthetic that `CALL LinFit` flattens to
    // reaches the kernel. This port wires the documented names to the very same
    // `lin_fit`, with the Java output mapping (`Evaluator.evalLinFit`:
    // slope → fit[0], intercept → fit[1], r2 → fit[2]). Deviation, documented:
    // it turns a Java "unknown function" into the documented value, never a
    // different value.
    lazy!("slope", Arity::Exact(2), eval_lin_fit_call),
    lazy!("intercept", Arity::Exact(2), eval_lin_fit_call),
    lazy!("r2", Arity::Exact(2), eval_lin_fit_call),
    // ----- Phase 4.1: correlation & descriptive (vector-argument) ------------
    lazy!("cov", Arity::Exact(2), |n, args, env| {
        let x = vector_arg(n, "x", &args[0], env)?;
        let y = vector_arg(n, "y", &args[1], env)?;
        crate::descriptive::sample_covariance(&x, &y)
    }),
    lazy!("pearson", Arity::Exact(2), |n, args, env| {
        let x = vector_arg(n, "x", &args[0], env)?;
        let y = vector_arg(n, "y", &args[1], env)?;
        crate::descriptive::pearson_correlation(&x, &y)
    }),
    lazy!("corrcoef", Arity::Exact(2), |n, args, env| {
        let x = vector_arg(n, "x", &args[0], env)?;
        let y = vector_arg(n, "y", &args[1], env)?;
        crate::descriptive::pearson_correlation(&x, &y)
    }),
    lazy!("spearman", Arity::Exact(2), |n, args, env| {
        let x = vector_arg(n, "x", &args[0], env)?;
        let y = vector_arg(n, "y", &args[1], env)?;
        crate::descriptive::spearman_correlation(&x, &y)
    }),
    lazy!("wmean", Arity::Exact(2), |n, args, env| {
        let w = vector_arg(n, "weights", &args[0], env)?;
        let x = vector_arg(n, "values", &args[1], env)?;
        crate::descriptive::weighted_mean(&w, &x)
    }),
    lazy!("wvar", Arity::Exact(2), |n, args, env| {
        let w = vector_arg(n, "weights", &args[0], env)?;
        let x = vector_arg(n, "values", &args[1], env)?;
        crate::descriptive::weighted_variance(&w, &x)
    }),
    lazy!("trimmedmean", Arity::Exact(2), |n, args, env| {
        let alpha = eval_in(&args[0], env)?;
        let x = vector_arg(n, "values", &args[1], env)?;
        crate::descriptive::trimmed_mean(alpha, &x)
    }),
    // ----- Phase 4.1: hypothesis tests (individual output accessors) ---------
    lazy!("ttest1_stat", Arity::Exact(2), eval_ttest1_call),
    lazy!("ttest1_pval", Arity::Exact(2), eval_ttest1_call),
    lazy!("ttest1_df", Arity::Exact(2), eval_ttest1_call),
    lazy!("ttest_paired_stat", Arity::Exact(2), eval_ttest_paired_call),
    lazy!("ttest_paired_pval", Arity::Exact(2), eval_ttest_paired_call),
    lazy!("ttest_paired_df", Arity::Exact(2), eval_ttest_paired_call),
    lazy!("ttest2_stat", Arity::Exact(2), eval_ttest2_call),
    lazy!("ttest2_pval", Arity::Exact(2), eval_ttest2_call),
    lazy!("ttest2_df", Arity::Exact(2), eval_ttest2_call),
    lazy!("anova1_f", Arity::AtLeast(2), eval_anova1_call),
    lazy!("anova1_pval", Arity::AtLeast(2), eval_anova1_call),
    lazy!("anova1_df_between", Arity::AtLeast(2), eval_anova1_call),
    lazy!("anova1_df_within", Arity::AtLeast(2), eval_anova1_call),
    lazy!("chi2gof_stat", Arity::Exact(2), eval_chi2gof_call),
    lazy!("chi2gof_pval", Arity::Exact(2), eval_chi2gof_call),
    lazy!("chi2gof_df", Arity::Exact(2), eval_chi2gof_call),
    lazy!("ci_mean_lo", Arity::Exact(2), eval_ci_mean_call),
    lazy!("ci_mean_hi", Arity::Exact(2), eval_ci_mean_call),
    lazy!("bootstrap_ci_lo", Arity::Exact(4), eval_bootstrap_ci_call),
    lazy!("bootstrap_ci_hi", Arity::Exact(4), eval_bootstrap_ci_call),
    lazy!("permtest_stat", Arity::Exact(4), eval_permtest_call),
    lazy!("permtest_pval", Arity::Exact(4), eval_permtest_call),
    lazy!("interp2", Arity::Exact(5), eval_interp2_call),
    // ----- dynamic array indexing -------------------------------------------
    // ArrayElmt(data[1:N], k): the range expands to N element args; the last
    // arg is the index. Lazy so only the selected element is evaluated.
    lazy!("arrayelmt", Arity::AtLeast(2), |n, args, env| {
        let count = args.len() - 1;
        let index = java_int(eval_in(&args[count], env)?);
        if index < 1 || index > count as i64 {
            return Err(domain(
                n,
                format_args!("index {index} is out of range 1..{count}"),
            ));
        }
        eval_in(&args[(index - 1) as usize], env)
    }),
    // ----- string/number conversion -----------------------------------------
    lazy!("baseconvert", Arity::Exact(3), eval_base_convert),
    // ----- seeded random numbers --------------------------------------------
    // Java seeds the seedless forms from object identity — irreproducible by
    // design, so this port requires the explicit third seed argument and then
    // reproduces `java.util.Random` bit for bit.
    strict!("random", Arity::Range(2, 3), |n, a| {
        let seed = random_seed(n, a)?;
        let r = JavaRandom::new(seed).next_double();
        Ok(a[0] + r * (a[1] - a[0]))
    }),
    strict!("randg", Arity::Range(2, 3), |n, a| {
        let seed = random_seed(n, a)?;
        let g = JavaRandom::new(seed).next_gaussian();
        Ok(a[0] + g * a[1])
    }),
    // ----- special functions (Apache Commons Math ports) ---------------------
    strict!("erfinv", Arity::Exact(1), |_, a| Ok(erf_inv(a[0]))),
    strict!("digamma", Arity::Exact(1), |_, a| Ok(digamma(a[0]))),
    strict!("normalinvcdf", Arity::Range(1, 3), |n, a| {
        let (mu, sigma) = normal_params(n, a)?;
        let p = a[0];
        // Java NormalDistribution.inverseCumulativeProbability: reject outside
        // [0, 1]; NaN slips through the comparison and yields NaN, as in Java.
        if !(0.0..=1.0).contains(&p) {
            return Err(domain(
                n,
                format_args!("probability must be in [0, 1], got {p}"),
            ));
        }
        Ok(mu + sigma * std::f64::consts::SQRT_2 * erf_inv(2.0 * p - 1.0))
    }),
    strict!("chi_square", Arity::Exact(2), |n, a| {
        let (x, df) = (a[0], a[1]);
        if x <= 0.0 {
            return Ok(0.0);
        }
        if df <= 0.0 {
            return Err(domain(
                n,
                format_args!("degrees of freedom must be > 0, got {df}"),
            ));
        }
        regularized_gamma_p(df / 2.0, x / 2.0)
    }),
    // ----- Bessel functions ---------------------------------------------------
    // The two-argument forms take (x, order) — the Java arms read the order
    // from args[1] — and the fixed-order forms are the Numerical-Recipes
    // rational approximations transcribed in `Evaluator.java`.
    strict!("besselj0", Arity::Exact(1), |_, a| Ok(bessj0(a[0]))),
    strict!("bessel_j0", Arity::Exact(1), |_, a| Ok(bessj0(a[0]))),
    strict!("besselj1", Arity::Exact(1), |_, a| Ok(bessj1(a[0]))),
    strict!("bessel_j1", Arity::Exact(1), |_, a| Ok(bessj1(a[0]))),
    strict!("besseli0", Arity::Exact(1), |_, a| Ok(bessi0(a[0]))),
    strict!("bessel_i0", Arity::Exact(1), |_, a| Ok(bessi0(a[0]))),
    strict!("besseli1", Arity::Exact(1), |_, a| Ok(bessi1(a[0]))),
    strict!("bessel_i1", Arity::Exact(1), |_, a| Ok(bessi1(a[0]))),
    strict!("bessely0", Arity::Exact(1), |n, a| bessy0(n, a[0])),
    strict!("bessel_y0", Arity::Exact(1), |n, a| bessy0(n, a[0])),
    strict!("bessely1", Arity::Exact(1), |n, a| bessy1(n, a[0])),
    strict!("bessel_y1", Arity::Exact(1), |n, a| bessy1(n, a[0])),
    strict!("besselk0", Arity::Exact(1), |n, a| bessk0(n, a[0])),
    strict!("bessel_k0", Arity::Exact(1), |n, a| bessk0(n, a[0])),
    strict!("besselk1", Arity::Exact(1), |n, a| bessk1(n, a[0])),
    strict!("bessel_k1", Arity::Exact(1), |n, a| bessk1(n, a[0])),
    strict!("besselj", Arity::Exact(2), |n, a| bessel_j(n, a[1], a[0])),
    strict!("bessel_j", Arity::Exact(2), |n, a| bessel_j(n, a[1], a[0])),
    strict!("bessely", Arity::Exact(2), |n, a| bessel_y(n, a[1], a[0])),
    strict!("bessel_y", Arity::Exact(2), |n, a| bessel_y(n, a[1], a[0])),
    strict!("besseli", Arity::Exact(2), |n, a| bessel_i(n, a[1], a[0])),
    strict!("bessel_i", Arity::Exact(2), |n, a| bessel_i(n, a[1], a[0])),
    strict!("besselk", Arity::Exact(2), |n, a| bessel_k(n, a[1], a[0])),
    strict!("bessel_k", Arity::Exact(2), |n, a| bessel_k(n, a[1], a[0])),
    // ----- compressible-flow stagnation properties ---------------------------
    strict!("stagnationtemp", Arity::Exact(3), |_, a| {
        Ok(a[0] + (a[1] * a[1]) / (2.0 * a[2]))
    }),
    strict!("stagnationpres", Arity::Exact(4), |_, a| {
        let (p, t, t0, k) = (a[0], a[1], a[2], a[3]);
        Ok(p * libm::pow(t0 / t, k / (k - 1.0)))
    }),
    // ----- radiation view factors (closed-form, Howell catalog) --------------
    strict!("viewfactor_perp", Arity::Exact(3), |n, a| {
        view_factor_perpendicular(n, a[0], a[1], a[2])
    }),
    strict!("viewfactor_plates", Arity::Exact(3), |n, a| {
        view_factor_parallel_plates(n, a[0], a[1], a[2])
    }),
    strict!("viewfactor_disks", Arity::Exact(3), |n, a| {
        view_factor_coaxial_disks(n, a[0], a[1], a[2])
    }),
    // ----- transient conduction (Heisler one-term approximation) -------------
    lazy!("heisler_temp", Arity::Exact(4), |n, args, env| {
        let geometry = heisler_geometry(&string_arg(n, &args[0])?)?;
        let bi = eval_in(&args[1], env)?;
        let fo = eval_in(&args[2], env)?;
        let x_star = eval_in(&args[3], env)?;
        Ok(heisler_temperature(geometry, bi, fo, x_star))
    }),
    lazy!("heisler_q", Arity::Exact(3), |n, args, env| {
        let geometry = heisler_geometry(&string_arg(n, &args[0])?)?;
        let bi = eval_in(&args[1], env)?;
        let fo = eval_in(&args[2], env)?;
        Ok(heisler_heat_ratio(geometry, bi, fo))
    }),
    // ----- ISA 1976 standard atmosphere --------------------------------------
    strict!("isa_t", Arity::Exact(1), |_, a| Ok(isa_temperature(a[0]))),
    strict!("isa_p", Arity::Exact(1), |_, a| Ok(isa_pressure(a[0]))),
    strict!("isa_rho", Arity::Exact(1), |_, a| {
        Ok(isa_pressure(a[0]) / (ISA_R_AIR * isa_temperature(a[0])))
    }),
    // ----- Wiebe heat release (engine combustion) ----------------------------
    strict!("wiebe", Arity::Exact(5), |n, a| wiebe(
        n, a[0], a[1], a[2], a[3], a[4], false
    )),
    strict!("wiebe_rate", Arity::Exact(5), |n, a| wiebe(
        n, a[0], a[1], a[2], a[3], a[4], true
    )),
    // ----- pneumatics (ISO 6358) ---------------------------------------------
    strict!("iso6358", Arity::Exact(5), |n, a| iso6358(
        n, a[0], a[1], a[2], a[3], a[4]
    )),
    // ----- flow networks (Darcy friction, Reynolds, minor losses) ------------
    strict!("friction_factor", Arity::Exact(2), |_, a| Ok(
        friction_factor(a[0], a[1])
    )),
    strict!("darcy_friction", Arity::Exact(2), |_, a| Ok(
        friction_factor(a[0], a[1])
    )),
    strict!("reynolds", Arity::Exact(4), |n, a| reynolds(
        n, a[0], a[1], a[2], a[3]
    )),
    strict!("re_number", Arity::Exact(4), |n, a| reynolds(
        n, a[0], a[1], a[2], a[3]
    )),
    strict!("minor_loss", Arity::Exact(3), |_, a| Ok(a[0]
        * 0.5
        * a[1]
        * a[2]
        * a[2])),
    // ----- convective heat transfer (Phase T3 correlations) ------------------
    strict!("nu_dittus_boelter", Arity::Exact(3), |n, a| {
        if a[0] <= 0.0 || a[1] <= 0.0 {
            return Err(domain(n, format_args!("Re and Pr must be > 0")));
        }
        Ok(0.023 * libm::pow(a[0], 0.8) * libm::pow(a[1], a[2]))
    }),
    strict!("nu_gnielinski", Arity::Exact(2), |n, a| {
        let (re, pr) = (a[0], a[1]);
        if re <= 0.0 || pr <= 0.0 {
            return Err(domain(n, format_args!("Re and Pr must be > 0")));
        }
        let f = libm::pow(0.790 * libm::log(re) - 1.64, -2.0);
        let num = (f / 8.0) * (re - 1000.0) * pr;
        let den = 1.0 + 12.7 * libm::sqrt(f / 8.0) * (libm::pow(pr, 2.0 / 3.0) - 1.0);
        Ok(num / den)
    }),
    strict!("chen_f", Arity::Exact(1), |n, a| {
        if a[0] <= 0.0 {
            return Err(domain(
                n,
                format_args!("Martinelli parameter X_tt must be > 0"),
            ));
        }
        let inv = 1.0 / a[0];
        Ok(if inv <= 0.1 {
            1.0
        } else {
            2.35 * libm::pow(inv + 0.213, 0.736)
        })
    }),
    strict!("chen_s", Arity::Exact(2), |n, a| {
        if a[0] <= 0.0 || a[1] <= 0.0 {
            return Err(domain(n, format_args!("Re_l and F must be > 0")));
        }
        let re_tp = a[0] * libm::pow(a[1], 1.25);
        Ok(1.0 / (1.0 + 2.53e-6 * libm::pow(re_tp, 1.17)))
    }),
    strict!("nu_shah", Arity::Exact(4), |n, a| {
        let (re_l, pr_l, x, p_red) = (a[0], a[1], a[2], a[3]);
        if re_l <= 0.0 || pr_l <= 0.0 {
            return Err(domain(n, format_args!("Re_l and Pr_l must be > 0")));
        }
        if p_red <= 0.0 || p_red >= 1.0 {
            return Err(domain(n, format_args!("reduced pressure must be in (0,1)")));
        }
        let xx = clip(x, 1e-6, 1.0 - 1e-6);
        let nu_l = 0.023 * libm::pow(re_l, 0.8) * libm::pow(pr_l, 0.4);
        Ok(nu_l
            * (libm::pow(1.0 - xx, 0.8)
                + 3.8 * libm::pow(xx, 0.76) * libm::pow(1.0 - xx, 0.04) / libm::pow(p_red, 0.38)))
    }),
    strict!("nu_cavallini_zecchin", Arity::Exact(5), |n, a| {
        let (re_l, pr_l, x, rho_l, rho_g) = (a[0], a[1], a[2], a[3], a[4]);
        if re_l <= 0.0 || pr_l <= 0.0 || rho_l <= 0.0 || rho_g <= 0.0 {
            return Err(domain(
                n,
                format_args!("Re_l, Pr_l and densities must be > 0"),
            ));
        }
        let xx = clip(x, 1e-6, 1.0 - 1e-6);
        let re_eq = re_l * ((1.0 - xx) + xx * libm::sqrt(rho_l / rho_g));
        Ok(0.05 * libm::pow(re_eq, 0.8) * libm::pow(pr_l, 0.33))
    }),
    strict!("zone_ramp", Arity::Exact(2), |n, a| {
        if a[1] <= 0.0 {
            return Err(domain(n, format_args!("smoothing width must be > 0")));
        }
        Ok(libm::tanh(java_max(0.0, a[0]) / a[1]))
    }),
    // ----- two-phase flow (Lockhart–Martinelli / Chisholm, void fractions) ---
    strict!("lm_phi2", Arity::Exact(2), |n, a| {
        if a[0] <= 0.0 {
            return Err(domain(
                n,
                format_args!("Martinelli parameter X must be > 0"),
            ));
        }
        Ok(1.0 + a[1] / a[0] + 1.0 / (a[0] * a[0]))
    }),
    strict!("lm_martinelli_tt", Arity::Exact(5), |n, a| {
        let (quality, rho_l, rho_g, mu_l, mu_g) = (a[0], a[1], a[2], a[3], a[4]);
        if quality <= 0.0 || quality >= 1.0 {
            return Err(domain(n, format_args!("quality x must be in (0, 1)")));
        }
        if rho_l <= 0.0 || rho_g <= 0.0 || mu_l <= 0.0 || mu_g <= 0.0 {
            return Err(domain(
                n,
                format_args!("densities and viscosities must be > 0"),
            ));
        }
        Ok(libm::pow((1.0 - quality) / quality, 0.9)
            * libm::pow(rho_g / rho_l, 0.5)
            * libm::pow(mu_l / mu_g, 0.1))
    }),
    strict!("void_homogeneous", Arity::Exact(3), |n, a| {
        void_fraction(n, a[0], a[1], a[2], VoidModel::Homogeneous)
    }),
    strict!("void_zivi", Arity::Exact(3), |n, a| {
        void_fraction(n, a[0], a[1], a[2], VoidModel::Zivi)
    }),
    strict!("void_rouhani", Arity::Exact(5), |n, a| void_rouhani(
        n, a[0], a[1], a[2], a[3], a[4]
    )),
    strict!("friedel_phi2", Arity::Exact(8), |n, a| friedel_phi2(
        n, a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7]
    )),
    strict!("momentum_flux", Arity::Exact(5), |n, a| {
        let (x, rho_l, rho_g, alpha, g) = (a[0], a[1], a[2], a[3], a[4]);
        two_phase_densities(n, rho_l, rho_g)?;
        let clamped = clip(alpha, 1e-9, 1.0 - 1e-9);
        Ok(g * g * (x * x / (rho_g * clamped) + (1.0 - x) * (1.0 - x) / (rho_l * (1.0 - clamped))))
    }),
    // ----- heat exchangers (effectiveness-NTU, LMTD, fins) -------------------
    lazy!("hx_effectiveness", Arity::Exact(3), |n, args, env| {
        let arrangement = hx_arrangement(n, &string_arg(n, &args[0])?)?;
        let ntu = eval_in(&args[1], env)?;
        let cr = eval_in(&args[2], env)?;
        hx_effectiveness(n, arrangement, ntu, cr)
    }),
    lazy!("hx_epsilon", Arity::Exact(3), |n, args, env| {
        let arrangement = hx_arrangement(n, &string_arg(n, &args[0])?)?;
        let ntu = eval_in(&args[1], env)?;
        let cr = eval_in(&args[2], env)?;
        hx_effectiveness(n, arrangement, ntu, cr)
    }),
    lazy!("hx_ntu", Arity::Exact(3), |n, args, env| {
        let arrangement = hx_arrangement(n, &string_arg(n, &args[0])?)?;
        let eps = eval_in(&args[1], env)?;
        let cr = eval_in(&args[2], env)?;
        hx_ntu(n, arrangement, eps, cr)
    }),
    strict!("lmtd", Arity::Exact(2), |n, a| {
        let (dt1, dt2) = (a[0], a[1]);
        if dt1 <= 0.0 || dt2 <= 0.0 {
            return Err(domain(
                n,
                format_args!(
                    "LMTD terminal differences must be positive (a temperature cross or \
                     pinch gives a non-physical LMTD); got {dt1}, {dt2}"
                ),
            ));
        }
        if libm::fabs(dt1 - dt2) < 1e-12 * java_max(dt1, dt2) {
            return Ok(0.5 * (dt1 + dt2));
        }
        Ok((dt1 - dt2) / libm::log(dt1 / dt2))
    }),
    strict!("fin_efficiency", Arity::Exact(1), |n, a| {
        let ml = a[0];
        if ml < 0.0 {
            return Err(domain(
                n,
                format_args!("fin parameter mL must be >= 0, got {ml}"),
            ));
        }
        Ok(if ml < 1e-8 { 1.0 } else { libm::tanh(ml) / ml })
    }),
    // ----- HX sizing correlations (the CoolProp-free subset) -----------------
    strict!("ua_hx", Arity::Exact(5), |n, a| {
        let (h1, a1, h2, a2, r_wall) = (a[0], a[1], a[2], a[3], a[4]);
        if h1 <= 0.0 || a1 <= 0.0 || h2 <= 0.0 || a2 <= 0.0 {
            return Err(domain(
                n,
                format_args!("film coefficients and areas must be positive"),
            ));
        }
        Ok(1.0 / (1.0 / (h1 * a1) + r_wall + 1.0 / (h2 * a2)))
    }),
    strict!("nu_zukauskas", Arity::Exact(2), |_, a| {
        Ok(0.27 * libm::pow(java_max(a[0], 1.0), 0.63) * libm::pow(a[1], 0.36))
    }),
    strict!("nu_colburn", Arity::Exact(3), |_, a| {
        Ok(a[0] * a[1] * libm::pow(a[2], 1.0 / 3.0))
    }),
    strict!("nu_churchill_chu", Arity::Exact(2), |_, a| {
        let (ra, pr) = (a[0], a[1]);
        let d = libm::pow(1.0 + libm::pow(0.492 / pr, 9.0 / 16.0), 8.0 / 27.0);
        let term = 0.825 + 0.387 * libm::pow(java_max(ra, 0.0), 1.0 / 6.0) / d;
        Ok(term * term)
    }),
    strict!("nu_blend", Arity::Exact(2), |_, a| {
        Ok(libm::cbrt(a[0] * a[0] * a[0] + a[1] * a[1] * a[1]))
    }),
    strict!("hx_dh", Arity::Exact(3), |n, a| {
        if !(a[1] > 0.0) {
            return Err(domain(n, format_args!("total area must be > 0")));
        }
        Ok(4.0 * a[0] * a[2] / a[1])
    }),
    strict!("hx_aconv", Arity::Exact(3), |n, a| {
        if !(a[2] > 0.0) {
            return Err(domain(n, format_args!("D_h must be > 0")));
        }
        Ok(4.0 * a[0] * a[1] / a[2])
    }),
    strict!("hx_sigma", Arity::Exact(2), |n, a| {
        if !(a[1] > 0.0) {
            return Err(domain(n, format_args!("frontal area must be > 0")));
        }
        Ok(a[0] / a[1])
    }),
    strict!("hx_eta_surf", Arity::Exact(3), |n, a| {
        if !(a[1] > 0.0) {
            return Err(domain(n, format_args!("total area must be > 0")));
        }
        Ok(1.0 - (a[0] / a[1]) * (1.0 - a[2]))
    }),
    lazy!("nu_tubebank", Arity::Exact(3), |n, args, env| {
        let arrangement = string_arg(n, &args[0])?;
        let re = eval_in(&args[1], env)?;
        let pr = eval_in(&args[2], env)?;
        Ok(nu_tube_bank(&arrangement, re, pr))
    }),
    strict!("nu_hilpert", Arity::Exact(2), |_, a| Ok(nu_hilpert(
        a[0], a[1]
    ))),
    strict!("nu_plate", Arity::Exact(3), |_, a| {
        let b = clip(a[2], 30.0, 60.0);
        let f = (b - 30.0) / 30.0;
        let c = 0.2 + 0.2 * f;
        let m = 0.6 + 0.14 * f;
        Ok(c * libm::pow(java_max(a[0], 1.0), m) * libm::pow(a[1], 1.0 / 3.0))
    }),
    strict!("hx_fin_len", Arity::Exact(4), |_, a| {
        let (depth, t, fin_density, h_tube) = (a[0], a[1], a[2], a[3]);
        let fin_a = h_tube - 2.0 * t;
        let fin_b = 1.0 / (2.0 * fin_density);
        Ok(2.0 * (depth - 2.0 * t) * fin_density * libm::sqrt(fin_a * fin_a + fin_b * fin_b))
    }),
    strict!("hx_area_direct", Arity::Exact(5), |_, a| {
        let (w, tube_count, h_tube, depth, t) = (a[0], a[1], a[2], a[3], a[4]);
        Ok(2.0 * w * tube_count * ((h_tube - 2.0 * t) + (depth - 2.0 * t)))
    }),
    strict!("hx_area_indirect", Arity::Exact(3), |_, a| {
        Ok(2.0 * a[0] * a[1] * a[2])
    }),
    strict!("dp_gravity", Arity::Exact(5), |_, a| {
        let (rho_l, rho_g, alpha, l, theta_deg) = (a[0], a[1], a[2], a[3], a[4]);
        let rho_mix = alpha * rho_g + (1.0 - alpha) * rho_l;
        Ok(rho_mix * 9.80665 * l * libm::sin(theta_deg * std::f64::consts::PI / 180.0))
    }),
    strict!("dp_compact_core", Arity::Exact(9), |n, a| {
        let (g, rho_in, rho_out, rho_mean) = (a[0], a[1], a[2], a[3]);
        let (sigma, fanning, a_over_ac, kc, ke) = (a[4], a[5], a[6], a[7], a[8]);
        if !(rho_in > 0.0) || !(rho_out > 0.0) || !(rho_mean > 0.0) {
            return Err(domain(n, format_args!("densities must be > 0")));
        }
        let s2 = sigma * sigma;
        Ok((g * g / (2.0 * rho_in))
            * ((kc + 1.0 - s2)
                + 2.0 * (rho_in / rho_out - 1.0)
                + fanning * a_over_ac * (rho_in / rho_mean)
                - (1.0 - s2 - ke) * (rho_in / rho_out)))
    }),
    strict!("mass_flux", Arity::Exact(2), |n, a| {
        if !(a[1] > 0.0) {
            return Err(domain(n, format_args!("A_flow must be > 0")));
        }
        Ok(a[0] / a[1])
    }),
    lazy!("j_fin", Arity::Exact(2), |n, args, env| {
        let surface = string_arg(n, &args[0])?;
        let re = eval_in(&args[1], env)?;
        let r = java_max(re, 1.0);
        Ok(match fin_surface(&surface) {
            "wavy" => 0.130 * libm::pow(r, -0.40),
            "louvered" => 0.174 * libm::pow(r, -0.40),
            "offset" => 0.300 * libm::pow(r, -0.40),
            _ => 0.080 * libm::pow(r, -0.40),
        })
    }),
    lazy!("f_fin", Arity::Exact(2), |n, args, env| {
        let surface = string_arg(n, &args[0])?;
        let re = eval_in(&args[1], env)?;
        let r = java_max(re, 1.0);
        Ok(match fin_surface(&surface) {
            "wavy" => 0.280 * libm::pow(r, -0.30),
            "louvered" => 0.420 * libm::pow(r, -0.30),
            "offset" => 0.560 * libm::pow(r, -0.30),
            _ => 0.150 * libm::pow(r, -0.30),
        })
    }),
    strict!("nu_gungor_winterton", Arity::Exact(3), |_, a| {
        let (nu_l, xtt, bo) = (a[0], a[1], a[2]);
        let e = 1.0
            + 24000.0 * libm::pow(java_max(bo, 0.0), 1.16)
            + 1.37 * libm::pow(1.0 / java_max(xtt, 1e-6), 0.86);
        Ok(nu_l * e)
    }),
    strict!("nu_traviss", Arity::Exact(3), |_, a| {
        let (re_l, pr_l, xtt) = (a[0], a[1], a[2]);
        let r = java_max(re_l, 1.0);
        let mut ft = 5.0 * pr_l
            + 5.0 * libm::log(1.0 + 5.0 * pr_l)
            + 2.5 * libm::log(0.00313 * libm::pow(r, 0.812));
        ft = java_max(ft, 1e-3);
        let x = java_max(xtt, 1e-6);
        Ok(0.15 * pr_l * libm::pow(r, 0.9) / ft * (1.0 / x + 2.85 / libm::pow(x, 0.476)))
    }),
    // ----- ideal-gas compressible flow (M, k closed forms) -------------------
    strict!("t0_t", Arity::Exact(2), |n, a| cf_t0_over_t(n, a[0], a[1])),
    strict!("isen_t0_t", Arity::Exact(2), |n, a| cf_t0_over_t(
        n, a[0], a[1]
    )),
    strict!("p0_p", Arity::Exact(2), |n, a| cf_p0_over_p(n, a[0], a[1])),
    strict!("isen_p0_p", Arity::Exact(2), |n, a| cf_p0_over_p(
        n, a[0], a[1]
    )),
    strict!("rho0_rho", Arity::Exact(2), |n, a| cf_rho0_over_rho(
        n, a[0], a[1]
    )),
    strict!("isen_rho0_rho", Arity::Exact(2), |n, a| cf_rho0_over_rho(
        n, a[0], a[1]
    )),
    strict!("a_astar", Arity::Exact(2), |n, a| cf_a_over_astar(
        n, a[0], a[1]
    )),
    strict!("isen_a_astar", Arity::Exact(2), |n, a| cf_a_over_astar(
        n, a[0], a[1]
    )),
    lazy!("mach_a_astar", Arity::Exact(3), |n, args, env| {
        let ratio = eval_in(&args[0], env)?;
        let k = eval_in(&args[1], env)?;
        let regime = string_arg(n, &args[2])?;
        cf_mach_from_area_ratio(n, ratio, k, &regime)
    }),
    strict!("m2_shock", Arity::Exact(2), |n, a| cf_mach_behind_shock(
        n, a[0], a[1]
    )),
    strict!("mach_shock", Arity::Exact(2), |n, a| cf_mach_behind_shock(
        n, a[0], a[1]
    )),
    strict!("p2_p1_shock", Arity::Exact(2), |n, a| {
        cf_require_supersonic(n, "normal shock", a[0])?;
        cf_require_k(n, a[1])?;
        Ok((2.0 * a[1] * a[0] * a[0] - (a[1] - 1.0)) / (a[1] + 1.0))
    }),
    strict!("t2_t1_shock", Arity::Exact(2), |n, a| {
        cf_require_supersonic(n, "normal shock", a[0])?;
        cf_require_k(n, a[1])?;
        let (m1s, k) = (a[0] * a[0], a[1]);
        Ok((2.0 + (k - 1.0) * m1s) * (2.0 * k * m1s - (k - 1.0)) / ((k + 1.0) * (k + 1.0) * m1s))
    }),
    strict!("rho2_rho1_shock", Arity::Exact(2), |n, a| {
        cf_require_supersonic(n, "normal shock", a[0])?;
        cf_require_k(n, a[1])?;
        let (m1s, k) = (a[0] * a[0], a[1]);
        Ok((k + 1.0) * m1s / (2.0 + (k - 1.0) * m1s))
    }),
    strict!("p02_p01_shock", Arity::Exact(2), |n, a| {
        cf_require_supersonic(n, "normal shock", a[0])?;
        cf_require_k(n, a[1])?;
        let (m1s, k) = (a[0] * a[0], a[1]);
        let ra = (k + 1.0) * m1s / (2.0 + (k - 1.0) * m1s);
        let rb = (k + 1.0) / (2.0 * k * m1s - (k - 1.0));
        Ok(libm::pow(ra, k / (k - 1.0)) * libm::pow(rb, 1.0 / (k - 1.0)))
    }),
    strict!("rayleigh_t0_t0star", Arity::Exact(2), |n, a| {
        cf_require_mach(n, a[0])?;
        cf_require_k(n, a[1])?;
        let (m2, k) = (a[0] * a[0], a[1]);
        let denom = 1.0 + k * m2;
        Ok((k + 1.0) * m2 * (2.0 + (k - 1.0) * m2) / (denom * denom))
    }),
    strict!("rayleigh_t_tstar", Arity::Exact(2), |n, a| {
        cf_require_mach(n, a[0])?;
        cf_require_k(n, a[1])?;
        let r = a[0] * (1.0 + a[1]) / (1.0 + a[1] * a[0] * a[0]);
        Ok(r * r)
    }),
    strict!("rayleigh_p_pstar", Arity::Exact(2), |n, a| {
        cf_require_mach(n, a[0])?;
        cf_require_k(n, a[1])?;
        Ok((1.0 + a[1]) / (1.0 + a[1] * a[0] * a[0]))
    }),
    strict!("rayleigh_p0_p0star", Arity::Exact(2), |n, a| {
        cf_require_mach(n, a[0])?;
        cf_require_k(n, a[1])?;
        let (m2, k) = (a[0] * a[0], a[1]);
        let base = (2.0 + (k - 1.0) * m2) / (k + 1.0);
        Ok(((1.0 + k) / (1.0 + k * m2)) * libm::pow(base, k / (k - 1.0)))
    }),
    strict!("fanno_t_tstar", Arity::Exact(2), |n, a| {
        cf_require_mach(n, a[0])?;
        cf_require_k(n, a[1])?;
        Ok((a[1] + 1.0) / (2.0 + (a[1] - 1.0) * a[0] * a[0]))
    }),
    strict!("fanno_p_pstar", Arity::Exact(2), |n, a| {
        cf_require_mach(n, a[0])?;
        cf_require_k(n, a[1])?;
        Ok((1.0 / a[0]) * libm::sqrt((a[1] + 1.0) / (2.0 + (a[1] - 1.0) * a[0] * a[0])))
    }),
    strict!("fanno_p0_p0star", Arity::Exact(2), |n, a| {
        cf_require_mach(n, a[0])?;
        cf_require_k(n, a[1])?;
        let (m2, k) = (a[0] * a[0], a[1]);
        let base = (2.0 + (k - 1.0) * m2) / (k + 1.0);
        Ok((1.0 / a[0]) * libm::pow(base, (k + 1.0) / (2.0 * (k - 1.0))))
    }),
    strict!("fanno_fld", Arity::Exact(2), |n, a| {
        cf_require_mach(n, a[0])?;
        cf_require_k(n, a[1])?;
        let (m2, k) = (a[0] * a[0], a[1]);
        Ok((1.0 - m2) / (k * m2)
            + (k + 1.0) / (2.0 * k) * libm::log((k + 1.0) * m2 / (2.0 + (k - 1.0) * m2)))
    }),
    strict!("prandtlmeyer", Arity::Exact(2), |n, a| cf_prandtl_meyer(
        n, a[0], a[1]
    )),
    strict!("prandtl_meyer", Arity::Exact(2), |n, a| cf_prandtl_meyer(
        n, a[0], a[1]
    )),
    strict!("mach_prandtlmeyer", Arity::Exact(2), |n, a| {
        cf_mach_from_prandtl_meyer(n, a[0], a[1])
    }),
    strict!("machangle", Arity::Exact(1), |n, a| {
        cf_require_supersonic(n, "Mach angle", a[0])?;
        Ok(libm::asin(1.0 / a[0]))
    }),
    strict!("theta_oblique", Arity::Exact(3), |n, a| cf_theta_oblique(
        n, a[0], a[1], a[2]
    )),
    lazy!("beta_oblique", Arity::Exact(4), |n, args, env| {
        let m1 = eval_in(&args[0], env)?;
        let theta = eval_in(&args[1], env)?;
        let k = eval_in(&args[2], env)?;
        let branch = string_arg(n, &args[3])?;
        cf_beta_oblique(n, m1, theta, k, &branch)
    }),
    // ----- Phase 5: cubic equation of state (SRK / PR) --------------------
    // `Evaluator.evalCall`: eos_*(fluid$, model$, T, P, phase$), except
    // eos_pressure(fluid$, model$, T, v) and eos_psat(fluid$, model$, T).
    // Independent of CoolProp — pure `props::cubiceos`.
    lazy!("eos_z", Arity::Exact(5), |n, args, env| eos_tp(
        n,
        args,
        env,
        crate::props::cubiceos::z
    )),
    lazy!("eos_volume", Arity::Exact(5), |n, args, env| eos_tp(
        n,
        args,
        env,
        crate::props::cubiceos::volume
    )),
    lazy!("eos_density", Arity::Exact(5), |n, args, env| eos_tp(
        n,
        args,
        env,
        crate::props::cubiceos::density
    )),
    lazy!("eos_enthalpy", Arity::Exact(5), |n, args, env| eos_tp(
        n,
        args,
        env,
        crate::props::cubiceos::enthalpy
    )),
    lazy!("eos_entropy", Arity::Exact(5), |n, args, env| eos_tp(
        n,
        args,
        env,
        crate::props::cubiceos::entropy
    )),
    lazy!("eos_pressure", Arity::Exact(4), |n, args, env| {
        let fluid = string_arg(n, &args[0])?;
        let model = string_arg(n, &args[1])?;
        let t = eval_in(&args[2], env)?;
        let v = eval_in(&args[3], env)?;
        crate::props::cubiceos::pressure(&fluid, &model, t, v)
    }),
    lazy!("eos_psat", Arity::Exact(3), |n, args, env| {
        let fluid = string_arg(n, &args[0])?;
        let model = string_arg(n, &args[1])?;
        let t = eval_in(&args[2], env)?;
        crate::props::cubiceos::saturation_pressure(&fluid, &model, t)
    }),
    // ----- Phase 5: combustion thermochemistry (NASA-7 / IdealGas) --------
    lazy!("adiabaticflametemp", Arity::Exact(3), flame_temp),
    lazy!("adiabaticflametemperature", Arity::Exact(3), flame_temp),
    lazy!("flametemp", Arity::Exact(3), flame_temp),
    // Ideal-gas mixture properties from a 'species:amount, ...' string.
    lazy!("mix_mw", Arity::Exact(1), |n, args, _env| {
        crate::props::thermochem::mixture_molar_mass(&string_arg(n, &args[0])?)
    }),
    lazy!("mix_molarmass", Arity::Exact(1), |n, args, _env| {
        crate::props::thermochem::mixture_molar_mass(&string_arg(n, &args[0])?)
    }),
    lazy!("mix_cp", Arity::Exact(2), |n, args, env| {
        let comp = string_arg(n, &args[0])?;
        crate::props::thermochem::mixture_cp(&comp, eval_in(&args[1], env)?)
    }),
    lazy!("mix_enthalpy", Arity::Exact(2), |n, args, env| {
        let comp = string_arg(n, &args[0])?;
        crate::props::thermochem::mixture_enthalpy(&comp, eval_in(&args[1], env)?)
    }),
    lazy!("mix_entropy", Arity::Exact(3), |n, args, env| {
        let comp = string_arg(n, &args[0])?;
        let t = eval_in(&args[1], env)?;
        let p = eval_in(&args[2], env)?;
        crate::props::thermochem::mixture_entropy(&comp, t, p)
    }),
    lazy!("mix_viscosity", Arity::Exact(2), |n, args, env| {
        let comp = string_arg(n, &args[0])?;
        crate::props::transport::mixture_viscosity(&comp, eval_in(&args[1], env)?)
    }),
    lazy!("mix_conductivity", Arity::Exact(2), |n, args, env| {
        let comp = string_arg(n, &args[0])?;
        crate::props::transport::mixture_conductivity(&comp, eval_in(&args[1], env)?)
    }),
    // Combustion-product chemical equilibrium (dissociation).
    lazy!("eq_molefraction", Arity::Exact(5), |n, args, env| {
        let fuel = string_arg(n, &args[0])?;
        let phi = eval_in(&args[1], env)?;
        let t = eval_in(&args[2], env)?;
        let p = eval_in(&args[3], env)?;
        let species = string_arg(n, &args[4])?;
        crate::props::equilibrium::mole_fraction(&fuel, phi, t, p, &species)
    }),
    lazy!("adiabaticflametempeq", Arity::Exact(4), flame_temp_eq),
    lazy!("flametemp_eq", Arity::Exact(4), flame_temp_eq),
    // ----- Phase 5: HX correlations that query the fluid backend ----------
    // `HxCorrelations.*`, whose CoolProp calls resolve through
    // `props::propfun::InstalledFluids` — same aliases, same backend, same
    // honest refusal when no backend is installed.
    lazy!("htc_1phase", Arity::Exact(6), |n, args, env| {
        let (fluid, a) = fluid_and_values::<5>(n, args, env)?;
        crate::props::hxcorr::htc_1phase(&FLUID_BACKEND, &fluid, a[0], a[1], a[2], a[3], a[4])
    }),
    lazy!("htc_evap", Arity::Exact(6), |n, args, env| {
        let (fluid, a) = fluid_and_values::<5>(n, args, env)?;
        crate::props::hxcorr::htc_evap(&FLUID_BACKEND, &fluid, a[0], a[1], a[2], a[3], a[4])
    }),
    lazy!("htc_cond", Arity::Exact(6), |n, args, env| {
        let (fluid, a) = fluid_and_values::<5>(n, args, env)?;
        crate::props::hxcorr::htc_cond(&FLUID_BACKEND, &fluid, a[0], a[1], a[2], a[3], a[4])
    }),
    lazy!("htc_extair", Arity::Exact(6), |n, args, env| {
        let (fluid, a) = fluid_and_values::<5>(n, args, env)?;
        crate::props::hxcorr::htc_ext_air(&FLUID_BACKEND, &fluid, a[0], a[1], a[2], a[3], a[4])
    }),
    lazy!("dp_1phase", Arity::Exact(7), |n, args, env| {
        let (fluid, a) = fluid_and_values::<6>(n, args, env)?;
        crate::props::hxcorr::dp_1phase(&FLUID_BACKEND, &fluid, a[0], a[1], a[2], a[3], a[4], a[5])
    }),
    lazy!("dp_2phase", Arity::Exact(7), |n, args, env| {
        let (fluid, a) = fluid_and_values::<6>(n, args, env)?;
        crate::props::hxcorr::dp_2phase(&FLUID_BACKEND, &fluid, a[0], a[1], a[2], a[3], a[4], a[5])
    }),
    lazy!("dp_mueller_steinhagen", Arity::Exact(7), |n, args, env| {
        let (fluid, a) = fluid_and_values::<6>(n, args, env)?;
        crate::props::hxcorr::dp_mueller_steinhagen(
            &FLUID_BACKEND,
            &fluid,
            a[0],
            a[1],
            a[2],
            a[3],
            a[4],
            a[5],
        )
    }),
    lazy!("dp_ms", Arity::Exact(7), |n, args, env| {
        let (fluid, a) = fluid_and_values::<6>(n, args, env)?;
        crate::props::hxcorr::dp_mueller_steinhagen(
            &FLUID_BACKEND,
            &fluid,
            a[0],
            a[1],
            a[2],
            a[3],
            a[4],
            a[5],
        )
    }),
    lazy!("dp_2phase_avg", Arity::Exact(9), |n, args, env| {
        let (fluid, a) = fluid_and_values::<8>(n, args, env)?;
        crate::props::hxcorr::dp_2phase_avg(
            &FLUID_BACKEND,
            &fluid,
            a[0],
            a[1],
            a[2],
            a[3],
            a[4],
            a[5],
            a[6],
            a[7],
        )
    }),
    // ----- ODE Table accessors (Phase 7) ---------------------------------
    //
    // `Evaluator.evalCall`'s ten-name arm:
    //
    //     String column = evalString(args.get(0));
    //     Double arg    = args.size() > 1 ? eval(args.get(1), …) : null;
    //     yield DynamicAccessorContext.resolve(c.function(), column, arg, values);
    //
    // Every one takes the column name lazily (it is a *string*, and `ODEValue`
    // / `TimeAt` take one further numeric argument), so all ten are `lazy!`.
    // The arity is `Range(1, 2)` across the board because that is what the
    // Java's `args.size() > 1` test admits — an aggregate called with a stray
    // second argument silently ignores it there, and does here too.
    //
    // With no `DYNAMIC` block in the document the context is `None` and
    // `accessors::resolve` yields `0.0` rather than an error: the Java's
    // null-thread-local answer, and the reason an accessor is harmless in a
    // steady document.
    lazy!("odevalue", Arity::Range(1, 2), ode_accessor),
    lazy!("finalvalue", Arity::Range(1, 2), ode_accessor),
    lazy!("maxvalue", Arity::Range(1, 2), ode_accessor),
    lazy!("minvalue", Arity::Range(1, 2), ode_accessor),
    lazy!("timeat", Arity::Range(1, 2), ode_accessor),
    lazy!("odeavg", Arity::Range(1, 2), ode_accessor),
    lazy!("odesum", Arity::Range(1, 2), ode_accessor),
    lazy!("odestddev", Arity::Range(1, 2), ode_accessor),
    lazy!("odemin", Arity::Range(1, 2), ode_accessor),
    lazy!("odemax", Arity::Range(1, 2), ode_accessor),
    // ----- parametric-table accessors (Phase 8) --------------------------
    //
    // `Evaluator.evalParametricAccessor`. `TableRun#`/`NParametricRuns` take
    // no arguments; the five aggregates take a column-name string;
    // `TableValue` takes two *rounded* indices and `IntegralValue` two column
    // names. Each delegates to `crate::analysis::parametric`, whose free
    // functions already carry the Java's null-context defaults.
    lazy!("tablerun#", Arity::Any, |_, _, env| Ok(
        crate::analysis::parametric::current_run(env.ctx().parametric)
    )),
    lazy!("tablerun", Arity::Any, |_, _, env| Ok(
        crate::analysis::parametric::current_run(env.ctx().parametric)
    )),
    lazy!("nparametricruns", Arity::Any, |_, _, env| Ok(
        crate::analysis::parametric::run_count(env.ctx().parametric)
    )),
    lazy!("tablevalue", Arity::Exact(2), |n, args, env| {
        // `(int) Math.round(...)` on both indices — `reduction_bound` is the
        // same cast with the silent `int` truncation turned into an error.
        let run = reduction_bound(n, "run", eval_in(&args[0], env)?)?;
        let col = reduction_bound(n, "column", eval_in(&args[1], env)?)?;
        crate::analysis::parametric::cell(env.ctx().parametric, run, col)
    }),
    lazy!("tablesum", Arity::Exact(1), |n, args, env| table_aggregate(
        "sum", n, args, env
    )),
    lazy!("tableavg", Arity::Exact(1), |n, args, env| table_aggregate(
        "avg", n, args, env
    )),
    lazy!("tablemin", Arity::Exact(1), |n, args, env| table_aggregate(
        "min", n, args, env
    )),
    lazy!("tablemax", Arity::Exact(1), |n, args, env| table_aggregate(
        "max", n, args, env
    )),
    lazy!("tablestddev", Arity::Exact(1), |n, args, env| {
        table_aggregate("stddev", n, args, env)
    }),
    lazy!("integralvalue", Arity::Exact(2), |n, args, env| {
        let y = string_arg(n, &args[0])?;
        let x = string_arg(n, &args[1])?;
        Ok(crate::analysis::parametric::integral(
            env.ctx().parametric,
            &y,
            &x,
        ))
    }),
];

/// The ten ODE Table accessors share one body — `Evaluator.evalCall`'s
/// `case "odevalue", "finalvalue", …` arm, which dispatches on the function
/// name inside [`crate::ode::accessors::compute`].
fn ode_accessor<'a>(function: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let column = string_arg(function, &args[0])?;
    let arg = match args.get(1) {
        Some(expr) => Some(eval_in(expr, env)?),
        None => None,
    };
    crate::ode::accessors::resolve(env.ctx().ode, function, &column, arg, &env.to_scope())
}

/// `TableSum`/`TableAvg`/`TableMin`/`TableMax`/`TableStdDev` — one column-name
/// string, dispatched by the aggregate keyword the Java hard-codes per arm.
fn table_aggregate<'a>(
    op: &str,
    function: &str,
    args: &'a [Expr],
    env: &'a Env<'a>,
) -> Result<f64> {
    let column = string_arg(function, &args[0])?;
    Ok(crate::analysis::parametric::aggregate(
        env.ctx().parametric,
        op,
        &column,
    ))
}

/// The heat-exchanger correlations' view of the installed property backend.
const FLUID_BACKEND: crate::props::propfun::InstalledFluids =
    crate::props::propfun::InstalledFluids;

/// `f(fluid$, model$, T, P, phase$)` — the shared shape of five `eos_*` arms.
fn eos_tp<'a>(
    name: &str,
    args: &'a [Expr],
    env: &'a Env<'a>,
    f: fn(&str, &str, f64, f64, &str) -> Result<f64>,
) -> Result<f64> {
    let fluid = string_arg(name, &args[0])?;
    let model = string_arg(name, &args[1])?;
    let t = eval_in(&args[2], env)?;
    let p = eval_in(&args[3], env)?;
    let phase = string_arg(name, &args[4])?;
    f(&fluid, &model, t, p, &phase)
}

/// `AdiabaticFlameTemp(fuel$, phi, T_react)`.
fn flame_temp<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let fuel = string_arg(name, &args[0])?;
    let phi = eval_in(&args[1], env)?;
    let t_react = eval_in(&args[2], env)?;
    crate::props::thermochem::adiabatic_flame_temp(&fuel, phi, t_react)
}

/// `AdiabaticFlameTempEq(fuel$, phi, T_react, P)`.
fn flame_temp_eq<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let fuel = string_arg(name, &args[0])?;
    let phi = eval_in(&args[1], env)?;
    let t_react = eval_in(&args[2], env)?;
    let p = eval_in(&args[3], env)?;
    crate::props::equilibrium::adiabatic_flame_temp(&fuel, phi, t_react, p)
}

/// A leading `fluid$` string argument followed by exactly `N` numeric ones —
/// the shape every `htc_*` / `dp_*` correlation takes.
fn fluid_and_values<'a, const N: usize>(
    name: &str,
    args: &'a [Expr],
    env: &'a Env<'a>,
) -> Result<(String, [f64; N])> {
    let fluid = string_arg(name, &args[0])?;
    let mut out = [0.0f64; N];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = eval_in(&args[i + 1], env)?;
    }
    Ok((fluid, out))
}

fn registry() -> &'static HashMap<&'static str, &'static Intrinsic> {
    static REGISTRY: OnceLock<HashMap<&'static str, &'static Intrinsic>> = OnceLock::new();
    REGISTRY.get_or_init(|| INTRINSICS.iter().map(|i| (i.name, i)).collect())
}

/// The intrinsic registered under `name` (already lowercase), if any.
pub fn lookup_intrinsic(name: &str) -> Option<&'static Intrinsic> {
    registry().get(name).copied()
}

/// Every registered intrinsic name, sorted — the `getReference` surface
/// consumes this at integration time.
pub fn intrinsic_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = INTRINSICS.iter().map(|i| i.name).collect();
    names.sort_unstable();
    names
}

// ---------------------------------------------------------------------------
// Known-but-unported intrinsic families
// ---------------------------------------------------------------------------

/// `(name, family)` for Java arms this pass deliberately does not implement.
///
/// Reporting them as *not yet supported* rather than *unknown function* keeps
/// the distinction the parent engine cares about: a refusal is honest, a wrong
/// answer is not.
/// The two families whose names are implemented as a `CALL` intrinsic or a
/// matrix construct rather than as a scalar expression function. Naming the
/// *reason* keeps the message honest now that Phase 9 has wired both.
const MATRIX_FAMILY: &str = "matrix constructs — use the CALL or matrix-assignment form";
const CONTROL_FAMILY: &str = "control systems — use `CALL <name>(inputs : outputs)`";

const UNPORTED: &[(&str, &str)] = &[
    // Phase 7/8 emptied two sections: the ten ODE Table accessors
    // (`ODEValue`/`FinalValue`/`MaxValue`/`MinValue`/`TimeAt`/`ODEAvg`/
    // `ODESum`/`ODEStdDev`/`ODEMin`/`ODEMax`) and the ten parametric-table
    // accessors (`TableRun#`/`TableRun`/`NParametricRuns`/`TableValue`/
    // `TableSum`/`TableAvg`/`TableMin`/`TableMax`/`TableStdDev`/
    // `IntegralValue`) are all registered above and dispatch through
    // `EvalContext::ode` / `EvalContext::parametric`.
    // Matrices
    ("inv", MATRIX_FAMILY),
    ("det", MATRIX_FAMILY),
    ("trace", MATRIX_FAMILY),
    ("transpose", MATRIX_FAMILY),
    ("eig", MATRIX_FAMILY),
    ("eigvec", MATRIX_FAMILY),
    ("rank", MATRIX_FAMILY),
    ("norm", MATRIX_FAMILY),
    ("cond", MATRIX_FAMILY),
    ("svd", MATRIX_FAMILY),
    ("qr", MATRIX_FAMILY),
    ("cholesky", MATRIX_FAMILY),
    ("matexp", MATRIX_FAMILY),
    // Control systems
    ("tf", CONTROL_FAMILY),
    ("ss", CONTROL_FAMILY),
    ("tf2ss", CONTROL_FAMILY),
    ("ss2tf", CONTROL_FAMILY),
    ("series", CONTROL_FAMILY),
    ("parallel", CONTROL_FAMILY),
    ("feedback", CONTROL_FAMILY),
    ("impulse", CONTROL_FAMILY),
    ("lsim", CONTROL_FAMILY),
    ("bode", CONTROL_FAMILY),
    ("nyquist", CONTROL_FAMILY),
    ("nichols", CONTROL_FAMILY),
    ("margin", CONTROL_FAMILY),
    ("stepinfo", CONTROL_FAMILY),
    ("c2d", CONTROL_FAMILY),
    ("d2c", CONTROL_FAMILY),
    ("rlocus", CONTROL_FAMILY),
    ("routh", CONTROL_FAMILY),
    ("pade", CONTROL_FAMILY),
    ("lqr", CONTROL_FAMILY),
    ("dlqr", CONTROL_FAMILY),
    ("dare", CONTROL_FAMILY),
    ("lyap", CONTROL_FAMILY),
    ("dlyap", CONTROL_FAMILY),
    ("ctrb", CONTROL_FAMILY),
    ("obsv", CONTROL_FAMILY),
    ("place", CONTROL_FAMILY),
    ("acker", CONTROL_FAMILY),
    ("lqe", CONTROL_FAMILY),
    ("gram", CONTROL_FAMILY),
    ("balreal", CONTROL_FAMILY),
    ("pidtune", CONTROL_FAMILY),
    // Phase 9 note on the two blocks above: every one of those names is now
    // implemented — the matrix ones by `parser::expand`, the control ones by
    // `control::flatten` + `control::eval` — but **not as a scalar expression
    // function**, which is the only thing this table is about. `lqr` is a
    // `CALL` intrinsic and `inv` is a matrix construct; neither has an arm in
    // the Java `Evaluator.evalCall` either (checked: it has none, so the Java
    // answers "Unknown function"). Naming them here trades that message for
    // "not yet supported", which is the friendlier half-truth the port has
    // always given and which `family_of` now spells accurately.
    //
    // (The whole Bessel family — J/Y/I/K, fixed- and arbitrary-order, both
    // spellings — is implemented above.)
    //
    // Phase 5 emptied this section: the seven `eos_*` arms, the five
    // `AdiabaticFlameTemp*` spellings, the seven `mix_*` arms,
    // `eq_molefraction` and the nine `htc_*`/`dp_*` correlations are all
    // registered above and dispatch into `crate::props`.
];

/// The family `name` belongs to, if it is a known-but-unported Java arm.
fn unported_family(name: &str) -> Option<&'static str> {
    // The synthetic families this port still refuses wholesale. `prop$…` is
    // *not* one of them any more (Phase 5) — nor are `det$`/`qr$`/`chol$`/
    // `expm$`/`svd$`/`proc$`/`fft$`/… , which `eval_synthetic` dispatches, nor
    // the control-systems set, which Phase 9 routed into `control::eval`.
    if name.contains('$') {
        return Some("synthetic property / procedure / matrix call");
    }
    UNPORTED
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, family)| *family)
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

/// Evaluate `expr` to a scalar under `scope`.
///
/// Unknown variables are an [`crate::diag::FreesError::Evaluation`]; the solver
/// always supplies a full scope for the block it is iterating.
pub fn eval(expr: &Expr, scope: &Scope) -> Result<f64> {
    let env = Env::Root(scope);
    eval_in(expr, &env)
}

/// Document-level context an evaluation may need beyond the variable scope —
/// the Rust counterpart of the Java `Evaluator.eval(expr, values, defs)`
/// third argument. Phase-4 contract: the evaluator dispatches unknown call
/// names into `defs` (user `FUNCTION`s via [`crate::procedures::call_function`],
/// `TABLE`s via [`crate::curvetable::lookup`]) when a context is supplied.
///
/// Phase 7 adds two more optional channels, and both follow the same rule: an
/// **absent** channel is not an error, it is the documented no-context answer.
///
/// * `ode` — the live ODE Table bridge. `MaxValue('h')` with no context
///   evaluates to `0.0`, exactly as `DynamicAccessorContext.resolve` returns
///   when its thread-local is null. That is what makes an accessor harmless in
///   a document with no `DYNAMIC` block.
/// * `parametric` — the `TableValue`/`TableRun#`/`TableSum`/… accessors, whose
///   Java null-defaults [`crate::analysis::parametric`] already reproduces per
///   accessor (`current_run` → 0, `run_count` → 0, `cell` → 0.0, …).
///
/// The Java hangs both on thread-locals; this port threads them explicitly.
#[derive(Clone, Copy, Default)]
pub struct EvalContext<'a> {
    pub defs: Option<&'a crate::parser::defs::Definitions>,
    /// The live ODE Table bridge, installed by [`crate::engine`] for the
    /// second-solve pass. `None` is the Java's null thread-local.
    ///
    /// Held as a **trait object** rather than as
    /// `&'a DynamicAccessorContext<'a>` on purpose: that concrete type holds
    /// `RefCell`s over `'a` data, which makes it invariant in `'a` and would
    /// make `EvalContext<'a>` invariant with it — and `eval_with(expr, scope,
    /// ctx)` needs `ctx` to shorten to the call's borrow. Erasing to
    /// `&'a dyn OdeTableAccessors` drops the inner lifetime and restores
    /// covariance.
    pub ode: Option<&'a dyn crate::ode::accessors::OdeTableAccessors>,
    /// The parametric-sweep accessors, installed by
    /// [`crate::analysis::parametric`] while a row is being re-solved.
    pub parametric: Option<&'a crate::analysis::parametric::ParametricAccessors>,
}

impl<'a> EvalContext<'a> {
    /// The common case: a document context that carries only its definitions.
    pub fn with_defs(defs: &'a crate::parser::defs::Definitions) -> EvalContext<'a> {
        EvalContext {
            defs: Some(defs),
            ode: None,
            parametric: None,
        }
    }
}

// `DynamicAccessorContext` holds `RefCell`s and a boxed runner, so it has no
// `Debug`; formatting the *presence* of each channel is all any caller needs
// (the one consumer is a `{ctx:?}` in a solver diagnostic).
impl std::fmt::Debug for EvalContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvalContext")
            .field("defs", &self.defs)
            .field("ode", &self.ode.map(|_| "<installed>"))
            .field("parametric", &self.parametric.map(|_| "<installed>"))
            .finish()
    }
}

/// [`eval`] with a document context. With an empty context this is exactly
/// [`eval`]; a populated context lets unknown call names dispatch into the
/// document's definitions — user `FUNCTION`s via
/// [`crate::procedures::call_function`] and `TABLE`s via
/// [`crate::curvetable::lookup`] — with the Java precedence: **a user
/// definition shadows every intrinsic of the same name**
/// (`Evaluator.evalCall` consults `defs` before its builtin switch).
pub fn eval_with(expr: &Expr, scope: &Scope, ctx: EvalContext<'_>) -> Result<f64> {
    let env = Env::Doc { scope, ctx };
    eval_in(expr, &env)
}

/// Evaluate `expr` against an arbitrary [`Env`] — the entry point the binding
/// intrinsics recurse through.
pub fn eval_in<'a>(expr: &'a Expr, env: &'a Env<'a>) -> Result<f64> {
    match expr {
        // Literals are already SI: the parser folded `value * factor + offset`.
        // `is_imaginary` is ignored here exactly as the Java arm ignores it;
        // complex handling happens in the (unported) ComplexExpansion pass.
        Expr::Num { value, .. } => Ok(*value),

        Expr::Str(value) => Err(FreesError::evaluation(format!(
            "string literal '{value}' cannot be evaluated as a number; \
             string literals are only valid as function arguments"
        ))),

        Expr::Var(name) => match env.get(name) {
            Some(value) => Ok(value),
            // `#`-suffixed built-ins are normally substituted at parse time;
            // resolve them here too so a hand-built AST behaves identically.
            None => match lookup_constant(name) {
                Some(constant) => Ok(constant.value),
                None => Err(FreesError::evaluation(format!(
                    "variable has no value: {name}"
                ))),
            },
        },

        Expr::Neg(operand) => Ok(-eval_in(operand, env)?),

        Expr::BinOp { op, left, right } => {
            let l = eval_in(left, env)?;
            let r = eval_in(right, env)?;
            apply_binop(*op, l, r)
        }

        Expr::Compare { op, left, right } => {
            let l = eval_in(left, env)?;
            let r = eval_in(right, env)?;
            let truth = match op {
                CmpOp::Lt => l < r,
                CmpOp::Gt => l > r,
                CmpOp::Le => l <= r,
                CmpOp::Ge => l >= r,
                CmpOp::Ne => l != r,
                CmpOp::Eq => l == r,
            };
            Ok(if truth { 1.0 } else { 0.0 })
        }

        // Java evaluates *both* operands before combining — no short-circuit.
        // Preserved so evaluation order (and therefore which error surfaces
        // first) matches the reference engine.
        Expr::Logical { op, left, right } => {
            let l = eval_in(left, env)?;
            let r = eval_in(right, env)?;
            let truth = match op {
                LogicOp::And => l != 0.0 && r != 0.0,
                LogicOp::Or => l != 0.0 || r != 0.0,
            };
            Ok(if truth { 1.0 } else { 0.0 })
        }

        Expr::Not(operand) => Ok(if eval_in(operand, env)? == 0.0 {
            1.0
        } else {
            0.0
        }),

        Expr::Call { function, args } => eval_call(function, args, env),

        // The parser rewrites `a[i]` into the scalar variable it names before a
        // solve; reaching the evaluator means that rewrite has not happened.
        Expr::ArrayAccess { name, indices } => Err(FreesError::evaluation(format!(
            "array access `{name}[…]` cannot be evaluated directly \
             ({} index expression(s)); it must be expanded to scalars first",
            indices.len()
        ))),

        Expr::Range { .. } => Err(FreesError::evaluation(
            "an index range `a:b` cannot be evaluated as a scalar".to_string(),
        )),

        Expr::ArrayLiteral(elements) => Err(FreesError::evaluation(format!(
            "an array literal ({} element(s)) cannot be evaluated as a scalar",
            elements.len()
        ))),
    }
}

/// `pub(crate)` since Wave A2: [`crate::solver::slots`] compiles a block's
/// arithmetic to slot-indexed postfix and must combine operands through this
/// exact function, or its answers would not be the oracle's.
pub(crate) fn apply_binop(op: BinOp, l: f64, r: f64) -> Result<f64> {
    // The four element-wise operators degenerate to their scalar forms.
    let op = op.scalar_equivalent();
    match op {
        BinOp::Add => Ok(l + r),
        BinOp::Sub => Ok(l - r),
        BinOp::Mul => Ok(l * r),
        BinOp::Div => {
            if r == 0.0 {
                return Err(FreesError::evaluation(format!("division by zero: {l} / 0")));
            }
            Ok(l / r)
        }
        // `a \ b` is `b / a` for scalars (A⁻¹B for matrices).
        BinOp::LeftDiv => {
            if l == 0.0 {
                return Err(FreesError::evaluation(format!(
                    "division by zero: 0 \\ {r} (left division divides by the left operand)"
                )));
            }
            Ok(r / l)
        }
        BinOp::Pow => {
            if l < 0.0 && r != libm::floor(r) {
                return Err(FreesError::evaluation(format!(
                    "negative base raised to a non-integer power: {l} ^ {r}"
                )));
            }
            if l == 0.0 && r < 0.0 {
                return Err(FreesError::evaluation(format!(
                    "division by zero: 0 raised to the negative power {r}"
                )));
            }
            // Java's `^` is `Math.pow`, which is **not** C's `pow`, and the two
            // disagree on exactly two families: `pow(1, NaN)` and `pow(±1, ±∞)`
            // are `NaN` in Java and `1` in C. Both invent a value out of a
            // missing one, so C's answer is not merely a parity difference —
            // it is a wrong number that looks like a measurement.
            //
            // Found twice, in the Phase-10 measurement calc tree and then
            // again here, because a formula's *function arguments* are
            // evaluated by this evaluator rather than compiled — so
            // `abs(b ^ e)` re-introduced the invented `1.0` after the calc
            // tree had removed it. The calc tree is gone (D11, 2026-08-24);
            // this site is the engine-wide one and it stands. Pinned by
            // `a_non_finite_exponent_answers_the_java_way`, which carries
            // over the assertions the deleted `measurement_parity.rs::
            // a_gap_in_the_exponent_stays_a_gap_inside_a_call_argument` made.
            if r.is_nan() || (r.is_infinite() && l.abs() == 1.0) {
                return Ok(f64::NAN);
            }
            Ok(libm::pow(l, r))
        }
        // `scalar_equivalent` maps every element-wise operator onto one of the
        // arms above, so nothing reaches here.
        BinOp::ElemMul | BinOp::ElemDiv | BinOp::ElemLeftDiv | BinOp::ElemPow => Err(
            FreesError::evaluation(format!("unhandled operator: {}", op.as_str())),
        ),
    }
}

fn eval_call<'a>(function: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    // Java precedence (`Evaluator.evalCall`): the definitions map is consulted
    // *first*, so a user TABLE or FUNCTION shadows an intrinsic of the same
    // name. (In Java both live in one map; here tables are checked before
    // functions to mirror the Java `instanceof` chain, and the parser rejects
    // duplicate names across kinds.)
    if let Some(defs) = env.ctx().defs {
        if let Some(table) = defs.table(function) {
            return eval_table_def_call(table, args, env);
        }
        if let Some(def) = defs.function(function) {
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(eval_in(arg, env)?);
            }
            return crate::procedures::call_function(def, &values, defs, &env.to_scope());
        }
    }

    let Some(intrinsic) = lookup_intrinsic(function) else {
        if function.contains('$') {
            return eval_synthetic(function, args, env);
        }
        return Err(match unported_family(function) {
            Some(family) => {
                FreesError::evaluation(format!("not yet supported: {function} ({family})"))
            }
            None => FreesError::evaluation(format!("unknown function: {function}")),
        });
    };

    if !intrinsic.arity.accepts(args.len()) {
        return Err(FreesError::evaluation(format!(
            "{function} expects {}, got {}",
            intrinsic.arity.describe(),
            args.len()
        )));
    }

    match intrinsic.body {
        Body::Lazy(f) => f(function, args, env),
        Body::Strict(f) => {
            let mut values = Vec::with_capacity(args.len());
            for arg in args {
                values.push(eval_in(arg, env)?);
            }
            f(function, &values)
        }
    }
}

// ---------------------------------------------------------------------------
// Definition & synthetic-call dispatch (Phase 4)
// ---------------------------------------------------------------------------

/// A user `TABLE` called by name: `t(x)` or `t(x, param)`
/// (`Evaluator.evalFunctionTable`).
fn eval_table_def_call<'a>(
    table: &crate::parser::defs::FunctionTableDef,
    args: &'a [Expr],
    env: &'a Env<'a>,
) -> Result<f64> {
    if args.is_empty() || args.len() > 2 {
        let n = &table.name;
        return Err(FreesError::evaluation(format!(
            "Function table '{n}' expects {n}(x) or {n}(x, param)."
        )));
    }
    let x = eval_in(&args[0], env)?;
    let param = match args.get(1) {
        Some(expr) => Some(eval_in(expr, env)?),
        None => None,
    };
    crate::curvetable::lookup(table, x, param)
}

/// The classic-solver table functions resolve their first (string) argument
/// against the document's `TABLE` definitions.
fn table_def<'a>(
    fn_name: &str,
    table_arg: &Expr,
    env: &'a Env<'a>,
) -> Result<&'a crate::parser::defs::FunctionTableDef> {
    let name = string_arg(fn_name, table_arg)?.to_lowercase();
    match env.ctx().defs.and_then(|defs| defs.table(&name)) {
        Some(table) => Ok(table),
        None => Err(domain(fn_name, format_args!("'{name}' is not a TABLE."))),
    }
}

/// Shared body of `Differentiate` / `Differentiate1`.
fn table_derivative<'a>(
    name: &str,
    args: &'a [Expr],
    env: &'a Env<'a>,
    cubic: bool,
) -> Result<f64> {
    let table = table_def(name, &args[0], env)?;
    let y_col = java_int(eval_in(&args[1], env)?);
    let x_col = java_int(eval_in(&args[2], env)?);
    let x_val = eval_in(&args[3], env)?;
    crate::curvetable::differentiate(table, y_col, x_col, x_val, cubic)
}

/// `Integral(f, t, a, b[, step])` — dispatch to [`crate::integral::integral`].
fn eval_integral_call<'a>(_name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let Expr::Var(var) = &args[1] else {
        return Err(FreesError::evaluation(
            "Integral expects Integral(f, t, lower, upper[, step]).",
        ));
    };
    let lower = eval_in(&args[2], env)?;
    let upper = eval_in(&args[3], env)?;
    if lower == upper {
        return Ok(0.0);
    }
    let step = match args.get(4) {
        Some(expr) => Some(eval_in(expr, env)?),
        None => None,
    };
    // `_with` rather than the bare contract: the Java threads its `defs` map
    // through `SimpsonContext.evalAt`, so a user FUNCTION or TABLE inside the
    // integrand has to resolve.
    let scope = env.to_scope();
    crate::integral::integral_with(&args[0], var, lower, upper, step, &scope, env.ctx())
}

/// `GaussIntegral(f, t, a, b[, points])` — dispatch to
/// [`crate::integral::gauss_integral`]. Points are rounded and clamped to
/// `[2, 64]` exactly as the Java arm does before handing off.
fn eval_gauss_integral_call<'a>(_name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let Expr::Var(var) = &args[1] else {
        return Err(FreesError::evaluation(
            "GaussIntegral expects GaussIntegral(f, t, lower, upper[, points]).",
        ));
    };
    let lower = eval_in(&args[2], env)?;
    let upper = eval_in(&args[3], env)?;
    if lower == upper {
        return Ok(0.0);
    }
    let points = match args.get(4) {
        Some(expr) => {
            let raw = java_int(eval_in(expr, env)?);
            Some(raw.clamp(2, 64) as usize)
        }
        None => None,
    };
    let scope = env.to_scope();
    crate::integral::gauss_integral_with(&args[0], var, lower, upper, points, &scope, env.ctx())
}

/// Normalize a bracket literal to its rows of cells, or `None` when `expr` is
/// not a literal of uniform depth.
///
/// The wasm port has no run-time array values — a bracket literal is the only
/// shape an unexpanded vector or grid can take in expression position — so the
/// literal is read straight off the AST. Three spellings normalize to the same
/// thing:
///
/// * `[1, 2, 3]` — what `parse_matrix_literal` builds: `ArrayLiteral` of one
///   `ArrayLiteral` per row, so a row vector arrives as `[[1, 2, 3]]`. A
///   hand-built flat `ArrayLiteral([1, 2, 3])` is accepted as that same row.
/// * `[1, 2; 3, 4]` — the frees matrix literal, `;` separating rows.
/// * `[[1, 2], [3, 4]]` — the JSON-ish spelling. The frees grammar parses that
///   as a *one-row* matrix whose two cells are themselves `1x2` matrices, which
///   is not a meaningful shape on its own; the single-element unwrap below
///   reads it as the 2x2 grid it obviously means.
fn literal_rows(expr: &Expr) -> Option<Vec<Vec<&Expr>>> {
    let Expr::ArrayLiteral(elements) = expr else {
        return None;
    };
    // A flat literal is one row of scalars.
    if elements.iter().all(|e| !matches!(e, Expr::ArrayLiteral(_))) {
        return Some(vec![elements.iter().collect()]);
    }
    // `[[1, 2], [3, 4]]`: one element that is itself already 2-D.
    if elements.len() == 1 {
        if let Some(inner) = literal_rows(&elements[0]) {
            if inner.len() > 1 {
                return Some(inner);
            }
        }
    }
    // Otherwise every element is one row, and each row must be 1-D.
    let mut out = Vec::with_capacity(elements.len());
    for element in elements {
        let row = literal_rows(element)?;
        if row.len() != 1 {
            return None;
        }
        out.push(row.into_iter().next().unwrap());
    }
    Some(out)
}

/// True when `expr` is a bracket literal that normalizes to a genuine 2-D grid
/// (at least 2 rows *and* 2 columns) — the shape only an interpolation grid can
/// have, which is what pins `Interp2`'s argument order.
fn is_grid_literal(expr: &Expr) -> bool {
    match literal_rows(expr) {
        Some(rows) => rows.len() >= 2 && rows.iter().all(|row| row.len() >= 2),
        None => false,
    }
}

/// A 1-D vector argument: a row literal `[1, 2, 3]` or a column `[1; 2; 3]`.
fn vector_arg<'a>(fn_name: &str, which: &str, arg: &'a Expr, env: &'a Env<'a>) -> Result<Vec<f64>> {
    let Some(rows) = literal_rows(arg) else {
        return Err(domain(
            fn_name,
            format_args!("{which} must be a list of numbers, e.g. [1, 2, 3]"),
        ));
    };
    let cells: Vec<&Expr> = if rows.len() == 1 {
        rows.into_iter().next().unwrap_or_default()
    } else if rows.iter().all(|row| row.len() == 1) {
        rows.into_iter().map(|row| row[0]).collect()
    } else {
        let cols = rows.first().map_or(0, Vec::len);
        return Err(domain(
            fn_name,
            format_args!(
                "{which} must be a 1-D list of numbers, not a {}x{cols} matrix",
                rows.len()
            ),
        ));
    };
    let mut out = Vec::with_capacity(cells.len());
    for cell in cells {
        out.push(eval_in(cell, env)?);
    }
    Ok(out)
}

/// A 2-D grid argument, row-major, every row the same length.
fn matrix_arg<'a>(fn_name: &str, arg: &'a Expr, env: &'a Env<'a>) -> Result<Vec<Vec<f64>>> {
    let Some(rows) = literal_rows(arg) else {
        return Err(domain(
            fn_name,
            format_args!("Z must be a grid literal, e.g. [0, 1; 1, 2] or [[0, 1], [1, 2]]"),
        ));
    };
    let cols = rows.first().map_or(0, Vec::len);
    if rows.iter().any(|row| row.len() != cols) {
        return Err(domain(
            fn_name,
            format_args!("Z has rows of unequal length"),
        ));
    }
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut values = Vec::with_capacity(row.len());
        for cell in row {
            values.push(eval_in(cell, env)?);
        }
        out.push(values);
    }
    Ok(out)
}

/// `slope(xvals, yvals)` / `intercept(…)` / `r2(…)` — the three
/// `FunctionRegistry` names for `Statistics.linFit`, output-mapped exactly as
/// `Evaluator.evalLinFit` maps the `linfit$…` synthetic.
fn eval_lin_fit_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let x = vector_arg(name, "xvals", &args[0], env)?;
    let y = vector_arg(name, "yvals", &args[1], env)?;
    let fit = crate::statistics::lin_fit(&x, &y)?;
    match name {
        "slope" => Ok(fit[0]),
        "intercept" => Ok(fit[1]),
        "r2" => Ok(fit[2]),
        // Unreachable: only the three names above register this body.
        other => Err(FreesError::evaluation(format!(
            "Unknown LinFit output: {other}"
        ))),
    }
}

// --- Phase 4.1: Hypothesis test dispatchers --------------------------------
//
// Each test function computes the full result struct, then the function name
// selects the field to return — the same pattern as `eval_lin_fit_call`.

/// Wrapper that calls `student_t_cdf` with the function-pointer signature
/// the descriptive module expects.
fn t_cdf_fn(x: f64, df: f64) -> Result<f64> {
    student_t_cdf(x, df)
}

fn t_inv_fn(p: f64, df: f64) -> Result<f64> {
    student_t_inv(p, df)
}

fn f_cdf_fn(x: f64, d1: f64, d2: f64) -> Result<f64> {
    f_dist_cdf(x, d1, d2)
}

fn chi2_cdf_fn(x: f64, df: f64) -> Result<f64> {
    regularized_gamma_p(df / 2.0, x / 2.0)
}

/// `ttest1_stat(mu0, data)` / `ttest1_pval(…)` / `ttest1_df(…)`
fn eval_ttest1_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let mu0 = eval_in(&args[0], env)?;
    let x = vector_arg(name, "data", &args[1], env)?;
    let r = crate::descriptive::ttest_one_sample(mu0, &x, t_cdf_fn)?;
    match name {
        "ttest1_stat" => Ok(r.statistic),
        "ttest1_pval" => Ok(r.p_value),
        "ttest1_df" => Ok(r.df),
        other => Err(FreesError::evaluation(format!(
            "Unknown ttest1 output: {other}"
        ))),
    }
}

/// `ttest_paired_stat(x, y)` / `ttest_paired_pval(…)` / `ttest_paired_df(…)`
fn eval_ttest_paired_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let x = vector_arg(name, "x", &args[0], env)?;
    let y = vector_arg(name, "y", &args[1], env)?;
    let r = crate::descriptive::ttest_paired(&x, &y, t_cdf_fn)?;
    match name {
        "ttest_paired_stat" => Ok(r.statistic),
        "ttest_paired_pval" => Ok(r.p_value),
        "ttest_paired_df" => Ok(r.df),
        other => Err(FreesError::evaluation(format!(
            "Unknown ttest_paired output: {other}"
        ))),
    }
}

/// `ttest2_stat(x, y)` / `ttest2_pval(…)` / `ttest2_df(…)`
fn eval_ttest2_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let x = vector_arg(name, "x", &args[0], env)?;
    let y = vector_arg(name, "y", &args[1], env)?;
    let r = crate::descriptive::ttest_welch(&x, &y, t_cdf_fn)?;
    match name {
        "ttest2_stat" => Ok(r.statistic),
        "ttest2_pval" => Ok(r.p_value),
        "ttest2_df" => Ok(r.df),
        other => Err(FreesError::evaluation(format!(
            "Unknown ttest2 output: {other}"
        ))),
    }
}

/// `anova1_f(g1, g2, …)` / `anova1_pval(…)` / `anova1_df_between(…)` /
/// `anova1_df_within(…)`
fn eval_anova1_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let mut groups: Vec<Vec<f64>> = Vec::with_capacity(args.len());
    for (i, arg) in args.iter().enumerate() {
        groups.push(vector_arg(name, &format!("group{}", i + 1), arg, env)?);
    }
    let group_refs: Vec<&[f64]> = groups.iter().map(|g| g.as_slice()).collect();
    let r = crate::descriptive::anova_oneway(&group_refs, f_cdf_fn)?;
    match name {
        "anova1_f" => Ok(r.f_statistic),
        "anova1_pval" => Ok(r.p_value),
        "anova1_df_between" => Ok(r.df_between),
        "anova1_df_within" => Ok(r.df_within),
        other => Err(FreesError::evaluation(format!(
            "Unknown anova1 output: {other}"
        ))),
    }
}

/// `chi2gof_stat(observed, expected)` / `chi2gof_pval(…)` / `chi2gof_df(…)`
fn eval_chi2gof_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let obs = vector_arg(name, "observed", &args[0], env)?;
    let exp = vector_arg(name, "expected", &args[1], env)?;
    let r = crate::descriptive::chi2_goodness_of_fit(&obs, &exp, chi2_cdf_fn)?;
    match name {
        "chi2gof_stat" => Ok(r.statistic),
        "chi2gof_pval" => Ok(r.p_value),
        "chi2gof_df" => Ok(r.df),
        other => Err(FreesError::evaluation(format!(
            "Unknown chi2gof output: {other}"
        ))),
    }
}

/// `ci_mean_lo(confidence, data)` / `ci_mean_hi(…)`
fn eval_ci_mean_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let confidence = eval_in(&args[0], env)?;
    let x = vector_arg(name, "data", &args[1], env)?;
    let (lo, hi) = crate::descriptive::ci_mean(confidence, &x, t_inv_fn)?;
    match name {
        "ci_mean_lo" => Ok(lo),
        "ci_mean_hi" => Ok(hi),
        other => Err(FreesError::evaluation(format!(
            "Unknown ci_mean output: {other}"
        ))),
    }
}

/// `bootstrap_ci_lo(confidence, n_resamples, seed, data)` / `bootstrap_ci_hi(…)`
fn eval_bootstrap_ci_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let confidence = eval_in(&args[0], env)?;
    let n_resamples = eval_in(&args[1], env)? as usize;
    let seed = eval_in(&args[2], env)? as i64;
    let x = vector_arg(name, "data", &args[3], env)?;
    let (lo, hi) = crate::descriptive::bootstrap_ci_mean(confidence, n_resamples, seed, &x)?;
    match name {
        "bootstrap_ci_lo" => Ok(lo),
        "bootstrap_ci_hi" => Ok(hi),
        other => Err(FreesError::evaluation(format!(
            "Unknown bootstrap_ci output: {other}"
        ))),
    }
}

/// `permtest_stat(n_perms, seed, x, y)` / `permtest_pval(…)`
fn eval_permtest_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let n_perms = eval_in(&args[0], env)? as usize;
    let seed = eval_in(&args[1], env)? as i64;
    let x = vector_arg(name, "x", &args[2], env)?;
    let y = vector_arg(name, "y", &args[3], env)?;
    let r = crate::descriptive::permutation_test_means(n_perms, seed, &x, &y)?;
    match name {
        "permtest_stat" => Ok(r.statistic),
        "permtest_pval" => Ok(r.p_value),
        other => Err(FreesError::evaluation(format!(
            "Unknown permtest output: {other}"
        ))),
    }
}

/// `Interp2` in **expression position** — regular-grid 2-D interpolation
/// through [`crate::interp2::interpolate`].
///
/// **Port extension, documented.** The Java engine reaches `Interpolation2D`
/// only through `CALL Interp2(x, y, Z, xq, yq : zq)`, which
/// `EquationParser.flattenInterp2` rewrites into the `interp2$<m>$<n>`
/// synthetic (dispatched below); there is no `interp2` arm in
/// `Evaluator.evalCall` and no `FunctionRegistry` entry, so `z = Interp2(…)`
/// is an "unknown function" error in Java. Wiring it here turns that error
/// into the value the same kernel computes — it can never disagree with the
/// oracle on a document the oracle accepts.
///
/// Two argument orders are accepted, and the 2-D grid pins which one is meant
/// (only `Z` can be `m x n` with `m, n >= 2`, so the choice is never ambiguous):
///
/// * `Interp2(x, y, Z, xq, yq)` — the `CALL Interp2` order, `Z` third.
/// * `Interp2(xq, yq, x, y, Z)` — query first, `Z` last.
fn eval_interp2_call<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let (xi, yi, zi, xqi, yqi) = if is_grid_literal(&args[2]) {
        (0, 1, 2, 3, 4)
    } else if is_grid_literal(&args[4]) {
        (2, 3, 4, 0, 1)
    } else {
        return Err(domain(
            name,
            format_args!(
                "expects Interp2(x, y, Z, xq, yq) or Interp2(xq, yq, x, y, Z), \
                 with Z an m x n grid literal such as [0, 1; 1, 2]"
            ),
        ));
    };
    let x = vector_arg(name, "x", &args[xi], env)?;
    let y = vector_arg(name, "y", &args[yi], env)?;
    let z = matrix_arg(name, &args[zi], env)?;
    let xq = eval_in(&args[xqi], env)?;
    let yq = eval_in(&args[yqi], env)?;
    crate::interp2::interpolate(&x, &y, &z, xq, yq)
}

/// Synthetic `$`-calls the flattening passes generate: the procedure, signal,
/// regression and interpolation families through the Phase-4 kernels, the
/// property calls through `props`, the dense linear algebra (incl. the eigen
/// decompositions) through `linalg`, and the control systems through
/// `control::eval`. Anything else reports *not yet supported*.
fn eval_synthetic<'a>(function: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let parts: Vec<&str> = function.split('$').collect();
    match parts[0] {
        // prop$<output>$<fluid>$<ind1>$<ind2>(values…): the fluid-property call
        // `parser::expr::build_property_call` emits for
        // `Enthalpy(R134a, T=T1, x=1)`, and the chemistry form
        // `prop$molarmass('C8H18')` from `build_chem_call`.
        //
        // Port of `Evaluator.evalProperty`: string-literal arguments are the
        // case-preserved *tokens* (fluid / formula / mode), everything else is
        // evaluated to a number, and the two lists keep their own order.
        "prop" => {
            let mut values = Vec::with_capacity(args.len());
            let mut tokens = Vec::new();
            for arg in args {
                match arg {
                    Expr::Str(s) => tokens.push(s.clone()),
                    other => values.push(eval_in(other, env)?),
                }
            }
            crate::props::propfun::evaluate_with_tokens(function, &values, &tokens)
        }
        // proc$<name>$<k>: the per-output call `procedures::flatten_calls`
        // emits for `CALL p(ins : outs)` — and, because the parser desugars
        // `FUNCTION [a, b] = f(x)` into a `ProcedureDef`, for the multi-output
        // destructuring `[a, b] = f(x)` too. Port of
        // `Evaluator.evalProcedureOutput`.
        //
        // **Deliberately not memoised.** The Java runs the *entire* body once
        // per output slot: `evalProcedureOutput` constructs a fresh
        // `ProcedureEvaluator` and calls `callProcedure` on every `proc$name$k`
        // it evaluates, so an N-output PROCEDURE executes N times per residual
        // sweep. That is observable — a body calling `Random`/`RandG` with a
        // seed derived from its inputs, or one that is merely expensive —
        // so this port matches it rather than caching behind the oracle's back.
        "proc" => {
            let inputs = eval_args(args, env)?;
            let scope = env.to_scope();
            match env.ctx().defs {
                Some(defs) => crate::procedures::call_proc_output(function, &inputs, defs, &scope),
                // Java reaches the same throw when `defs.get(name)` is absent.
                None => Err(FreesError::evaluation(format!(
                    "Unknown procedure output call: {function}"
                ))),
            }
        }
        // fft$re|im$<k>$<n> / ifft$…: DFT of the complex sequence carried as
        // n real args then n imaginary args.
        "fft" | "ifft" if parts.len() == 4 => {
            let inverse = parts[0] == "ifft";
            let want_re = parts[1] == "re";
            let k = synthetic_index(function, parts[2])?;
            let n = synthetic_index(function, parts[3])?;
            check_synthetic_args(function, args, 2 * n)?;
            let re = eval_args(&args[..n], env)?;
            let im = eval_args(&args[n..2 * n], env)?;
            let (out_re, out_im) = crate::signal::dft(&re, &im, inverse)?;
            let out = if want_re { out_re } else { out_im };
            out.get(k)
                .copied()
                .ok_or_else(|| synthetic_bounds(function, k, out.len()))
        }
        // conv$<k>$<m>$<n>: k-th element of the linear convolution.
        "conv" if parts.len() == 4 => {
            let k = synthetic_index(function, parts[1])?;
            let m = synthetic_index(function, parts[2])?;
            let n = synthetic_index(function, parts[3])?;
            check_synthetic_args(function, args, m + n)?;
            let a = eval_args(&args[..m], env)?;
            let b = eval_args(&args[m..m + n], env)?;
            let out = crate::signal::convolve(&a, &b)?;
            out.get(k)
                .copied()
                .ok_or_else(|| synthetic_bounds(function, k, out.len()))
        }
        // linfit$slope|intercept|r2$<n>: OLS line fit over (x, y).
        "linfit" if parts.len() == 3 => {
            let which = parts[1];
            let n = synthetic_index(function, parts[2])?;
            check_synthetic_args(function, args, 2 * n)?;
            let x = eval_args(&args[..n], env)?;
            let y = eval_args(&args[n..2 * n], env)?;
            let fit = crate::statistics::lin_fit(&x, &y)?;
            match which {
                "slope" => Ok(fit[0]),
                "intercept" => Ok(fit[1]),
                "r2" => Ok(fit[2]),
                other => Err(FreesError::evaluation(format!(
                    "Unknown LinFit output: {other}"
                ))),
            }
        }
        // polyfit$<k>$<deg>$<n>: k-th ascending-power least-squares coefficient.
        "polyfit" if parts.len() == 4 => {
            let k = synthetic_index(function, parts[1])?;
            let degree = synthetic_index(function, parts[2])?;
            let n = synthetic_index(function, parts[3])?;
            check_synthetic_args(function, args, 2 * n)?;
            let x = eval_args(&args[..n], env)?;
            let y = eval_args(&args[n..2 * n], env)?;
            let coeffs = crate::statistics::poly_fit(&x, &y, degree)?;
            coeffs
                .get(k)
                .copied()
                .ok_or_else(|| synthetic_bounds(function, k, coeffs.len()))
        }
        // interp2$<m>$<n>: regular-grid 2-D interpolation at (xq, yq).
        // Args: x (m), y (n), Z (m×n row-major), xq, yq.
        "interp2" if parts.len() == 3 => {
            let m = synthetic_index(function, parts[1])?;
            let n = synthetic_index(function, parts[2])?;
            check_synthetic_args(function, args, m + n + m * n + 2)?;
            let x = eval_args(&args[..m], env)?;
            let y = eval_args(&args[m..m + n], env)?;
            let mut z = Vec::with_capacity(m);
            for i in 0..m {
                let start = m + n + i * n;
                z.push(eval_args(&args[start..start + n], env)?);
            }
            let xq = eval_in(&args[m + n + m * n], env)?;
            let yq = eval_in(&args[m + n + m * n + 1], env)?;
            crate::interp2::interpolate(&x, &y, &z, xq, yq)
        }
        // The dense linear-algebra synthetics, all of which take their matrix
        // flattened row-major in the argument list:
        //   det$<n>, qr$q|r$<i>$<j>$<m>$<n>, chol$l$<i>$<j>$<n>,
        //   expm$<i>$<j>$<n>, svd$s$<k>$<m>$<n>, svd$u|smat|v$<i>$<j>$<m>$<n>,
        //   eigen$val|re|im$<k>$<n>, eigen$vec$<i>$<k>$<n>
        // Port of the `startsWith("det$")` / `qr$` / `chol$` / `expm$` / `svd$`
        // / `eigen$` chain in `Evaluator.evalCall`, which routes each into
        // `core.LinearAlgebra` (eigen into `Evaluator.evalEigen`). `det$<n>` is
        // the one a user reaches without a CALL: `EquationParser` (and
        // `parser::expand`) emit it for `det(A)` whenever `A` is larger than
        // 3×3, because the closed-form cofactor expansion is O(n!).
        "det" | "qr" | "chol" | "expm" | "svd" | "eigen" => {
            let values = eval_args(args, env)?;
            match crate::linalg::eval_intrinsic(function, &values) {
                Some(result) => result,
                None => Err(unsupported_synthetic(function)),
            }
        }
        // The control-systems synthetics (`ss2tf$…`, `step$…`, `lqr$…`, 42
        // heads in all) that `control::flatten` emits. Port of the
        // `startsWith("<op>$")` chain in `Evaluator.evalCall` plus the
        // `ControlSystemsEvaluator` delegation. Arguments are evaluated
        // eagerly, the same shape `det$`/`qr$`/`svd$` already use — see
        // `control::eval`'s note on the Java evaluators that read only a
        // subset of their arguments.
        _ if crate::control::eval::handles(function) => {
            let values = eval_args(args, env)?;
            match crate::control::eval::eval_intrinsic(function, &values) {
                Some(result) => result,
                None => Err(unsupported_synthetic(function)),
            }
        }
        _ => Err(unsupported_synthetic(function)),
    }
}

fn unsupported_synthetic(function: &str) -> FreesError {
    FreesError::evaluation(format!(
        "not yet supported: {function} ({})",
        unported_family(function).unwrap_or("synthetic property / procedure / matrix call")
    ))
}

fn eval_args<'a>(args: &'a [Expr], env: &'a Env<'a>) -> Result<Vec<f64>> {
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        values.push(eval_in(arg, env)?);
    }
    Ok(values)
}

fn synthetic_index(function: &str, part: &str) -> Result<usize> {
    part.parse::<usize>()
        .map_err(|_| FreesError::evaluation(format!("malformed synthetic call: {function}")))
}

fn check_synthetic_args(function: &str, args: &[Expr], expected: usize) -> Result<()> {
    if args.len() != expected {
        return Err(FreesError::evaluation(format!(
            "{function} expects {expected} argument(s), got {}",
            args.len()
        )));
    }
    Ok(())
}

fn synthetic_bounds(function: &str, k: usize, len: usize) -> FreesError {
    FreesError::evaluation(format!(
        "{function}: output index {k} is out of range 0..{len}"
    ))
}

// ---------------------------------------------------------------------------
// Lazy intrinsic bodies
// ---------------------------------------------------------------------------

/// `If(a, b, lt, eq, gt)` — the frees/EES five-argument form — plus the
/// three-argument `if(cond, then, else)` shorthand. Only the taken branch is
/// evaluated in either form.
fn eval_if<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    if args.len() == 3 {
        let condition = eval_in(&args[0], env)?;
        // Same truthiness rule as `and`/`or`/`not`: non-zero is true.
        return if condition != 0.0 {
            eval_in(&args[1], env)
        } else {
            eval_in(&args[2], env)
        };
    }
    let a = eval_in(&args[0], env)?;
    let b = eval_in(&args[1], env)?;
    if a < b {
        eval_in(&args[2], env)
    } else if a == b {
        eval_in(&args[3], env)
    } else if a > b {
        eval_in(&args[4], env)
    } else {
        // Only reachable when a comparison against NaN made every branch false.
        Err(domain(
            name,
            format_args!("cannot compare {a} with {b} (not a number)"),
        ))
    }
}

/// Iterations a bounded `sum`/`product` may run before it is treated as a
/// runaway. The Java engine has no cap; in a browser an unbounded loop is a
/// hang, so this port refuses instead.
const MAX_REDUCTION_ITERATIONS: i64 = 1 << 24;

/// `sum(v, lo, hi, body)` / `product(v, lo, hi, body)` bind `v` over `body`;
/// every other shape is a plain reduction over the argument list. This mirrors
/// the special case in `Expr::variables()` — `v` never escapes as an unknown.
fn eval_reduction<'a>(
    name: &str,
    args: &'a [Expr],
    env: &'a Env<'a>,
    identity: f64,
    combine: fn(f64, f64) -> f64,
) -> Result<f64> {
    if args.len() == 4 {
        if let Expr::Var(index) = &args[0] {
            let lo = reduction_bound(name, "lower", eval_in(&args[1], env)?)?;
            let hi = reduction_bound(name, "upper", eval_in(&args[2], env)?)?;
            let span = (hi - lo).abs() + 1;
            if span > MAX_REDUCTION_ITERATIONS {
                return Err(domain(
                    name,
                    format_args!("{span} iterations exceeds the {MAX_REDUCTION_ITERATIONS} limit"),
                ));
            }
            let step = if lo <= hi { 1 } else { -1 };
            let mut acc = identity;
            let mut i = lo;
            loop {
                let inner = Env::Bind {
                    name: index.as_str(),
                    value: i as f64,
                    parent: env,
                };
                acc = combine(acc, eval_in(&args[3], &inner)?);
                if i == hi {
                    break;
                }
                i += step;
            }
            return Ok(acc);
        }
    }
    let mut acc = identity;
    for arg in args {
        acc = combine(acc, eval_in(arg, env)?);
    }
    Ok(acc)
}

/// `(int) Math.round(x)` with the silent truncation turned into an error.
fn reduction_bound(name: &str, which: &str, value: f64) -> Result<i64> {
    if !value.is_finite() {
        return Err(domain(
            name,
            format_args!("{which} bound is not a finite number ({value})"),
        ));
    }
    let rounded = java_round(value);
    if libm::fabs(rounded) > i32::MAX as f64 {
        return Err(domain(
            name,
            format_args!("{which} bound {value} is out of range"),
        ));
    }
    Ok(rounded as i64)
}

/// Java's `Evaluator.evalString`: a quoted literal, or a bare identifier
/// treated as a string label for backward compatibility.
fn string_arg(name: &str, expr: &Expr) -> Result<String> {
    match expr {
        Expr::Str(value) => Ok(value.clone()),
        Expr::Var(value) => Ok(value.clone()),
        other => Err(domain(
            name,
            format_args!("expected a string argument, got {other:?}"),
        )),
    }
}

// ---------------------------------------------------------------------------
// Numeric helpers
// ---------------------------------------------------------------------------

fn domain(function: &str, message: std::fmt::Arguments<'_>) -> FreesError {
    FreesError::evaluation(format!("{function}: {message}"))
}

fn check_log(name: &str, x: f64) -> Result<f64> {
    if x == 0.0 {
        return Err(domain(name, format_args!("logarithm of zero")));
    }
    if x < 0.0 {
        return Err(domain(
            name,
            format_args!("logarithm of a negative number ({x})"),
        ));
    }
    Ok(x)
}

fn check_unit_interval(name: &str, x: f64) -> Result<f64> {
    if !(-1.0..=1.0).contains(&x) {
        return Err(domain(
            name,
            format_args!("argument must be in [-1, 1], got {x}"),
        ));
    }
    Ok(x)
}

fn gamma_checked(name: &str, x: f64) -> Result<f64> {
    if x <= 0.0 && x == libm::floor(x) {
        return Err(domain(
            name,
            format_args!("pole at non-positive integer {x}"),
        ));
    }
    Ok(libm::tgamma(x))
}

fn normal_params(name: &str, args: &[f64]) -> Result<(f64, f64)> {
    let mu = if args.len() > 1 { args[1] } else { 0.0 };
    let sigma = if args.len() > 2 { args[2] } else { 1.0 };
    if sigma <= 0.0 {
        return Err(domain(
            name,
            format_args!("standard deviation must be positive, got {sigma}"),
        ));
    }
    Ok((mu, sigma))
}

fn poly_order(name: &str, order: f64) -> Result<usize> {
    if !order.is_finite() {
        return Err(domain(
            name,
            format_args!("polynomial order must be a finite integer, got {order}"),
        ));
    }
    let n = java_round(order);
    if n < 0.0 {
        return Err(domain(
            name,
            format_args!("polynomial order must be >= 0, got {n}"),
        ));
    }
    if n > 100_000.0 {
        return Err(domain(
            name,
            format_args!("polynomial order {n} is too large"),
        ));
    }
    Ok(n as usize)
}

/// `Math.round`: half rounds toward positive infinity (so `round(-2.5) == -2`),
/// unlike Rust's `f64::round`, which rounds half away from zero.
/// Non-finite input is returned unchanged rather than saturating to a `long`.
fn java_round(x: f64) -> f64 {
    if x.is_nan() || x.is_infinite() {
        return x;
    }
    libm::floor(x + 0.5)
}

/// `Math.signum`: `-0.0` and `NaN` are returned unchanged, unlike Rust's
/// `f64::signum`, which maps `-0.0` to `-1.0`.
fn java_signum(x: f64) -> f64 {
    if x.is_nan() {
        f64::NAN
    } else if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        x
    }
}

/// `Math.min`: NaN-propagating, `-0.0 < 0.0`.
fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b {
        a
    } else if b < a {
        b
    } else if a.is_sign_negative() {
        a
    } else {
        b
    }
}

/// `Math.max`: NaN-propagating, `0.0 > -0.0`.
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a > b {
        a
    } else if b > a {
        b
    } else if a.is_sign_positive() {
        a
    } else {
        b
    }
}

/// Java's `%` on doubles: truncated remainder, sign of the dividend.
fn java_mod(name: &str, x: f64, y: f64) -> Result<f64> {
    if y == 0.0 {
        return Err(domain(name, format_args!("division by zero: {x} mod 0")));
    }
    Ok(libm::fmod(x, y))
}

fn gcd_i64(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.unsigned_abs(), b.unsigned_abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a as i64
}

/// `(int) Math.round(x)` widened to i64: NaN → 0, ±∞ saturate — the Java
/// narrowing-cast rules, which Rust's saturating `as` reproduces.
fn java_int(x: f64) -> i64 {
    let rounded = java_round(x);
    if rounded.is_nan() {
        0
    } else {
        rounded as i64
    }
}

// ---------------------------------------------------------------------------
// Special-function kernels (Apache Commons Math + Evaluator.java ports)
// ---------------------------------------------------------------------------

/// Inverse error function — exact port of Apache `Erf.erfInv` (the Giles
/// rational approximations, including the `w = ∞` branch Apache added for
/// `x = ±1`).
fn erf_inv(x: f64) -> f64 {
    // The logarithm argument must stay (1-x)(1+x): simplifying to 1-x² loses
    // accuracy near ±1 (Apache's own comment).
    let mut w = -libm::log((1.0 - x) * (1.0 + x));
    // Late init on purpose: the four-branch cascade below is Apache's own
    // control flow, and each branch mutates `w` before choosing its Horner
    // coefficients. Folding it into a `let p = if ...` expression would
    // restructure a transcribed numerical routine for a lint's benefit.
    #[allow(clippy::needless_late_init)]
    let p;
    if w < 6.25 {
        w -= 3.125;
        p = horner(
            &[
                -3.644_412_064_017_82e-21,
                -1.685_059_138_182_016_6e-19,
                1.285_848_071_525_64e-18,
                1.115_787_767_802_518_1e-17,
                -1.333_171_662_854_621e-16,
                2.097_276_787_596_856_2e-17,
                6.637_638_134_358_324e-15,
                -4.054_566_272_975_207e-14,
                -8.151_934_197_605_472e-14,
                2.633_509_315_308_232_3e-12,
                -1.297_513_325_345_353_2e-11,
                -5.415_412_054_294_628e-11,
                1.051_212_273_321_532_3e-9,
                -4.112_633_980_346_984e-9,
                -2.907_036_995_788_200_5e-8,
                4.234_787_782_793_240_4e-7,
                -1.365_469_200_083_467_9e-6,
                -1.388_252_336_278_646_9e-5,
                1.867_342_080_340_571_4e-4,
                -7.407_025_341_662_67e-4,
                -6.033_670_871_430_149e-3,
                2.401_581_824_255_896_2e-1,
                1.653_654_562_683_102_7,
            ],
            w,
        );
    } else if w < 16.0 {
        w = libm::sqrt(w) - 3.25;
        p = horner(
            &[
                2.213_737_692_177_578_7e-9,
                9.075_656_193_888_539e-8,
                -2.751_740_629_706_454_4e-7,
                1.823_962_921_438_922_8e-8,
                1.502_740_396_890_982_8e-6,
                -4.013_867_526_981_546e-6,
                2.923_444_908_995_544_6e-6,
                1.247_530_448_167_177_9e-5,
                -4.731_822_900_905_573_4e-5,
                6.828_485_145_957_318e-5,
                2.403_111_038_709_789_4e-5,
                -3.550_375_203_628_475e-4,
                9.532_893_797_373_805e-4,
                -1.688_275_556_023_504_7e-3,
                2.491_442_096_107_851e-3,
                -3.751_208_507_569_241e-3,
                5.370_914_553_590_064e-3,
                1.005_258_967_694_159_2,
                3.083_885_610_492_220_8,
            ],
            w,
        );
    } else if !w.is_infinite() {
        w = libm::sqrt(w) - 5.0;
        p = horner(
            &[
                -2.710_992_061_643_857_3e-11,
                -2.555_641_816_996_525e-10,
                1.507_657_269_350_054_8e-9,
                -3.789_465_440_126_737e-9,
                7.615_701_208_078_34e-9,
                -1.496_002_662_714_924e-8,
                2.914_795_345_090_108e-8,
                -6.771_199_775_845_234e-8,
                2.290_048_222_802_665_5e-7,
                -9.929_827_294_231_7e-7,
                4.526_062_597_223_154e-6,
                -1.968_177_810_553_167e-5,
                7.599_527_703_001_776e-5,
                -2.150_301_193_004_447_7e-4,
                -1.387_193_183_362_312_2e-4,
                1.010_300_464_864_534_4,
                4.849_906_401_408_584,
            ],
            w,
        );
    } else {
        // x = ±1: w is +∞; the polynomial would produce −∞ from the negative
        // leading coefficient, so Apache special-cases +∞.
        p = f64::INFINITY;
    }
    p * x
}

/// Evaluate the coefficient chain `((c₀·w + c₁)·w + c₂)…` exactly as the
/// unrolled Apache code does.
fn horner(coefficients: &[f64], w: f64) -> f64 {
    let mut p = coefficients[0];
    for &c in &coefficients[1..] {
        p = c + p * w;
    }
    p
}

/// Euler–Mascheroni constant (Apache `Gamma.GAMMA`).
const EULER_GAMMA: f64 = 0.577_215_664_901_532_9;

/// Digamma ψ(x) — port of Apache `Gamma.digamma` (Bernardo's AS 103), with the
/// tail recursion `ψ(x) = ψ(x+1) − 1/x` unrolled into a loop so a far-negative
/// argument cannot overflow the stack.
fn digamma(x: f64) -> f64 {
    if x.is_nan() || x.is_infinite() {
        return x;
    }
    let mut x = x;
    let mut shift = 0.0;
    loop {
        if x > 0.0 && x <= 1e-5 {
            // Accurate to O(x) for small positive arguments.
            return shift - EULER_GAMMA - 1.0 / x;
        }
        if x >= 49.0 {
            // Asymptotic expansion: log(x) − 1/2x − 1/12x² + 1/120x⁴ − 1/252x⁶.
            let inv = 1.0 / (x * x);
            return shift + libm::log(x)
                - 0.5 / x
                - inv * (1.0 / 12.0 + inv * (1.0 / 120.0 - inv / 252.0));
        }
        shift -= 1.0 / x;
        x += 1.0;
    }
}

/// Iteration cap for the gamma series / continued fraction. Apache runs to
/// `Integer.MAX_VALUE`; both converge in tens of iterations for any argument
/// that matters, and a browser engine must not hang, so this port caps far
/// lower and errors past it.
const GAMMA_MAX_ITERATIONS: usize = 10_000_000;
const GAMMA_EPSILON: f64 = 1e-14;

/// Regularized lower incomplete gamma P(a, x) — port of Apache
/// `Gamma.regularizedGammaP` (series form, deferring to Q when it converges
/// faster).
fn regularized_gamma_p(a: f64, x: f64) -> Result<f64> {
    if a.is_nan() || x.is_nan() || a <= 0.0 || x < 0.0 {
        return Ok(f64::NAN);
    }
    if x == 0.0 {
        return Ok(0.0);
    }
    if x >= a + 1.0 {
        return Ok(1.0 - regularized_gamma_q(a, x)?);
    }
    let mut n = 0.0_f64;
    let mut an = 1.0 / a;
    let mut sum = an;
    while libm::fabs(an / sum) > GAMMA_EPSILON
        && (n as usize) < GAMMA_MAX_ITERATIONS
        && sum < f64::INFINITY
    {
        n += 1.0;
        an *= x / (a + n);
        sum += an;
    }
    if n as usize >= GAMMA_MAX_ITERATIONS {
        return Err(FreesError::evaluation(
            "regularized gamma P failed to converge",
        ));
    }
    if sum.is_infinite() {
        return Ok(1.0);
    }
    Ok(libm::exp(-x + a * libm::log(x) - libm::lgamma(a)) * sum)
}

/// Regularized upper incomplete gamma Q(a, x) — the continued-fraction form,
/// evaluated with Apache's modified-Lentz `ContinuedFraction.evaluate`.
fn regularized_gamma_q(a: f64, x: f64) -> Result<f64> {
    // a_n = (2n+1) − a + x, b_n = n(a − n).
    let small = 1e-50;
    let coeff_a = |n: f64| (2.0 * n + 1.0) - a + x;
    let coeff_b = |n: f64| n * (a - n);
    let mut h_prev = coeff_a(0.0);
    if libm::fabs(h_prev) <= small {
        h_prev = small;
    }
    let mut d_prev = 0.0_f64;
    let mut c_prev = h_prev;
    let mut h_n = h_prev;
    let mut n = 1usize;
    while n < GAMMA_MAX_ITERATIONS {
        let an = coeff_a(n as f64);
        let bn = coeff_b(n as f64);
        let mut d_n = an + bn * d_prev;
        if libm::fabs(d_n) <= small {
            d_n = small;
        }
        let mut c_n = an + bn / c_prev;
        if libm::fabs(c_n) <= small {
            c_n = small;
        }
        d_n = 1.0 / d_n;
        let delta = c_n * d_n;
        h_n = h_prev * delta;
        if h_n.is_infinite() || h_n.is_nan() {
            return Err(FreesError::evaluation(
                "regularized gamma Q continued fraction diverged",
            ));
        }
        if libm::fabs(delta - 1.0) < GAMMA_EPSILON {
            break;
        }
        d_prev = d_n;
        c_prev = c_n;
        h_prev = h_n;
        n += 1;
    }
    if n >= GAMMA_MAX_ITERATIONS {
        return Err(FreesError::evaluation(
            "regularized gamma Q failed to converge",
        ));
    }
    Ok(libm::exp(-x + a * libm::log(x) - libm::lgamma(a)) / h_n)
}

// --- Regularized incomplete beta I_x(a, b) ---------------------------------
//
// Used by the Student-t and F distribution CDFs.  Implements the same
// continued-fraction algorithm as Apache Commons Math
// `Beta.regularizedBeta` (Lentz's method with the DLMF 8.17.22 fraction).

/// Iteration cap for the beta continued fraction.
const BETA_MAX_ITERATIONS: usize = 10_000;
const BETA_EPSILON: f64 = 1e-14;

/// Regularized incomplete beta function I_x(a, b) = B(x; a, b) / B(a, b).
///
/// Returns the probability P(X ≤ x) for X ~ Beta(a, b).
pub fn regularized_beta(x: f64, a: f64, b: f64) -> Result<f64> {
    if x.is_nan() || a.is_nan() || b.is_nan() {
        return Ok(f64::NAN);
    }
    if x < 0.0 || x > 1.0 || a <= 0.0 || b <= 0.0 {
        return Ok(f64::NAN);
    }
    if x == 0.0 {
        return Ok(0.0);
    }
    if x == 1.0 {
        return Ok(1.0);
    }
    // Use the symmetry relation I_x(a, b) = 1 − I_{1−x}(b, a) to ensure the
    // continued fraction converges (it converges when x < (a + 1) / (a + b + 2)).
    if x > (a + 1.0) / (a + b + 2.0) {
        return Ok(1.0 - regularized_beta(1.0 - x, b, a)?);
    }
    // Log of the prefactor: x^a (1−x)^b / (a * B(a, b))
    let log_prefix = a * libm::log(x) + b * libm::log(1.0 - x) - libm::lgamma(a) - libm::lgamma(b)
        + libm::lgamma(a + b)
        - libm::log(a);
    let prefix = libm::exp(log_prefix);
    // Lentz's continued fraction for I_x(a, b).
    // CF coefficients from DLMF 8.17.22.
    let small = 1e-50_f64;
    let mut c = 1.0_f64;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if libm::fabs(d) < small {
        d = small;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=BETA_MAX_ITERATIONS {
        let mf = m as f64;
        // Even step: d_{2m} = m(b − m)x / ((a + 2m − 1)(a + 2m))
        let num_even = mf * (b - mf) * x / ((a + 2.0 * mf - 1.0) * (a + 2.0 * mf));
        d = 1.0 + num_even * d;
        if libm::fabs(d) < small {
            d = small;
        }
        c = 1.0 + num_even / c;
        if libm::fabs(c) < small {
            c = small;
        }
        d = 1.0 / d;
        h *= d * c;
        // Odd step: d_{2m+1} = −(a + m)(a + b + m)x / ((a + 2m)(a + 2m + 1))
        let num_odd = -((a + mf) * (a + b + mf) * x) / ((a + 2.0 * mf) * (a + 2.0 * mf + 1.0));
        d = 1.0 + num_odd * d;
        if libm::fabs(d) < small {
            d = small;
        }
        c = 1.0 + num_odd / c;
        if libm::fabs(c) < small {
            c = small;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if libm::fabs(delta - 1.0) < BETA_EPSILON {
            return Ok(prefix * h);
        }
    }
    Err(FreesError::evaluation(
        "regularized beta continued fraction failed to converge",
    ))
}

// --- Student-t distribution ------------------------------------------------

/// Student-t cumulative distribution function: P(T ≤ x) for T ~ t(df).
///
/// Uses the regularized beta identity:
///   CDF(x) = 1 − ½ I_{df/(df+x²)}(df/2, 1/2)   for x ≥ 0
///   CDF(x) = ½ I_{df/(df+x²)}(df/2, 1/2)         for x < 0
pub fn student_t_cdf(x: f64, df: f64) -> Result<f64> {
    if df <= 0.0 {
        return Err(FreesError::evaluation(format!(
            "tcdf: degrees of freedom must be > 0, got {df}"
        )));
    }
    if x == 0.0 {
        return Ok(0.5);
    }
    let t2 = x * x;
    let z = df / (df + t2);
    let beta_val = regularized_beta(z, df / 2.0, 0.5)?;
    if x > 0.0 {
        Ok(1.0 - 0.5 * beta_val)
    } else {
        Ok(0.5 * beta_val)
    }
}

/// Student-t probability density function.
pub fn student_t_pdf(x: f64, df: f64) -> Result<f64> {
    if df <= 0.0 {
        return Err(FreesError::evaluation(format!(
            "tpdf: degrees of freedom must be > 0, got {df}"
        )));
    }
    let half_df = df / 2.0;
    let log_pdf = libm::lgamma(half_df + 0.5)
        - libm::lgamma(half_df)
        - 0.5 * libm::log(df * std::f64::consts::PI)
        - (half_df + 0.5) * libm::log(1.0 + x * x / df);
    Ok(libm::exp(log_pdf))
}

/// Inverse of the Student-t CDF via Newton–Raphson with a rational initial
/// guess.  Returns the value `t` such that P(T ≤ t) = p for T ~ t(df).
pub fn student_t_inv(p: f64, df: f64) -> Result<f64> {
    if df <= 0.0 {
        return Err(FreesError::evaluation(format!(
            "tinv: degrees of freedom must be > 0, got {df}"
        )));
    }
    if p <= 0.0 {
        return Ok(f64::NEG_INFINITY);
    }
    if p >= 1.0 {
        return Ok(f64::INFINITY);
    }
    if (p - 0.5).abs() < 1e-15 {
        return Ok(0.0);
    }
    // Use the symmetry: if p < 0.5, tinv(p, df) = −tinv(1−p, df).
    if p < 0.5 {
        return Ok(-student_t_inv(1.0 - p, df)?);
    }
    // Initial guess from normal approximation for large df.
    let z = std::f64::consts::SQRT_2 * erf_inv(2.0 * p - 1.0);
    let mut t = if df > 100.0 {
        z
    } else {
        // Cornish–Fisher expansion for moderate df.
        let z2 = z * z;
        z + (z2 * z + z) / (4.0 * df)
            + (5.0 * z2 * z2 * z + 16.0 * z2 * z + 3.0 * z) / (96.0 * df * df)
    };
    // Newton–Raphson refinement.
    for _ in 0..50 {
        let cdf = student_t_cdf(t, df)?;
        let pdf = student_t_pdf(t, df)?;
        if pdf < 1e-300 {
            break;
        }
        let dt = (cdf - p) / pdf;
        t -= dt;
        if libm::fabs(dt) < 1e-14 * libm::fabs(t).max(1.0) {
            break;
        }
    }
    Ok(t)
}

// --- F distribution --------------------------------------------------------

/// F distribution CDF: P(X ≤ x) for X ~ F(d1, d2).
///
/// Uses the regularized beta identity:
///   CDF(x) = I_{d1·x/(d1·x+d2)}(d1/2, d2/2)
pub fn f_dist_cdf(x: f64, d1: f64, d2: f64) -> Result<f64> {
    if d1 <= 0.0 || d2 <= 0.0 {
        return Err(FreesError::evaluation(format!(
            "fcdf: degrees of freedom must be > 0, got d1={d1}, d2={d2}"
        )));
    }
    if x <= 0.0 {
        return Ok(0.0);
    }
    let z = d1 * x / (d1 * x + d2);
    regularized_beta(z, d1 / 2.0, d2 / 2.0)
}

/// F distribution probability density function.
pub fn f_dist_pdf(x: f64, d1: f64, d2: f64) -> Result<f64> {
    if d1 <= 0.0 || d2 <= 0.0 {
        return Err(FreesError::evaluation(format!(
            "fpdf: degrees of freedom must be > 0, got d1={d1}, d2={d2}"
        )));
    }
    if x <= 0.0 {
        return Ok(0.0);
    }
    let log_pdf = (d1 / 2.0) * libm::log(d1 / d2) + (d1 / 2.0 - 1.0) * libm::log(x)
        - ((d1 + d2) / 2.0) * libm::log(1.0 + d1 * x / d2)
        - libm::lgamma(d1 / 2.0)
        - libm::lgamma(d2 / 2.0)
        + libm::lgamma((d1 + d2) / 2.0);
    Ok(libm::exp(log_pdf))
}

/// Inverse of the F distribution CDF via Newton–Raphson.
/// Returns `x` such that P(X ≤ x) = p for X ~ F(d1, d2).
pub fn f_dist_inv(p: f64, d1: f64, d2: f64) -> Result<f64> {
    if d1 <= 0.0 || d2 <= 0.0 {
        return Err(FreesError::evaluation(format!(
            "finv: degrees of freedom must be > 0, got d1={d1}, d2={d2}"
        )));
    }
    if p <= 0.0 {
        return Ok(0.0);
    }
    if p >= 1.0 {
        return Ok(f64::INFINITY);
    }
    // Initial guess: use the mean of the F distribution (d2 / (d2 − 2)) scaled
    // by a normal quantile adjustment.
    let z = std::f64::consts::SQRT_2 * erf_inv(2.0 * p - 1.0);
    let mean_f = if d2 > 2.0 { d2 / (d2 - 2.0) } else { 1.0 };
    let mut x = mean_f * libm::exp(z * 0.5);
    if x <= 0.0 {
        x = 0.01;
    }
    // Newton–Raphson.
    for _ in 0..100 {
        let cdf = f_dist_cdf(x, d1, d2)?;
        let pdf = f_dist_pdf(x, d1, d2)?;
        if pdf < 1e-300 {
            break;
        }
        let dx = (cdf - p) / pdf;
        x -= dx;
        if x <= 0.0 {
            x = 1e-10;
        }
        if libm::fabs(dx) < 1e-14 * x.max(1.0) {
            break;
        }
    }
    Ok(x)
}

// --- Bessel functions (Numerical-Recipes transcriptions from Evaluator.java) —

fn bessj0(x: f64) -> f64 {
    let ax = libm::fabs(x);
    if ax < 8.0 {
        let y = x * x;
        let ans1 = 57568490574.0
            + y * (-13362590354.0
                + y * (651619640.7 + y * (-11214424.18 + y * (77392.33017 + y * (-184.9052456)))));
        let ans2 = 57568490411.0
            + y * (1029532985.0
                + y * (9494680.718 + y * (59272.64853 + y * (267.8532712 + y * 1.0))));
        ans1 / ans2
    } else {
        let z = 8.0 / ax;
        let y = z * z;
        let xx = ax - 0.785398164;
        let ans1 = 1.0
            + y * (-0.1098628627e-2
                + y * (0.2734510407e-4 + y * (-0.2073370639e-5 + y * 0.2093887211e-6)));
        let ans2 = -0.1562499995e-1
            + y * (0.1430488765e-3
                + y * (-0.6911147651e-5 + y * (0.7621095161e-6 - y * 0.934935152e-7)));
        libm::sqrt(0.636619772 / ax) * (libm::cos(xx) * ans1 - z * libm::sin(xx) * ans2)
    }
}

fn bessj1(x: f64) -> f64 {
    let ax = libm::fabs(x);
    if ax < 8.0 {
        let y = x * x;
        let ans1 = x
            * (72362614232.0
                + y * (-7895059235.0
                    + y * (242396853.1
                        + y * (-2972611.439 + y * (15704.48260 + y * (-30.16036606))))));
        let ans2 = 144725228442.0
            + y * (2300535178.0
                + y * (18583304.74 + y * (99447.43394 + y * (376.9991397 + y * 1.0))));
        ans1 / ans2
    } else {
        let z = 8.0 / ax;
        let y = z * z;
        let xx = ax - 2.356194491;
        let ans1 = 1.0
            + y * (0.183105e-2
                + y * (-0.3516396496e-4 + y * (0.2457520174e-5 + y * (-0.240337019e-6))));
        let ans2 = 0.04687499995
            + y * (-0.2002690873e-3
                + y * (0.8449199096e-5 + y * (-0.88228987e-6 + y * 0.105787412e-6)));
        let mut ans =
            libm::sqrt(0.636619772 / ax) * (libm::cos(xx) * ans1 - z * libm::sin(xx) * ans2);
        if x < 0.0 {
            ans = -ans;
        }
        ans
    }
}

fn bessy0(name: &str, x: f64) -> Result<f64> {
    if x <= 0.0 {
        return Err(domain(
            name,
            format_args!("BesselY0 requires x > 0, got {x}"),
        ));
    }
    Ok(if x < 8.0 {
        let y = x * x;
        let ans1 = -2957821389.0
            + y * (7062834065.0
                + y * (-512359803.6 + y * (10879881.29 + y * (-86327.92757 + y * 228.4622733))));
        let ans2 = 40076544269.0
            + y * (745249964.8
                + y * (7189466.438 + y * (47447.26470 + y * (226.1030244 + y * 1.0))));
        (ans1 / ans2) + 0.636619772 * bessj0(x) * libm::log(x)
    } else {
        let z = 8.0 / x;
        let y = z * z;
        let xx = x - 0.785398164;
        let ans1 = 1.0
            + y * (-0.1098628627e-2
                + y * (0.2734510407e-4 + y * (-0.2073370639e-5 + y * 0.2093887211e-6)));
        let ans2 = -0.1562499995e-1
            + y * (0.1430488765e-3
                + y * (-0.6911147651e-5 + y * (0.7621095161e-6 + y * (-0.934945152e-7))));
        libm::sqrt(0.636619772 / x) * (libm::sin(xx) * ans1 + z * libm::cos(xx) * ans2)
    })
}

fn bessy1(name: &str, x: f64) -> Result<f64> {
    if x <= 0.0 {
        return Err(domain(
            name,
            format_args!("BesselY1 requires x > 0, got {x}"),
        ));
    }
    Ok(if x < 8.0 {
        let y = x * x;
        let ans1 = x
            * (-0.4900604943e13
                + y * (0.1275274390e13
                    + y * (-0.5153438139e11
                        + y * (0.7349264551e9 + y * (-0.4237922726e7 + y * 0.8511937935e4)))));
        let ans2 = 0.2499580570e14
            + y * (0.4244419664e12
                + y * (0.3733650367e10
                    + y * (0.2245904002e8 + y * (0.1020426050e6 + y * (0.3549632885e3 + y)))));
        (ans1 / ans2) + 0.636619772 * (bessj1(x) * libm::log(x) - 1.0 / x)
    } else {
        let z = 8.0 / x;
        let y = z * z;
        let xx = x - 2.356194491;
        let ans1 = 1.0
            + y * (0.183105e-2
                + y * (-0.3516396496e-4 + y * (0.2457520174e-5 + y * (-0.240337019e-6))));
        let ans2 = 0.04687499995
            + y * (-0.2002690873e-3
                + y * (0.8449199096e-5 + y * (-0.88228987e-6 + y * 0.105787412e-6)));
        libm::sqrt(0.636619772 / x) * (libm::sin(xx) * ans1 + z * libm::cos(xx) * ans2)
    })
}

fn bessi0(x: f64) -> f64 {
    let ax = libm::fabs(x);
    if ax < 3.75 {
        let mut y = x / 3.75;
        y *= y;
        1.0 + y
            * (3.5156229
                + y * (3.0899424
                    + y * (1.2067492 + y * (0.2659732 + y * (0.360768e-1 + y * 0.45813e-2)))))
    } else {
        let y = 3.75 / ax;
        (libm::exp(ax) / libm::sqrt(ax))
            * (0.39894228
                + y * (0.1328592e-1
                    + y * (0.225319e-2
                        + y * (-0.157565e-2
                            + y * (0.916281e-2
                                + y * (-0.2057706e-1
                                    + y * (0.2635537e-1
                                        + y * (-0.1647633e-1 + y * 0.392377e-2))))))))
    }
}

fn bessi1(x: f64) -> f64 {
    let ax = libm::fabs(x);
    let ans = if ax < 3.75 {
        let mut y = x / 3.75;
        y *= y;
        ax * (0.5
            + y * (0.87890594
                + y * (0.51498869
                    + y * (0.15084934 + y * (0.2658733e-1 + y * (0.301532e-2 + y * 0.32411e-3))))))
    } else {
        let y = 3.75 / ax;
        let mut ans = 0.2282967e-1 + y * (-0.2895312e-1 + y * (0.1787654e-1 - y * 0.420059e-2));
        ans = 0.39894228
            + y * (-0.3988024e-1
                + y * (-0.362018e-2 + y * (0.163801e-2 + y * (-0.1031555e-1 + y * ans))));
        ans * (libm::exp(ax) / libm::sqrt(ax))
    };
    if x < 0.0 {
        -ans
    } else {
        ans
    }
}

fn bessk0(name: &str, x: f64) -> Result<f64> {
    if x <= 0.0 {
        return Err(domain(
            name,
            format_args!("BesselK0 requires x > 0, got {x}"),
        ));
    }
    Ok(if x <= 2.0 {
        let y = x * x / 4.0;
        (-libm::log(x / 2.0) * bessi0(x))
            + (-0.57721566
                + y * (0.42278420
                    + y * (0.23069756
                        + y * (0.3488590e-1 + y * (0.262698e-2 + y * (0.10750e-3 + y * 0.74e-5))))))
    } else {
        let y = 2.0 / x;
        (libm::exp(-x) / libm::sqrt(x))
            * (1.25331414
                + y * (-0.7832358e-1
                    + y * (0.2189568e-1
                        + y * (-0.1062446e-1
                            + y * (0.587872e-2 + y * (-0.251540e-2 + y * 0.53208e-3))))))
    })
}

fn bessk1(name: &str, x: f64) -> Result<f64> {
    if x <= 0.0 {
        return Err(domain(
            name,
            format_args!("BesselK1 requires x > 0, got {x}"),
        ));
    }
    Ok(if x <= 2.0 {
        let y = x * x / 4.0;
        (libm::log(x / 2.0) * bessi1(x))
            + (1.0 / x)
                * (1.0
                    + y * (0.15443144
                        + y * (-0.67278579
                            + y * (-0.18156897
                                + y * (-0.1919402e-1 + y * (-0.110404e-2 + y * (-0.4686e-4)))))))
    } else {
        let y = 2.0 / x;
        (libm::exp(-x) / libm::sqrt(x))
            * (1.25331414
                + y * (0.23498619
                    + y * (-0.3655620e-1
                        + y * (0.1504268e-1
                            + y * (-0.780353e-2 + y * (0.325614e-2 + y * (-0.68245e-3)))))))
    })
}

/// Yₙ(x) by upward recurrence from Y₀/Y₁.
fn bessy_n(name: &str, n: i64, x: f64) -> Result<f64> {
    if x <= 0.0 {
        return Err(domain(
            name,
            format_args!("BesselY requires x > 0, got {x}"),
        ));
    }
    if n < 0 {
        let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
        return Ok(sign * bessy_n(name, -n, x)?);
    }
    if n == 0 {
        return bessy0(name, x);
    }
    if n == 1 {
        return bessy1(name, x);
    }
    let tox = 2.0 / x;
    let mut by = bessy1(name, x)?;
    let mut bym = bessy0(name, x)?;
    for j in 1..n {
        let byp = j as f64 * tox * by - bym;
        bym = by;
        by = byp;
    }
    Ok(by)
}

/// Kₙ(x) by upward recurrence from K₀/K₁ (`K₋ₙ = Kₙ`).
fn bessk_n(name: &str, n: i64, x: f64) -> Result<f64> {
    if x <= 0.0 {
        return Err(domain(
            name,
            format_args!("BesselK requires x > 0, got {x}"),
        ));
    }
    if n < 0 {
        return bessk_n(name, -n, x);
    }
    if n == 0 {
        return bessk0(name, x);
    }
    if n == 1 {
        return bessk1(name, x);
    }
    let tox = 2.0 / x;
    let mut bkm = bessk0(name, x)?;
    let mut bk = bessk1(name, x)?;
    for j in 1..n {
        let bkp = bkm + j as f64 * tox * bk;
        bkm = bk;
        bk = bkp;
    }
    Ok(bk)
}

/// `Evaluator.besselY`: integer orders only.
fn bessel_y(name: &str, order: f64, x: f64) -> Result<f64> {
    if order != libm::rint(order) {
        return Err(domain(
            name,
            format_args!("BesselY requires an integer order, got {order}"),
        ));
    }
    bessy_n(name, java_int(order), x)
}

/// `Evaluator.besselK`: integer orders only.
fn bessel_k(name: &str, order: f64, x: f64) -> Result<f64> {
    if order != libm::rint(order) {
        return Err(domain(
            name,
            format_args!("BesselK requires an integer order, got {order}"),
        ));
    }
    bessk_n(name, java_int(order), x)
}

/// Modified Bessel I of the first kind, real order — `Evaluator.besselI`'s
/// overflow-safe log-space ascending series.
fn bessel_i(name: &str, order: f64, x: f64) -> Result<f64> {
    if x < 0.0 {
        if order != libm::rint(order) {
            return Err(domain(
                name,
                format_args!("BesselI of a negative argument requires an integer order"),
            ));
        }
        let sign = if (libm::rint(order) as i64) % 2 == 0 {
            1.0
        } else {
            -1.0
        };
        return Ok(sign * bessel_i(name, order, -x)?);
    }
    let mut order = order;
    if order < 0.0 {
        if order != libm::rint(order) {
            return Err(domain(
                name,
                format_args!("BesselI supports integer or non-negative orders, got {order}"),
            ));
        }
        order = -order; // I₋ₙ(x) = Iₙ(x) for integer n
    }
    if x == 0.0 {
        return Ok(if order == 0.0 { 1.0 } else { 0.0 });
    }
    let ln_half_x = libm::log(x / 2.0);
    let mut sum = 0.0;
    for k in 0..2000 {
        let kf = k as f64;
        let ln_term = (2.0 * kf + order) * ln_half_x
            - libm::lgamma(kf + 1.0)
            - libm::lgamma(kf + order + 1.0);
        let term = libm::exp(ln_term);
        sum += term;
        if term < sum * 1e-17 && kf > x / 2.0 {
            break;
        }
    }
    Ok(sum)
}

/// Bessel J of arbitrary real order — port of Apache `BesselJ.value`, which
/// wraps Cody's RJBESL ([`rj_besl`]). The Java arm reads the order from the
/// *second* argument at the call site.
fn bessel_j(name: &str, order: f64, x: f64) -> Result<f64> {
    let n = order as i32; // Java (int) cast: truncate toward zero, saturating
    let alpha = order - n as f64;
    if n < 0 {
        // nb = n + 1 <= 0: rjBesl reports the argument error path.
        return Err(domain(
            name,
            format_args!("Bessel function of order {order} cannot be computed for x = {x}"),
        ));
    }
    let nb = n as usize + 1;
    let (vals, ncalc) = rj_besl(x, alpha, nb);
    if ncalc >= nb as isize {
        Ok(vals[n as usize])
    } else if ncalc < 0 {
        Err(domain(
            name,
            format_args!("Bessel function of order {order} cannot be computed for x = {x}"),
        ))
    } else if ncalc >= 1 && libm::fabs(vals[(ncalc - 1) as usize]) < 1e-100 {
        Ok(vals[n as usize]) // underflow; value is zero
    } else {
        Err(domain(
            name,
            format_args!("Bessel function of order {order} failed to converge for x = {x}"),
        ))
    }
}

/// Cody's RJBESL exactly as Apache Commons Math 3.6.1 transcribes it —
/// including Apache's dead `ncalc` store in the overflow branch, which
/// diverges from the original Fortran and is kept for bug-compatibility.
/// Returns the sequence `J_{alpha+k}(x)` for `k = 0..nb-1` and the count of
/// values computed to full precision.
#[allow(clippy::needless_range_loop)]
fn rj_besl(x: f64, alpha: f64, nb: usize) -> (Vec<f64>, isize) {
    const PI2: f64 = 0.636_619_772_367_581_3;
    const TWOPI1: f64 = 6.28125;
    const TWOPI2: f64 = 1.935_307_179_586_476_9e-3;
    const TWOPI: f64 = TWOPI1 + TWOPI2;
    const ENTEN: f64 = 1.0e308;
    const ENSIG: f64 = 1.0e16;
    const RTNSIG: f64 = 1.0e-4;
    const ENMTEN: f64 = 8.90e-308;
    const X_MIN: f64 = 0.0;
    const X_MAX: f64 = 1.0e4;
    const FACT: [f64; 25] = [
        1.0,
        1.0,
        2.0,
        6.0,
        24.0,
        120.0,
        720.0,
        5040.0,
        40320.0,
        362880.0,
        3628800.0,
        39916800.0,
        479001600.0,
        6227020800.0,
        87178291200.0,
        1.307674368e12,
        2.0922789888e13,
        3.55687428096e14,
        6.402373705728e15,
        1.21645100408832e17,
        2.43290200817664e18,
        5.109094217170944e19,
        1.124_000_727_777_607_7e21,
        2.585_201_673_888_498e22,
        6.204_484_017_332_394e23,
    ];

    let mut b = vec![0.0_f64; nb];
    // Late init on purpose: `ncalc` is settled by one of several deeply
    // nested branches below, mirroring the reference implementation. An
    // expression form would mean restructuring the whole routine.
    #[allow(clippy::needless_late_init)]
    let ncalc: isize;
    let magx = x as i32;
    if nb > 0 && (X_MIN..=X_MAX).contains(&x) && (0.0..1.0).contains(&alpha) {
        let mut ncalc_v = nb as isize;
        let mut tempa;
        let mut tempb;
        let mut tempc;
        let mut tover;
        let mut alpem;
        let mut alp2em;
        if x < RTNSIG {
            // Two-term ascending series for small x.
            tempa = 1.0;
            alpem = 1.0 + alpha;
            let mut halfx = 0.0;
            if x > ENMTEN {
                halfx = 0.5 * x;
            }
            if alpha != 0.0 {
                tempa = libm::pow(halfx, alpha) / (alpha * libm::tgamma(alpha));
            }
            tempb = 0.0;
            if x + 1.0 > 1.0 {
                tempb = -halfx * halfx;
            }
            b[0] = tempa + (tempa * tempb / alpem);
            if x != 0.0 && b[0] == 0.0 {
                ncalc_v = 0;
            }
            if nb != 1 {
                if x <= 0.0 {
                    for slot in b.iter_mut().take(nb).skip(1) {
                        *slot = 0.0;
                    }
                } else {
                    // Higher-order functions.
                    tempc = halfx;
                    tover = if tempb != 0.0 {
                        ENMTEN / tempb
                    } else {
                        2.0 * ENMTEN / x
                    };
                    for n in 1..nb {
                        tempa /= alpem;
                        alpem += 1.0;
                        tempa *= tempc;
                        if tempa <= tover * alpem {
                            tempa = 0.0;
                        }
                        b[n] = tempa + (tempa * tempb / alpem);
                        if b[n] == 0.0 && ncalc_v > n as isize {
                            ncalc_v = n as isize;
                        }
                    }
                }
            }
        } else if x > 25.0 && nb as i32 <= magx + 1 {
            // Asymptotic series for x > 25.
            let xc = libm::sqrt(PI2 / x);
            let mul = 0.125 / x;
            let xin = mul * mul;
            let m: i32 = if x >= 130.0 {
                4
            } else if x >= 35.0 {
                8
            } else {
                11
            };
            let xm = 4.0 * m as f64;
            // Argument reduction for sin and cos.
            let mut t = ((x / TWOPI) + 0.5) as i64 as f64;
            let z = x - t * TWOPI1 - t * TWOPI2 - (alpha + 0.5) / PI2;
            let mut vsin = libm::sin(z);
            let mut vcos = libm::cos(z);
            let mut gnu = 2.0 * alpha;
            let mut capp;
            let mut capq;
            let mut s;
            let mut t1;
            let mut xk;
            for i in 1..=2usize {
                s = (xm - 1.0 - gnu) * (xm - 1.0 + gnu) * xin * 0.5;
                t = (gnu - (xm - 3.0)) * (gnu + (xm - 3.0));
                capp = (s * t) / FACT[(2 * m) as usize];
                t1 = (gnu - (xm + 1.0)) * (gnu + (xm + 1.0));
                capq = (s * t1) / FACT[(2 * m + 1) as usize];
                xk = xm;
                let mut k = 2 * m;
                t1 = t;
                for _j in 2..=m {
                    xk -= 4.0;
                    s = (xk - 1.0 - gnu) * (xk - 1.0 + gnu);
                    t = (gnu - (xk - 3.0)) * (gnu + (xk - 3.0));
                    capp = (capp + 1.0 / FACT[(k - 2) as usize]) * s * t * xin;
                    capq = (capq + 1.0 / FACT[(k - 1) as usize]) * s * t1 * xin;
                    k -= 2;
                    t1 = t;
                }
                capp += 1.0;
                capq = (capq + 1.0) * ((gnu * gnu) - 1.0) * (0.125 / x);
                b[i - 1] = xc * (capp * vcos - capq * vsin);
                if nb == 1 {
                    return (b, ncalc_v);
                }
                t = vsin;
                vsin = -vcos;
                vcos = t;
                gnu += 2.0;
            }
            // If nb > 2, recur upward.
            if nb > 2 {
                gnu = 2.0 * alpha + 2.0;
                for j in 2..nb {
                    b[j] = gnu * b[j - 1] / x - b[j - 2];
                    gnu += 2.0;
                }
            }
        } else {
            // Backward recurrence.
            let nbmx = nb as i32 - magx;
            let mut n = magx + 1;
            let mut nstart;
            let mut nend;
            let mut en = 2.0 * (n as f64 + alpha);
            let mut plast = 1.0;
            let mut p = en / x;
            let mut pold;
            let mut test = 2.0 * ENSIG;
            let mut ready_to_initialize = false;
            if nbmx >= 3 {
                // Calculate p*s until n = nb-1, watching for overflow.
                tover = ENTEN / ENSIG;
                nstart = magx + 2;
                nend = nb as i32 - 1;
                en = 2.0 * ((nstart - 1) as f64 + alpha);
                let mut psave;
                let mut psavel;
                for k in nstart..=nend {
                    n = k;
                    en += 2.0;
                    pold = plast;
                    plast = p;
                    p = (en * plast / x) - pold;
                    if p > tover {
                        // Divide p*s by tover; iterate until abs(p) > 1.
                        tover = ENTEN;
                        p /= tover;
                        plast /= tover;
                        psave = p;
                        psavel = plast;
                        nstart = n + 1;
                        loop {
                            n += 1;
                            en += 2.0;
                            pold = plast;
                            plast = p;
                            p = (en * plast / x) - pold;
                            if p > 1.0 {
                                break;
                            }
                        }
                        tempb = en / x;
                        // Backward test; Apache assigns ncalc inside this scan
                        // and then unconditionally overwrites it with nend
                        // below (its divergence from the Fortran original), so
                        // the scan result is discarded here exactly the same.
                        test = pold * plast * (0.5 - 0.5 / (tempb * tempb));
                        test /= ENSIG;
                        p = plast * tover;
                        n -= 1;
                        en -= 2.0;
                        nend = std::cmp::min(nb as i32, n);
                        for _l in nstart..=nend {
                            pold = psavel;
                            psavel = psave;
                            psave = (en * psavel / x) - pold;
                            if psave * psavel > test {
                                break;
                            }
                        }
                        ncalc_v = nend as isize;
                        ready_to_initialize = true;
                        break;
                    }
                }
                if !ready_to_initialize {
                    n = nend;
                    en = 2.0 * (n as f64 + alpha);
                    // Special significance test for nbmx > 2.
                    test = java_max(test, libm::sqrt(plast * ENSIG) * libm::sqrt(2.0 * p));
                }
            }
            // Calculate p*s until the significance test passes.
            if !ready_to_initialize {
                loop {
                    n += 1;
                    en += 2.0;
                    pold = plast;
                    plast = p;
                    p = (en * plast / x) - pold;
                    if p >= test {
                        break;
                    }
                }
            }
            // Initialize the backward recursion and the normalization sum.
            n += 1;
            en += 2.0;
            tempb = 0.0;
            tempa = 1.0 / p;
            let mut m = (2 * n) - 4 * (n / 2);
            let mut sum = 0.0;
            let mut em = (n / 2) as f64;
            alpem = em - 1.0 + alpha;
            alp2em = 2.0 * em + alpha;
            if m != 0 {
                sum = tempa * alpem * alp2em / em;
            }
            let mut nend2 = n - nb as i32;
            let mut ready_to_normalize = false;
            let mut calculated_b0 = false;
            // Recur backward (without storing) until n = nb.
            for _l in 1..=nend2 {
                n -= 1;
                en -= 2.0;
                tempc = tempb;
                tempb = tempa;
                tempa = (en * tempb / x) - tempc;
                m = 2 - m;
                if m != 0 {
                    em -= 1.0;
                    alp2em = 2.0 * em + alpha;
                    if n == 1 {
                        break;
                    }
                    alpem = em - 1.0 + alpha;
                    if alpem == 0.0 {
                        alpem = 1.0;
                    }
                    sum = (sum + tempa * alp2em) * alpem / em;
                }
            }
            // Store b[nb-1].
            b[(n - 1) as usize] = tempa;
            if nend2 >= 0 {
                if nb <= 1 {
                    alp2em = alpha;
                    if alpha + 1.0 == 1.0 {
                        alp2em = 1.0;
                    }
                    sum += b[0] * alp2em;
                    ready_to_normalize = true;
                } else {
                    // Calculate and store b[nb-2].
                    n -= 1;
                    en -= 2.0;
                    b[(n - 1) as usize] = (en * tempa / x) - tempb;
                    if n == 1 {
                        calculated_b0 = true;
                    } else {
                        m = 2 - m;
                        if m != 0 {
                            em -= 1.0;
                            alp2em = 2.0 * em + alpha;
                            alpem = em - 1.0 + alpha;
                            if alpem == 0.0 {
                                alpem = 1.0;
                            }
                            sum = (sum + (b[(n - 1) as usize] * alp2em)) * alpem / em;
                        }
                    }
                }
            }
            if !ready_to_normalize && !calculated_b0 {
                nend2 = n - 2;
                if nend2 != 0 {
                    // Recur downward storing b[n] until n = 2.
                    for _l in 1..=nend2 {
                        n -= 1;
                        en -= 2.0;
                        b[(n - 1) as usize] = (en * b[n as usize] / x) - b[(n + 1) as usize];
                        m = 2 - m;
                        if m != 0 {
                            em -= 1.0;
                            alp2em = 2.0 * em + alpha;
                            alpem = em - 1.0 + alpha;
                            if alpem == 0.0 {
                                alpem = 1.0;
                            }
                            sum = (sum + b[(n - 1) as usize] * alp2em) * alpem / em;
                        }
                    }
                }
            }
            // Calculate b[0].
            if !ready_to_normalize {
                if !calculated_b0 {
                    b[0] = 2.0 * (alpha + 1.0) * b[1] / x - b[2];
                }
                em -= 1.0;
                alp2em = 2.0 * em + alpha;
                if alp2em == 0.0 {
                    alp2em = 1.0;
                }
                sum += b[0] * alp2em;
            }
            // Normalize: divide all b[n] by sum.
            if libm::fabs(alpha) > 1e-16 {
                sum *= libm::tgamma(alpha) * libm::pow(x * 0.5, -alpha);
            }
            tempa = ENMTEN;
            if sum > 1.0 {
                tempa *= sum;
            }
            for slot in b.iter_mut().take(nb) {
                if libm::fabs(*slot) < tempa {
                    *slot = 0.0;
                }
                *slot /= sum;
            }
        }
        ncalc = ncalc_v;
    } else {
        // Error return: x, nb or alpha out of range.
        if !b.is_empty() {
            b[0] = 0.0;
        }
        ncalc = std::cmp::min(nb as isize, 0) - 1;
    }
    (b, ncalc)
}

// --- java.util.Random (for the seeded Random/RandG forms) -------------------

/// Bit-exact `java.util.Random`: the 48-bit LCG, `nextDouble`, and the
/// Marsaglia-polar `nextGaussian`.
struct JavaRandom {
    seed: u64,
}

impl JavaRandom {
    const MULTIPLIER: u64 = 0x5_DEEC_E66D;
    const INCREMENT: u64 = 0xB;
    const MASK: u64 = (1 << 48) - 1;

    fn new(seed: i64) -> JavaRandom {
        JavaRandom {
            seed: (seed as u64 ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    fn next(&mut self, bits: u32) -> i64 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::INCREMENT)
            & Self::MASK;
        (self.seed >> (48 - bits)) as i64
    }

    fn next_double(&mut self) -> f64 {
        const DOUBLE_UNIT: f64 = 1.0 / ((1u64 << 53) as f64);
        (((self.next(26) << 27) + self.next(27)) as f64) * DOUBLE_UNIT
    }

    fn next_gaussian(&mut self) -> f64 {
        loop {
            let v1 = 2.0 * self.next_double() - 1.0;
            let v2 = 2.0 * self.next_double() - 1.0;
            let s = v1 * v1 + v2 * v2;
            if s < 1.0 && s != 0.0 {
                let multiplier = libm::sqrt(-2.0 * libm::log(s) / s);
                return v1 * multiplier;
            }
        }
    }
}

/// The explicit seed for `Random`/`RandG`. Java's seedless forms draw the seed
/// from `System.identityHashCode` — unreproducible, so they are refused here.
fn random_seed(name: &str, args: &[f64]) -> Result<i64> {
    if args.len() < 3 {
        return Err(domain(
            name,
            format_args!(
                "without a seed the Java engine draws from object identity, which is \
                 not reproducible; pass an explicit non-zero seed as the third argument"
            ),
        ));
    }
    let seed = args[2] as i64; // Java (long) cast: truncating, saturating, NaN → 0
    if seed == 0 {
        return Err(domain(
            name,
            format_args!("seed 0 falls back to object identity in Java; use a non-zero seed"),
        ));
    }
    Ok(seed)
}

// --- BaseConvert ------------------------------------------------------------

/// `BaseConvert(digits, fromBase, toBase)` — port of `Evaluator.evalBaseConvert`.
fn eval_base_convert<'a>(name: &str, args: &'a [Expr], env: &'a Env<'a>) -> Result<f64> {
    let digits: String = match &args[0] {
        Expr::Str(s) => s.trim().to_string(),
        other => {
            let v = eval_in(other, env)?;
            if v != libm::rint(v) || v.is_infinite() {
                return Err(domain(
                    name,
                    format_args!("BaseConvert input must be an integer, got {v}"),
                ));
            }
            format!("{}", v as i64)
        }
    };
    let rint_arg = |v: f64| -> i64 {
        let r = libm::rint(v);
        if r.is_nan() {
            0
        } else {
            r as i64
        }
    };
    let from_base = rint_arg(eval_in(&args[1], env)?);
    let to_base = rint_arg(eval_in(&args[2], env)?);
    if !(2..=36).contains(&from_base) || !(2..=36).contains(&to_base) {
        return Err(domain(
            name,
            format_args!(
                "BaseConvert bases must be between 2 and 36, got {from_base} and {to_base}"
            ),
        ));
    }
    let value = i64::from_str_radix(&digits, from_base as u32).map_err(|_| {
        domain(
            name,
            format_args!("'{digits}' is not a valid base-{from_base} number"),
        )
    })?;
    let converted = long_to_string_radix(value, to_base as u32);
    if !converted
        .strip_prefix('-')
        .unwrap_or(&converted)
        .bytes()
        .all(|b| b.is_ascii_digit())
    {
        return Err(domain(
            name,
            format_args!(
                "BaseConvert result '{}' in base {to_base} contains letter digits and cannot \
                 be represented as a number; use toBase <= 10",
                converted.to_uppercase()
            ),
        ));
    }
    converted
        .parse::<f64>()
        .map_err(|_| domain(name, format_args!("BaseConvert result overflow")))
}

/// `Long.toString(value, radix)`: lowercase digits, leading `-` for negatives.
fn long_to_string_radix(value: i64, radix: u32) -> String {
    if value == 0 {
        return "0".to_string();
    }
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let negative = value < 0;
    let mut magnitude = value.unsigned_abs();
    let mut out = Vec::new();
    while magnitude > 0 {
        out.push(DIGITS[(magnitude % radix as u64) as usize]);
        magnitude /= radix as u64;
    }
    if negative {
        out.push(b'-');
    }
    out.reverse();
    String::from_utf8(out).expect("radix digits are ASCII")
}

// --- Radiation view factors (Howell catalog closed forms) -------------------

/// Perpendicular rectangles sharing a common edge of length `l` (Howell C-14).
fn view_factor_perpendicular(name: &str, w1: f64, w2: f64, l: f64) -> Result<f64> {
    if l <= 0.0 || w1 <= 0.0 || w2 <= 0.0 {
        return Err(domain(
            name,
            format_args!("all dimensions must be positive"),
        ));
    }
    let w = w1 / l;
    let h = w2 / l;
    let w2s = w * w;
    let h2s = h * h;
    let sum = w2s + h2s;
    let term = w * libm::atan(1.0 / w) + h * libm::atan(1.0 / h)
        - libm::sqrt(sum) * libm::atan(1.0 / libm::sqrt(sum));
    let log_arg = ((1.0 + w2s) * (1.0 + h2s) / (1.0 + sum))
        * libm::pow(w2s * (1.0 + sum) / ((1.0 + w2s) * sum), w2s)
        * libm::pow(h2s * (1.0 + sum) / ((1.0 + h2s) * sum), h2s);
    Ok((term + 0.25 * libm::log(log_arg)) / (std::f64::consts::PI * w))
}

/// Identical, directly opposed parallel rectangles `a × b` at distance `l`
/// (Howell C-11).
fn view_factor_parallel_plates(name: &str, a: f64, b: f64, l: f64) -> Result<f64> {
    if l <= 0.0 || a <= 0.0 || b <= 0.0 {
        return Err(domain(
            name,
            format_args!("all dimensions must be positive"),
        ));
    }
    let x = a / l;
    let y = b / l;
    let x2 = x * x;
    let y2 = y * y;
    let term = libm::log(libm::sqrt((1.0 + x2) * (1.0 + y2) / (1.0 + x2 + y2)))
        + x * libm::sqrt(1.0 + y2) * libm::atan(x / libm::sqrt(1.0 + y2))
        + y * libm::sqrt(1.0 + x2) * libm::atan(y / libm::sqrt(1.0 + x2))
        - x * libm::atan(x)
        - y * libm::atan(y);
    Ok((2.0 / (std::f64::consts::PI * x * y)) * term)
}

/// Coaxial parallel disks of radii `r1`, `r2` at distance `l` (Howell C-41).
fn view_factor_coaxial_disks(name: &str, r1: f64, r2: f64, l: f64) -> Result<f64> {
    if l <= 0.0 || r1 <= 0.0 || r2 <= 0.0 {
        return Err(domain(
            name,
            format_args!("all dimensions must be positive"),
        ));
    }
    let big_r1 = r1 / l;
    let big_r2 = r2 / l;
    let s = 1.0 + (1.0 + big_r2 * big_r2) / (big_r1 * big_r1);
    let ratio = big_r2 / big_r1;
    Ok(0.5 * (s - libm::sqrt(s * s - 4.0 * ratio * ratio)))
}

// ---------------------------------------------------------------------------
// Engineering-correlation kernels (props/* ports)
// ---------------------------------------------------------------------------

/// `HxCorrelations.clip` / the quality clamps used across the props classes.
fn clip(v: f64, a: f64, b: f64) -> f64 {
    if v < a {
        a
    } else if v > b {
        b
    } else {
        v
    }
}

// --- Heisler one-term transient conduction (core/HeislerCharts.java) --------

#[derive(Clone, Copy, PartialEq, Eq)]
enum HeislerGeometry {
    Wall,
    Cylinder,
    Sphere,
}

fn heisler_geometry(spelling: &str) -> Result<HeislerGeometry> {
    match spelling.to_lowercase().as_str() {
        "wall" | "planewall" | "plane" | "slab" => Ok(HeislerGeometry::Wall),
        "cylinder" | "cyl" => Ok(HeislerGeometry::Cylinder),
        "sphere" | "ball" => Ok(HeislerGeometry::Sphere),
        other => Err(FreesError::evaluation(format!(
            "Heisler geometry must be 'wall', 'cylinder' or 'sphere', got '{other}'."
        ))),
    }
}

/// J₀/J₁ by the ascending power series — exact to machine precision for the
/// small arguments the Heisler eigenvalue problem produces (ζ ≤ π), replacing
/// the Apache `BesselJ.value(0|1, ·)` calls with the same function values.
fn bessel_j0_series(x: f64) -> f64 {
    let q = x * x / 4.0;
    let mut term = 1.0;
    let mut sum = 1.0;
    for k in 1..60 {
        term *= -q / ((k * k) as f64);
        sum += term;
        if libm::fabs(term) < 1e-18 * libm::fabs(sum) {
            break;
        }
    }
    sum
}

fn bessel_j1_series(x: f64) -> f64 {
    let q = x * x / 4.0;
    let mut term = 1.0;
    let mut sum = 1.0;
    for k in 1..60 {
        term *= -q / ((k * (k + 1)) as f64);
        sum += term;
        if libm::fabs(term) < 1e-18 * libm::fabs(sum) {
            break;
        }
    }
    sum * x / 2.0
}

/// Eigenvalue-equation residual f(ζ) − Bi for the first root.
fn heisler_residual(geometry: HeislerGeometry, zeta: f64, bi: f64) -> f64 {
    match geometry {
        HeislerGeometry::Wall => zeta * libm::tan(zeta) - bi,
        HeislerGeometry::Cylinder => zeta * bessel_j1_series(zeta) / bessel_j0_series(zeta) - bi,
        HeislerGeometry::Sphere => 1.0 - zeta / libm::tan(zeta) - bi,
    }
}

fn heisler_upper_bound(geometry: HeislerGeometry) -> f64 {
    match geometry {
        HeislerGeometry::Wall => std::f64::consts::PI / 2.0,
        HeislerGeometry::Cylinder => 2.404_825_557_695_773, // first zero of J0
        HeislerGeometry::Sphere => std::f64::consts::PI,
    }
}

fn heisler_first_eigenvalue(geometry: HeislerGeometry, bi: f64) -> f64 {
    let mut lo = 1e-9;
    let mut hi = heisler_upper_bound(geometry) - 1e-9;
    // The residual is monotone increasing from −Bi (at 0⁺) to +∞ (asymptote).
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if heisler_residual(geometry, mid, bi) > 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
        if hi - lo < 1e-12 {
            break;
        }
    }
    0.5 * (lo + hi)
}

fn heisler_coefficient(geometry: HeislerGeometry, zeta: f64) -> f64 {
    match geometry {
        HeislerGeometry::Wall => 4.0 * libm::sin(zeta) / (2.0 * zeta + libm::sin(2.0 * zeta)),
        HeislerGeometry::Cylinder => {
            let j0 = bessel_j0_series(zeta);
            let j1 = bessel_j1_series(zeta);
            (2.0 / zeta) * j1 / (j0 * j0 + j1 * j1)
        }
        HeislerGeometry::Sphere => {
            4.0 * (libm::sin(zeta) - zeta * libm::cos(zeta)) / (2.0 * zeta - libm::sin(2.0 * zeta))
        }
    }
}

fn heisler_spatial(geometry: HeislerGeometry, zeta: f64, x_star: f64) -> f64 {
    match geometry {
        HeislerGeometry::Wall => libm::cos(zeta * x_star),
        HeislerGeometry::Cylinder => bessel_j0_series(zeta * x_star),
        HeislerGeometry::Sphere => {
            if x_star == 0.0 {
                1.0
            } else {
                libm::sin(zeta * x_star) / (zeta * x_star)
            }
        }
    }
}

fn heisler_temperature(geometry: HeislerGeometry, bi: f64, fo: f64, x_star: f64) -> f64 {
    let zeta = heisler_first_eigenvalue(geometry, bi);
    let theta_centre = heisler_coefficient(geometry, zeta) * libm::exp(-zeta * zeta * fo);
    theta_centre * heisler_spatial(geometry, zeta, x_star)
}

fn heisler_heat_ratio(geometry: HeislerGeometry, bi: f64, fo: f64) -> f64 {
    let zeta = heisler_first_eigenvalue(geometry, bi);
    let theta_centre = heisler_coefficient(geometry, zeta) * libm::exp(-zeta * zeta * fo);
    match geometry {
        HeislerGeometry::Wall => 1.0 - theta_centre * libm::sin(zeta) / zeta,
        HeislerGeometry::Cylinder => 1.0 - 2.0 * theta_centre * bessel_j1_series(zeta) / zeta,
        HeislerGeometry::Sphere => {
            1.0 - 3.0 * theta_centre * (libm::sin(zeta) - zeta * libm::cos(zeta))
                / (zeta * zeta * zeta)
        }
    }
}

// --- ISA 1976 standard atmosphere (props/Atmosphere.java) --------------------

const ISA_T0: f64 = 288.15;
const ISA_P0: f64 = 101_325.0;
const ISA_LAPSE: f64 = 0.0065;
const ISA_R_AIR: f64 = 287.058;
const ISA_G0: f64 = 9.80665;
const ISA_H_TROPO: f64 = 11_000.0;
const ISA_T_TROPO: f64 = ISA_T0 - ISA_LAPSE * ISA_H_TROPO; // 216.65 K

fn isa_temperature(alt: f64) -> f64 {
    if alt <= ISA_H_TROPO {
        ISA_T0 - ISA_LAPSE * alt
    } else {
        ISA_T_TROPO
    }
}

fn isa_pressure(alt: f64) -> f64 {
    if alt <= ISA_H_TROPO {
        let t = ISA_T0 - ISA_LAPSE * alt;
        ISA_P0 * libm::pow(t / ISA_T0, ISA_G0 / (ISA_R_AIR * ISA_LAPSE))
    } else {
        let p_tropo = ISA_P0 * libm::pow(ISA_T_TROPO / ISA_T0, ISA_G0 / (ISA_R_AIR * ISA_LAPSE));
        p_tropo * libm::exp(-ISA_G0 * (alt - ISA_H_TROPO) / (ISA_R_AIR * ISA_T_TROPO))
    }
}

// --- Wiebe heat release (props/Engine.java) ---------------------------------

fn wiebe(
    name: &str,
    theta: f64,
    theta0: f64,
    dtheta: f64,
    a: f64,
    m: f64,
    rate: bool,
) -> Result<f64> {
    if !(dtheta > 0.0) {
        return Err(domain(
            name,
            format_args!("combustion duration dtheta must be > 0, got {dtheta}"),
        ));
    }
    if theta <= theta0 {
        return Ok(0.0);
    }
    let xn = (theta - theta0) / dtheta;
    Ok(if rate {
        a * (m + 1.0) / dtheta * libm::pow(xn, m) * libm::exp(-a * libm::pow(xn, m + 1.0))
    } else {
        1.0 - libm::exp(-a * libm::pow(xn, m + 1.0))
    })
}

// --- ISO 6358 pneumatic mass flow (props/Pneumatics.java) -------------------

fn iso6358(name: &str, c: f64, b: f64, p_up: f64, t_up: f64, p_down: f64) -> Result<f64> {
    const RHO_ANR: f64 = 1.185;
    const T_ANR: f64 = 293.15;
    if c < 0.0 {
        return Err(domain(
            name,
            format_args!("sonic conductance C must be >= 0"),
        ));
    }
    if !(0.0..1.0).contains(&b) {
        return Err(domain(
            name,
            format_args!("critical pressure ratio b must be in [0, 1)"),
        ));
    }
    if p_up <= 0.0 || t_up <= 0.0 {
        // A Newton iterate may stray non-physical; no flow rather than an error.
        return Ok(0.0);
    }
    let choked = c * RHO_ANR * p_up * libm::sqrt(T_ANR / t_up);
    let pr = p_down / p_up;
    if pr <= b {
        return Ok(choked);
    }
    if pr >= 1.0 {
        return Ok(0.0);
    }
    let x = (pr - b) / (1.0 - b);
    Ok(choked * libm::sqrt(1.0 - x * x))
}

// --- Flow resistance (props/FlowResistance.java) ----------------------------

fn reynolds(name: &str, rho: f64, velocity: f64, diameter: f64, viscosity: f64) -> Result<f64> {
    if viscosity <= 0.0 {
        return Err(domain(name, format_args!("dynamic viscosity must be > 0")));
    }
    Ok(rho * libm::fabs(velocity) * diameter / viscosity)
}

fn friction_factor(re: f64, relative_roughness: f64) -> f64 {
    let re = if re <= 1.0e-6 { 1.0e-6 } else { re };
    let laminar = 64.0 / re;
    if re < 2300.0 {
        return laminar;
    }
    let turbulent = colebrook(java_max(re, 4000.0), relative_roughness);
    if re < 4000.0 {
        let t = (re - 2300.0) / (4000.0 - 2300.0);
        return laminar + t * (turbulent - laminar);
    }
    turbulent
}

fn colebrook(re: f64, relative_roughness: f64) -> f64 {
    let eps = java_max(relative_roughness, 0.0);
    // Haaland explicit initial guess, then fixed-point Colebrook–White.
    let inv_sqrt = -1.8 * libm::log10(libm::pow(eps / 3.7, 1.11) + 6.9 / re);
    let mut f = 1.0 / (inv_sqrt * inv_sqrt);
    for _ in 0..60 {
        let rhs = -2.0 * libm::log10(eps / 3.7 + 2.51 / (re * libm::sqrt(f)));
        let f_new = 1.0 / (rhs * rhs);
        if libm::fabs(f_new - f) <= 1e-13 {
            return f_new;
        }
        f = f_new;
    }
    f
}

// --- Two-phase flow (props/TwoPhase.java) -----------------------------------

const TWO_PHASE_GRAVITY: f64 = 9.80665;

fn two_phase_densities(name: &str, rho_l: f64, rho_g: f64) -> Result<()> {
    if rho_l <= 0.0 || rho_g <= 0.0 {
        return Err(domain(name, format_args!("densities must be > 0")));
    }
    Ok(())
}

enum VoidModel {
    Homogeneous,
    Zivi,
}

fn void_fraction(name: &str, x: f64, rho_l: f64, rho_g: f64, model: VoidModel) -> Result<f64> {
    if x <= 0.0 {
        return Ok(0.0);
    }
    if x >= 1.0 {
        return Ok(1.0);
    }
    two_phase_densities(name, rho_l, rho_g)?;
    Ok(match model {
        VoidModel::Homogeneous => 1.0 / (1.0 + ((1.0 - x) / x) * (rho_g / rho_l)),
        VoidModel::Zivi => {
            let s = libm::cbrt(rho_l / rho_g);
            1.0 / (1.0 + ((1.0 - x) / x) * (rho_g / rho_l) * s)
        }
    })
}

fn void_rouhani(name: &str, x: f64, rho_l: f64, rho_g: f64, g: f64, sigma: f64) -> Result<f64> {
    if x <= 0.0 {
        return Ok(0.0);
    }
    if x >= 1.0 {
        return Ok(1.0);
    }
    two_phase_densities(name, rho_l, rho_g)?;
    if g <= 0.0 || sigma <= 0.0 || rho_l <= rho_g {
        return Err(domain(
            name,
            format_args!("mass flux G>0, surface tension sigma>0 and rho_l>rho_g required"),
        ));
    }
    let c0 = 1.0 + 0.12 * (1.0 - x);
    let ugu = 1.18
        * (1.0 - x)
        * libm::pow(
            TWO_PHASE_GRAVITY * sigma * (rho_l - rho_g) / (rho_l * rho_l),
            0.25,
        );
    let denom = c0 * (x / rho_g + (1.0 - x) / rho_l) + ugu / g;
    let alpha = (x / rho_g) / denom;
    Ok(java_max(0.0, java_min(1.0, alpha)))
}

/// Blasius Fanning friction factor 0.079·Re^-0.25 with a laminar floor 16/Re.
fn blasius_fanning(re: f64) -> f64 {
    let r = java_max(re, 1.0);
    if r < 1187.0 {
        16.0 / r
    } else {
        0.079 * libm::pow(r, -0.25)
    }
}

#[allow(clippy::too_many_arguments)]
fn friedel_phi2(
    name: &str,
    x: f64,
    rho_l: f64,
    rho_g: f64,
    mu_l: f64,
    mu_g: f64,
    g: f64,
    d: f64,
    sigma: f64,
) -> Result<f64> {
    two_phase_densities(name, rho_l, rho_g)?;
    if mu_l <= 0.0 || mu_g <= 0.0 || g <= 0.0 || d <= 0.0 || sigma <= 0.0 {
        return Err(domain(
            name,
            format_args!("viscosities, G, D and sigma must be > 0"),
        ));
    }
    let xx = clip(x, 1e-6, 1.0 - 1e-6);
    let rho_h = 1.0 / (xx / rho_g + (1.0 - xx) / rho_l);
    let f_lo = blasius_fanning(g * d / mu_l);
    let f_go = blasius_fanning(g * d / mu_g);
    let e = (1.0 - xx) * (1.0 - xx) + xx * xx * (rho_l * f_go) / (rho_g * f_lo);
    let f = libm::pow(xx, 0.78) * libm::pow(1.0 - xx, 0.224);
    let h = libm::pow(rho_l / rho_g, 0.91)
        * libm::pow(mu_g / mu_l, 0.19)
        * libm::pow(1.0 - mu_g / mu_l, 0.7);
    let fr = g * g / (TWO_PHASE_GRAVITY * d * rho_h * rho_h);
    let we = g * g * d / (rho_h * sigma);
    Ok(e + 3.24 * f * h / (libm::pow(fr, 0.045) * libm::pow(we, 0.035)))
}

// --- Heat-exchanger effectiveness-NTU (props/HeatExchanger.java) ------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum HxArrangement {
    Counterflow,
    Parallelflow,
    CrossflowBothUnmixed,
    CrossflowCmaxMixed,
    CrossflowCminMixed,
    ShellAndTube,
}

/// Resolves a user-supplied arrangement spelling, ignoring case, spaces and
/// punctuation, exactly like `HeatExchanger.arrangement`.
fn hx_arrangement(name: &str, spelling: &str) -> Result<HxArrangement> {
    let key: String = spelling
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        .collect();
    match key.as_str() {
        "counterflow" | "counter" | "countercurrent" => Ok(HxArrangement::Counterflow),
        "parallelflow" | "parallel" | "cocurrent" | "coflow" => Ok(HxArrangement::Parallelflow),
        "crossflow"
        | "crossflowbothunmixed"
        | "crossbothunmixed"
        | "crossflowunmixed"
        | "bothunmixed" => Ok(HxArrangement::CrossflowBothUnmixed),
        "crossflowcmaxmixed" | "cmaxmixed" | "crossflowcminunmixed" => {
            Ok(HxArrangement::CrossflowCmaxMixed)
        }
        "crossflowcminmixed" | "cminmixed" | "crossflowcmaxunmixed" => {
            Ok(HxArrangement::CrossflowCminMixed)
        }
        "shelltube" | "shellandtube" | "shellandtube1" | "shell" | "shelltube1" => {
            Ok(HxArrangement::ShellAndTube)
        }
        _ => Err(domain(
            name,
            format_args!(
                "unknown flow arrangement '{spelling}'. Use one of counterflow, parallelflow, \
                 crossflow_both_unmixed, crossflow_cmax_mixed, crossflow_cmin_mixed, shell&tube"
            ),
        )),
    }
}

fn hx_effectiveness(name: &str, arrangement: HxArrangement, ntu: f64, cr: f64) -> Result<f64> {
    if !(ntu >= 0.0) {
        return Err(domain(name, format_args!("NTU must be >= 0, got {ntu}")));
    }
    if !(0.0..=1.0).contains(&cr) {
        return Err(domain(
            name,
            format_args!("capacity ratio Cr = Cmin/Cmax must be in [0, 1], got {cr}"),
        ));
    }
    // Boiling/condensing limit: one stream isothermal, identical for all types.
    if cr == 0.0 {
        return Ok(1.0 - libm::exp(-ntu));
    }
    Ok(match arrangement {
        HxArrangement::Counterflow => {
            if libm::fabs(cr - 1.0) < 1e-10 {
                ntu / (1.0 + ntu)
            } else {
                let e = libm::exp(-ntu * (1.0 - cr));
                (1.0 - e) / (1.0 - cr * e)
            }
        }
        HxArrangement::Parallelflow => (1.0 - libm::exp(-ntu * (1.0 + cr))) / (1.0 + cr),
        HxArrangement::CrossflowBothUnmixed => {
            let n022 = libm::pow(ntu, 0.22);
            let n078 = libm::pow(ntu, 0.78);
            1.0 - libm::exp((1.0 / cr) * n022 * (libm::exp(-cr * n078) - 1.0))
        }
        HxArrangement::CrossflowCmaxMixed => {
            (1.0 / cr) * (1.0 - libm::exp(-cr * (1.0 - libm::exp(-ntu))))
        }
        HxArrangement::CrossflowCminMixed => {
            1.0 - libm::exp(-(1.0 / cr) * (1.0 - libm::exp(-cr * ntu)))
        }
        HxArrangement::ShellAndTube => {
            let root = libm::sqrt(1.0 + cr * cr);
            let e = libm::exp(-ntu * root);
            2.0 / (1.0 + cr + root * (1.0 + e) / (1.0 - e))
        }
    })
}

fn hx_max_effectiveness(arrangement: HxArrangement, cr: f64) -> f64 {
    match arrangement {
        HxArrangement::Counterflow | HxArrangement::CrossflowBothUnmixed => 1.0,
        HxArrangement::Parallelflow => 1.0 / (1.0 + cr),
        HxArrangement::CrossflowCmaxMixed => (1.0 / cr) * (1.0 - libm::exp(-cr)),
        HxArrangement::CrossflowCminMixed => 1.0 - libm::exp(-1.0 / cr),
        HxArrangement::ShellAndTube => 2.0 / (1.0 + cr + libm::sqrt(1.0 + cr * cr)),
    }
}

fn hx_ntu(name: &str, arrangement: HxArrangement, eps: f64, cr: f64) -> Result<f64> {
    if !(eps > 0.0 && eps < 1.0) {
        return Err(domain(
            name,
            format_args!("effectiveness must be in (0, 1), got {eps}"),
        ));
    }
    if !(0.0..=1.0).contains(&cr) {
        return Err(domain(
            name,
            format_args!("capacity ratio Cr = Cmin/Cmax must be in [0, 1], got {cr}"),
        ));
    }
    if cr == 0.0 {
        return Ok(-libm::log(1.0 - eps));
    }
    let eps_max = hx_max_effectiveness(arrangement, cr);
    if eps >= eps_max {
        return Err(domain(
            name,
            format_args!(
                "effectiveness {eps:.4} is unreachable for this arrangement at Cr={cr:.4} \
                 (limit {eps_max:.4} as NTU->inf)"
            ),
        ));
    }
    Ok(match arrangement {
        HxArrangement::Counterflow => {
            if libm::fabs(cr - 1.0) < 1e-10 {
                eps / (1.0 - eps)
            } else {
                (1.0 / (cr - 1.0)) * libm::log((eps - 1.0) / (eps * cr - 1.0))
            }
        }
        HxArrangement::Parallelflow => -libm::log(1.0 - eps * (1.0 + cr)) / (1.0 + cr),
        HxArrangement::CrossflowCmaxMixed => -libm::log(1.0 + libm::log(1.0 - cr * eps) / cr),
        HxArrangement::CrossflowCminMixed => {
            -(1.0 / cr) * libm::log(1.0 + cr * libm::log(1.0 - eps))
        }
        HxArrangement::ShellAndTube => {
            let root = libm::sqrt(1.0 + cr * cr);
            let e = (2.0 / eps - (1.0 + cr)) / root;
            -libm::log((e - 1.0) / (e + 1.0)) / root
        }
        HxArrangement::CrossflowBothUnmixed => hx_bisect_ntu(name, |n| {
            hx_effectiveness("hx_ntu", HxArrangement::CrossflowBothUnmixed, n, cr)
                .map(|value| value - eps)
        })?,
    })
}

/// Bisection for a monotone-increasing-in-NTU residual on [0, 200]
/// (`HeatExchanger.bisectNtu`).
fn hx_bisect_ntu(name: &str, residual: impl Fn(f64) -> Result<f64>) -> Result<f64> {
    let mut lo = 0.0;
    let mut hi = 200.0;
    let mut flo = residual(lo)?;
    let fhi = residual(hi)?;
    if flo * fhi > 0.0 {
        return Err(domain(
            name,
            format_args!("requested effectiveness is out of the solvable NTU range"),
        ));
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let fm = residual(mid)?;
        if libm::fabs(fm) < 1e-12 || (hi - lo) < 1e-12 {
            return Ok(mid);
        }
        if (fm > 0.0) == (flo > 0.0) {
            lo = mid;
            flo = fm;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

// --- Tube-bank / cylinder Nusselt correlations (HxCorrelations) -------------

fn nu_tube_bank(arrangement: &str, re: f64, pr: f64) -> f64 {
    let staggered = arrangement.to_lowercase().starts_with("stag");
    let re_eff = java_max(re, 1.0);
    let (c, m) = if re_eff < 100.0 {
        (if staggered { 0.90 } else { 0.80 }, 0.40)
    } else if re_eff < 1000.0 {
        (0.51, 0.50)
    } else if re_eff < 2e5 {
        (
            if staggered { 0.40 } else { 0.27 },
            if staggered { 0.60 } else { 0.63 },
        )
    } else {
        (if staggered { 0.022 } else { 0.21 }, 0.84)
    };
    c * libm::pow(re_eff, m) * libm::pow(pr, 0.36)
}

fn nu_hilpert(re: f64, pr: f64) -> f64 {
    let re_eff = java_max(re, 0.4);
    let (c, m) = if re_eff < 4.0 {
        (0.989, 0.330)
    } else if re_eff < 40.0 {
        (0.911, 0.385)
    } else if re_eff < 4000.0 {
        (0.683, 0.466)
    } else if re_eff < 4e4 {
        (0.193, 0.618)
    } else {
        (0.027, 0.805)
    };
    c * libm::pow(re_eff, m) * libm::pow(pr, 1.0 / 3.0)
}

fn fin_surface(s: &str) -> &'static str {
    match s.to_lowercase().as_str() {
        "wavy" => "wavy",
        "louvered" => "louvered",
        "offset" => "offset",
        _ => "plain",
    }
}

// --- Ideal-gas compressible flow (props/CompressibleFlow.java) --------------

fn cf_require_k(name: &str, k: f64) -> Result<()> {
    if !(k > 1.0) {
        return Err(domain(
            name,
            format_args!("ratio of specific heats k must be > 1, got {k}"),
        ));
    }
    Ok(())
}

fn cf_require_mach(name: &str, m: f64) -> Result<()> {
    if !(m > 0.0) {
        return Err(domain(
            name,
            format_args!("Mach number must be > 0, got {m}"),
        ));
    }
    Ok(())
}

fn cf_require_supersonic(name: &str, what: &str, m: f64) -> Result<()> {
    if !(m >= 1.0) {
        return Err(domain(
            name,
            format_args!("{what} requires a supersonic Mach number M >= 1, got {m}"),
        ));
    }
    Ok(())
}

fn cf_t0_over_t(name: &str, m: f64, k: f64) -> Result<f64> {
    cf_require_mach(name, m)?;
    cf_require_k(name, k)?;
    Ok(1.0 + 0.5 * (k - 1.0) * m * m)
}

fn cf_p0_over_p(name: &str, m: f64, k: f64) -> Result<f64> {
    Ok(libm::pow(cf_t0_over_t(name, m, k)?, k / (k - 1.0)))
}

fn cf_rho0_over_rho(name: &str, m: f64, k: f64) -> Result<f64> {
    Ok(libm::pow(cf_t0_over_t(name, m, k)?, 1.0 / (k - 1.0)))
}

fn cf_a_over_astar(name: &str, m: f64, k: f64) -> Result<f64> {
    cf_require_mach(name, m)?;
    cf_require_k(name, k)?;
    let t = 1.0 + 0.5 * (k - 1.0) * m * m;
    let exponent = (k + 1.0) / (2.0 * (k - 1.0));
    Ok((1.0 / m) * libm::pow((2.0 / (k + 1.0)) * t, exponent))
}

fn cf_mach_from_area_ratio(name: &str, ratio: f64, k: f64, regime: &str) -> Result<f64> {
    cf_require_k(name, k)?;
    if ratio < 1.0 {
        return Err(domain(name, format_args!("A/A* must be >= 1, got {ratio}")));
    }
    let r = regime.trim().to_lowercase();
    let subsonic = r.starts_with("sub");
    let supersonic = r.starts_with("sup");
    if !subsonic && !supersonic {
        return Err(domain(
            name,
            format_args!("mach_A_Astar branch must be 'subsonic' or 'supersonic', got '{regime}'"),
        ));
    }
    if ratio == 1.0 {
        return Ok(1.0);
    }
    let (lo, hi) = if subsonic { (1e-6, 1.0) } else { (1.0, 50.0) };
    cf_bisect(name, |m| Ok(cf_a_over_astar(name, m, k)? - ratio), lo, hi)
}

fn cf_mach_behind_shock(name: &str, m1: f64, k: f64) -> Result<f64> {
    cf_require_supersonic(name, "normal shock", m1)?;
    cf_require_k(name, k)?;
    let m1s = m1 * m1;
    Ok(libm::sqrt(
        ((k - 1.0) * m1s + 2.0) / (2.0 * k * m1s - (k - 1.0)),
    ))
}

fn cf_prandtl_meyer(name: &str, m: f64, k: f64) -> Result<f64> {
    cf_require_supersonic(name, "Prandtl-Meyer function", m)?;
    cf_require_k(name, k)?;
    let t = libm::sqrt((k + 1.0) / (k - 1.0));
    let s = libm::sqrt(m * m - 1.0);
    Ok(t * libm::atan(s / t) - libm::atan(s))
}

fn cf_mach_from_prandtl_meyer(name: &str, nu: f64, k: f64) -> Result<f64> {
    cf_require_k(name, k)?;
    let nu_max = 0.5 * std::f64::consts::PI * (libm::sqrt((k + 1.0) / (k - 1.0)) - 1.0);
    if nu < 0.0 || nu >= nu_max {
        return Err(domain(
            name,
            format_args!("Prandtl-Meyer angle {nu:.4} rad is outside (0, {nu_max:.4}) for k={k}"),
        ));
    }
    if nu == 0.0 {
        return Ok(1.0);
    }
    cf_bisect(name, |m| Ok(cf_prandtl_meyer(name, m, k)? - nu), 1.0, 1e4)
}

fn cf_theta_oblique(name: &str, m1: f64, beta: f64, k: f64) -> Result<f64> {
    cf_require_supersonic(name, "oblique shock", m1)?;
    cf_require_k(name, k)?;
    let m1n2 = m1 * m1 * libm::sin(beta) * libm::sin(beta);
    let num = 2.0 / libm::tan(beta) * (m1n2 - 1.0);
    let den = m1 * m1 * (k + libm::cos(2.0 * beta)) + 2.0;
    Ok(libm::atan(num / den))
}

fn cf_beta_oblique(name: &str, m1: f64, theta: f64, k: f64, branch: &str) -> Result<f64> {
    cf_require_supersonic(name, "oblique shock", m1)?;
    cf_require_k(name, k)?;
    if theta <= 0.0 {
        return Err(domain(
            name,
            format_args!("oblique-shock deflection theta must be > 0, got {theta}"),
        ));
    }
    let b = branch.trim().to_lowercase();
    let weak = b.starts_with("weak");
    let strong = b.starts_with("strong");
    if !weak && !strong {
        return Err(domain(
            name,
            format_args!("beta_oblique branch must be 'weak' or 'strong', got '{branch}'"),
        ));
    }
    let beta_min = libm::asin(1.0 / m1); // Mach wave (theta -> 0)
    let beta_max = 0.5 * std::f64::consts::PI; // normal shock (theta -> 0)
                                               // theta(beta) rises from 0 at betaMin to a maximum then falls to 0 at pi/2:
                                               // locate the peak, then bisect the requested monotone branch.
    let mut beta_peak = beta_min;
    let mut theta_peak = 0.0;
    let n = 400;
    for i in 0..=n {
        let beta = beta_min + (beta_max - beta_min) * i as f64 / n as f64;
        let th = cf_theta_oblique(name, m1, beta, k)?;
        if th > theta_peak {
            theta_peak = th;
            beta_peak = beta;
        }
    }
    if theta > theta_peak {
        return Err(domain(
            name,
            format_args!(
                "deflection theta={theta:.4} rad exceeds the maximum {theta_peak:.4} rad \
                 for M1={m1}, k={k} (shock detaches)"
            ),
        ));
    }
    let residual = |beta: f64| Ok(cf_theta_oblique(name, m1, beta, k)? - theta);
    if weak {
        cf_bisect(name, residual, beta_min, beta_peak) // increasing branch
    } else {
        cf_bisect(name, residual, beta_peak, beta_max) // decreasing branch
    }
}

/// Bisection root-finder for a continuous residual sign-bracketed by
/// `[lo, hi]` (`CompressibleFlow.bisect`).
fn cf_bisect(name: &str, f: impl Fn(f64) -> Result<f64>, lo: f64, hi: f64) -> Result<f64> {
    let mut lo = lo;
    let mut hi = hi;
    let mut flo = f(lo)?;
    let fhi = f(hi)?;
    if flo == 0.0 {
        return Ok(lo);
    }
    if fhi == 0.0 {
        return Ok(hi);
    }
    if flo * fhi > 0.0 {
        return Err(domain(
            name,
            format_args!("target is outside the solvable range for the requested branch"),
        ));
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid)?;
        if libm::fabs(fm) < 1e-12 || (hi - lo) < 1e-12 {
            return Ok(mid);
        }
        if (fm > 0.0) == (flo > 0.0) {
            lo = mid;
            flo = fm;
        } else {
            hi = mid;
        }
    }
    Ok(0.5 * (lo + hi))
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

fn median(v: &[f64]) -> f64 {
    let mut sorted = v.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        0.5 * (sorted[n / 2 - 1] + sorted[n / 2])
    }
}

/// Unbiased (n−1) sample variance; `0` for a single value, as in Java.
fn sample_variance(v: &[f64]) -> f64 {
    if v.len() == 1 {
        return 0.0;
    }
    let m = mean(v);
    let ss: f64 = v.iter().map(|x| (x - m) * (x - m)).sum();
    ss / (v.len() - 1) as f64
}

/// Apache Commons `Percentile` in its default (LEGACY / R-6) estimation type,
/// which is what the Java arm uses; `p` is clamped to `[1e-9, 100]` first.
///
/// A `NaN` percentile is refused rather than clamped. `Math.min`/`Math.max`
/// propagate `NaN`, so the clamp leaves it alone and every subsequent
/// comparison answers `false` — which walked straight into `sorted[index - 1]`
/// with `index == 0` and panicked. Apache's own `Percentile` throws
/// `OutOfRangeException` for a non-finite quantile, so an error is also the
/// faithful answer. `NaN` reaches here through ordinary arithmetic
/// (`percentile(1e999 - 1e999, …)`) and through any iterate the solver pushes
/// into an invalid region, so this is a live path, not a theoretical one.
fn percentile(name: &str, p: f64, data: &[f64]) -> Result<f64> {
    if p.is_nan() {
        return Err(domain(
            name,
            format_args!("the percentile must be a number in [0, 100], got NaN"),
        ));
    }
    let p = java_max(1.0e-9, java_min(100.0, p));
    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    if n <= 1 {
        // `Arity::AtLeast(2)` guarantees one data point; an empty slice can only
        // come from a direct call, and answering NaN beats indexing off the end.
        return Ok(sorted.first().copied().unwrap_or(f64::NAN));
    }
    let pos = p / 100.0 * (n as f64 + 1.0);
    if pos < 1.0 {
        return Ok(sorted[0]);
    }
    if pos >= n as f64 {
        return Ok(sorted[n - 1]);
    }
    let floor = libm::floor(pos);
    // `1 <= pos < n` and `p` is finite, so `floor` is in `[1, n)` — the clamp is
    // a no-op that makes the indices below true by construction rather than by
    // argument.
    let index = (floor as usize).clamp(1, n - 1);
    let frac = pos - floor;
    let lower = sorted[index - 1];
    let upper = sorted[index];
    Ok(lower + frac * (upper - lower))
}

/// P₀=1, P₁=x, (n+1)Pₙ₊₁=(2n+1)x·Pₙ − n·Pₙ₋₁.
fn legendre_p(n: usize, x: f64) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let (mut prev, mut curr) = (1.0, x);
    for k in 1..n {
        let k = k as f64;
        let next = ((2.0 * k + 1.0) * x * curr - k * prev) / (k + 1.0);
        prev = curr;
        curr = next;
    }
    curr
}

/// T₀=1, T₁=x, Tₙ₊₁=2x·Tₙ − Tₙ₋₁.
fn chebyshev_t(n: usize, x: f64) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let (mut prev, mut curr) = (1.0, x);
    for _ in 1..n {
        let next = 2.0 * x * curr - prev;
        prev = curr;
        curr = next;
    }
    curr
}

/// U₀=1, U₁=2x, Uₙ₊₁=2x·Uₙ − Uₙ₋₁.
fn chebyshev_u(n: usize, x: f64) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let (mut prev, mut curr) = (1.0, 2.0 * x);
    for _ in 1..n {
        let next = 2.0 * x * curr - prev;
        prev = curr;
        curr = next;
    }
    curr
}

/// Physicists' Hermite: H₀=1, H₁=2x, Hₙ₊₁=2x·Hₙ − 2n·Hₙ₋₁.
fn hermite_h(n: usize, x: f64) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let (mut prev, mut curr) = (1.0, 2.0 * x);
    for k in 1..n {
        let next = 2.0 * x * curr - 2.0 * k as f64 * prev;
        prev = curr;
        curr = next;
    }
    curr
}

/// L₀=1, L₁=1−x, (n+1)Lₙ₊₁=(2n+1−x)Lₙ − n·Lₙ₋₁.
fn laguerre_l(n: usize, x: f64) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let (mut prev, mut curr) = (1.0, 1.0 - x);
    for k in 1..n {
        let k = k as f64;
        let next = ((2.0 * k + 1.0 - x) * curr - k * prev) / (k + 1.0);
        prev = curr;
        curr = next;
    }
    curr
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinOp, CmpOp, Expr, LogicOp};

    const EPS: f64 = 1e-12;

    fn scope(pairs: &[(&str, f64)]) -> Scope {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    /// Evaluate against an empty scope, panicking on error.
    fn ev(e: &Expr) -> f64 {
        eval(e, &Scope::default()).unwrap_or_else(|err| panic!("eval failed: {err}"))
    }

    /// Evaluate expecting an error; returns the rendered message.
    fn err(e: &Expr) -> String {
        match eval(e, &Scope::default()) {
            Ok(v) => panic!("expected an error, got {v}"),
            Err(e) => e.to_string(),
        }
    }

    fn n(v: f64) -> Expr {
        Expr::num(v)
    }

    /// `f(args…)` evaluated against an empty scope.
    fn call(name: &str, args: &[f64]) -> Result<f64> {
        eval(
            &Expr::call(name, args.iter().copied().map(n).collect()),
            &Scope::default(),
        )
    }

    fn c(name: &str, args: &[f64]) -> f64 {
        call(name, args).unwrap_or_else(|e| panic!("{name} failed: {e}"))
    }

    fn cerr(name: &str, args: &[f64]) -> String {
        match call(name, args) {
            Ok(v) => panic!("expected {name} to fail, got {v}"),
            Err(e) => e.to_string(),
        }
    }

    fn close(a: f64, b: f64) {
        assert!(
            (a - b).abs() <= 1e-10 * b.abs().max(1.0),
            "expected {b}, got {a}"
        );
    }

    // -- literals, strings, variables --------------------------------------

    #[test]
    fn numeric_literals_evaluate_to_their_si_value() {
        assert_eq!(ev(&n(140_000.0)), 140_000.0);
        assert_eq!(
            ev(&Expr::Num {
                value: 3.5,
                unit: Some("Pa".into()),
                is_imaginary: false,
            }),
            3.5
        );
    }

    #[test]
    fn imaginary_literals_yield_their_magnitude_like_java() {
        // The Java arm destructures `isImaginary` and ignores it.
        assert_eq!(
            ev(&Expr::Num {
                value: 2.0,
                unit: None,
                is_imaginary: true,
            }),
            2.0
        );
    }

    #[test]
    fn string_literals_cannot_be_evaluated_numerically() {
        let message = err(&Expr::Str("water".into()));
        assert!(message.contains("string literal 'water'"), "{message}");
        assert!(
            message.contains("cannot be evaluated as a number"),
            "{message}"
        );
    }

    #[test]
    fn variables_resolve_from_the_scope() {
        let s = scope(&[("t_in", 300.0)]);
        assert_eq!(eval(&Expr::var("T_in"), &s).unwrap(), 300.0);
    }

    #[test]
    fn unknown_variables_are_an_error_not_a_zero() {
        let message = err(&Expr::var("t_in"));
        assert!(message.contains("variable has no value: t_in"), "{message}");
    }

    #[test]
    fn scope_bindings_shadow_builtin_constants() {
        let s = scope(&[("pi#", 3.0)]);
        assert_eq!(eval(&Expr::var("pi#"), &s).unwrap(), 3.0);
    }

    // -- built-in constants -------------------------------------------------

    #[test]
    fn builtin_constants_resolve_without_a_scope_entry() {
        assert_eq!(ev(&Expr::var("pi#")), std::f64::consts::PI);
        assert_eq!(ev(&Expr::var("e#")), std::f64::consts::E);
        assert_eq!(ev(&Expr::var("r#")), 8.314_462_618);
        assert_eq!(ev(&Expr::var("g#")), 9.806_65);
        assert_eq!(ev(&Expr::var("na#")), 6.022_140_76e23);
        assert_eq!(ev(&Expr::var("k#")), 1.380_649e-23);
        assert_eq!(ev(&Expr::var("h#")), 6.626_070_15e-34);
        assert_eq!(ev(&Expr::var("c#")), 299_792_458.0);
        assert_eq!(ev(&Expr::var("sigma#")), 5.670_374_419e-8);
        assert_eq!(ev(&Expr::var("gc#")), 6.674_30e-11);
        assert_eq!(ev(&Expr::var("qe#")), 1.602_176_634e-19);
    }

    #[test]
    fn the_constant_table_matches_the_java_registry_exactly() {
        // ConstantsRegistry defines eleven constants, in this order.
        let names: Vec<&str> = CONSTANTS.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            vec!["pi#", "e#", "R#", "g#", "Na#", "k#", "h#", "c#", "sigma#", "Gc#", "qe#"]
        );
        assert!(CONSTANTS.iter().all(|c| c.name.ends_with('#')));
        assert_eq!(
            lookup_constant("SIGMA#").map(|c| c.unit),
            Some(Some("W/m^2-K^4"))
        );
        assert_eq!(lookup_constant("pi#").unwrap().unit, None);
        assert!(lookup_constant("nope#").is_none());
    }

    // -- arithmetic operators -----------------------------------------------

    #[test]
    fn every_arithmetic_operator_evaluates() {
        let b = |op, l: f64, r: f64| ev(&Expr::bin(op, n(l), n(r)));
        assert_eq!(b(BinOp::Add, 2.0, 3.0), 5.0);
        assert_eq!(b(BinOp::Sub, 2.0, 3.0), -1.0);
        assert_eq!(b(BinOp::Mul, 2.0, 3.0), 6.0);
        assert_eq!(b(BinOp::Div, 3.0, 2.0), 1.5);
        assert_eq!(b(BinOp::Pow, 2.0, 10.0), 1024.0);
    }

    #[test]
    fn left_division_divides_by_the_left_operand() {
        // a \ b == b / a
        assert_eq!(ev(&Expr::bin(BinOp::LeftDiv, n(2.0), n(10.0))), 5.0);
        assert_eq!(ev(&Expr::bin(BinOp::LeftDiv, n(4.0), n(1.0))), 0.25);
    }

    #[test]
    fn element_wise_operators_use_their_scalar_equivalents() {
        assert_eq!(ev(&Expr::bin(BinOp::ElemMul, n(3.0), n(4.0))), 12.0);
        assert_eq!(ev(&Expr::bin(BinOp::ElemDiv, n(3.0), n(4.0))), 0.75);
        assert_eq!(ev(&Expr::bin(BinOp::ElemLeftDiv, n(4.0), n(3.0))), 0.75);
        assert_eq!(ev(&Expr::bin(BinOp::ElemPow, n(2.0), n(3.0))), 8.0);
    }

    #[test]
    fn negation_and_nesting() {
        // -(2 + 3) * 4
        let e = Expr::bin(
            BinOp::Mul,
            Expr::Neg(Box::new(Expr::bin(BinOp::Add, n(2.0), n(3.0)))),
            n(4.0),
        );
        assert_eq!(ev(&e), -20.0);
    }

    #[test]
    fn division_by_zero_is_an_error_not_an_infinity() {
        let message = err(&Expr::bin(BinOp::Div, n(1.0), n(0.0)));
        assert!(message.contains("division by zero"), "{message}");
        assert!(matches!(
            eval(&Expr::bin(BinOp::Div, n(1.0), n(0.0)), &Scope::default()),
            Err(FreesError::Evaluation { .. })
        ));
    }

    #[test]
    fn left_division_by_zero_is_an_error() {
        let message = err(&Expr::bin(BinOp::LeftDiv, n(0.0), n(1.0)));
        assert!(message.contains("division by zero"), "{message}");
        assert!(message.contains("left division"), "{message}");
    }

    #[test]
    fn element_wise_division_by_zero_is_also_an_error() {
        assert!(err(&Expr::bin(BinOp::ElemDiv, n(1.0), n(0.0))).contains("division by zero"));
        assert!(err(&Expr::bin(BinOp::ElemLeftDiv, n(0.0), n(1.0))).contains("division by zero"));
    }

    #[test]
    fn pow_domain_errors_replace_silent_nan_and_infinity() {
        let negative = err(&Expr::bin(BinOp::Pow, n(-8.0), n(0.5)));
        assert!(negative.contains("negative base"), "{negative}");
        let zero = err(&Expr::bin(BinOp::Pow, n(0.0), n(-1.0)));
        assert!(zero.contains("division by zero"), "{zero}");
        // Integral exponents on a negative base stay legal.
        assert_eq!(ev(&Expr::bin(BinOp::Pow, n(-2.0), n(3.0))), -8.0);
        assert_eq!(ev(&Expr::bin(BinOp::Pow, n(0.0), n(0.0))), 1.0);
    }

    /// `^` is Java's `Math.pow`, not C's — the ledger's one *closed* Phase-10
    /// divergence, and the only part of that section that is still live code.
    ///
    /// C answers `1` for `pow(1, NaN)` and `pow(±1, ±∞)`; Java answers `NaN`,
    /// and so does `ast/Evaluator`, so a document that raises exactly 1 to a
    /// missing exponent must not have a sample invented for it. This test
    /// carries over the assertions that
    /// `measurement_parity.rs::a_gap_in_the_exponent_stays_a_gap_inside_a_call_argument`
    /// made before D11 deleted that suite with its surface; the rule it pins
    /// lives here, in the document evaluator, and is engine-wide.
    #[test]
    fn a_non_finite_exponent_answers_the_java_way() {
        let pow = |l: f64, r: f64| ev(&Expr::bin(BinOp::Pow, n(l), n(r)));
        // The two families C gets wrong.
        assert!(pow(1.0, f64::NAN).is_nan());
        assert!(pow(1.0, f64::INFINITY).is_nan());
        assert!(pow(1.0, f64::NEG_INFINITY).is_nan());
        assert!(pow(-1.0, f64::INFINITY).is_nan());
        assert!(pow(-1.0, f64::NEG_INFINITY).is_nan());
        // A NaN exponent is NaN whatever the base — including inside a call
        // argument, which is the second site the rule was needed at.
        assert!(pow(2.0, f64::NAN).is_nan());
        assert!(c("abs", &[pow(1.0, f64::NAN)]).is_nan());
        // Everything else keeps IEEE's answer: |base| ≠ 1 against ±∞ is not
        // one of the two families, and must not be swept up by the guard.
        assert_eq!(pow(2.0, f64::INFINITY), f64::INFINITY);
        assert_eq!(pow(2.0, f64::NEG_INFINITY), 0.0);
        assert_eq!(pow(0.5, f64::INFINITY), 0.0);
    }

    #[test]
    fn errors_propagate_out_of_nested_subexpressions() {
        // 1 + (2 / 0)
        let e = Expr::bin(BinOp::Add, n(1.0), Expr::bin(BinOp::Div, n(2.0), n(0.0)));
        assert!(err(&e).contains("division by zero"));
    }

    // -- comparison, logic, truthiness ---------------------------------------

    #[test]
    fn every_comparison_operator_yields_exactly_one_or_zero() {
        let cmp = |op, l: f64, r: f64| {
            ev(&Expr::Compare {
                op,
                left: Box::new(n(l)),
                right: Box::new(n(r)),
            })
        };
        assert_eq!(cmp(CmpOp::Lt, 1.0, 2.0), 1.0);
        assert_eq!(cmp(CmpOp::Lt, 2.0, 1.0), 0.0);
        assert_eq!(cmp(CmpOp::Gt, 2.0, 1.0), 1.0);
        assert_eq!(cmp(CmpOp::Gt, 1.0, 2.0), 0.0);
        assert_eq!(cmp(CmpOp::Le, 2.0, 2.0), 1.0);
        assert_eq!(cmp(CmpOp::Le, 3.0, 2.0), 0.0);
        assert_eq!(cmp(CmpOp::Ge, 2.0, 2.0), 1.0);
        assert_eq!(cmp(CmpOp::Ge, 1.0, 2.0), 0.0);
        assert_eq!(cmp(CmpOp::Ne, 1.0, 2.0), 1.0);
        assert_eq!(cmp(CmpOp::Ne, 2.0, 2.0), 0.0);
        assert_eq!(cmp(CmpOp::Eq, 2.0, 2.0), 1.0);
        assert_eq!(cmp(CmpOp::Eq, 1.0, 2.0), 0.0);
    }

    #[test]
    fn logical_operators_use_the_java_truthiness_rule() {
        let logic = |op, l: f64, r: f64| {
            ev(&Expr::Logical {
                op,
                left: Box::new(n(l)),
                right: Box::new(n(r)),
            })
        };
        // Non-zero is true — including negatives and fractions.
        assert_eq!(logic(LogicOp::And, -1.0, 0.5), 1.0);
        assert_eq!(logic(LogicOp::And, 1.0, 0.0), 0.0);
        assert_eq!(logic(LogicOp::And, 0.0, 0.0), 0.0);
        assert_eq!(logic(LogicOp::Or, 0.0, -3.0), 1.0);
        assert_eq!(logic(LogicOp::Or, 0.0, 0.0), 0.0);
    }

    #[test]
    fn logical_operators_do_not_short_circuit_like_java() {
        // `0 and (1/0)` still evaluates the right operand, so the domain error
        // surfaces — matching the Java evaluator, which is eager on both sides.
        let e = Expr::Logical {
            op: LogicOp::And,
            left: Box::new(n(0.0)),
            right: Box::new(Expr::bin(BinOp::Div, n(1.0), n(0.0))),
        };
        assert!(err(&e).contains("division by zero"));
    }

    #[test]
    fn not_inverts_truthiness_to_one_or_zero() {
        let not = |v: f64| ev(&Expr::Not(Box::new(n(v))));
        assert_eq!(not(0.0), 1.0);
        assert_eq!(not(1.0), 0.0);
        assert_eq!(not(-2.5), 0.0);
        // Double negation normalises any truthy value to exactly 1.
        assert_eq!(ev(&Expr::Not(Box::new(Expr::Not(Box::new(n(42.0)))))), 1.0);
    }

    // -- non-scalar variants --------------------------------------------------

    #[test]
    fn array_access_range_and_literals_are_rejected_with_a_reason() {
        let access = err(&Expr::ArrayAccess {
            name: "speed".into(),
            indices: vec![n(1.0)],
        });
        assert!(access.contains("speed"), "{access}");
        assert!(access.contains("cannot be evaluated directly"), "{access}");

        let range = err(&Expr::Range {
            start: Box::new(n(1.0)),
            end: Box::new(n(5.0)),
        });
        assert!(range.contains("index range"), "{range}");

        let literal = err(&Expr::ArrayLiteral(vec![n(1.0), n(2.0)]));
        assert!(literal.contains("array literal"), "{literal}");
        assert!(literal.contains("2 element"), "{literal}");
    }

    // -- registry health -------------------------------------------------------

    #[test]
    fn the_registry_has_no_duplicate_or_mis_cased_names() {
        let mut seen = std::collections::BTreeSet::new();
        for intrinsic in INTRINSICS {
            assert_eq!(
                intrinsic.name,
                intrinsic.name.to_ascii_lowercase(),
                "intrinsic names must be lowercase: {}",
                intrinsic.name
            );
            assert!(
                seen.insert(intrinsic.name),
                "duplicate intrinsic: {}",
                intrinsic.name
            );
        }
        assert_eq!(seen.len(), INTRINSICS.len());
        assert_eq!(registry().len(), INTRINSICS.len());
    }

    #[test]
    fn every_registered_intrinsic_is_reachable_by_lookup() {
        for intrinsic in INTRINSICS {
            assert!(
                lookup_intrinsic(intrinsic.name).is_some(),
                "{} is not reachable",
                intrinsic.name
            );
        }
        assert!(lookup_intrinsic("definitely_not_a_function").is_none());
    }

    #[test]
    fn no_intrinsic_is_also_listed_as_unported() {
        for intrinsic in INTRINSICS {
            assert!(
                unported_family(intrinsic.name).is_none(),
                "{} is both implemented and marked unported",
                intrinsic.name
            );
        }
    }

    // -- elementary intrinsics: known values ------------------------------------

    #[test]
    fn abs_sign_and_the_power_shorthands() {
        assert_eq!(c("abs", &[-3.5]), 3.5);
        assert_eq!(c("abs", &[3.5]), 3.5);
        assert_eq!(c("sign", &[-2.0]), -1.0);
        assert_eq!(c("sign", &[2.0]), 1.0);
        assert_eq!(c("sign", &[0.0]), 0.0);
        assert_eq!(c("sqr", &[-4.0]), 16.0);
        assert_eq!(c("cube", &[-3.0]), -27.0);
        assert_eq!(c("hypot", &[3.0, 4.0]), 5.0);
    }

    #[test]
    fn roots_and_their_domains() {
        assert_eq!(c("sqrt", &[9.0]), 3.0);
        assert_eq!(c("sqrt", &[0.0]), 0.0);
        close(c("cbrt", &[-27.0]), -3.0);
        let message = cerr("sqrt", &[-1.0]);
        assert!(
            message.contains("square root of a negative number"),
            "{message}"
        );
    }

    #[test]
    fn exponential_and_logarithms() {
        close(c("exp", &[0.0]), 1.0);
        close(c("exp", &[1.0]), std::f64::consts::E);
        close(c("ln", &[std::f64::consts::E]), 1.0);
        close(c("log10", &[1000.0]), 3.0);
        close(c("log2", &[1024.0]), 10.0);
        // `log` is the base-10 alias.
        close(c("log", &[1000.0]), 3.0);
        // exp/ln round-trip
        close(c("ln", &[c("exp", &[2.75])]), 2.75);
    }

    #[test]
    fn logarithm_domain_errors() {
        for name in ["ln", "log10", "log2", "log"] {
            let zero = cerr(name, &[0.0]);
            assert!(zero.contains("logarithm of zero"), "{name}: {zero}");
            let negative = cerr(name, &[-1.0]);
            assert!(
                negative.contains("logarithm of a negative number"),
                "{name}: {negative}"
            );
        }
    }

    #[test]
    fn trigonometry_is_in_radians() {
        // sin(pi/6) = 0.5 only if the argument is radians; in degrees it would
        // be ~0.00914. This is the assertion that pins the angle convention.
        close(c("sin", &[std::f64::consts::PI / 6.0]), 0.5);
        close(c("cos", &[std::f64::consts::PI / 3.0]), 0.5);
        close(c("tan", &[std::f64::consts::PI / 4.0]), 1.0);
        assert!(
            (c("sin", &[30.0]) - 0.5).abs() > 1e-3,
            "30 must not be read as degrees"
        );
    }

    #[test]
    fn inverse_trigonometry_returns_radians_under_both_spellings() {
        for name in ["asin", "arcsin"] {
            close(c(name, &[0.5]), std::f64::consts::PI / 6.0);
        }
        for name in ["acos", "arccos"] {
            close(c(name, &[0.5]), std::f64::consts::PI / 3.0);
        }
        for name in ["atan", "arctan"] {
            close(c(name, &[1.0]), std::f64::consts::PI / 4.0);
        }
    }

    #[test]
    fn inverse_trigonometry_domain_errors() {
        for name in ["asin", "arcsin", "acos", "arccos"] {
            let message = cerr(name, &[1.5]);
            assert!(message.contains("must be in [-1, 1]"), "{name}: {message}");
            assert!(cerr(name, &[-1.5]).contains("must be in [-1, 1]"));
        }
        // The endpoints are legal.
        close(c("asin", &[1.0]), std::f64::consts::FRAC_PI_2);
        close(c("acos", &[-1.0]), std::f64::consts::PI);
    }

    #[test]
    fn atan2_is_four_quadrant_with_y_first() {
        close(c("atan2", &[1.0, 1.0]), std::f64::consts::PI / 4.0);
        close(c("atan2", &[1.0, -1.0]), 3.0 * std::f64::consts::PI / 4.0);
        close(c("atan2", &[-1.0, -1.0]), -3.0 * std::f64::consts::PI / 4.0);
        assert_eq!(c("atan2", &[0.0, 0.0]), 0.0);
    }

    #[test]
    fn hyperbolic_functions_and_their_inverses() {
        close(c("sinh", &[0.0]), 0.0);
        close(c("cosh", &[0.0]), 1.0);
        close(c("tanh", &[0.0]), 0.0);
        close(c("sinh", &[1.0]), 1.175_201_193_643_801_4);
        close(c("cosh", &[1.0]), 1.543_080_634_815_243_7);
        close(c("tanh", &[1.0]), 0.761_594_155_955_764_9);
        // Round-trips through the closed forms Java uses.
        close(c("arcsinh", &[c("sinh", &[0.7])]), 0.7);
        close(c("arccosh", &[c("cosh", &[0.7])]), 0.7);
        close(c("arctanh", &[c("tanh", &[0.7])]), 0.7);
    }

    #[test]
    fn inverse_hyperbolic_domain_errors() {
        let acosh = cerr("arccosh", &[0.5]);
        assert!(acosh.contains(">= 1.0"), "{acosh}");
        assert_eq!(c("arccosh", &[1.0]), 0.0);
        let atanh = cerr("arctanh", &[1.0]);
        assert!(atanh.contains("(-1, 1)"), "{atanh}");
        assert!(cerr("arctanh", &[-1.0]).contains("(-1, 1)"));
    }

    #[test]
    fn rounding_family_matches_java() {
        assert_eq!(c("floor", &[2.7]), 2.0);
        assert_eq!(c("floor", &[-2.1]), -3.0);
        assert_eq!(c("ceil", &[2.1]), 3.0);
        assert_eq!(c("ceil", &[-2.7]), -2.0);
        assert_eq!(c("trunc", &[2.9]), 2.0);
        assert_eq!(c("trunc", &[-2.9]), -2.0);
        assert_eq!(c("int", &[-2.9]), -2.0);
        assert_eq!(c("round", &[2.4]), 2.0);
        assert_eq!(c("round", &[2.5]), 3.0);
        // Java's Math.round breaks ties toward +infinity, so -2.5 -> -2.
        assert_eq!(c("round", &[-2.5]), -2.0);
        assert_eq!(c("round", &[-2.6]), -3.0);
    }

    #[test]
    fn round_takes_an_optional_digit_count() {
        close(c("round", &[1.23456, 2.0]), 1.23);
        close(c("round", &[1.23456, 4.0]), 1.2346);
        assert_eq!(c("round", &[1234.0, -2.0]), 1200.0);
        let message = cerr("round", &[1.0, 2.0, 3.0]);
        assert!(message.contains("1 to 2 arguments"), "{message}");
    }

    #[test]
    fn step_and_ramp_switch_at_the_origin() {
        assert_eq!(c("step", &[-0.001]), 0.0);
        assert_eq!(c("step", &[0.0]), 1.0);
        assert_eq!(c("step", &[3.0]), 1.0);
        assert_eq!(c("ramp", &[-3.0]), 0.0);
        assert_eq!(c("ramp", &[0.0]), 0.0);
        assert_eq!(c("ramp", &[3.0]), 3.0);
    }

    #[test]
    fn min_and_max_are_variadic() {
        assert_eq!(c("min", &[3.0]), 3.0);
        assert_eq!(c("min", &[3.0, -1.0, 7.0]), -1.0);
        assert_eq!(c("max", &[3.0, -1.0, 7.0]), 7.0);
        assert_eq!(c("max", &[-5.0]), -5.0);
        let message = cerr("min", &[]);
        assert!(message.contains("at least 1 argument"), "{message}");
    }

    #[test]
    fn mod_and_rem_take_the_sign_of_the_dividend() {
        assert_eq!(c("mod", &[7.0, 3.0]), 1.0);
        assert_eq!(c("mod", &[-7.0, 3.0]), -1.0);
        assert_eq!(c("mod", &[7.0, -3.0]), 1.0);
        assert_eq!(c("rem", &[-7.0, 3.0]), -1.0);
        close(c("mod", &[5.5, 2.0]), 1.5);
        let message = cerr("mod", &[1.0, 0.0]);
        assert!(message.contains("division by zero"), "{message}");
    }

    #[test]
    fn number_theory_and_bitwise_operations() {
        assert_eq!(c("gcd", &[12.0, 18.0]), 6.0);
        assert_eq!(c("gcd", &[-12.0, 18.0]), 6.0);
        assert_eq!(c("gcd", &[0.0, 0.0]), 0.0);
        assert_eq!(c("lcm", &[4.0, 6.0]), 12.0);
        assert_eq!(c("lcm", &[0.0, 5.0]), 0.0);
        assert_eq!(c("bitand", &[12.0, 10.0]), 8.0);
        assert_eq!(c("bitor", &[12.0, 10.0]), 14.0);
        assert_eq!(c("bitxor", &[12.0, 10.0]), 6.0);
        assert_eq!(c("bitnot", &[0.0]), -1.0);
        assert_eq!(c("bitshiftl", &[1.0, 10.0]), 1024.0);
        assert_eq!(c("bitshiftr", &[-1024.0, 3.0]), -128.0);
    }

    #[test]
    fn erf_gamma_and_factorial_known_values() {
        assert_eq!(c("erf", &[0.0]), 0.0);
        close(c("erf", &[1.0]), 0.842_700_792_949_714_9);
        close(c("erfc", &[1.0]), 1.0 - 0.842_700_792_949_714_9);
        close(c("gamma", &[5.0]), 24.0);
        close(c("gamma", &[0.5]), libm::sqrt(std::f64::consts::PI));
        close(c("factorial", &[0.0]), 1.0);
        close(c("factorial", &[5.0]), 120.0);
        close(c("factorial", &[10.0]), 3_628_800.0);
        close(c("loggamma", &[10.0]), libm::log(362_880.0));
        close(c("beta", &[2.0, 3.0]), 1.0 / 12.0);
    }

    #[test]
    fn gamma_and_factorial_domain_errors() {
        let pole = cerr("gamma", &[0.0]);
        assert!(pole.contains("pole at non-positive integer"), "{pole}");
        assert!(cerr("gamma", &[-3.0]).contains("pole"));
        // Non-integer negatives are fine.
        close(c("gamma", &[-0.5]), -2.0 * libm::sqrt(std::f64::consts::PI));
        let factorial = cerr("factorial", &[-1.5]);
        assert!(factorial.contains("must be > -1"), "{factorial}");
        assert!(cerr("factorial", &[-1.0]).contains("must be > -1"));
        assert!(cerr("beta", &[0.0, 1.0]).contains("must be > 0"));
    }

    /// `loggamma` is `log Γ(x)`, not `log |Γ(x)|`.
    ///
    /// The Java arm is Apache's `Gamma.logGamma`, which returns NaN for every
    /// `x <= 0` — Γ is negative on (−1, 0), (−3, −2), … so its logarithm does
    /// not exist there. `libm::lgamma` computes the *other* function
    /// `log |Γ(x)|` and returns a perfectly finite 1.2655… for `-0.5`, which is
    /// a value the reference engine never produces. Guard the whole
    /// non-positive half-line, not just the poles.
    #[test]
    fn loggamma_refuses_the_non_positive_half_line() {
        // Positive arguments are unaffected.
        close(c("loggamma", &[1.0]), 0.0);
        close(c("loggamma", &[10.0]), libm::log(362_880.0));
        close(
            c("loggamma", &[0.5]),
            libm::log(libm::sqrt(std::f64::consts::PI)),
        );

        for x in [-0.5, -1.5, -2.5, -0.0001] {
            let message = cerr("loggamma", &[x]);
            assert!(
                message.contains("must be > 0"),
                "loggamma({x}) must be refused, got {message}"
            );
            // `log |Γ|` really is finite there — that is the trap.
            assert!(libm::lgamma(x).is_finite(), "premise check for {x}");
        }
        assert!(cerr("loggamma", &[0.0]).contains("must be > 0"));
        assert!(cerr("loggamma", &[-3.0]).contains("must be > 0"));
        // `gamma` itself is unchanged: it is defined on the non-integer negatives.
        close(c("gamma", &[-0.5]), -2.0 * libm::sqrt(std::f64::consts::PI));
    }

    #[test]
    fn orthogonal_polynomials_match_their_closed_forms() {
        // P2(x) = (3x^2 - 1)/2
        close(c("legendrep", &[2.0, 0.3]), (3.0 * 0.09 - 1.0) / 2.0);
        assert_eq!(c("legendrep", &[0.0, 7.0]), 1.0);
        // T3(x) = 4x^3 - 3x
        close(c("chebyshevt", &[3.0, 0.4]), 4.0 * 0.064 - 1.2);
        // U2(x) = 4x^2 - 1
        close(c("chebyshevu", &[2.0, 0.4]), 4.0 * 0.16 - 1.0);
        // H3(x) = 8x^3 - 12x
        close(c("hermiteh", &[3.0, 0.5]), 8.0 * 0.125 - 6.0);
        // L2(x) = 1 - 2x + x^2/2
        close(c("laguerrel", &[2.0, 0.5]), 1.0 - 1.0 + 0.125);
        let message = cerr("legendrep", &[-1.0, 0.5]);
        assert!(message.contains("must be >= 0"), "{message}");
    }

    #[test]
    fn descriptive_statistics() {
        assert_eq!(c("mean", &[1.0, 2.0, 3.0, 4.0]), 2.5);
        assert_eq!(c("median", &[3.0, 1.0, 2.0]), 2.0);
        assert_eq!(c("median", &[4.0, 1.0, 3.0, 2.0]), 2.5);
        // Unbiased (n-1) variance of 1..5 is 2.5.
        close(c("variance", &[1.0, 2.0, 3.0, 4.0, 5.0]), 2.5);
        close(c("var", &[1.0, 2.0, 3.0, 4.0, 5.0]), 2.5);
        assert_eq!(c("variance", &[7.0]), 0.0);
        close(c("stdev", &[1.0, 2.0, 3.0, 4.0, 5.0]), libm::sqrt(2.5));
        close(c("std", &[1.0, 2.0, 3.0, 4.0, 5.0]), libm::sqrt(2.5));
        close(c("stddev", &[1.0, 2.0, 3.0, 4.0, 5.0]), libm::sqrt(2.5));
        close(c("rms", &[3.0, 4.0]), libm::sqrt(12.5));
        close(c("average", &[1.0, 2.0, 6.0]), 3.0);
        close(c("avg", &[1.0, 2.0, 6.0]), 3.0);
        // Java's `average` returns 0 for an empty list; `mean` refuses.
        assert_eq!(c("average", &[]), 0.0);
        assert!(cerr("mean", &[]).contains("at least 1 argument"));
    }

    #[test]
    fn percentile_uses_the_apache_legacy_estimator() {
        // Apache's default estimator on sorted data: pos = p/100 * (n+1).
        assert_eq!(c("percentile", &[50.0, 1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
        assert_eq!(c("percentile", &[0.0, 5.0, 1.0, 3.0]), 1.0);
        assert_eq!(c("percentile", &[100.0, 5.0, 1.0, 3.0]), 5.0);
        close(c("percentile", &[25.0, 1.0, 2.0, 3.0, 4.0]), 1.25);
        let message = cerr("percentile", &[50.0]);
        assert!(message.contains("at least 2 argument"), "{message}");
    }

    #[test]
    fn normal_distribution_helpers() {
        close(c("normalcdf", &[0.0]), 0.5);
        close(c("normalcdf", &[1.0]), 0.841_344_746_068_543);
        close(c("normalcdf", &[10.0, 10.0, 2.0]), 0.5);
        close(
            c("normalpdf", &[0.0]),
            1.0 / libm::sqrt(2.0 * std::f64::consts::PI),
        );
        // P(-1 <= X <= 1) for the standard normal.
        close(
            c("probability", &[-1.0, 1.0, 0.0, 1.0]),
            0.682_689_492_137_086,
        );
        let message = cerr("normalcdf", &[0.0, 0.0, -1.0]);
        assert!(
            message.contains("standard deviation must be positive"),
            "{message}"
        );
        assert!(cerr("probability", &[0.0, 1.0, 0.0, 0.0]).contains("must be > 0"));
    }

    #[test]
    fn degenerate_complex_helpers_treat_a_real_as_z_with_zero_imaginary_part() {
        assert_eq!(c("real", &[2.5]), 2.5);
        assert_eq!(c("imag", &[2.5]), 0.0);
        assert_eq!(c("conj", &[2.5]), 2.5);
        assert_eq!(c("magnitude", &[-2.5]), 2.5);
        assert_eq!(c("angle", &[2.5]), 0.0);
        close(c("anglerad", &[-2.5]), std::f64::consts::PI);
        close(c("angledeg", &[-2.5]), 180.0);
        close(c("cis", &[0.0]), 1.0);
    }

    #[test]
    fn pi_is_available_as_both_a_constant_and_a_nullary_call() {
        assert_eq!(c("pi", &[]), std::f64::consts::PI);
        assert_eq!(ev(&Expr::var("pi#")), std::f64::consts::PI);
        let message = cerr("pi", &[1.0]);
        assert!(message.contains("0 arguments"), "{message}");
    }

    // -- if --------------------------------------------------------------------

    #[test]
    fn three_argument_if_selects_on_truthiness() {
        let e = |condition: f64| ev(&Expr::call("if", vec![n(condition), n(10.0), n(20.0)]));
        assert_eq!(e(1.0), 10.0);
        assert_eq!(e(-0.5), 10.0);
        assert_eq!(e(0.0), 20.0);
    }

    #[test]
    fn three_argument_if_only_evaluates_the_taken_branch() {
        // if(1, 5, 1/0) must not raise: the else branch is never touched.
        let taken = Expr::call(
            "if",
            vec![n(1.0), n(5.0), Expr::bin(BinOp::Div, n(1.0), n(0.0))],
        );
        assert_eq!(ev(&taken), 5.0);

        let not_taken = Expr::call(
            "if",
            vec![n(0.0), Expr::bin(BinOp::Div, n(1.0), n(0.0)), n(5.0)],
        );
        assert_eq!(ev(&not_taken), 5.0);
    }

    #[test]
    fn five_argument_if_is_the_frees_three_way_branch() {
        let e = |a: f64, b: f64| ev(&Expr::call("if", vec![n(a), n(b), n(-1.0), n(0.0), n(1.0)]));
        assert_eq!(e(1.0, 2.0), -1.0);
        assert_eq!(e(2.0, 2.0), 0.0);
        assert_eq!(e(3.0, 2.0), 1.0);
    }

    #[test]
    fn five_argument_if_evaluates_only_the_chosen_branch() {
        let boom = || Expr::bin(BinOp::Div, n(1.0), n(0.0));
        // a < b: only the `lt` branch runs.
        let e = Expr::call("if", vec![n(1.0), n(2.0), n(7.0), boom(), boom()]);
        assert_eq!(ev(&e), 7.0);
        // a > b: only the `gt` branch runs.
        let e = Expr::call("if", vec![n(3.0), n(2.0), boom(), boom(), n(9.0)]);
        assert_eq!(ev(&e), 9.0);
        // a == b: only the `eq` branch runs.
        let e = Expr::call("if", vec![n(2.0), n(2.0), boom(), n(8.0), boom()]);
        assert_eq!(ev(&e), 8.0);
    }

    /// **Recorded divergence from `Evaluator.evalIf`.**
    ///
    /// Java writes the three-way branch as `if (a < b) … else if (a == b) … else
    /// return eval(args.get(4))`, so when either operand is NaN both tests are
    /// false and it silently takes the *greater-than* branch. This port refuses
    /// instead, which is the honest answer but is not what the oracle does: a
    /// document that reaches `If` with a NaN gets a number from Java and a
    /// non-finite residual here. Pinned so the difference is visible rather than
    /// discovered during a parity run.
    #[test]
    fn five_argument_if_refuses_a_nan_comparison_where_java_falls_through() {
        // inf/inf is the reachable NaN: `/` only raises when the divisor is 0.
        let nan = Expr::bin(
            BinOp::Div,
            Expr::call("exp", vec![n(1000.0)]),
            Expr::call("exp", vec![n(1000.0)]),
        );
        assert!(ev(&nan).is_nan(), "premise: the operand really is NaN");

        let e = Expr::call("if", vec![nan, n(2.0), n(-1.0), n(0.0), n(1.0)]);
        let message = err(&e);
        assert!(
            message.contains("cannot compare"),
            "expected the NaN refusal, got {message}"
        );
        // Java would have returned the `gt` branch, 1.0, here.
    }

    #[test]
    fn if_rejects_arities_other_than_three_or_five() {
        for count in [0usize, 1, 2, 4, 6] {
            let args: Vec<Expr> = (0..count).map(|i| n(i as f64)).collect();
            let message = err(&Expr::call("if", args));
            assert!(message.contains("3 or 5 arguments"), "{count}: {message}");
        }
    }

    // -- sum / product ----------------------------------------------------------

    #[test]
    fn sum_and_product_over_a_plain_argument_list() {
        assert_eq!(c("sum", &[1.0, 2.0, 3.0]), 6.0);
        assert_eq!(c("sum", &[]), 0.0);
        assert_eq!(c("product", &[2.0, 3.0, 4.0]), 24.0);
        assert_eq!(c("product", &[]), 1.0);
    }

    #[test]
    fn sum_binds_its_index_variable_over_the_body() {
        // sum(i, 1, 4, i) = 10
        let e = Expr::call("sum", vec![Expr::var("i"), n(1.0), n(4.0), Expr::var("i")]);
        assert_eq!(ev(&e), 10.0);

        // sum(i, 1, 4, i^2) = 30
        let e = Expr::call(
            "sum",
            vec![
                Expr::var("i"),
                n(1.0),
                n(4.0),
                Expr::bin(BinOp::Pow, Expr::var("i"), n(2.0)),
            ],
        );
        assert_eq!(ev(&e), 30.0);
    }

    #[test]
    fn product_binds_its_index_variable_over_the_body() {
        // product(k, 1, 5, k) = 120
        let e = Expr::call(
            "product",
            vec![Expr::var("k"), n(1.0), n(5.0), Expr::var("k")],
        );
        assert_eq!(ev(&e), 120.0);
    }

    #[test]
    fn the_bound_index_shadows_and_then_restores_the_outer_scope() {
        let s = scope(&[("i", 100.0)]);
        // sum(i, 1, 3, i) sees 1, 2, 3 — not the outer 100.
        let inner = Expr::call("sum", vec![Expr::var("i"), n(1.0), n(3.0), Expr::var("i")]);
        assert_eq!(eval(&inner, &s).unwrap(), 6.0);
        // The outer binding is untouched afterwards.
        assert_eq!(eval(&Expr::var("i"), &s).unwrap(), 100.0);
        // And an `i` outside the body still resolves to the outer value.
        let mixed = Expr::bin(BinOp::Add, inner, Expr::var("i"));
        assert_eq!(eval(&mixed, &s).unwrap(), 106.0);
    }

    #[test]
    fn a_reduction_body_still_sees_the_enclosing_scope() {
        // sum(i, 1, 3, i * gain) with gain = 2 -> 12
        let s = scope(&[("gain", 2.0)]);
        let e = Expr::call(
            "sum",
            vec![
                Expr::var("i"),
                n(1.0),
                n(3.0),
                Expr::bin(BinOp::Mul, Expr::var("i"), Expr::var("gain")),
            ],
        );
        assert_eq!(eval(&e, &s).unwrap(), 12.0);
    }

    #[test]
    fn reduction_bounds_may_be_expressions_and_may_run_downward() {
        let s = scope(&[("nmax", 3.0)]);
        // sum(i, 1, nmax, i) = 6
        let e = Expr::call(
            "sum",
            vec![Expr::var("i"), n(1.0), Expr::var("nmax"), Expr::var("i")],
        );
        assert_eq!(eval(&e, &s).unwrap(), 6.0);

        // A descending range still runs (Java iterates with dir = -1).
        let down = Expr::call("sum", vec![Expr::var("i"), n(3.0), n(1.0), Expr::var("i")]);
        assert_eq!(ev(&down), 6.0);

        // lo == hi runs exactly once.
        let once = Expr::call(
            "product",
            vec![Expr::var("i"), n(4.0), n(4.0), Expr::var("i")],
        );
        assert_eq!(ev(&once), 4.0);
    }

    #[test]
    fn reduction_bounds_are_rounded_like_java() {
        // (int) Math.round(2.6) == 3
        let e = Expr::call("sum", vec![Expr::var("i"), n(1.0), n(2.6), Expr::var("i")]);
        assert_eq!(ev(&e), 6.0);
    }

    #[test]
    fn nested_reductions_bind_independently() {
        // sum(i, 1, 3, sum(j, 1, 2, i * j)) = sum_i 3*i = 18
        let inner = Expr::call(
            "sum",
            vec![
                Expr::var("j"),
                n(1.0),
                n(2.0),
                Expr::bin(BinOp::Mul, Expr::var("i"), Expr::var("j")),
            ],
        );
        let outer = Expr::call("sum", vec![Expr::var("i"), n(1.0), n(3.0), inner]);
        assert_eq!(ev(&outer), 18.0);
    }

    #[test]
    fn a_four_argument_reduction_without_a_variable_head_is_a_plain_list() {
        // sum(1, 2, 3, 4) — args[0] is not a Var, so it is not the bound form.
        assert_eq!(c("sum", &[1.0, 2.0, 3.0, 4.0]), 10.0);
        assert_eq!(c("product", &[1.0, 2.0, 3.0, 4.0]), 24.0);
    }

    #[test]
    fn a_runaway_reduction_is_refused_rather_than_hanging() {
        let e = Expr::call(
            "sum",
            vec![Expr::var("i"), n(1.0), n(1.0e9), Expr::var("i")],
        );
        let message = err(&e);
        assert!(message.contains("exceeds"), "{message}");
    }

    #[test]
    fn non_finite_reduction_bounds_are_refused() {
        let s = scope(&[("huge", f64::INFINITY)]);
        let e = Expr::call(
            "sum",
            vec![Expr::var("i"), n(1.0), Expr::var("huge"), Expr::var("i")],
        );
        let message = eval(&e, &s).unwrap_err().to_string();
        assert!(message.contains("not a finite number"), "{message}");

        // A finite but absurd bound is rejected too.
        let s = scope(&[("huge", 1.0e30)]);
        let e = Expr::call(
            "sum",
            vec![Expr::var("i"), n(1.0), Expr::var("huge"), Expr::var("i")],
        );
        let message = eval(&e, &s).unwrap_err().to_string();
        assert!(message.contains("out of range"), "{message}");
    }

    #[test]
    fn errors_inside_a_reduction_body_propagate() {
        // sum(i, -1, 1, ln(i)) fails at i = -1.
        let e = Expr::call(
            "sum",
            vec![
                Expr::var("i"),
                n(-1.0),
                n(1.0),
                Expr::call("ln", vec![Expr::var("i")]),
            ],
        );
        assert!(err(&e).contains("logarithm of a negative number"));
    }

    // -- string intrinsics -------------------------------------------------------

    #[test]
    fn string_intrinsics_read_literal_arguments() {
        let len = Expr::call("stringlen", vec![Expr::Str("hello".into())]);
        assert_eq!(ev(&len), 5.0);
        let val = Expr::call("stringval", vec![Expr::Str("  2.5 ".into())]);
        assert_eq!(ev(&val), 2.5);
        let pos = Expr::call(
            "stringpos",
            vec![Expr::Str("abcdef".into()), Expr::Str("cd".into())],
        );
        assert_eq!(ev(&pos), 3.0);
        let missing = Expr::call(
            "stringpos",
            vec![Expr::Str("abcdef".into()), Expr::Str("zz".into())],
        );
        assert_eq!(ev(&missing), 0.0);
    }

    #[test]
    fn string_intrinsics_reject_non_string_arguments() {
        let message = err(&Expr::call("stringlen", vec![n(1.0)]));
        assert!(message.contains("expected a string argument"), "{message}");
        let bad = err(&Expr::call("stringval", vec![Expr::Str("nope".into())]));
        assert!(bad.contains("is not a number"), "{bad}");
    }

    // -- error paths ---------------------------------------------------------------

    #[test]
    fn unknown_functions_say_so() {
        let message = err(&Expr::call("frobnicate", vec![n(1.0)]));
        assert_eq!(message, "evaluation error: unknown function: frobnicate");
    }

    #[test]
    fn unported_java_arms_are_refused_by_name_and_family() {
        for (name, family) in [
            ("det", MATRIX_FAMILY),
            ("bode", CONTROL_FAMILY),
            ("lqr", CONTROL_FAMILY),
        ] {
            let message = err(&Expr::call(name, vec![n(1.0)]));
            assert!(
                message.contains(&format!("not yet supported: {name}")),
                "{name}: {message}"
            );
            assert!(message.contains(family), "{name}: {message}");
        }
    }

    /// The twenty accessors Phase 7/8 registered must be *reachable*, and with
    /// no context installed each must answer the Java's null-context default
    /// rather than erroring — that is what makes `MaxValue('h')` harmless in a
    /// document with no `DYNAMIC` block.
    #[test]
    fn the_accessors_are_registered_and_default_to_the_null_context_answer() {
        let scope = Scope::default();
        for name in [
            "odevalue",
            "finalvalue",
            "maxvalue",
            "minvalue",
            "timeat",
            "odeavg",
            "odesum",
            "odestddev",
            "odemin",
            "odemax",
        ] {
            assert!(lookup_intrinsic(name).is_some(), "{name} is not registered");
            // One column argument, plus the numeric second argument the two
            // time-indexed accessors take — `Arity::Range(1, 2)` accepts both.
            let call = Expr::call(name, vec![Expr::Str("h".into()), n(1.0)]);
            assert_eq!(eval(&call, &scope).unwrap(), 0.0, "{name}");
        }

        for (name, args, expected) in [
            // `TableRun#` reports **1** outside a sweep, not 0 — a document that
            // is not being swept is conceptually on its only run. The other nine
            // default to 0. Both defaults are `analysis::parametric`'s, which
            // transcribed them from `ParametricAccessorContext`.
            ("tablerun#", vec![], 1.0),
            ("tablerun", vec![], 1.0),
            ("nparametricruns", vec![], 0.0),
            ("tablevalue", vec![n(1.0), n(1.0)], 0.0),
            ("tablesum", vec![Expr::Str("x".into())], 0.0),
            ("tableavg", vec![Expr::Str("x".into())], 0.0),
            ("tablemin", vec![Expr::Str("x".into())], 0.0),
            ("tablemax", vec![Expr::Str("x".into())], 0.0),
            ("tablestddev", vec![Expr::Str("x".into())], 0.0),
            (
                "integralvalue",
                vec![Expr::Str("y".into()), Expr::Str("x".into())],
                0.0,
            ),
        ] {
            assert!(lookup_intrinsic(name).is_some(), "{name} is not registered");
            assert_eq!(
                eval(&Expr::call(name, args), &scope).unwrap(),
                expected,
                "{name}"
            );
        }
    }

    /// An accessor's column argument goes through `string_arg`, which accepts a
    /// quoted literal *or* a bare identifier — the Java `evalString`'s
    /// backward-compatible pair — and refuses anything else by name.
    #[test]
    fn an_accessor_column_must_be_a_string_or_a_bare_name() {
        let scope = Scope::default();
        assert_eq!(
            eval(&Expr::call("finalvalue", vec![Expr::var("temp")]), &scope).unwrap(),
            0.0
        );
        let message = err(&Expr::call("finalvalue", vec![n(3.0)]));
        assert!(message.contains("expected a string argument"), "{message}");
    }

    /// The self-consistency invariant this table has broken twice: a name may
    /// be **either** registered **or** listed as unported, never both. A stale
    /// `UNPORTED` row shadows nothing (the registry is consulted first), so the
    /// only symptom would be a lie in the language reference.
    #[test]
    fn no_registered_intrinsic_is_also_listed_as_unported() {
        let stale: Vec<&str> = UNPORTED
            .iter()
            .map(|(name, _)| *name)
            .filter(|name| lookup_intrinsic(name).is_some())
            .collect();
        assert!(
            stale.is_empty(),
            "registered but listed unported: {stale:?}"
        );
    }

    /// Every intrinsic Phase 5 wired must actually be reachable by name — the
    /// mirror image of the test above, so deleting a registration without
    /// deleting its claim fails loudly.
    #[test]
    fn phase_five_property_intrinsics_are_all_registered() {
        for name in [
            "eos_z",
            "eos_volume",
            "eos_density",
            "eos_pressure",
            "eos_enthalpy",
            "eos_entropy",
            "eos_psat",
            "adiabaticflametemp",
            "adiabaticflametemperature",
            "flametemp",
            "adiabaticflametempeq",
            "flametemp_eq",
            "mix_mw",
            "mix_molarmass",
            "mix_cp",
            "mix_enthalpy",
            "mix_entropy",
            "mix_viscosity",
            "mix_conductivity",
            "eq_molefraction",
            "htc_1phase",
            "htc_evap",
            "htc_cond",
            "htc_extair",
            "dp_1phase",
            "dp_2phase",
            "dp_mueller_steinhagen",
            "dp_ms",
            "dp_2phase_avg",
        ] {
            assert!(lookup_intrinsic(name).is_some(), "{name} is not registered");
        }
    }

    #[test]
    fn synthetic_dollar_calls_are_refused_as_a_family() {
        // `det$`/`qr$`/`chol$`/`expm$`/`svd$` are deliberately *not* in this
        // list any more: they route into `crate::linalg`, exactly as the Java
        // `Evaluator.evalCall` routes them into `core.LinearAlgebra`. See
        // `linear_algebra_synthetics_are_dispatched`. `prop$…` left the list in
        // Phase 5 — see `property_synthetics_are_dispatched`, the whole
        // control-systems set left it in Phase 9 — see
        // `control_systems_synthetics_are_dispatched` — and `eigen$…` left it
        // when ledger item 34 closed, which leaves the Euler pair alone here.
        let name = "eulerdecompose$1$3";
        let message = err(&Expr::call(name, vec![n(1.0)]));
        assert!(message.contains("not yet supported"), "{name}: {message}");
        assert!(message.contains("synthetic"), "{name}: {message}");
    }

    /// Phase 9: `control::eval` claims 42 synthetic heads. Reaching them from
    /// here is what makes every control-systems `CALL` evaluable — without it
    /// the flattener emits equations whose right-hand sides cannot be computed.
    #[test]
    fn control_systems_synthetics_are_dispatched() {
        // series$<num|den>$<index>$<len1>$<len2> over (num1, den1, num2, den2),
        // each left-padded to its transfer function's length. Cascading
        // 1/(s+1) with 1/(s+2) gives a denominator s^2 + 3s + 2, so element 1
        // (0-based, descending powers) is 3.
        let value = ev(&Expr::call(
            "series$den$1$2$2",
            nums(&[0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 2.0]),
        ));
        assert!((value - 3.0).abs() < 1e-12, "{value}");
        // A malformed one is an evaluation error from `control::eval`, not the
        // family-level "not yet supported" refusal.
        let message = err(&Expr::call("routh$0", vec![]));
        assert!(!message.contains("not yet supported"), "{message}");
    }

    /// `prop$…` reaches `props::propfun`, which is what makes every fluid,
    /// solid-material and chemistry function in the language reachable.
    #[test]
    fn property_synthetics_are_dispatched() {
        // A solid material: no property backend needed, so this is a value.
        let k = ev(&Expr::call("prop$k_", vec![Expr::Str("Aluminum".into())]));
        assert_eq!(k, crate::props::solids::lookup("Aluminum", "k_").unwrap());
        // Chemistry: MolarMass('C8H18') = 0.11423 kg/mol.
        let m = ev(&Expr::call(
            "prop$molarmass",
            vec![Expr::Str("C8H18".into())],
        ));
        assert!((m - 0.11423).abs() < 1e-4, "{m}");
        // An ideal gas resolves without any backend at all.
        let h = ev(&Expr::call("prop$enthalpy$n2$t", vec![n(500.0)]));
        assert!(h.is_finite() && h > 0.0, "{h}");
        // A real fluid with no backend installed is refused *by state*, not
        // with the old blanket "not yet supported: prop$…".
        crate::props::propfun::test_without_backend(|| {
            let msg = err(&Expr::call(
                "prop$enthalpy$water$t$p",
                vec![n(300.0), n(101325.0)],
            ));
            assert!(msg.contains("Water"), "{msg}");
            assert!(msg.contains("T=300"), "{msg}");
            assert!(!msg.contains("not yet supported"), "{msg}");
        });
        // With the tables this build links, the same call answers — within the
        // error D1 measured. Oracle (CoolProp 8.0.0, tools/golden-dumper):
        // Enthalpy(Water, T=300 [K], P=101325 [Pa]) = 112654.89965464505.
        crate::props::propfun::test_with_builtin_tables(|| {
            let h = ev(&Expr::call(
                "prop$enthalpy$water$t$p",
                vec![n(300.0), n(101325.0)],
            ));
            let rel = (h - 112_654.899_654_645_05).abs() / 112_654.899_654_645_05;
            assert!(rel < 1e-4, "h = {h}, rel = {rel:e}");
        });
        // Malformed / unknown outputs still refuse.
        let msg = err(&Expr::call("prop$bogus$water$t$p", vec![n(1.0), n(2.0)]));
        assert!(msg.contains("Unknown property function: bogus"), "{msg}");
    }

    /// `parser::expand` emits `det$<n>` for any `det(A)` with `n > 3` (the
    /// closed-form cofactor expansion is O(n!)), so this arm is reachable from
    /// plain user text and has to agree with Commons Math `LUDecomposition`.
    #[test]
    fn linear_algebra_synthetics_are_dispatched() {
        // 4×4 diagonal: 2·3·4·5 = 120.
        close(
            c(
                "det$4",
                &[
                    2.0, 0.0, 0.0, 0.0, //
                    0.0, 3.0, 0.0, 0.0, //
                    0.0, 0.0, 4.0, 0.0, //
                    0.0, 0.0, 0.0, 5.0,
                ],
            ),
            120.0,
        );
        // One row swap flips the sign (Commons Math `getDeterminant`).
        close(
            c(
                "det$4",
                &[
                    0.0, 1.0, 0.0, 0.0, //
                    1.0, 0.0, 0.0, 0.0, //
                    0.0, 0.0, 1.0, 0.0, //
                    0.0, 0.0, 0.0, 1.0,
                ],
            ),
            -1.0,
        );
        // Singular: LU convention is exactly 0.0, never NaN.
        close(c("det$2", &[1.0, 1.0, 1.0, 1.0]), 0.0);
        close(c("chol$l$0$0$2", &[4.0, 0.0, 0.0, 9.0]), 2.0);
        close(c("svd$s$0$2$2", &[3.0, 0.0, 0.0, 2.0]), 3.0);
        // A wrong argument count is an evaluation error naming the shape, not
        // the "not yet supported" refusal.
        let msg = err(&Expr::call("det$4", vec![n(1.0)]));
        assert!(msg.contains("expected 4x4 = 16 entries"), "{msg}");
    }

    #[test]
    fn proc_synthetics_without_a_document_report_the_java_message() {
        // `Evaluator.evalProcedureOutput` throws "Unknown procedure output
        // call: …" when `defs` has no such PROCEDURE — including when there is
        // no document context at all, which is this case.
        let message = err(&Expr::call("proc$mypro$0", vec![n(1.0)]));
        assert!(
            message.contains("Unknown procedure output call: proc$mypro$0"),
            "{message}"
        );
        // Malformed shapes take the same arm, exactly like the Java's
        // `split("\\$", 3).length != 3` guard.
        let short = err(&Expr::call("proc$mypro", vec![n(1.0)]));
        assert!(
            short.contains("Unknown procedure output call: proc$mypro"),
            "{short}"
        );
    }

    /// The correlations that genuinely need a real-fluid backend now dispatch
    /// and fail **at the missing property**, naming it — not by refusing the
    /// function. `mix_*` and `eq_molefraction` do not need one at all: they are
    /// NASA-7 / ideal-gas math and answer outright.
    #[test]
    fn coolprop_backed_correlations_dispatch_and_name_what_they_cannot_reach() {
        // Aluminium-tube water loop: the correlation resolves the fluid alias,
        // then asks for viscosity at (P,T).
        let htc = || {
            Expr::call(
                "htc_1phase",
                vec![
                    Expr::Str("Water".into()),
                    n(1e5),
                    n(320.0),
                    n(0.1),
                    n(0.01),
                    n(1e-4),
                ],
            )
        };

        // A (P,h) split table stores no transport, so it must decline — and name
        // the fluid while doing it. Pinned to that backend rather than left to
        // the global slot: D9 put one in there that *can* serve this call, so the
        // ambient installation no longer decides the same way, and which test
        // ran first is not a premise an assertion may rest on.
        let message = crate::props::propfun::test_with_builtin_tables(|| {
            match eval(&htc(), &Scope::default()) {
                Ok(v) => panic!("expected the table backend to decline, got {v}"),
                Err(e) => e.to_string(),
            }
        });
        assert!(message.contains("Water"), "{message}");
        assert!(
            // a backend that declines the input pair / a (P,h) table asked for a
            // transport property it does not store.
            message.contains("not tabulated") || message.contains("needs a full property backend"),
            "{message}"
        );
        assert!(!message.contains("not yet supported"), "{message}");

        // The other half of D9: the accuracy path is precisely what stops this
        // correlation being unreachable, so under rustprop the same call has to
        // answer with a physical film coefficient.
        #[cfg(feature = "rustprop-backend")]
        {
            let h = crate::props::propfun::test_with_rustprop(|| {
                eval(&htc(), &Scope::default()).expect("rustprop serves Water transport at (P,T)")
            });
            assert!(
                h.is_finite() && h > 1e2 && h < 1e5,
                "htc_1phase(Water, 1 bar, 320 K, 0.1 m/s, D = 10 mm) = {h}"
            );
        }

        // Ideal-gas mixture properties need no backend and must answer.
        let mw = ev(&Expr::call(
            "mix_mw",
            vec![Expr::Str("N2:0.79,O2:0.21".into())],
        ));
        assert!((mw - 0.028_85).abs() < 5e-4, "{mw}");
        let cp = ev(&Expr::call(
            "mix_cp",
            vec![Expr::Str("N2:0.79,O2:0.21".into()), n(300.0)],
        ));
        assert!(cp > 900.0 && cp < 1100.0, "{cp}");
        let mu = ev(&Expr::call(
            "mix_viscosity",
            vec![Expr::Str("N2:0.79,O2:0.21".into()), n(300.0)],
        ));
        assert!(mu > 1e-5 && mu < 3e-5, "{mu}");
    }

    #[test]
    fn wrong_arity_is_reported_with_the_expected_shape() {
        let too_many = err(&Expr::call("sqrt", vec![n(1.0), n(2.0)]));
        assert!(
            too_many.contains("sqrt expects 1 argument, got 2"),
            "{too_many}"
        );
        let too_few = err(&Expr::call("atan2", vec![n(1.0)]));
        assert!(
            too_few.contains("atan2 expects 2 arguments, got 1"),
            "{too_few}"
        );
        let none = err(&Expr::call("abs", vec![]));
        assert!(none.contains("abs expects 1 argument, got 0"), "{none}");
    }

    #[test]
    fn every_error_is_an_evaluation_error_never_a_panic() {
        let cases = vec![
            Expr::var("missing"),
            Expr::Str("s".into()),
            Expr::bin(BinOp::Div, n(1.0), n(0.0)),
            Expr::call("sqrt", vec![n(-1.0)]),
            Expr::call("nope", vec![]),
            Expr::call("integral", vec![]),
            Expr::ArrayLiteral(vec![]),
            Expr::Range {
                start: Box::new(n(1.0)),
                end: Box::new(n(2.0)),
            },
            Expr::ArrayAccess {
                name: "a".into(),
                indices: vec![n(1.0)],
            },
        ];
        for case in cases {
            match eval(&case, &Scope::default()) {
                Err(FreesError::Evaluation { .. }) => {}
                other => panic!("expected an Evaluation error for {case:?}, got {other:?}"),
            }
        }
    }

    #[test]
    fn no_implemented_intrinsic_returns_a_silent_nan_on_a_legal_input() {
        // A representative sweep: every unary intrinsic on a benign argument.
        for name in [
            "abs",
            "sign",
            "sqrt",
            "cbrt",
            "sqr",
            "cube",
            "exp",
            "ln",
            "log10",
            "log2",
            "log",
            "floor",
            "ceil",
            "trunc",
            "int",
            "round",
            "sin",
            "cos",
            "tan",
            "asin",
            "acos",
            "atan",
            "arcsin",
            "arccos",
            "arctan",
            "sinh",
            "cosh",
            "tanh",
            "arcsinh",
            "arctanh",
            "step",
            "ramp",
            "erf",
            "erfc",
            "gamma",
            "loggamma",
            "factorial",
            "real",
            "imag",
            "conj",
            "magnitude",
            "angle",
            "anglerad",
            "angledeg",
            "cis",
            "mean",
            "median",
            "rms",
            "normalcdf",
            "normalpdf",
        ] {
            let v = c(name, &[0.5]);
            assert!(!v.is_nan(), "{name}(0.5) returned NaN");
        }
    }

    // -- composed / integration-ish -------------------------------------------------

    #[test]
    fn a_realistic_expression_tree_evaluates_end_to_end() {
        // eta = 1 - (T_c / T_h)
        let s = scope(&[("t_c", 300.0), ("t_h", 500.0)]);
        let eta = Expr::bin(
            BinOp::Sub,
            n(1.0),
            Expr::bin(BinOp::Div, Expr::var("T_c"), Expr::var("T_h")),
        );
        close(eval(&eta, &s).unwrap(), 0.4);

        // max(0, round(eta * 100, 1))
        let rounded = Expr::call(
            "max",
            vec![
                n(0.0),
                Expr::call("round", vec![Expr::bin(BinOp::Mul, eta, n(100.0)), n(1.0)]),
            ],
        );
        close(eval(&rounded, &s).unwrap(), 40.0);
    }

    #[test]
    fn a_conditional_over_a_bound_sum_composes() {
        // if(sum(i, 1, n, i) > 5, 1, 0) with n = 4 -> sum is 10 -> 1
        let s = scope(&[("n", 4.0)]);
        let total = Expr::call(
            "sum",
            vec![Expr::var("i"), n(1.0), Expr::var("n"), Expr::var("i")],
        );
        let e = Expr::call(
            "if",
            vec![
                Expr::Compare {
                    op: CmpOp::Gt,
                    left: Box::new(total),
                    right: Box::new(n(5.0)),
                },
                n(1.0),
                n(0.0),
            ],
        );
        assert_eq!(eval(&e, &s).unwrap(), 1.0);
    }

    #[test]
    fn env_lookup_walks_the_binding_chain() {
        let s = scope(&[("a", 1.0), ("b", 2.0)]);
        let root = Env::Root(&s);
        let inner = Env::Bind {
            name: "b",
            value: 99.0,
            parent: &root,
        };
        let innermost = Env::Bind {
            name: "cc",
            value: 7.0,
            parent: &inner,
        };
        assert_eq!(innermost.get("a"), Some(1.0));
        assert_eq!(innermost.get("b"), Some(99.0));
        assert_eq!(innermost.get("cc"), Some(7.0));
        assert_eq!(innermost.get("d"), None);
        assert_eq!(root.get("b"), Some(2.0));
    }

    #[test]
    fn eval_in_is_usable_directly_with_a_custom_environment() {
        let s = Scope::default();
        let root = Env::Root(&s);
        let bound = Env::Bind {
            name: "x",
            value: 4.0,
            parent: &root,
        };
        let e = Expr::call("sqrt", vec![Expr::var("x")]);
        assert_eq!(eval_in(&e, &bound).unwrap(), 2.0);
    }

    #[test]
    fn java_helpers_match_their_java_semantics() {
        assert_eq!(java_round(0.5), 1.0);
        assert_eq!(java_round(-0.5), 0.0);
        assert_eq!(java_round(1.5), 2.0);
        assert_eq!(java_round(-1.5), -1.0);
        assert!(java_round(f64::NAN).is_nan());

        assert_eq!(java_signum(-0.0), 0.0);
        assert!(java_signum(-0.0).is_sign_negative());
        assert!(java_signum(f64::NAN).is_nan());

        assert!(java_min(1.0, f64::NAN).is_nan());
        assert!(java_max(1.0, f64::NAN).is_nan());
        assert_eq!(java_min(2.0, 3.0), 2.0);
        assert_eq!(java_max(2.0, 3.0), 3.0);
        assert!(java_min(-0.0, 0.0).is_sign_negative());
        assert!(java_max(-0.0, 0.0).is_sign_positive());

        assert_eq!(gcd_i64(0, 5), 5);
        assert_eq!(gcd_i64(-12, -18), 6);
        assert!((mean(&[1.0, 2.0]) - 1.5).abs() < EPS);
    }

    // =====================================================================
    // Phase 4: registry surface
    // =====================================================================

    #[test]
    fn intrinsic_names_is_sorted_and_complete() {
        let names = intrinsic_names();
        assert_eq!(names.len(), INTRINSICS.len());
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        for probe in ["sin", "interpolate", "integral", "besselk1", "hx_ntu"] {
            assert!(names.contains(&probe), "{probe} missing");
        }
    }

    // =====================================================================
    // Phase 4: definition dispatch (EvalContext)
    // =====================================================================

    use crate::parser::defs::{Curve, Definitions, FunctionDef, FunctionTableDef, ProcStatement};

    fn linear_table(name: &str) -> FunctionTableDef {
        FunctionTableDef {
            name: name.to_string(),
            arg_names: vec!["x".into()],
            x_log: false,
            y_log: false,
            curves: vec![Curve {
                param: None,
                xs: vec![0.0, 1.0, 2.0],
                ys: vec![0.0, 10.0, 40.0],
            }],
            output_unit: None,
            arg_units: None,
        }
    }

    fn family_table(name: &str) -> FunctionTableDef {
        FunctionTableDef {
            name: name.to_string(),
            arg_names: vec!["x".into(), "p".into()],
            x_log: false,
            y_log: false,
            curves: vec![
                Curve {
                    param: Some(1.0),
                    xs: vec![0.0, 1.0],
                    ys: vec![0.0, 10.0],
                },
                Curve {
                    param: Some(3.0),
                    xs: vec![0.0, 1.0],
                    ys: vec![0.0, 30.0],
                },
            ],
            output_unit: None,
            arg_units: None,
        }
    }

    fn defs_with_table(table: FunctionTableDef) -> Definitions {
        Definitions {
            tables: vec![table],
            ..Definitions::default()
        }
    }

    fn eval_ctx(e: &Expr, scope: &Scope, defs: &Definitions) -> Result<f64> {
        eval_with(e, scope, EvalContext::with_defs(defs))
    }

    #[test]
    fn a_user_table_evaluates_by_name() {
        let defs = defs_with_table(linear_table("curve"));
        let e = Expr::call("curve", vec![n(0.5)]);
        assert_eq!(eval_ctx(&e, &Scope::default(), &defs).unwrap(), 5.0);
        // Family form with the second argument.
        let defs = defs_with_table(family_table("fam"));
        let e = Expr::call("fam", vec![n(1.0), n(2.0)]);
        assert_eq!(eval_ctx(&e, &Scope::default(), &defs).unwrap(), 20.0);
    }

    #[test]
    fn a_user_table_rejects_wrong_arity_with_the_java_message() {
        let defs = defs_with_table(linear_table("curve"));
        let e = Expr::call("curve", vec![n(1.0), n(2.0), n(3.0)]);
        let msg = eval_ctx(&e, &Scope::default(), &defs)
            .unwrap_err()
            .to_string();
        assert!(msg.contains("expects curve(x) or curve(x, param)"), "{msg}");
    }

    #[test]
    fn a_user_table_shadows_an_intrinsic_of_the_same_name() {
        // Java consults defs before the builtin switch, so a TABLE named `sin`
        // wins over the trigonometric intrinsic.
        let defs = defs_with_table(linear_table("sin"));
        let e = Expr::call("sin", vec![n(1.0)]);
        assert_eq!(eval_ctx(&e, &Scope::default(), &defs).unwrap(), 10.0);
        // Without the context the intrinsic still answers.
        close(eval(&e, &Scope::default()).unwrap(), libm::sin(1.0));
    }

    #[test]
    fn a_user_function_dispatches_into_call_function() {
        // FUNCTION double(x) := x * 2 — hand-built body per the frozen defs
        // contract. Until the procedures agent lands, call_function is a stub
        // that errors; afterwards this must evaluate to 14. Accept both, but
        // never "unknown function" (which would mean dispatch failed).
        let def = FunctionDef {
            name: "double".into(),
            params: vec!["x".into()],
            body: vec![ProcStatement::Assign {
                var_name: "double".into(),
                value: Expr::bin(BinOp::Mul, Expr::var("x"), Expr::num(2.0)),
            }],
            output_unit: None,
            param_units: None,
        };
        let defs = Definitions {
            functions: vec![def],
            ..Definitions::default()
        };
        let e = Expr::call("double", vec![n(7.0)]);
        match eval_ctx(&e, &Scope::default(), &defs) {
            Ok(v) => assert_eq!(v, 14.0),
            Err(err) => {
                let msg = err.to_string();
                assert!(
                    !msg.contains("unknown function"),
                    "dispatch fell through: {msg}"
                );
            }
        }
    }

    #[test]
    fn plain_eval_with_empty_context_matches_eval() {
        let e = Expr::call("sqrt", vec![n(9.0)]);
        assert_eq!(
            eval_with(&e, &Scope::default(), EvalContext::default()).unwrap(),
            3.0
        );
    }

    // =====================================================================
    // Phase 4: classic-solver table functions
    // =====================================================================

    #[test]
    fn table_functions_resolve_the_named_table() {
        let defs = defs_with_table(linear_table("t"));
        let s = Scope::default();
        let call_str = |f: &str, args: Vec<Expr>| {
            let mut all = vec![Expr::Str("t".into())];
            all.extend(args);
            Expr::call(f, all)
        };
        assert_eq!(
            eval_ctx(&call_str("interpolate", vec![n(0.5)]), &s, &defs).unwrap(),
            5.0
        );
        assert_eq!(
            eval_ctx(&call_str("nlookuprows", vec![]), &s, &defs).unwrap(),
            3.0
        );
        assert_eq!(
            eval_ctx(&call_str("lookup", vec![n(2.0), n(2.0)]), &s, &defs).unwrap(),
            10.0
        );
        assert_eq!(
            eval_ctx(&call_str("lookuprow", vec![n(2.0), n(25.0)]), &s, &defs).unwrap(),
            2.5
        );
        // Segment slope of the linear interpolant between rows 2 and 3.
        assert_eq!(
            eval_ctx(&call_str("dtable", vec![n(1.5)]), &s, &defs).unwrap(),
            30.0
        );
        assert_eq!(
            eval_ctx(
                &call_str("differentiate", vec![n(2.0), n(1.0), n(1.5)]),
                &s,
                &defs
            )
            .unwrap(),
            30.0
        );
        // Table names are case-insensitive (lowercased before lookup).
        let upper = Expr::call("interpolate", vec![Expr::Str("T".into()), n(0.5)]);
        assert_eq!(eval_ctx(&upper, &s, &defs).unwrap(), 5.0);
    }

    #[test]
    fn interpolate1_uses_the_natural_spline() {
        // Parabola samples (0,0),(1,1),(2,4): natural spline value at 0.5 is
        // 0.3125 (worked in curvetable.rs tests).
        let table = FunctionTableDef {
            name: "p".into(),
            arg_names: vec!["x".into()],
            x_log: false,
            y_log: false,
            curves: vec![Curve {
                param: None,
                xs: vec![0.0, 1.0, 2.0],
                ys: vec![0.0, 1.0, 4.0],
            }],
            output_unit: None,
            arg_units: None,
        };
        let defs = defs_with_table(table);
        let e = Expr::call("interpolate1", vec![Expr::Str("p".into()), n(0.5)]);
        close(eval_ctx(&e, &Scope::default(), &defs).unwrap(), 0.3125);
        let d = Expr::call("dtable1", vec![Expr::Str("p".into()), n(1.0)]);
        close(eval_ctx(&d, &Scope::default(), &defs).unwrap(), 2.0);
    }

    #[test]
    fn interpolate2d_blends_a_curve_family() {
        let defs = defs_with_table(family_table("fam"));
        let e = Expr::call(
            "interpolate2d",
            vec![Expr::Str("fam".into()), n(1.0), n(2.0)],
        );
        assert_eq!(eval_ctx(&e, &Scope::default(), &defs).unwrap(), 20.0);
    }

    #[test]
    fn table_functions_without_a_matching_table_error_like_java() {
        let e = Expr::call("interpolate", vec![Expr::Str("nope".into()), n(1.0)]);
        // No context at all: same "'nope' is not a TABLE" as Java with Map.of().
        let msg = err(&e);
        assert!(msg.contains("'nope' is not a TABLE"), "{msg}");
    }

    // =====================================================================
    // Phase 4: quadrature dispatch
    // =====================================================================

    #[test]
    fn integral_with_equal_bounds_is_zero_without_touching_the_kernel() {
        let e = Expr::call(
            "integral",
            vec![Expr::var("t"), Expr::var("t"), n(2.0), n(2.0)],
        );
        assert_eq!(ev(&e), 0.0);
        let g = Expr::call(
            "gaussintegral",
            vec![Expr::var("t"), Expr::var("t"), n(-1.0), n(-1.0)],
        );
        assert_eq!(ev(&g), 0.0);
    }

    #[test]
    fn integral_requires_a_variable_as_its_second_argument() {
        let e = Expr::call("integral", vec![Expr::var("t"), n(3.0), n(0.0), n(1.0)]);
        let msg = err(&e);
        assert!(msg.contains("Integral expects"), "{msg}");
    }

    #[test]
    fn integral_dispatches_into_the_quadrature_kernel() {
        // ∫₀³ t² dt = 9. Until the integral agent lands its kernel the stub
        // reports "not yet supported"; afterwards the value must be right.
        let e = Expr::call(
            "integral",
            vec![
                Expr::bin(BinOp::Mul, Expr::var("t"), Expr::var("t")),
                Expr::var("t"),
                n(0.0),
                n(3.0),
            ],
        );
        match eval(&e, &Scope::default()) {
            Ok(v) => assert!((v - 9.0).abs() < 1e-6, "got {v}"),
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("not yet supported"), "{msg}");
            }
        }
    }

    // =====================================================================
    // Phase 4: synthetic $-calls
    // =====================================================================

    fn nums(values: &[f64]) -> Vec<Expr> {
        values.iter().copied().map(n).collect()
    }

    #[test]
    fn fft_synthetics_evaluate_the_dft() {
        // DFT([0,1,0,1]) = [2, 0, −2, 0].
        let mut args = nums(&[0.0, 1.0, 0.0, 1.0]);
        args.extend(nums(&[0.0; 4]));
        assert_eq!(ev(&Expr::call("fft$re$2$4", args.clone())), -2.0);
        close(ev(&Expr::call("fft$im$1$4", args.clone())), 0.0);
        // Inverse of the flat spectrum [1,1,1,1] is the unit impulse.
        let mut flat = nums(&[1.0; 4]);
        flat.extend(nums(&[0.0; 4]));
        close(ev(&Expr::call("ifft$re$0$4", flat.clone())), 1.0);
        close(ev(&Expr::call("ifft$re$1$4", flat)), 0.0);
    }

    #[test]
    fn conv_synthetics_evaluate_the_convolution() {
        // [1,2,3] ⊛ [4,5] = [4,13,22,15].
        let args = nums(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(ev(&Expr::call("conv$1$3$2", args.clone())), 13.0);
        assert_eq!(ev(&Expr::call("conv$3$3$2", args)), 15.0);
    }

    #[test]
    fn linfit_synthetics_expose_slope_intercept_r2() {
        // (1,2),(2,3),(3,5): slope 1.5, intercept 1/3, R² = 27/28.
        let args = nums(&[1.0, 2.0, 3.0, 2.0, 3.0, 5.0]);
        close(ev(&Expr::call("linfit$slope$3", args.clone())), 1.5);
        close(
            ev(&Expr::call("linfit$intercept$3", args.clone())),
            1.0 / 3.0,
        );
        close(ev(&Expr::call("linfit$r2$3", args.clone())), 27.0 / 28.0);
        let msg = err(&Expr::call("linfit$median$3", args));
        assert!(msg.contains("Unknown LinFit output: median"), "{msg}");
    }

    #[test]
    fn polyfit_synthetics_recover_polynomial_coefficients() {
        // y = 1 − 2x + 0.5x² sampled at −1..4 (6 points).
        let xs = [-1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
        let mut args = nums(&xs);
        let ys: Vec<f64> = xs.iter().map(|&x| 1.0 - 2.0 * x + 0.5 * x * x).collect();
        args.extend(nums(&ys));
        close(ev(&Expr::call("polyfit$0$2$6", args.clone())), 1.0);
        close(ev(&Expr::call("polyfit$1$2$6", args.clone())), -2.0);
        close(ev(&Expr::call("polyfit$2$2$6", args)), 0.5);
    }

    #[test]
    fn interp2_synthetics_interpolate_the_grid() {
        // 5x5 grid of z = x² + y², queried at (1.5, 2.5): the Akima-based
        // piecewise bicubic is exact on quadratics → 8.5.
        let axis = [0.0, 1.0, 2.0, 3.0, 4.0];
        let mut args = nums(&axis);
        args.extend(nums(&axis));
        for &x in &axis {
            for &y in &axis {
                args.push(n(x * x + y * y));
            }
        }
        args.push(n(1.5));
        args.push(n(2.5));
        close(ev(&Expr::call("interp2$5$5", args)), 8.5);
    }

    // =====================================================================
    // Phase 4: vector-argument kernels (slope / intercept / r2 / Interp2)
    // =====================================================================

    fn arr(values: &[f64]) -> Expr {
        Expr::ArrayLiteral(nums(values))
    }

    fn grid(rows: &[&[f64]]) -> Expr {
        Expr::ArrayLiteral(rows.iter().map(|r| arr(r)).collect())
    }

    #[test]
    fn lin_fit_names_expose_the_same_numbers_as_the_linfit_synthetic() {
        // (1,2),(2,3),(3,5): slope 1.5, intercept 1/3, R² = 27/28 — the very
        // values `linfit_synthetics_expose_slope_intercept_r2` pins.
        let x = arr(&[1.0, 2.0, 3.0]);
        let y = arr(&[2.0, 3.0, 5.0]);
        close(ev(&Expr::call("slope", vec![x.clone(), y.clone()])), 1.5);
        close(
            ev(&Expr::call("intercept", vec![x.clone(), y.clone()])),
            1.0 / 3.0,
        );
        close(ev(&Expr::call("r2", vec![x, y])), 27.0 / 28.0);
    }

    #[test]
    fn lin_fit_names_reject_non_vector_and_ragged_arguments() {
        let msg = err(&Expr::call("slope", vec![n(1.0), arr(&[1.0, 2.0])]));
        assert!(msg.contains("xvals must be a list of numbers"), "{msg}");
        let msg = err(&Expr::call(
            "intercept",
            vec![arr(&[1.0, 2.0]), grid(&[&[1.0, 2.0], &[3.0, 4.0]])],
        ));
        assert!(msg.contains("yvals must be a 1-D list"), "{msg}");
        assert!(msg.contains("2x2 matrix"), "{msg}");
        let msg = err(&Expr::call(
            "r2",
            vec![arr(&[1.0, 2.0, 3.0]), arr(&[1.0, 2.0])],
        ));
        assert!(msg.contains("equal length"), "{msg}");
    }

    #[test]
    fn a_column_literal_reads_as_the_same_vector_as_a_row_literal() {
        // `[1; 2; 3]` (one cell per row) and `[1, 2, 3]` are the same vector.
        let column = grid(&[&[1.0], &[2.0], &[3.0]]);
        let row = arr(&[2.0, 3.0, 5.0]);
        close(ev(&Expr::call("slope", vec![column, row])), 1.5);
    }

    #[test]
    fn interp2_accepts_both_documented_argument_orders() {
        // 2x2 grid → bilinear. Z[i][j] = f(x[i], y[j]) with x = y = [0, 1] and
        // Z = [[0, 1], [1, 2]] is f(x, y) = x + y.
        let x = arr(&[0.0, 1.0]);
        let y = arr(&[0.0, 1.0]);
        let z = grid(&[&[0.0, 1.0], &[1.0, 2.0]]);
        // CALL order: Interp2(x, y, Z, xq, yq).
        close(
            ev(&Expr::call(
                "interp2",
                vec![x.clone(), y.clone(), z.clone(), n(1.0), n(1.0)],
            )),
            2.0,
        );
        close(
            ev(&Expr::call(
                "interp2",
                vec![x.clone(), y.clone(), z.clone(), n(0.5), n(0.5)],
            )),
            1.0,
        );
        // Query-first order: Interp2(xq, yq, x, y, Z).
        close(
            ev(&Expr::call("interp2", vec![n(1.0), n(1.0), x, y, z])),
            2.0,
        );
    }

    #[test]
    fn interp2_clamps_outside_the_grid_and_agrees_with_its_synthetic() {
        let axis: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let rows: Vec<Vec<f64>> = axis
            .iter()
            .map(|&x| axis.iter().map(|&y| x * x + y * y).collect())
            .collect();
        let z = Expr::ArrayLiteral(rows.iter().map(|r| arr(r)).collect());
        let call = |xq: f64, yq: f64| {
            ev(&Expr::call(
                "interp2",
                vec![arr(&axis), arr(&axis), z.clone(), n(xq), n(yq)],
            ))
        };
        // Same 5x5 quadratic the `interp2$5$5` test uses: exact at (1.5, 2.5).
        close(call(1.5, 2.5), 8.5);
        // Outside the grid the query clamps to the boundary (no extrapolation).
        close(call(99.0, 99.0), 32.0);
        close(call(-5.0, -5.0), 0.0);
    }

    #[test]
    fn interp2_needs_a_nested_literal_to_locate_the_grid() {
        let msg = err(&Expr::call(
            "interp2",
            vec![n(1.0), n(1.0), n(1.0), n(1.0), n(1.0)],
        ));
        assert!(msg.contains("Interp2(x, y, Z, xq, yq)"), "{msg}");
        assert!(msg.contains("Interp2(xq, yq, x, y, Z)"), "{msg}");
        // A 1xN literal is a vector, never the grid — the order stays pinned.
        let msg = err(&Expr::call(
            "interp2",
            vec![
                arr(&[0.0, 1.0]),
                arr(&[0.0, 1.0]),
                arr(&[0.0, 1.0]),
                n(0.5),
                n(0.5),
            ],
        ));
        assert!(msg.contains("m x n grid literal"), "{msg}");
        // A grid that does not match the axes is refused by the kernel.
        let msg = err(&Expr::call(
            "interp2",
            vec![
                arr(&[0.0, 1.0, 2.0]),
                arr(&[0.0, 1.0]),
                grid(&[&[0.0, 1.0], &[1.0, 2.0]]),
                n(0.5),
                n(0.5),
            ],
        ));
        assert!(msg.contains("must be 3x2"), "{msg}");
    }

    // =====================================================================
    // Phase 4: proc$ synthetic dispatch (Evaluator.evalProcedureOutput)
    // =====================================================================

    /// Evaluate with a document's definitions in context.
    fn ev_in_doc(source: &str, e: &Expr) -> Result<f64> {
        let doc = crate::parser::parse_document(source)
            .unwrap_or_else(|err| panic!("parse failed: {err}"));
        eval_with(e, &Scope::default(), EvalContext::with_defs(&doc.defs))
    }

    const SWAP_DOC: &str = "PROCEDURE p(a : b, c)\n  b := a * 2\n  c := a + 1\nEND\n";

    #[test]
    fn proc_synthetic_runs_the_body_and_picks_the_output_slot() {
        let call0 = Expr::call("proc$p$0", vec![n(3.0)]);
        let call1 = Expr::call("proc$p$1", vec![n(3.0)]);
        assert_eq!(ev_in_doc(SWAP_DOC, &call0).unwrap(), 6.0);
        assert_eq!(ev_in_doc(SWAP_DOC, &call1).unwrap(), 4.0);
    }

    #[test]
    fn proc_synthetic_evaluates_its_inputs_in_the_callers_scope() {
        let doc = crate::parser::parse_document(SWAP_DOC).unwrap();
        let scope = scope(&[("q", 5.0)]);
        let e = Expr::call(
            "proc$p$0",
            vec![Expr::bin(BinOp::Add, Expr::var("q"), n(1.0))],
        );
        let got = eval_with(&e, &scope, EvalContext::with_defs(&doc.defs)).unwrap();
        assert_eq!(got, 12.0);
    }

    #[test]
    fn proc_synthetic_reports_an_out_of_range_slot_and_an_unknown_name() {
        let msg = ev_in_doc(SWAP_DOC, &Expr::call("proc$p$9", vec![n(1.0)]))
            .unwrap_err()
            .to_string();
        assert!(msg.contains("Unknown procedure output call"), "{msg}");
        let msg = ev_in_doc(SWAP_DOC, &Expr::call("proc$nope$0", vec![n(1.0)]))
            .unwrap_err()
            .to_string();
        assert!(msg.contains("Unknown procedure output call"), "{msg}");
    }

    #[test]
    fn proc_synthetic_reports_an_input_arity_mismatch() {
        let msg = ev_in_doc(SWAP_DOC, &Expr::call("proc$p$0", vec![n(1.0), n(2.0)]))
            .unwrap_err()
            .to_string();
        assert!(msg.contains("expects 1 input(s), got 2"), "{msg}");
    }

    #[test]
    fn a_multi_output_function_is_reachable_through_the_same_arm() {
        // `FUNCTION [p, q] = two(u)` desugars to a ProcedureDef named `two`.
        let source = "FUNCTION [p, q] = two(u)\n  p := u\n  q := u * 2\nEND\n";
        assert_eq!(
            ev_in_doc(source, &Expr::call("proc$two$0", vec![n(4.0)])).unwrap(),
            4.0
        );
        assert_eq!(
            ev_in_doc(source, &Expr::call("proc$two$1", vec![n(4.0)])).unwrap(),
            8.0
        );
    }

    #[test]
    fn proc_synthetic_runs_repeat_until_and_conditionals_in_the_body() {
        let source = "PROCEDURE acc(n : total, parity)\n\
                      \x20 total := 0\n\
                      \x20 i := 1\n\
                      \x20 REPEAT\n\
                      \x20   total := total + i\n\
                      \x20   i := i + 1\n\
                      \x20 UNTIL i > n\n\
                      \x20 IF total > 10 THEN\n\
                      \x20   parity := 1\n\
                      \x20 ELSE\n\
                      \x20   parity := 0\n\
                      \x20 END\n\
                      END\n";
        // 1+2+3+4+5 = 15 → parity 1; 1+2 = 3 → parity 0.
        assert_eq!(
            ev_in_doc(source, &Expr::call("proc$acc$0", vec![n(5.0)])).unwrap(),
            15.0
        );
        assert_eq!(
            ev_in_doc(source, &Expr::call("proc$acc$1", vec![n(5.0)])).unwrap(),
            1.0
        );
        assert_eq!(
            ev_in_doc(source, &Expr::call("proc$acc$0", vec![n(2.0)])).unwrap(),
            3.0
        );
        assert_eq!(
            ev_in_doc(source, &Expr::call("proc$acc$1", vec![n(2.0)])).unwrap(),
            0.0
        );
    }

    #[test]
    fn proc_synthetic_runs_while_and_for_bodies() {
        // WHILE … DO … END with an assignment body.
        let while_doc = "PROCEDURE dbl(seed : out)\n\
                         \x20 out := seed\n\
                         \x20 WHILE out < 100 DO\n\
                         \x20   out := out * 2\n\
                         \x20 END\n\
                         END\n";
        assert_eq!(
            ev_in_doc(while_doc, &Expr::call("proc$dbl$0", vec![n(3.0)])).unwrap(),
            192.0
        );
        // A FOR inside a procedural body carries *equations*, not `:=`
        // assignments — `AstBuilder.toProcStatement` converts only equations and
        // nested FORs, so this port's parser rejects `:=` there exactly as Java
        // does. The loop variable survives the loop with its final value.
        let for_doc = "PROCEDURE ramp(n : last)\n\
                       \x20 FOR i = 1 TO 3\n\
                       \x20   last = i * 2\n\
                       \x20 END\n\
                       END\n";
        assert_eq!(
            ev_in_doc(for_doc, &Expr::call("proc$ramp$0", vec![n(0.0)])).unwrap(),
            6.0
        );
    }

    #[test]
    fn a_runaway_while_in_a_proc_body_is_refused_not_hung() {
        let source = "PROCEDURE spin(seed : out)\n\
                      \x20 out := seed\n\
                      \x20 WHILE out > 0 DO\n\
                      \x20   out := out + 1\n\
                      \x20 END\n\
                      END\n";
        let msg = ev_in_doc(source, &Expr::call("proc$spin$0", vec![n(1.0)]))
            .unwrap_err()
            .to_string();
        assert!(msg.contains("WHILE loop exceeded"), "{msg}");
    }

    #[test]
    fn synthetic_calls_validate_shape_and_bounds() {
        let msg = err(&Expr::call("fft$re$0$4", nums(&[1.0, 2.0])));
        assert!(msg.contains("expects 8 argument(s), got 2"), "{msg}");
        let mut args = nums(&[0.0; 8]);
        args[0] = n(1.0);
        let msg = err(&Expr::call("fft$re$9$4", args));
        assert!(msg.contains("out of range"), "{msg}");
        let msg = err(&Expr::call("conv$x$2$2", nums(&[1.0; 4])));
        assert!(msg.contains("malformed synthetic call"), "{msg}");
    }

    #[test]
    fn unported_synthetic_families_still_refuse_honestly() {
        // `det$` left this list when `crate::linalg` was wired in; `prop$` left
        // it in Phase 5 when `crate::props::propfun` was; the control-systems
        // heads left it in Phase 9 when `crate::control::eval` was; `eigen$`
        // left it when ledger item 34 closed, leaving the Euler pair alone.
        let name = "eulerrotate$1$3";
        let msg = err(&Expr::call(name, vec![n(1.0)]));
        assert!(msg.contains("not yet supported"), "{name}: {msg}");
    }

    // =====================================================================
    // Phase 4: special functions
    // =====================================================================

    #[test]
    fn erfinv_matches_apache_values() {
        close(c("erfinv", &[0.5]), 0.4769362762044699);
        close(c("erfinv", &[0.9]), 1.1630871536766743);
        close(c("erfinv", &[-0.3]), -0.2724627147267544);
        assert_eq!(c("erfinv", &[0.0]), 0.0);
        assert_eq!(c("erfinv", &[1.0]), f64::INFINITY);
        assert_eq!(c("erfinv", &[-1.0]), f64::NEG_INFINITY);
        // erf ∘ erfinv round trip.
        close(c("erf", &[c("erfinv", &[0.42])]), 0.42);
    }

    #[test]
    fn digamma_matches_apache_values() {
        // These are the outputs of the *Apache algorithm* (AS 103 with the
        // C_LIMIT = 49 recurrence), which accumulates ~1e-9 of rounding against
        // the true ψ — Java parity means matching Apache's values, not scipy's.
        close(c("digamma", &[1.0]), -0.5772156677920671);
        close(c("digamma", &[4.7]), 1.4374238069006702);
        close(c("digamma", &[-1.5]), 0.7031566378697294);
        close(c("digamma", &[120.0]), 4.783319289038156);
        // At x ≥ 49 the direct expansion is ~1e-10 from the true digamma.
        assert!((c("digamma", &[120.0]) - 4.783319289118529).abs() < 1e-9);
    }

    #[test]
    fn chi_square_is_the_regularized_gamma_cdf() {
        close(c("chi_square", &[1.0, 1.0]), 0.6826894921370859);
        close(c("chi_square", &[2.0, 2.0]), 0.6321205588285577);
        close(c("chi_square", &[7.5, 4.0]), 0.8882907071839568);
        assert_eq!(c("chi_square", &[-1.0, 3.0]), 0.0);
        assert_eq!(c("chi_square", &[0.0, 3.0]), 0.0);
        let msg = cerr("chi_square", &[1.0, 0.0]);
        assert!(msg.contains("degrees of freedom"), "{msg}");
    }

    #[test]
    fn normalinvcdf_matches_the_apache_normal_distribution() {
        close(c("normalinvcdf", &[0.975]), 1.9599639845400538);
        close(c("normalinvcdf", &[0.3, 10.0, 2.0]), 8.951198974583917);
        assert_eq!(c("normalinvcdf", &[0.5]), 0.0);
        assert_eq!(c("normalinvcdf", &[0.0]), f64::NEG_INFINITY);
        assert_eq!(c("normalinvcdf", &[1.0]), f64::INFINITY);
        let msg = cerr("normalinvcdf", &[1.5]);
        assert!(msg.contains("probability"), "{msg}");
        let msg = cerr("normalinvcdf", &[0.5, 0.0, -1.0]);
        assert!(msg.contains("standard deviation"), "{msg}");
    }

    // =====================================================================
    // Phase 4.1: Student-t, F, regularized beta, descriptive stats
    // =====================================================================

    #[test]
    fn student_t_cdf_known_values() {
        // t(0, 1) = 0.5 for any df.
        close(c("tcdf", &[0.0, 5.0]), 0.5);
        // Reference (Simpson quadrature of the incomplete beta): 0.94903026058507.
        close(c("tcdf", &[2.0, 5.0]), 0.949_030_260_585_07);
        // Negative argument: symmetry.
        let neg = c("tcdf", &[-2.0, 5.0]);
        close(neg, 1.0 - 0.949_030_260_585_07);
        // Large df approaches normal: 0.97498824 vs the normal 0.975.
        let large_df = c("tcdf", &[1.96, 10000.0]);
        assert!((large_df - 0.975).abs() < 1e-4, "got {large_df}");
    }

    #[test]
    fn student_t_pdf_at_zero() {
        // t PDF at x=0 with df=1 (Cauchy) = 1/π ≈ 0.31831
        let val = c("tpdf", &[0.0, 1.0]);
        close(val, 1.0 / std::f64::consts::PI);
    }

    #[test]
    fn student_t_inv_round_trips() {
        // tinv(tcdf(x, df), df) ≈ x.
        let x = 1.5;
        let df = 10.0;
        let p = c("tcdf", &[x, df]);
        let recovered = c("tinv", &[p, df]);
        close(recovered, x);
    }

    #[test]
    fn student_t_inv_critical_values() {
        // Standard t-table value t(0.975, 9) = 2.262157162798.
        let val = c("tinv", &[0.975, 9.0]);
        close(val, 2.262_157_162_798);
    }

    #[test]
    fn f_dist_cdf_known_values() {
        // fcdf(0, d1, d2) = 0 for all d1, d2 > 0.
        assert_eq!(c("fcdf", &[0.0, 5.0, 10.0]), 0.0);
        // F(5, 10) CDF at 3 = I_0.6(2.5, 5); reference quadrature: 0.93444243790617.
        close(c("fcdf", &[3.0, 5.0, 10.0]), 0.934_442_437_906_17);
    }

    #[test]
    fn f_dist_pdf_positive() {
        let val = c("fpdf", &[1.0, 5.0, 10.0]);
        assert!(val > 0.0, "F PDF at x=1 should be positive, got {val}");
    }

    #[test]
    fn f_dist_inv_round_trips() {
        let x = 2.5;
        let d1 = 3.0;
        let d2 = 15.0;
        let p = c("fcdf", &[x, d1, d2]);
        let recovered = c("finv", &[p, d1, d2]);
        close(recovered, x);
    }

    #[test]
    fn regularized_beta_boundary_values() {
        assert_eq!(c("betainc", &[0.0, 2.0, 3.0]), 0.0);
        assert_eq!(c("betainc", &[1.0, 2.0, 3.0]), 1.0);
        // I_{0.5}(1, 1) = 0.5 (uniform distribution).
        close(c("betainc", &[0.5, 1.0, 1.0]), 0.5);
    }

    #[test]
    fn regularized_beta_known_value() {
        // Integer a, b close form: I_0.3(2, 5) = 1 - 0.7^6 - 6(0.3)(0.7^5) = 0.579825.
        close(c("betainc", &[0.3, 2.0, 5.0]), 0.579_825);
    }

    #[test]
    fn skewness_dispatch() {
        // Symmetric data: skewness ≈ 0.
        close(c("skewness", &[1.0, 2.0, 3.0, 4.0, 5.0]), 0.0);
    }

    #[test]
    fn kurtosis_dispatch() {
        // [1, 2, 3, 4, 5]: excess kurtosis = -1.2.
        close(c("kurtosis", &[1.0, 2.0, 3.0, 4.0, 5.0]), -1.2);
    }

    #[test]
    fn mad_dispatch() {
        // [1, 2, 3, 4, 5]: median = 3, deviations = [2, 1, 0, 1, 2], MAD = 1.
        close(c("mad", &[1.0, 2.0, 3.0, 4.0, 5.0]), 1.0);
    }

    // =====================================================================
    // Phase 4: Bessel functions
    // =====================================================================

    /// The fixed-order forms are the Numerical-Recipes approximations the Java
    /// transcribes; they are accurate to ~1e-7 against the true functions.
    fn close_nr(a: f64, b: f64) {
        assert!(
            (a - b).abs() <= 2e-7 * b.abs().max(1.0),
            "expected {b}, got {a}"
        );
    }

    #[test]
    fn fixed_order_bessel_values() {
        close_nr(c("besselj0", &[1.0]), 0.7651976865579666);
        close_nr(c("besselj0", &[10.0]), -0.24593576445134832);
        close_nr(c("besselj1", &[1.0]), 0.44005058574493355);
        close_nr(c("bessely0", &[1.0]), 0.088256964215677);
        close_nr(c("bessely1", &[1.0]), -0.7812128213002889);
        close_nr(c("besseli0", &[1.0]), 1.2660658777520084);
        close_nr(c("besseli1", &[1.0]), 0.565159103992485);
        close_nr(c("besselk0", &[1.0]), 0.42102443824070834);
        close_nr(c("besselk1", &[1.0]), 0.6019072301972346);
        // The bessel_* spellings are the same arms.
        close_nr(c("bessel_j0", &[1.0]), 0.7651976865579666);
        close_nr(c("bessel_k1", &[1.0]), 0.6019072301972346);
        // Odd symmetry of J1 and I1.
        close_nr(c("besselj1", &[-1.0]), -0.44005058574493355);
        close_nr(c("besseli1", &[-1.0]), -0.565159103992485);
    }

    #[test]
    fn arbitrary_order_bessel_values_take_x_then_order() {
        // The Java arms read the order from args[1]: besselY(order=a[1], x=a[0]).
        close_nr(c("bessely", &[1.0, 2.0]), -1.6506826068162548);
        assert!((c("besselk", &[1.0, 3.0]) - 7.101262824737944).abs() < 1e-5);
        // The I series is exact to machine precision, real orders included.
        close(c("besseli", &[2.0, 0.0]), 2.2795853023360673);
        close(c("besseli", &[1.5, 2.5]), 0.17166202218829626);
        // Integer negative argument uses the parity rule.
        close(c("besseli", &[-2.0, 1.0]), -c("besseli", &[2.0, 1.0]));
    }

    #[test]
    fn bessel_domain_errors_match_java() {
        assert!(cerr("bessely0", &[0.0]).contains("requires x > 0"));
        assert!(cerr("besselk0", &[-1.0]).contains("requires x > 0"));
        assert!(cerr("bessely", &[1.0, 0.5]).contains("integer order"));
        assert!(cerr("besselk", &[1.0, 0.5]).contains("integer order"));
        assert!(cerr("besseli", &[-1.0, 0.5]).contains("integer order"));
        assert!(cerr("besseli", &[1.0, -0.5]).contains("orders"));
    }

    // =====================================================================
    // Phase 4: seeded random numbers
    // =====================================================================

    #[test]
    fn seeded_random_reproduces_java_util_random() {
        close(c("random", &[0.0, 1.0, 42.0]), 0.7275636800328681);
        close(
            c("random", &[10.0, 20.0, 42.0]),
            10.0 + 0.7275636800328681 * 10.0,
        );
        close(c("random", &[0.0, 1.0, 123.0]), 0.7231742029971469);
        close(c("randg", &[0.0, 1.0, 42.0]), 1.141905315473055);
        close(c("randg", &[5.0, 2.0, 42.0]), 5.0 + 2.0 * 1.141905315473055);
    }

    #[test]
    fn unseeded_random_is_refused_as_nondeterministic() {
        assert!(cerr("random", &[0.0, 1.0]).contains("seed"));
        assert!(cerr("randg", &[0.0, 1.0]).contains("seed"));
        assert!(cerr("random", &[0.0, 1.0, 0.0]).contains("seed"));
    }

    // =====================================================================
    // Phase 4: strings, arrays, conversions
    // =====================================================================

    #[test]
    fn baseconvert_round_trips_between_bases() {
        let bc = |digits: Expr, from: f64, to: f64| {
            eval(
                &Expr::call("baseconvert", vec![digits, n(from), n(to)]),
                &Scope::default(),
            )
        };
        assert_eq!(bc(Expr::Str("FF".into()), 16.0, 10.0).unwrap(), 255.0);
        assert_eq!(bc(Expr::Str("11111111".into()), 2.0, 10.0).unwrap(), 255.0);
        assert_eq!(bc(n(255.0), 10.0, 2.0).unwrap(), 11111111.0);
        assert_eq!(bc(Expr::Str("-ff".into()), 16.0, 10.0).unwrap(), -255.0);
        let msg = bc(Expr::Str("255".into()), 10.0, 16.0)
            .unwrap_err()
            .to_string();
        assert!(msg.contains("letter digits"), "{msg}");
        assert!(msg.contains("FF"), "{msg}");
        let msg = bc(Expr::Str("12".into()), 40.0, 10.0)
            .unwrap_err()
            .to_string();
        assert!(msg.contains("between 2 and 36"), "{msg}");
        let msg = bc(Expr::Str("zz".into()), 10.0, 10.0)
            .unwrap_err()
            .to_string();
        assert!(msg.contains("not a valid base-10 number"), "{msg}");
        let msg = bc(n(2.5), 10.0, 2.0).unwrap_err().to_string();
        assert!(msg.contains("must be an integer"), "{msg}");
    }

    #[test]
    fn arrayelmt_selects_lazily_and_bounds_checks() {
        let e = Expr::call("arrayelmt", vec![n(10.0), n(20.0), n(30.0), n(2.0)]);
        assert_eq!(ev(&e), 20.0);
        // Index rounds like Java (half up): 2.5 → 3.
        let e = Expr::call("arrayelmt", vec![n(10.0), n(20.0), n(30.0), n(2.5)]);
        assert_eq!(ev(&e), 30.0);
        // Only the selected element is evaluated — a poisoned sibling is fine.
        let poison = Expr::bin(BinOp::Div, n(1.0), n(0.0));
        let e = Expr::call("arrayelmt", vec![n(10.0), poison, n(1.0)]);
        assert_eq!(ev(&e), 10.0);
        let msg = err(&Expr::call("arrayelmt", vec![n(10.0), n(20.0), n(5.0)]));
        assert!(msg.contains("out of range 1..2"), "{msg}");
    }

    #[test]
    fn uncertaintyof_reads_the_injected_scope_entry() {
        let mut s = Scope::default();
        s.insert("uncertaintyof$x".into(), 0.25);
        let e = Expr::call("uncertaintyof", vec![Expr::Str("X".into())]);
        assert_eq!(eval(&e, &s).unwrap(), 0.25);
        // Absent → 0.0 (no uncertainty pass ran), matching Java.
        let e = Expr::call("uncertaintyof", vec![Expr::var("y")]);
        assert_eq!(eval(&e, &s).unwrap(), 0.0);
    }

    // =====================================================================
    // Phase 4: stagnation properties & view factors
    // =====================================================================

    #[test]
    fn stagnation_properties_are_the_closed_forms() {
        // T0 = T + V²/(2cp)
        close(
            c("stagnationtemp", &[300.0, 100.0, 1005.0]),
            300.0 + 10000.0 / 2010.0,
        );
        // P0 = P (T0/T)^(k/(k-1))
        close(
            c("stagnationpres", &[100_000.0, 300.0, 320.0, 1.4]),
            100_000.0 * libm::pow(320.0 / 300.0, 3.5),
        );
    }

    #[test]
    fn view_factors_match_the_howell_closed_forms() {
        close(c("viewfactor_perp", &[1.0, 1.0, 1.0]), 0.20004377607540316);
        close(
            c("viewfactor_plates", &[1.0, 1.0, 1.0]),
            0.19982489569838746,
        );
        close(c("viewfactor_disks", &[1.0, 1.0, 1.0]), 0.3819660112501051);
        assert!(cerr("viewfactor_perp", &[1.0, -1.0, 1.0]).contains("positive"));
    }

    // =====================================================================
    // Phase 4: transient conduction (Heisler)
    // =====================================================================

    fn heisler(name: &str, geometry: &str, rest: &[f64]) -> Result<f64> {
        let mut args = vec![Expr::Str(geometry.into())];
        args.extend(rest.iter().copied().map(n));
        eval(&Expr::call(name, args), &Scope::default())
    }

    #[test]
    fn heisler_wall_matches_the_one_term_solution() {
        // Bi = 1: ζ₁ = 0.86033…, C₁ = 1.11913…
        close(
            heisler("heisler_temp", "wall", &[1.0, 0.5, 0.0]).unwrap(),
            0.7729556933327831,
        );
        close(
            heisler("heisler_temp", "wall", &[1.0, 0.5, 1.0]).unwrap(),
            0.5041098181547095,
        );
        close(
            heisler("heisler_q", "wall", &[1.0, 0.5]).unwrap(),
            0.3189305529648938,
        );
    }

    #[test]
    fn heisler_cylinder_uses_the_bessel_eigenvalue_problem() {
        close(
            heisler("heisler_temp", "cylinder", &[1.0, 0.5, 0.5]).unwrap(),
            0.4958980458136078,
        );
        close(
            heisler("heisler_q", "cyl", &[1.0, 0.5]).unwrap(),
            0.5526190515382895,
        );
    }

    #[test]
    fn heisler_sphere_bi_one_has_the_pi_over_two_eigenvalue() {
        // Bi = 1 for a sphere gives ζ₁ = π/2 exactly, C₁ = 4/π.
        let expected = 4.0 / std::f64::consts::PI
            * libm::exp(-(std::f64::consts::PI / 2.0) * (std::f64::consts::PI / 2.0) * 0.5);
        close(
            heisler("heisler_temp", "sphere", &[1.0, 0.5, 0.0]).unwrap(),
            expected,
        );
        let msg = heisler("heisler_temp", "cube", &[1.0, 0.5, 0.0])
            .unwrap_err()
            .to_string();
        assert!(msg.contains("'wall', 'cylinder' or 'sphere'"), "{msg}");
    }

    // =====================================================================
    // Phase 4: engineering correlations
    // =====================================================================

    #[test]
    fn isa_atmosphere_layers() {
        close(c("isa_t", &[0.0]), 288.15);
        close(c("isa_t", &[5000.0]), 255.64999999999998);
        close(c("isa_t", &[15000.0]), 216.64999999999998);
        close(c("isa_p", &[0.0]), 101_325.0);
        close(c("isa_p", &[5000.0]), 54020.49540145998);
        close(c("isa_p", &[15000.0]), 12045.011233214942);
        close(c("isa_rho", &[0.0]), 1.2249781262066513);
    }

    #[test]
    fn wiebe_burn_fraction_and_rate() {
        close(
            c("wiebe", &[370.0, 350.0, 40.0, 5.0, 2.0]),
            0.4647385714810097,
        );
        close(
            c("wiebe_rate", &[370.0, 350.0, 40.0, 5.0, 2.0]),
            0.05018075892365534,
        );
        assert_eq!(c("wiebe", &[340.0, 350.0, 40.0, 5.0, 2.0]), 0.0);
        assert!(cerr("wiebe", &[370.0, 350.0, 0.0, 5.0, 2.0]).contains("dtheta"));
    }

    #[test]
    fn iso6358_flow_regimes() {
        let choked = c("iso6358", &[2e-8, 0.3, 600_000.0, 300.0, 100_000.0]);
        close(choked, 0.014056717547137382);
        close(
            c("iso6358", &[2e-8, 0.3, 600_000.0, 300.0, 480_000.0]),
            0.00983765298540381,
        );
        // No forward flow at pr >= 1; no flow for non-physical upstream states.
        assert_eq!(c("iso6358", &[2e-8, 0.3, 100_000.0, 300.0, 200_000.0]), 0.0);
        assert_eq!(c("iso6358", &[2e-8, 0.3, -1.0, 300.0, 0.0]), 0.0);
        assert!(cerr("iso6358", &[-1e-8, 0.3, 1.0, 1.0, 1.0]).contains("C must be >= 0"));
        assert!(cerr("iso6358", &[1e-8, 1.0, 1.0, 1.0, 1.0]).contains("b must be in"));
    }

    #[test]
    fn friction_factor_regimes_and_flow_helpers() {
        close(c("friction_factor", &[1000.0, 0.0]), 0.064);
        close(c("friction_factor", &[1e5, 1e-4]), 0.018513866077475145);
        close(c("friction_factor", &[3000.0, 0.0]), 0.0289813195131093);
        close(c("darcy_friction", &[1000.0, 0.0]), 0.064);
        close(
            c("reynolds", &[1.2, -2.0, 0.05, 1.8e-5]),
            1.2 * 2.0 * 0.05 / 1.8e-5,
        );
        assert!(cerr("reynolds", &[1.2, 2.0, 0.05, 0.0]).contains("viscosity"));
        close(
            c("minor_loss", &[2.5, 1000.0, 3.0]),
            2.5 * 0.5 * 1000.0 * 9.0,
        );
    }

    #[test]
    fn convective_heat_correlations() {
        close(
            c("nu_dittus_boelter", &[1e4, 7.0, 0.4]),
            0.023 * libm::pow(1e4, 0.8) * libm::pow(7.0, 0.4),
        );
        close(c("nu_gnielinski", &[1e4, 7.0]), 79.49264509410906);
        assert_eq!(c("chen_f", &[20.0]), 1.0); // 1/Xtt = 0.05 ≤ 0.1
        close(c("chen_f", &[0.5]), 4.2167143599552865);
        close(c("chen_s", &[1e4, 2.0]), 0.7497848720424972);
        close(c("nu_shah", &[1e4, 3.0, 0.5, 0.3]), 227.57788444064022);
        close(
            c("nu_cavallini_zecchin", &[1e4, 3.0, 0.5, 1000.0, 50.0]),
            254.7543010335075,
        );
        close(c("zone_ramp", &[2.0, 1.0]), libm::tanh(2.0));
        assert_eq!(c("zone_ramp", &[-1.0, 1.0]), 0.0);
        assert!(cerr("nu_gnielinski", &[0.0, 1.0]).contains("must be > 0"));
    }

    #[test]
    fn two_phase_correlations() {
        close(c("lm_phi2", &[0.5, 20.0]), 1.0 + 40.0 + 4.0);
        close(
            c("lm_martinelli_tt", &[0.3, 1000.0, 50.0, 2e-4, 1e-5]),
            0.6467956769365347,
        );
        close(
            c("void_homogeneous", &[0.5, 1000.0, 50.0]),
            0.9523809523809523,
        );
        assert_eq!(c("void_homogeneous", &[-0.1, 1000.0, 50.0]), 0.0);
        assert_eq!(c("void_homogeneous", &[1.5, 1000.0, 50.0]), 1.0);
        close(c("void_zivi", &[0.5, 1000.0, 50.0]), 0.8804980315844955);
        close(
            c("void_rouhani", &[0.4, 1000.0, 50.0, 300.0, 0.01]),
            0.846466030954813,
        );
        close(
            c(
                "friedel_phi2",
                &[0.4, 1000.0, 50.0, 2e-4, 1.2e-5, 300.0, 0.01, 0.01],
            ),
            9.885206655409064,
        );
        close(
            c("momentum_flux", &[0.4, 1000.0, 50.0, 0.7, 300.0]),
            519.4285714285716,
        );
        assert!(cerr("lm_phi2", &[0.0, 20.0]).contains("must be > 0"));
        assert!(cerr("lm_martinelli_tt", &[1.5, 1.0, 1.0, 1.0, 1.0]).contains("(0, 1)"));
    }

    fn hx(name: &str, arrangement: &str, rest: &[f64]) -> Result<f64> {
        let mut args = vec![Expr::Str(arrangement.into())];
        args.extend(rest.iter().copied().map(n));
        eval(&Expr::call(name, args), &Scope::default())
    }

    #[test]
    fn hx_effectiveness_all_arrangements() {
        close(
            hx("hx_effectiveness", "counterflow", &[2.0, 0.5]).unwrap(),
            0.7746003264394359,
        );
        close(
            hx("hx_effectiveness", "counter-flow", &[2.0, 1.0]).unwrap(),
            2.0 / 3.0,
        );
        close(
            hx("hx_effectiveness", "crossflow", &[1.5, 0.7]).unwrap(),
            0.6186623444738465,
        );
        close(
            hx("hx_effectiveness", "parallel", &[1.0, 0.8]).unwrap(),
            0.46372283987689633,
        );
        close(
            hx("hx_effectiveness", "shell&tube", &[1.2, 0.6]).unwrap(),
            0.5665428281626369,
        );
        // Cr = 0 is the boiling/condensing limit for every arrangement.
        close(
            hx("hx_effectiveness", "parallel", &[2.0, 0.0]).unwrap(),
            1.0 - libm::exp(-2.0),
        );
        // hx_epsilon is an alias.
        close(
            hx("hx_epsilon", "counterflow", &[2.0, 0.5]).unwrap(),
            0.7746003264394359,
        );
        let msg = hx("hx_effectiveness", "diagonal", &[1.0, 0.5])
            .unwrap_err()
            .to_string();
        assert!(msg.contains("unknown flow arrangement"), "{msg}");
        let msg = hx("hx_effectiveness", "counterflow", &[-1.0, 0.5])
            .unwrap_err()
            .to_string();
        assert!(msg.contains("NTU"), "{msg}");
    }

    #[test]
    fn hx_ntu_inverts_the_effectiveness() {
        close(
            hx("hx_ntu", "counterflow", &[0.7, 0.5]).unwrap(),
            1.5463797764669633,
        );
        // Cr = 0 (boiling/condensing) has the arrangement-independent inverse,
        // but the arrangement string is still parsed first, as in Java.
        close(
            hx("hx_ntu", "counterflow", &[0.5, 0.0]).unwrap(),
            -libm::log(0.5),
        );
        // Crossflow both-unmixed inverts by bisection: round-trip through the
        // forward correlation.
        let eps = hx("hx_effectiveness", "crossflow", &[1.5, 0.7]).unwrap();
        let ntu = hx("hx_ntu", "crossflow", &[eps, 0.7]).unwrap();
        assert!((ntu - 1.5).abs() < 1e-9, "got {ntu}");
        // Unreachable effectiveness names the limit.
        let msg = hx("hx_ntu", "parallel", &[0.9, 0.8])
            .unwrap_err()
            .to_string();
        assert!(msg.contains("unreachable"), "{msg}");
    }

    #[test]
    fn lmtd_and_fin_efficiency() {
        close(c("lmtd", &[60.0, 40.0]), 49.326069247528636);
        close(c("lmtd", &[50.0, 50.0]), 50.0);
        assert!(cerr("lmtd", &[60.0, -1.0]).contains("positive"));
        assert_eq!(c("fin_efficiency", &[0.0]), 1.0);
        close(c("fin_efficiency", &[2.0]), libm::tanh(2.0) / 2.0);
        assert!(cerr("fin_efficiency", &[-1.0]).contains(">= 0"));
    }

    #[test]
    fn hx_sizing_correlations() {
        close(
            c("ua_hx", &[100.0, 2.0, 50.0, 4.0, 0.001]),
            1.0 / (1.0 / 200.0 + 0.001 + 1.0 / 200.0),
        );
        close(c("nu_zukauskas", &[1e4, 0.7]), 78.63195229232574);
        close(c("nu_colburn", &[0.005, 1000.0, 0.7]), 4.439520008713004);
        close(c("nu_churchill_chu", &[1e6, 0.7]), 16.530366876407225);
        close(c("nu_blend", &[3.0, 4.0]), libm::cbrt(27.0 + 64.0));
        close(c("nu_hilpert", &[500.0, 0.7]), 10.977583242918563);
        close(c("nu_plate", &[2000.0, 5.0, 45.0]), 83.52208999535316);
        close(c("hx_dh", &[0.01, 2.0, 1.0]), 4.0 * 0.01 * 1.0 / 2.0);
        close(c("hx_aconv", &[0.01, 1.0, 0.002]), 4.0 * 0.01 * 1.0 / 0.002);
        close(c("hx_sigma", &[0.3, 0.5]), 0.6);
        close(c("hx_eta_surf", &[8.0, 10.0, 0.9]), 1.0 - 0.8 * 0.1);
        close(
            c("hx_fin_len", &[0.02, 2e-4, 800.0, 0.01]),
            0.30169334619112836,
        );
        close(
            c("hx_area_direct", &[0.5, 20.0, 0.01, 0.02, 2e-4]),
            2.0 * 0.5 * 20.0 * ((0.01 - 4e-4) + (0.02 - 4e-4)),
        );
        close(
            c("hx_area_indirect", &[0.5, 20.0, 0.3]),
            2.0 * 0.5 * 20.0 * 0.3,
        );
        close(
            c("dp_gravity", &[1000.0, 50.0, 0.3, 2.0, 30.0]),
            7011.754749999998,
        );
        close(
            c(
                "dp_compact_core",
                &[25.0, 1.2, 1.0, 1.1, 0.6, 0.02, 200.0, 0.4, 0.2],
            ),
            1373.863636363636,
        );
        close(c("mass_flux", &[0.05, 0.002]), 25.0);
        close(
            c("nu_gungor_winterton", &[50.0, 0.25, 0.001]),
            673.0212845104246,
        );
        close(c("nu_traviss", &[1e4, 3.0, 0.25]), 514.2309275467284);
        assert!(cerr("ua_hx", &[0.0, 1.0, 1.0, 1.0, 0.0]).contains("positive"));
    }

    #[test]
    fn fin_surface_factors_take_a_string_argument() {
        let e = Expr::call("j_fin", vec![Expr::Str("louvered".into()), n(1000.0)]);
        close(ev(&e), 0.01097865779395536);
        let e = Expr::call("f_fin", vec![Expr::Str("louvered".into()), n(1000.0)]);
        close(ev(&e), 0.052874867295355024);
        // Unknown surfaces fall back to plain, as in Java.
        let e = Expr::call("f_fin", vec![Expr::Str("weird".into()), n(1000.0)]);
        close(ev(&e), 0.150 * libm::pow(1000.0, -0.3));
        let e = Expr::call(
            "nu_tubebank",
            vec![Expr::Str("staggered".into()), n(5000.0), n(0.7)],
        );
        close(ev(&e), 58.30117241319823);
    }

    // =====================================================================
    // Phase 4: compressible flow
    // =====================================================================

    #[test]
    fn isentropic_relations_at_mach_two() {
        close(c("t0_t", &[2.0, 1.4]), 1.7999999999999998);
        close(c("isen_t0_t", &[2.0, 1.4]), 1.7999999999999998);
        close(c("p0_p", &[2.0, 1.4]), 7.824449066867263);
        close(c("rho0_rho", &[2.0, 1.4]), libm::pow(1.8, 2.5));
        close(c("a_astar", &[2.0, 1.4]), 1.6875000000000002);
        assert!(cerr("t0_t", &[0.0, 1.4]).contains("Mach"));
        assert!(cerr("t0_t", &[2.0, 1.0]).contains("k must be > 1"));
    }

    #[test]
    fn normal_shock_relations_at_mach_two() {
        close(c("m2_shock", &[2.0, 1.4]), 0.5773502691896257);
        close(c("mach_shock", &[2.0, 1.4]), 0.5773502691896257);
        close(c("p2_p1_shock", &[2.0, 1.4]), 4.5);
        close(c("t2_t1_shock", &[2.0, 1.4]), 1.6874999999999998);
        close(c("rho2_rho1_shock", &[2.0, 1.4]), 8.0 / 3.0);
        close(c("p02_p01_shock", &[2.0, 1.4]), 0.7208738614847455);
        assert!(cerr("m2_shock", &[0.8, 1.4]).contains("supersonic"));
    }

    #[test]
    fn rayleigh_and_fanno_relations() {
        close(c("rayleigh_t0_t0star", &[0.5, 1.4]), 0.691358024691358);
        close(c("rayleigh_p_pstar", &[0.5, 1.4]), 2.4 / 1.35);
        close(c("fanno_fld", &[0.5, 1.4]), 1.0690603127182559);
        close(c("fanno_t_tstar", &[1.0, 1.4]), 1.0);
        close(c("fanno_p_pstar", &[1.0, 1.4]), 1.0);
        close(c("rayleigh_t_tstar", &[1.0, 1.4]), 1.0);
    }

    #[test]
    fn prandtl_meyer_and_mach_angle() {
        close(c("prandtlmeyer", &[2.0, 1.4]), 0.46041368208269473);
        close(c("prandtl_meyer", &[2.0, 1.4]), 0.46041368208269473);
        close(c("machangle", &[2.0]), 0.5235987755982989);
        // Inverse round trip.
        let nu = c("prandtlmeyer", &[2.0, 1.4]);
        assert!((c("mach_prandtlmeyer", &[nu, 1.4]) - 2.0).abs() < 1e-9);
        assert!(cerr("mach_prandtlmeyer", &[-0.1, 1.4]).contains("outside"));
    }

    #[test]
    fn area_ratio_inversion_honours_the_branch() {
        let sup = |regime: &str, ratio: f64| {
            eval(
                &Expr::call(
                    "mach_a_astar",
                    vec![n(ratio), n(1.4), Expr::Str(regime.into())],
                ),
                &Scope::default(),
            )
        };
        let ratio = c("a_astar", &[2.0, 1.4]);
        assert!((sup("supersonic", ratio).unwrap() - 2.0).abs() < 1e-9);
        let ratio_sub = c("a_astar", &[0.5, 1.4]);
        assert!((sup("subsonic", ratio_sub).unwrap() - 0.5).abs() < 1e-9);
        assert_eq!(sup("sup", 1.0).unwrap(), 1.0);
        assert!(sup("sideways", 2.0)
            .unwrap_err()
            .to_string()
            .contains("branch"));
        assert!(sup("sub", 0.5).unwrap_err().to_string().contains(">= 1"));
    }

    #[test]
    fn oblique_shock_theta_beta_relations() {
        let beta40 = 40.0_f64.to_radians();
        close(c("theta_oblique", &[2.0, beta40, 1.4]), 0.18540474909716562);
        let theta = c("theta_oblique", &[2.0, beta40, 1.4]);
        let call_beta = |branch: &str| {
            eval(
                &Expr::call(
                    "beta_oblique",
                    vec![n(2.0), n(theta), n(1.4), Expr::Str(branch.into())],
                ),
                &Scope::default(),
            )
        };
        let weak = call_beta("weak").unwrap();
        assert!((weak - beta40).abs() < 1e-6, "weak {weak}");
        let strong = call_beta("strong").unwrap();
        assert!(strong > 1.0, "strong {strong}");
        close(c("theta_oblique", &[2.0, strong, 1.4]), theta);
        // Detachment: theta above the peak errors.
        let e = Expr::call(
            "beta_oblique",
            vec![n(2.0), n(0.5), n(1.4), Expr::Str("weak".into())],
        );
        assert!(err(&e).contains("detaches"));
    }
}
