# D9 — rustprop is the wasm build's only property backend, and the tables leave the bundle

**Status:** implemented
**Date:** 2026-08-17
**Implements** [D8](0008-coolprop-wasm.md). **Settles** the bundle question D8
left open. **Supersedes** [D1](0001-property-backend.md)'s *linked* tables for
the browser build only — the fetch seam D1 asked for is what survives.

## Decision

Four parts, and the fourth is what keeps the first three honest.

1. **The wasm bundle ships rustprop as its only in-bundle property backend.**
   `props::tables::install_builtin_once` — the function every public entry point
   calls — installs `RustpropBackend` when the `rustprop-backend` feature is on,
   and the pre-D9 `TableBackend` when it is off. `frees` turns it on.
2. **The linked `.phtab` / `.fraux` data leaves that bundle.** The
   `include_bytes!` calls in `props/tables.rs::packed` now sit behind a
   `linked-tables` Cargo feature, on by default and off in exactly one place:
   `frees` depends on `frees-core` with `default-features = false`.
   `build.rs` skips the deflate entirely when the feature is off, so a stray
   include is a build error rather than 684 KiB quietly returning.
3. **The decoders and the `install_from_bytes` seam stay compiled.** They are
   what is left of D1 in the browser: the offline / speed fallback a host can
   fetch into. Only the *bytes* are conditional — `unpack`,
   `SaturationSplitTable`, `AuxTable` and `install_from_bytes` compile in every
   configuration, and `BUILTIN_TABLES` / `BUILTIN_AUX` become empty slices
   rather than disappearing.
4. **Parity is pinned to one backend.** Wherever the accuracy path is compiled
   the gate grades *it*, through `fixtures/tolerances-rustprop.json`; the table
   configuration keeps `fixtures/tolerances.json` unchanged. The harness reads
   exactly one of the two, chosen by the same `cfg` that chooses the backend, so
   there is no configuration in which the gate grades a backend the product does
   not ship. Note *why* the native gate lands on rustprop without anything
   opting in: `cargo test --workspace` resolves `frees-core`'s features as the
   union over the members being built, and `frees` is one of them. CI's
   `native` and `parity` jobs therefore grade the same backend the browser gets.
   `cargo test -p frees-core` on its own does not unify, and grades the table
   path against `tolerances.json` — which is why that file had to stay correct
   rather than be edited.

   > **The last sentence stopped being true at Wave-3, and Wave-4 F9 made the
   > tree say so.** Once F6/F8 promoted twelve documents no table backend can
   > serve, the single-package form did not grade the table path — it failed
   > on those twelve, and not even legibly (a property error inside Newton
   > becomes a `NaN` residual, so all twelve reported "Newton iteration
   > stalled after 0 iteration(s)"). `tests/parity.rs` now refuses that
   > configuration up front and names the two commands that work. The
   > `tolerances.json` branch is kept, unreachable, next to the reason:
   > the file is still correct about the configuration it describes, and a
   > corpus without those twelve would reach it again. See the Wave-4 F9
   > amendment at the end of this record.

## Context — the question D8 refused to guess at

D8 chose the accuracy path and named the constraint that would decide whether it
could ship:

> **Bundle budget is the gating constraint, and it is already tight.** The wasm
> is 3042.3 KiB against the 3072 KiB budget — **29.7 KiB of headroom**. […]
> Either the budget moves, or the `.phtab` / `.fraux` artefacts it replaces come
> out (~678 KiB of data section today), or the property backend lazy-loads as a
> separate chunk. This has to be settled as part of the integration, not
> discovered by it.

It is settled by the middle option, and the budget does not move. `ci.yml`'s own
comment block has asked twice for exactly this — "Move the 528 KB of linked
property tables onto the fetch seam that already exists" is debt item (1), open
since Phase 6 and unpaid through four phases.

## Measured — `wasm-pack build --release --target web`, the CI command

All four corners, so the two changes can be told apart. **These are the
switch-over numbers, taken on 2026-08-17**; the tree has moved since and the
shipped row was re-measured by the Wave-4 F9 amendment at the end of this
record. The *deltas* between corners are what this section is for, and they
are what survived re-measurement.

| configuration | raw | gzipped | vs. budget |
|---|---:|---:|---:|
| pre-D9: linked tables *reachable*, no rustprop | 3042.3 KiB | 1597.7 KiB | 99.0 % |
| **shipped: rustprop, `linked-tables` off** | **2700.3 KiB** | **1109.5 KiB** | **87.9 %** |
| rustprop, `linked-tables` **on** | 2700.4 KiB | 1109.5 KiB | 87.9 % |
| neither — the engine with no property source | 2358.3 KiB | 912.8 KiB | 76.8 % |

* **the tables cost 683.9 KiB raw / 684.8 KiB gzipped.** Those two numbers being
  equal is the point D7 made and `build.rs` acts on: the artifacts are already
  deflated on the way in, so the browser's own gzip finds nothing left.
* **rustprop costs 342.0 KiB raw / 196.7 KiB gzipped** — well under the ≤545 KiB
  the integration was budgeted for, and it *compresses*, because it is code
  rather than `f32` grids.
* **net: −341.9 KiB raw, −488.2 KiB gzipped.** Headroom goes from 29.7 KiB to
  371.7 KiB. The gzipped column is what a browser downloads, and it is the
  bigger win: a strictly more accurate engine that is 31 % smaller on the wire.

### The row that changes what part 2 of this decision is *for*

Rows two and three differ by **one byte**. Feature-gating the `include_bytes!`
calls is not what removes the 684 KiB — **installing rustprop is**. The moment
`install_builtin_once` stops calling `install_builtin`, nothing in the wasm
crate's reachable graph touches `BUILTIN_TABLES`, and fat LTO drops the data
section on its own.

That does not make `linked-tables` redundant; it makes it a **guard rather than
a saving**, and the guard is the load-bearing half:

* Part 3 of this decision deliberately keeps `install_from_bytes` compiled as
  the fetched fallback. That function calls `builtin_tables()` and
  `builtin_aux()`. One `#[wasm_bindgen]` export wired to it — the obvious next
  step for anyone implementing the offline path — makes the whole 684 KiB
  reachable again, and the only symptom would be the budget step going red with
  no line of the diff explaining why.
* `build.rs` skips the deflate when the feature is off, so OUT_DIR is empty and
  an `include_bytes!` that escaped its `cfg` is a build failure, not a silent
  regression.

A saving that depends on the optimiser's reachability analysis is a saving that
one innocuous call re-spends. The feature makes it structural.

## What it clears

**Twelve of the 23 entries in `fixtures/tolerances.json` are dead.** They existed
because D1's tables sit 1e-7…1e-4 from CoolProp; rustprop *is* CoolProp 8.0.0, so
that error is gone. Measured on the switch-over run:

> **Thirteen, as of Wave-2.** `sysdesign-ex16-moving-boundary-evaporator` is the
> thirteenth: at this change it kept a numeric entry and gained a
> `solver_floor` one, and both died when rustprop's TOMS748 landed (see the
> RESOLVED note under Finding 1). `tolerances.json` still has 23 entries;
> `tolerances-rustprop.json` has 10.

| fixture | table backend | rustprop |
|---|---:|---:|
| `sysdesign-ex11-liquid-cooling-loop` | 1.3e-4 | **0** |
| `sysdesign-ex17-ac-expansion-valve` | 7.9e-8 | 1.2e-16 |
| `components_wave2_flash_tank` | 5.3e-8 | 4.1e-16 |
| `components_wave2_ejector_oilsep` | 5.1e-8 | 2.7e-15 |
| `sysdesign-ex20-zeotropic-blend` | 2.9e-8 | 1.3e-15 |
| `components_family_fluid` | 1.4e-6 | 4.7e-14 |
| `rankine-cycle`, `rankine-cycle-2` | 6.4e-7 | 7.7e-14 |
| `components_family_liquid` | 1.2e-6 | 7.9e-14 |
| `components_wave8_hydro_turbine` | 1.6e-6 | 3.9e-13 |
| `components_bsweep_valve_characteristics` | 1.6e-6 | 4.2e-13 |
| `state-tables-multifluid` | 2.9e-6 | 4.4e-11 |

(eleven rows, twelve entries — `rankine-cycle` and `rankine-cycle-2` share one.)

`ev-thermal-management`, the loosest entry in the file at 2e-3, is not retired
but improves **178×** (8.95e-4 → 5.0e-6): its worst variables were a glycol
film coefficient off the `INCOMP::MEG` `FRAUX1` grid, amplified by the
Re = 2987 laminar/turbulent blend, and rustprop answers `INCOMP::MEG` exactly.

Eleven entries survive, and **not one of them is this port's error any more** —
the reasons in `fixtures/tolerances-rustprop.json` name the mechanism per
fixture. Ten are the *golden* side: `props/PropertyFunctions.java` answers output
`T`/`Dmass`/`Smass` at the input pair `(P, Hmass)` from its own run-time
256/96/48 table, gated at 1e-4, so those goldens are not CoolProp values and the
gap is the Java's interpolation error. `components_wave2_liquid_tms` shows it
cleanly: the document imposes 400 K at a source, converts to `h`, and inverts —
this port returns 399.999_999_91, the Java 400.000_678_12. The eleventh
(`refrigeration-vcr`, 3e-9) is cancellation in a COP.

> **Amended by Wave-3 F5** (below). Two claims in the paragraph above did not
> survive re-measurement. There are **ten** survivors, not eleven — the
> `solver_floor` entry died at Wave-2 integration. And `refrigeration-vcr` is
> **not** cancellation of otherwise bit-exact goldens: its leaf
> `Enthalpy(R134a, P, s)` is itself 1.6e-10 from the wheel, because upstream's
> own flash stopped 4.06e-9 short in pressure. The headline claim — *not one of
> them is this port's error* — held, and F5 proved it per fixture against the
> CoolProp 8.0.0 wheel rather than by inference.

D8 predicted the trap and it was real: the file's own rule is that an entry
passing at the default *fails*, so the dead entries had to go in the same change.
They could not simply be deleted, because the table configuration is still
supported and still needs them — hence two files, one per backend.

## Finding 1: iterative flashes have a noise floor

One fixture, `sysdesign-ex16-moving-boundary-evaporator`, stopped converging.
Its cause is general enough to belong in this record rather than in a comment.

A `(P,h)` table answers `Temperature(fluid, P, h)` from a bilinear surface, so the
residual `T_out − Temperature(P, out.h)` is smooth in `h` and Newton drives it to
the engine's 1e-12 default. rustprop answers the same call with an **iterative
flash**, converged to ~8e-10 relative in `h`. Measured on R134a at 350 kPa near
h = 423 193 J/kg: stepping `h` by 1e-3 J/kg moves `T` by 1.129_662_4e-6 K nine
times out of ten and by 7.531_083e-7 K the tenth. The step is a staircase, not a
slope, and no line search can descend a staircase. The engine's report — "no
full, halved or damped step reduces the residual" — is the truth.

This bites only where the *inverted* variable is unknown **in the same Newton
block**; ex16 is the one document in 707 that does that. The response is a
declared, measured, per-fixture stop-criterion relaxation
(`solver_floor` in the rustprop tolerance file: 1e-10 against a residual
bisected to between 5e-11 and 6e-11), carrying the same two honesty guards as a
numeric tolerance — an entry whose fixture converges at the default fails, and
an entry with no fixture fails. The **values** are still graded, at 1e-8. What
moved is only the point at which the solver stops chasing arithmetic noise.

The engine's global default was deliberately **not** touched: it is the Java
oracle's `SolverSettings.DEFAULTS` value, and loosening it for everybody to
rescue one document would trade a real parity property for a convenience.

> **RESOLVED 2026-08-18 — the noise floor was rustprop's, and rustprop fixed
> it.** This finding was written against a rustprop whose single-phase `(P,X)`
> flash still used a documented 30-bit bisection **stand-in** for upstream's
> solver; the ~8e-10 staircase measured above is that stand-in's granularity,
> not a property of iterative flashes in general. Wave-2 R8 replaced it with
> upstream's own Boost TOMS748 plus a warm-density carry across probes, and
> rustprop's own witness is that the median displacement from the CoolProp
> wheel over 1 433 `(P, caloric)` goldens moved 1.77e-10 → 2.04e-16. At Wave-2
> integration ex16's Newton block reaches the 1e-12 engine default unaided and
> the fixture grades at 4.207_713_303_802_4e-12 — under the 1e-9 default. Both
> of its entries were therefore deleted and the `solver_floor` section is now
> empty; the mechanism it guarded is real, so the section and its harness
> support stay, with no instance. The paragraph above is left as written
> because the reasoning was correct on the evidence available then, and
> because the honesty guards it describes are precisely what made the fix
> announce itself: the parity run failed with "delete the entry rather than
> leaving a dead relaxation in the file".

## Finding 2: the adapter has to enforce `RealFluid`'s contract, not just forward

`props_robustness`'s exhaustive key sweep — every output key × every input pair ×
a hostile value set — found two things a 1:1 forward cannot survive. Both are
rustprop being *correct*, and neither is usable through this seam.

**Non-finite out.** `PropsSI("Hmass", "T", 0, "Smass", 101325, "Water")` returns
`Ok(NaN)`: upstream's behaviour at that degenerate state, faithfully ported. But
`RealFluid`'s doc says an implementation "must decline (`Err`) rather than
extrapolate. A backend that answers outside its valid range is worse than no
backend, because a wrong answer reaches the user as a solved variable" — and a
`NaN` is the worst case, entering the Newton residual as data and converging
nothing while looking like progress.

**Non-finite in — and this one is not a nicety.**
`PropsSI("Dmass", "Smass", NaN, "Hmass", 101325, "Water")` **panics**, on
`assert!(fa * fb <= 0.0)` in rustprop's Chebyshev root finder: a bracketing
assertion is precisely what a `NaN` defeats. The shipped wasm is
`panic = "abort"`, so that is not a recoverable error — it kills the engine
worker and `engineClient` has to respawn it. And the inputs to a property call
are *document* values, which a diverging solve or a `props_si_or_nan` diagram
sweep produces as `NaN` routinely. A user document could have taken the engine
down.

`RustpropBackend` now guards **both** sides on all three calls: non-finite inputs
are refused before rustprop sees them, non-finite answers are refused before the
engine sees them. Both are pinned by tests, and the output test asserts
upstream's `NaN` *first*, so a future rustprop that throws there is a signal
rather than a silent pass.

The general shape is worth carrying into the rest of the integration: **the
adapter is not a pass-through.** Faithfulness to CoolProp is rustprop's contract;
survivability of hostile input from a Newton solver is this crate's, and they are
not the same contract. F1 wrote the forward; the sweep is what found the gap
between them.

## Consequences and open risks

* ~~**GitHub CI cannot build the wasm or web jobs until rustprop is
  published.**~~ **Closed 2026-08-20 by commit `cb1d7be`**, which switched the
  three rustprop dependency lines from the sibling-path form to
  `git = "https://github.com/ernsoylu/RustProp", tag = "v0.1.0"` — the runners
  resolve the tag like any other git dependency and every CI job builds
  without a sibling checkout. What remains open is only the *cosmetic* half:
  when the claimed crates.io names are published (pending the owner's registry
  token), the git/tag keys become a plain version requirement. The original
  text follows for the record: `frees-core` reached rustprop through a `path`
  dependency on a sibling checkout, which F1 made *optional* so the runner
  could still build; D9 made `frees` require it, so the `wasm`, `web` and
  (through feature unification) `native`/`parity` jobs needed `../rustprop`
  present, and until publication the gate was a local gate.
* **The fetch fallback is narrower than the seam suggests, and deliberately not
  widened here.** `install_from_bytes` takes one `FRPHTAB1` artifact and rebuilds
  the backend from the linked floor plus that artifact. With the floor now empty
  in the wasm build, (i) a second call replaces the first fluid instead of adding
  to it, and (ii) the `FRAUX1` transport grids have no runtime install path at
  all, because `FRPHTAB1` is the only format the seam reads. Nothing calls it
  today; a host that actually takes the offline path will need the seam widened
  to accumulate and to dispatch on the artifact magic. Recorded rather than
  speculatively built.
* **`Air` left the property-diagram picker.** `plot_fluids_available` narrows
  `plot_fluids` by the backend's `served_fluids`, and rustprop's list is Water,
  R134a, R1234yf, `INCOMP::MEG`, `INCOMP::MPG` — no `Air`, because rustprop
  serves the pseudo-pure Air only at `(P,T)`/`(Q,T)`/`(P,Q)` and a diagram needs
  full states. `air.fraux` was the only backing Air had, and it left with the
  other artifacts. Air transport and `Z` at `(T,P)` still answer through
  `props_si`; only the picker entry is gone. Restoring it is a rustprop
  pseudo-pure-flash question, not a frees one.
  **Reversed at Wave-2/Wave-3** — that rustprop question got answered. `Air` is
  back on `served_fluids` and back in the picker; see the D6 amendment at the
  end of this record.
* **Two tolerance files is a real cost.** They can drift. What stops it is that
  each is read in exactly one configuration and both configurations are gated,
  so a stale entry in either fails its own build rather than sitting unnoticed.
* **`frees-cli` gained a `rustprop-backend` passthrough feature**, off by
  default. Without it the accuracy path was reachable only from the wasm module
  and from `cargo test`, and a backend nobody can drive by hand is a backend
  nobody can debug: `frees-cli --features rustprop-backend solve doc.frees` is
  how the staircase in Finding 1 was measured. Default-off keeps a checkout
  without the sibling rustprop tree building the CLI.
* **`HAPropsSI` now answers, and this record does not claim the humid-air
  fixtures.** D8 counted 7 pending fixtures blocked on humid air and 3 on an Air
  state table; rustprop implements the first. Promoting those fixtures is
  measurement work on the pending corpus and is deliberately left outside a
  change whose subject is the switch and the bundle.
* **The `linked-tables` default is load-bearing for the test suite.** Every
  suite that grades an artifact (`props::tables`, `props::auxtable`,
  `props_robustness`, and everything reaching `test_with_builtin_tables`) needs
  the bytes, so `--no-default-features` is not a tested configuration. The
  library compiles without them; the test targets do not, and that is on purpose
  — the alternative is `cfg` noise across five modules to support a
  configuration nothing ships. *(Since 2026-08-22, Wave D6, the library-only
  half is pinned: CI's native job runs
  `cargo check -p frees-core --no-default-features`, so "compiles without
  them" can no longer rot silently. The test targets stay excluded, as this
  paragraph intends.)*

## What was explicitly not done

* **The tables were not deleted.** D8 asked for exactly this restraint — "prefer
  selecting a backend over removing one until the size question is answered with
  a measurement". The artifacts, the generators (`tools/table-gen`,
  `tools/aux-gen`), the decoders and `tolerances.json` are all intact, and the
  table configuration is one `--no-default-features` away.
* **The dead tolerance entries were not deleted from `tolerances.json`.** They
  are not dead *there* — they are dead under rustprop. Deleting them would have
  broken the configuration they describe.
* **The engine's stop criterion was not loosened globally**, and rustprop was not
  modified to tighten its flash convergence. Both were on the table for ex16; a
  single declared per-fixture relaxation asserts strictly more than either.
* **No lazy-loading chunk.** D8's third option is unnecessary at 2700 KiB and
  would have to survive the synchronous-solve constraint that
  `props/tables.rs` records against D1. If the budget goes red again it is still
  the largest lever available.

---

## Amendment — Wave-3 D6: the warm adapter's `Air` path is retired

**Status:** implemented
**Date:** 2026-08-18
**Amends** this record's `Air` bullet under *Consequences and open risks*, and
the warm-state adapter Wave-2 F3 added on top of it
(`crates/frees-core/src/props/rustprop_warm.rs`).

Recorded here rather than as `0010-*.md` because it does not decide anything new
— it withdraws half of an existing mechanism whose other half stays exactly as
it was — and because "D6" is this wave's owner-call label, which would collide
with [D6 / `0006-remove-mdf4.md`](0006-remove-mdf4.md) as a file number.

### Decision

`rustprop_warm` no longer claims **pseudo-pure** fluids. It declines them at the
door of `try_props_si`, before either counter moves, and the call reaches
`rustprop::props_si` untouched. The **pure-fluid path is unchanged** — same two
gates, same constants, same cache.

`Air` stays on `RealFluid::served_fluids` and stays in the property-diagram
picker. It is served, just not by the warm path.

### Why — both of F3's justifications moved, one of them all the way

F3 widened the adapter to `Air` for a reason that no longer exists, and kept
water for a reason that shrank but survived.

| | when F3 wrote it | now | verdict |
|---|---|---|---|
| `Air` `(P,Hmass)`/`(P,Smass)` | rustprop: loud `NotImplemented`. The adapter was the **only** way to answer. | rustprop's own pseudo-pure `HSU_P` flash (Wave-2 R6/R7), 5.0 us cold against the adapter's 4.5 us warm | **~1.1x — retire** |
| `Water` and the pure fluids | cold 311-353 us, warm 13-15 us | Wave-2 R8's TOMS748 made cold ~5x faster: cold 60.1-67.2 us, warm 11.8-13.5 us | **~5.2x — keep** |

1.1x does not pay for a hand-rolled locality gate. It pays for even less than
that number suggests, because the gate's *stability* half — the one that makes a
wrong root impossible rather than merely unlikely — cannot function on a
pseudo-pure at all: there is no superancillary, hence no `rho_l(T)`/`rho_v(T)`
to bracket a root with. F3 handled that honestly by claiming only `T > T_crit`
and declining everything below, which for Air means declining below 132.5 K —
so the retired path was a 1.1x speed-up on a partial region of one fluid.

### Verified, not assumed: `Air` is still *correct*, not merely still answering

Deleting the code that computed something is the moment to grade its replacement
against the oracle rather than against itself. Nine `(P, Hmass)` states from
100 K to 800 K and 1 bar to 100 bar, `T` and `Dmass`, against the pinned CoolProp
8.0.0 wheel (`rustprop/tools/golden-gen/.venv`):

**worst relative deviation 1.705e-15 on `T`, 1.695e-15 on `Dmass`** — round-off.
Pinned at 1e-12 in `tests/rustprop_warm.rs::air_p_hmass_matches_the_coolprop_wheel`,
with the wheel's own `T` as the reference rather than the round number the state
was built from (upstream's pseudo-pure flash carries ~1e-9 bracket granularity of
its own — at 150 K it returns 149.999_999_869 — so grading against 150.0 would
grade the wheel, not the port).

A second test, `air_is_served_by_rustprop_and_never_by_the_adapter`, holds the
seam in both directions over six states x two caloric pairs x two input orders x
six outputs: every value is bit-for-bit `rustprop::props_si`'s, the `(P,X)` round
trip lands back on the `(T,P)` temperature, and both adapter counters stay at
zero *even after a deliberate attempt to seed the cache from a neighbouring Air
state* — which is exactly the traffic shape that used to be served warm.

### What was deleted

Not disabled, not left behind a flag: `COLD_MAX_STEPS`, `PSEUDO_T_MARGIN`,
`PSEUDO_T_SEED`, `Start::t_band` and the step clamp it drove, `accept_state`'s
pseudo-pure branch, `cold_serve`'s unseeded ideal-gas Newton, Air's four rows in
the acceptance grid and three in the calibration bases, and the `cold.is_nan()`
escape in the cost assertion that existed only because Air had no cold path to
measure. That is **44 lines of executable code out of `rustprop_warm.rs`**
against 9 added — the `is_pure` predicate and one decline at each of the two
entry points. The module's own line count barely moves (+74/−89) because most of
what replaced the deleted code is the record of why it went.

One consequence is load-bearing and is asserted by construction: past the new
door the superancillary is **unconditional**, so `accept_state` may call
`PtFlash::sat`, which *panics* without one. Both entry points into the module —
`try_props_si` and the `calibration_warm_solve` seam — carry the same decline.

### What did not change

* The pure-fluid gates and their constants. `GATE_LN_P = 0.10`,
  `GATE_DT_REL = 0.01`, `WARM_MAX_STEPS = 4`: re-derived on the Air-free base
  list and **not one rung moved** (pressure still converges fully out to 0.35;
  the caloric axis still breaks between 1e-2 and 2e-2; three steps still fails
  six of the swept states where four fails none). Only the denominators in the
  doc comments changed, 34 -> 28.
* The calibration sweep still runs every build, and still measures the same
  thing: worst accepted deviation 1.141e-13 on `T` and 8.470e-13 on `Dmolar`
  over 672 accepted solves, and nothing outside the gate lies (worst 3.172e-13).
* The warm-vs-cold grid: 30 states instead of 38, every quoted number
  re-measured and unmoved (worst warm 6.063e-14 against cold's 8.441e-10).
* `Water`'s speed-up floor, still 3x against a measured 5.2x.

### Open

`rustprop_warm` now has exactly one customer class — HEOS pure fluids in a
document's Newton loops — and one number justifying it, 5.2x. That number came
down 4x in one wave because rustprop got faster, and rustprop is still getting
faster. The next person to weigh this module should weigh the whole of it, not
another fluid's worth of it.

*Re-weighed 2026-08-22 (Wave D3), against the v0.1.0 git tag the build now
consumes: five standalone release runs of
`tests/rustprop_warm.rs::warm_t_of_p_hmass_costs_tens_of_microseconds` at load
~2.5-3 measured **4.70-5.09x** (warm 39.5-42.5 us, cold 200.1-213.0 us — the
absolutes are ~3x the quiet-box figures, the load; the ratio is the
measurement). The 5.2x band held: no cold-path speed-up landed between the F7
measurement and the tag, so the "keep" verdict stands unchanged, as does the 3x
floor. The number to re-check is this one, whenever the tag is next bumped.*

---

## Amendment — Wave-3 F5: the tolerance list is re-baselined and re-labelled

**Status:** implemented
**Date:** 2026-08-18
**Amends** this record's *What it clears* section and
`fixtures/tolerances-rustprop.json` in full.

Recorded here rather than as a new ADR for the same reason D6 was: it decides
nothing new. It re-measures a number this decision already asserted, and
corrects the two places where that assertion had drifted from what the corpus
actually does.

### The question

D9 adopted rustprop and, in the same change, deleted the twelve tolerance
entries that the accuracy path made dead. It did **not** re-derive the ones that
stayed; it inherited their reasons from D1-era analysis and reasoned about them.
That left an open question worth a task of its own: **of the entries that
survived a perfect CoolProp, which are real and which were artifacts of the old
backend's error masking something else?**

The trap was named in advance: not every entry dies with a perfect port, because
some *goldens are themselves wrong*. Widening a band to accommodate reference
error and widening one to accommodate port error look identical in the file and
mean opposite things.

### What was measured

Every one of the 707 fixtures was replayed and its worst variable recorded, then
each surviving entry was traced to its leaf property call and that call was put
to a **third oracle** — the CoolProp 8.0.0 wheel in
`../rustprop/tools/golden-gen/.venv`, queried at the fixture's own inputs. Three
values per leaf: what the golden says, what rustprop says, what CoolProp says.

The result is unambiguous, and it is stronger than what D9 claimed:

| | entries | leaf: rustprop vs CoolProp 8.0.0 | leaf: golden vs CoolProp 8.0.0 |
|---|---:|---|---|
| Java `(P,h)` table | 9 | 0 … 2.1e-13 (bit-identical in three) | 2.3e-7 … 6.7e-6 |
| upstream flash residual | 1 | 2.9e-15 (self-consistent) | 1.6e-10 |

**No entry in the file is port error.** At every leaf that feeds a surviving
tolerance, rustprop is nearer to CoolProp than the golden is — by four to nine
orders of magnitude, and in three cases bit-identical to it.

The sharpest of these is `props_realfluid_r134a_states`, a flat property matrix
with literal inputs, so **all 42 of its variables** were checked against the
wheel one for one: its **24 table-shape variables** ((P,h) → T/Dmass/Smass) miss
the wheel by 7.8e-10…1.8e-6, while **all 18 non-table variables are
bit-identical to it** — including `Quality` and `IntEnergy` *at the same
(P, h)*, which `PhTableRegistry.TABLE_OUTPUTS` excludes by name. The dividing
line in the data is exactly the dividing line in the Java source.
`props_realfluid_water_states` repeats it over all 49 of its variables with one
extra wrinkle: 27 of its 28 non-table variables are bit-identical and the 28th
is `Volume`, which the Java reports as `1/Dmass` off the tabulated density.

### What changed in the file

* **Nothing was deleted.** The dead-tolerance guard flagged no entry: all ten
  fixtures still exceed the 1e-9 default. (This is the third consecutive change
  to that file in which the guard, not a human, decided what could go.)
* **Every `relative` is now `measured × 1.5`, to two significant figures**,
  replacing hand-picked round numbers. The nine table entries are frozen by
  construction — a committed golden minus a value rustprop reproduces to
  2.2e-13 or better is arithmetic on two constants — so a tight band there costs
  nothing and any movement is a rustprop regression.
* **`mechanism` is now a machine-checked slug**, `oracle-ph-table` or
  `upstream-ps-flash-residual`, defined in a new top-level `mechanisms`
  catalogue, and each `reason` carries the three-way numbers for *its own*
  fixture rather than pointing at a shared prose paragraph.
* **The catalogue is guarded in both directions.** An entry naming an undefined
  slug fails; a defined slug that no entry names *also* fails. That second guard
  exists because this file had already grown a dead explanation — the retired
  stop-criterion mechanism, kept as an empty section with a live-sounding
  description for a whole wave after its last instance converged at the default.
  All three directions were verified by perturbing the file and watching the
  gate go red.

### The one entry that is not the Java's table

`refrigeration-vcr` is the only fixture in the corpus with no table shape
anywhere in it — `P_sat`, `(T,x)`, `(P,x)` and `(P,s)` all fall through to the
native library — so D9 reasoned that its residual had to be cancellation of
bit-exact values in the COP. It is not. `p1`, `p2`, `s1` and `t_evap` *are*
bit-identical, but the leaf `h2s = Enthalpy(R134a, P = p2, s = s1)` is 1.6e-10
away on its own, before any subtraction.

The cause is upstream. A fresh
`AbstractState('HEOS','R134a').update(PSmass_INPUTS, p2, s1)` converges to a
state whose **own pressure is 4.06e-9 relative from the pressure that was
asked for**, and reports `hmass` evaluated there. Ask the wheel for `hmass` at
its own converged `(T, rho)` and it answers 426479.692_669_698_04; ask it
through the flash and it answers 426479.692_738_112. rustprop returns
426479.692_669_696_8 — the first, to 2.9e-15, on a state that reproduces *both*
inputs to ~1e-15.

So the golden is reference-side here too, just tainted by CoolProp rather than
by the Java. But it is the one entry that is **not frozen**, and it points two
ways at once: a port that is bitwise faithful to upstream would reproduce the
residual and this fixture would go to zero. Either direction of travel makes the
entry fail — a regression widens it, a fidelity fix kills it — which is the
correct behaviour for both, and the reason the entry says in writing not to
widen it in place.

Probed across four superheated R134a states the same residual is 4.06e-9,
7.31e-12, 1.16e-16 and 4.37e-16: a stopping criterion, not a systematic offset,
which is why exactly one fixture in 707 shows it.

### The prediction that did not come true

The investigation flagged `props_realfluid_r134a_states` as the entry most
likely to **newly fail** under an accurate backend — 24 of its 42 variables are
table-shape, so a port that stopped hiding behind the old error might overshoot
the band. It does not fail. It measures 1.752_325e-6 against a 2.7e-6 band, and
its worst variable `t_h2s` reads 319.438_780_796_629_9 on rustprop against the
wheel's 319.438_780_796_629_8 — one ulp — while the golden reads
319.439_340_558_266_05. The fixture's entire error is on the golden side, to
five significant figures.

### Open

The nine table entries can only be retired by re-dumping their goldens from a
Java oracle with `PhTableRegistry` disabled. That is a change to
`../frees/backend/core`, not to this repo, and it would delete nine of the ten
remaining accuracy exceptions at a stroke. Whether that is worth doing depends
on whether the Java engine is still a reference this project intends to keep.

## Amendment — Wave-3 F6: the fixtures D8 predicted are promoted

*2026-08-18.* "Consequences and open risks" above says, of the humid-air
fixtures, that *"promoting those fixtures is measurement work on the pending
corpus and is deliberately left outside a change whose subject is the switch and
the bundle."* That measurement is done.

### What was measured

Each pending document in the three groups
[D8](0008-coolprop-wasm.md) named — humid air (7), `(P,T)` transport off the
dome (`hx-correlations-fluid`), and `CompressibilityFactor`
(`thermo-compliance`) — was replayed with `tests/parity.rs`'s own comparison
logic pointed at `fixtures/corpus-pending/golden` and at **no** tolerance file,
so the grade is the corpus default `1e-9` and no per-fixture exception exists to
fall back on. All nine clear:

| Document | Worst variable deviation |
|---|---:|
| `adv_moistair_W_passthrough` | 0 |
| `adv_moistair_dryair_three_way` | 0 |
| `hvac-problem9-air-supply-wet-bulb` | 0 |
| `sysdesign-ex12-moist-air-ahu` | 0 |
| `sysdesign-ex13-humidifier` | 0 |
| `hvac-problem3-psychrometric-balancing` | 1.25e-15 |
| `hvac-problem2-face-and-bypass` | 2.29e-15 |
| `hx-correlations-fluid` | 1.24e-13 |
| `thermo-compliance` | 1.31e-11 |

A `0` is the harness's `rel_diff` returning zero inside its `1e-12` absolute
band — on a psychrometric enthalpy of order 5e4 J/kg that is the last few bits,
not a rounded report. Corpus **707 → 716**, pending **26 → 17**, and
`fixtures/tolerances-rustprop.json` is untouched: nine documents entered the
gate and the file still carries exactly the ten entries F5 re-baselined.

### D8's twelfth, eleventh and tenth also clear

The one group F6 did not promote — the `Air` **state**-table three,
`sysdesign-ex06-pneumatic`, `sysdesign-ex06-pneumatic-2` and
`sysdesign-ex07-pneumatic-servo` — was replayed under the same rules and grades
at 1.43e-14, 0 and 0. D8's prediction was *twelve of the 26*; the measurement
says twelve of twelve. They are staged rather than promoted only because F6's
scope was the nine, and promoting them is a file move plus a table edit
(pending 17 → 14). **Done by Wave-3 F8 — see the second amendment below.**

### What this does *not* close

The remaining twelve holds were re-measured in the same run and **none is a
property blocker**: five `linalg::svd` column-sign documents plus
`estimator-gramian-balreal` (sign-only element flips, relative deviation ~2.0 by
construction), three `eqsys-*` awaiting `CALL eigenvalues`/`eigen`,
`module_inside_for_loop` (pipeline ordering), `pressure-cooker`
(`method = ida`), and `sysdesign-ex01-thermal-network-2` (a signal decaying
through zero, so the relative measure has no denominator).

The `HAPropsSI` deviation D9 already records stands — rustprop returns humid-air
errors as a `Result` where upstream returns `+inf` with a global error string.
Nothing in these nine documents reaches an error state, so the promotion neither
tests nor closes it.

---

## Amendment — Wave-3 F8: `Air` closed, twelve for twelve

*2026-08-18.* The three documents the F6 amendment left staged —
`sysdesign-ex06-pneumatic`, `sysdesign-ex06-pneumatic-2` and
`sysdesign-ex07-pneumatic-servo` — are promoted. Corpus **716 → 719**, pending
**17 → 14**, `fixtures/tolerances-rustprop.json` untouched at ten entries. D8's
count is now exact: twelve of the then-26 pending fixtures cleared by putting a
real CoolProp behind `prop$`.

Replayed the same way (`tests/parity.rs`'s comparison logic, pointed at
`corpus-pending/golden`, tolerance path aimed at a nonexistent file):

| Document | Graded worst | Raw worst relative |
|---|---:|---:|
| `sysdesign-ex06-pneumatic` | 0 | 2.46e-15 (`ori.t_in`) |
| `sysdesign-ex07-pneumatic-servo` | 0 | 2.46e-15 (`sv.t_in`) |
| `sysdesign-ex06-pneumatic-2` | 2.50e-14 | 4.63e-11 (`ode` `vol$in$mdot`) |

The second column is new relative to F6: `parity.rs`'s `rel_diff` returns 0
inside a `1e-12` *absolute* band, and the raw column drops that band. On the two
steady documents the graded 0 is
`Temperature(Air, P, h) = 300.00000000000097` against the Java oracle's
`300.0000000000002` — 13 ulp on 300 K.

`sysdesign-ex06-pneumatic-2`'s graded figure also corrects F6's. F6 reported
1.43e-14, measured over `variables`; that document has exactly **one** variable
(`Pf`) and puts its whole trajectory in a 100x20 `ode_tables` entry the figure
never touched. Swept over the trajectory too, the graded worst is 2.50e-14 and
the raw worst 4.63e-11, on different cells: `cap$port$p` at 652 kPa (absolute
gap 1.6e-8 Pa, outside the band) and `vol$in$mdot` at 6.09e-5 kg/s (absolute gap
2.8e-19 kg/s, well inside it). `parity.rs` compares those cells either way, so
F6's verdict stood; only its number was short.

### Nothing here still narrows `Air` out

This record's original `Air` bullet, and the D6 amendment that reversed it, are
the whole story: `RealFluid::served_fluids` carries `Air`, and the
property-diagram picker is `plot_fluids_available`, which is **derived** from
`served_fluids` rather than being a second list — so widening one widened both.
`crates/frees/src/lib.rs` asserts the published picker is exactly
`["Air", "R1234yf", "R134a", "Water"]`. The only surviving narrowing is
`TableBackend::served_fluids` (`props/propfun.rs`), which is the *other*
backend: its `(P,h)` build has D7's `(P,T)` transport grid for air and no state
table, so a picker entry there really would fail at every plot point. Correct as
written; left alone.

### One open rustprop-side question, out of scope here

Sweeping `h = h_L + q·(h_V − h_L)` through Air's dome directly against
`rustprop::props_si` finds a low-quality refusal window on the bubble side: at
1 bar (79 K) `Temperature(Air, P, Hmass)` answers at `q = 0` and `1e-6`, refuses
at `1e-4`, `1e-3` and `0.01`, and answers again from `0.05` up; at 7 bar (101 K)
only `1e-3` and `0.01` refuse. The message is rustprop's own — *"unable to
bracket the (p,X) solution in [59.77, 78.80] … the derivative path is not
ported"* — and it names itself an **unported path**, i.e. a rustprop-side gap,
not upstream behaviour being reproduced. (Not verified against upstream from
here; the wording is rustprop's own, at `flash_px.rs`'s
`px_solve_single_phase`.) It is not this repo's to fix and no fixture reaches
it: all three pneumatic documents ask at
`h = 424 950 J/kg` and 1–7 bar, i.e. `T = 298.7…300.0 K` and `Q = -1`,
single-phase gas some 167 K above Air's critical temperature. Recorded so the
next session does not re-derive it.

---

## Amendment — Wave-4 F9: every number in this record, re-measured

**Status:** implemented
**Date:** 2026-08-19
**Amends** the *Measured* table (which was taken mid-wave), part 4 of the
*Decision* (whose last sentence no longer described the tree), and the entry
count in *What it clears*.

This decision was written across a moving tree — rustprop advanced twice under
it and twelve fixtures joined the corpus after it — so its numbers were audited
one by one against the checkout rather than carried forward. Three moved.

### The bundle, re-measured

`wasm-pack build crates/frees --release --target web`, then CI's own
`stat -c%s` and `gzip -c`, on `wave4-f9` with rustprop at `d5a7331`:

| | raw | gzipped | of the 3072 KiB budget | headroom |
|---|---:|---:|---:|---:|
| D9, 2026-08-17 | 2700.3 KiB | 1109.5 KiB | 87.9 % | 371.7 KiB |
| **F9, 2026-08-19** | **2721.9 KiB** | **1118.2 KiB** | **88.6 %** | **350.1 KiB** |
| drift | +21.6 KiB | +8.7 KiB | +0.7 pt | −21.6 KiB |

21.6 KiB in two waves, against 350.1 KiB of headroom. The conclusions this
record draws from the four-corner table are unaffected — the bundle is still
smaller *and* more accurate than the pre-D9 3042.3 / 1597.7 it replaced, by
320.4 KiB raw and 479.5 KiB gzipped.

**Do not quote the D9 row as current.** It is the switch-over measurement and
is labelled as such now; this row is the one to re-measure against.

### The "one byte" row still holds — at two bytes

The finding that makes part 2 of this decision a *guard* rather than a saving
was re-run, because it is the one row whose failure would silently re-spend
684 KiB. Built the shipped configuration and then the same thing with
`--features frees-core/linked-tables`:

| | bytes |
|---|---:|
| shipped (`linked-tables` off) | 2 787 262 |
| `linked-tables` **on** | 2 787 264 |
| difference | **2 bytes** |

Both round to 2721.9 KiB. Turning the artifacts back on still does not bring
the 684 KiB back, because `install_builtin_once` no longer calls
`install_builtin` and fat LTO drops an unreachable data section on its own —
exactly as recorded. One `#[wasm_bindgen]` export wired to `install_from_bytes`
would still make all of it reachable again, which is what the feature exists to
turn into a build error rather than a red budget step.

### Part 4: the single-package form does not grade the table path any more

Re-run to see it rather than assume it: `cargo test --release -p frees-core
--test parity` fails on **12 of 719** documents, and the failure text names
nothing useful. `adv_moistair_W_passthrough`, the six other humid-air
documents, `hx-correlations-fluid`, `thermo-compliance` and the three pneumatic
documents all report

```
Java solved but Rust failed: solver error: Block 4 (5 equations) failed:
Newton iteration stalled after 0 iteration(s): no full, halved or damped step
reduces the residual (norm NaN).
```

— not "not a tabulated output", and not "does not implement HAPropsSI". The
port is being faithful when it does that: a property error inside Newton
becomes a `NaN` residual, because that is what `NewtonSolver.residuals()` does
in the Java ("an invalid state point is a bad region, not a fatal error"). The
cost is that the *cause* never reaches the reader.

`tests/parity.rs` now refuses the configuration before it replays anything:

```
the parity corpus cannot be replayed by the (P,h) TableBackend, and this
build has `rustprop-backend` OFF.
[…]
    cargo test --workspace --test parity
    cargo test -p frees-core --features rustprop-backend --test parity
```

It is a runtime check on `cfg!(feature = "rustprop-backend")`, not a
`#[cfg]` on the test and not `required-features` on the target, and the reason
is this record's own discipline about dead things failing loudly: both of those
would make `cargo test -p frees-core` **skip** the gate and report green. This
file's tolerance guards exist precisely to stop a gate quietly asserting
nothing, and a gate that can disappear from a run is the same failure one level
up. Verified in all three directions:

| command | before | after |
|---|---|---|
| `cargo test -p frees-core --test parity` | 12 failures, 49 s, cause unnamed | refuses in 0.00 s, names both working commands |
| `cargo test -p frees-core` | same, buried in the suite | same refusal; with `--no-fail-fast` the other 24 targets run and pass (2,986 tests) |
| `cargo test -p frees-core --features rustprop-backend --test parity` | — | **719 fixtures match**, 10 at a declared tolerance, 0 at a floor |
| `cargo test --workspace --test parity` (CI) | 719 match | unchanged — 719 match |

`fixtures/tolerances.json` therefore has no configuration that reads it. That
does not make it wrong and it was not touched: it describes a supported
configuration, and the corpus is what has outgrown it. The `TOLERANCE_FILE`
branch that selects it is kept, unreachable, with the reason written beside it.

### Verified and unmoved

Checked against the tree, found correct, changed nothing:

* `fixtures/golden/` holds **719** fixtures and `corpus-pending/` **14** —
  the counts F6 and F8 claim.
* `tolerances-rustprop.json` has exactly **ten** entries and two catalogued
  mechanisms, and the replay reports all ten as used, `solver_floor` empty.
* `RealFluid` still has the five methods D8 named, and `RustpropBackend`'s
  `served_fluids` is `Water, R134a, R1234yf, Air, INCOMP::MEG, INCOMP::MPG`.
  `crates/frees/src/lib.rs:1988` still asserts the published picker is
  `["Air", "R1234yf", "R134a", "Water"]`. *(Both superseded 2026-08-24 by the
  Wave C1 amendment at the end of this record: `CO2` joined the list, and the
  assertion is now `["Air", "CO2", "R1234yf", "R134a", "Water"]`.)*
* `fixtures/humidair/reference.json` is still 912 points over six case groups
  at CoolProp 8.0.0, and still graded in full.

## Amendment — Wave C1: `CarbonDioxide` joins `served_fluids`

**Status:** implemented
**Date:** 2026-08-24
**Amends** this record's `served_fluids` list under *Consequences and open
risks* and the Wave-4 F9 amendment's "Verified and unmoved" bullet that quotes
it, plus the picker assertion in `crates/frees/src/lib.rs`.

**Owner request.** This is not a technical re-derivation like the D6 `Air`
amendment below — nothing in the engine changed to make CO2 possible. Its data
has been linked since **Wave G2 (2026-08-23)**, which added the
`carbondioxide` feature to `rustprop-data` at a measured +25.5 KiB to close the
`docs_fluids_materials_03` linkage hold. That change deliberately stopped short
of the picker, and said so in `Cargo.toml`: *"Deliberately NOT added to
`served_fluids` — the diagram picker is a separate, deliberate list."* The owner
has now asked for the fluid. What follows is what was measured before agreeing.

### The list

```
Water, R134a, R1234yf, Air, CO2, INCOMP::MEG, INCOMP::MPG
```

`CO2` is the canonical spelling — `plot_fluids()` emits it and
`plot_fluids_available()` filters against it, so `"CarbonDioxide"` (a document
alias) would have matched nothing and silently changed no list. The published
picker is now `["Air", "CO2", "R1234yf", "R134a", "Water"]`; the glycol families
are served for property calls but have no dome to draw.

### What was verified

CO2 is the interesting case precisely because its critical point (304.128 K,
7.3773 MPa) and triple point (216.592 K, 0.51796 MPa) put most CO2 engineering
**above** the dome, where a dome-based generator can degrade quietly. So the
diagram generator and the property surface were driven directly, not assumed:

* **Constants land on CoolProp**: `Tcrit` 304.1282000029807 K, `pcrit`
  7377298.373446752 Pa, `Ttriple` 216.592 K, `ptriple` 517964.34344772575 Pa,
  `molar_mass` 0.0440098. Reachable under all four spellings the resolver
  accepts (`CO2`, `CarbonDioxide`, `R744`, `co2`).
* **The dome is complete**: 400/400 points across both branches, 216.592 K to
  the critical nudge, on all six diagram kinds; 200/200 on the single-branch
  `P-T`. Nine of nine quality lines and seven of seven isentropes are full at
  120/120. The critical marker reads (1433.06 J/kg-K, 304.1279 K) on `T-s` and
  (332075.0 J/kg, 7377246.5 Pa) on `P-h`.
* **Ordinary states round-trip to machine precision.** Saturation `(T,Q)` at
  217/230/250/270/290/300/304 K for Q = 0, 0.5, 1; subcooled liquid, superheated
  vapour, near-critical (304.2 K / 7.4 MPa, where `cp` reaches 76.2 kJ/kg-K —
  the critical anomaly, present and finite) and supercritical to 400 K / 20 MPa.
  `T(P,h)`, `T(P,s)` and `P(T,D)` return the input to 0 … 1.6e-10 K absolute
  across all of them. No refusal, no non-finite value, anywhere in that sweep.

### The one thing that broke, and it is CoolProp's physics

Two of CO2's three isobars came back as **empty arrays** on `T-s`, `T-v` and
`h-s` — not lines of gaps, zero-length curves — and one of its eight isotherms
did the same on `P-h`/`P-v`.

The cause is named by the error itself:

```
For now, we don't support T [217.092 K] below Tmelt(p) [217.5460 K]
```

That is CoolProp's melting-line guard, ported verbatim into rustprop. The
generator anchors every isobar's cold end at a fixed `Ttriple + 0.5 K`, which is
a fluid state for water (whose melting line leans the other way) and for R134a
(whose triple pressure is 390 Pa) — both were probed at their own anchors and
both answer at 0.5×, 1× and 2.5× `pcrit`. CO2's melting line is steep out of a
0.518 MPa triple point (`Tmelt` = 217.12 K at 3 MPa, 220.36 K at 18.44 MPa), so
that anchor is **solid** above ~2.9 MPa. The Java oracle with real CoolProp
would produce exactly the same empty curves; this is not a port defect.

It still costs CO2 a third of its `T-s` chart, because `ptriple`/`pcrit` span
14× for CO2 against water's 36000× and `nice_log_values` therefore yields only
three isobars in total. So the port now walks the cold anchor up to the coldest
temperature at that pressure the backend will serve, by bisecting the refusal
boundary between the known-bad anchor and the known-good hot end — the predicate
is monotone in `T` at fixed `P`, so it converges on the melting line, which is
where a physical isobar starts. **Ledger item 38**, and strictly additive: it
runs only after a refusal that the Java turns into an empty curve, so it can
only replace nothing with something. All three CO2 isobars now draw 120/120.
Water, R134a, R1234yf and Air answer at the first anchor and are untouched,
which is pinned by
`props/diagrams.rs::the_cold_anchor_walk_is_inert_for_the_fluids_that_answer_at_once`.

**The 220 K isotherm is left absent on purpose.** Walking its dense anchor down
would "work" and would still be worthless: `Psat(220 K)` is 0.599 MPa, below the
dilute anchor at `ptriple*1.2` = 0.622 MPa, so the entire pressure window at
220 K is compressed liquid spanning 1166 → ~1170 kg/m3. The recovered curve
would be a near-vertical 0.4 % stub — a worse answer than the honest absence.
The other seven isotherms (240–360 K) are complete.

### Parity

One fixture promoted: **`props_realfluid_co2_transcritical`** — a booster cycle
whose gas cooler runs at 9 MPa / 308.15 K, above the critical pressure. Corpus
983 → **984**.

Thirteen of its 25 variables are **bit-identical** to the Java oracle running
real CoolProp, including every dome value (`P1`, `h1`, `s1`, `d1`, `hf`), both
transport calls, the molar mass, and the supercritical density `d3`. Exactly one
leaf differs — `h2s = Enthalpy(CarbonDioxide, P = 9 MPa, s = s1)`, the `(P,Smass)`
flash, at 2.5361e-10 — and `w_c = h2 - h1` amplifies it 9.4× to 2.3731e-9, which
`cop` inherits. It therefore needs a tolerance entry at **3.6e-9** (measured
×1.5, this file's rule), on the existing `upstream-ps-flash-residual` mechanism,
which now has a second instance and says so.

The evidence that the golden is the loose side needs no third library here.
Evaluated **at CoolProp's own reported state** (T = 356.5829506828403 K,
rho = 184.1525333317221 kg/m3), rustprop returns `s`, `h` and `p` bit-identical
to it — 1898.4802265842322, 487203.2950423514, 9000000.000000004. The equation
of state is not in question at all; only the flash's stopping point is. That
state reproduces the requested **pressure** to 4.4e-16 but the requested
**entropy** only to 3.78e-10, and CoolProp's own two enthalpies for the leaf
(487203.29466312914 through the flash, 487203.2950423514 at the state the flash
reports) differ by 7.78e-10 — **three times the gap being graded**. rustprop's
flash reproduces the pressure exactly and the entropy to 5.99e-16, and its
487203.2947866893 lies between CoolProp's two.

Note the residual surfaces in **entropy** here where the R134a instance showed
it in **pressure**; the mechanism catalogue now records both signatures, since
its text read as though pressure were the thing that slips.

### The bundle

Adding a fluid to a *list* should cost nothing, because Wave G2 already paid for
the data. Measured, `wasm-pack build --release --target web`, post-`wasm-opt`:

| | raw | gzipped |
|---|---:|---:|
| before | 3 189 435 B (3114.7 KiB) | 1 300 309 B (1269.8 KiB) |
| after | 3 189 577 B (3114.8 KiB) | 1 300 348 B (1269.9 KiB) |
| **delta** | **+142 B (+0.14 KiB)** | **+39 B (+0.04 KiB)** |

142 bytes: the `"CO2"` string, its `Vec` entry, and the cold-anchor bisection.
**3114.8 KiB of the 4096 KiB budget, 981.2 KiB headroom.** If a future
re-measurement of this kind shows more than a few hundred bytes, something other
than the list changed.

### Open

* `Hydrogen` remains unlinked and unasked-for — the other half of the pair
  `fixtures/README.md` has tracked since the Phase-6 triage.
* The cold-anchor walk defends the isobars' cold end only. The hot anchor
  (`Tcrit * 1.15`) sits comfortably inside every served fluid's EOS range and
  never refused in this sweep, so it was left alone rather than defended
  speculatively.
* **`rustprop_warm.rs`'s `warm_t_of_p_hmass_costs_tens_of_microseconds` fails
  on a loaded machine, and did so before this change.** It asserts a 50 µs
  wall-clock median and measured 83.1 µs during this wave's gate run, at load
  average 16.9 with concurrent agents on the box. Verified not to be this
  change's doing by stashing the whole diff and re-running at HEAD: it fails
  there too, at 85.8 µs. Nothing here touches the warm path — `served_fluids`
  is consulted only by `plot_fluids_available`, never by `props_si`. This is
  the zero-headroom timing assertion `docs/status-wave3-f7.md` already flags;
  it has a one-shot retry built in and that is evidently not enough under this
  kind of load.
