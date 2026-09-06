# Wave-3 F7 — robustness and performance under the rustprop backend

Wave 2 replaced the whole property layer: decision
[D9](decisions/0009-rustprop-backend.md) made **rustprop** the engine's only
in-bundle real-fluid backend, the linked `(P,h)` / `FRAUX1` artifacts left the
wasm, and a warm-state adapter and a humid-air grade landed on top. This
document is the "does it still hold up under load and abuse" pass over that
change. **No engine line changed in this task.** What changed is two
measurement `println!`s, one test assertion that contradicted its own doc
comment, and this file.

---

## How to read the numbers

**The box was shared for every measurement below.** Two other agent lanes were
building and running on the same 8-core machine throughout: a rustprop lane
(seven `golden-gen` Python workers plus a `validity_scan`) and a second frees
worktree. Load average is quoted with **every** number, and every
backend-to-backend comparison was run **back-to-back and alternating** so the
ratio survives the contention even where the absolute does not.

Where a number is quoted alone, treat it as an upper bound on a quiet box, not
as the quiet-box value.

Toolchain: `cargo 1.97.1`, `--release` (`opt-level = "s"`, fat LTO,
`codegen-units = 1`). rustprop at `2db1df7` (Wave-2 R8 + its docs commit).

### Re-measured at Wave-3 integration on a quiet box

The instruction above — treat a lone number as an upper bound — was taken up at
integration. Every headline figure was re-run at **load 1.6–2.2** on the same
8-core box (F7 measured at 6.6–17.9), against the merged tree: all five frees
branches in, and rustprop advanced from `2db1df7` to the Wave-3 merge (R11's
validity gates + R9's sweep). The corpus is **719** documents here, not 707 —
F6 and F8 promoted twelve.

| figure | F7, under load | integration, quiet | change |
|---|---|---|---|
| parity replay, wall | 43.44 s best (load 6.59); 43.4–72.3 s | **37.06 / 36.98 / 38.36 s** (load 2.2 / 1.8 / 1.7) | ~1.2x faster on **12 more** documents |
| hostile `props_si` sweep, slowest call | 377.66 ms (load 8.07) | **119.67 ms** (load 1.9) | 3.2x smaller; same call; 16.7x inside the 2 s budget |
| `all_survive`, worst document | 155.09 ms (load 8.07) | **63.49 ms** (load 1.9) | 2.4x smaller; same document; 315x inside the 20 s ceiling |
| warm `T(P,Hmass)`, Water | 29.6 us warm / 154.5 us cold = **5.22x** (load 13.8) | **12.9 / 67.2 = 5.22x**; 12.9 / 67.1 = 5.21x; 13.1 / 70.2 = 5.35x (load 1.6) | absolutes **2.3x** smaller, **ratio identical** |

That last row is the cleanest statement of this document's own thesis: under a
2.3x change in absolute cost the ratio moved by 0.00–0.13x. Quote ratios; quote
absolutes only with their load. It also retires §8's thin-margin warning for
now — Water's warm median is 12.9 us of the 50 us budget on a quiet box,
against the 40.3 us it reached at load ~10.4.

**One figure moved the wrong way, and load is not the reason.** The two-phase
plateau query reads **2.81 ms** in the full suite here and **1.86–1.99 ms** run
in isolation at load 1.00, against F7's 500.522 us. Isolation makes it
*slower*, which rules out contention and points at one-time initialisation:
F7's reading came after other tests had already built the shared fluid registry
and superancillary tables, and a cold reading pays for them. So this number
measures test ordering, not throughput. Both readings sit ~700–1000x inside the
`< 2 s` assertion F7 added, which is the property that actually matters.

---

## 0. The gate, raw

Every command exit 0, on the final tree.

| Gate | Result | Wall | Load |
|---|---|---:|---:|
| `cargo fmt --all --check` | clean | 4 s | 6.0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean | 37 s | 7.3 |
| `cargo clippy -p frees-core --features rustprop-backend --all-targets -- -D warnings` | clean | 26 s | 6.1 |
| the same two `--target wasm32-unknown-unknown` | clean | 49 s / 29 s | 11.9 / 10.9 |
| `cargo test --release --workspace` | **3131 passed, 0 failed, 7 ignored** (29 suites) | 194 s | 8.6 |
| — of which `golden_corpus_parity` | 707 fixtures, 10 at a declared tolerance, 0 at a stop-criterion floor | 77.10 s | 8.6 |
| `cargo test --release -p frees-core --features rustprop-backend` | **3012 passed, 0 failed, 6 ignored** | 520 s | 13.8 |
| release `rustprop_warm` | 8 passed | 1 s | 13.8 |
| release `rustprop_warm_calibration` | 1 passed, 1 ignored (the `--ignored` table) | 5 s | 13.2 |
| release `humidair_grading` | 1 passed — 912 points, median 0, worst 4.211e-3 at the known freezing-wet-bulb point | 1 s | 13.1 |
| `wasm-pack build … --release --target web` + the ci.yml budget step | **2715 KiB raw / 1115 KiB gzipped**, 88 % of the 3072 KiB budget, 357 KiB headroom | 112 s | — |
| frees-core links no wasm-bindgen | `cargo tree -p frees-core --features rustprop-backend -e normal`, native **and** `--target wasm32-unknown-unknown`: **0** occurrences (`frees` has 9, as it must) | — | — |

The `rustprop_warm` cost line on that green run, for the record:

```
Water: warm T(P,Hmass) median 29.6 us, cold median 154.5 us (5.22x, floor 3x)
Air:   warm T(P,Hmass) median 11.9 us, cold median  12.7 us (1.07x, floor not asserted)
```

`cargo test --workspace` is run here in **release**, which is what `CLAUDE.md`
documents locally (the replay solves 707 documents); CI runs it in debug.
`dynamics_robustness` was additionally run in **debug**, per `CLAUDE.md`'s note
that the stack-overflow defect it once found only reproduces unoptimised:
`cargo test --workspace --test dynamics_robustness` — **42 passed, 0 failed**,
46.8 s at load 8.6.

One thing worth writing down because it is not obvious: **`cargo test
--workspace` already exercises the rustprop backend.** `frees` depends on
`frees-core` with `rustprop-backend`, and resolver-2 unifies features across
selected workspace members, so the `frees-core` test binaries in a `--workspace`
build have the feature on — which is why `humidair_grading` reports `1 passed`
there rather than being compiled away, and why the `--workspace` parity binary
grades against `tolerances-rustprop.json`. The table backend is only reached by
`cargo test -p frees-core` on its own.

---

## 1. Parity wall-clock: ~180 s → 43–72 s, and rustprop is not why

The anchor is in `Cargo.toml`: the **704**-document replay measured **179.95 s**
at `opt-level = "s"` on 2026-08-06, with the `(P,h)` table backend. Today's
corpus is **707** documents.

Six alternating release runs of `golden_corpus_parity`, prebuilt test binaries
invoked directly so no cargo rebuild pollutes the clock:

| pass | rustprop wall / CPU | table wall / CPU | load before | ratio |
|---|---|---|---|---|
| warm-up | 66.83 / 65.24 s | — | 7.16 | — |
| 1 | 72.27 / 65.03 s | 54.28 / 53.80 s | 7.59 → 8.45 | 1.33 |
| 2 | 43.44 / 43.34 s | 44.52 / 44.38 s | 6.59 → 4.79 | 0.98 |
| 3 | 45.44 / 45.25 s | 55.77 / 54.63 s | 4.82 → 4.30 | 0.81 |
| 4 | 58.12 / 56.66 s | 58.90 / 57.72 s | 4.71 → 6.57 | 0.99 |
| 5 | 128.36 / 90.67 s | 72.46 / 67.53 s | 7.29 → **17.73** | *discarded — load spike mid-pair* |

**Best observed: 43.44 s wall / 43.34 s CPU (rustprop, load 6.59).** For
reference the same replay inside a full `cargo test --release --workspace`
under load ~16 took **142.80 s** — which is what a contended absolute looks
like, and why the table above alternates.

Two honest conclusions:

* **The replay really is 2.5–4x under its ~180 s anchor.** CPU time, which is
  the load-insensitive half of the measurement, is 43–65 s against 180 s of
  wall clock on a machine whose state is not recorded.
* **The property backend is not the reason.** The `(P,h)` table backend
  measures the same today (44.5–58.9 s over the clean pairs); the median of the
  four clean paired ratios is **0.99**. Whatever bought the 3x happened
  engine-side between 2026-08-06 and now — `d935791` ("Give Scope a
  non-cryptographic hasher: EV transient 57.5 s → 41.0 s") is the largest
  single candidate in the log — plus an unknown amount of box difference.

The spread *within* one backend (43.4 → 72.3 s) is larger than the difference
*between* backends. Do not quote a single parity second without its load.

---

## 2. The per-call budgets: every one met, with room

### The 9,216-call hostile sweep

`props_robustness::the_installed_backend_answers_or_errors_for_every_key_combination`
— 18 outputs × 8 input keys × 8 input keys × 8 values, every combination pushed
at `props_si(…, "Water")` through the installed backend.

```
hostile props_si sweep: 9216 calls, slowest 377.660112ms
  on props_si(Dmass, Hmass=0, Smass=101325, Water)
```

Budget is **2 s per call**; the worst call is **377.7 ms**, a **5.3x** margin,
measured at load 8.07 with the rustprop backend. The slowest call is the one
you would predict: a `(Hmass, Smass)` flash at a degenerate enthalpy, which is
the deepest iterative path rustprop owns.

The count and the worst call are now **printed**, not just asserted — the
assertion alone could not tell "comfortably inside" from "one regression away".

### The two-phase plateau

`an_inverse_lookup_on_a_two_phase_plateau_is_refused_rather_than_guessed`, the
`Enthalpy(Water, P=101325, T=373.1243)` query that sits dead centre of the
saturation plateau:

```
plateau (P,T) on the saturation line answered in 500.522µs: Err
```

**500.5 µs**, and it takes the `Err` arm — rustprop reproduces upstream
CoolProp's own guard, which rejects a `(P,T)` flash whose pressure is within
1e-4 % of `p_sat(T)`. The `Ok` arm of that test is therefore dead under this
backend; it is kept because a future backend that *can* resolve the plateau
must still land inside the dome. A `< 2 s` assertion was added around the call:
the failure a plateau invites is a bracketed inverse that never terminates, and
"it refused" is only half the contract.

### `all_survive` — the 20 s hang detector

Both hostile-corpus helpers now print their worst document instead of only
asserting it. Worst per corpus, rustprop backend, release:

| suite | corpus | worst document | worst |
|---|---:|---|---:|
| props_robustness | 272 | `x = Temperature(Water, P=101325, v=0.001)` | **155.09 ms** |
| props_robustness | 496 | `x = flametemp('CH4', 1e-300, 0)` | 13.74 ms |
| props_robustness | 50 | `x = WetBulb(AirH2O, T=300, P=101325, R=0)` | 11.89 ms |
| props_robustness | 4 | `T = 300 / x = Enthalpy(Water, P=P, T=T) / P = x / 1000` | 9.85 ms |
| props_robustness | 2250 | `x = mach_prandtlmeyer(1e300, 1e300)` | 1.25 ms |
| component_robustness | 590 | (nonsense string parameter sweep) | **8.63 ms** |

The ceiling is **20 s per document**. The worst document in the whole property
surface is **155 ms** — a **129x** margin — and that at load 8.07. Whole-file
wall clock: `props_robustness` 163.7 s for 20 tests, `component_robustness`
3.39 s for 58.

---

## 3. Fuzz: 64–128x the CI case count, clean

`fuzz_properties`, release, rustprop backend. The file's own budget is "~a
minute in CI at the default counts".

| cases | wall | result | load |
|---|---:|---|---:|
| default (512/512/256/256/512) | 0.75 s | 5 passed | 16.6 |
| `PROPTEST_CASES=8192` | 14.97 s | 5 passed | 16.7 |
| `PROPTEST_CASES=32768` | 62.05 s | 5 passed | 17.3 |

No panic, no abort, no hang, and `solve_is_deterministic` holds bitwise at
every case count — the backend swap introduced no hidden global state that a
second identical solve can observe. (The warm-state adapter *does* carry a
process-global seed cache; this is the property that says it cannot change an
answer.)

---

## 4. Benches: the property-bound one costs ~1.35x, the rest cost nothing

`solve_bench`, criterion, `--warm-up-time 2 --measurement-time 5`, load
6.8–11.6, rustprop and table binaries run back-to-back:

| bench | rustprop | table |
|---|---:|---:|
| `scalar_two_block` | 37.6 – 42.2 µs | 41.1 µs |
| `rankine_cycle` | 715 – 1170 µs | 443 µs |
| `component_mvem` | 397 – 402 µs | 391 µs |
| `transient_dyn` | 28.2 – 29.7 ms | 27.5 ms |
| `control_lqr` | 1.64 – 1.74 ms | 1.57 ms |

`rankine_cycle` is the only one with a real-fluid backend in the Newton loop,
so it is the only one where the backends can differ. Three focused alternating
passes at load 7.0–7.9:

```
pass 1  rustprop 857.11 µs   table 467.29 µs   1.83x
pass 2  rustprop 718.92 µs   table 533.87 µs   1.35x
pass 3  rustprop 730.08 µs   table 588.07 µs   1.24x
```

**~1.35x median** for bit-exact CoolProp 8.0.0 values in place of a table
interpolation whose own error is 1e-7…1e-4 (the trade D9 made deliberately).
Everything else is inside the noise.

---

## 5. `ev-thermal-management` is no longer the slow one

It was named as the slowest document in the corpus and its parity tolerance
improved 178x in Wave 2 without being retired, so it was timed on its own.

Alternating, load 8.40, wall including ~10 ms of process start:

```
rustprop 57 / 35 / 28 ms      table 57 / 40 / 36 ms
```

**~30 ms, and identical between backends.** The solve is 169 blocks over 229
variables, 216 Newton iterations, max residual 1.69e-8. Whatever made it
expensive before, it is not expensive now, and the 178x tolerance improvement
did not cost throughput.

The actual slowest document in the corpus today, found by timing all 707
through the CLI:

| document | rustprop | table | load |
|---|---:|---:|---:|
| `odelib-p20-stiff-reaction-chain-ode15s` | 19.17 / 17.20 s | 21.95 / 18.53 s | 7.1 – 8.0 |
| `av_stor_two_masses_dynamic` | 1.54 – 1.65 s | 1.50 – 1.56 s | 8.3 |
| `av_mol_rosenbrock_rod` | 1.15 – 1.18 s | 1.15 – 1.40 s | 8.6 |
| `av_unc_with_integral` | 0.93 – 1.08 s | 0.86 – 0.91 s | 8.8 |

`odelib-p20-stiff-reaction-chain-ode15s` is a ten-line `ode15s` DYNAMIC over a
stiff three-species chain (`k1 = 1000`, `rtol = 1e-7`, 150 output points) with
**no property call in it at all** — so it is backend-independent (the table
backend is if anything slower), and it is roughly **a third of the entire
707-document replay's wall clock on its own**. That is the largest single
performance item left in the corpus and it belongs to the ODE layer, not to
this wave.

A first, non-alternating pass over the corpus appeared to show 2–3x
regressions on `av_mol_rosenbrock_rod`, `av_stor_two_masses_dynamic` and
`av_unc_with_integral`. Re-measured back-to-back they are all within ±20 %, and
two of the three are *faster* under rustprop. That pass is recorded here as the
worked example of why an absolute taken on this box is not evidence.

---

## 6. Audit: the `nominal_enthalpy` seeding does not run

The brief flagged the now-live Phase-A seeding, with `block_count` exactness as
the tripwire. **The finding is that `nominal_enthalpy` is never called at all.**

### What was measured

`engine.rs::seed_consistent_enthalpy` was temporarily instrumented behind an
env var and the **whole 707-document corpus** was solved through
`frees-cli --features rustprop-backend`:

```
documents                      = 707
seed_consistent_enthalpy hits  =  28
needs_seed = true              =   0
nominal_enthalpy calls         =   0
```

All 28 hits are `present=true guess=Some(100000.0)`, across four fluids
(`r134a` ×14, `eg50` ×9, `water` ×4, `r1234yf` ×1). The instrumentation was
removed; the engine is byte-identical to `rustprop-backend@066a038`.

### Why

`seed_property_argument_guesses` runs four passes in order, and pass 2
(`seed_prop_args_in`) already assigns every `h`-indicator argument the generic
`PROP_ARG_NOMINAL["h"] = 1.0e5`. Pass 4 (`seed_consistent_enthalpy`) then asks
`needs_seed`, which is `spec.guess == DEFAULT_GUESS` — and the guess is now
`1e5`, not `1.0`. So the fluid-consistent enthalpy is never computed. Even
without the `needs_seed` short-circuit the result would be discarded, because
`apply_nominal_guess` returns early on `spec.guess != DEFAULT_GUESS`.

**This is faithful to the oracle**, which is why it is reported and not fixed.
`EquationSystemSolver.java` has the identical ordering
(`seedPropArgsIn` at line 803, `seedConsistentEnthalpy` at line 811) and the
identical `applyNominalGuess` early-return at line 989, so the Java computes
`nominalEnthalpy` and throws the answer away. The Rust's `needs_seed` gate is
behaviour-identical and merely skips paying for it. Changing the order would
change initial guesses at 28 call sites and is a parity decision, not a
robustness fix.

What this does mean: the two doc comments that credit
`seed_consistent_enthalpy` with fixing the closed-loop cold-start NaN
(`engine.rs` around the `Missing` enum and around `seed_consistent_enthalpy`
itself) are crediting the wrong pass. The generic `1e5` enthalpy seed and the
fluid-aware **pressure** seed are what actually run.

### What *is* live and backend-dependent

`seed_refrigerant_pressure` → `nominal_pressure` → `props1_si(fluid, "Pcrit")`,
and it now reads rustprop's **numerical** critical pressure:

```
p_seed(R134a)   = 1420746.7308268733 = 0.35 x 4059276.3737910665
p_seed(R1234yf) = 1184530.7934481383 = 0.35 x 3384373.6955661094
```

Both were confirmed against `P_crit(<fluid>)` through the CLI. Under the table
backend the same seed came from the artifact's metadata, so the refrigerant
pressure seed moved by ~1e-6 relative when D9 landed — visible, harmless, and
recorded here because it is the one place the backend swap really does change
an initial guess.

### The `block_count` tripwire

Checked four ways, all clean:

1. **Ordering.** `block_system` runs *before* any seeding at both call sites —
   `engine.rs:676` before `:693` in `solve_with`, `:3436` before `:3444` in
   `solve_equation_list`. `check` (`:1393`) blocks and never seeds at all. A
   guess cannot reach the blocker.
2. **`Missing::Skip` at document level.** The document-level pass cannot insert
   a spec, and `specs.keys()` is what feeds
   `unknown_count: surfaced_count(...)` at `:751`. `solve_equation_list` uses
   `Missing::Create`, but on a `specs.clone()` that dies with the call and is
   read only through `initial_guess` for names already in `unknowns(...)`.
3. **The goldens.** `tests/parity.rs` compares `block_count` **exactly** for all
   **707** fixtures — **10,379 blocks**, 547 fixtures with a non-zero count,
   `root-locus-analysis` the largest at 917. Green.
4. **Both backends.** The same 707 exact block counts hold under the rustprop
   build *and* the `(P,h)` table build, so the blocking is backend-independent
   by measurement and not only by argument.

And, trivially, a call that never happens cannot perturb a block count.

---

## 7. The one defect this sweep found, and it is in a test

`rustprop_warm::warm_t_of_p_hmass_costs_tens_of_microseconds` asserts a
per-fluid cold/warm speed-up floor. F4 set Air's floor to `1.0` and its own doc
comment described that as "**NOT ASSERTED**". A `1.0` floor *is* an assertion —
`median * 1.0 <= cold` — and Air's measured ratio is 1.1x, so the band shipped
with 10 % of headroom where every other band in the file carries ~1.7x.

The load sweep caught it on the first full run:

| run | conditions | warm | cold | ratio | verdict |
|---|---|---:|---:|---:|---|
| `cargo test --release --workspace` | load ~16, two other lanes building | 12.8 µs | 8.5 µs | **0.66** | **FAILED** |
| standalone ×5 | load 10.4 | 11.4 – 15.2 µs | 12.3 – 16.6 µs | 1.08 – 1.10 | passed |

Nothing regressed — 1.08–1.10x is exactly F4's measured 1.1x. The assertion was
simply the wrong shape. Air's floor is now `Option<f64> = None`: the ratio is
**printed on every run and asserted on none**, which is what the doc comment
already claimed. Water's `3.0` floor is untouched (measured 4.7–5.0x under the
same load) and so is the 50 µs absolute warm budget.

**No budget was loosened to make a run pass.** For the record, the one budget
that is now thin is that 50 µs: Water's warm median reached **40.3 µs** at load
10.4, against 11.8–13.5 µs on F4's quiet box. It is a load artifact, it was
left alone, and it is written down here so the next overrun is not a surprise.

---

## What this task did **not** do

* **It did not run the `web` CI job** (`npm ci` + vitest + `npm run build`).
  Nothing outside `crates/frees-core/tests/` and `docs/` changed, so the
  frontend cannot be affected — but the job was not executed, and that is a gap
  in the evidence, not a claim about it.
* **It did not measure on a quiet box.** The lowest load seen in six hours was
  4.30. Every absolute here is an upper bound.
* **It did not fix the seeding-comment inaccuracy** in `engine.rs`, because
  editing prose around a faithful port is a change the owner should see
  described (section 6) before it is made.
* **It did not touch `odelib-p20-stiff-reaction-chain-ode15s`**, which is now
  the single largest wall-clock item in the corpus (~1/3 of the replay) and is
  an ODE-layer problem, not a property-layer one.
* **It did not answer the question F4 handed forward** — whether `Air` should
  stay in `served_fluids`/`rustprop_warm` at all now that its adapter buys
  1.1x. This task only stopped that number from failing the suite at random.
