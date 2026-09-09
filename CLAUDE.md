# CLAUDE.md

Guidelines and reference architecture for AI coding assistants and developers working on the `frees` repository.

---

## Repository Overview & Philosophy

`frees` is a high-performance, declarative physical systems modeling, equation solving, and simulation platform written in Rust and WebAssembly, paired with a modern React 19 / TypeScript frontend.

### Core Tenets

1. **Zero External Backend**: The entire computational engine compiles to WebAssembly (`wasm32-unknown-unknown`) and executes client-side inside Web Workers, complemented by a native desktop CLI (`frees-cli`). There are no `/api/` network round-trips.
2. **Root-Cause Engineering**: Fix defects at their origin rather than patching downstream callers. Avoid speculative scaffolding, unrequested abstractions, or temporary bypasses.
3. **Target-Agnostic Core**: `crates/frees-core` contains pure mathematics, algorithms, and models; it must remain completely decoupled from browser APIs and `wasm-bindgen`.
4. **Deterministic Reproducibility**: High-precision numerical calculations, exact symbolic derivatives, frozen regression goldens, and strict tolerance tracking.

---

## Current Status & Verification Metrics

- **Solvers**: Scaled Newton-Raphson, trust-region line search, Powell hybrid dogleg, rank-deficient merge recovery, polish pass, and reusable prepared solvers (`PreparedDocument`).
- **Dynamic Systems**: Adaptive Dormand-Prince Runge-Kutta (`ode45`), 5th-order Radau IIA (`radau5`/`radauiia`), and variable-coefficient DAE BDF/IDA with zero-crossing event root-finding.
- **Properties**: Pure-Rust CoolProp 8.0.0 implementation (`rustprop`) for high-accuracy Helmholtz equations of state, cubic EoS, incompressibles, and psychrometrics (`HAPropsSI`).
- **Component Library**: 295+ standard acausal components spanning fluid networks, thermal systems, moist air HVAC, mechanics, and electrical circuits.
- **Worker Pool**: Up to 4 Web Workers executing independent parametric sweep chunks in parallel with weighted progress, preserving deterministic row ordering.
- **WASM Bundle Budget**: Strictly gated at $\le 4,096\text{ KiB}$ raw (current build: ~3,380 KiB raw / ~1,412 KiB gzipped, ~716 KiB headroom).
- **Test Suite Health**:
  - `cargo test --workspace -- --skip golden_corpus_parity`: passes across all workspace crates.
  - `cargo test --release --test parity`: 1,308/1,308 golden fixtures passing.
  - `vitest run` (Node 22): 55 test files, 620 tests passing.
  - `cargo clippy`: 0 warnings with `-D warnings` on native and `wasm32-unknown-unknown`.
  - `npm run lint`: 0 errors.

---

## Workspace Layout & Module Boundaries

```
frees-wasm/
├── Cargo.toml                    # Root workspace configuration & compiler profiles
├── rust-toolchain.toml           # Pinned stable Rust toolchain & wasm32 target
├── crates/
│   ├── frees-core/               # Pure Rust numerical engine (solvers, AST, DAE, ODE, CAS, props)
│   ├── frees/                    # WASM boundary crate (`wasm-bindgen` JSON bridge)
│   └── frees-cli/                # Headless command-line binary (`solve`, `check`)
├── web/                          # React 19 frontend
│   ├── src/
│   │   ├── wasm/                 # engineClient.ts (worker pool singleton), engine.worker.ts
│   │   ├── schematic/            # Acausal component canvas, symbols, and wiring validation
│   │   ├── tablesGrid/           # Glide Data Grid workbook, CSV import/export, formulas
│   │   ├── plots/                # Plotly.js charts, thermodynamic diagrams, decimation
│   │   ├── modelRevision.ts      # Monotonic revision coordinator preventing stale state
│   │   └── projectStore.ts       # IndexedDB multi-tab project library & autosave mirror
│   └── scripts/                  # compile-docs.js, manifest builders, parity checks
└── fixtures/                     # Frozen regression fixtures, goldens, and tolerances
```

### Module Boundary Invariants

- **`crates/frees-core` must remain target-agnostic**: Never add `wasm-bindgen`, `js-sys`, `web-sys`, or browser-specific dependencies to `frees-core`.
- **`crates/frees` is the sole WASM bridge**: Metadata and non-bulk results use JSON strings. Browser solves and sweeps carry bulk numeric tables in JS-owned `Float64Array`s transferred from workers; native JSON exports remain available.
- **Worker pool isolation (`web/src/wasm/engineClient.ts`)**:
  - Lazily spawns Web Workers up to the configured limit (clamped between 1 and 4, automatically limited to 2 on devices with $\le 4\text{ GB}$ memory).
  - Correlates in-flight requests by monotonic request ID.
  - Rejects stranded requests with `'Operation stopped'` when extra workers are retired (`retireExtraWorkers`).
  - Guards message and error callbacks (`if (!pool.includes(w)) return`) against late events from retired or terminated workers.
  - On fatal WASM traps, terminates all active workers and respawns cleanly on subsequent requests.
- **Prepared Solver Reuse (`PreparedDocument`)**:
  - Structural compilation (AST parsing, component expansion, call flattening, block decomposition, analytic differentiation, variable specs, dense plans) is hoisted.
  - Numeric pin mutations update in-place without re-compilation.
  - Lowered pins (complex mode coordinates `_r`/`_i`, stepped integrals, linearizations, array indices `x[1]`, members `part.x`) automatically route through `with_source_pins` for sound AST expansion.
  - Preserves user-declared casing in `display_names`.
- **Model Revision Coordination (`ModelRevisionTracker`)**:
  - Any edit to document text, tables, sliders, or settings increments the monotonic revision counter.
  - Background solves, checks, and sweeps verify `isCurrent(revision)` before writing results to React state, eliminating race conditions and stale UI overwrites.

---

## Essential Development Commands

### 1. Rust Engine Development

```bash
# Check formatting and strict clippy across all targets
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings

# Run core unit and integration tests (skipping lengthy unsharded golden replay)
cargo test --workspace -- --skip golden_corpus_parity

# Run specific integration test suites
cargo test -p frees-core --test prepared_solver
cargo test -p frees-core --test dae_sparse_direct
cargo test -p frees-core --test constrained_optimizer
cargo test -p frees --test overrides

# Replay full golden regression corpus (sharded or release)
cargo test --release --test parity

# Run CLI headlessly
cargo run -qp frees-cli -- solve path/to/model.frees
cargo run -qp frees-cli -- check path/to/model.frees
```

### 2. WebAssembly & Frontend Development

> **Node Version Requirement**: Node 22 is required (`web/.nvmrc`). Under Node 20, Vitest fails during DOM initialization due to a `jsdom` / `undici` uncloneable error.

```bash
# Build the WebAssembly engine package into web/src/wasm/pkg
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg

# Install dependencies and compile documentation
cd web
npm ci
npm run compile-docs

# Run frontend tests
npm test

# Run specific Vitest test file
npx vitest run src/wasm/engineClient.test.ts
npx vitest run src/analysisDialogs.test.tsx

# Run ESLint
npm run lint

# Compile production bundle and generate PWA assets
npm run build
```

---

## Coding Rules & Implementation Discipline

1. **Root-Cause Resolution**: Fix the underlying condition where all execution flows converge, rather than placing guards in individual callers.
2. **Minimal Working Diffs**: Write the simplest code that completely solves the problem. Delete dead code and unneeded abstractions.
3. **Documentation Integrity**: Keep docstrings and architectural explanations current. Quote exact user source spans in diagnostic error messages.
4. **Symbol Case-Insensitivity**: Variable and function lookup in the solver is case-insensitive, but user-defined casing must be preserved in display maps.
5. **Frozen Fixtures**: Golden fixtures in `fixtures/corpus` and `fixtures/golden` represent verified reference behavior. Never alter golden outputs merely to accommodate a code change; investigate any discrepancy down to the numerical algorithm.
6. **Lazy UI Evaluation**: Do not evaluate expensive or validating operations (such as table DTO extraction `toFunctionTableDtos()`) during React rendering; defer evaluation to user action callbacks within structured error handlers.
