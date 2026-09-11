# frees

A high-performance declarative equation solver, physical systems modeler, and simulation environment built in Rust and WebAssembly that runs **entirely client-side in the browser** and as a **native desktop CLI**. Zero cloud compute dependencies, zero external API traffic, and zero network latency.

```frees
m_dot = 2.5 [kg/s]
h_in = Enthalpy(Water, T = 300 [K], P = 101325 [Pa])
h_out = Enthalpy(Water, T = 360 [K], P = 101325 [Pa])
Q_dot = m_dot * (h_out - h_in)
```

Equations are declarative and order-independent, variable names are case-insensitive, quantities solve in SI units with parse-time unit conversions, and systems are blocked and solved using exact symbolic derivatives.

---

## Core Capabilities

- **Declarative Equation Modeling**: Acausal mathematical modeling with automatic unit consistency verification, dimensional analysis, and unit conversions.
- **Robust Nonlinear Solvers**: Incidence graph decomposition into strongly connected components (Tarjan algorithm), scaled Newton-Raphson iteration with adaptive trust-region line search, Powell hybrid dogleg methods, and automatic rank-deficient merge recovery.
- **Reusable Prepared Solvers**: Hoisted structural compilation (`PreparedDocument`) reusing AST, variable specs, block decomposition, and analytical Jacobians for in-place numeric mutations, delivering over 18× speedup on parametric sweeps.
- **Transient Simulation (ODE & DAE)**:
  - Explicit & Stiff ODE Solvers: Adaptive Dormand-Prince Runge-Kutta (`ode45`) and 5th-order Radau IIA (`radau5` / `radauiia`).
  - Stiff Differential-Algebraic Equations (DAE): Variable-coefficient Backward Differentiation Formulas (BDF / IDA) with adaptive order (1–5) and step-size control.
  - State Event Handling: Root-finding zero crossings (`EVENT condition -> action`), discrete mode switching, and trajectory stop events.
- **Thermodynamic Property Engine (`rustprop`)**: Pure-Rust CoolProp 8.0.0 implementation supporting high-accuracy Helmholtz equations of state. 26 pure fluids are linked and served on the diagram picker — every one the property alias table names — spanning Water/Steam, Air, CO2, the R-series refrigerants, Ammonia, the light hydrocarbons, and the permanent and noble gases (Nitrogen, Oxygen, Argon, Helium, Hydrogen). Cubic EoS, incompressibles (`INCOMP::MEG`, `MPG`) and humid air psychrometrics (`HAPropsSI`) sit alongside them.
- **Standard Acausal Component Library**: 295+ built-in acausal components across 13 physical domains: fluid networks, heat transfer, moist air HVAC, electrical systems, mechanics, and control blocks.
- **Computer Algebra (CAS) & Control Systems**:
  - CAS: Exact rational arithmetic over $\mathbb{Q}$, polynomial operations, Zassenhaus factorization, partial fractions, and symbolic Laplace transforms.
  - Control Systems: Transfer functions, state-space models ($A, B, C, D$), pole placement, LQR/LQE, frequency response (Bode, Nyquist), and automated PID tuning.
- **Experimental Data, Statistics & Uncertainty**:
  - Inference: covariance and correlation, robust summaries, Student-t and F primitives, confidence intervals, t-tests, one-way ANOVA, chi-square goodness-of-fit, seeded bootstrap and permutation tests.
  - Fitting: weighted, bounded and robust curve fitting and dynamic parameter calibration, reporting parameter covariance, standard errors, confidence/prediction bands, and rank/conditioning diagnostics that flag an unidentifiable fit rather than inventing a finite uncertainty for it.
  - Uncertainty: first-order propagation with declared input correlations (`Correlation(A, B) = ρ`), six input distributions (`DistributionOf(X) = Uniform(…)`) drawn by truncated inverse CDF, and Monte Carlo over seeded Latin-hypercube or scrambled Sobol designs.
  - Global sensitivity: Sobol' first-order and total-order variance indices with bootstrap error bars, and Morris elementary-effects screening.
  - Signal processing: an `O(n log n)` transform at any length (radix-2 and Bluestein), plus `Detrend`, `Smooth`, `Window`, `Filter`, `FiltFilt`, `XCorr`, `Welch` power spectra and peak detection.
- **Multi-Worker Parametric Sweeps**: Hardware-aware worker pool (up to 4 Web Workers) executing independent table sweep chunks in parallel with weighted progress reporting, plus fixed-point Gauss-Seidel iteration for table-wide accessor functions (`TableRun#`, `TableAvg`, etc.).
- **Interactive Visualization & Inspection**: Fast Plotly.js charts with smart series decimation for dense trajectories (> 2,000 points rendered responsively while preserving 100% raw data in memory), dual cursors with delta/slope inspection, and thermodynamic phase diagrams ($T\text{-}s$, $P\text{-}h$, Psychrometric).
- **Offline-First PWA & Storage**: Installable Progressive Web App with Service Worker precaching, durable dual-write autosave (synchronous `localStorage` boot slot + durable `IndexedDB` mirror), and FileSystem Access API support.

---

## Architecture Overview

```
                      +------------------------------------------+
                      |         Browser UI (React 19)            |
                      |  CodeMirror 6 | Dockview | Glide Grid   |
                      |  Plotly.js Visualizations | Mantine UI   |
                      +--------------------+---------------------+
                                           |
                                [engineClient.ts]
                         Worker Pool (1..4 Web Workers)
                                           |
                      +--------------------+---------------------+
                      |           Web Worker Host                |
                      |          (engine.worker.ts)              |
                      +--------------------+---------------------+
                                           |
                                   WASM Boundary
                                 (`crates/frees`)
                                           |
                      +--------------------+---------------------+
                      |        frees-core (Pure Rust)            |
                      |  - AST & Unit Checker                    |
                      |  - Tarjan Block Decomposition            |
                      |  - Scaled Newton & Prepared Solver       |
                      |  - ODE (ode45, radau5) & DAE (BDF/IDA)   |
                      |  - rustprop Thermodynamic Engine         |
                      |  - CAS & Control Systems                 |
                      |  - Single & Multi-Objective Optimizer   |
                      |  - Statistics, Fitting & Uncertainty     |
                      |  - Sampling, Sensitivity & Signal DSP    |
                      +------------------------------------------+
```

| Component | Responsibility |
|---|---|
| [`crates/frees-core`](crates/frees-core) | Core numerical engine. Target-agnostic, zero browser dependencies. |
| [`crates/frees`](crates/frees) | WebAssembly boundary crate (`wasm-bindgen`). Serializes and deserializes typed JSON payloads. |
| [`crates/frees-cli`](crates/frees-cli) | Native command-line binary for headless solving, verification, and automated batch processing. |
| [`web`](web) | Modern React 19 web application, Web Worker client, and PWA packaging. |
| [`fixtures`](fixtures) | Frozen regression corpus of 1,308+ test models, reference solutions, and tolerance definitions. |

---

## Quick Start

### 1. Command-Line Interface (CLI)

The workspace requires a stable Rust toolchain (managed via `rustup` and configured in `rust-toolchain.toml`).

```bash
# Solve an equation system via stdin
printf 'x + y = 10\nx * y = 21\n' | cargo run -qp frees-cli -- solve

# Check model solvability and degrees of freedom
cargo run -qp frees-cli -- check path/to/model.frees

# Execute solve with detailed JSON output
cargo run -qp frees-cli -- solve --json path/to/model.frees
```

### 2. Browser Web Application

Building the web application requires Node 22 (pinned in `web/.nvmrc`) and `wasm-pack`.

```bash
# 1. Compile the Rust engine to WebAssembly
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg

# 2. Install web dependencies and compile reference documentation
cd web
npm ci
npm run compile-docs

# 3. Start local development server
npm run dev
```

Open `http://localhost:5173` to access the interactive workspace.

---

## Verification & Testing

Every change in the repository is validated against strict automated quality gates:

```bash
# 1. Rust code formatting and linting
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings

# 2. Native engine unit & integration tests
cargo test --workspace -- --skip golden_corpus_parity

# 3. Full golden regression corpus replay (1,308 fixtures)
cargo test --release --test parity

# 4. Frontend unit tests (Vitest under Node 22)
cd web && npm test

# 5. Production frontend bundle build & PWA validation
cd web && npm run build

# 6. Reference-documentation coverage and cross-link invariants
cd web && npm run check-docs

# 7. Golden corpus replayed through the COMPILED WASM module under Node,
#    so the browser engine is graded, not just the native build
cd web && node scripts/wasm-parity.mjs --shard 0/4
```

### Bundle Budget Gate

The WebAssembly engine binary is strictly gated in CI against a ceiling of **5,120 KiB** raw to maintain fast download and startup performance on web and mobile devices (current build: ~3,888 KiB raw / ~1,753 KiB gzipped). The ceiling was raised from 4,096 KiB on 2026-09-10 to link every pure fluid the property alias table names; `.github/workflows/ci.yml` carries the full ledger of why each raise happened.

---

## Documentation

- [`ARCHITECTURE_AND_REQUIREMENTS.md`](ARCHITECTURE_AND_REQUIREMENTS.md): Detailed architectural specification, numerical solver pipelines, property algorithms, DAE formulations, and system requirements.
- [`CLAUDE.md`](CLAUDE.md): Developer and AI assistant guidelines, build instructions, invariant constraints, and debugging notes.
- [`NEXT_STEPS.md`](NEXT_STEPS.md): Active engineering roadmap, upcoming capabilities, and change control standards.
