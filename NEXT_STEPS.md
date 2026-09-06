# frees — Phased Improvement Plan

Date: 2026-09-06
Original baseline commit: `3376d61`; plot/table planning spot checks: `33ea6db`.
Status: Phases 0–6 and 8 recorded complete; Phase 9A–9D implemented (PR #13, 2026-09-06) but the phase is **not closed** — Playwright browser journeys and the R15 user pilot remain; Phase 10 planned; Phase 7 remains conditional.

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
| 8 | Scaled numerical performance: flattened Newton GE, reused sparse LU scratch, opt-in Radau | Phase 0 baseline | Medium |
| 9 | Trustworthy syntax assistance, component discovery, and modeling UX | Phases 0–1 and 6 foundations; independent of Phase 8 | Large, staged |
| 10 | Trustworthy tables and plots: precision, source ownership, engineering workflows, measured performance | Phases 0–1 and 6 foundations; coordinate shared revision/unit work with Phase 9 | Large, staged |

Effort describes relative implementation complexity, not a delivery-date commitment. Each phase should land in small, independently reviewable changes. Fix incorrect executable documentation early even though the broader documentation work belongs to Phase 6.

## Phase 0 — Establish the browser correctness and performance gates

### Work

- [x] Build a fresh WASM artifact and record commit, Cargo features, Rust version, wasm-pack/wasm-opt versions, and browser/Node versions.
- [x] Add corpus replay through that artifact. Carry solver-request and function-table sidecars into each run; compare variables, error classifications, trajectories, event times, and relevant structural metadata.
- [x] Reuse the existing numerical comparison rules. Record each remaining browser divergence with its mechanism and measured magnitude; do not loosen global tolerances to obtain a green gate.
- [x] Add real-boundary regression cases for overrides, multiple roots, stopping controls, and imported tables. Tests must verify returned values or explicit refusals, not only that the frontend sends a field.
- [x] Add a worker-level test for fatal failure/recovery and an application test for stale response handling as Phase 1 fixes land.
- [x] Pin tested build-tool versions where the current configuration follows a moving release, and record deliberate upgrades.
- [x] Extend existing benchmarks with a constrained optimization, a 1,000-row sweep, a large component network, a stiff thermofluid transient, and a large sparse DAE.
- [x] Measure cold initialization separately from warm execution. Separate model preparation, numerical solving, serialization, worker round trip, and rendering where practical.

### Acceptance criteria

- [x] CI tests the actual generated WASM artifact, in addition to the existing native corpus and offline browser smoke test.
- [x] A deliberately perturbed result or trajectory causes the WASM comparison to fail.
- [x] No unexplained browser divergences are hidden by the baseline; any temporary known-failure accounting is explicit and cannot silently grow.
- [x] Benchmarks verify successful, correct results before timing. Each retained optimization has before/after results from equivalent builds and workloads.

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

- [x] Extract a reusable preparation path from existing engine machinery rather than introducing a second parser or a general compilation framework.
- [x] Start with ordinary parametric rows. Reuse parsed definitions, expansion results, block structure, derivatives, and scratch storage when structure is unchanged.
- [x] Update numeric pins between runs. Rebuild when the source, definitions, complex mode, or set of pinned variables changes; table rows may have different filled-input sets.
- [x] Extend reuse to optimization, Monte Carlo, and parameter fitting after the table path is verified.
- [x] Preserve warm-start order, property-backend seed behavior, diagnostics, and accessor-dependent fixed-point passes.
- [x] Cache parsed document definitions in the REPL session and invalidate them when the solved session changes.
- [x] Investigate returning Monte Carlo's base solution to its caller to avoid the boundary's additional base solve, verifying that request/spec conversion remains equivalent before reusing it.

### Acceptance criteria

- [x] Repeated runs with identical structure prepare once; structural changes trigger preparation and never reuse stale state.
- [x] Changing tables, definitions, mode, or input pin sets has dedicated regression coverage.
- [x] Numerical outputs retain the required parity, including trajectories and branch-sensitive cases.
- [x] Sweeps and repeated analyses show reduced preparation counts and measured throughput gains without unbounded cache growth.

Keep caches scoped to the operation or current session. Do not introduce a global model cache without evidence that these lifetimes are insufficient.

Primary files: [engine and PreparedPinnedSolver](crates/frees-core/src/engine.rs), [analysis routines](crates/frees-core/src/analysis), [REPL session](crates/frees/src/repl.rs).

## Phase 5 — Reduce numerical memory and remaining measured overhead

### Work

- [x] Add a colored finite-difference Jacobian path that writes directly into existing CSC storage, removing the sparse DAE path's dense intermediate.
- [x] Reuse perturbation and residual buffers where measurements justify it, retaining perturbation sizes and arithmetic order.
- [x] Measure moving settings validation and bound preparation out of repeated prepared Newton calls. Retain validation at public/trust boundaries and invalidate cached bounds when inputs change.
- [x] Measure a separate speed-oriented native build profile. Evaluate WASM compiler settings independently against download size, startup, and solve time.
- [x] Profile large-result serialization and rendering. Remove redundant payload construction only if material; retain numeric precision and required response compatibility.

### Acceptance criteria

- [x] Sparse Jacobian construction no longer allocates an `n × n` buffer. Record peak memory versus dimension and nonzero count; factorization fill may still grow.
- [x] Dense/sparse equivalence and stiff-transient regressions pass.
- [x] Retain only changes that outperform measurement noise on their target workload without material regressions elsewhere.
- [x] Browser bundle remains within the current 4 MiB raw budget, and offline behavior remains verified.

Preserve the one-entry property cache, slot evaluator, dense variable storage, and reusable Newton workspaces. Larger property caches have already been investigated and rejected for fidelity or performance reasons; reopen that decision only with new evidence.

Primary files: [DAE Jacobian](crates/frees-core/src/dae/jacobian.rs), [DAE solver](crates/frees-core/src/dae/solver.rs), [Newton solver](crates/frees-core/src/solver/newton.rs), [slot evaluation](crates/frees-core/src/solver/slots.rs), [release profile](Cargo.toml), [response serialization](crates/frees/src/lib.rs).

## Phase 6 — Complete capabilities and make usage trustworthy

### Work

- [x] Wire the existing all-roots solver through the engine, WASM request, response, and UI. Reuse the prepared model rather than duplicating expansion.
- [x] Describe the feature as bounded multiple-solution search. Expose bounds, search limits, and the 32-solution cap; do not imply mathematical completeness.
- [x] Pass imported function tables consistently through optimization and Pareto operations, preserving the established definition-collision rules.
- [x] Derive supported symbols and capability flags from the shipped engine. Retain authored descriptions, but stop treating the Java reference inventory as proof of browser support.
- [x] Correct invalid units syntax, double conversion examples, the dimensionally inconsistent README example, and claims of universal symbolic differentiability.
- [x] Replace in-app RabbitMQ/Redis/REST deployment instructions with the actual browser-worker architecture and supported local workflows.
- [x] Classify documentation snippets as runnable documents or syntax fragments. Run complete examples through WASM and assert expected values or intentional diagnostics.
- [x] Improve signature-only reference pages by prioritizing commonly used functions and functions associated with observed failures.
- [x] Improve solver diagnostics using existing block/residual data: identify the failed block, limiting budget, and relevant bounds or property failures. Add counters only when they answer a concrete diagnostic question.
- [x] Document CLI/browser feature differences, especially the CLI's default property backend. Add request-JSON support if scripting needs the same table and solver inputs as the browser.

### Acceptance criteria

- [x] `x^2 = 4` with suitable bounds and multiple-solution search returns both `-2` and `+2`, with the correct response structure.
- [x] A model using an imported `curve(x)` table works in both Solve and Optimize; equivalent Pareto coverage passes.
- [x] Every active capability control maps to tested engine behavior or an explicit unsupported state.
- [x] Runnable unit examples produce correct values and intended warnings; syntax fragments are not presented as complete runnable models.
- [x] Documentation capability checks remain valid in a standalone checkout without the Java reference repository.

Primary files: [all-roots solver](crates/frees-core/src/analysis/allroots.rs), [WASM analysis boundary](crates/frees/src/analysis.rs), [editor](web/src/EquationEditor.tsx), [documentation sources](web/src/docs), [manifest generator](web/scripts/build-doc-manifest.mjs), [documentation checker](web/scripts/check-doc-coverage.mjs), [CLI](crates/frees-cli/src/main.rs).

## Phase 8 — Scaled numerical performance: linear algebra, stiff DAE time-stepping, and ODE event handling

Closed without claiming unmeasured speedups or oracle-breaking wirings. `colamd.rs` and `hermite_root.rs` stay as tested helpers; applying them is Phase 7.

### Work

- [x] Flatten dense Newton matrix workspaces (`crates/frees-core/src/solver/newton.rs`) from `Vec<Vec<f64>>` to contiguous 1D buffers (`Vec<f64>`) with flat index `i * n + j`. Eliminate pointer indirection overhead, improve L1/L2 cache locality, and enable LLVM SIMD auto-vectorization (`f64x2` in WebAssembly, AVX2 natively).
- [x] Implement register-tiled elimination for dense blocks ($N > 16$), unrolling row updates in $2 \times 2$ or $4 \times 4$ blocks to increase arithmetic intensity and close the gap with dense BLAS without adding external dependencies.
- [x] Add a reusable, zero-allocation `SparseLuWorkspace` to the sparse DAE solver (`crates/frees-core/src/dae/solver.rs`), hoisting scratch vectors (`x`, `pinv`, `mark`, `stack`, `pstack`, `order`) out of `SparseLu::factor` so transient solves avoid heap allocation churn across ODE steps.
- [x] Implement early Newton divergence detection in the DAE nonlinear corrector loop (`crates/frees-core/src/dae/solver.rs`, derived from `diffsol`'s $\theta$-rate estimator), aborting diverging steps at iteration 2 rather than burning maximum iterations before reducing step size. (Already present as IDA `RATEMAX = 0.9`.)
- [x] Add an optional 5th-order Radau IIA implicit Runge-Kutta solver. Shipped as opt-in `OdeMethod` `radau`/`radau5`/`radauiia` for `y' = f(t,y)`. Does not replace IDA.

### Acceptance criteria

- [x] All 1,308 golden corpus fixtures maintain 100% bit/oracle parity across all shards.
- [x] Raw WASM bundle size remains strictly < 4,096 KiB (CI `wasm build + size gate` on PR #12).

Primary files: [Newton solver](crates/frees-core/src/solver/newton.rs), [DAE sparse solver](crates/frees-core/src/dae/solver.rs), [ODE methods](crates/frees-core/src/ode/methods.rs), [native benchmarks](crates/frees-core/benches/solve_bench.rs), [browser benchmarks](web/bench/wasm-bench.spec.ts).

## Phase 9 — Align syntax assistance, components, and modeling UX

Source: [syntax and component UX review](docs/syntax-component-ux-review.md), dated 2026-09-06. Recommendation IDs below map to that report. Preserve the existing language and physics; repair assistance that generates incorrect source or misrepresents the engine before expanding discovery and learning features.

The report records a 312-component engine inventory versus 295 catalog entries, reproduced unit-suffix and signature-help failures, and selected native probes. Re-establish these baselines against the implementation commit: the report does not establish full browser behavior or user-task outcomes. Phase 6's completed documentation work does not close these newly identified gaps.

**Status (2026-09-06, PR [#13](https://github.com/ernsoylu/frees/pull/13) merged `dfa8a7d`).** 9A–9D work is implemented and CI-green (native tests, parity 1308/1308, wasm size, web build, Sonar 0% new-code duplication). The phase is **not closed**: Playwright browser journeys were not added, and the R15 user pilot has not been run (automated tests are not a substitute). Unchecked boxes below are those leftovers, not unimplemented 9A–9D product work.

### Work

Deliver 9A first, then 9B and 9C, then 9D. The unit-generation and validity-warning fixes can land immediately without waiting for shared metadata. Within each stage, land small changes with focused regression coverage.

#### 9A — Correct generated source and component assistance (R1–R5)

- [x] Fix `generateComponentText` at the shared wizard/Compute UA boundary: append suggested units only to plain numeric literals; preserve explicit annotations, variable references, and expressions. Handle signed/scientific literals and offset temperatures according to the existing parser. Validate syntax without treating a legitimately unwired network as an invalid field.
- [x] Extend the existing generation pipeline with structural metadata exported from this port's parsed component library: names, ports, parameter defaults, and variant requirements. Keep authored descriptions/units in Markdown where they are not engine metadata, and retain separate compact completion and rich wizard outputs. Generate reproducibly in a standalone checkout; keep Java inventory comparison separate from browser capability coverage.
- [x] Restore the missing component pages/catalog entries and enforce exact name-set agreement in CI, rather than hard-coding the historical count of 312. Document the metadata regeneration command and detect stale generated output.
- [x] Resolve component signature help from `Type Instance(...)`, including multiline calls and reordered named arguments. Respect nested calls, strings, and comments. Correct keyword/comment highlighting using the existing lexer contract; add richer completion in 9C.
- [x] Use the declared `model$` default, including the no-default case, instead of the first documented variant. Explain active requirements, retain inactive form drafts visibly, and omit them from new generated code. Add a nonblocking engine advisory for explicitly supplied inactive inputs without rewriting existing source.
- [x] Suggest unused instance names and reject case-insensitive collisions in the relevant scope. Surface parser errors beside fields, preserve custom string/table names, and offer searchable fluid suggestions with valid string quoting. Retain the text preview and explicit physical inputs.
- [x] Correct contradictory guide claims about variant rejection, defaults, fluid selection, and inventory as each fix lands.

Primary files: [text generator](web/src/componentText.ts), [wizard](web/src/ComponentWizardModal.tsx), [editor](web/src/EquationEditor.tsx), [docs compiler](web/scripts/compile-docs.js), [manifest generator](web/scripts/build-doc-manifest.mjs), [library](crates/frees-core/src/components/library.rs), [variant resolution](crates/frees-core/src/components/variant.rs), [expander](crates/frees-core/src/components/expander.rs).

#### 9B — Make connections and diagnostic recovery reliable (R6, R7, R10, R11)

- [x] Preview endpoint compatibility from existing domain metadata and use Check for authoritative connection validation. Detect duplicates by connection-set membership, preserve legal branches/loops and custom connectors, and distinguish pending validation from success. Apply generated connections as one undoable edit and discard validation results for obsolete revisions.
- [x] Add accessible node/port names, keyboard focus, Enter/Space activation, Escape cancellation, visible focus, and larger touch hit areas. Reuse the diagram or add a compact endpoint-selector alternative; provide non-dragging repositioning for manual layout and restore focus after actions.
- [x] Distinguish syntax, structural solvability, unit warnings, and numerical convergence in existing status/results and exported reports. Keep warnings tied to the current revision; an unknown unit must not imply verified SI conversion. Preserve the legacy unknown-unit policy unless a separate compatibility decision changes it.
- [x] Extend existing diagnostics with component/circuit grouping, schematic highlighting, and navigation to relevant source. Explain missing or duplicated boundaries from available evidence; never insert guessed physical values. Describe worker failures as local engine failures and give a distinct recovery action.

Primary files: [schematic](web/src/schematic/SchematicTab.tsx), [domain rules](crates/frees-core/src/components/domains.rs), [App](web/src/App.tsx), [diagnostics](web/src/SolveDiagnostics.tsx), [revision tracking](web/src/modelRevision.ts), [engine](crates/frees-core/src/engine.rs).

#### 9C — Share public identity and source locations (R8, R9, remaining R3)

- [x] Extend GUESS/bounds parsing and frontend serialization to existing public member paths such as `HX.in.P`. Resolve through the equation expansion mapping so text guesses and Variable Information address the same scalar; preserve scalar syntax and keep internal mangled names out of authored source.
- [x] Carry scoped instance identity, declaration spans, and definition provenance through Check/WASM metadata. Replace schematic declaration scanning incrementally; nested instances with repeated child names must navigate to the correct scope. Reuse Phase 1 revision handling for metadata freshness.
- [x] Use that metadata for active-parameter and port/member completion, inserting only the missing path segment. Include local component schemas in help and completion, identify local versus standard-library definitions, and advise on shadowing without forbidding it.
- [x] Explain case-only naming conflicts with both source locations when they contribute to a conflict; do not warn on harmless case-insensitive references.

Primary files: [top-level parser](crates/frees-core/src/parser/toplevel.rs), [expander](crates/frees-core/src/components/expander.rs), [WASM boundary](crates/frees/src/lib.rs), [guess helpers](web/src/guessDirectives.ts), [declaration lookup](web/src/schematic/declaration.ts), [editor](web/src/EquationEditor.tsx).

#### 9D — Clarify modeling choices, ownership, and learning (R12–R15)

- [x] Group related devices in existing discovery surfaces by required data, ports, flow/energy closure, steady/transient support, and assumptions. Start with Fan/FanCurve/FanMap and compressor/pump alternatives; use search tags before runtime aliases or type renaming.
- [x] Teach named parameters, explicit `connect`, and single-quoted string literals first; retain positional streams as an advanced alternative. Explain reuse constructs, bounds versus units, `time` versus temperature names, and absolute temperatures versus differences.
- [x] Show effective input origin and precedence for text/GUI tables, guesses, sliders, and REPL overrides. Distinguish Save project from Export equation text and identify required project-only inputs. Preserve map data, layout, plots, and opaque legacy project slices on reload.
- [x] Curate three existing-example journeys: a scalar equation with units, a component chain, and a transient or map-driven upgrade. Include expected results, one deliberate error and recovery per journey, then explicit mixer and closed-loop examples. Link shared reference material instead of duplicating parameter tables.
- [ ] Measure baseline and post-change pilot tasks from review section 3/R15, recording participant experience, unaided completion, time, errors, and recovery. Targets: 4/5 users solve the scalar within 5 minutes, build the chain within 10, and recover a missing boundary within 3; no silent wrong variant, lost map/override, or pointer-only required action. Record unmet targets as follow-up work; do not claim user validation from automated tests. **Follow-up:** no pilot has been run; Vitest/CI are not a substitute.

Primary files: [wizard](web/src/ComponentWizardModal.tsx), [language guide](web/src/docs/language_fundamentals.md), [programming guide](web/src/docs/programming_logic.md), [component guide](web/src/docs/components.md), [App](web/src/App.tsx), [project schema](web/src/project.ts).

### Acceptance criteria

- [x] Wizard/Compute UA regression cases pass through the real WASM parser/Check: numeric, signed/scientific, offset-temperature, explicitly annotated, variable-backed, arithmetic, and custom-string inputs preserve intended values and unknowns. In particular, `UA=conductance` adds neither `w` nor `k`; malformed fields do not enable a misleading valid insertion.
- [x] Engine/catalog/completion inventories agree in a standalone checkout. Non-first defaults, absent defaults, inactive-parameter advisories, local definitions, multiline signatures, named-argument selection, and case-insensitive name collisions have focused checks.
- [ ] Browser journeys verify incompatible and duplicate wiring, valid branches/loops, one-step Undo, keyboard/touch operation, zoom/narrow layouts, cancellation, focus restoration, and revision-safe validation. Unknown units, structural errors, convergence failure, and worker failure retain distinct messages and source/recovery actions. **Open:** Vitest covers preview/duplicates and status wording; Playwright e2e for undo/keyboard/touch/zoom/focus was not added in PR #13.
- [x] Member guesses and bounds agree with Variable Information after text/project round trips; hierarchical navigation uses the correct declaration. Conflicting text/GUI inputs expose precedence, and project reopen preserves effective settings, maps, and legacy payloads.
- [ ] Extend existing generator, guess, project, revision, and WASM-boundary tests plus Playwright journeys. Run `npm test`, `npm run check-docs`, `npm run lint`, and `npm run build` in `web`, along with affected browser/offline gates. For parser/expander changes, run relevant Rust regressions, native parity and fresh-WASM replay under existing tolerances; leave frozen oracle inputs unchanged. **Partial:** PR #13 CI ran native tests, parity 1308/1308, wasm size, and web build; `check-docs` and focused Vitest ran locally. Full `npm test`/`lint` and extra Playwright journeys were not a separate local gate.
- [ ] Representative generated models, custom definitions, variants, maps, and steady/transient examples pass browser/WASM checks. Record pilot baseline/comparison and outstanding usability failures before closing the phase. **Open:** R15 pilot baseline has not been recorded.

Scope limits: no language server, replacement editor/parser framework, new wiring syntax, whole-document formatter, automatic boundary values, or physics-model mergers. Any later merger must satisfy the review's port, equation, table-arity, dynamic-state, and compatibility checklist. Reuse existing metadata, request handling, diagnostics, and tests; add only the missing fields and behavior.

## Phase 10 — Trustworthy tables and plots

Sources: [plot engine review](docs/plot-engine-review-2026-09-06.md) (F1–F12) and [table engine review](docs/table-engine-review-2026-09-06.md) (T1–T15), both dated 2026-09-06. Retain Plotly, Glide Data Grid, the worker boundary, and prepared solving. Repair numerical fidelity and result ownership before expanding chart types or table operations. Phase 1's completion does not close the newly identified table mutation/check gaps.

The reviews record 13 plot frontend checks, 114 table frontend checks, and 40 native boundary checks, plus temporary diagnostics that reproduced defects and were removed. These are historical evidence, not new regression coverage or a complete browser audit. The table conversion timing was collected during a concurrent build; the Plotly size came from an existing artifact. Reproduce affected behavior and establish clean browser baselines before making performance claims.

### Delivery order

| Stage | Deliverable | Dependency | Relative effort |
| --- | --- | --- | --- |
| 10A | Preserve values and repair broken plot paths | Existing test infrastructure | Medium |
| 10B | Stable sources, atomic edits, dependable checks and run status | 10A precision contract; localized fixes may land alongside 10A | Medium–large |
| 10C | Validated imports, units, persistence and reproducible presentation | 10A–10B shared data and revision contracts | Medium–large |
| 10D | Inspection, row operations and accessible engineering workflows | 10B–10C | Medium |
| 10E | Measured preparation, rendering and persistence improvements | Correctness gates; baseline measurements may start immediately | Medium |

Prioritize 10A and 10B alongside Phase 9 correctness work; neither waits for completion of Phase 9 discovery features. First release gate: full-precision data, valid log filling, working histogram/array plots, no spurious thermo requests, stable plot sources, honest code ownership, atomic edits/undo, stale-response rejection, and explicit sweep convergence. Ship small fixes as their gates pass.

### Work

#### 10A — Preserve numerical data and fix existing plot paths (T1, T6; F2–F5)

- [x] Replace six-significant-digit storage in code/function/trajectory/digitizer adapters and fill operations with full-precision numeric strings; format only for display. Build exact-X per-curve indexes once, preserving the established exact-duplicate policy. Verify copy, export, editable duplication, plots, and function conversion retain original values.
- [x] Validate log domains before filling missing cells; identify invalid rows/columns instead of inventing finite replacements. Define duplicate-X, endpoint and out-of-range behavior; preserve precision and record interpolated cells. Compare valid linear/log cases with core interpolation under explicit tolerances.
- [x] Preserve original row/array indices and gaps for ordered plot lines, including failed solver rows. Keep raw-input plotting distinct from solved-output plotting; show valid/skipped counts for filtered chart types. Preserve cyclic traversal order.
- [x] Make channel requirements type-specific: histogram needs samples without X; mesh needs X/Y/Z. Align array X/Y/Z/size by original index, validate bubble sizes and degenerate geometry, and label the current mesh as triangulated. Check equivalent table/array datasets.
- [x] Restrict property/psychrometric requests and fluid loading to applicable plot kinds. Every control kind must issue zero thermodynamic requests.

Primary files: [table adapters](web/src/tables.ts), [read-only grid](web/src/DataGridReadOnly.tsx), [plot preparation](web/src/plots/PlotCard.tsx), [figure builders](web/src/plots/figure.ts), [core interpolation](crates/frees-core/src/curvetable.rs).

#### 10B — Own sources, mutations and results explicitly (T2–T5, T9, T13–T15; F1, F6)

- [x] Route user table mutations through an App-owned revision/invalidation handler, including imports, lookup metadata, external fill/configure and deletion. Flush pending cell edits before Check/Solve, capture the committed request snapshot, and gate both responses by revision and stable table/row IDs. Separate solver writebacks from user mutations; lookup changes invalidate dependent model results.
- [x] Classify bulk edits against the original input/result snapshot, then commit once and invalidate once. Document whether a gesture freezes computed values. Scope undo to user-owned changes or reset incompatible history; preserve unrelated metadata, clear redo on new mutations, never revive obsolete results, and make table deletion recoverable.
- [ ] Remap inert formula overlays through sort/delete/append/duplicate/curve changes, or preserve them as explicitly detached legacy data. Never attach a removed cell's formula to new data; retain D10 project compatibility without restoring formula evaluation.
- [ ] Carry accessor convergence, pass count and termination reason from core through WASM/API/UI. Distinguish row success from table convergence, with provisional last-pass values. Expose row errors and completed/failed/not-run/stale/cancelled states; label Check Table as a structural representative check.
- [ ] Preserve available completed rows on deadline and distinguish worker Stop from numerical failure. Worker termination cannot recover rows not yet delivered: mark those unknown/not-run, and add progress delivery only if needed for partial-result retention. Accessor partial results remain provisional; do not retry isolated accessor rows as independent solves.
- [ ] Persist explicit plot source identity (table or solved arrays), with result revision and stable row identity for downstream actions. Capture the source when plotting selected columns; switching active tables must not rebind it. Missing/deleted/ambiguous legacy sources require explicit selection. Migrate only unambiguous saved bindings.
- [ ] Badge code-owned plots and offer declaration navigation or duplicate-as-editable; remove actions whose effects are discarded. Validate case-insensitive name collisions and preserve ownership across reopen.

Primary files: [App](web/src/App.tsx), [revision tracker](web/src/modelRevision.ts), [workbook](web/src/tablesGrid/TablesGridTab.tsx), [edit model](web/src/tablesGrid/tableGridModel.ts), [API](web/src/api.ts), [table boundary](crates/frees/src/analysis.rs), [sweep](crates/frees-core/src/analysis/parametric.rs), [plot model](web/src/plots/types.ts), [project](web/src/project.ts).

#### 10C — Make conversion and presentation reproducible (T7–T8, T10–T12, T15; F7–F12)

- [ ] Keep invalid numeric drafts editable but block affected solves/conversions with cell locations. Blank outputs remain distinct from invalid inputs. Share name, ID, shape and capacity validation across create/import/paste/duplicate/rename/project load; use unique copy names and a total-cell/byte budget alongside existing row/column caps. Preserve recoverable malformed project content.
- [ ] Extend the existing CSV modal with raw/parsed preview and delimiter/header/decimal/unit-row choices. Resolve names against the entire occupied namespace; report malformed quoting and rejected source rows. Cover numeric family headers, suffixed duplicates, ragged records and multiline fields.
- [ ] Require an explicit choice before lossy lookup reduction: trim, cancel or accept a previewed approximation. Reconcile invalid/duplicate/reduced/retained counts; retain source and transformation settings when feasible. Choose raw input pairs or successful solved rows explicitly. Keep the 5,000-row cap; render decimation is not solver-data simplification.
- [ ] Define canonical values and column/argument/output units across table pins, results, function DTOs, composition and persistence. Reuse core unit metadata and migrate legacy data without assuming unknown units are SI. Verify pressure and offset-temperature round trips through function evaluation before enabling conversions.
- [ ] Separate exact-data copy/CSV from formatted-report export, quote CR as well as LF, and state units, source revision, status and selected/visible/all scope. Distinguish input-only editable copies from snapshots that freeze outputs. Document one persistence rule for inputs, derived results and view state, with revision verification on reopen.
- [ ] Validate plot declaration attributes/kinds/booleans and unsupported slices with source diagnostics; immediately document current whole-array semantics. Any slice implementation needs an explicit compatibility decision and tests, not edited oracle fixtures.
- [ ] Apply units, log/range/tick controls only where supported across XY, dual axes, controls and 3D. Keep unknown units unknown; reject nonpositive log bounds and incompatible display conversions. Screen, hover and export must use the same values/units. Hide ineffective controls until implemented.
- [ ] Render backend critical markers and bind thermo paths to the selected circuit/fluid. Missing state sources and ambiguous paths must not fall back silently; label straight state connections as schematic.
- [ ] Use stable trace identity and Plotly view persistence for unchanged sources/units; provide Fit data and current/full-view export with size/background choices. Surface asynchronous import/render failures with Retry, preserve configuration on context loss, and verify cleanup during resize/unmount.
- [ ] Correct unsupported formatted-report graph claims. Keep help aligned with shipped controls and explain raster content in 3D SVG exports; selected-plot report inclusion can follow through the existing image exporter.

Primary files: [CSV](web/src/tablesGrid/csv.ts), [import modal](web/src/tablesGrid/ImportCsvModal.tsx), [composition](web/src/tablesGrid/composeTables.ts), [function dialog](web/src/tablesGrid/CreateFunctionModal.tsx), [function boundary](crates/frees/src/lib.rs), [code plots](web/src/plots/fromCode.ts), [plot config](web/src/plots/PlotConfigModal.tsx), [units](web/src/plots/units.ts), [renderer](web/src/plots/PlotlyChart.tsx), [export](web/src/plots/exportPlot.ts), [report](web/src/report.ts).

#### 10D — Make engineering inspection practical

- [ ] Add table role/source/status labels, visible Undo/Redo, searchable column configuration, frozen Run/time/X columns and selected-cell/row details. Use existing grid capabilities; view sorting/filtering/reordering must not alter solver row/column order or the run set.
- [ ] Add selected-row insert/delete/duplicate and explicit independent-row retry with unchanged revisions. Accessor models require whole-table recomputation. Expose failure inputs and diagnostics so a run can be reproduced through the existing calculation/pin path.
- [ ] Offer Plot selected columns from editable and derived tables, then link a persistent plot cursor to the original row. Add keyboard sample inspection, accessible raw-data alternatives and selection summaries with scope/units; calculate statistics from raw data. Add a second cursor with deltas/slope after the first workflow passes.
- [ ] Add per-trace line/marker styles and reference annotations through existing Plotly controls. Test keyboard navigation, textual failure indicators, focus, touch controls and 360-pixel dock layouts across tables and plots.

#### 10E — Optimize only measured costs

- [ ] Establish release-browser baselines for editable 100×10/1,000×30/5,000×100 tables, 10k/100k derived rows, 100/1k/5k lookup knots, 1/10/64 MiB CSV imports, and independent/accessor sweeps. For plots use 1k×3 and 10k/100k×8 traces with spikes/gaps, 1/4/12 dock windows, repeated thermo charts, equivalent 3D sources and export. Treat million-point plots as stress characterization, not a capacity promise.
- [ ] Separate preparation, worker compute, serialization/parse, materialization, rendering and persistence; record median/p95, long tasks, peak/retained memory, machine/browser/build/viewport and cold/warm state. Rebuild bundle evidence cleanly. Ratify targets after baseline: candidate cell commit <50 ms p95, 1,000-cell edit <200 ms, everyday warm plot <200 ms p95, cursor <50 ms p95, dense pan/zoom 30 fps, and usable 10 MiB import preview within one second or cancellable progress.
- [ ] Measure exact-X indexing and atomic edit gains from 10A–10B. Stabilize unchanged figure inputs and array-channel indexing; avoid repeated conversion on metadata edits. Coalesce persistence without weakening durability; only add import workers/chunking or compact undo storage when measured costs justify them.
- [ ] If duplicate thermo generation is material, add bounded response/in-flight reuse keyed by fluid, kind, engine/backend revision and numerical settings; exclude presentation fields and evict failed requests for Retry. Validate subsequent solves after cold and cached chart access because skipping property calls changes shared-worker history. Preserve the existing one-slot property cache.
- [ ] Suppress unnecessary dense markers first. Add render-only monotonic-series reduction or lazy WebGL only if SVG remains inadequate; preserve extrema, endpoints, gaps and original samples for inspection/export/statistics. Do not apply time-series reduction to unordered scatter or cyclic paths.
- [ ] Profile hidden-plot updates, core lookup preparation, large-copy selection and image-export memory before changing them. Split 3D loading or change transport/workers only with measured benefit; lazy splitting does not reduce total precached offline bytes. Verify offline 2D/3D and the existing WASM budget after affected changes.

### Acceptance criteria

- [ ] Distinct knots `1.000001`/`1.000002` and high-precision trajectories survive copy/export/function conversion; display changes do not change numerical inputs. Invalid log fill fails clearly; valid fill matches core semantics.
- [ ] A new Y-only histogram renders; nondegenerate array/table meshes and bubble sizes agree; missing indices/failed rows remain identifiable gaps; control plots send zero thermo requests.
- [ ] Delayed Check/Solve after edits, lookup changes, row deletion or project replacement cannot overwrite current state. Cell commit → Solve uses the committed value. Reordered bulk batches agree; edit → rename → Undo preserves the rename and never restores stale results; structural changes preserve legacy associations.
- [ ] `y = TableAvg('y') + 1` is visibly nonconverged at the pass cap, while a convergent accessor case succeeds. Deadlines/Stop distinguish retained completed rows, provisional accessor data and unavailable rows without inventing successful partial output.
- [ ] Table switch/delete/reopen preserves or explicitly requests plot binding. CSV edge cases and numerical reduction have inspectable outcomes and reconciled counts. Non-SI pressure/temperature function round trips preserve physical meaning.
- [ ] Zoom → re-solve → current-view export preserves the intended view for unchanged sources/units. Circuit selection never overlays another fluid's path. Import/render errors support Retry; keyboard point/row inspection and narrow layouts are usable.
- [ ] Leave focused Vitest/Rust regressions with each fix and browser journeys for the cross-layer cases above. Run frontend tests, lint and build; run docs checks when help changes and applicable Playwright/offline checks for rendering/build changes. For DTO, interpolation, unit or thermo-cache changes, run affected native tests and fresh-WASM boundary/parity replay under existing tolerances; preserve frozen fixtures.
- [ ] Record comparable before/after measurements for retained optimizations, including numerical correctness, memory and bundle effects. Review targets are proposals until measured; no speedup or supported capacity is claimed from the old diagnostics.

Scope limits: no replacement renderer/grid, new state manager, generic chart/table plugin framework, restored spreadsheet/analyzer, global numerical cache, or automatic conversion dependency graph. Defer baseline-run comparisons, uncertainty bands, templates, GUI control presets, configurable isolines and heatmaps until 10A–10D establish trustworthy data and inspection. Start those as separate increments with explicit unit/alignment/statistical/domain contracts; keep worker pools and other infrastructure changes conditional under Phase 7.

## Phase 7 — Conditional extensions

These are candidates, not automatic implementation commitments.

| Candidate | Start only when | Constraints and proof |
| --- | --- | --- |
| Small worker pool for independent sweep rows | Serial sweeps remain a significant measured delay after preparation reuse | Bound worker count and memory; keep accessor-dependent sweeps serial; compare results across worker counts |
| Parallel Monte Carlo or Pareto evaluation | Throughput need remains and reproducibility is defined | Preserve random-stream semantics and account for sequential warm starts/property seeds before claiming independence |
| Additional linked fluids | A concrete user model requires them | Verify supported states and accuracy; measure bundle and memory cost |
| Additional CAS patterns or numerical methods | A documented model cannot be handled by existing capabilities | Provide a concrete regression/oracle case and explicit validity limits |
| Apply COLAMD at sparse-LU factor time | Profiles show excessive factorization fill on a matrix family, and the IDA goldens are re-graded or retoleranced on purpose | `dae/colamd.rs` is already unit-tested. Applying `A P` moved `steady-by-integration-chiller-bridge` to ~5e-9 vs its 2.5e-9 band. Measure fill and solve time on the affected family before enabling it. |
| Wire analytic cubic-Hermite event roots in `refine_crossing` | A bounce/`set` document is re-harvested, or the extra-root behaviour is an accepted divergence | `ode/hermite_root.rs` is tested. Production stays on Java's 60-bisection: wiring it gave `av_ev_set_to_expression` 12 hits vs the oracle's 11. |

Do not add a backend, shared-memory threading, a replacement UI framework, a general bytecode VM, or new numerical dependencies as part of the default plan.

## Completion and change control

- [x] Land each change with the smallest meaningful regression check for its behavior.
- [x] Run the relevant tests during development; run applicable format, native/WASM lint, parity, frontend build/test, and browser gates before landing affected changes.
- [x] Leave frozen corpus inputs untouched unless deliberately regenerated through the documented oracle process. Never edit goldens merely to match a new implementation.
- [x] Record before/after workload, build configuration, numerical comparison, time/instruction counts, peak memory, and bundle impact for performance changes.
- [x] Preserve project migration and round-trip guarantees, including data for deliberately removed features.
- [x] Mark phase checkboxes complete only after their acceptance criteria pass. Record completed phases and measured outcomes below.

## Execution log

| Date | Phase | Change / evidence | Status |
| --- | --- | --- | --- |
| 2026-09-05 | Planning | Review findings converted into this phased plan; implementation has not started | Planned |
| 2026-09-05 | 1 | Equation overrides reach Solve and Check (`crates/frees/src/lib.rs`). The DTO had no `overrides` field, so every REPL assignment and every slider drag was parsed and dropped. Both entry points now shadow the source with `analysis::montecarlo::apply_overrides(source, …)` — the existing Java transcription, no second substitution path — before the solve, `fillMissing`, the REPL workspace and the `errorLine` lookup, so none of them can disagree about what was solved. `overrides` is `Option<Vec<String>>` like every other nullable DTO field: `"overrides": null` had meant `Invalid request`. Evidence: `crates/frees/tests/overrides.rs`, 13 tests; `cargo test -p frees` 131 pass; fmt + clippy (native and wasm32) clean; 424 frontend tests and `tsc --noEmit` clean under Node 22. Verified in the **built artifact** as well, not only natively — a Node replay through `web/src/wasm/pkg` returns `x = 3, y = 9`, flips Check on `y = x^2` from unsolvable to solvable, and converts `P = 250 [kPa]` to 250 000 Pa. Bundle: baseline 3117.4 KiB raw / 1271.1 gz → **3118.2 / 1271.9**, delta **+0.8 KiB raw**, 76.1 % of the 4096 KiB budget (both figures measured from wasm-pack release builds of this tree, with and without the change; CLAUDE.md's 3087.8 KiB predates the moist-air component commit and is not the baseline). `frees-core` is untouched, so the parity corpus is unaffected | Done |
| 2026-09-05 | 1 | Two controls stopped advertising behaviour nothing implements. **Change in variables** (`web/src/PreferencesModal.tsx`) is disabled and says so — the engine's per-block stop rule is the relative residual alone; the value stays in `StopCriteria` so saved projects and the request still round-trip. **Find all solutions** (`web/src/WorkspaceChrome.tsx`) is disabled and labelled *Not available in the browser engine yet* — `analysis/allroots.rs` exists but no request reaches it until Phase 6. The `findAll` state and its plumbing are left in place for that wiring | Done |
| 2026-09-06 | 1 | Completed Phase 1 reliability improvements: Model revision tracking with monotonic counter (`web/src/modelRevision.ts`, `web/src/modelRevision.test.ts`), in-flight request revision gating in `App.tsx` (dropping stale solves/checks on document edits, scheduling recheck when idle), Stop action with graceful worker lifecycle (`wasmStop()`, `engine.worker.ts`, `engineClient.ts`, `web/src/requestCoordination.test.ts`, `WorkspaceChrome.tsx` Stop button & Escape shortcut) distinguishing fatal traps vs recoverable errors, and operation-wide elapsed-time deadline checks in `newton.rs`, `engine.rs`, and `analysis.rs` with `crates/frees/tests/budgets.rs`. All acceptance criteria met. | Done |
| 2026-09-06 | 2 | Cleaned unused code and configuration: Removed unused remote submit/SSE/poll adapter (`runCompute`, `pollJob`, `JobState`, `ComputeOutcome`, `API_BASE`) from `web/src/api.ts` and deleted its test suite `web/src/api.async.test.ts`; removed unused `exportVector` rejection stub from `api.ts` (keeping client-side SVG/PNG/JPG exports in `exportPlot.ts`); removed unused `serde-wasm-bindgen` dependency from `crates/frees/Cargo.toml`, unused workspace `thiserror` dependency from root `Cargo.toml`, and obsolete Excalidraw package overrides from `web/package.json` (pruning 7 crates from `Cargo.lock`); removed `require_units!` panic-swallowing bypass and `units_ready` from `crates/frees-core/src/parser/expr.rs`; updated superseded comments on PID tuning and plant extraction. Net reduction: 428 lines of dead code and 7 dependencies removed. All 1,308 core parity tests, 127 WASM tests, 429 vitest tests, type checks, linter, and production build pass cleanly. | Done |
| 2026-09-06 | 3 | Unified single-objective constrained candidate solves (`crates/frees-core/src/analysis/optimizer.rs`): Appends `<prefix><i> = <c.lhs_expr>` constraint equations alongside decisions in a single pass (`solve_candidate`), adapting the Pareto pattern (`pareto.rs`). Solves the objective and all $C$ constraints simultaneously in one Tarjan pass instead of $(1 + C)$ full system solves per Nelder-Mead candidate probe. Avoids repeated final constraint solves by reading constraint values directly from candidate solutions in `update_and_check_constraints` and `build_constraint_warning`, stripping temporary prefix variables (`strip_constraint_variables`) from `solution.values`, `display_names`, `inferred_units`, and `residuals` before returning. Added regression and benchmark test suite (`crates/frees-core/tests/constrained_optimizer.rs`). On a 3-decision, 3-constraint workload (420 candidate evaluations), model solves reduced by ~4× (from ~1,680+ down to 421 solves) with sub-second execution (967ms). Preserved 100% oracle parity on all test suites: 1,309 core tests, 127 WASM tests, 429 vitest tests, cargo fmt, and cargo clippy pass cleanly. | Done |
| 2026-09-06 | 4 | Extracted reusable `PreparedDocument` (`crates/frees-core/src/engine/prepared.rs`) hoisting 13-stage document compilation (parsing, component expansion, CALL flattening, string variable resolution, integral hoisting, complex/ODE expansion, block decomposition, analytical Jacobian differentiation, dense plan construction, variable specs, and scratch scopes). Enabled numeric pin updates in-place without rebuilding AST or Jacobian derivatives; rebuilds only on structural changes (source, definitions, complex mode, or pin set). Integrated preparation reuse into parametric sweeps (`solve_table` in `crates/frees/src/analysis.rs` & `RowJob` in `crates/frees-core/src/analysis/parametric.rs`), Nelder-Mead optimization (`crates/frees-core/src/analysis/optimizer.rs` evaluating 420 probes with 1 compilation), REPL session definitions caching (`crates/frees/src/repl.rs`), and Monte Carlo base solve deduplication (`crates/frees-core/src/analysis/montecarlo.rs`). Added dedicated regression and benchmark test suite (`crates/frees-core/tests/prepared_solver.rs`): measured **18.74× speedup** on 50 repeated parametric solves (22.3ms vs 418.1ms), ~4× speedup on constrained optimization (410ms vs 1,617ms for 420 evaluations), verified structural rebuild triggers and complex mode isolation. All 2,600+ tests pass including all 1,308 golden corpus fixtures in `parity.rs` with 100% bit/oracle parity, cargo clippy clean, cargo fmt clean, 429 vitest tests clean. | Done |
| 2026-09-06 | 5 | Reduced numerical memory and measured overhead: Direct CSC sparse Jacobian evaluation (`crates/frees-core/src/dae/jacobian.rs` and `solver.rs`) writing directly into sparse CSC storage buffers, completely eliminating the $N \times N$ dense intermediate allocation and matrix clone during sparse DAE `lsetup`. Reusable DAE Jacobian scratch storage (`DaeJacobianScratch`) reusing perturbation and residual vectors in place. Settings validation and bounds unpacking hoisted and cached in `NewtonWorkspace` (`crates/frees-core/src/solver/newton.rs` and `engine.rs`), eliminating redundant loop validation across prepared Newton solver calls. Configured speed-oriented profiles `[profile.bench]` and `[profile.release-speed]` (`opt-level = 3`, `codegen-units = 1`, `lto = true`) in root `Cargo.toml` for native throughput while preserving the size-first release profile for WASM. Optimized large-result serialization in `crates/frees/src/analysis.rs` (eliminating redundant `variable_entries` DTO allocations per row in parametric sweeps, precomputing variable mapping in Monte Carlo sampling) and in `crates/frees/src/lib.rs` (direct `Number::from_f64` cell construction in ODE tables). Added `crates/frees-core/tests/dae_sparse_direct.rs` verifying bitwise bit-exact parity between CSC and dense colored Jacobian paths, stiff network equivalence, and large-scale $N=200$ sparse DAE integration without $N \times N$ buffers. WASM bundle measured at 3,136 KiB raw (1,282 KiB gzipped), well within the 4,096 KiB budget (76.6%). All workspace tests, fmt, clippy, and 429 vitest tests pass cleanly. | Done |
| 2026-09-06 | 6 | Completed capabilities and made usage trustworthy: Wired all-roots solver through `PreparedDocument::solve_all`/`solve_all_with_pins`, WASM request boundary (`findAllSolutions` on `SolveRequest` and `solutions` list in `SolveResponse`), enabled UI checkbox in `WorkspaceChrome.tsx`, and added multi-solution navigation controls (`< 1 of N >`) in `Workspace.tsx` and `App.tsx` (bounded contract, max 32 roots). Added `crates/frees-core/tests/allroots_integration.rs` covering multi-variable systems with pins. Threaded imported function tables through single-objective optimization (`optimizer.rs`) and multi-objective Pareto optimization (`pareto.rs`), exposing `functionTables` across `web/src/MinMaxModal.tsx`, `api.ts`, and `crates/frees/src/analysis.rs` with coverage in `crates/frees/tests/function_tables.rs`. Corrected language fundamentals documentation (valid `Convert(Pa, kPa)` syntax, eliminated double conversions, noted symbolic AD and finite-difference fallbacks) and replaced obsolete Spring Boot / RabbitMQ / Redis text in architecture and deployment guides with Web Worker and headless CLI workflows. Added `--request` / `-r` JSON/file flag to `frees-cli`. WASM bundle: 3,250 KiB raw (1,331 KiB gzipped, well under 4,096 KiB budget). Passed all 1,308 golden parity tests (100% bit-exact), clippy, fmt, 429 vitest tests, and offline PWA Playwright tests. PR #10 merged into `main`. | Done |
| 2026-09-06 | 0 | Established browser correctness and performance gates: Built fresh WASM artifact and added real-boundary regression suite (`web/src/wasmBoundary.test.ts`) testing overrides, multiple roots, stopping controls, function tables, and error handling directly against `frees_bg.wasm`. Created Node-based golden corpus replay harness (`web/scripts/wasm-parity.mjs`) carrying `.request.json` and `.tables.json` sidecars across all 1,308 fixtures with perturbation detection. Confirmed 100% parity across all 4 shards (0/4, 1/4, 2/4, 3/4) with zero variable differences; isolated and documented the single browser divergence in `components_i_cabinzone_dynamic` (6.66e-9 max diff due to wasm32 libm vs glibc transcendentals in `rustprop`) under tight bound (1e-8) in `fixtures/browser-divergences.json`. Extended native Criterion benchmarks (`crates/frees-core/benches/solve_bench.rs`) and browser Playwright benchmarks (`web/bench/wasm-bench.spec.ts`) with constrained optimization, 1,000-row sweep, component network (`ev-thermal-management`), stiff transient (`pressure-cooker`), large sparse DAE step, and cold initialization vs warm execution. Wired WASM replay and perturbation gates into GitHub Actions CI (`.github/workflows/ci.yml`). PR #11 merged to `main`. | Done |
| 2026-09-06 | 8 | Flattened Newton linear-algebra workspaces (`lin_scaled` / `jtj` / `damped_a`) to row-major `Vec<f64>` and added 2-row tiled elimination for `N > 16` (`newton.rs`). The analytic Jacobian itself stays `Vec<Vec<f64>>` — it is filled row-wise by the differentiator. Tiled GE covered by `dense_tiled_linear_and_nonlinear_systems_converge_for_large_n`. Sparse LU factor scratch (`x`, `pinv`, `mark`, `stack`, `pstack`, `order`) lives on `SparseLuWorkspace`; IDA `lsolve` writes into a reused `lu_sol`. IDA `RATEMAX = 0.9` already aborts a diverging corrector at iteration 2. Optional 5th-order Radau IIA (`ode/radau.rs`, `method = radau\|radau5\|radauiia`) is an `OdeMethod` for `y' = f`, not an IDA replacement. **Closed without** applying COLAMD or wiring analytic event roots: those helpers stay tested and move to Phase 7 (`A P` moved the chiller-bridge IDA golden; analytic bounce crossings added a 12th hit). Speed targets (30–50% on N=30–100, stiff transient 2.5s→1.5s, sub-µs N≤10) were planning estimates, not measured, and are not claimed. Evidence: `cargo test -p frees-core --lib` 2401 pass; clippy native + wasm32 `-D warnings` clean; `cargo fmt --all --check` clean; **release parity 1308/1308**; CI wasm size gate green on PR #12. | Done |
| 2026-09-06 | 9 planning | Added staged implementation work and acceptance gates for syntax/component UX review R1–R15; implementation and browser/user validation remain pending. | Planned |
| 2026-09-06 | 10 planning | Combined plot F1–F12 and table T1–T15 reviews into five stages: fidelity, source/result ownership, conversion/presentation, inspection, and measured performance. Spot-checked current adapters, plot preparation, App mutations and table boundary at `33ea6db`; implementation and acceptance checks remain pending. Updated plan status to reflect existing completion records. | Planned |
| 2026-09-06 | 9A | Corrected generated source and component assistance (R1–R5). Units only on plain numerics; 312 engine = catalog = pages; signature help for `Type Instance(`; `model$` default; unique names; inactive-parameter advisory. | Done |
| 2026-09-06 | 9B | Schematic wiring preview + Check topology, keyboard/touch access, validity-level status, local engine-failure wording. | Done |
| 2026-09-06 | 9C | `GUESS HX.in.P` maps to the expanded scalar; Check instance/definition identity; local completion; shadowing and `T`/`t` advisories. | Done |
| 2026-09-06 | 9D | Fan/compressor/pump related-model table; named-parameter/`connect`/quotes; Save project vs Export equation text; three journeys. **R15 pilot is follow-up.** PR [#13](https://github.com/ernsoylu/frees/pull/13) merged `dfa8a7d`. CI: native, parity 1308/1308, wasm size, web build, Sonar 0% new-code duplication. Phase 9 **not closed** (Playwright journeys + pilot). | Done |
