//! # frees-core
//!
//! The frees computational engine, ported from
//! `../frEES/backend/core` (Java, 134 files / 38,181 LOC).
//!
//! This crate is **target-agnostic on purpose**: it has no `wasm-bindgen`
//! dependency and compiles for `x86_64` and `wasm32-unknown-unknown` alike. The
//! native build is what runs the parity harness against the Java oracle at full
//! speed; the wasm build is what ships. A port that can only be tested in a
//! browser cannot be trusted (see `PLAN.md` §2).
//!
//! ## Engine invariants
//!
//! These come from the parent engine and must survive the port:
//!
//! * frees is an **equation solver**, not a sequential language — statements are
//!   order-independent.
//! * Variable names are **case-insensitive** (stored lowercase).
//! * **All calculation is in SI.** Unit-annotated literals convert at parse
//!   time; unit warnings never block a solve.
//! * Types follow naming convention: `$` → string, `#` → constant, `[]` →
//!   array, `_r`/`_i` → complex components.
//! * Solving is **Tarjan SCC blocking, then Newton with step-halving** per block.

#![forbid(unsafe_code)]

pub mod analysis;
pub mod ast;
pub mod cas;
pub mod components;
pub mod control;
pub mod curvetable;
pub mod dae;
pub mod descriptive;
pub mod diag;
pub mod differentiator;
pub mod engine;
pub mod eval;
pub mod integral;
pub mod interp2;
pub mod lexer;
pub mod linalg;
pub mod ode;
pub mod parser;
pub mod procedures;
pub mod progress;
pub mod props;
pub mod signal;
pub mod solver;
pub mod statistics;
pub mod token;
pub mod units;

pub use ast::{BinOp, CmpOp, Equation, Expr, LogicOp, Statement};
pub use diag::{Diagnostic, FreesError, Result, Severity, Span};
pub use engine::{
    check, check_with, check_with_tables, check_with_tables_complex, solve, solve_all,
    solve_all_with, solve_all_with_tables, solve_with, solve_with_parametric,
    solve_with_parametric_tables, solve_with_tables, CheckReport, EquationResidual,
    PartialDiagnostics, PreparedDocument, Solution, SolveFailure, SolveStats, SyntaxErrorInfo,
    VariableOverride,
};
pub use parser::{parse_document, Document, GuessDirective};
pub use solver::{Block, BlockingReport, NewtonReport, SolverSettings};
pub use token::{Token, TokenKind};

/// Crate version, surfaced in the About dialog and the worker handshake.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
