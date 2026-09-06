//! The D10 equivalence oracle for injected Function Tables, at the core seam
//! (`solve_with_tables` / `check_with_tables` /
//! `Definitions::merge_extra_tables`): the SAME table data injected through
//! the request channel and written as an in-document `TABLE` block must
//! produce **bit-identical** solve results — values, display names, block
//! count. The wasm-level tests (`crates/frees/tests/function_tables.rs`)
//! grade the DTO conversion on top of this; here the defs are built directly,
//! already lowercase, as the boundary hands them in.
//!
//! The collision rule graded below is the Java's, verified against
//! `EquationSystemSolver.withExtraDefs`: **source definitions win** —
//! `merged = new HashMap<>(extraDefs); merged.putAll(parsed.defs())`, and its
//! own doc comment says "source definitions win on name collision". (The REPL
//! cache is the one place the Java merges the other way round;
//! `frees/src/repl.rs` mirrors that separately.)

use frees_core::parser::defs::{Curve, FunctionTableDef};
use frees_core::{solve_with, solve_with_tables, SolverSettings};

fn table(name: &str, arg_names: &[&str], curves: Vec<Curve>) -> FunctionTableDef {
    FunctionTableDef {
        name: name.to_string(),
        arg_names: arg_names.iter().map(|s| s.to_string()).collect(),
        x_log: false,
        y_log: false,
        curves,
        output_unit: None,
        arg_units: None,
    }
}

fn curve(param: Option<f64>, points: &[(f64, f64)]) -> Curve {
    Curve {
        param,
        xs: points.iter().map(|p| p.0).collect(),
        ys: points.iter().map(|p| p.1).collect(),
    }
}

fn solve_plain(source: &str) -> frees_core::Solution {
    solve_with(source, &SolverSettings::default(), &[]).expect("document solves")
}

fn solve_injected(source: &str, tables: &[FunctionTableDef]) -> frees_core::Solution {
    solve_with_tables(source, &SolverSettings::default(), &[], tables).expect("document solves")
}

/// Values (bit for bit), display names and block count must agree — the D10
/// acceptance bar for "the injected table IS the TABLE block".
fn assert_equivalent(injected: &frees_core::Solution, in_document: &frees_core::Solution) {
    assert_eq!(
        injected.values.keys().collect::<Vec<_>>(),
        in_document.values.keys().collect::<Vec<_>>(),
        "the two solves report different variables"
    );
    for (name, value) in &injected.values {
        let expected = in_document.values[name];
        assert_eq!(
            value.to_bits(),
            expected.to_bits(),
            "{name}: injected {value} != in-document {expected}"
        );
    }
    assert_eq!(injected.display_names, in_document.display_names);
    assert_eq!(
        injected.blocks.len(),
        in_document.blocks.len(),
        "block count"
    );
}

#[test]
fn a_one_d_curve_injected_equals_the_table_block() {
    let in_document = solve_plain(
        "TABLE fcurve(x)\n\
         1 10\n\
         2 20\n\
         4 25\n\
         END\n\
         y = fcurve(1.5)\n\
         z = fcurve(3)\n",
    );
    let injected = solve_injected(
        "y = fcurve(1.5)\nz = fcurve(3)\n",
        &[table(
            "fcurve",
            &["x"],
            vec![curve(None, &[(1.0, 10.0), (2.0, 20.0), (4.0, 25.0)])],
        )],
    );
    assert_equivalent(&injected, &in_document);
    assert_eq!(injected.values["y"], 15.0);
}

#[test]
fn a_two_d_curve_family_injected_equals_the_table_block() {
    let in_document = solve_plain(
        "TABLE nu(re : t = 100, 200)\n\
         1 10 30\n\
         2 20 40\n\
         END\n\
         a = nu(1.5, 100)\n\
         b = nu(1.5, 150)\n\
         c = nu(2, 200)\n",
    );
    let injected = solve_injected(
        "a = nu(1.5, 100)\nb = nu(1.5, 150)\nc = nu(2, 200)\n",
        &[table(
            "nu",
            &["re", "t"],
            vec![
                curve(Some(100.0), &[(1.0, 10.0), (2.0, 20.0)]),
                curve(Some(200.0), &[(1.0, 30.0), (2.0, 40.0)]),
            ],
        )],
    );
    assert_equivalent(&injected, &in_document);
}

#[test]
fn log_axes_injected_equal_the_xlog_ylog_flags() {
    let in_document = solve_plain(
        "TABLE damping(f) XLOG YLOG\n\
         1 10\n\
         100 1000\n\
         END\n\
         y = damping(10)\n",
    );
    let injected = solve_injected(
        "y = damping(10)\n",
        &[{
            let mut t = table(
                "damping",
                &["f"],
                vec![curve(None, &[(1.0, 10.0), (100.0, 1000.0)])],
            );
            t.x_log = true;
            t.y_log = true;
            t
        }],
    );
    assert_equivalent(&injected, &in_document);
    // Log-log interpolation is geometric, not linear — proves the flags took.
    assert!(
        (injected.values["y"] - 100.0).abs() < 1e-9,
        "log-log midpoint should be 100, got {}",
        injected.values["y"]
    );
}

#[test]
fn the_classic_table_functions_see_an_injected_table() {
    // `Interpolate1('name', x)` resolves through the same definitions map
    // (`Evaluator`'s TABLE dispatch), so the Java answers this too.
    let in_document = solve_plain(
        "TABLE fcurve(x)\n\
         1 10\n\
         2 20\n\
         3 30\n\
         END\n\
         y = Interpolate1('fcurve', 2.5)\n",
    );
    let injected = solve_injected(
        "y = Interpolate1('fcurve', 2.5)\n",
        &[table(
            "fcurve",
            &["x"],
            vec![curve(None, &[(1.0, 10.0), (2.0, 20.0), (3.0, 30.0)])],
        )],
    );
    assert_equivalent(&injected, &in_document);
}

#[test]
fn on_a_name_collision_the_document_table_wins() {
    // The Java `withExtraDefs`: `new HashMap<>(extraDefs)` overwritten by
    // `putAll(parsed.defs())` — the in-document definition survives.
    let source = "TABLE fcurve(x)\n\
                  1 10\n\
                  2 20\n\
                  END\n\
                  y = fcurve(1.5)\n";
    let document_only = solve_plain(source);
    let collided = solve_injected(
        source,
        &[table(
            "fcurve",
            &["x"],
            vec![curve(None, &[(1.0, 1000.0), (2.0, 2000.0)])],
        )],
    );
    assert_equivalent(&collided, &document_only);
    assert_eq!(
        collided.values["y"], 15.0,
        "the DOCUMENT's data must answer"
    );
}

#[test]
fn on_a_name_collision_a_document_function_wins_too() {
    // In the Java the defs map is one namespace across FUNCTION / PROCEDURE /
    // MODULE / TABLE, so a source FunctionDef shadows the injected table.
    let source = "FUNCTION fcurve(x)\n\
                  fcurve := 7 * x\n\
                  END\n\
                  y = fcurve(2)\n";
    let document_only = solve_plain(source);
    let collided = solve_injected(
        source,
        &[table(
            "fcurve",
            &["x"],
            vec![curve(None, &[(1.0, 10.0), (2.0, 20.0)])],
        )],
    );
    assert_equivalent(&collided, &document_only);
    assert_eq!(collided.values["y"], 14.0, "the FUNCTION must answer");
}

#[test]
fn among_duplicate_extras_the_last_wins() {
    // `SolveDtos.functionDefsOf` builds a HashMap with `put`, so a repeated
    // name keeps the last table. The boundary dedupes before core, but
    // `merge_extra_tables` keeps the rule for direct callers.
    let last_only = solve_injected(
        "y = fcurve(1.5)\n",
        &[table(
            "fcurve",
            &["x"],
            vec![curve(None, &[(1.0, 100.0), (2.0, 200.0)])],
        )],
    );
    let both = solve_injected(
        "y = fcurve(1.5)\n",
        &[
            table(
                "fcurve",
                &["x"],
                vec![curve(None, &[(1.0, 10.0), (2.0, 20.0)])],
            ),
            table(
                "fcurve",
                &["x"],
                vec![curve(None, &[(1.0, 100.0), (2.0, 200.0)])],
            ),
        ],
    );
    assert_equivalent(&both, &last_only);
    assert_eq!(both.values["y"], 150.0);
}

#[test]
fn no_injection_is_byte_for_byte_the_plain_solve() {
    let source = "TABLE fcurve(x)\n\
                  1 10\n\
                  2 20\n\
                  END\n\
                  y = fcurve(1.5)\n\
                  z = y + 1\n";
    let plain = solve_plain(source);
    let empty = solve_injected(source, &[]);
    assert_equivalent(&empty, &plain);
    assert_eq!(empty.residuals.len(), plain.residuals.len());
    assert_eq!(empty.stats.iterations, plain.stats.iterations);
}

#[test]
fn an_uninjected_name_still_fails_as_unknown() {
    // A table skipped by the boundary's tolerance (or simply never sent)
    // leaves the call unresolvable — the ordinary eval error, not a panic.
    let failure = solve_with_tables("y = fcurve(1.5)\n", &SolverSettings::default(), &[], &[])
        .expect_err("unknown function must fail");
    assert!(
        failure.to_string_message().contains("unknown function"),
        "{}",
        failure.to_string_message()
    );
}

#[test]
fn check_with_tables_matches_the_table_block_check() {
    let in_document = frees_core::check(
        "TABLE fcurve(x)\n\
         1 10\n\
         2 20\n\
         END\n\
         y = fcurve(1.5)\n",
    )
    .expect("check runs");
    let injected = frees_core::check_with_tables(
        "y = fcurve(1.5)\n",
        &[],
        &[table(
            "fcurve",
            &["x"],
            vec![curve(None, &[(1.0, 10.0), (2.0, 20.0)])],
        )],
    )
    .expect("check runs");
    assert!(injected.solvable, "{}", injected.message);
    assert_eq!(injected.solvable, in_document.solvable);
    assert_eq!(injected.equation_count, in_document.equation_count);
    assert_eq!(injected.unknown_count, in_document.unknown_count);
    assert_eq!(injected.message, in_document.message);
}

#[test]
fn a_dynamic_block_reads_an_injected_table() {
    // The merged defs feed the transient path too (the Java hands the ODE
    // system `parsed.defs()` — the merged map).
    let in_document = solve_plain(
        "TABLE gain(y)\n\
         0 0.5\n\
         2 1.5\n\
         END\n\
         DYNAMIC decay (method = ode45, time = 0 .. 1, points = 5)\n  \
         der(y) = -gain(y) * y\n  y(0) = 1\nEND\n",
    );
    let injected = solve_injected(
        "DYNAMIC decay (method = ode45, time = 0 .. 1, points = 5)\n  \
         der(y) = -gain(y) * y\n  y(0) = 1\nEND\n",
        &[table(
            "gain",
            &["y"],
            vec![curve(None, &[(0.0, 0.5), (2.0, 1.5)])],
        )],
    );
    assert_eq!(injected.ode_tables.len(), 1);
    assert_eq!(in_document.ode_tables.len(), 1);
    let (a, b) = (&injected.ode_tables[0], &in_document.ode_tables[0]);
    assert_eq!(a.columns, b.columns);
    assert_eq!(a.rows.len(), b.rows.len());
    for (ra, rb) in a.rows.iter().zip(&b.rows) {
        for (va, vb) in ra.iter().zip(rb) {
            assert_eq!(va.to_bits(), vb.to_bits(), "trajectory diverged");
        }
    }
}
