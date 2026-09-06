# Phases 7–8 — robustness and close-out

**Read this after [`status-phase7.md`](status-phase7.md).** Phase 7 wired the
transient path end to end and made the parity gate prove it. This pass is the
hardening that should have followed it: an adversarial sweep of the `DYNAMIC`,
event and analysis surfaces, the defects that sweep found, and a browser proof
that the fixes hold where they actually matter — inside the worker.

It ships **no new features**. Everything below is a guard, a regression test, a
measurement, or a correction to the written record.

> **One caveat on the numbers, stated up front.** A sibling agent was working in
> this same tree throughout this session, wiring the uncertainty / Monte Carlo
> path into `engine.rs` and growing the fixture corpus. It twice left the
> workspace failing a gate mid-edit. Every number below was run by me, raw, and
> is what I saw — but the tree contains that agent's work as well as mine, so
> read the *deltas* (+42 tests, +110 fixtures, +56.8 KiB of wasm) as
> session-wide, not as this pass's, except where a file-level diff says
> otherwise. Three small repairs to their in-flight code were needed to get a
> green gate at all, and I made them rather than report red:
>
> * `analysis/uncertainty.rs:948` — one call still passing `&defs` after their
>   `&Definitions` → `EvalContext` signature change; the workspace did not
>   compile for ~15 minutes.
> * `analysis/uncertainty.rs:75` — the now-unused top-level `Definitions` import
>   moved into `mod tests`, which is the only place still using it.
> * `engine.rs:2014` — a `#[allow(clippy::neg_cmp_op_on_partial_ord)]` with a
>   justifying comment on their new `override_uncertainties`, whose
>   `!(unc > 0.0)` is the port's NaN-rejecting parity form and must stay
>   negated, plus `rustfmt` on the same function.
>
> `cargo fmt --all` also re-wrapped several of their lines, in
> `analysis/uncertainty.rs`, `engine.rs` and `frees/src/lib.rs`. None of
> these changed any semantics.
>
> **The tree was still being edited when I finished.** My last green run of all
> four Rust gates was at 15:57; `frees/src/lib.rs` had been written 9
> seconds earlier. Treat every number here as a snapshot, and re-run the gates
> before trusting them.

## Every gate, run raw and reported honestly

| Gate | Result |
|---|---|
| `cargo test --workspace --release` | **2534 passed, 0 failed, 4 ignored** (Phase 7 baseline, re-measured at the start of this session: 2492/0/4) |
| `cargo test -p frees-core --test parity -- --nocapture` | **500 fixtures match the Java oracle** (17 at a declared table tolerance) — see the note on the count below |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 errors, 0 warnings |
| `cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings` | 0 errors, 0 warnings |
| `cargo fmt --all --check` (captured with `> file 2>&1`) | 0 diff lines |
| `npx vitest run` | **36 files, 342 passed** — *but only under Node 22; see below* |
| `npm run build` | clean |
| `wasm-pack build --release` | **2450.0 KiB raw / 1177.8 KiB gzipped** |
| Browser proof (Playwright, `dist` served by `tools/serve-dist.py`) | transient solved, ODE table rendered, trajectory plotted, values bit-identical to the Java golden, **0 `/api/` requests** |

### The vitest gate needs Node 22, and that is new information

Under the shell's default `node v20.20.2`, `npx vitest run` fails **all 36
files** before running a test:

```
TypeError: webidl.util.markAsUncloneable is not a function
  ❯ new CacheStorage node_modules/undici/lib/web/cache/cachestorage.js:20:17
  ❯ Object.<anonymous> node_modules/jsdom/lib/api.js:12:33
```

That is `jsdom` → `undici` against a Node 20 that lacks the API undici expects;
it is an environment mismatch, not a code failure, and `--pool=threads` does not
avoid it. Under `~/.nvm/versions/node/v22.23.2` the same command passes 36/342.
`web/.nvmrc` already asks for Node 22 — the gate simply cannot be run with the
version that is on `PATH` by default, and previous status documents recorded the
pass without recording that. **Use Node 22 to run the web tests.**

## What shipped, by area

### 1. `ode/` — three guards, one of them for a silent wrong answer

| Guard | Where | Replaces |
|---|---|---|
| `MAX_OUTPUT_SAMPLES = 100_000` | `ode/problem.rs`, enforced in `integrator::integrate` | an 8 GB allocation that **aborted the process** |
| `MAX_CONSECUTIVE_SET_RESTARTS = 1_000` | `ode/integrator.rs::run` | a `set`-event restart loop that ran for tens of minutes |
| finite-span screen + `check_finite` on `y0` | `ode/integrator.rs::run` | a table of `[NaN, inf, inf, …]` returned as a trajectory |

The sample ceiling is checked **before** `run`, not after, so the refusal costs
nothing regardless of the span. It lives at the allocation site rather than in
the parser because `analysis/` builds an `OdeProblem` directly.

The set-event guard is the interesting one, because **its first cut was wrong**
and the corpus is what said so. Counting consecutive restarts refuses a
legitimate model: once the adaptive step outgrows a fast switching period,
*every* step brackets a crossing and there is never an ordinary accepted step to
reset the counter. A 500 s ramp reset at `Level = 0.1` fires ~5 000 times and is
a perfectly good document. The shipped guard therefore uses the count only to
open the question and decides it on **projected completion**: at the rate this
window advanced time, can the run reach `tf` inside the remaining `MAX_STEPS`?
The two cases are not close — ~4×10³ projected steps for the sawtooth against
~9×10¹⁰ for a self-re-arming `set`.

### 2. `analysis/` — two unbounded pre-allocations

`montecarlo::run` reserved `Vec::with_capacity(sample_count)`; at `samples = 1e9`
that is **56 GB**, and it aborted *before* the deadline predicate was consulted
once. `pareto::optimize_multi` had the same shape over `population_size`. Both
now bound the reservation at the Java controller's own ceiling (1 000 samples,
200 individuals), so no in-range request behaves differently.

This is worth reading as a symptom rather than two bugs: the Java's validation
for these lives in `SolveController` / `OptimizeController`, and **this port has
no equivalent, because none of `analysis/` is reachable from a document or the
wasm boundary yet.** See the gaps list.

### 3. `tests/dynamics_robustness.rs` — 42 new tests

A new integration-test file, in the shape of `tests/robustness.rs`, stating one
rule for a surface that can run forever rather than merely crash: **every entry
point answers with a `Result` in bounded time** — not a panic, not an abort, not
a hang, and not a plausible-looking wrong answer.

Every case the brief asked for is covered, and each records what was measured:

| Case | Outcome |
|---|---|
| stiff problem on an explicit method | `Err` — either step underflow or the step budget. Terminates: measured 182 s at document level for the budget path |
| zero / negative / `t_end == t_start` span | `Err`, both endpoints quoted in the Java `Double.toString` spelling |
| **non-finite span** | **was a silent wrong answer — 200 NaN/inf rows. Now `Err`** |
| `points = 0`, `points = 1` | not errors — floor to `DEFAULT_SAMPLE_COUNT`, and the trajectory is checked against e⁻¹ |
| **`points = 1e9`** | **was an 8 GB abort. Now `Err` in 0.03 s** |
| `rtol = 0`, `rtol = 1e300`, `atol = 0` | solve, and are checked to be *correct*, not merely to have solved |
| `rtol = atol = 0` | `Err` (the scaled norm is `0/0`) |
| negative tolerances | terminate **and** give the right answer to 1e-6 — measured, not assumed |
| state with no initial condition / two `der(X)` / `der` with no state / IC for a non-state | `Err`, each naming the offending state |
| event that never crosses | runs to `tf`, 0 events, `stopped = false` |
| event firing on almost every step | terminates; hits strictly ordered |
| **self-re-arming `set` action** | **was >45 min. Now `Err` in 1.1 s** |
| legitimate sawtooth firing 1 000+ times | still solves — the regression for the guard's own first cut |
| NaN/Inf initial condition (5 spellings) | `Err` at whichever of the three layers sees it first |
| **NaN `y0` at the library boundary** | **was a full 10⁶-step burn blamed on stiffness. Now an immediate, accurate `Err`** |
| `PARAMETRIC` sweep of 1e9 rows | `Err` at parse time (`MAX_RANGE_ELEMENTS`), boundary pinned at 100 000 |
| optimizer on a flat objective (4 methods) | returns a feasible in-box point with the right objective |
| NSGA-II population 0 / 1 / 2 | floored to 8; front non-empty, objectives finite, decisions in box |
| **NSGA-II population `usize::MAX`** | **clamped to 200, not allocated** |
| LM on a singular Jacobian | `Err`, or a finite answer that actually fits — never a hang |
| **Monte Carlo `samples = 1e9`** | **was a 56 GB abort. Now bounded by the deadline predicate** |
| Monte Carlo 0 samples | empty outcome, no statistics invented from no data |

Four of these are regressions for defects found here; the rest are the standing
corpus. Three tests also assert the *absence* of a wrong diagnostic (e.g. that a
NaN `y0` is no longer reported as stiffness), which is the part that would rot
silently otherwise.

## Browser proof

Rebuilt `web/src/wasm/pkg`, built `web`, served `web/dist` with
`tools/serve-dist.py`, drove it with Playwright.

**Solved a transient** — `fixtures/corpus/dyn_plain_ode.frees` verbatim (Newton
cooling, `k = 0.05`, `Tinf = 20`, `Temp(0) = 95`, ode45, 0..60, 4 points).

* the **ODE Table renders** in the Tables window, columns `time` / `temp`;
* the **trajectory plots** — Plotly trace `temp` vs `time`, 4 points, monotone
  decay (screenshot in the session scratchpad as `p78-browser-proof.png`);
* the **Variable Explorer holds only `k` and `Tinf`** — the "a solved `DYNAMIC`
  block puts nothing in `variables`" invariant, visible in the product;
* **values match the Java golden bit for bit.** Read back out of the engine
  worker at full precision, not from the display-rounded grid:

  ```
  vars ["time","temp"]   method "ode45"   endTime 60   stopped false
  rows [[0,95], [20,47.59095803046333], [40,30.15014623853744], [60,23.734030127668667]]
  ```

  which is character-for-character the oracle trajectory recorded in
  `CLAUDE.md`.

**The new guards were then exercised in the browser**, through the real worker:

| Document | Result in the worker |
|---|---|
| `points = 1e9` | `DYNAMIC: points = 1000000000 would materialise more than 100000 output rows.` — **60 ms** |
| self-re-arming `set` | `EVENT r: the set action re-arms its own crossing — 1000 consecutive set events fired between t = 1.0000000999000114 and t = 1.0000001999000196, which is too little progress to reach t = 10.0 within the 1000000-step budget.` — **3.9 s** |
| legitimate sawtooth | solves, 11 rows, 2 events — **48 ms** |

and **the worker was still alive afterwards** (a subsequent `version` call
answered). That is the whole point: before these guards both documents took the
worker down with an allocation abort or an unbounded loop, and `panic = "abort"`
means nothing downstream can turn that into a diagnostic.

**Zero `/api/` requests.** 28 network requests, all static assets plus
`frees_bg-*.wasm`. The single URL matching `/api/` is
`/assets/api-jHsz8Gy2.js` — the bundled fetch→RPC shim, served as a file. Two
console errors, both benign 404s (`build-info.js`, `favicon.ico`).

## Fixtures: promoted and still pending

**Promoted by this pass: none.** It added regression tests, not fixtures.

**The corpus nevertheless went 390 → 500 during this session, and none of that
is mine.** The sibling agent working in this tree authored and dumped 110 new
golden fixtures while I was running gates; my first parity run of the session
reported 390 and my last reported 500, with the same 17 declared tolerances and
the same `fixtures/corpus-pending/` contents both times. Both numbers are real
and both were run raw. Attribute the +110 to that agent's work, not to this
pass — and treat the 500 as a snapshot that may have moved again by the time
this is read.

**`fixtures/corpus-pending/` is unchanged at 23**, which is the number that
matters here: nothing was unblocked and nothing regressed. Their reasons are
unchanged from `status-phase7.md`, with two label corrections applied to the
older tables (below):

| Blocker | Count | Fixtures |
|---|---|---|
| control-systems `CALL`s not ported (**Phase 9**) | 11 | `control-analysis-report`, `controller-design-lqr-pid`, `cruise-control`, `digital-control-c2d`, `estimator-gramian-balreal`, `inverse-laplace-residue`, `multi-output-destructuring`, `nichols-chart`, `root-locus-analysis`, `routh-stability`, `step-impulse-response` |
| property-backend limits (Phase 5, ledger 9) | 6 | `adv_moistair_W_passthrough`, `adv_moistair_dryair_three_way`, `ev-battery-cooling-pid`, `ev-thermal-management`, `hx-correlations-fluid`, `thermo-compliance` |
| `SYMBOLIC` / `MODULE` inside `FOR` | 2 | `partial-fractions`, `module_inside_for_loop` |
| string-valued variables in a numeric position | 1 | `heisler-transient` |
| `method = ida` — assembled, not routed | 1 | `pressure-cooker` |
| table-vs-CoolProp accuracy (worst 2.9e-6) | 1 | `state-tables-multifluid` |
| cost, not correctness | 1 | `dyn_accessor_live` |

### The doc error that was corrected

`status-phase5.md` and `status-phase6.md` both labelled their pending rows
wrongly, and both are now struck through and annotated in place (nothing
deleted), dated 2026-07-31:

* **"`DYNAMIC` (ODE/DAE) blocks (Phase 8)" → Phase 7.** Per `PLAN.md` §5,
  Phase 7 *is* Dynamics; Phase 8 is Analysis & design. Phase 7 has since closed
  that row outright.
* **"`PLOT` blocks (Phase 7)" → Phase 9.** The five documents in that row are
  control-systems documents. `PLOT` is not what is missing from them — `rlocus`,
  `nichols`, `step`, `pole` and `tf` are, and those are Phase 9. The observable
  confirmation that the old label was wrong is that Phase 7 shipped and moved
  none of them.

The `PARAMETRIC` row in `status-phase6.md` is also now marked closed by Phase 7,
and the two `ev-*` / `pressure-cooker` rows updated to say what actually blocks
them now.

## Bundle size against the budget

| | raw | gzipped | % of 3072 KiB |
|---|---|---|---|
| Phase 5 | 1867 KiB | — | 60.8 % |
| Phase 6 | 2184.5 KiB | 1086.2 KiB | 71.1 % |
| Phase 7 | 2393.2 KiB | 1156.8 KiB | 77.9 % |
| **Phases 7–8 (measured here)** | **2450.0 KiB** | **1177.8 KiB** | **79.8 %** |

**+56.8 KiB** since Phase 7, leaving 622 KiB of headroom. The guards themselves
are a handful of comparisons and three format strings; most of that delta is the
sibling agent's concurrent work, and I have not separated the two.

**Neither budget debt in `.github/workflows/ci.yml` moved.** Both are still
open, exactly as Phase 6 recorded them:

1. **Move the ~526 KB of linked property tables onto the fetch seam** that
   already exists (`props/tables.rs::install_from_bytes`). Still blocked on the
   solver being synchronous.
2. **Split the engine into lazily-loaded chunks.** Still true that
   `engine.rs::expand_component_layer` early-returns before touching the
   component library, so the chunk boundary exists in the code and only the
   linker does not know about it.

The trend is the thing to watch: 60.8 % → 71.1 % → 77.9 % → 79.8 %. Phase 9
(the CAS and control systems) is the largest remaining port, and at this rate it
will breach 3072 KiB. **Pay one of the two debts before Phase 9, not after.**

## What these phases did NOT deliver — ranked, honestly

1. **Phase 8 is not wired to anything.** This is the big one. `analysis/` is
   2 300 lines of tested library code — `optimize`, `optimize_multi`,
   `curvefit::fit`, `montecarlo::run`, `paramfit::run`, `AllRootsSolver`,
   `run_sweep`, `propagate` — and **none of it is reachable from a document or
   from the wasm boundary.** `crates/frees/src/lib.rs` exports exactly
   `solve`, `check`, `reference`, `fluids`, `property_diagram`,
   `psychrometric_chart`; `engine.rs` calls nothing in `analysis/` except the
   parametric *accessors*. The REST surface `CLAUDE.md` lists — `/api/optimize`,
   `/api/optimize/multi`, `/api/curve-fit`, `/api/solve/table`,
   `/api/solve/montecarlo` — has no in-browser counterpart. The two
   pre-allocation aborts found here are a direct consequence: with no boundary,
   there is no place for the Java controllers' input validation to live, so the
   library is the only line of defence and had none. **A `PARAMETRIC` block
   still cannot be solved from the Tables tab in the browser.**
   *(First slice closed 2026-08-22, Wave B1: `frees/src/analysis.rs`
   exports `solve_table`, driving `run_sweep` end-to-end — with
   `engine::solve_with_parametric` finally delivering the accessor channel —
   behind the transcribed controller caps (5 000 rows, a cooperative 120 s
   deadline) and their verbatim messages. The Tables workbook Solve works
   in-browser. The optimizer/Monte-Carlo/curve-fit/param-fit surfaces remain
   unwired; their validation homes arrive with their exports.)*
   *(Second slice, same day, Wave B2: `monte_carlo` exports
   `analysis::montecarlo::run` — seeded JavaRandom, the 2–1000 sample cap,
   the 120 s truncating budget, internal-temp filtering and display-unit
   sigma-width conversion per the Java controller — and the MonteCarloModal
   returns from D5's own commit. Optimizer/curve-fit/param-fit remain.)*
   *(Third slice, Wave B3: all four OptimizeController surfaces — optimize,
   optimize/multi, curve-fit, parameter-fit — exported with their controller
   validation, and the three modals restored. Every analysis module with a
   REST endpoint is now reachable in-browser; `allroots` (the findAllSolutions
   solve flag) and the document-level PARAMETRIC surfacing stay open.)*
   *(Fourth slice, Wave B4: `pid_tune` + `extract_plant` — the last two
   NOT_IN_BROWSER_ENGINE stubs with engine backing. Gap 1's headline is now
   closed: every analysis and control surface with a REST endpoint runs
   in-browser. What stays open from this item: `allroots`, the PARAMETRIC
   document-block surfacing, and `exportVector` (a product decision).)*
2. **The fuzz is adversarial, not generative.** These are 42 hand-chosen cases
   from a brief, plus the four defects they turned up. There is no property-based
   or coverage-guided fuzzing of the integrator anywhere in the tree — no
   `proptest`, no `cargo-fuzz` target. Cases nobody thought of are still
   unfound, and the honest read on a 4-defects-from-42-probes hit rate is that
   the surface has more.
3. **The wall-clock exposure is bounded but still large.** `MAX_STEPS = 10⁶` is
   the only bound on an ordinary stiff run, and at document level each step is a
   full algebraic inner solve: **measured 182 s to reach it.** In a browser that
   is three minutes of a spinning worker with no progress indication and no
   cancel. The Java's `deadlineNanos` covers this and the port dropped it
   because wasm32 has no clock — but the *boundary* does (`Date.now` is already
   imported in `frees/src/lib.rs`), so an injected deadline predicate is
   available and was not built. `montecarlo::run` already takes exactly such a
   predicate; the integrator does not.
   *(Closed 2026-08-22, Wave C1: `ode/deadline.rs` — a boundary-installed
   thread-local predicate + message, checked in `integrator::guard` (whose
   doc comment had recorded the dropped `nanoTime()` half for nine phases)
   and in the IDA step loop. The wasm `solve` export installs it from the
   request's `elapsedTimeSeconds`, clamped to the Java's 60 s
   `MAX_ELAPSED_SECONDS_CAP` and defaulted to it; a drop-guard clears it on
   every exit path so the reused worker thread cannot inherit a strike.
   Nothing installs on the native/parity path, so the replay cannot be
   killed and no golden can meet the message. The steady Newton path stays
   unclocked, as before.)*
4. **`dyn_accessor_live` is still unfixed, and neither hypothesis was tested.**
   Phase 7 named two things to look at — whether the univariate bracketing path
   is gated too narrowly (`uses_property_call`), and whether `solve_pinned`
   should cache its blocking. This pass did neither. It remains the one fixture
   blocked on cost rather than correctness.
   *(Tested 2026-08-21, Wave A5. Hypothesis 1 was already closed negatively
   (the Java gates identically); hypothesis 2 was right and is built —
   `engine.rs::PreparedPinnedSolver` hoists the structural half of
   `solve_pinned` out of the per-step loop. The fixture now solves
   bit-identically in ~12 min and the full replay dropped ~122 s → ~82 s;
   the fixture stays held on cost alone — see `fixtures/README.md`.)*
5. **The DAE path is still not routed.** `dae/solver.rs` is 2 115 lines of
   ported IDA and it has unit tests against `fixtures/dae-oracle.json`, but
   `method = ida` does not reach it from a document, so `pressure-cooker` stays
   pending. Nothing here changed that, and the DAE surface was **not fuzzed** —
   the robustness sweep covers `ode/` and `analysis/` only.
   *(Closed 2026-08-21, Wave A3: `ode/dynamic.rs::solve_with_ida` routes it,
   and the first routed document exposed two transcription bugs against
   SUNDIALS' `ida.c` — the `IDARestore` phi-rescale range and `IDASetCoeffs`'
   `alpha0` index — both fixed and recorded in `fixtures/README.md`'s
   `pressure-cooker` row. The document now integrates fully and is held only
   by the decayed-signal comparison rule. The fuzzing gap stands.)*
6. **The boot document tells users a lie.** The default example in the editor
   still reads *"Not yet ported to the browser engine: COMPONENT / connect
   models, DYNAMIC (ODE) blocks, fluid properties, TABLE / PARAMETRIC / PLOT /
   STATE blocks…"*. Components shipped in Phase 6, properties in Phase 5,
   `DYNAMIC` in Phase 7 — three of those five are now wrong, and the browser
   proof above solves a `DYNAMIC` document in the very tab that says it cannot.
   `web/src/defaultExample.ts` needs rewriting; not done here because it is a
   parity fixture (`default-boot-document`) and changing it means re-dumping the
   golden.
7. **No performance work at all.** No benchmark against the JVM engine exists in
   the tree, so "is the browser engine fast enough" remains unanswered by
   anything except the two anecdotes in this document (182 s to the step ceiling;
   `dyn_accessor_live` not terminating in 7 minutes). PLAN.md §5 puts that in
   Phase 12 and it is still there.
8. **The concurrency hazard is unaddressed.** Two agents editing one working
   tree left it failing a gate twice — once uncompilable for a quarter of an
   hour — and produced fmt churn in files neither fully owned. Nothing in the
   repo guards against that: no lock, no worktree convention, no CODEOWNERS-style
   split. It cost real time this session and will again. The cheap fix is a
   per-agent `git worktree`; the cheaper one is not running two agents on one
   tree.
