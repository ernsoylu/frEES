# Status — Phase 4 complete

**Date:** 2026-07-30 · Supersedes `docs/status-phase4-partial.md` (deleted).

Phase 4 ports the parts of the Java engine that turn a scalar equation solver
into the frees engine: symbolic differentiation, arrays and matrices, complex
expansion, procedural bodies, code tables, integrals, the dense linear-algebra /
signal / statistics kernels, LaTeX rendering, and the Java solve **retry
ladder**. All of it is wired into the solve path and replayed against the real
Java engine.

```
1341 Rust tests passed, 0 failed, 1 ignored (17 suites)
204/204 golden fixtures match the Java oracle
clippy -D warnings clean (host and wasm32-unknown-unknown)   cargo fmt clean
wasm 1147.7 KiB raw / 436.1 KiB gzipped   (budget 2048 KiB raw — 56% used)
web 328 tests / 34 files green, vite build green
Browser proof: matrix + FUNCTION + TABLE solved in-tab, ZERO /api/ requests
```

Every number above was measured in this pass, raw, not carried forward. See
[Gate evidence](#gate-evidence).

---

## What Phase 4 delivers, by area

| Area | Module(s) | What works | Java oracle it was checked against |
|---|---|---|---|
| **Differentiator** | `differentiator.rs` (907 ln) | Symbolic `d/dx` over the whole expression grammar; feeds the **analytic Jacobian** (`engine.rs` pre-differentiates each block's dependent (equation, variable) pairs; `newton.rs:294` uses it, all-or-nothing, FD otherwise). `asin`/`acos`/`atan` deliberately return `None`, matching the Java switch, and fall back to FD. | `Differentiator.java`, `analyticalJacobian` |
| **Matrix / arrays** | `parser/expand.rs` (4590 ln), `linalg.rs` (863 ln) | Bare-name matrix creation, row `[1, 2]` vs column `[1; 2]`, element naming `A[i,j]` with source casing, transpose, elementwise ops, cross product, scalar broadcast, explicit-index products, `Inverse`/`Det` aliases, `SolveLinear`, `zeros`/`identity`/`linspace`, `det$<n>` for n > 3 (LU, not cofactor). | `EquationParser.expandExpr`, `buildElementVars`, `parseMatrixInfo` |
| **Complex** | `parser/complex.rs` (1518 ln) | `_r`/`_i` component expansion, display-name propagation (`Zed = 3 + 4i` reports `Zed_r`), complex arithmetic lowered to real pairs. | `ComplexExpansion.java` |
| **Procedural** | `procedures.rs` (1510 ln), `parser/toplevel.rs` | `FUNCTION` / `PROCEDURE` / `MODULE` bodies with `:=`, `IF`, `FOR`, `WHILE`, `REPEAT…UNTIL`; multi-output procedures; MODULE `<name>$<n>$` namespacing and instance numbering; dynamic scoping into the caller; body locals do not leak; formals shadow. Runaway loops hit the Java ceiling instead of hanging. | `ProceduralFeaturesTest.java`, `ProcedureEvaluator` |
| **Tables** | `curvetable.rs` (665 ln), `interp2.rs` (398 ln) | `TABLE` lookup inside Newton blocks, XLOG/YLOG interpolation **in log space**, clamping at both ends, curve-family blending with parameter clamping, descending and duplicate x columns, single-point tables, a table name shadowing a variable, `CALL Interp2` bilinear grids. | `CodeTableTest.java` |
| **Integrals** | `integral.rs` (1588 ln) | `Integral(f, x, a, b)` and `GaussIntegral`, constant and **variable** limits, reversed and degenerate limits, the integration-variable pin, `FUNCTION`/`TABLE` integrands, nested integrals (hoisted), a structural view so `check` sees a well-posed system. Step error reproduces the Java value exactly (`1.9999993342983777` for `sin` over `[0, π]`). | `IntegralSolver` |
| **Kernels** | `linalg.rs`, `signal.rs`, `statistics.rs` | Ten CALL-form intrinsics: `QR`, `Cholesky`, `MatExp`, `SingularValues`, `SVD`, `FFT`, `IFFT`, `Convolve`, `LinFit`, `PolyFit` — line-for-line ports of Java's `LIN_ALG_SIGNAL_STATS_CALLS` half of `flattenCallProc`, including the QR sign convention (Q = R = −I for the identity) and descending singular values. Plus `LUDecompose` and the `det$`/`qr$`/`chol$`/`expm$`/`svd$` synthetic dispatch. | `EquationParser.flattenX`, `flattenCallProc` |
| **LaTeX** | `parser/latex.rs` (1163 ln) | Expression → LaTeX rendering for the editor's formula view. | `LatexRenderer` |
| **Solver hardening** | `solver/newton.rs` (2524 ln), `engine.rs` (2662 ln) | The Java **retry ladder** — rung 1 `retryWithTransformedGuesses`, rung 2 `tryUnivariateBracketingSolve`, rung 3 `tryMergeBidirectional`, capped at `MAX_RETRY_ITERATIONS = 500`. Bounds are now **enforced**, not advisory: candidates are clamped into `[lo, hi]` at all three Java `Math.clamp` sites (Jacobian probe, `backtrackLineSearch`, `dampedRescue`). Partial diagnostics on failure (blocks, residuals at the stalled iterate, stats, `failed_block_index`). | `EquationSystemSolver.solveBlockWithFallback` |
| **Reporting** | `engine.rs` | `display_names` accumulated exactly where Java accumulates it (not reconstructed by a lexer pass). Ignored-output sinks (`~ignored~N`) are hidden from the result map, from `check`'s variable list, and from both reported counts — see [the sink fix](#the-one-engine-fix-this-pass-made). | `EquationSystemSolver`, `ParseResult.displayNames` |

Phase 4's delta on `crates/frees-core/src` (`git diff ad00922` — the Phase-3
commit — including the working tree): **19 files changed, +22,377 / −1,478**,
bringing `crates/frees-core/src` to **38,675 lines**.

---

## Gate evidence

Every gate below was re-run raw in this pass. **Note for the next session:** the
`rtk` output filter rewrites `cargo`/`npx` invocations and *condenses* their
output — `cargo test` came back as a one-line summary with no per-suite results,
and clippy/fmt warnings are swallowed entirely. Invoke the binary by absolute
path (`"$HOME/.cargo/bin/cargo"`, `./node_modules/.bin/vitest`) and redirect to
a file to see real output.

| Gate | Command | Result |
|---|---|---|
| Tests | `cargo test --release --workspace` | **1341 passed, 0 failed, 1 ignored** (17 suites) |
| Parity | `cargo test -p frees-core --test parity` | **204/204 fixtures match the Java oracle** |
| Clippy (host) | `cargo clippy --workspace --all-targets -- -D warnings` | exit **0**, no output |
| Clippy (wasm32) | same, `--target wasm32-unknown-unknown` | exit **0**, no output |
| Format | `cargo fmt --all --check` | exit **0**, no output |
| wasm bundle | `wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg` | **1,175,269 B = 1147.7 KiB raw**, **446,593 B = 436.1 KiB gzipped**. Budget 2048 KiB raw → **56.0% used, 900.3 KiB headroom** |
| Web tests | `cd web && nvm use 22 && npx vitest run` | **328 passed / 34 files**, 0 failed |
| Web build | `npm run build` | exit **0** (only pre-existing rollup `/*#__PURE__*/` and chunk-size warnings from vendored deps) |

The one ignored test is
`robustness.rs::the_slowest_quadrature_inputs_still_terminate`
(`#[ignore = "bounded but slow (~6 min); documents the quadrature step budget"]`) —
see [non-deliveries](#what-phase-4-did-not-deliver).

### Browser proof

`web/dist` served by a static server **with an SPA fallback** (plain
`python3 -m http.server` 404s on `/help`; the fallback script is
`scratchpad/spa_server.py`, and `.wasm` needs `application/wasm`). Driven with
the Playwright MCP tools.

Document set into the CodeMirror view via
`document.querySelector('.cm-content').cmTile.view.dispatch(...)` — note the
property is `cmTile`, **not** `cmView`, in the CodeMirror build this app ships.
It exercises a matrix solve, a user FUNCTION and a TABLE lookup together
(frozen as `fixtures/corpus/browser_matrix_function_table.frees`):

```
TABLE htc(re)          FUNCTION Scale(v)        A = [2 1; 1 3]
  1000   50              Scale := 2 * v + 1     b = [5; 10]
  2000   80            END                      x = SolveLinear(A, b)
  4000   120                                    Re = 3000
END                                             U = htc(Re) · S = Scale(U) · Q = S * x[1]
```

F2 → **Solved**, `12 eqns · 11 blocks · 11 iters · max residual 0`. Variable
Explorer, expanded:

| Feature | Variable | Browser | Java oracle |
|---|---|---|---|
| matrix solve | `x` (2×1) | `[1, 3]` | `x[1]=1.0, x[2]=3.0` |
| matrix input | `A` (2×2) | `[[2,1],[1,3]]` | identical |
| TABLE lookup | `U` | `100` | `100.0` |
| user FUNCTION | `S` | `201` | `201.0` |
| composed | `Q` | `201` | `201.0` |

`browser_network_requests` filtered by `/api/`: **empty, on both the workspace
and the Help page.** The full unfiltered list is the static bundle plus exactly
two engine artefacts — `assets/engine.worker-*.js` and
`assets/frees_bg-*.wasm`. The only non-200s in the whole session were
`/build-info.js` (injected by nginx in the Docker deploy, absent from a bare
`dist`) and `/favicon.ico`.

**Help/reference renders real data.** `/help#ref-units` shows **136 unit
badges across 24 dimension groups** and **23 built-in constants** read live from
the wasm engine's own registries — `pi# = 3.1415927`, `R# = 8.3144626`,
`g# = 9.80665`, `Na#`, `k#`, `h#`, `c#`, `sigma#`, matching the corpus ground
truth. No "backend not reachable" alert, no stuck "Loading reference…". The
`getReference` stub is gone.

---

## Fixtures

```
fixtures/corpus + fixtures/golden   204 promoted (was 161)
fixtures/corpus-pending             36 staged, 0 promotable this pass
```

> **Superseded 2026-07-30 by Phase 5.** The corpus is now **268 promoted / 29
> pending**; the live breakdown is in
> [`docs/status-phase5.md`](status-phase5.md#pending-29-replayed-document-by-document).
> The table below is kept as the Phase-4 record. What changed:
>
> * **Eleven documents promoted out of this table.** The three CoolProp fluid
>   documents `rankine-cycle`, `rankine-cycle-2` and `refrigeration-vcr` now
>   solve against the linked `(P,h)` tables, and all five "property / material
>   kernels" documents (`adiabatic-flame-temp`, `cubic-eos-properties`,
>   `karman-rocket`, `material-conduction`, `multi-objective-beam`) plus three
>   more promoted with the Phase-5 intrinsics. That row is **closed**.
> * **The tolerance question this table raised is answered.** The "no
>   table-backed engine can pass a `1e-9` gate" problem is resolved by
>   `fixtures/tolerances.json` — a per-fixture *relative* tolerance with a
>   recorded measurement, a stated mechanism, and two guards that fail the gate
>   if the entry is stale or unnecessary. Five fixtures use it. See
>   [`docs/status-phase5.md`](status-phase5.md#parity-tolerance--the-gate-change-and-its-guards)
>   and `fixtures/README.md`.
> * **The remaining CoolProp documents moved to a different blocker.**
>   `ev-battery-cooling-pid`, `ev-thermal-management` and `pressure-cooker` are
>   now refused at `COMPONENT instantiation` (Phase 6) rather than at properties;
>   `state-tables-multifluid` at `STATE TABLE`. Only `thermo-compliance` and the
>   new `hx-correlations-fluid` still wait on Phase-5 work, specifically the
>   transport properties (`Viscosity`, `Conductivity`, `Cp`, `Z`) the `(P,h)`
>   tables do not store.

### Promoted this pass (3)

| Fixture | Why it is new |
|---|---|
| `call_linfit_omitted_r2` | Trailing CALL output omitted; the hidden sink must not surface. Unblocked by the engine fix below. |
| `call_linfit_omitted_b_and_r2` | Two outputs omitted at once — checks that the equation *and* variable counts both drop. |
| `browser_matrix_function_table` | The end-to-end document the browser proof drives; matrix + FUNCTION + TABLE in one solve. |

The other 40 `call_*` fixtures in `fixtures/corpus/` were added by the previous
stage when the ten kernel intrinsics landed (161 → 201).

### Pending: all 36 re-checked, none promotable

Every staged document was replayed through the Rust engine with the **exact**
comparison logic of `tests/parity.rs` (a throwaway copy of the harness pointed
at `corpus-pending/golden`, then deleted). The parity test was not weakened.

| # | Group | Blocked on | Documents |
|---|---|---|---|
| 8 | ~~**CoolProp-poisoned — no usable golden**~~ **De-poisoned 2026-07-30; now blocked on engine features only.** ~~**Partly closed 2026-07-30 (Phase 5)**~~ — three promoted, four moved to Phase 6/7/8 blockers, one still on Phase 5 | `tools/golden-dumper/run.sh` now exports `COOLPROP_LIBRARY` itself, and all eight goldens were regenerated against CoolProp 8.0.0 — **all eight now carry real values** (`rankine-cycle` `eta_th = 0.39119716208990235`, `refrigeration-vcr` `COP = 3.223576728376346`, …), and the only files that changed were those eight. **Phase 5 promoted `rankine-cycle`, `rankine-cycle-2` and `refrigeration-vcr`** against the linked `(P,h)` tables (worst 6.4e-07, 6.4e-07, 1.5e-06 vs. the oracle) at a declared tolerance. `thermo-compliance` still needs `CompressibilityFactor` and transport properties; the other four are now refused at `COMPONENT`/`STATE TABLE`. | `ev-battery-cooling-pid`, `ev-thermal-management`, `pressure-cooker`, ~~`rankine-cycle`~~, ~~`rankine-cycle-2`~~, ~~`refrigeration-vcr`~~, `state-tables-multifluid`, `thermo-compliance` |
| 6 | **Phase-9 control-systems CALLs** | `lqr`, `lqe`, `c2d`, `routh`, `residue`, `tf2ss` — refused by name from `UNPORTED_CALL_INTRINSICS` | `controller-design-lqr-pid`, `estimator-gramian-balreal`, `digital-control-c2d`, `routh-stability`, `inverse-laplace-residue`, `multi-output-destructuring` |
| 5 | **PLOT blocks** | Block type the engine refuses (Phase 7) | `control-analysis-report`, `cruise-control`, `nichols-chart`, `root-locus-analysis`, `step-impulse-response` |
| 5 | **DYNAMIC (ODE/DAE) blocks** | Block type the engine refuses (Phase 8) | `damped-oscillator-ode`, `engine-cycle-wiebe`, `newton-cooling-transient`, `sounding-rocket-trajectory`, `transient-heat-rod` |
| 5 | ~~**Property / material kernels**~~ **Closed 2026-07-30 (Phase 5)** — all five promoted | `AdiabaticFlameTemp`, `eos_z`, `MolarMass`, `k_()`, `E_()` all landed in `props/`; every one of these five is now in `fixtures/corpus/` | ~~`adiabatic-flame-temp`~~, ~~`cubic-eos-properties`~~, ~~`karman-rocket`~~, ~~`material-conduction`~~, ~~`multi-objective-beam`~~ |
| 3 | **PARAMETRIC blocks** | Error fixtures where the classifications still disagree: Java raises `SolverException` (underspecified when solved directly, since these are swept from the Tables tab); Rust raises `ParseException` (block type unsupported). Both refuse — but the gate compares classification. | `damped-oscillator`, `driving-cycle-energy`, `projectile-trajectory` |
| 1 | **SYMBOLIC / CAS** | Symja replacement undecided (Phase 9) | `partial-fractions` |
| 1 | **String variables** | `geom$ = 'wall'` — string-typed variables not ported | `heisler-transient` |
| 1 | **MODULE inside FOR** | Pipeline-ordering deviation, see below | `module_inside_for_loop` |
| 1 | **Ill-posed by construction — held** | Structurally square but rank-deficient (`x = y+1`, `y = z+1`, `z = x-2` reduce to `x = z+2` twice), so the solution set is a *line*. It **passes today** (Java `(2, 1, 0)`, Rust within 6.6e-14) but promoting it would freeze an arbitrary point of a continuum into the gate. Held deliberately — a judgement call the next session may overturn. | `solver_singular_linear_cycle` |

**36 total.** The nine that a naive replay scores green (the CoolProp eight plus
the rank-deficient one) are exactly the nine the previous status doc already
flagged as deliberately withheld. Nothing regressed and nothing newly unblocked.

#### Update 2026-07-30 — the CoolProp eight, re-run against the real library

The dumper's missing `COOLPROP_LIBRARY` is fixed and
`fixtures/corpus-pending/golden/` was regenerated in full (36 documents, 33
solved / 3 errored). Exactly the eight fluid documents changed; the other 28
goldens are byte-identical, so nothing else moved under the fixtures.

The trap the old row described is gone — a replay can no longer score these
green by matching "library missing" against "Rust refuses". What each one now
waits on:

| Document | Now blocked on | Phase-5 outcome (2026-07-30) |
|---|---|---|
| `rankine-cycle` | Phase-5 property functions only — `Enthalpy/Entropy/Volume` with `(P,x)`, `(P,T)`, `(P,s)` inputs. The **closest to promotable of the eight.** | **Promoted.** Worst variable `eta_th`, 6.41e-07 vs. the oracle. |
| `rankine-cycle-2` | Same as above. | **Promoted.** `eta_th`, 6.42e-07. |
| `refrigeration-vcr` | Same, plus `P_sat(fluid, T=…)`. R134a rather than water. | **Promoted.** `cop`, 1.53e-06. `P_sat` is served by inverting the tabulated saturation line. |
| `thermo-compliance` | `CompressibilityFactor`, `Volume`, `T_crit`, `P_crit` (Phase 5) + `StagnationTemp`/`StagnationPres` (compressible flow, Phase 5). | Still pending. `T_crit`/`P_crit` and the compressible functions landed; **`CompressibilityFactor` did not** — the `(P,h)` tables do not store `Z`. |
| `state-tables-multifluid` | Phase-5 properties **and** the `STATE TABLE … FLUID = … END` block type. | Still pending, now refused at `STATE TABLE`. |
| `ev-battery-cooling-pid` | Phase-5 properties + `TABLE` blocks + `DYNAMIC (method = ode23s)` (Phase 8) + the component library. | Still pending, now refused at `COMPONENT instantiation` (Phase 6). |
| `pressure-cooker` | Phase-5 two-phase properties + `DYNAMIC (method = ida)` (Phase 8) + ~9 component types incl. `BoilingVessel`. | Still pending, now refused at `COMPONENT instantiation`. |
| `ev-thermal-management` | Phase-5 two-phase R1234yf **and incompressible `EG50`** + ~20 component types. The furthest out. | Still pending, now refused at `COMPONENT instantiation`. Also still needs R1234yf and `EG50` tables, neither of which this build ships. |

**A tolerance question these eight now raise.** ~~`tests/parity.rs` compares
variables at `1e-9` relative…~~ **Answered 2026-07-30 (Phase 5).** The problem
statement below was correct and is preserved; the resolution is
`fixtures/tolerances.json` — a per-fixture *relative* tolerance carrying the
measured error and the mechanism that causes it, guarded so a stale or
unnecessary entry fails the gate. Five fixtures use it. `display_names`,
`block_count` and the error classification remain exact for all 268. The
original text:

> `tests/parity.rs` compares variables at `1e-9` relative. The regenerated
> goldens hold full-accuracy CoolProp values, and the browser build has no
> CoolProp — D1 lands on precomputed tables, whose measured error is
> `~1e-5…1e-4` relative (`docs/decisions/0001-property-backend.md`). **No
> table-backed engine can pass a `1e-9` gate on these documents.** Promoting any
> of the eight therefore needs a decision first: a per-fixture tolerance in the
> harness, or shipping `coolprop.wasm` as the accuracy path. Flagged, not
> silently resolved.

The second option — `coolprop.wasm` as the accuracy path — is **still open** and
is the only route that restores `1e-9` on those five. See
[`docs/status-phase5.md`](status-phase5.md#what-phase-5-did-not-deliver), item 2.

---

## The one engine fix this pass made

**Ignored-output sinks leaked into results.** Omitting a trailing CALL output
mints a hidden sink variable (`~ignored~N`) that the solver must still determine
— it backs a real equation. Java never surfaces it
(`EquationSystemSolver.java:1888`, `if (isIgnoredSink(name)) return;`); this
port had `parser::toplevel::is_ignored_sink` but never called it on the result
path, so `CALL LinFit(x, y : m, b)` reported a bogus `~ignored~0 = 1.0`.

Fixed in `engine.rs` at the three places Java filters:

* the surfaced `values` map (`solve_with`),
* `check`'s `variables` list,
* both reported counts — via a new `surfaced_count`, the port of
  `surfacedVarCount`, plus Java's
  `surfacedEqs = equations.size() - (allVars.size() - surfacedVars)` so hiding a
  sink's variable also hides the equation that determines it. `check` on the
  LinFit document now says *"8 equations and 8 variables"*, not *"9 and 8"*.

Verified document-by-document against the oracle. Guarded by two new tests in
`tests/procedural.rs` and the two new fixtures.

**What this fix cannot close, and why.** Java's sink counter
(`EquationParser.IGNORED_SINK_SEQ`) is a **static `AtomicLong`, process-global
and never reset per document**. A *scalar* sink is invisible in both maps, so
the counter never shows — those fixtures are reproducible. A *matrix or vector*
sink is different: Java hides it from `variables` but **keeps** its elements in
`display_names` (`~ignored~1[1,1]` …), so the frozen value depends on how many
documents ran before it in the same JVM. Proved by running one document twice:
in a batch it emitted `~ignored~1`, alone it emitted `~ignored~0`. **No
matrix-sink fixture can be frozen** (`[L] = LUDecompose(A)`, `CALL SVD(A : U, S)`
and friends), and the Rust per-document counter will never match the Java
batch value. This is a Java wart, not a port gap.

---

## What Phase 4 did **not** deliver

Ranked by how likely each is to bite the next session.

1. **43 CALL intrinsics are still refused by name** (`UNPORTED_CALL_INTRINSICS`,
   `parser/expand.rs:3120`, 44 entries of which `ss2tfij` is a variant): the
   eigen/Euler decompositions (`eigenvalues`, `eigen`, `eulerrotate`,
   `eulerdecompose`) and the whole control-systems suite (`ss2tf`, `tf2ss`,
   `zp2tf`, `tf2zp`, `series`, `parallel`, `feedback`, `pole`, `zero`, `bode`,
   `nyquist`, `margin`, `step`, `impulse`, `lsim`, `lqr`, `dlqr`, `dare`,
   `lyap`, `dlyap`, `place`, `acker`, `lqe`, `gram`, `balreal`, `pidtune`,
   `rank`, `ctrb`, `obsv`, `ss2ss`, `stepinfo`, `pade`, `rlocus`, `routh`,
   `c2d`, `d2c`, `residue`, `nichols`, `errorconst`). These land in **Phase 9**
   with the CAS. `expected_output_count` is already ported in full for all of
   them, so trailing-omission padding will be Java-exact the day they wire.
   They block 6 pending fixtures.

2. **`MODULE` CALL inside a `FOR` loop is refused, not unrolled.** Java unrolls
   `FOR` *during* flattening, so `CALL Twice(i : r[i])` in a two-iteration loop
   yields `twice$1$…` and `twice$2$…`. This port flattens CALLs in a pass that
   runs *before* unrolling and refuses the shape loudly rather than grafting one
   instance across both iterations. Fixing it means moving MODULE flattening
   past the unroller and re-basing the shared instance counter — not a local
   change. Golden staged as `module_inside_for_loop`.

3. **SVD's U/V sign convention diverges on square, non-symmetric inputs.**
   `linalg::svd` is a one-sided Jacobi; Commons Math is Golub–Reinsch, and the
   two pick opposite signs for sign-indeterminate columns. Measured:
   `A = [2 0; 0 3]` → Java `U[1,2] = −1.0`, Rust `+1.0`; `A = [3 0; 0 2]` →
   Java `U[1,1] = −1.0`, Rust `+1.0`; `A = [1 2; 3 4]` → column 2 flipped.
   **Singular values always agree**, and tall/thin (3×2) and symmetric (2×2)
   inputs match exactly — which is why `call_svd` and `call_svd_symmetric` use
   those shapes and **no diagonal / square-non-symmetric SVD fixture exists**.
   Correcting it means replacing the Jacobi kernel with a JAMA/Golub–Reinsch
   port. Accepted and documented in `linalg.rs`.

4. **Kernel CALL memory is quadratic where Java is linear.** Java's
   `new Expr.Call(name, entries)` shares one argument `List` across every
   emitted equation; Rust's `Expr::Call { args: Vec<Expr> }` owns them, so each
   kernel CALL materialises equations × entries element references. A `reserve`
   guard (port of `BoundedEquationList.addAll`) caps the worst case at the Java
   equation budget — `CALL FFT(re[1:3000], …)` went from 96 s / 3.7 GB to a
   refusal in 0.02 s / 8 MB with Java's own error — but **within** that budget
   the gap remains: FFT n=800 costs 6.1 s / 275 MB in Rust and is trivial in
   Java. A real fix needs a shared argument list (`Rc<[Expr]>`/`Arc<[Expr]>`) in
   `Expr::Call`, a change to the frozen contract file `ast.rs`.

5. **The slowest quadrature inputs are bounded but slow.**
   `robustness.rs::the_slowest_quadrature_inputs_still_terminate` is
   `#[ignore]`d because it takes ~6 minutes. It is the only ignored test in the
   workspace. The behaviour is correct — the step budget terminates — but it is
   not exercised on every run, so a regression there ships silently.

6. **Non-symmetric Cholesky reports a different (both-refuse) error.**
   `A = [1 2; 3 4]` + `CALL Cholesky(A : L)`: Java throws
   `NonSymmetricMatrixException` at evaluation; Rust reports
   `SolverException: "…Cholesky requires a positive-definite matrix"` because
   the retry ladder re-solves the merged block with A's entries back at their
   initial guess of 1.0 — symmetric but singular. Both engines refuse and the
   parity rule accepts it, so it is not frozen as a fixture. The kernel's own
   *"requires a symmetric matrix (within tolerance)"* message is being masked;
   worth a look when the retry ladder is next revisited.

7. **Matrix-sink fixtures are unfreezable** — see the JVM-global counter
   argument above. Not a port gap, but it permanently limits fixture coverage of
   the omitted-output path to scalar outputs.

8. **Newline tolerance inside `[...]`/`(...)`** in two spots (`multiAssign`
   outputs, CALL args) where ANTLR would reject — Rust is more permissive.
   Carried forward from Phase 1; **not re-verified this pass**.

9. **NaN in the five-argument `If`** errors here; Java silently falls through to
   the `gt` branch. Deliberate and tested
   (`eval.rs::five_argument_if_refuses_a_nan_comparison_where_java_falls_through`).

10. **`PLOT` / `DYNAMIC` / `PARAMETRIC` / `SYMBOLIC` blocks and string
    variables** are refused explicitly (never silently skipped). They belong to
    Phases 5, 7, 8 and 9 and block 15 pending fixtures between them.

---

## Next

1. **Phase 5 — properties.** CoolProp via Emscripten behind the same four-call
   façade (`PropsSI`, `Props1SI`, `HAPropsSI`, `get_global_param_string`) plus
   the material database (`k_`, `E_`), `MolarMass`, `eos_z` and
   `AdiabaticFlameTemp`. Unblocks 13 pending fixtures — the largest single
   block — and lets the eight poisoned goldens be regenerated honestly.
2. **Phase 6 — the component/connect layer**, and with it the 295 library
   components as corpus.
3. **Phase 9 — CAS + control systems**, the 43 refused CALL intrinsics, and the
   `SYMBOLIC` block.
4. **Opportunistic**: the `Rc<[Expr]>` argument sharing in `ast.rs` (item 4
   above) is worth doing the next time that contract file is opened for another
   reason.
