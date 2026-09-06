# Phase 7 — the transient path, wired end to end

**Read this after [`status-phase6.md`](status-phase6.md).** Phase 6 shipped the
component layer and the 295-component library; Phase 7 makes `DYNAMIC` and
`LINEARIZE` *reach the engine*, publishes the ODE Table on the wasm boundary,
registers the twenty accessor intrinsics, and — the load-bearing part — makes
`tests/parity.rs` **compare `ode_tables`**, so a transient fixture can no longer
pass vacuously.

## Gate numbers, all raw

| Gate | Result |
|---|---|
| `cargo test --workspace --release` | **2492 passed, 0 failed, 4 ignored** (was 2055) |
| `cargo test -p frees-core --test parity` | **390/390** fixtures match the Java oracle (was 361) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean |
| `cd web && npx vitest run` | 36 files, **341 passed** |
| `cd web && npm run build` | clean |
| `wasm-pack build --release` | **2393.2 KiB** raw / 1156.8 KiB gzipped — 77.9 % of the 3072 KiB budget (Phase 6: 2184.5 KiB, so **+208.7 KiB**) |

## What landed

### 1. `DYNAMIC` and `LINEARIZE` parse (`parser/toplevel.rs`, `parser/blocks.rs`)

`unsupported_construct` is now **empty** — every block form the grammar admits
has a home on `Document`. Two new fields: `Document::dynamics`
(`Vec<DynamicSystem>`) and `Document::linearizes` (`Vec<LinearizeSystem>`).

Ported from `AstBuilder.buildDynamicDef` / `buildDynamicOptions` /
`buildDynamicInit` / `buildDynamicEvent` / `buildLinearizeDef`, including every
rejection each one makes. The spelling rules the Java applies unevenly are
reproduced exactly, and each is covered by a test:

| Identifier | Case |
|---|---|
| `DYNAMIC` block name | **source case** (`ctx.IDENT().getText()`) — it is the ODE Table's title |
| `LINEARIZE` block name | lowercased |
| initial-condition state | lowercased |
| `EVENT` name | **source case** — and the recorded hit carries it verbatim into the table |
| event direction / action / `set` target | lowercased |
| matrix names (`a = Am`) | header's case |

Two decisions worth naming:

* **`DynItemInit` vs `DynItemEq` is settled by lookahead.** ANTLR resolves
  `T(0) = 95` to an initial condition by alternative order; the port's
  `at_dynamic_init` tests the full `IDENT (LBRACKET … RBRACKET)? LPAREN
  signedNumber RPAREN EQ` prefix. The trailing `EQ` is part of the test —
  without it `q = f(0)` (a call in an *rhs*) is indistinguishable by prefix.
* **`source_text` inside a block is ANTLR's `getText()`, not the source slice.**
  `Parser::tokens_text_since` concatenates token texts with no separators, so a
  `DYNAMIC` body equation reads `der(T)=-k*T`, not `der(T) = -k * T`. This is
  observable — the index-2 diagnostic quotes it — so it is pinned by a test
  alongside the verbatim `text_since` a top-level statement still gets.

### 2. The engine pipeline (`engine.rs`), at the Java positions

Each stage cites the Java method it ports. In `EquationParser.parseResult`:

* `rewrite_dynamic_bodies` (`rewriteDynamicBodies`) — a block's body lives
  *inside* the block, so `rewrite_statements` never reached it. Without this a
  scheduled input written as `RIN.out.mdot = f(time)` leaves the port variable
  free.
* `route_storage_into_dynamic` (`routeStorageIntoDynamic`) — **this is the
  steady ↔ transient duality's other half.** With storage and no `DYNAMIC`
  block, `steady_storage_equations` still turns each `der(X) = rhs` into
  `rhs = 0` (Phase 6's behaviour, unchanged). With storage *and* exactly one
  block, the component equations become that block's body and the `init(...)`
  lines its initial conditions.

In `EquationSystemSolver.solve`:

* `inject_linearizations` (`injectLinearizations`) — after complex expansion,
  before the blocker. Emits `A[i,j] = value` equations plus the 1-subscript form
  for a single-column matrix, and registers display names `putIfAbsent`.
* the **ODE-only shortcut** — a document that is *only* `DYNAMIC` blocks has no
  analytic equations, so the blocker (which rejects an empty system) is skipped
  and the blocks run directly. `check()` takes the same shortcut and reports the
  block's own equation count as both counts.
* the **accessor / second-solve pass** — `augment_accessor_dependencies`
  (`augmentAccessorDependencies`) adds `+ 0·v` terms so Tarjan and the Newton
  Jacobian see the coupling, `accessor_bridge` (`installAccessorContext`)
  installs the live bridge, and `relaxed_ode_settings` loosens the outer solve to
  `1e-4` (inner to `1e-7`).
* `solve_dynamic_systems` (`solveDynamicSystems`) — after the analytic solve,
  with the solved scalars as the base values. The result rides out on the new
  `Solution::ode_tables`.

`EvalContext` grew two optional channels (`ode`, `parametric`) and the
block-solving chain now threads an `EvalContext` where it threaded a
`&Definitions`. The ODE bridge is held as `&dyn OdeTableAccessors` rather than
as the concrete `DynamicAccessorContext<'a>`, which is invariant in its lifetime
and would infect `EvalContext<'a>` with it.

### 3. Twenty accessor intrinsics (`eval.rs`)

The `UNPORTED` list lost both accessor sections. Registered under the exact
names and arities in `FunctionRegistry.java`:

* ODE: `ODEValue`, `FinalValue`, `MaxValue`, `MinValue`, `TimeAt`, `ODEAvg`,
  `ODESum`, `ODEStdDev`, `ODEMin`, `ODEMax` — all `Arity::Range(1, 2)`, which is
  what `Evaluator.evalCall`'s `args.size() > 1` test admits.
* parametric: `TableRun#`, `TableRun`, `NParametricRuns`, `TableValue`,
  `TableSum`, `TableAvg`, `TableMin`, `TableMax`, `TableStdDev`, `IntegralValue`.

**With no context installed each answers the Java's null-context default rather
than erroring** — `0.0` for nineteen of them, `1.0` for `TableRun#`/`TableRun`.
That is what makes `MaxValue('h')` harmless in a steady document, and it is
tested.

### 4. `tests/parity.rs` compares `ode_tables` — and was proved able to fail

See [`fixtures/README.md`](../fixtures/README.md#ode_tables--why-a-transient-fixture-needs-it)
for the policy and the four perturbation classes that were each observed going
red before the golden was restored.

### 5. Fixture promotion: **29 promoted, 361 → 390**

Promoted (all match the oracle at the default `1e-9`, no new tolerance entries):

* the 19 `dyn_*` probes staged by the DYNAMIC agent, minus `dyn_accessor_live`
* `damped-oscillator-ode`, `newton-cooling-transient`, `transient-heat-rod`,
  `sounding-rocket-trajectory`, `engine-cycle-wiebe` — five of the eight named
  DYNAMIC documents. Their goldens predated the dumper's `ode_tables` section
  and were **re-dumped from the Java oracle**, so their trajectories are
  compared, not ignored. `sounding-rocket-trajectory` is the strongest of these:
  500 rows × 8 columns plus a `stop` event, all matching.
* `linearize-thermal-siso`, `linearize-thermal-2x2`
* `damped-oscillator`, `driving-cycle-energy`, `projectile-trajectory`,
  `solver_singular_linear_cycle`

### 6. `odeTables` reaches the frontend

`crates/frees/src/lib.rs::solve_success` emits `OdeTableDto` — `vars` /
`units` / `rows` / `events` / `method` / `stopped` / `endTime`, the shape
`web/src/api.ts` already declared and `tables.ts::odeTableFromDto` already
consumed. Nothing in `web/src` needed changing: `mergeCodeTables` was already
wired for it and `App.tsx` already passed `response.odeTables`. The gap was
entirely on the Rust side.

## What is still blocked, and by what

23 fixtures remain in `fixtures/corpus-pending/`. **None is blocked by this
phase's work**, and the reason is recorded per fixture:

> **Updated 2026-08-01 by the Phase 9 close-out.** Twelve of these were
> promoted (`fixtures/corpus` is now 531, up from 390 at the end of Phase 7);
> the `Status` column below records what happened to each row. The full,
> document-by-document re-check of every remaining hold lives in
> [`fixtures/README.md`](../fixtures/README.md) under *"Re-check 2026-07-31,
> Phase 9"*, and the phase write-up is
> [`docs/status-phase9.md`](status-phase9.md).

| Blocker | Fixtures | Status |
|---|---|---|
| control-systems `CALL`s not ported (`lqr`, `ss2tf`, `c2d`, `lqe`, `residue`, `nichols`, `rlocus`, `routh`, `step`, `pole`, `tf2ss`) | `control-analysis-report`, `controller-design-lqr-pid`, `cruise-control`, `digital-control-c2d`, `estimator-gramian-balreal`, `inverse-laplace-residue`, `multi-output-destructuring`, `nichols-chart`, `root-locus-analysis`, `routh-stability`, `step-impulse-response` | **CLOSED 2026-08-01** for ten of the eleven — Phase 9 ports all 41 control `CALL`s and they are promoted. `estimator-gramian-balreal` **remains open** for a *different* reason: `lqe`/`gram`/`balreal` all work and match, but `balreal`'s state signs differ from the oracle's (divergence ledger item 24) |
| property-backend limits (Phase 5: no `HAPropsSI`, no `INCOMP::` fluids, no viscosity/conductivity in the `(P,h)` split table, states outside the tabulated box) | `adv_moistair_W_passthrough`, `adv_moistair_dryair_three_way`, `ev-battery-cooling-pid`, `ev-thermal-management`, `hx-correlations-fluid`, `thermo-compliance` | open. **Re-diagnosed 2026-07-31:** `ev-battery-cooling-pid` is *not* a table-coverage limit — every state it really uses is servable; it fails because a property failure at the **initial guess** is fatal before Newton in this port and merely `NaN` inside it in the Java. `thermo-compliance` is blocked by `CompressibilityFactor` alone |
| `SYMBOLIC` / `MODULE` inside `FOR` | `partial-fractions`, `module_inside_for_loop` | **`partial-fractions` CLOSED 2026-08-01** — `SYMBOLIC` reaches `cas::engine::solve_coefficients` and the fixture is promoted (`A = 2`, `B = −1`), proved in-browser as well. `module_inside_for_loop` open: MODULE flattening must still move past the `FOR` unroller |
| string-valued variables in a numeric position | `heisler-transient` | open, and **re-diagnosed 2026-07-31 as a real divergence, not a missing feature**: `parser/StringVariables.java` (~130 LOC + one call site) is named in this port's own pipeline docstring but does not exist |
| `method = ida` — the implicit-DAE path is assembled but not routed | `pressure-cooker` | open, unchanged |
| table-vs-CoolProp accuracy (worst 2.9e-6) — needs a `tolerances.json` entry, deliberately not added here | `state-tables-multifluid` | **CLOSED 2026-07-31** — the entry was added (`relative: 1e-5`, `measured: 2.8963e-6`) with the mechanism recorded in its `reason`; 17 fixtures already carry the same one |
| **cost, not correctness** — see below | `dyn_accessor_live` | open. Re-timed 2026-07-31: **no output after 420 s**. Follow-up hypothesis 1 below is now **closed negatively** — see the note under it |

### `dyn_accessor_live`: the one honest performance finding

The live-accessor second-solve pass is **correct** — with the fixture's
integration made cheap (`maxstep = 60` instead of the default `span/100 = 0.6`)
it converges to the oracle's `dk = 0.03258171700906962` to 4e-9 in 4.0 s, and a
reduced version of the same document (`0 .. 1`, `points = 3`) solves in 0.06 s
and hits its target exactly. At the fixture's own settings it does **not
terminate in 7 minutes**.

The mechanism: `FinalValue('Temp') = 30` is solved for `k` starting from the
default guess `1.0`, where the residual `75·e^(−60k) − 10` is flat to ~1e−24.
Newton's step is astronomical, every halving fails, and the block walks the whole
retry ladder. That costs ~2000 integrations — tolerable when an integration is
one step, fatal when `max_step = 0.6` forces 100+ steps, each running a full
`solve_pinned` (fresh `block_system` + Tarjan + Newton).

Two things a follow-up should look at, in this order:

1. **Is the ladder the same as the Java's?** The oracle converged. If Java's
   1×1 block path brackets before it iterates, this port's
   `try_univariate_bracketing_solve` is gated too narrowly (it currently
   requires `uses_property_call`).

   > **CLOSED NEGATIVELY, 2026-07-31.** It is *not* gated too narrowly: the
   > Java gates it the same way, at `EquationSystemSolver.java:1148-1152`, and
   > says why — *"Scope this resort to property inversions … For ordinary
   > algebra a bracketing rescue would bypass the user's Newton iteration-limit
   > stop criterion and could pick a different root than Newton's basin."*
   > `FinalValue('Temp') = 30` has no property call, so **neither** engine
   > brackets it. The remaining lever is hypothesis 2. Worth knowing while
   > pulling it: the Java's ladder is wall-clock bounded
   > (`config.deadlineNanos()`, checked inside the bracketing sampler); wasm32
   > has no clock, so the port cannot inherit that escape hatch and the ladder
   > has to be *cheap* rather than interruptible.
2. **`solve_pinned` re-blocks from scratch every step.** So does the Java, but a
   cached blocking for an unchanging subsystem is a pure win on the hottest loop
   the transient path has.

## Divergences opened by this phase

1. **`complete_display_names` is now `putIfAbsent`, not `put`.** It composed
   `base_display + suffix` for every array element, which downcased the
   `A[1,1]` spellings `emit_matrix` registers from a `LINEARIZE` header. Element
   names are otherwise always absent (the parser registers only an array's base
   spelling), so every pre-existing fixture is unaffected — confirmed by the
   390-fixture replay.
2. **No `changeInVariables` in `relaxed_ode_settings`.** The Java clamps
   `Math.max(base.changeInVariables(), 1e-9)`; this port's Newton has no such
   knob (its stop rule is the residual only), so only `rel_tolerance` is
   transcribed.
3. **The accessor bridge is dropped before the final `solveDynamicSystems`.**
   The Java reaches there with its thread-local still installed. Re-integrating
   from scratch against the final values is the safer reading — a cached table
   from a Newton iterate must not become the published trajectory — and no
   fixture distinguishes the two.
4. **`ode_tables` absent from a golden is "not a claim", not "no tables".** A
   pre-Phase-7 golden makes no assertion about trajectories. But a Rust engine
   that *produces* a table where the golden is silent fails with a "re-dump this
   fixture" message, so absence can never hide a transient.
