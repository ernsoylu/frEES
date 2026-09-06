# frees — Phased Improvement Plan

Date: 2026-09-05
Baseline commit: `3376d61`
Status: planned; no implementation changes have been made for this plan.

Improve current-code performance and maintainability first, then complete and improve capabilities. Establish browser correctness and request integrity before accepting numerical optimizations.

This plan follows the repository review. [PLAN.md](PLAN.md) remains the historical port plan; this file tracks the next implementation work. Existing decision records and frozen oracle fixtures remain authoritative for compatibility and numerical behavior.

## Baseline and evidence

The review verified:

- Native release tests passed, with the suite's existing ignored tests unchanged.
- Native parity replay matched all 1,308 fixtures under the declared tolerances.
- All 424 frontend tests passed.
- Direct calls to the existing local WASM artifact reproduced ignored equation overrides, ineffective `findAllSolutions`, missing function-table support in optimization, and errors in documented unit examples.
- Code inspection identified redundant constraint solves, repeated preparation in analysis loops, a dense intermediate in the sparse DAE path, missing response-revision checks, and incomplete worker recovery.
- The repository records 107 native-versus-WASM numerical differences. That count is historical evidence from Wave T5, not a fresh full-WASM replay performed during this review. Differences must be classified against the numerical contract rather than assumed to be 107 incorrect results.

Performance improvements below are candidates, not measured speedup promises. Rebuild artifacts and establish comparable measurements before implementation.

## Delivery order

| Phase | Outcome | Dependency | Relative effort |
| --- | --- | --- | --- |
| 0 | Real WASM regression coverage and reproducible performance baselines | None | Medium |
| 1 | Correct request handling, cancellation, and result ownership | Phase 0 contract tests | Medium |
| 2 | Remove unused code and obsolete configuration | Phase 0 baseline | Small |
| 3 | Eliminate redundant optimization solves | Phases 0–1 | Medium |
| 4 | Reuse preparation across repeated solves | Phase 3 | Medium–large |
| 5 | Reduce sparse-solver memory and remaining measured overhead | Phase 4 profile | Medium |
| 6 | Complete existing capabilities and align documentation | Phases 0–1; performance phases precede broader feature work | Medium |
| 7 | Add further throughput or modeling capabilities when justified | Measured need after earlier phases | Conditional |

Effort describes relative implementation complexity, not a delivery-date commitment. Each phase should land in small, independently reviewable changes. Fix incorrect executable documentation early even though the broader documentation work belongs to Phase 6.

## Phase 0 — Establish the browser correctness and performance gates

### Work

- [ ] Build a fresh WASM artifact and record commit, Cargo features, Rust version, wasm-pack/wasm-opt versions, and browser/Node versions.
- [ ] Add corpus replay through that artifact. Carry solver-request and function-table sidecars into each run; compare variables, error classifications, trajectories, event times, and relevant structural metadata.
- [ ] Reuse the existing numerical comparison rules. Record each remaining browser divergence with its mechanism and measured magnitude; do not loosen global tolerances to obtain a green gate.
- [ ] Add real-boundary regression cases for overrides, multiple roots, stopping controls, and imported tables. Tests must verify returned values or explicit refusals, not only that the frontend sends a field.
- [ ] Add a worker-level test for fatal failure/recovery and an application test for stale response handling as Phase 1 fixes land.
- [ ] Pin tested build-tool versions where the current configuration follows a moving release, and record deliberate upgrades.
- [ ] Extend existing benchmarks with a constrained optimization, a 1,000-row sweep, a large component network, a stiff thermofluid transient, and a large sparse DAE.
- [ ] Measure cold initialization separately from warm execution. Separate model preparation, numerical solving, serialization, worker round trip, and rendering where practical.

### Acceptance criteria

- [ ] CI tests the actual generated WASM artifact, in addition to the existing native corpus and offline browser smoke test.
- [ ] A deliberately perturbed result or trajectory causes the WASM comparison to fail.
- [ ] No unexplained browser divergences are hidden by the baseline; any temporary known-failure accounting is explicit and cannot silently grow.
- [ ] Benchmarks verify successful, correct results before timing. Each retained optimization has before/after results from equivalent builds and workloads.

Use instruction counts for small native hot-path changes where practical, following the repository's existing profiling approach. Use repeated browser measurements for user-visible latency; avoid interpreting a contended wall-clock run as an engine regression.

Primary files: [.github/workflows/ci.yml](.github/workflows/ci.yml), [native parity harness](crates/frees-core/tests/parity.rs), [native benchmarks](crates/frees-core/benches/solve_bench.rs), [browser benchmarks](web/bench/wasm-bench.spec.ts), [recorded browser differences](CLAUDE.md).

## Phase 1 — Make requests and long operations reliable

### Work

- [x] Deserialize equation overrides in the WASM request and apply the same semantics to Check and Solve. Preserve case-insensitive, last-write-wins behavior and unit conversion.
- [x] Reuse and harden the existing override implementation used by Monte Carlo and parameter fitting. Cover comments, semicolon-separated equations, and block-local names before extending its use; do not implement a separate frontend substitution path.
- [x] Associate Check, Solve, and table requests with a model revision. Include relevant input changes such as source, tables, overrides, complex mode, and project replacement. Discard responses for obsolete revisions.
- [x] Prevent Check-then-Solve from using a check result for a different model revision. Schedule a fresh check when an edit invalidates an in-flight check.
- [x] Add Stop using worker termination and the existing reject/reset mechanism. Reject pending work, ignore late messages, preserve the document, and explicitly reset worker-owned REPL state.
- [x] Distinguish fatal initialization failures and WASM traps from ordinary document errors. Fatal failures must retire the worker; ordinary errors must leave it usable.
- [x] Define and enforce operation-wide elapsed-time budgets for steady solving and analysis operations, including nested transients. A table or fitting budget must not apply only between potentially lengthy solves.
- [x] Remove or disable the active `changeInVariables` control until its behavior is implemented and tested. Preserve compatibility data where necessary; do not silently advertise an ignored criterion.
- [x] Make unsupported behavior explicit for the multiple-solutions option until Phase 6 wires it.

### Acceptance criteria

- [x] For `x = 2; y = x^2`, an override `x = 3` returns `x = 3` and `y = 9`; Check evaluates the same overridden model.
- [x] Changing the document or project during a solve cannot display its obsolete results as current or overwrite new variable/table state.
- [x] Stop ends worker computation promptly, the editor remains usable, and a subsequent solve succeeds on a fresh worker.
- [x] A recoverable document error does not reset the worker; a fatal failure does not strand subsequent requests on a broken instance.
- [x] Each visible stopping control either changes verified behavior or clearly states its supported scope.

Primary files: [WASM request boundary](crates/frees/src/lib.rs), [override helper](crates/frees-core/src/analysis/montecarlo.rs), [analysis boundary](crates/frees/src/analysis.rs), [engine client](web/src/wasm/engineClient.ts), [worker](web/src/wasm/engine.worker.ts), [App](web/src/App.tsx), [preferences](web/src/PreferencesModal.tsx).

## Phase 2 — Remove unused code and configuration

### Work

- [x] Remove the unused remote submit/SSE/poll adapter and its dedicated tests after confirming there are still no product callers.
- [x] Remove the unused PDF/EPS `exportVector` rejection stub. Retain working SVG/PNG/JPG exports.
- [x] Remove the unused `serde-wasm-bindgen` dependency, unused workspace `thiserror` declaration, and obsolete Excalidraw package overrides. Update lockfiles as appropriate.
- [x] Remove the temporary `require_units!` panic-swallowing test bypass; unit-registry failures must fail tests.
- [x] Replace superseded comments about removed or unwired features with current behavior. Keep historical rationale in decision documents.

### Acceptance criteria

- [x] No remaining imports or product callers reference removed code.
- [x] Applicable Rust and frontend checks pass, including a production build and offline smoke test for frontend/build changes.
- [x] Existing project round-trip tests continue preserving inert spreadsheet/analyzer data.
- [x] Record actual source/dependency reduction. The review's roughly 350-line and one-direct-dependency estimate is a planning estimate, not a guaranteed bundle reduction.

Keep `lodash` and `react-responsive-carousel`: they satisfy grid peer dependencies. Do not remove numerical compatibility code solely because it resembles an existing library function.

Primary files: [api.ts](web/src/api.ts), [remote-adapter tests](web/src/api.async.test.ts), [parser tests](crates/frees-core/src/parser/expr.rs), [workspace manifest](Cargo.toml), [WASM manifest](crates/frees/Cargo.toml), [frontend manifest](web/package.json).

## Phase 3 — Solve each optimization candidate once

### Work

- [x] Change single-objective constrained optimization to obtain the objective and constraint values from one candidate solve.
- [x] Adapt the existing Pareto implementation, which already appends constraint expressions and solves them together.
- [x] Preserve expression context, temporary-name collision handling, infeasible-candidate behavior, diagnostics, and the public meaning of evaluation counts.
- [x] Avoid repeated final constraint solves when the corresponding candidate result is already available and valid.

### Acceptance criteria

- [x] A candidate with several constraints no longer triggers one complete model solve per constraint.
- [x] Feasible and infeasible outcomes, objectives, decisions, and diagnostics remain correct under the existing contracts.
- [x] Benchmarks report both full-model solve counts and end-to-end time. Reduced solve count must produce a measured improvement on representative constrained workloads.

Primary files: [single-objective optimizer](crates/frees-core/src/analysis/optimizer.rs), [existing Pareto pattern](crates/frees-core/src/analysis/pareto.rs).

## Phase 4 — Prepare once, solve repeatedly

### Work

- [ ] Extract a reusable preparation path from existing engine machinery rather than introducing a second parser or a general compilation framework.
- [ ] Start with ordinary parametric rows. Reuse parsed definitions, expansion results, block structure, derivatives, and scratch storage when structure is unchanged.
- [ ] Update numeric pins between runs. Rebuild when the source, definitions, complex mode, or set of pinned variables changes; table rows may have different filled-input sets.
- [ ] Extend reuse to optimization, Monte Carlo, and parameter fitting after the table path is verified.
- [ ] Preserve warm-start order, property-backend seed behavior, diagnostics, and accessor-dependent fixed-point passes.
- [ ] Cache parsed document definitions in the REPL session and invalidate them when the solved session changes.
- [ ] Investigate returning Monte Carlo's base solution to its caller to avoid the boundary's additional base solve, verifying that request/spec conversion remains equivalent before reusing it.

### Acceptance criteria

- [ ] Repeated runs with identical structure prepare once; structural changes trigger preparation and never reuse stale state.
- [ ] Changing tables, definitions, mode, or input pin sets has dedicated regression coverage.
- [ ] Numerical outputs retain the required parity, including trajectories and branch-sensitive cases.
- [ ] Sweeps and repeated analyses show reduced preparation counts and measured throughput gains without unbounded cache growth.

Keep caches scoped to the operation or current session. Do not introduce a global model cache without evidence that these lifetimes are insufficient.

Primary files: [engine and PreparedPinnedSolver](crates/frees-core/src/engine.rs), [analysis routines](crates/frees-core/src/analysis), [REPL session](crates/frees/src/repl.rs).

## Phase 5 — Reduce numerical memory and remaining measured overhead

### Work

- [ ] Add a colored finite-difference Jacobian path that writes directly into existing CSC storage, removing the sparse DAE path's dense intermediate.
- [ ] Reuse perturbation and residual buffers where measurements justify it, retaining perturbation sizes and arithmetic order.
- [ ] Measure moving settings validation and bound preparation out of repeated prepared Newton calls. Retain validation at public/trust boundaries and invalidate cached bounds when inputs change.
- [ ] Measure a separate speed-oriented native build profile. Evaluate WASM compiler settings independently against download size, startup, and solve time.
- [ ] Profile large-result serialization and rendering. Remove redundant payload construction only if material; retain numeric precision and required response compatibility.

### Acceptance criteria

- [ ] Sparse Jacobian construction no longer allocates an `n × n` buffer. Record peak memory versus dimension and nonzero count; factorization fill may still grow.
- [ ] Dense/sparse equivalence and stiff-transient regressions pass.
- [ ] Retain only changes that outperform measurement noise on their target workload without material regressions elsewhere.
- [ ] Browser bundle remains within the current 4 MiB raw budget, and offline behavior remains verified.

Preserve the one-entry property cache, slot evaluator, dense variable storage, and reusable Newton workspaces. Larger property caches have already been investigated and rejected for fidelity or performance reasons; reopen that decision only with new evidence.

Primary files: [DAE Jacobian](crates/frees-core/src/dae/jacobian.rs), [DAE solver](crates/frees-core/src/dae/solver.rs), [Newton solver](crates/frees-core/src/solver/newton.rs), [slot evaluation](crates/frees-core/src/solver/slots.rs), [release profile](Cargo.toml), [response serialization](crates/frees/src/lib.rs).

## Phase 6 — Complete capabilities and make usage trustworthy

### Work

- [ ] Wire the existing all-roots solver through the engine, WASM request, response, and UI. Reuse the prepared model rather than duplicating expansion.
- [ ] Describe the feature as bounded multiple-solution search. Expose bounds, search limits, and the 32-solution cap; do not imply mathematical completeness.
- [ ] Pass imported function tables consistently through optimization and Pareto operations, preserving the established definition-collision rules.
- [ ] Derive supported symbols and capability flags from the shipped engine. Retain authored descriptions, but stop treating the Java reference inventory as proof of browser support.
- [ ] Correct invalid units syntax, double conversion examples, the dimensionally inconsistent README example, and claims of universal symbolic differentiability.
- [ ] Replace in-app RabbitMQ/Redis/REST deployment instructions with the actual browser-worker architecture and supported local workflows.
- [ ] Classify documentation snippets as runnable documents or syntax fragments. Run complete examples through WASM and assert expected values or intentional diagnostics.
- [ ] Improve signature-only reference pages by prioritizing commonly used functions and functions associated with observed failures.
- [ ] Improve solver diagnostics using existing block/residual data: identify the failed block, limiting budget, and relevant bounds or property failures. Add counters only when they answer a concrete diagnostic question.
- [ ] Document CLI/browser feature differences, especially the CLI's default property backend. Add request-JSON support if scripting needs the same table and solver inputs as the browser.

### Acceptance criteria

- [ ] `x^2 = 4` with suitable bounds and multiple-solution search returns both `-2` and `+2`, with the correct response structure.
- [ ] A model using an imported `curve(x)` table works in both Solve and Optimize; equivalent Pareto coverage passes.
- [ ] Every active capability control maps to tested engine behavior or an explicit unsupported state.
- [ ] Runnable unit examples produce correct values and intended warnings; syntax fragments are not presented as complete runnable models.
- [ ] Documentation capability checks remain valid in a standalone checkout without the Java reference repository.

Primary files: [all-roots solver](crates/frees-core/src/analysis/allroots.rs), [WASM analysis boundary](crates/frees/src/analysis.rs), [editor](web/src/EquationEditor.tsx), [documentation sources](web/src/docs), [manifest generator](web/scripts/build-doc-manifest.mjs), [documentation checker](web/scripts/check-doc-coverage.mjs), [CLI](crates/frees-cli/src/main.rs).

## Phase 7 — Conditional extensions

These are candidates, not automatic implementation commitments.

| Candidate | Start only when | Constraints and proof |
| --- | --- | --- |
| Small worker pool for independent sweep rows | Serial sweeps remain a significant measured delay after preparation reuse | Bound worker count and memory; keep accessor-dependent sweeps serial; compare results across worker counts |
| Parallel Monte Carlo or Pareto evaluation | Throughput need remains and reproducibility is defined | Preserve random-stream semantics and account for sequential warm starts/property seeds before claiming independence |
| Additional linked fluids | A concrete user model requires them | Verify supported states and accuracy; measure bundle and memory cost |
| Additional CAS patterns or numerical methods | A documented model cannot be handled by existing capabilities | Provide a concrete regression/oracle case and explicit validity limits |
| Further sparse ordering improvements | Profiles show excessive factorization fill | Measure fill and solve time on the affected matrix family before adding an ordering implementation |

Do not add a backend, shared-memory threading, a replacement UI framework, a general bytecode VM, or new numerical dependencies as part of the default plan.

## Completion and change control

- [ ] Land each change with the smallest meaningful regression check for its behavior.
- [ ] Run the relevant tests during development; run applicable format, native/WASM lint, parity, frontend build/test, and browser gates before landing affected changes.
- [ ] Leave frozen corpus inputs untouched unless deliberately regenerated through the documented oracle process. Never edit goldens merely to match a new implementation.
- [ ] Record before/after workload, build configuration, numerical comparison, time/instruction counts, peak memory, and bundle impact for performance changes.
- [ ] Preserve project migration and round-trip guarantees, including data for deliberately removed features.
- [ ] Mark phase checkboxes complete only after their acceptance criteria pass. Record completed phases and measured outcomes below.

## Execution log

| Date | Phase | Change / evidence | Status |
| --- | --- | --- | --- |
| 2026-09-05 | Planning | Review findings converted into this phased plan; implementation has not started | Planned |
| 2026-09-05 | 1 | Equation overrides reach Solve and Check (`crates/frees/src/lib.rs`). The DTO had no `overrides` field, so every REPL assignment and every slider drag was parsed and dropped. Both entry points now shadow the source with `analysis::montecarlo::apply_overrides(source, …)` — the existing Java transcription, no second substitution path — before the solve, `fillMissing`, the REPL workspace and the `errorLine` lookup, so none of them can disagree about what was solved. `overrides` is `Option<Vec<String>>` like every other nullable DTO field: `"overrides": null` had meant `Invalid request`. Evidence: `crates/frees/tests/overrides.rs`, 13 tests; `cargo test -p frees` 131 pass; fmt + clippy (native and wasm32) clean; 424 frontend tests and `tsc --noEmit` clean under Node 22. Verified in the **built artifact** as well, not only natively — a Node replay through `web/src/wasm/pkg` returns `x = 3, y = 9`, flips Check on `y = x^2` from unsolvable to solvable, and converts `P = 250 [kPa]` to 250 000 Pa. Bundle: baseline 3117.4 KiB raw / 1271.1 gz → **3118.2 / 1271.9**, delta **+0.8 KiB raw**, 76.1 % of the 4096 KiB budget (both figures measured from wasm-pack release builds of this tree, with and without the change; CLAUDE.md's 3087.8 KiB predates the moist-air component commit and is not the baseline). `frees-core` is untouched, so the parity corpus is unaffected | Done |
| 2026-09-05 | 1 | Two controls stopped advertising behaviour nothing implements. **Change in variables** (`web/src/PreferencesModal.tsx`) is disabled and says so — the engine's per-block stop rule is the relative residual alone; the value stays in `StopCriteria` so saved projects and the request still round-trip. **Find all solutions** (`web/src/WorkspaceChrome.tsx`) is disabled and labelled *Not available in the browser engine yet* — `analysis/allroots.rs` exists but no request reaches it until Phase 6. The `findAll` state and its plumbing are left in place for that wiring | Done |
| 2026-09-06 | 1 | Completed Phase 1 reliability improvements: Model revision tracking with monotonic counter (`web/src/modelRevision.ts`, `web/src/modelRevision.test.ts`), in-flight request revision gating in `App.tsx` (dropping stale solves/checks on document edits, scheduling recheck when idle), Stop action with graceful worker lifecycle (`wasmStop()`, `engine.worker.ts`, `engineClient.ts`, `web/src/requestCoordination.test.ts`, `WorkspaceChrome.tsx` Stop button & Escape shortcut) distinguishing fatal traps vs recoverable errors, and operation-wide elapsed-time deadline checks in `newton.rs`, `engine.rs`, and `analysis.rs` with `crates/frees/tests/budgets.rs`. All acceptance criteria met. | Done |
| 2026-09-06 | 2 | Cleaned unused code and configuration: Removed unused remote submit/SSE/poll adapter (`runCompute`, `pollJob`, `JobState`, `ComputeOutcome`, `API_BASE`) from `web/src/api.ts` and deleted its test suite `web/src/api.async.test.ts`; removed unused `exportVector` rejection stub from `api.ts` (keeping client-side SVG/PNG/JPG exports in `exportPlot.ts`); removed unused `serde-wasm-bindgen` dependency from `crates/frees/Cargo.toml`, unused workspace `thiserror` dependency from root `Cargo.toml`, and obsolete Excalidraw package overrides from `web/package.json` (pruning 7 crates from `Cargo.lock`); removed `require_units!` panic-swallowing bypass and `units_ready` from `crates/frees-core/src/parser/expr.rs`; updated superseded comments on PID tuning and plant extraction. Net reduction: 428 lines of dead code and 7 dependencies removed. All 1,308 core parity tests, 127 WASM tests, 429 vitest tests, type checks, linter, and production build pass cleanly. | Done |
| 2026-09-06 | 3 | Unified single-objective constrained candidate solves (`crates/frees-core/src/analysis/optimizer.rs`): Appends `<prefix><i> = <c.lhs_expr>` constraint equations alongside decisions in a single pass (`solve_candidate`), adapting the Pareto pattern (`pareto.rs`). Solves the objective and all $C$ constraints simultaneously in one Tarjan pass instead of $(1 + C)$ full system solves per Nelder-Mead candidate probe. Avoids repeated final constraint solves by reading constraint values directly from candidate solutions in `update_and_check_constraints` and `build_constraint_warning`, stripping temporary prefix variables (`strip_constraint_variables`) from `solution.values`, `display_names`, `inferred_units`, and `residuals` before returning. Added regression and benchmark test suite (`crates/frees-core/tests/constrained_optimizer.rs`). On a 3-decision, 3-constraint workload (420 candidate evaluations), model solves reduced by ~4× (from ~1,680+ down to 421 solves) with sub-second execution (967ms). Preserved 100% oracle parity on all test suites: 1,309 core tests, 127 WASM tests, 429 vitest tests, cargo fmt, and cargo clippy pass cleanly. | Done |
