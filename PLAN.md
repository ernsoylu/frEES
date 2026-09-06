# frees — Complete Port Plan

**Goal:** run the entire frees engine — parser, unit checker, solvers, property models, component system, CAS, control design, and data analyzer — as Rust compiled to WebAssembly, inside the browser tab, with no backend.

**Source of truth:** `../frEES` (MIT). Read-only reference. Nothing in this plan modifies it.

Companion documents:
- [`docs/dependency-map.md`](docs/dependency-map.md) — every Java/native dependency, its version, and its Rust replacement
- [`docs/feature-inventory.md`](docs/feature-inventory.md) — every feature, its source files, and the phase that ports it

---

## 1. What is actually being ported

Measured, not estimated:

| Surface | Size |
|---|---|
| `backend/core` (the port target) | **134 Java files, 38,181 LOC** |
| `backend/web` (mostly deleted, contract retained) | 28 files, 6,788 LOC |
| Test suite (the correctness oracle) | **197 files, 24,359 LOC, 1,237 `@Test` methods** *(stale — the Phase-12 harvest re-measured the reference checkout at 184 files / 1,179 `@Test` methods, of which only ~625 carry a `.frees` document that can become a fixture; see `docs/status-phase12.md` §1. Every later "1,237-test corpus" mention in this plan inherits this correction.)* |
| ANTLR grammar `Frees.g4` | 632 lines, ~55 parser rules |
| Standard component library | **295 components** across 13 `.frees` files (168 KB) |
| Intrinsic function names (`FunctionRegistry`) | 275 |
| `Evaluator` dispatch arms | 226 |
| Frontend (reused nearly as-is) | 155 TS/TSX files, 78,891 LOC |
| Frontend → backend call sites | **22 functions in exactly 2 files** |

### Three facts that make this tractable

1. **The component library is data, not code.** All 295 components are `.frees` DSL text in `core/src/main/resources/components/`. Once the parser and expander work, the entire library ports by `include_str!`. Zero translation.
2. **The Symja dependency is four files and a string boundary.** `CasEngine` sends a string to Symja and parses a string back. Only 13 operations are reachable. The heavy control-theory math (`PolynomialHelpers` 988 LOC, `ControllerDesign` 912 LOC, `TransferFunction`, `TimeResponse`, `StateSpace`) is **frees' own Java**, not Symja.
3. **The native dependency surface is four C functions.** `CoolProp.java` binds `PropsSI`, `Props1SI`, `HAPropsSI`, `get_global_param_string` — nothing else. CoolProp already ships an official Emscripten build (`coolprop.js` + `coolprop.wasm`).

### One fact that makes it expensive

`backend/core` is dense engineering, not boilerplate. The five largest files alone are 10,779 LOC: `EquationParser` (3,042), `EquationSystemSolver` (2,441), `Evaluator` (2,053), `ControlSystemsFlattener` (1,978), `ComponentExpander` (1,656). There is no shortcut through them.

---

## 2. Target architecture

```
frees/
├── crates/
│   ├── frees-core/          # pure Rust engine — compiles for native AND wasm32
│   │   ├── ast/             # Expr, Equation, Statement, ComponentDef, …
│   │   ├── parser/          # lexer + recursive-descent parser, expander, registries
│   │   ├── units/           # UnitRegistry, UnitChecker, Quantity
│   │   ├── solver/          # Blocker (Tarjan), NewtonSolver, EquationSystemSolver
│   │   ├── ode/  dae/       # integrators, DynamicSolver, events
│   │   ├── props/           # property models + table backend
│   │   ├── cas/             # rational-function CAS, transfer functions, control design
│   │   ├── measurement/     # MDF4/CSV, calculated signals
│   │   └── components/      # expander + include_str! of the 295-component library
│   ├── frees-props-coolprop/ # optional CoolProp bridge (native FFI / wasm JS-import)
│   ├── frees/          # wasm-bindgen boundary + worker protocol (thin)
│   └── frees-cli/           # native binary: headless solve + parity harness runner
├── tools/
│   └── golden-dumper/       # small Java app depending on the frEES core jar; emits
│                            # golden fixtures from the 1,237-test corpus
├── web/                     # frontend, vendored from ../frEES/frontend
├── fixtures/                # language-neutral golden test corpus (JSON)
└── xtask/                   # build orchestration (wasm-pack, table generation, size gate)
```

**`frees-core` never depends on `wasm-bindgen`.** It compiles natively so the parity harness runs at full speed against the Java oracle, and it compiles to `wasm32` for the browser. The `frees` crate is a thin adapter. This is non-negotiable: a port you can only test in a browser is a port you cannot test.

### Runtime topology in the browser

```
main thread                      dedicated Web Worker
┌────────────────────┐          ┌──────────────────────────┐
│ React 19 UI        │          │ frees.wasm          │
│ (unchanged)        │  post-   │  ├─ frees-core           │
│                    │ Message  │  ├─ property tables      │
│ api.ts  ───────────┼─────────►│  └─ coolprop.wasm (lazy) │
│ (fetch → RPC shim) │◄─────────┤                          │
└────────────────────┘          └──────────────────────────┘
```

The 22 `api.ts` functions keep their exact async signatures. `fetch(...)` becomes `worker.request(...)`. The React tree does not know the server is gone.

### What disappears

RabbitMQ, Redis, the job store, SSE streaming, `RequestGuardFilter` rate limiting, CORS config, the nginx `/api` proxy, client-IP trust logic, the `202 Accepted` + poll protocol, OpenTelemetry/Jaeger, the Python `mdf-sidecar`, Docker Compose, and both Dockerfiles. Roughly 6,800 LOC of `backend/web` plus the entire deployment substrate collapses into a static bundle on a CDN.

### What this unlocks (new capability, no upstream equivalent)

Offline operation, zero-cost hosting, instant solve latency (no network round-trip), true data privacy (measurement files never leave the machine), and a PWA install path.

---

## 3. The three Phase-0 decisions

These change everything downstream. Each gets a timeboxed spike before any porting starts.

### D1 — Property backend strategy ★ highest leverage

CoolProp is called from inside the Newton inner loop. Every Jacobian column re-evaluates properties, so call cost multiplies by (variables × iterations × blocks).

| Option | Cost per call | Bundle | Coverage |
|---|---|---|---|
| A. `coolprop.wasm` as a second module, called through JS | JS boundary crossing per miss | ~3–6 MB | Complete |
| B. Whole engine on `wasm32-unknown-emscripten`, CoolProp linked in via `extern "C"` | Native FFI, no boundary | ~3–6 MB | Complete; loses `wasm-bindgen` tooling |
| C. **Precomputed property tables** (`PhPropertyTable` + `SaturationSplitTable` + `PhTableRegistry`, already in `core` but dormant) generated at build time by native CoolProp | Table interpolation, in-Rust | ~100s of KB per fluid | Common fluids/ranges only |

**Recommendation: C as the hot path, A as the lazy-loaded fallback.** frees already contains a bicubic `(P,h)` table backend with *analytic* first derivatives — which is strictly better for Newton than finite-differencing CoolProp, and is exactly what `../frEES/docs/roadmap.md` item 18 ("wire the dormant `PhPropertyTable`") points at. Generate tables offline with native CoolProp via an `xtask`; ship `coolprop.wasm` as a lazily fetched chunk for fluids, ranges, and `HAPropsSI` calls the tables don't cover. Keep the existing 20k-entry LRU caches in Rust so the JS boundary is crossed only on a genuine miss.

**Spike:** benchmark a representative thermofluid document (from `CycleExamplesTest` / `HvacExamplesTest`) under A and C. Decide on measured numbers.

### D2 — Wasm target and toolchain

`wasm32-unknown-unknown` + `wasm-bindgen`/`wasm-pack` (mature tooling, clean Vite integration) versus `wasm32-unknown-emscripten` (needed only if D1 lands on option B). **Default: `wasm32-unknown-unknown`.** Revisit only if the D1 spike shows the JS boundary dominating.

### D3 — Threading

`wasm-bindgen-rayon`/SharedArrayBuffer requires COOP/COEP headers, which constrains hosting and blocks embedding in third-party pages. **Recommendation: single-threaded engine; parallelism via a pool of independent workers**, each with its own wasm instance. Parametric sweeps, Monte Carlo, and NSGA-II are embarrassingly parallel and need no shared memory. This keeps the app deployable as plain static files with no special headers.

---

## 4. Correctness strategy — the parity harness

This is the single most important engineering decision in the project. 1,237 JUnit tests encode the engine's behavior; they are the specification.

**Do not hand-translate 24,359 lines of test code.**

1. Build `tools/golden-dumper` — a small Java program in *this* repo that depends on the published `frees-core` jar (or a locally built one) and, for each corpus document, records: solved variable values, units, block structure, diagnostics, and error messages.
2. Emit a language-neutral corpus in `fixtures/`: `{ source: "<frees DSL>", settings: {...}, expect: { vars: {...}, units: {...}, errors: [...] } }`.
3. `frees-cli` replays that corpus against the Rust engine in CI. Every phase adds fixtures; no phase is "done" until its slice of the corpus is green.
4. Seed the corpus from the existing example documents (`frontend/src/examples.ts`, 51 KB), the docs corpus (`frontend/src/docs/`), and the `*ExamplesTest` classes — these are already whole-document round-trips.

**Numerical tolerance policy.** WASM `f64` arithmetic is IEEE-754 deterministic, but transcendentals (`exp`, `ln`, `pow`, `sin`) are not specified and differ between the JVM and Rust's libm. Compare with relative tolerance (`1e-9` for solved values, looser for iterative/stochastic results), never bit-equality. Use the `libm` crate uniformly so native and wasm builds of the Rust engine agree with each other exactly.

---

## 5. Phased plan

Every phase ends in something that compiles, tests, and runs — inheriting the parent's "Working Software First" rule.

Effort is in **dev-weeks for one experienced Rust engineer**, order-of-magnitude.

### Stage I — Foundations (Phases 0–3): a browser that solves

| # | Phase | Ports | Exit criteria | Effort |
|---|---|---|---|---|
| **0** | **Foundations & spikes** | — | `rustup` + `wasm32-unknown-unknown` + `wasm-pack` installed; workspace skeleton builds native **and** wasm; D1/D2/D3 decided on measured numbers and recorded in `docs/decisions/`; golden-dumper emits a first fixture; bundle-size gate wired into CI | 3–4 |
| **1** | **Language core** | `Frees.g4` (632 lines) → Rust lexer + recursive-descent parser; `ast/Expr`, `Equation`, `Statement`, `ProcDef`; `AstBuilder` (1,587); `Evaluator` (2,053, 226 arms); `units/` (`UnitRegistry` 588, `UnitChecker` 798, `Quantity` 93); `ConstantsRegistry`; `StringVariables` | `frees-cli parse` + `eval` handle every construct in the grammar; parser/unit fixtures green; all 295 library components **parse** | 8–10 |
| **2** | **Steady solver** | `Blocker` (384, Tarjan via `petgraph`); `NewtonSolver` (832, step-halving); `EquationSystemSolver` (2,441); `Block`, `VariableSpec`, `SolverSettings`, `SolverException`; `GuessDirective`; check-before-solve | `POST /api/check` and `/api/solve` semantics reproduced headlessly; `EquationSystemSolverTest` corpus green; diagnostics carry source positions | 6–8 |
| **3** | **★ Browser vertical slice** | `frees` bindings; Web Worker protocol; `api.ts` → RPC shim; vendor `web/` from `../frEES/frontend` | **The app solves in the browser with the network disconnected.** Editor → Check → Solve → Solution table, end to end, no server. Bundle under budget. This is the milestone that proves the thesis. | 4–5 |

### Stage II — The engine's breadth (Phases 4–7)

| # | Phase | Ports | Exit criteria | Effort |
|---|---|---|---|---|
| **4** | **Function library & kernels** | `FunctionRegistry` (275 names); `Differentiator` (536); `IntegralSolver` (439); `ComplexExpansion` (618); `LinearAlgebra`, `Statistics`, `SignalProcessing`, `Interpolation2D`, `CurveInterpolator`; `ProcedureEvaluator`; `LatexConverter` (271); arrays, matrices, `FUNCTION`/`PROCEDURE`/`MODULE`, `SYMBOLIC` | Every intrinsic in the reference (`/api/reference`) resolves and matches golden values; matrix/complex/procedural fixtures green | 8–10 |
| **5** | **Properties & materials** | All 28 files in `props/` (~5.4k LOC): `IdealGas`, `NasaThermo`, `CubicEos`, `Psychrometrics`, `Thermochemistry`, `Combustion`, `Equilibrium`, `GasTransport`, `HeatExchanger`, `HxCorrelations`, `CompressibleFlow`, `ConvectiveHeat`, `TwoPhase`, `Pneumatics`, `FlowResistance`, `Atmosphere`, `SolidProperties`, `HeislerCharts`, `PeriodicTable`, `ChemicalFormula`, `PropertyDiagrams`; the D1 table backend + `xtask` generator; `coolprop.wasm` lazy chunk | Fluid list, property diagrams, psychrometric chart render from the wasm engine; property fixtures within tolerance; table-vs-CoolProp error bounded and documented | 10–12 |
| **6** | **Component system** | `ComponentExpander` (1,656); `ComponentLibrary`; `ComponentDef`/`ComponentInst`/`ConnectDecl`; four domains + junction rules; `domain$` connector separation; `VARIANT`/`REQUIRE`; `ComponentMetadata`; `CyclePathResolver` (669); embed the 295-component library | All 295 components instantiate and solve; connector-domain violations still hard-fail; cycle plots and schematic readouts work; `CycleExamplesTest`/`HvacExamplesTest` corpora green | 6–8 |
| **7** | **Dynamics** | `ode/` (17 files, ~2.9k): `OdeIntegrator`, `RungeKuttaMethod`, `BdfMethod` (ode15s), `RosenbrockMethod` (ode23s), `ButcherTableau`, events, dense output, `OdeAccessors`, `DynamicSolver` (1,194), `DynamicAnalysis`; `dae/` (7 files, ~1.0k): `DaeAssembly`, `DaeResidual`, `DaeJacobian`, `DaeRootFn`, sparse steady path; `LINEARIZE` | `DYNAMIC` blocks integrate in-browser; storage components (`ThermalMass`/`Inertia`/`Capacitor`/`Accumulator`/SOC) drive transients; event roots fire; ODE fixtures green | 8–10 |

**Note on Phase 7:** port frees' *own* ODE implementations directly — `BdfMethod` is 99 lines, `RosenbrockMethod` 95, `RungeKuttaMethod` 131. Swapping in a third-party solver would change numerical behavior for no gain. The DAE path is the only place an external solver (`diffsol`, or SUNDIALS-via-Emscripten) is worth evaluating, and a WASM build sidesteps the SUNDIALS v6/v7 ABI trap documented in `../frEES/CLAUDE.md` entirely.

### Stage III — Analysis, symbolics, data (Phases 8–10)

| # | Phase | Ports | Exit criteria | Effort |
|---|---|---|---|---|
| **8** | **Analysis & design** | `Optimizer` (719); `MultiObjectiveOptimizer` NSGA-II (477); `AllRootsSolver` (386); `CurveFitter` LM (288); `MonteCarlo` (153); `ParameterFit` (298); `ParametricTable` + `ParametricAccessorContext`; uncertainty propagation (SVD Jacobian + RSS, `UncertaintyOf`) | Min/max, Pareto fronts, parametric sweeps, curve fits, Monte Carlo, and `val ± unc` display all run in-browser; sweeps parallelised across the worker pool (D3) | 6–8 |
| **9** | **CAS & control systems ★ highest risk** | Own rational-function CAS replacing Symja's 13 ops; `ExprToSymja`/`SymjaOutputNormalizer` → internal IR; `CasIdentity`; `PolynomialHelpers` (988); `TransferFunction`; `StateSpace`; `TimeResponse`; `ControllerDesign` LQR/place (912); `PidTuner` (345); `ControlSystemsFlattener` (1,978); `ControlSystemsEvaluator` (1,140) | REPL CAS commands, `series`/`feedback`/`ss`/`tf`, Laplace round-trips, LQR/pole-placement, and PID tuning all match golden output | 10–14 |
| **10** | **Data analyzer & measurement** | `measurement/` (11 files, ~1.5k): `Mf4Parser` → Rust MDF4 crate, `TimeSeriesEvaluator` (374), `MergedRaster`, `EnvelopeDecimator`, `SampledSeries`, fallback ladder; `/api/measurements/*` equivalents | `.mf4` and CSV load **client-side** (files never leave the machine); calculated signals, decimation, and channel windows work; **the Python `mdf-sidecar` is deleted** | 4–6 |

**Note on Phase 9 — the CAS.** Nine of the 13 Symja operations (`Factor`, `Expand`, `Simplify`, `Together`, `Cancel`, `Numerator`, `Denominator`, `Apart`, `Collect`) are polynomial/rational-function algebra over exact rationals — tractable in Rust with `num-rational` + `num-bigint`. `D` routes to the already-ported `Differentiator`. `LaplaceTransform`/`InverseLaplaceTransform` for the textbook cases reduce to a transform table plus partial fractions, which `Apart` provides. **Only symbolic `Integrate` is genuinely hard** (Risch); scope it to a pattern-matched table for v1 and record the gap. `symbolica` is deliberately **rejected**: it is source-available with commercial licensing, incompatible with frees' MIT license.

### Stage IV — Product and hardening (Phases 11–12)

| # | Phase | Ports / builds | Exit criteria | Effort |
|---|---|---|---|---|
| **11** | **Browser-native product layer** | OPFS/IndexedDB project storage for `.frees` files; service worker + offline PWA manifest; share links (`lz-string` already a dependency); optional remote-fallback adapter for oversized jobs; static-hosting deploy | Installable PWA; full offline session including reload; project open/save without a server; deploys as static files with no COOP/COEP requirement | 5–7 |
| **12** | **Parity, performance, hardening** | Full 1,237-test corpus replay; benchmark suite vs the JVM engine; parser fuzzing; bundle-size budget enforcement; `console_error_panic_hook` + structured error reporting; memory-ceiling guards | Corpus green within tolerance; documented perf comparison; documented bundle breakdown; no panics on fuzzed input; wasm32 4 GB ceiling handled gracefully | 6–8 |

### Totals and critical path

**~84–110 dev-weeks ≈ 2 person-years** for one engineer. With three engineers and honest parallelization losses: **roughly 9–12 months**.

The critical path is **0 → 1 → 2 → 3**. Nothing else can start until the parser, evaluator, and solver exist. After Phase 3 the work fans out: Phases 4/5/8 are largely independent, Phase 6 needs 4+5, Phase 7 needs 6, Phase 9 needs 4, Phase 10 is independent of everything after Phase 3.

```
0 ──► 1 ──► 2 ──► 3 ──┬──► 4 ──┬──► 6 ──► 7 ──┐
                      │        └──► 9 ────────┤
                      ├──► 5 ────────────────►├──► 11 ──► 12
                      ├──► 8 ────────────────►│
                      └──► 10 ───────────────►┘
```

---

## 6. Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| **CAS from scratch (Phase 9)** underestimated | High | Timebox a Phase-0 spike on `Apart` + `Factor` over exact rationals. Fallback: keep a *thin optional* remote CAS endpoint — it is REPL-only and never in the solve loop, so an offline build degrades gracefully rather than breaking. |
| **CoolProp bundle size** blows the budget | High | D1 option C (tables) is the primary path precisely because of this. Lazy-load `coolprop.wasm` only when a document touches an uncovered fluid. Set the budget in Phase 0 and gate CI on it. |
| **Numerical drift** from the Java engine | Medium | Tolerance-based parity harness from Phase 0, not retrofitted at the end. Port frees' own integrators rather than substituting libraries. |
| **`Evaluator`'s 226 dispatch arms** are a long tail of undocumented behavior | Medium | Drive from `/api/reference` output + golden fixtures per intrinsic, not from reading code. |
| **wasm32 4 GB memory ceiling** on large parametric/Monte Carlo runs | Medium | Stream sweep results instead of accumulating; per-run memory guard; worker recycling. Phase 12. |
| **MDF4 Rust crates are immature**, and several use `memmap2` (unavailable in wasm) | Medium | Evaluate `mf4-rs` / `mdf4-rs` / `asammdf` / `mdfr` against real `.mf4` files in Phase 0 for slice-based reading. Keep the existing `FallbackMeasurementParser` ladder so CSV always works. |
| **Single-threaded solve** feels slower than the JVM on big systems | Medium | Benchmark early (Phase 3). JIT-warm JVM beats cold wasm on long runs; wasm wins on latency because there is no network hop. Worker-pool parallelism for sweeps. |
| **Frontend assumes async job semantics** | Low | Preserve the promise-based `api.ts` signatures exactly; the shim keeps the shape even though results are now immediate. |
| **LGPL in a statically linked binary** | Low but real | Java dynamically links Symja (LGPL-3.0) and JGraphT (LGPL-2.1), which satisfies relinking freedom. A wasm binary is statically linked, so **both must be replaced with permissive crates** — `petgraph` (MIT/Apache-2.0) and an own CAS. Already the plan; do not regress it. |

---

## 7. Scope decisions taken

Stated as assumptions so they can be overridden:

1. **Full feature parity is the goal** — every feature in `docs/feature-inventory.md` ships. No feature is dropped.
2. **Offline-first, server-optional.** The product works with zero backend. A remote fallback is an optional adapter (heavy jobs, symbolic integration), never a requirement.
3. **The frontend is reused, not rewritten.** React 19 + Mantine 9 + CodeMirror 6 + Plotly + Excalidraw + Univer + dockview stay. Only the 22 API functions change.
4. **MIT licensing is preserved**, which rules out Symbolica and static LGPL linkage.
5. **No COOP/COEP requirement**, which rules out SharedArrayBuffer threading in v1.
6. **The `.frees` file format and DSL are unchanged** — documents must round-trip between the JVM and wasm engines.

---

## 8. Immediate next actions (Phase 0, week 1)

1. Install `rustup`, `cargo`, `wasm32-unknown-unknown`, `wasm-pack` (the machine currently has only a distro `rustc 1.75.0` and **no cargo**).
2. Create the workspace skeleton in §2; get `cargo build` and `wasm-pack build` both green on a stub.
3. Build `tools/golden-dumper` against the frEES `core` jar; emit fixtures for ten documents from `frontend/src/examples.ts`.
4. Run the D1 property spike and the Phase-9 CAS spike; write both up in `docs/decisions/`.
5. Set the bundle-size budget and wire the CI gate before there is anything to gate.
