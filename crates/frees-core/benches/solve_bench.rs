//! Phase 12: the benchmark suite. Five documents spanning the engine's cost
//! centres, timed end-to-end through the public `solve` — parse, unit check,
//! Tarjan blocking, Newton, and (where the document asks) the component
//! expander, the property backend and the ODE integrators. End-to-end because
//! that is what a keystroke costs the user; nothing here times internals.
//!
//! The JVM comparison is NOT in this file — an oracle timing run is
//! `time tools/golden-dumper/run.sh <dir> <out>` over the same documents (see
//! docs/status-phase12.md for the measured table and its caveats). Keep the
//! document list here and the oracle timing directory in sync.
//!
//! Native-only: criterion's dependency tree does not build on
//! wasm32-unknown-unknown, and CI clippy compiles `--all-targets` for that
//! target — hence the per-item cfg guards and the stub `main` for wasm32.

#[cfg(not(target_arch = "wasm32"))]
use criterion::{criterion_group, criterion_main, Criterion};
#[cfg(not(target_arch = "wasm32"))]
use frees_core::{
    analysis::optimizer::{optimize, Problem},
    analysis::parametric::{run_sweep, RowJob, RowOutcome},
    dae::assembly::ClosureResidual,
    dae::solver::IdaDaeSolver,
    parser::blocks::ParametricTable,
    solve, PreparedDocument, SolverSettings,
};

/// The canonical two-block scalar document (Phase 0's first fixture).
#[cfg(not(target_arch = "wasm32"))]
const SCALAR: &str = "\
x = 4 [m] - y\n\
y = x / 2\n\
a = 2 * x\n\
";

#[cfg(not(target_arch = "wasm32"))]
fn doc(name: &str) -> String {
    let path = format!(
        "{}/../../fixtures/corpus/{name}.frees",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn bench_solve(c: &mut Criterion) {
    let settings = SolverSettings::default();

    let cases: Vec<(&str, String)> = vec![
        ("scalar_two_block", SCALAR.to_string()),
        // Real-fluid Rankine cycle: the property-table backend in the loop.
        ("rankine_cycle", doc("rankine-cycle")),
        // Component network: library expansion + the mixed system it emits.
        ("component_mvem", doc("components_bsweep_mvem_wotmap")),
        // Transient: a DYNAMIC block driving the ODE path end to end.
        ("transient_dyn", doc("dyn_accessor_read")),
        // Control CALLs: state space, LQR and the CAS-backed helpers.
        ("control_lqr", doc("ctl-lqr_3state")),
        // Large component network: multi-domain EV thermal management system.
        ("large_component_network", doc("ev-thermal-management")),
        // Stiff thermofluid transient: multi-domain BoilingVessel + relief valve on IDA.
        ("stiff_thermofluid_transient", doc("pressure-cooker")),
    ];

    for (name, source) in &cases {
        // Fail loudly outside the timer if a document stops solving — a bench
        // that times an error path reports a fantasy speedup.
        let probe = solve(source, &settings);
        assert!(probe.is_ok(), "{name} no longer solves: {:?}", probe.err());

        c.bench_function(name, |b| {
            b.iter(|| solve(std::hint::black_box(source), &settings))
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn bench_constrained_opt(c: &mut Criterion) {
    let p = Problem {
        text: "f = (x - 10)^2 + (y - 10)^2 + (z - 10)^2\n".to_string(),
        settings: SolverSettings::default(),
        overrides: Vec::new(),
        objective: "f".to_string(),
        decisions: vec!["x".to_string(), "y".to_string(), "z".to_string()],
        lowers: vec![0.0, 0.0, 0.0],
        uppers: vec![15.0, 15.0, 15.0],
        method: Some("nelder-mead".to_string()),
        maximize: false,
        constraints: vec![
            "x + y <= 8".to_string(),
            "y + z <= 8".to_string(),
            "x >= 1".to_string(),
        ],
        extra_tables: Vec::new(),
    };

    let probe = optimize(&p);
    assert!(
        probe.is_ok(),
        "constrained_opt no longer solves: {:?}",
        probe.err()
    );

    c.bench_function("constrained_optimization", |b| {
        b.iter(|| optimize(std::hint::black_box(&p)))
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn bench_sweep_1000(c: &mut Criterion) {
    let source = "y = 2 * x + 1\n";
    let rows: Vec<Vec<Option<f64>>> = (1..=1000).map(|i| vec![Some(i as f64), None]).collect();
    let table = ParametricTable {
        name: "bench_table".to_string(),
        vars: vec!["x".to_string(), "y".to_string()],
        rows,
    };
    let settings = SolverSettings::default();

    let mut prep = PreparedDocument::new(source, &settings, &[], &[]).unwrap();
    let sweep = run_sweep(&table, source, |job: RowJob<'_>| {
        let res = prep.solve_with_pins(job.cells, job.accessors);
        match res {
            Ok(sol) => RowOutcome::solved(sol.values),
            Err(e) => RowOutcome::failed(format!("{e}")),
        }
    });
    assert_eq!(sweep.outcomes.len(), 1000, "sweep must produce 1000 rows");

    c.bench_function("sweep_1000_rows", |b| {
        b.iter(|| {
            let mut prep = PreparedDocument::new(source, &settings, &[], &[]).unwrap();
            run_sweep(&table, source, |job: RowJob<'_>| {
                let res = prep.solve_with_pins(job.cells, job.accessors);
                match res {
                    Ok(sol) => RowOutcome::solved(sol.values),
                    Err(e) => RowOutcome::failed(format!("{e}")),
                }
            })
        })
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn bench_large_sparse_dae(c: &mut Criterion) {
    const N: usize = 200;
    let res = ClosureResidual::new(move |_t, y, yp, r: &mut [f64]| {
        for i in 0..N {
            let prev = if i > 0 { y[i - 1] } else { y[N - 1] };
            let next = if i + 1 < N { y[i + 1] } else { y[0] };
            r[i] = yp[i] - 5.0 * (prev - 2.0 * y[i] + next);
        }
        Ok(())
    });

    let mut y0 = vec![0.0; N];
    for (i, val) in y0.iter_mut().enumerate() {
        *val = (i as f64 * std::f64::consts::PI * 2.0 / N as f64).sin();
    }
    let yp0 = vec![0.0; N];

    let sparsity: Vec<Vec<usize>> = (0..N)
        .map(|i| {
            let mut cols = Vec::new();
            if i > 0 {
                cols.push(i - 1);
            } else {
                cols.push(N - 1);
            }
            cols.push(i);
            if i + 1 < N {
                cols.push(i + 1);
            } else {
                cols.push(0);
            }
            cols.sort();
            cols
        })
        .collect();

    let mut probe_solver = IdaDaeSolver::new(N, &res).unwrap();
    probe_solver.set_tolerances(1e-5, 1e-7);
    probe_solver.set_sparsity(&sparsity).unwrap();
    probe_solver.init(0.0, &y0, &yp0).unwrap();
    let step = probe_solver.step(0.2);
    assert!(step.is_ok(), "sparse dae step failed");

    c.bench_function("large_sparse_dae_step", |b| {
        b.iter(|| {
            let mut s = IdaDaeSolver::new(N, &res).unwrap();
            s.set_tolerances(1e-5, 1e-7);
            s.set_sparsity(&sparsity).unwrap();
            s.init(0.0, &y0, &yp0).unwrap();
            s.step(std::hint::black_box(0.2)).unwrap()
        })
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn bench_cold_vs_warm(c: &mut Criterion) {
    let settings = SolverSettings::default();
    let source = doc("rankine-cycle");

    // Cold initialization: end-to-end model preparation + solve from scratch
    c.bench_function("cold_preparation_rankine", |b| {
        b.iter(|| solve(std::hint::black_box(&source), &settings))
    });

    // Warm execution: reuse prepared document
    let mut prep = PreparedDocument::new(&source, &settings, &[], &[]).unwrap();
    c.bench_function("warm_execution_rankine", |b| {
        b.iter(|| prep.solve_with_pins(&[], None))
    });
}

#[cfg(not(target_arch = "wasm32"))]
criterion_group!(
    benches,
    bench_solve,
    bench_constrained_opt,
    bench_sweep_1000,
    bench_large_sparse_dae,
    bench_cold_vs_warm
);
#[cfg(not(target_arch = "wasm32"))]
criterion_main!(benches);

/// wasm32: the bench target must still link under `clippy --all-targets`.
#[cfg(target_arch = "wasm32")]
fn main() {}
