# Phase 9 — the CAS and the control-systems suite, wired end to end

**Read this after [`status-phase78.md`](status-phase78.md).** Phase 9 is the one
PLAN.md flags as *★ highest risk*, for one reason: the Java reaches **Symja**
(LGPL-3.0) for thirteen symbolic operations, and neither Symja nor `symbolica`
can ship inside an MIT-licensed statically-linked wasm binary. So the CAS is
**written**, not depended on — exact rational arithmetic over ℚ in
`cas/{poly,ratfun,ops}.rs`, a Laplace transform table in `cas/laplace.rs`, and
`CasEngine`/`CasIdentity` on top in `cas/engine.rs`.

The half of the phase everyone assumes is Symja's is **not**: transfer-function
algebra, symbolic `ss ↔ tf`, LQR/Riccati, pole placement, PID tuning and time
responses are all frees' own Java (`ControlSystemsFlattener` 1,978 LOC +
`ControlSystemsEvaluator` 1,140 + six `cas/*.java` files), and they transcribe
like any other phase. `control/mod.rs` says so at the top so nobody re-derives
it.

This document is the **robustness and close-out pass** over that work: what
landed, the gate numbers I ran myself, the two defects the adversarial sweep
found and fixed, and — the part that matters most for whoever picks this up — an
honest ranked list of what Phase 9 did **not** deliver.

---

## Gate numbers, all raw

Every number below was produced by running the command, gated through a file and
read raw (`rtk` swallows clippy warnings and truncates output; `cargo fmt
--check` writes its diff to *stdout*, so `2>` alone looks falsely clean).

| Gate | Result |
|---|---|
| `cargo test --release --workspace` | **2933 passed, 0 failed, 4 ignored**, exit 0 (Phase 7: 2492) |
| `cargo test -p frees-core --test parity` | `golden_corpus_parity` **ok** — all **531** fixtures match the Java oracle in 47.3 s (Phase 7: 390) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings` | clean |
| `cargo fmt --all --check` | clean (empty stdout **and** stderr) |
| `cd web && npx vitest run` (Node 22) | 37 files, **352 passed** |
| `cd web && npm run build` | clean |
| `wasm-pack build --release` | **3336 KiB raw / 1497 KiB gzipped** — **OVER the 3072 KiB budget by 264 KiB (108.6 %)**. See [Bundle](#bundle-the-one-red-gate). |

The browser proof is [below](#browser-proof); it was driven with Playwright
against `web/dist` served by `tools/serve-dist.py`, and it recorded **zero
`/api/` requests**.

> **One caveat on the test total, stated because it is the kind of thing that
> gets quietly assumed.** Another agent was working in this same tree while the
> final gate ran, so the +23 over the 2910 this pass started from is *not*
> wholly attributable to the 22 tests added here — one test is someone else's.
> The run itself is clean and self-consistent: it was executed in an isolated
> `CARGO_TARGET_DIR` against the tree as it stood, exit 0, no `FAILED` and no
> `panicked at` anywhere in its output. `fmt` and both `clippy` invocations were
> re-run *after* the last source edit in this pass, not before it.

---

## What shipped, by area

### 1. The CAS core — exact rational algebra (`cas/{poly,ratfun,ops}.rs`)

Nine of the thirteen Symja operations are polynomial/rational-function algebra
and are met by a from-scratch implementation over ℚ:

* `poly.rs` — `UPoly`/`MPoly` over `BigRational`, Yun square-free
  decomposition, modular + primitive-PRS GCD, and a **Zassenhaus factoriser**
  (distinct-degree + equal-degree splitting mod *p*, Hensel lifting, subset
  recombination). Bounded by `MAX_MODULAR_FACTORS = 32`,
  `MAX_RECOMBINATIONS = 60_000`, `MAX_MGCD_STEPS = 20_000`.
* `ratfun.rs` — `RatFun`/`URatFun` in lowest terms by construction, plus the
  partial-fraction decomposition `Apart` needs.
* `ops.rs` — the lowering from `Expr` to the rational IR and back, the printer
  (`display`), and the nine operations. Bounded by `MAX_CAS_DEPTH = 256`,
  `MAX_POW = 64`, `MAX_TERMS = 4096`, `MAX_APART_DEGREE = 64`.

**New dependencies, named loudly:** `num-bigint`, `num-integer`, `num-rational`,
`num-traits` — all **MIT OR Apache-2.0**, all compatible with frees' MIT licence
and with static linking into wasm. They are the ones PLAN.md §5 anticipated. No
others were added.

### 2. `Laplace` / `InverseLaplace` (`cas/laplace.rs`)

A transform table plus partial fractions, with five explicit ceilings
(`MAX_DEPTH = 64`, `MAX_POWER = 18` — because `19!` is no longer an exact `f64`
and a silently-rounded factorial is the "plausible and wrong" failure this
module exists to refuse — `MAX_DERIVATIVE_ORDER = 8`, `MAX_T_MULTIPLY = 4`,
`MAX_DEGREE = 24`). Everything outside the table is refused **by name**, with
the reason in the message (*"— the table covers `exp(a*t)` only"*).

### 3. `CasEngine` and `CasIdentity` (`cas/engine.rs`)

The Java shape survives — parse a string, run one named operation, hand back an
expression plus its LaTeX plus the engine's printed form. What does not survive
is Symja's plumbing, and `cas/engine.rs`'s header records why each piece is
gone: the three-second executor timeout has no wasm analogue (bounds are
structural instead), the rule-engine warm-up does not exist, and
`SymjaOutputNormalizer`'s `Log(` → `ln(` rewrite is unnecessary because
`ops::display` emits `ln(` directly.

`CasIdentity::solve_coefficients` is what a `SYMBOLIC` declaration feeds. The
Java hands the coefficient equations to Symja's general `Solve`; its own doc
comment records that they are **linear in the unknowns**, so this port solves
them by exact Gaussian elimination over ℚ and **refuses anything nonlinear by
name** rather than half-attempting it.

### 4. The control suite (`control/{tf,ss,design,pid,response}.rs`)

Transfer-function algebra, `ss2tf`/`tf2ss`, `zp2tf`/`tf2zp`, series/parallel/
feedback in both TF and SS form, `bode`/`nyquist`/`nichols`/`margin`/`routh`,
`residue`, `c2d`/`d2c`, `pade`, `rlocus`, `lyap`/`dlyap`/`dare`, `lqr`/`dlqr`/
`lqe`, `place`/`acker`, `gram`/`balreal`, `ctrb`/`obsv`/`rank`/`ss2ss`,
`stepinfo`, `pidtune`, and `step`/`impulse`/`lsim`.

`linalg.rs` grew the pieces genuinely missing before: `inverse`, `solve`,
`pinv`, `solve_or_pinv` and a general real `eigen`. `design.rs` documents the
one design decision worth naming — the continuous ARE goes through the **matrix
sign function** of the Hamiltonian, not an ordered Schur form, so the classic
failure mode (mixing the stable and anti-stable invariant subspaces and
returning a gain that looks plausible and does not stabilise) cannot arise on
that path.

### 5. The 41 control `CALL`s reach a document (`parser/expand.rs`)

`flatten_call_proc` dispatches all 41 names (`control::flatten::CALL_NAMES`, 40
+ `mason`) into `control::flatten::flatten` through a `ControlHost`/
`ControlShapes` adapter pair — the typed form of the Java's `csFlattener`
back-reference, supplying `parseMatrixInfo` / `parseVectorInfo` / `expandExpr` /
`constIndex` / `registerShape` and `ctx.out().add`. Outputs auto-size from the
inputs (`control::flatten::auto_size`), so `[K, cpr, cpi] = rlocus(num, den)`
needs no restated lengths, and the generated equations go through the **same**
`MAX_GENERATED_EQUATIONS = 25_000` budget every other flattener uses, asserted
*before* a batch is built.

### 6. The REPL, `PLOT` and `SYMBOLIC` reach the browser

`crates/frees/src/repl.rs` (new, 973 lines) ports `ReplEvaluator`'s
seven-way dispatch: the 13 CAS ops, assignment, bare-variable echo and general
expressions are **ported**; `CALL` lines, matrix/vector literals, range vectors
and single-unknown equation solves are **refused by name** so a user never gets
a wrong answer in place of a missing feature. `definedPlots[]` now reaches the
boundary from both `solve` and `check`.

---

## What the adversarial sweep found

The rule for this pass is `tests/dynamics_robustness.rs`'s: **every entry point
answers with a `Result` in bounded time.** Not a panic, not an abort, not a
hang, and not a plausible-looking wrong answer. The new regression file is
`crates/frees-core/tests/cas_control_robustness.rs` (22 tests).

Phase 9's risk profile is different from Phase 7's. An integrator can run
forever; a CAS over exact rationals can *also* blow up in memory (a coefficient
is a `BigInt`, and nothing about the type caps its size) and a factoriser can
blow up in time. And there is no clock: wasm is single-threaded, so the Java's
"submit to a daemon executor and cancel after three seconds" has no analogue and
**every bound must be structural**.

### Defect 1 — the CAS was `O(n⁴)` in its generator count. FIXED.

**The headline finding, and it does not need an exotic input to trigger.** The
worst case measured was `Expand` over a sum of 200 distinct symbols —
`a + b + c + …`, no powers, no functions, nothing unusual — at **256 seconds**.

Two independent causes, both fixed, and it is worth reading them in this order
because the first one alone is worth 5× and looks like the whole answer:

1. **`align_terms` recomputed its variable-position map once per term.** Every
   ring operation re-aligns both operands onto the union of their variable
   lists, and the map from the target list to the source's is the *same for
   every monomial*. Computed inside the per-term loop it made each operation
   `O(terms · |to| · |from|)`. Hoisted into a new `align_index`, called once per
   `align_terms`.
2. **`RatFun::normalise` ran a multivariate GCD for plain polynomial
   arithmetic.** This is the big one. Every `add`/`sub`/`mul`/`div` normalises,
   and `normalise` unconditionally called `MPoly::gcd(num, den)` — even when
   `den` was the constant `1`, which is *every* polynomial operation.
   `MPoly::gcd` runs a 20,000-step recursive primitive-PRS plus two certifying
   exact divisions. `normalise` now short-circuits on a constant denominator
   (the value is provably identical: the general path reduces by
   `gcd(num, c) = 1` and then divides by exactly that same constant), and
   `MPoly::gcd` returns `1` immediately when either side is a non-zero constant
   — over ℚ every non-zero rational is a unit, so that *is* the answer the
   general path computes.

Measured on the same machine, in `--release`, on `Expand` of a left-associated
sum of *n* distinct symbols:

| *n* | before | after fix 1 only | after both | speed-up |
|---|---|---|---|---|
| 80 | 4.94 s | 2.45 s | **0.027 s** | 183× |
| 140 | 57.2 s | 14.3 s | **0.107 s** | 535× |
| 200 | **256.1 s** | 50.4 s | **0.412 s** | **622×** |

`Factor` tracks it within 30 % at every point. A **dense polynomial of degree
200** — the same cost curve reached through exponents rather than symbols — went
from **66 s** (`Expand`) and **90 s** (`Factor`) to under 0.1 s; `Simplify` and
`Apart` on it cost 67 s each before and are now instant. A product of forty
`(sin(kx) + 1)` factors, which correctly hits `MAX_TERMS` and refuses, took
1.9 s to reach that refusal and now takes 0.056 s.

In the browser, through the REPL, against the shipped wasm:
`expand(v0 + … + v199)` **103 ms**, `factor(1 + 2x + … + 201x²⁰⁰)` **303 ms**.
Before the fix the first of those would have wedged the Web Worker for over four
minutes.

Why an ordinary input reaches it: exponents above `ops::MAX_POW = 64` each
intern as their own **opaque generator**, so a dense polynomial of degree
*n* > 64 is secretly a multivariate polynomial in *n* − 63 variables. Degree and
symbol count are the same axis.

Regressions: `a_wide_sum_of_distinct_generators_stays_fast`,
`a_dense_degree_200_polynomial_stays_fast`. Both assert the *answer* as well as
the time, so a future "optimisation" that drops terms fails them.

### Nothing else. Every other attack found the bounds already in place.

Stated plainly because it is the useful result: the attack list was
degree-200 polynomials, 10,000-digit coefficients, `Factor` on a product of many
distinct irreducibles, `Apart` at a 50-fold repeated pole, division by a zero
polynomial, empty/constant polynomials everywhere a polynomial is expected,
`Integrate` on deeply nested expressions, `Laplace` of an expression with no
transform, LQR on singular and non-stabilisable pairs, pole placement with
repeated desired poles, `tf` with a zero denominator, and 200-block series/
feedback chains. **Every one of them already answered with a clean `Result` in
bounded time.** They are now pinned as regressions rather than left to chance:

| Attack | Outcome | Pinned by |
|---|---|---|
| 10,000-digit numerator | refused by name — *"number is not representable as an exact rational"*, instantly, no `BigInt` allocated | `a_huge_integer_literal_is_refused_not_rounded` |
| 12 distinct irreducible quadratics; 16 distinct linear factors; a Swinnerton-Dyer octic | all factor correctly, < 0.02 s | `factoring_a_product_of_many_irreducibles_terminates` |
| `Apart(x^49/(x+1)^50)` | all fifty residues, 1.25 s | `apart_at_a_fifty_fold_repeated_pole_answers` |
| `1/0`, `x/(x-x)`, `1/(0*x)`, `(x+1)/(x^2-x^2)` at five ops each | `CasError::DivisionByZero` | `a_zero_denominator_is_refused_by_every_operation` |
| `0`, `1`, `-3/4`, `x-x` at all eleven ops | exact answers, no panics | `constant_and_empty_polynomials_are_handled_everywhere` |
| 400-deep nest | refused at the **parser** (`too deeply nested`); `MAX_CAS_DEPTH` is the belt to that brace | `deeply_nested_input_is_refused_at_the_parser_and_at_the_lowering` |
| `laplace` of `exp(t²)`, `1/t`, `ln(t)`, `t^t`, `sin(sin(t))`, `abs(t)`, `t^40`; `ilaplace` of `s`, `1`, `exp(s)`, `1/(s-s)` | refused by name, each saying why | `laplace_refuses_by_name_outside_its_table` |
| LQR non-stabilisable / `R = 0` / `Q = −I` / empty / ragged / mismatched; 40-state chain | four refusals with distinct messages, three shape guards, 40 states in 0.034 s | `lqr_refuses_singular_and_non_stabilisable_pairs` |
| `place` with repeated poles; wrong pole count; uncontrollable; 30 repeated poles on 30 states | repeated poles **succeed** (verified by trace and determinant of `A − bK`), the rest refuse | `place_handles_repeated_poles_and_refuses_the_rest` |
| zero/empty denominator at 13 transfer-function entry points | all refuse | `a_zero_denominator_is_refused_across_the_transfer_function_surface` |
| 200 × series, 200 × feedback, 200 × `ss_series` + poles + `ss2tf` | bounded; slowest step is `ss2tf` at 201 states, **5.0 s** | `two_hundred_block_chains_stay_bounded` |
| descending / duplicated / non-finite / 5000-sample time grids | non-finite refused, the rest answer | `degenerate_time_grids_are_bounded` |

### Two behaviours measured and *documented* rather than changed

1. **A non-finite plant yields a non-finite gain, not a refusal.**
   `lqr` with `NaN` or `inf` in `A` returns `Ok([[NaN, NaN]])`. The matrix-sign
   iteration's convergence test is `‖Z − Z_prev‖ < SIGN_TOL`, which a NaN never
   satisfies, so it runs its full `SIGN_MAX_ITERS = 100` and hands back NaN.
   `place` with a NaN desired pole behaves the same way. This is **bounded**
   (the test asserts that) and it is **visible** — a NaN in the solution table
   is not a plausible number — so it is recorded rather than guarded, and the
   test asserts the gain is *entirely* NaN so a future change cannot make it
   finite-looking. `a_non_finite_plant_yields_a_non_finite_gain_in_bounded_time`.
2. **`rlocus` and `routh` answer on a degenerate denominator; that is
   transcribed.** `rlocus(num, [0])` returns its gain vector with **empty**
   pole rows, because the Java sizes the table `M × (den.length − 1)` and
   degree 0 yields empty rows; only an *empty* denominator (the Java's
   `NegativeArraySizeException`) is refused. `routh([])` and `routh([0,0,0])`
   return `0` unstable roots, and `margin` of an all-zero loop returns the
   `1e9` infinite-margin sentinels. All four are pinned so a future "cleanup"
   cannot quietly diverge from the oracle.

### One rough edge, left as-is with the reason

A model that is degenerate **only at its solution** cannot be caught at parse
time. `n = [1]; d = [0]; [y, t] = step(n, d)` flattens fine — `d` is a variable,
not a literal — and the generated `step$…` intrinsics start failing as Newton
walks toward `d = 0`. The outcome is the solver's own bounded report,
*"Newton's method did not converge within 250 iterations"*, not the underlying
*"tf2ss: leading denominator coefficient cannot be zero"*. That is how **every**
failing intrinsic behaves inside a Newton block and is not specific to the
control suite, so it is pinned
(`a_plant_that_degenerates_at_the_solution_terminates_with_a_diagnostic`) rather
than special-cased. A general fix belongs with the residual-error plumbing, not
here.

---

## Browser proof

Rebuilt `web/src/wasm/pkg`, built `web/dist`, served it with
`python3 tools/serve-dist.py web/dist 8900`, and drove it with Playwright MCP.
The CodeMirror handle is `.cm-content`'s `cmTile.view`.

| Step | Result |
|---|---|
| Solve a document using a **control CALL chain** — `ss2tf` → `series` → `margin` → `pole` on a cruise-control plant | **Solved.** 39 workspace variables; `gm = 1.000000e+9` (the infinite-margin sentinel, correct for a loop with a pole at the origin), `pm = 90`, `cl_pr` a 3×1 vector, `num_ol`/`den_ol` 4×1 vectors |
| REPL `factor(x^4 - 1)` | `= (-1+x)*(1+x)*(1+x^2)` |
| REPL `apart((s+3)/(s^2+3*s+2), s)` | `= 2/(1+s)-1/(2+s)` |
| REPL `laplace(sin(2*t), t, s)` | `= 2/(s^2+4)` |
| REPL `ilaplace(1/(s^2+4), s, t)` | `= 1/2*Sin(2*t)` |
| REPL `integrate(exp(x^2), x)` | `✗ integrate: no closed form found for this input.` — the refusal renders as a refusal |
| Solve a **`SYMBOLIC`** document — `tf([1,3],[1,3,2]) = A/(s+1) + B/(s+2)` | **Solved.** `A = 2`, `B = -1`, `y_initial = 1` — the residues the fixture documents, computed by `CasIdentity` in the browser |
| REPL `expand(v0 + … + v199)` | 103 ms (Defect 1's regression, in the shipped wasm) |
| REPL `factor(1 + 2x + … + 201x²⁰⁰)` | 303 ms |
| **`/api/` requests** | **ZERO.** 24 requests total, all static assets plus `frees_bg-*.wasm`. The only non-200s are `/build-info.js` and `/favicon.ico`, both 404 and both pre-existing |

---

## Fixtures: 31 promoted, 500 → 531. 11 pending.

Promoted from `fixtures/corpus-pending/` (12) — **ten** of the eleven that
[`status-phase7.md`](status-phase7.md) held under "control-systems `CALL`s not
ported" (all but `estimator-gramian-balreal`, see below), plus `partial-fractions`
which the CAS unblocked and `state-tables-multifluid` which a `tolerances.json`
entry unblocked:

`control-analysis-report`, `controller-design-lqr-pid`, `cruise-control`,
`digital-control-c2d`, `inverse-laplace-residue`, `multi-output-destructuring`,
`nichols-chart`, `partial-fractions`, `root-locus-analysis`, `routh-stability`,
`step-impulse-response`, `state-tables-multifluid`.

New probe fixtures staged and promoted (19), each dumped from the Java oracle:
`ctl-c2d_d2c`, `ctl-ctrb_obsv_rank`, `ctl-interconnect`, `ctl-lqe_pidtune`,
`ctl-lqr-marginal_uncontrollable`,
`ctl-lqr-stabilisable_stable_uncontrollable`,
`ctl-lqr-stabilisable_two_stable`, `ctl-lqr_3state`, `ctl-lqr_dblint`,
`ctl-lqr_mimo`, `ctl-lyap_dare`, `ctl-mason_graph`, `ctl-pade_delay`,
`ctl-place_acker`, `ctl-residue_routh_rlocus`, `ctl-response_analysis`,
`ctl-ss2ss_ij`, `ctl-tf_roundtrip_proper`, `ctl-zp_roundtrip`.

One new `fixtures/tolerances.json` entry: `state-tables-multifluid` at `1e-5`
(measured `2.8963e-6`), with the full mechanism recorded in the entry's `reason`
field. It is the D1 table-vs-CoolProp gap 17 other fixtures already carry.

### The 11 still pending, and why

**None is blocked by Phase 9's work.** The complete re-check of every hold —
document by document, replayed through `frees-cli` and compared against its
golden with the gate's own rules — is in
[`fixtures/README.md`](../fixtures/README.md) under *"Re-check 2026-07-31, Phase
9"*. Summary:

| Blocker | Fixtures |
|---|---|
| **property backend, D1** — no `HAPropsSI`, no `INCOMP::` fluids, no viscosity / conductivity / `Z` in the `(P,h)` split table | `adv_moistair_W_passthrough`, `adv_moistair_dryair_three_way`, `hx-correlations-fluid`, `thermo-compliance`, `ev-thermal-management` |
| **real divergence** — `parser/StringVariables.java` (~130 LOC + one call site) is listed in the port's own pipeline docstring but does not exist, so `geom$ = 'wall'` reaches the blocker as an equation | `heisler-transient` |
| **real divergence** — a property failure at the *initial guess* is fatal before Newton in the port and merely `NaN` inside it in the Java; the 79-equation inner block never gets a finite first residual | `ev-battery-cooling-pid` |
| `method = ida` — the implicit-DAE path is assembled but not routed | `pressure-cooker` |
| MODULE flattening must move past the `FOR` unroller | `module_inside_for_loop` |
| **cost, not correctness** — re-timed at **no output after 420 s** | `dyn_accessor_live` |
| **`balreal`'s state signs differ from the oracle's** — a Phase 9 debt, not the Phase 7 "control `CALL`s not ported" one | `estimator-gramian-balreal` ← **diagnosed by this pass, see below.** It *solves*: 31 of its 35 variables match the golden to better than `1e-9`, including `L`, `Wc` and `Wo`. The four that do not are `Ab[1,2]`, `Ab[2,1]`, `Bb[2,1]`, `Cb[1,2]` — right magnitude, wrong sign. Divergence ledger item 24 |

---

## Bundle: the one red gate

> **Stale — corrected 2026-08-21.** These numbers are pre-commit: the
> `opt-level = "s"` lever this section describes as "measured, not taken" was
> taken in this phase's own commit (worth 535 KiB re-measured at the then-HEAD
> — see `docs/status-phase10.md`'s bundle section), and D9 later removed the
> property tables from the browser bundle entirely. As of 2026-08-19 the
> bundle is **2721.9 KiB raw / 1118.2 KiB gzipped — 88.6 % of budget, green**.
> The section is kept for its section-breakdown analysis, which still holds.

```
wasm-pack build crates/frees --release --target web
  →  3,416,518 bytes  =  3336 KiB raw  /  1497 KiB gzipped
     budget            =  3072 KiB raw
     OVER BY              264 KiB  (108.6 % of budget)
```

Phase 8 left it at 2462 KiB (recorded when this pass began, not re-measured
here). Phase 9 therefore cost **+874 KiB**, essentially all code:
section breakdown of the shipped module is **code 2478 KiB, data 852 KiB**, and
there are no debug or name sections left for `wasm-opt` to remove (verified —
`--strip-debug --strip-producers` on the shipped artifact *grows* it by 5 KiB).
The data section is unchanged from Phase 6 (526 KB of property tables + 122 KB
of embedded component library). So the +874 KiB is the CAS's bignum arithmetic,
the factoriser, and the control suite's linear algebra.

**Neither budget debt in `.github/workflows/ci.yml` moved.** The property tables
are still linked rather than fetched, and the engine is still one chunk. That
file says, in terms: *"Do not raise this again without doing one of them
first."* This pass did neither, so **the budget was not raised** and the wasm CI
job is red.

### The measured lever, not taken

`[profile.release-wasm]` already exists in `Cargo.toml`, with the comment
*"Optimise for size where it does not cost solver throughput"*, and it already
sets `opt-level = "s"`. It is **inert** — `wasm-pack 0.13.1` can select only
`--dev`/`--profiling`/`--release`, so nothing ever builds with it.

Measured by temporarily setting `opt-level = "s"` on `[profile.release]` and
rebuilding (Cargo.toml restored afterwards; the working tree is unchanged):

| build | raw | gzipped | vs 3072 KiB budget |
|---|---|---|---|
| `opt-level = 3` (what ships today) | 3336 KiB | 1497 KiB | **108.6 % — over** |
| `opt-level = "s"` | **2797 KiB** | **1326 KiB** | **91.0 % — under** |

That is a **539 KiB** saving and it clears the budget outright, without touching
either recorded debt. It was **not applied**, for one honest reason: the
throughput cost was not measured, and `opt-level = "s"` on `[profile.release]`
would also slow the native parity replay and the CLI. Wiring it for wasm only
needs either `RUSTFLAGS="-C opt-level=s"` on the wasm-pack invocation or a newer
wasm-pack that accepts `--profile release-wasm`. **Decide this deliberately** —
it is a product trade (bundle size against in-browser solve speed), not a
cleanup.

---

## What Phase 9 did **not** deliver — ranked

1. **`Integrate` is a closed pattern table, not an integrator.** This is the
   phase's known soft spot and PLAN.md §5 scoped it that way on purpose
   (*"Only symbolic `Integrate` is genuinely hard (Risch); scope it to a
   pattern-matched table for v1 and record the gap"*). The **exact** boundary,
   transcribed from `cas/ops.rs::integrate`:

   | Handled | Result |
   |---|---|
   | integrand free of `v` | `e*v` |
   | `c*v^m`, `m` integer | `c*v^(m+1)/(m+1)`, or `c*ln(v)` at `m = -1` |
   | `(a*v+b)^n`, `n` integer, `b ≠ 0`, `a` rational | `(a*v+b)^(n+1)/(a*(n+1))` |
   | `N/(a*v+b)`, `N` and `a` constant | `N/a*ln(a*v+b)` |
   | `N/D` where `D' = k*N` | `ln(D)/k` |
   | `p*F(u)`, `F ∈ {exp, sin, cos}`, `p = k*u'` | `k·∫F` at `u` |
   | `exp`/`sin`/`cos` of `a*v+b` with symbolic `a` | `∫F(u)/a` |
   | `Sin(v)^n*Cos(v)`, `Cos(v)^n*Sin(v)` | `Sin^(n+1)/(n+1)`, `-Cos^(n+1)/(n+1)` |
   | polynomial in `v` over ℚ | term-by-term power rule |

   **Refused by name — Symja finds every one of these:** `exp(x^2)`,
   `sin(x)/x`, `tan(x)`, `ln(x)`, `sqrt(x)`, `1/(x^2+1)`, `x^x`, `exp(x)/x`,
   `ln(ln(x))`, and **anything needing integration by parts**. Note the sharp
   edge a user hits immediately: `∫1/x dx` works and `∫sin(x)·cos(x) dx` works,
   but `∫1/(x²+1) dx` and `∫ln(x) dx` do not. Nine of those refusals are pinned
   by `integrate_names_what_it_cannot_do`, which also pins five inside-the-table
   cases so the boundary cannot drift in either direction.

2. **`balreal` returns a balanced realisation in the opposite state-sign
   convention from the oracle**, which is why `estimator-gramian-balreal` is
   still pending and why it is a Phase 9 debt rather than a Phase 7 one.
   *(Closed 2026-08-21, Wave A2: `linalg::svd` became a transcription of
   Commons Math's own JAMA-derived algorithm — signs included — and the
   fixture promoted with the five other sign holds; ledger item 24.)*
   Diagnosed here from the golden: 31 of 35 variables match to better than
   `1e-9` — the Kalman gain `L` and both gramians included — and the four that
   do not are exactly `Ab[1,2]`, `Ab[2,1]`, `Bb[2,1]`, `Cb[1,2]`, each with the
   right magnitude and the wrong sign. That pattern is the signature of a joint
   sign flip on column 2 of the SVD's `U` and `V` inside
   `T = Lc·V·S^{-1/2}` / `T⁻¹ = S^{-1/2}·Uᵀ·Loᵀ`. Singular vectors are only
   determined up to that sign, so neither engine is wrong — but the numbers
   differ, and matching would mean inferring Commons Math's sign output from a
   single data point, which this repo's parity rule forbids. What *is* asserted
   is that the realisation is genuinely balanced (`Wc = Wo = diag(σ)`) and that
   the transfer function survives the change of basis
   (`balreal_is_internally_balanced_even_though_its_state_signs_differ`).
   **Whoever picks this up needs the Java oracle, not more Rust:** dump
   `balreal` for two or three more systems and see whether Commons Math's
   convention is statable (e.g. "first non-zero component of each `u_i` is
   positive"). If it is, one normalisation pass in `linalg::svd` closes both
   this and the fixture.

3. **The bundle is 264 KiB over budget** and the fix is known, measured and
   deliberately not applied. See above.

4. **`Factor` silently declines above `ops::MAX_POW = 64`.** `factor(x^100 + 1)`
   returns `1+x^100` in **0.000 s** — not wrong, but not factored either,
   because `x^100` interned as one opaque generator. `factor(x^50 + 1)` at the
   same moment returns a genuine 51-character factorisation. There is no
   diagnostic distinguishing "irreducible" from "above the exponent ceiling",
   and a user cannot tell which they got. `Apart` has the same shape at
   `MAX_APART_DEGREE = 64` (it returns the input unchanged). Both need a
   "declined, not proved" channel.

5. **`ss2tf` at 201 states costs 5.0 s.** The slowest single operation measured
   anywhere in Phase 9, and there is no ceiling on the state count reaching it.
   A 200-block chain is an unusual document but not an adversarial one.

6. **Spelling is mixed and unverified against the oracle for the trigonometric
   generators.** `integrate(sin(x)*cos(x), x)` returns `Sin(x)^2/2` (capital),
   while `integrate(1/x, x)` returns `ln(x)` (lowercase). Both are deliberate —
   generator names are Symja's printed forms so the alphabetical ordering agrees
   (`Cos(x)` before `Sin(x)`), while `ops::display` emits `ln(`/`log10(`
   directly in place of `SymjaOutputNormalizer` — but the *combination* was not
   checked against a golden dump of the REPL, because CAS ops are REPL-only in
   the Java and the golden dumper drives documents. The `SYMBOLIC` path *is*
   covered (`partial-fractions` is a promoted fixture); the printed REPL strings
   are not.

7. **`analysis/` is still mostly unreachable.** Unchanged from
   [`status-phase78.md`](status-phase78.md): the optimizer, NSGA-II, curve
   fitter, Monte Carlo, parameter fit, all-roots solver and parametric sweep
   driver are still absent from the boundary. Phase 9 did not touch this, and
   the PID Tuner / Min-Max / Curve Fit buttons in the shipped UI still have no
   engine behind them.

8. **The REPL still refuses four of its seven arms by name.** `CALL` lines,
   matrix/vector literals, range vectors and single-unknown equation solves
   print a refusal. `ReplTerminal.tsx`'s own `help` text still advertises all
   seven.

9. **`ReplDimensions.dimensionOf` is not ported**, so a computed REPL expression
   reports its **SI** value with no unit where the Java reports the display
   value and unit. A bare variable echo is exact; `2*T_1` with `T_1` in `[C]` is
   not. Recorded in `crates/frees/src/repl.rs`'s header.

10. **No CAS operation is covered by a golden-dumper fixture at the REPL level.**
    Everything in `cas/` is verified by unit tests written against the Java's
    documented behaviour plus the `SYMBOLIC` document path. Reaching
    `ReplEvaluator` from the dumper would need a REPL harness the dumper does
    not have.

---

## Divergences opened by this pass

Recorded in the ledger at
[`status-phase1.md`](status-phase1.md#opened-by-the-phase-9-robustness-pass-2026-08-01)
as items 24–25.
