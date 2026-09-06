//! Golden-corpus parity replay — the Rust engine against the Java oracle.
//!
//! Every fixture in `fixtures/golden/` was produced by running the document in
//! `fixtures/corpus/` through the reference Java engine
//! (`tools/golden-dumper`). This test replays the same documents through the
//! Rust engine and compares.
//!
//! Comparison policy (`fixtures/README.md`):
//! * `variables` — relative tolerance `1e-9`, absolute `1e-12` near zero. A
//!   *named variable of a named fixture* may instead be graded by a declared
//!   **absolute** tolerance — see "The absolute channel" below.
//!   Golden keys carry the Java first-seen spelling (`T_out`); the Rust engine
//!   keys by lowercase canonical name, so keys are folded before matching.
//!
//!   The fold is the Java's own: `EquationSystemSolver.buildResult` keys
//!   `Result.variables()` with `displayNames.getOrDefault(name, name)`, so the
//!   golden's key is a *display* name, not a canonical one. For a scalar
//!   document the two coincide once lowercased (`T_out` → `t_out`), which is
//!   why folding the golden alone worked for the whole pre-component corpus.
//!   It stops coinciding the moment components expand: the canonical name is
//!   `s2$p` and its display name is `s2.P`. So this replay routes the **Rust**
//!   side through the same `display_names` map before lowercasing — identical
//!   to the previous behaviour on every non-component fixture, and the only
//!   thing that makes a component fixture comparable at all.
//! * `display_names` — **exact**: keys and values must match the Java
//!   `ParseResult.displayNames` map the dumper recorded, spelling included.
//! * `block_count` — exact. A different blocking is a real divergence.
//! * `error` — the *classification* must agree (both solve, or both fail with
//!   the equivalent error type). Messages are not compared verbatim.
//! * `ode_tables` — one entry per `DYNAMIC` block, compared in declaration
//!   order. `name`/`method`/`columns`/`stopped` and the event `name`s are
//!   **exact**; `end_time` and every event `time` go through the same numeric
//!   tolerance as `variables`. Row cells take the same tolerance **plus a
//!   scale anchor**: a cell also passes when `|a − e| ≤ rel_tol · scale`,
//!   where `scale` is the max `|expected|` over that signal's own trajectory
//!   (the `time` column is excluded — sample times are grid structure, not
//!   integrated state). See "The decayed-signal measure" below.
//!
//! # The decayed-signal measure (Wave G1)
//!
//! A pointwise relative measure has no denominator on a signal that decays
//! through zero: `pressure-cooker`'s `steel$port$qdot` falls from 1 497.86 W
//! to 2.27e-10 W, where a 4.4e-5 W absolute agreement — 2.9e-8 of the
//! signal — reads as rel 5.7e-3 purely by denominator collapse. The digits a
//! pointwise measure demands there were never controlled by either engine:
//! both integrators run error control of the `atol + rtol·|y|` shape (the
//! IDA path at rtol 1e-6 / atol 1e-8), so the trailing digits of a decayed
//! tail are integration noise on both sides. Anchoring the tolerance to the
//! signal's own dynamic range is the same measure the integrators
//! themselves use, and it changes nothing for a healthy signal (there
//! `|e| ≈ scale`). The anchor uses the *expected* side's range only, so a
//! wrongly-huge Rust value cannot widen its own gate; a column whose golden
//! is all zeros gets `scale = 0` and keeps the pointwise measure (which
//! `ABS_TOL` already handles).
//!
//! Validated the same way the row comparison itself was: perturbing a
//! decayed-tail cell of `pressure-cooker`'s golden (`steel$port$qdot`,
//! row 240, −2.2719987170207744e-10 → 0.02) produces
//!
//! ```text
//!   [pressure-cooker] ode_tables[0] `cooker` rows row 240 col `steel$port$qdot` =
//!   -0.00000000022851085812685354 but Java got 0.02 (rel 1.000000011425543e0,
//!   scaled 1.3e-5 of column max |e| 1.498e3, tolerance 1.1e-6)
//! ```
//!
//! and the classic `dyn_plain_ode` 47.59… → 47.6 perturbation still goes red
//! (rel 1.8995734320745376e-4, scaled 9.5e-5 of column max |e| 9.500e1 —
//! five decades over the default either way). Both were observed in the same
//! 2/775 run on 2026-08-23, then the goldens were restored.
//!
//! # Why the `ode_tables` comparison is not optional
//!
//! **A solved `DYNAMIC` block puts nothing in `variables`.** The trajectory is a
//! first-class ODE Table, so a transient document's `variables` map holds only
//! its analytic parameters — `dyn_plain_ode` has exactly `{k, Tinf}` in it. A
//! fixture that compared `variables` alone would therefore pass *vacuously* on
//! every transient document in the corpus: the whole integration could be wrong,
//! or absent, and the gate would stay green.
//!
//! The comparison was validated the way the harness itself was in Phase 1 — by
//! perturbing a golden and watching the gate go red. Perturbing
//! `dyn_plain_ode`'s row `[20, 47.59095803046333]` to `47.6` produces
//!
//! ```text
//!   [dyn_plain_ode] ode_tables[0] `cooling` row 1 col `temp` = 47.59095803046333
//!   but Java got 47.6 (rel 1.9e-4, tolerance 1e-9)
//! ```
//!
//! and dropping the table entirely produces "Java recorded 1 ODE table(s), Rust
//! produced 0". Both were observed, then the golden was restored.
//!
//! # Per-fixture tolerance
//!
//! `fixtures/tolerances.json` may relax the *numeric* tolerance for a named
//! fixture, and nothing else. It exists because this build resolves real-fluid
//! properties from precomputed tables whose measured error is `1e-7…1e-4`
//! (decision D1) while the goldens hold full-accuracy CoolProp values — a gap no
//! table-backed engine can close, and one that must not be hidden by loosening
//! the gate for everybody. Three guards keep it honest:
//!
//! * a fixture named there but **absent** from `fixtures/golden/` fails;
//! * a fixture named there that **passes at the default** fails, so a tolerance
//!   that is no longer needed cannot sit in the file pretending it is. "Passes
//!   at the default" is judged over *everything numeric the fixture grades* —
//!   variables, ODE row cells (under the decayed-signal measure above),
//!   `end_time` and event times — because a transient fixture's divergence
//!   usually lives in its table, not its variables, and a guard that only read
//!   `variables` would kill every transient entry as dead on arrival;
//! * if the file catalogues its `mechanisms`, every entry must name one that
//!   exists and every catalogued mechanism must be named by an entry — see
//!   [`declared_tolerances`].
//!
//! # The absolute channel (Wave P1)
//!
//! A relative measure has no denominator against an **exact zero**, and the
//! corpus contains quantities that are identically zero by physics: a condenser
//! outlet still inside the dome makes `SC = Tcond − Temperature(P,h)` exactly 0,
//! an evaporator outlet below `hg` makes `SH = Temperature(P,h) − Tsat` exactly
//! 0. The CoolProp 8.0.0 wheel confirms both — at all four states in
//! `fixtures/tolerances-rustprop.json`'s `absolute` section, `Temperature(P,h)`
//! equals `T_sat(P)` *to the last bit* — and this engine returns `0` or one ulp
//! of a ~300 K temperature (`±5.7e-14`). The Java oracle answers the same call
//! from its `(P,Hmass)` interpolation table and returns `2.8e-7 … 1.2e-6` K, so
//! the golden asserts the oracle's own table error and every such variable reads
//! `rel ≈ 1.0` by denominator collapse. Nothing is wrong with the engine; the
//! *channel* was missing.
//!
//! So the tolerance file may carry an `absolute` section, and it is deliberately
//! **narrower than a relative entry in two ways**:
//!
//! * it is **per variable**, not per fixture — an absolute entry names the
//!   variables it covers, so a second divergence anywhere else in the same
//!   document still fails at the ordinary relative tolerance;
//! * it grades `variables` only. ODE row cells, `end_time` and event times keep
//!   the relative measure plus the scale anchor above, which is the same idea
//!   applied to a trajectory.
//!
//! A covered variable passes when `|a − e| ≤ absolute`, and is left **out of the
//! `worst` accumulator** that feeds the dead-relative-tolerance guard — its
//! `rel ≈ 1.0` would otherwise keep a dead relative entry looking alive
//! forever.
//!
//! Five guards: four mirror the relative channel's, and the last is one the
//! relative channel cannot have. [`declared_absolutes`] owns the checks that
//! need only the file, [`replay`] the ones that need the golden value, and
//! [`golden_corpus_parity`] the two stale-entry sweeps:
//!
//! * an entry whose fixture is not in `fixtures/golden/` fails;
//! * an entry naming a variable the golden does not have fails, and so does one
//!   the replay never reaches — a typo must not silently grade nothing;
//! * a variable that passes at its fixture's relative tolerance fails, exactly
//!   as a dead relative entry does;
//! * `ABS_TOL < absolute < ABS_CEILING`. The floor is the harness's own
//!   near-zero acceptance, below which the entry could not change an outcome.
//!   `ABS_CEILING` is `1e-4`, and the number is argued in kelvin because that is
//!   the unit every instance is in: the smallest *legitimately non-zero*
//!   superheat this corpus grades is 0.2522 K
//!   (`chiller-higher-refrigerant-flow-delivers-more-cooling-2`, held open by a
//!   zone ramp and agreed to four figures by both engines), so `1e-4` K is
//!   2 500× below the smallest real signal of this kind, and it also sits under
//!   the worst `(P,Hmass)→T` table error the corpus has measured (1.53e-4 …
//!   1.56e-4 K, the three chiller entries) — an entry needing more than the
//!   ceiling is claiming a bigger oracle artifact than any yet measured and owes
//!   fresh evidence rather than a bigger number;
//! * `absolute ≤ 2 · |expected|`, checked per variable against the golden. This
//!   is the bound that makes the channel self-limiting: where the true value is
//!   exactly zero, `|expected|` *is* the oracle's error, so forgiving more than
//!   twice it stops hiding the oracle's artifact and starts hiding the port's
//!   own. It also means the channel can never be pointed at a healthy variable
//!   to widen it. The file's measured-×1.5 rule leaves ~33 % of slack under it,
//!   deliberately.
//!
//! One limit follows from that last bound and is accepted: a golden of exactly
//! `0.0` admits no absolute entry at all. If the *port* returns more than
//! `ABS_TOL` where the oracle returns a true zero, the artifact is on this side
//! and deserves an investigation, not a widening.
//!
//! Validated red the way the decayed-signal measure and the `ode_tables`
//! comparison were — by breaking it four ways in one run on 2026-08-24 and
//! watching each report land, then restoring. Observed, in a `4/1257` run:
//!
//! ```text
//!   [chgclosed-charge-chain-is-well-posed] `cnd.sc` = 0.00000000000005684341886080802
//!   but Java got 0.00000027976881256108754 (abs 2.797687557176687e-7, declared
//!   absolute tolerance 1e-7)
//!
//!   [tpcharge-charge-sets-condensing-pressure-and-subcooling] declares an absolute
//!   tolerance for `cond.sc` of 2e-6, which is more than 2x the 0.0000005685998871740594
//!   the golden itself asserts there. Where the true value is an exact zero the golden
//!   IS the oracle's error, so a tolerance above twice it stops forgiving the oracle's
//!   artifact and starts hiding this engine's
//!
//!   [chgclosed-condensing-pressure-floats-with-ambient-and-charge] declares an
//!   absolute tolerance for `cnd.rho_out`, which passes at the fixture's relative
//!   tolerance 1.4e-6 (rel 4.7017152870485336e-8). Delete the absolute entry rather
//!   than leaving a dead channel in the file.
//!
//!   [accomp-air-coil-cools-and-dehumidifies] declares an absolute tolerance for
//!   `coil.ev.superheat`, which is not a variable of the golden fixture
//!   [accomp-air-coil-cools-and-dehumidifies] declares an absolute tolerance for
//!   `coil.ev.superheat` in fixtures/tolerances-rustprop.json, which the replay
//!   never reached
//! ```
//!
//! The fourth breakage reports twice, by design: a misspelled variable both
//! grades nothing and leaves the entry unreached. It also produced the proof
//! that this channel is load-bearing rather than decorative — with the entry
//! pointed elsewhere, the variable it should have covered fell through to the
//! ordinary measure and failed there:
//!
//! ```text
//!   [accomp-air-coil-cools-and-dehumidifies] `coil.ev.sh` = 0 but Java got
//!   -0.0000012156851880718025 (rel 1e0, tolerance 2e-7)
//! ```
//!
//! # The solver request (Wave Q)
//!
//! A fixture may carry a top-level **`request`** object — the non-default
//! parts of what the Java test handed
//! `EquationSystemSolver.solve(source, settings, specs, extraDefs)`. It exists
//! because a document is only half of a solve: `x^2 = 4` from a guess of −1
//! and from a guess of +1 are two different roots, and grading either at the
//! engine defaults would assert an answer the Java test never made. Before it,
//! the harvester dropped every such site by tag (`SKIP_SITE_TAGS`) — 47 of
//! them, complex mode included.
//!
//! The chain is Wave I's, one channel over: the harvester evaluates the Java
//! arguments into a `<name>.request.json` sidecar, `tools/golden-dumper`
//! rebuilds the same `SolverSettings` and `Map<String, VariableSpec>` from it
//! and records the golden **under them**, then embeds the sidecar verbatim
//! here; [`request_settings_of`] and [`request_overrides_of`] turn it back
//! into the pair `solve_with_tables` takes. Two sidecars can coexist on one
//! document (`curvefn-solves-inverse-problem-through-newton` carries a
//! Function Table *and* a guess), and the shapes are the wasm boundary's own
//! `stopCriteria` / `variableInfo` DTOs rather than a format invented here.
//!
//! **An absent field is the engine default and the empty override slice** —
//! byte-for-byte the previous call — so the 1281 fixtures that predate the
//! channel replay unchanged.
//!
//! # This replay needs the `rustprop-backend` feature
//!
//! Since Wave-3 F6/F8 the corpus holds twelve documents the `(P,h)`
//! `TableBackend` cannot serve **at all**, so it is replayable by exactly one
//! backend. Rather than fail twelve times with an error that names nothing,
//! [`golden_corpus_parity`] refuses up front and prints the command that
//! works — see [`WRONG_BACKEND`].
//!
//! # Sharding — keeping a growing gate bounded (Wave Q2)
//!
//! This is the project's longest gate and it grows with every wave: 983
//! fixtures at ~145 s two waves ago, **1 281 at ~362 s** now (release, this
//! box; 317.4 s of it inside the replay loop), and ~15 min of CI wall clock.
//! The replay is embarrassingly parallel across fixtures — each one is an
//! independent `solve` against its own golden — so two environment variables
//! may split it across processes:
//!
//! * `PARITY_SHARD_COUNT` — how many processes are replaying the corpus;
//! * `PARITY_SHARD_INDEX` — which one this is, `0 <= index < count`.
//!
//! **With neither set the replay is exactly what it always was**: one process,
//! every fixture, every comparison unchanged. With both set this process
//! replays the fixtures whose position in the *sorted* golden listing satisfies
//! `i % count == index`.
//!
//! Sorted-then-strided is a **partition**: every fixture lands in exactly one
//! shard and the union over `index in 0..count` is the whole corpus, by
//! construction rather than by convention. Striding rather than slicing is
//! deliberate — the listing is sorted by name and adjacent names are usually
//! the same family (the two-phase-cycle `chgclosed-*` documents, the `dyn_*`
//! transients), so a contiguous slice would hand one shard every expensive
//! document while another finished in seconds.
//!
//! ## How many shards — and the ceiling one fixture puts on all of them
//!
//! Measured per fixture on 2026-08-25 (release, an instrumented run over the
//! 1 281 fixtures, 317.4 s inside the replay loop, mean 248 ms), the cost is not
//! merely skewed, it is **concentrated in one document**:
//!
//! ```text
//!   ev-battery-cooling-pid                193.0 s   60.8 % of the whole replay
//!   component-port-units-fan-duct          12.0 s
//!   component-networks-fan-duct-real       11.1 s
//!   docs_tutorials_05                       8.3 s
//!   odelib-p20-stiff-reaction-chain-ode15s  6.1 s
//!   … top 20 of 1 281 = 89.4 % of the total
//! ```
//!
//! Read the *shares* rather than the seconds: this box is shared with other
//! agents and whole-run wall clock was seen anywhere between 178 s and 406 s.
//! The shape is the stable part, and it was measured twice to establish that —
//! a second independent instrumented run put the same fixture at 191.3 s of
//! 316.8 s (60.4 %), the top 20 at 90.0 %, and every partition figure below
//! within 2 % of the numbers quoted.
//!
//! No partition can put that fixture in two places, so **193 s is the floor for
//! any shard count** and the useful range of `N` is small. Measured against
//! that distribution:
//!
//! | strategy | N=2 | N=4 | N=8 |
//! |---|---:|---:|---:|
//! | stride over the sorted names (what this does) | 273.8 s | **227.7 s** | 202.7 s |
//! | greedy bin-pack by golden file size | 269.5 s | 209.8 s | 213.6 s |
//! | greedy bin-pack by *measured* cost (the oracle) | 193.0 s | 193.0 s | 193.0 s |
//!
//! **N = 4**, and the plain stride. Four takes the longest shard from 317.4 s
//! to 227.7 s and leaves the other three at 33.0 / 10.6 / 46.1 s; eight buys a
//! further 25 s for four more runners, and the file-size bin-pack — the only
//! cost proxy available without committing a cost table that would go stale —
//! is worth 8 % at N=4 and *negative* at N=8 (Spearman 0.74 against true cost:
//! good enough to rank, not good enough to pack; `component-port-units-fan-duct`
//! is a 1 KiB golden and 12 s of solve).
//!
//! The point of four is not today's 1.39×, it is the mandate: **the gate stays
//! bounded while the corpus grows.** New fixtures spread across four bins while
//! the critical path stays where it is, so the largest non-critical shard
//! (46.1 s) has room for the corpus to grow ~4× — 1 281 → ~5 400 fixtures —
//! before it reaches 193 s and the wall clock starts moving again.
//!
//! The honest reading of that table is that the parity gate's real lever is not
//! sharding at all: **one fixture is 61 % of it.** `ev-battery-cooling-pid` is a
//! PID-controlled transient graded through `ode_tables`, and until it gets
//! cheaper no amount of parallelism takes this job below ~3 min of replay.
//!
//! ### That lever has been pulled once — 2026-08-25, Wave R2
//!
//! The paragraph above is kept as written, because its *reasoning* is what the
//! shard table was derived from. Its seconds are now history. R2 ported the LRU
//! the Java façade keeps in front of CoolProp — `props/propfun.rs`'s `cache`
//! module, and `props/CoolProp.java` for the original — which this engine had
//! been missing: `ev-battery-cooling-pid` makes **5 539 832** property calls for
//! 162 893 distinct argument tuples, and 84 % of them repeat the call
//! immediately before them. Re-measured on a quiet box, paired and alternated,
//! user CPU:
//!
//! ```text
//!                                        before      after
//!   ev-battery-cooling-pid alone         79.4 s     44.5 s    1.78x
//!     (PARITY_SHARD_COUNT=1308 PARITY_SHARD_INDEX=855)
//!   the whole 1 308-fixture replay      167.9 s    132.3 s    -21.2 %
//!   the document's share of the gate     47.3 %     33.6 %
//! ```
//!
//! Two things there are worth carrying forward. The **whole** saving is this one
//! document — the rest of the corpus measures the same before and after, which
//! is exactly the concentration the table above describes. And the document is
//! *still* the critical path at a third of the gate, so the floor argument
//! survives with a smaller floor: ~44 s rather than ~193 s. The shard table's
//! own seconds predate both this change and Q3's, and want re-measuring before
//! anyone re-derives `N` from them.
//!
//! ## Why the gate is not weaker across the union
//!
//! Three things could make a shard assert less than a whole run does, and each
//! is closed:
//!
//! * **A shard that silently replays nothing.** Half a configuration (one
//!   variable set, the other absent or empty), a non-numeric count, a zero
//!   count, an index outside `0..count`, or a stride that selects no fixture
//!   all **panic**. There is no input to [`declared_shard`] that yields a green
//!   run over an empty set — which is the whole reason the split is safe to
//!   make.
//! * **The "declared in the tolerance file but no such fixture" sweep.** That
//!   is a property of the *corpus*, not of a slice of it, so it keeps reading
//!   the corpus: `paths` stays the full sorted directory listing and only the
//!   replay loop is strided. Every shard runs the sweep over all 1 281 stems —
//!   it reads a directory listing and not one fixture, so repeating it costs
//!   nothing — which means a stale entry fails `count` times rather than
//!   escaping into the shard nobody looked at.
//! * **The dead-entry guards** — a `fixtures` tolerance whose fixture matches
//!   at the default, a `solver_floor` whose fixture converges at the default,
//!   an `absolute` entry that passes relatively or names a variable the golden
//!   lacks. Each needs the replayed *value*, so each can only run where its
//!   fixture runs. That is exactly right: a fixture is in exactly one shard, so
//!   across the union every such entry is graded **exactly once** — never zero
//!   times, and never (as a whole-file check would under sharding) declared
//!   dead merely because this process did not replay it. The one that needed
//!   changing is the "the replay never reached this absolute entry" sweep in
//!   [`golden_corpus_parity`], which now considers only entries whose fixture
//!   is in *this* shard; every other entry is another shard's business. An
//!   entry naming a fixture that does not exist at all is not lost by that
//!   scoping — it belongs to no shard, and the whole-corpus sweep above already
//!   fails it in every shard.
//!
//! The `mechanisms` catalogue check ([`declared_tolerances`]) is untouched and
//! stays whole-file: every entry must name a catalogued slug and every
//! catalogued slug must be named by an entry. It reads only the tolerance file,
//! so it is as true in a shard as in a whole run, and it runs in all of them.
//!
//! Every run — sharded or not — prints a machine-readable census line
//!
//! ```text
//!   parity-shard: index=0 count=4 replayed=321 corpus=1281
//! ```
//!
//! so the union can be *checked* rather than assumed: sum `replayed` across the
//! shards and it must equal `corpus`. Measured that way on 2026-08-25 at N=4:
//! 321 + 320 + 320 + 320 = 1 281, and the unsharded run replayed 1 281.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use frees_core::{solve_with_tables, FreesError, SolverSettings, VariableOverride};

const REL_TOL: f64 = 1e-9;
const ABS_TOL: f64 = 1e-12;

/// The loosest absolute tolerance the `absolute` section may declare, in the
/// graded variable's own SI unit. Argued in the module docs; in one line, it is
/// 2 500× below the smallest legitimately non-zero superheat this corpus grades
/// and below the worst `(P,Hmass)→T` table error it has measured.
const ABS_CEILING: f64 = 1e-4;

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/golden")
}

/// Which tolerance file grades this build — one per property backend.
///
/// Decision D9 pins the gate to **one** backend, but "one" cannot mean "one
/// file": the entries in `fixtures/tolerances.json` exist because of the (P,h)
/// tables' own interpolation error, and under rustprop most of them are dead
/// (the file's own rule then makes them failures) while the survivors have a
/// completely different cause — the *golden* side, where the Java answered
/// `(P,Hmass) → T/Dmass/Smass` from its own run-time 256/96/48 table. So each
/// backend is graded by the file that describes it, selected by the same `cfg`
/// that decides which backend `install_builtin_once` installs. There is no
/// configuration in which both files are read, and none in which neither is.
///
/// The table branch is **currently unreachable**, and deliberately kept. Since
/// Wave-3 F6/F8 this corpus cannot be replayed by the table backend at all
/// ([`WRONG_BACKEND`]), so nothing reads `tolerances.json` today. That does not
/// make it wrong — it describes a configuration D9 still supports and that a
/// smaller corpus would reach again — so the branch stays, next to the reason
/// it does not fire.
#[cfg(feature = "rustprop-backend")]
const TOLERANCE_FILE: &str = "tolerances-rustprop.json";
#[cfg(not(feature = "rustprop-backend"))]
const TOLERANCE_FILE: &str = "tolerances.json";

fn tolerance_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(TOLERANCE_FILE)
}

/// Does this build have the one backend that can serve the whole corpus?
///
/// A runtime `const` rather than a `#[cfg]` on the test item, on purpose: a
/// `cfg`-ed-out replay would leave every helper below it unused, and — worse —
/// a `required-features` key on the target would make `cargo test -p
/// frees-core` **skip** the gate silently and report green. A gate that can
/// vanish without saying so is the failure mode this file's other guards exist
/// to prevent, so the wrong configuration is loud instead.
const CORPUS_IS_SERVABLE: bool = cfg!(feature = "rustprop-backend");

/// What to print when it is not. Names both working commands, because the two
/// invocations that land here are reached from different intents.
///
/// The message has to *say* this, because the failure does not look like a
/// missing backend. A property error inside Newton becomes a `NaN` residual —
/// faithfully, that is what the Java does (`NewtonSolver.residuals()` treats an
/// invalid state as a bad region, not a fatal error) — so all twelve documents
/// come back as `Newton iteration stalled after 0 iteration(s) … (norm NaN)`,
/// with nothing in the text about `HAPropsSI` or a tabulated output.
const WRONG_BACKEND: &str = "\
the parity corpus cannot be replayed by the (P,h) TableBackend, and this build \
has `rustprop-backend` OFF.

Twelve of the corpus documents ask for HAPropsSI (the seven humid-air ones), \
single-phase (P,T) transport (hx-correlations-fluid), CompressibilityFactor \
(thermo-compliance) or Air Enthalpy (the three pneumatic documents). The table \
backend serves none of those at all, so they do not miss a tolerance — they \
come back as `Newton iteration stalled after 0 iteration(s) … (norm NaN)`, \
which names neither the fixture's real problem nor this one.

Run the gate with the backend the corpus was promoted against:

    cargo test --workspace --test parity
    cargo test -p frees-core --features rustprop-backend --test parity

The first is what CI runs: frees requires the feature, and resolver-v2 \
unifies it onto frees-core. See docs/decisions/0009-rustprop-backend.md and \
fixtures/README.md.";

/// The two variables that split the corpus across processes. See "Sharding" in
/// the module docs; both must be set, or neither.
const SHARD_INDEX_VAR: &str = "PARITY_SHARD_INDEX";
const SHARD_COUNT_VAR: &str = "PARITY_SHARD_COUNT";

/// Which slice of the sorted corpus this process replays.
///
/// `{ index: 0, count: 1 }` is the unsharded default and selects everything, so
/// every expression below that mentions a shard reduces to the whole corpus
/// when nothing is configured.
struct Shard {
    index: usize,
    count: usize,
}

/// Read the shard from the environment, refusing every configuration that could
/// under-replay.
///
/// The refusals are the point. A gate that can be handed a stride and quietly
/// assert less than it claims is the same failure mode as a `required-features`
/// key that makes the whole replay vanish ([`CORPUS_IS_SERVABLE`]), so a
/// half-set, malformed, zero or out-of-range configuration panics rather than
/// falling back to "replay something". The only silent behaviour is the one
/// that replays *more*: both variables absent means the whole corpus.
fn declared_shard() -> Shard {
    // Empty counts as absent: a workflow that interpolates an undefined matrix
    // value sets the variable to "", and that must not read as shard 0.
    let var = |name: &str| {
        std::env::var(name)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let number = |name: &str, raw: &str| -> usize {
        raw.parse()
            .unwrap_or_else(|e| panic!("{name}={raw:?} is not a whole number: {e}"))
    };
    match (var(SHARD_INDEX_VAR), var(SHARD_COUNT_VAR)) {
        (None, None) => Shard { index: 0, count: 1 },
        (Some(index), Some(count)) => {
            let count = number(SHARD_COUNT_VAR, &count);
            let index = number(SHARD_INDEX_VAR, &index);
            assert!(count > 0, "{SHARD_COUNT_VAR}=0 would replay nothing at all");
            assert!(
                index < count,
                "{SHARD_INDEX_VAR}={index} is outside 0..{count}. That shard replays no \
                 fixture, so the union of the shards would not be the corpus — and a gate \
                 that covers less than it claims is worse than no gate"
            );
            Shard { index, count }
        }
        (index, count) => panic!(
            "the parity replay is half-sharded: {SHARD_INDEX_VAR}={index:?}, \
             {SHARD_COUNT_VAR}={count:?}. Set both (to replay one shard of the corpus) or \
             neither (to replay all of it in one process) — a partial configuration is \
             exactly the silent under-replay this gate must never do."
        ),
    }
}

/// Declared relative tolerance per fixture stem, from `fixtures/tolerances.json`.
///
/// # The mechanism catalogue
///
/// A tolerance file **may** carry a top-level `mechanisms` object mapping a
/// slug to its explanation, and when it does, the slug becomes load-bearing:
/// every fixture entry must name one that exists, and every catalogued
/// mechanism must be named by at least one entry.
///
/// The second half is the one that earns its keep. A mechanism whose last
/// instance dies leaves prose behind that still reads like a live description
/// of the build — the retired stop-criterion mechanism sat in
/// `tolerances-rustprop.json` for a whole wave after its only fixture reached
/// the engine default — and the file's entire value is that a future session
/// can trust what it says about *this* backend. So the discipline the
/// `fixtures` section already has ("a dead tolerance fails") applies one level
/// up: a dead *explanation* fails too.
///
/// A file with no `mechanisms` object is unaffected. That is
/// `fixtures/tolerances.json`, whose 23 entries describe the table backend and
/// predate the catalogue.
///
/// The citation set spans **both** graded sections — `fixtures` and `absolute`
/// — because a mechanism can perfectly well have all of its instances in one of
/// them, and the orphan check lives here. `declared_absolutes` checks the other
/// direction for its own entries; each direction has exactly one owner.
fn declared_tolerances() -> BTreeMap<String, f64> {
    let path = tolerance_path();
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        // Absent is legitimate: it means every fixture is held to the default.
        Err(_) => return BTreeMap::new(),
    };
    let doc: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display()));
    let catalogue: BTreeSet<String> = doc["mechanisms"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let mut cited: BTreeSet<String> = BTreeSet::new();
    let tolerances: BTreeMap<String, f64> = doc["fixtures"]
        .as_object()
        .unwrap_or_else(|| panic!("{} needs a `fixtures` object", path.display()))
        .iter()
        .map(|(name, entry)| {
            let rel = entry["relative"].as_f64().unwrap_or_else(|| {
                panic!(
                    "{}: fixture `{name}` needs a numeric `relative`",
                    path.display()
                )
            });
            assert!(
                entry["reason"].as_str().is_some_and(|r| r.len() > 40),
                "{}: fixture `{name}` needs a `reason` that says which mechanism \
                 produces the error, not a placeholder",
                path.display()
            );
            assert!(
                rel > REL_TOL && rel < 1e-2,
                "{}: fixture `{name}` declares {rel:e}, which is either tighter than \
                 the default or loose enough to hide a real divergence",
                path.display()
            );
            if !catalogue.is_empty() {
                let mechanism = entry["mechanism"].as_str().unwrap_or_else(|| {
                    panic!(
                        "{}: fixture `{name}` needs a `mechanism` naming one of {catalogue:?}",
                        path.display()
                    )
                });
                assert!(
                    catalogue.contains(mechanism),
                    "{}: fixture `{name}` names mechanism `{mechanism}`, which the file's \
                     `mechanisms` catalogue does not define (it has {catalogue:?})",
                    path.display()
                );
                cited.insert(mechanism.to_string());
            }
            (name.clone(), rel)
        })
        .collect();
    for entry in doc["absolute"].as_object().into_iter().flatten() {
        if let Some(mechanism) = entry.1["mechanism"].as_str() {
            cited.insert(mechanism.to_string());
        }
    }
    let orphans: Vec<&String> = catalogue.difference(&cited).collect();
    assert!(
        orphans.is_empty(),
        "{}: `mechanisms` defines {orphans:?}, which no fixture entry names. A mechanism \
         with no instance is a dead explanation — delete it, exactly as a dead tolerance \
         would be deleted.",
        path.display()
    );
    tolerances
}

/// Declared Newton **stop criterion** per fixture stem, from the same file's
/// optional `solver_floor` object.
///
/// This is a different knob from `fixtures`, and it exists for a mechanism only
/// the accuracy path has. A `(P,h)` table is a bilinear surface, so a residual
/// like `T_out = Temperature(fluid, P, h)` is smooth in `h` and Newton drives it
/// to the `1e-12` default. rustprop answers the same call with an *iterative*
/// flash, whose output has a floor: it is the exact value to within its own
/// convergence, and stepping `h` by less than that moves `T` by a jump instead of
/// a slope. A block that carries such a residual therefore cannot be driven
/// below that floor by any line search — the engine reports "no full, halved or
/// damped step reduces the residual", which is the truth.
///
/// Relaxing the stop criterion for the named fixture is the honest response:
/// the *values* are still compared against the Java oracle at the ordinary
/// tolerance, so the assertion is intact — only the point at which the solver
/// stops chasing arithmetic noise moves. The guards mirror `fixtures`': an entry
/// whose fixture converges at the default is dead and fails, and an entry with
/// no fixture fails.
fn declared_solver_floors() -> BTreeMap<String, f64> {
    let path = tolerance_path();
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(_) => return BTreeMap::new(),
    };
    let doc: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display()));
    let Some(entries) = doc["solver_floor"].as_object() else {
        // Absent is legitimate: it means every fixture solves at the default.
        return BTreeMap::new();
    };
    entries
        .iter()
        .map(|(name, entry)| {
            let rel = entry["rel_tolerance"].as_f64().unwrap_or_else(|| {
                panic!(
                    "{}: solver_floor `{name}` needs a numeric `rel_tolerance`",
                    path.display()
                )
            });
            assert!(
                entry["reason"].as_str().is_some_and(|r| r.len() > 40),
                "{}: solver_floor `{name}` needs a `reason` naming the residual whose \
                 property call has the floor, not a placeholder",
                path.display()
            );
            let default = SolverSettings::default().rel_tolerance;
            assert!(
                rel > default && rel < 1e-6,
                "{}: solver_floor `{name}` declares {rel:e}; the engine default is \
                 {default:e} and anything at or above 1e-6 stops the solver before the \
                 physics, not before the noise",
                path.display()
            );
            (name.clone(), rel)
        })
        .collect()
}

/// Declared **absolute** tolerances, from the same file's optional `absolute`
/// object: fixture stem → variable key (the golden's key, lowercased) → the
/// tolerance in that variable's own SI unit.
///
/// This is the channel for a quantity whose true answer is a *structurally
/// exact zero*, where a relative measure has no denominator at all. The module
/// docs carry the physics, the scoping argument and the two bounds; this
/// function owns the three checks that need only the file:
///
/// * every entry states a `unit`, a `mechanism` the catalogue defines and a
///   `reason` carrying the evidence — which variable, which leaf call, what the
///   third-party oracle says. The harness cannot verify a unit string, but an
///   absolute number without one is unreadable, so the field is mandatory;
/// * every covered variable states both the declared `absolute` and the
///   `measured` gap it was drawn from, and the declaration must cover its own
///   measurement (the file's rule is measured ×1.5, to two significant figures);
/// * `ABS_TOL < absolute < ABS_CEILING`.
///
/// The two guards that need the golden — the per-variable `2 · |expected|`
/// ceiling and the dead-entry check — are in [`replay`], and the "names a
/// variable nothing replayed" guard is in [`golden_corpus_parity`].
fn declared_absolutes() -> BTreeMap<String, BTreeMap<String, f64>> {
    let path = tolerance_path();
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        // Absent is legitimate: it means no fixture needs the channel.
        Err(_) => return BTreeMap::new(),
    };
    let doc: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display()));
    let catalogue: BTreeSet<String> = doc["mechanisms"]
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    let Some(entries) = doc["absolute"].as_object() else {
        return BTreeMap::new();
    };
    entries
        .iter()
        .map(|(name, entry)| {
            assert!(
                entry["reason"].as_str().is_some_and(|r| r.len() > 40),
                "{}: absolute `{name}` needs a `reason` giving the evidence that the true \
                 value is an exact zero — which variable, which leaf call, what the \
                 third-party oracle says — not a placeholder",
                path.display()
            );
            assert!(
                entry["unit"].as_str().is_some_and(|u| !u.is_empty()),
                "{}: absolute `{name}` needs a `unit`. An absolute tolerance is a \
                 dimensional quantity and the number is unreadable without it",
                path.display()
            );
            if !catalogue.is_empty() {
                let mechanism = entry["mechanism"].as_str().unwrap_or_else(|| {
                    panic!(
                        "{}: absolute `{name}` needs a `mechanism` naming one of {catalogue:?}",
                        path.display()
                    )
                });
                assert!(
                    catalogue.contains(mechanism),
                    "{}: absolute `{name}` names mechanism `{mechanism}`, which the file's \
                     `mechanisms` catalogue does not define (it has {catalogue:?})",
                    path.display()
                );
            }
            let vars = entry["variables"].as_object().unwrap_or_else(|| {
                panic!(
                    "{}: absolute `{name}` needs a `variables` object naming the variables it \
                     covers. An absolute entry never widens a whole fixture",
                    path.display()
                )
            });
            assert!(
                !vars.is_empty(),
                "{}: absolute `{name}` covers no variable",
                path.display()
            );
            let vars = vars
                .iter()
                .map(|(var, decl)| {
                    let abs = decl["absolute"].as_f64().unwrap_or_else(|| {
                        panic!(
                            "{}: absolute `{name}` variable `{var}` needs a numeric `absolute`",
                            path.display()
                        )
                    });
                    let measured = decl["measured"].as_f64().unwrap_or_else(|| {
                        panic!(
                            "{}: absolute `{name}` variable `{var}` needs a numeric `measured` — \
                             the gap the tolerance was drawn from",
                            path.display()
                        )
                    });
                    assert!(
                        abs >= measured,
                        "{}: absolute `{name}` variable `{var}` declares {abs:e} but records a \
                         measured gap of {measured:e}, which it does not cover",
                        path.display()
                    );
                    assert!(
                        abs > ABS_TOL && abs < ABS_CEILING,
                        "{}: absolute `{name}` variable `{var}` declares {abs:e}; at or under \
                         {ABS_TOL:e} the harness already accepts the difference and the entry \
                         changes nothing, and at or above {ABS_CEILING:e} it is loose enough to \
                         hide a real divergence in a quantity whose true value is zero",
                        path.display()
                    );
                    (var.clone(), abs)
                })
                .collect();
            (name.clone(), vars)
        })
        .collect()
}

/// `Double.toString` output, or `"NaN"` / `"Infinity"` / `"-Infinity"` strings.
fn as_f64(v: &serde_json::Value) -> f64 {
    match v {
        serde_json::Value::Number(n) => n.as_f64().expect("numeric fixture value"),
        serde_json::Value::String(s) => match s.as_str() {
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            other => panic!("unexpected string number {other:?}"),
        },
        other => panic!("unexpected fixture value {other:?}"),
    }
}

/// The variables whose value is not finite, as `name = value`, sorted by name.
///
/// Split out of the replay so it can be tested directly: a guard nobody has
/// watched fail is not yet a guard, and this one cannot be provoked from a
/// fixture — the whole point of it is that no promoted document reaches a
/// non-finite value today.
fn non_finite_values<'a>(values: impl IntoIterator<Item = (&'a String, &'a f64)>) -> Vec<String> {
    let mut bad: Vec<String> = values
        .into_iter()
        .filter(|(_, value)| !value.is_finite())
        .map(|(var, value)| format!("{var} = {value}"))
        .collect();
    bad.sort();
    bad
}

#[test]
fn the_non_finite_guard_reports_every_shape_and_nothing_else() {
    let finite: BTreeMap<String, f64> = [
        ("a".to_string(), 0.0),
        ("b".to_string(), -273.15),
        ("c".to_string(), f64::MAX),
        ("d".to_string(), f64::MIN_POSITIVE),
    ]
    .into_iter()
    .collect();
    assert!(non_finite_values(finite.iter()).is_empty());

    // All three shapes, and the report names the variable so a red gate is
    // actionable without re-running anything.
    let bad: BTreeMap<String, f64> = [
        ("ok".to_string(), 1.0),
        ("nan".to_string(), f64::NAN),
        ("pinf".to_string(), f64::INFINITY),
        ("ninf".to_string(), f64::NEG_INFINITY),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        non_finite_values(bad.iter()),
        vec![
            "nan = NaN".to_string(),
            "ninf = -inf".to_string(),
            "pinf = inf".to_string(),
        ]
    );
}

/// Relative difference, `0.0` when both are NaN or exactly equal (which covers
/// the infinities).
fn rel_diff(actual: f64, expected: f64) -> f64 {
    if (actual.is_nan() && expected.is_nan()) || actual == expected {
        return 0.0;
    }
    let diff = (actual - expected).abs();
    if diff <= ABS_TOL {
        return 0.0;
    }
    diff / expected.abs().max(actual.abs()).max(f64::MIN_POSITIVE)
}

fn close(actual: f64, expected: f64, rel_tol: f64) -> bool {
    if actual.is_nan() && expected.is_nan() {
        return true;
    }
    if actual == expected {
        return true; // covers infinities and exact hits
    }
    let diff = (actual - expected).abs();
    diff <= ABS_TOL || diff <= rel_tol * expected.abs().max(actual.abs())
}

/// Map a golden `error.type` (a Java exception simple name) to the Rust error
/// classification it must correspond to.
fn error_matches(java_type: &str, rust: &FreesError) -> bool {
    match java_type {
        "SolverException" => matches!(rust, FreesError::Solver { .. }),
        "ParseException" => matches!(rust, FreesError::Parse { .. }),
        "PropertyEvaluationException" => matches!(rust, FreesError::Property { .. }),
        // Unmapped Java exception types: accept any Rust error — both engines
        // refused the document, which is the parity that matters here.
        _ => true,
    }
}

struct Failure {
    fixture: String,
    detail: String,
}

/// The tolerance a cell would *need* to pass — the smaller of the pointwise
/// relative error and the scale-anchored error (see the module docs). This is
/// what feeds the dead-tolerance guard's `worst`, so an entry stays exactly as
/// alive as the loosest measure that still fails the default.
fn needed_tol(actual: f64, expected: f64, scale: f64) -> f64 {
    let rel = rel_diff(actual, expected);
    if scale <= 0.0 {
        return rel;
    }
    let diff = (actual - expected).abs();
    if diff <= ABS_TOL {
        return 0.0;
    }
    rel.min(diff / scale)
}

/// Compare the golden's `ode_tables` array against what the engine integrated.
///
/// A golden dumped before the dumper grew this section has `ode_tables` absent
/// (`Value::Null`), which is **not** the same as an empty array: absent means
/// "this fixture predates the section", empty means "the Java engine produced no
/// tables". Only the second is a claim, so only the second is checked against a
/// Rust engine that produced tables — otherwise every pre-Phase-7 fixture in the
/// corpus would fail the moment a `DYNAMIC` block started working.
///
/// `worst` accumulates the largest [`needed_tol`] across every numeric
/// comparison here, for the dead-tolerance guard in [`replay`].
fn compare_ode_tables(
    golden: &serde_json::Value,
    actual: &[frees_core::ode::problem::OdeTableResult],
    rel_tol: f64,
    worst: &mut f64,
    fail: &mut impl FnMut(String),
) {
    let Some(expected) = golden.as_array() else {
        if !actual.is_empty() {
            fail(format!(
                "Rust produced {} ODE table(s) but the golden has no `ode_tables` section — \
                 re-dump this fixture with tools/golden-dumper so the trajectory is compared \
                 instead of ignored",
                actual.len()
            ));
        }
        return;
    };

    if expected.len() != actual.len() {
        fail(format!(
            "Java recorded {} ODE table(s), Rust produced {}",
            expected.len(),
            actual.len()
        ));
        return;
    }

    for (i, (want, got)) in expected.iter().zip(actual).enumerate() {
        let at = |what: &str| format!("ode_tables[{i}] `{}` {what}", got.name);

        // Identity and shape are exact: a renamed block, a different solver or a
        // reordered column set is a real divergence, not a rounding difference.
        for (field, expected_str, actual_str) in [
            (
                "name",
                want["name"].as_str().unwrap_or("?"),
                got.name.as_str(),
            ),
            (
                "method",
                want["method"].as_str().unwrap_or("?"),
                got.method.as_str(),
            ),
        ] {
            if expected_str != actual_str {
                fail(format!(
                    "{} = {actual_str:?} but Java got {expected_str:?}",
                    at(field)
                ));
            }
        }
        let want_columns: Vec<&str> = want["columns"]
            .as_array()
            .map(|a| a.iter().filter_map(|c| c.as_str()).collect())
            .unwrap_or_default();
        if want_columns != got.columns {
            fail(format!(
                "{} = {:?} but Java got {want_columns:?}",
                at("columns"),
                got.columns
            ));
            // Every row comparison below indexes by column, so a shape mismatch
            // would only produce noise.
            continue;
        }
        if want["stopped"].as_bool().unwrap_or(false) != got.stopped {
            fail(format!(
                "{} = {} but Java got {}",
                at("stopped"),
                got.stopped,
                want["stopped"]
            ));
        }
        let want_end = as_f64(&want["end_time"]);
        *worst = worst.max(rel_diff(got.end_time, want_end));
        if !close(got.end_time, want_end, rel_tol) {
            fail(format!(
                "{} = {} but Java got {want_end} (rel {:e}, tolerance {rel_tol:e})",
                at("end_time"),
                got.end_time,
                rel_diff(got.end_time, want_end)
            ));
        }

        let want_rows = want["rows"].as_array().cloned().unwrap_or_default();
        if want_rows.len() != got.rows.len() {
            fail(format!(
                "{} — Java sampled {} row(s), Rust produced {}",
                at("rows"),
                want_rows.len(),
                got.rows.len()
            ));
            continue;
        }
        // The per-signal anchor for the decayed-signal measure (module docs):
        // max |expected| over the column's own trajectory, expected side only.
        // `time` is grid structure, not integrated state, so it keeps the
        // pointwise measure via scale 0. NaN cells fall out of the max on
        // their own (`f64::max` ignores them).
        let scales: Vec<f64> = (0..want_columns.len())
            .map(|c| {
                if want_columns[c] == "time" {
                    return 0.0;
                }
                want_rows
                    .iter()
                    .filter_map(|row| row.as_array()?.get(c))
                    .map(|cell| as_f64(cell).abs())
                    .fold(0.0f64, f64::max)
            })
            .collect();
        for (r, (want_row, got_row)) in want_rows.iter().zip(&got.rows).enumerate() {
            let cells = want_row.as_array().cloned().unwrap_or_default();
            if cells.len() != got_row.len() {
                fail(format!(
                    "{} row {r} has {} cell(s), Java had {}",
                    at("rows"),
                    got_row.len(),
                    cells.len()
                ));
                continue;
            }
            for (c, (want_cell, &got_cell)) in cells.iter().zip(got_row).enumerate() {
                let want_value = as_f64(want_cell);
                let scale = scales.get(c).copied().unwrap_or(0.0);
                *worst = worst.max(needed_tol(got_cell, want_value, scale));
                let diff = (got_cell - want_value).abs();
                let anchored = scale > 0.0 && diff <= rel_tol * scale;
                if !close(got_cell, want_value, rel_tol) && !anchored {
                    let col = got.columns.get(c).map(String::as_str).unwrap_or("?");
                    if scale > 0.0 {
                        fail(format!(
                            "{} row {r} col `{col}` = {got_cell} but Java got {want_value} \
                             (rel {:e}, scaled {:.1e} of column max |e| {scale:.3e}, \
                             tolerance {rel_tol:e})",
                            at("rows"),
                            rel_diff(got_cell, want_value),
                            diff / scale
                        ));
                    } else {
                        fail(format!(
                            "{} row {r} col `{col}` = {got_cell} but Java got {want_value} \
                             (rel {:e}, tolerance {rel_tol:e})",
                            at("rows"),
                            rel_diff(got_cell, want_value)
                        ));
                    }
                }
            }
        }

        // Events: the recorded name keeps its *source* case (the Java reads
        // `ctx.IDENT(0).getText()` raw), so it is compared exactly; the crossing
        // time is a solve output and takes the numeric tolerance.
        let want_events = want["events"].as_array().cloned().unwrap_or_default();
        if want_events.len() != got.events.len() {
            fail(format!(
                "{} — Java recorded {} event hit(s) ({:?}), Rust recorded {} ({:?})",
                at("events"),
                want_events.len(),
                want_events
                    .iter()
                    .map(|e| e["name"].as_str().unwrap_or("?"))
                    .collect::<Vec<_>>(),
                got.events.len(),
                got.events
                    .iter()
                    .map(|e| e.name.as_str())
                    .collect::<Vec<_>>()
            ));
            continue;
        }
        for (e, (want_hit, got_hit)) in want_events.iter().zip(&got.events).enumerate() {
            let want_name = want_hit["name"].as_str().unwrap_or("?");
            if want_name != got_hit.name {
                fail(format!(
                    "{} hit {e} named `{}` but Java recorded `{want_name}`",
                    at("events"),
                    got_hit.name
                ));
            }
            let want_time = as_f64(&want_hit["time"]);
            *worst = worst.max(rel_diff(got_hit.time, want_time));
            if !close(got_hit.time, want_time, rel_tol) {
                fail(format!(
                    "{} hit {e} (`{}`) fired at {} but Java got {want_time} \
                     (rel {:e}, tolerance {rel_tol:e})",
                    at("events"),
                    got_hit.name,
                    got_hit.time,
                    rel_diff(got_hit.time, want_time)
                ));
            }
        }
    }
}

/// The fixture's optional `request` field as solver stop criteria.
///
/// The field is the harvester's `.request.json` sidecar embedded verbatim by
/// the dumper, which built the *same* `SolverSettings` before recording the
/// golden — so this is not a re-interpretation of the oracle's inputs, it is
/// the same object read twice. The shape is the boundary's own
/// `StopCriteriaDto` (`SolverApiSupport.StopCriteriaDto`, `StopCriteria` in
/// `api.ts`), deliberately: the browser already speaks it, so the fixture
/// format carries no schema of its own.
///
/// Two of the five keys have no counterpart in this port and are read by the
/// Java side only, exactly as the boundary treats them:
/// `changeInVariables` (this Newton's stop rule is residual-based) and
/// `elapsedTimeSeconds` (core has no clock on `wasm32-unknown-unknown`; the
/// boundary installs the deadline). A fixture whose Java answer depended on
/// either would be a divergence this replay *should* report, not paper over.
///
/// An absent field is [`SolverSettings::default`] — byte-for-byte the old
/// call, which is why all 1281 fixtures that predate this channel replay
/// unchanged.
fn request_settings_of(v: &serde_json::Value) -> SolverSettings {
    let mut settings = SolverSettings::default();
    let stop = &v["stopCriteria"];
    if let Some(iterations) = stop["maxIterations"].as_u64() {
        settings.max_iterations = iterations.max(1) as usize;
    }
    if let Some(tolerance) = stop["relativeResiduals"].as_f64() {
        if tolerance.is_finite() && tolerance > 0.0 {
            settings.rel_tolerance = tolerance;
        }
    }
    settings.complex_mode = stop["complexMode"].as_bool().unwrap_or(false);
    settings
}

/// The fixture's optional `request` field as per-variable overrides.
///
/// The shape is the boundary's `VariableInfoDto` (`VariableInfo` in `api.ts`)
/// minus its `units` key: a `VariableSpec` is the Java *engine*'s record, not
/// its HTTP DTO, so every value the harvester can read off one is already SI
/// and there is nothing to convert. An **absent** `lower`/`upper` is the
/// record's own ±∞ default — JSON has no infinity literal, and
/// [`VariableOverride`]'s `None` means exactly that. An absent `guess` is
/// `DEFAULT_GUESS` clamped into the bounds, which `engine::override_spec`
/// already does.
fn request_overrides_of(v: &serde_json::Value) -> Vec<VariableOverride> {
    let Some(rows) = v["variableInfo"].as_array() else {
        return Vec::new();
    };
    rows.iter()
        .map(|row| VariableOverride {
            name: row["name"].as_str().unwrap_or_default().to_string(),
            guess: row["guess"].as_f64(),
            lower: row["lower"].as_f64(),
            upper: row["upper"].as_f64(),
            unit: None,
            uncertainty: row["uncertainty"].as_f64(),
        })
        .collect()
}

/// The fixture's optional `function_tables` field as solver extra defs.
///
/// The field is the harvester's `.tables.json` sidecar embedded verbatim by
/// the dumper (which installed the same tables on the Java side), in the
/// core def shape: `name` (already trimmed + lowercased, as
/// `SolveDtos.functionDefsOf` keys them), `arg_names`, `x_log`/`y_log`, and
/// `curves` of `{param, xs, ys}`. No units travel on this channel — the
/// Java's 5-argument `FunctionTableDef` constructor.
fn function_tables_of(v: &serde_json::Value) -> Vec<frees_core::parser::defs::FunctionTableDef> {
    let Some(tables) = v.as_array() else {
        return Vec::new();
    };
    let strings = |v: &serde_json::Value| -> Vec<String> {
        v.as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| s.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    };
    let floats = |v: &serde_json::Value| -> Vec<f64> {
        v.as_array()
            .map(|a| a.iter().map(as_f64).collect())
            .unwrap_or_default()
    };
    tables
        .iter()
        .map(|t| frees_core::parser::defs::FunctionTableDef {
            name: t["name"]
                .as_str()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase(),
            arg_names: strings(&t["arg_names"]),
            x_log: t["x_log"].as_bool().unwrap_or(false),
            y_log: t["y_log"].as_bool().unwrap_or(false),
            curves: t["curves"]
                .as_array()
                .map(|cs| {
                    cs.iter()
                        .map(|c| frees_core::parser::defs::Curve {
                            param: c["param"].as_f64(),
                            xs: floats(&c["xs"]),
                            ys: floats(&c["ys"]),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            output_unit: None,
            arg_units: None,
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn replay(
    path: &Path,
    tolerances: &BTreeMap<String, f64>,
    floors: &BTreeMap<String, f64>,
    absolutes: &BTreeMap<String, BTreeMap<String, f64>>,
    used: &mut BTreeSet<String>,
    used_floors: &mut BTreeSet<String>,
    used_absolutes: &mut BTreeSet<(String, String)>,
    failures: &mut Vec<Failure>,
) {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string();
    let rel_tol = tolerances.get(&name).copied().unwrap_or(REL_TOL);
    let raw = fs::read_to_string(path).expect("fixture readable");
    let fixture: serde_json::Value = serde_json::from_str(&raw).expect("fixture is valid JSON");

    let source = fixture["source"].as_str().expect("fixture has source");
    let expect = &fixture["expect"];
    let expected_error = &expect["error"];

    // Request-level Function Tables (the GUI channel, decision D10): the
    // harvester stages them as a `.tables.json` sidecar, the dumper installs
    // them on the Java side and embeds them here, and the replay hands them to
    // `solve_with_tables` — the same merge position every solving endpoint
    // uses. An absent field is the empty slice, which is byte-for-byte
    // `solve`, so every table-less fixture replays exactly as before.
    let extra_tables = function_tables_of(&fixture["function_tables"]);

    // The solver request (Wave Q): the stop criteria and per-variable
    // guesses/bounds the Java test passed to
    // `solve(source, settings, specs, defs)`, staged by the harvester as a
    // `.request.json` sidecar and embedded here by the dumper, which recorded
    // the golden *under them*. Absent is the engine default plus the empty
    // override slice, so every fixture without the field replays as before.
    let request = &fixture["request"];
    let requested = request_settings_of(request);
    let overrides = request_overrides_of(request);
    let run =
        |settings: &SolverSettings| solve_with_tables(source, settings, &overrides, &extra_tables);

    // A declared stop-criterion floor is a claim that the default cannot be
    // reached, so it is verified before it is used — exactly as a declared
    // numeric tolerance is. A fixture that solves at the default has a dead
    // entry; one the relaxation does not rescue has the wrong entry. "The
    // default" here means *this fixture's* request, not the engine's: a
    // relaxation is measured against what the oracle itself was given.
    let settings = match floors.get(&name) {
        None => requested,
        Some(&rel_tolerance) => {
            if run(&requested).is_ok() {
                failures.push(Failure {
                    fixture: name.clone(),
                    detail: format!(
                        "{TOLERANCE_FILE} relaxes this fixture's stop criterion to \
                         {rel_tolerance:e}, but it solves at the engine default. Delete the \
                         solver_floor entry rather than leaving a dead relaxation in the file."
                    ),
                });
            } else {
                used_floors.insert(name.clone());
            }
            SolverSettings {
                rel_tolerance,
                ..requested
            }
        }
    };

    let mut fail = |detail: String| {
        failures.push(Failure {
            fixture: name.clone(),
            detail,
        });
    };

    match run(&settings) {
        Ok(solution) => {
            if !expected_error.is_null() {
                fail(format!(
                    "Java failed with {} but Rust solved: {:?}",
                    expected_error["type"].as_str().unwrap_or("?"),
                    solution.values
                ));
                return;
            }

            // This replay's own blind spot, closed here rather than by a second
            // pass over the corpus. `close`/`rel_diff` below treat NaN against
            // NaN as agreement and infinities as exact hits — deliberately, so
            // that a golden recording the oracle's own non-finite answer can be
            // graded at all — but that means a fixture could match its golden
            // while this engine produced garbage. The corpus is solved here
            // anyway, so asserting finiteness costs nothing.
            //
            // It is the assertion `props_robustness`'s
            // `no_promoted_fixture_solves_to_a_non_finite_value` used to make by
            // solving all 1308 documents a SECOND time, in the one CI job that
            // was already the workflow's critical path. One difference is
            // deliberate and is an improvement: that test always used
            // `SolverSettings::default()`, so for the fixtures carrying a
            // `.request.json` it graded a configuration the document was never
            // meant to run under (and silently skipped the ones that then failed
            // to solve at all). Here every fixture is checked under the settings
            // it actually ships with.
            let non_finite = non_finite_values(solution.values.iter());
            if !non_finite.is_empty() {
                // Return: with a non-finite in the solution every tolerance
                // reading below is meaningless, and letting them run would bury
                // this line under a decade of derived failures.
                fail(format!(
                    "solved to non-finite value(s): {}",
                    non_finite.join(", ")
                ));
                return;
            }

            // Fold golden keys to lowercase to match the Rust canonical keys.
            let golden_vars: BTreeMap<String, f64> = expect["variables"]
                .as_object()
                .expect("variables object")
                .iter()
                .map(|(k, v)| (k.to_ascii_lowercase(), as_f64(v)))
                .collect();

            // The Rust side keyed the way `Result.variables()` is keyed: through
            // `displayNames`, then lowercased. See the module docs.
            let actual_vars: BTreeMap<String, f64> = solution
                .values
                .iter()
                .map(|(name, value)| {
                    let display = solution.display_names.get(name).unwrap_or(name);
                    (display.to_ascii_lowercase(), *value)
                })
                .collect();

            // The variables this fixture grades by an absolute tolerance
            // instead — see "The absolute channel" in the module docs.
            let covered = absolutes.get(&name);
            let mut worst = 0.0f64;
            for (var, &expected) in &golden_vars {
                match actual_vars.get(var) {
                    None => fail(format!("missing variable `{var}` (expected {expected})")),
                    Some(&actual) => match covered.and_then(|c| c.get(var)) {
                        // The ordinary path: relative tolerance, and the
                        // reading feeds the dead-relative-tolerance guard.
                        None => {
                            worst = worst.max(rel_diff(actual, expected));
                            if !close(actual, expected, rel_tol) {
                                fail(format!(
                                    "`{var}` = {actual} but Java got {expected} (rel {:e}, \
                                     tolerance {rel_tol:e})",
                                    rel_diff(actual, expected)
                                ));
                            }
                        }
                        // The absolute channel. `worst` deliberately does NOT
                        // see this variable: its rel is ~1.0 by denominator
                        // collapse, which would keep a dead *relative* entry on
                        // the same fixture looking alive for ever.
                        Some(&abs_tol) => {
                            used_absolutes.insert((name.clone(), var.clone()));
                            let diff = (actual - expected).abs();
                            if abs_tol > 2.0 * expected.abs() {
                                fail(format!(
                                    "declares an absolute tolerance for `{var}` of {abs_tol:e}, \
                                     which is more than 2x the {expected} the golden itself \
                                     asserts there. Where the true value is an exact zero the \
                                     golden IS the oracle's error, so a tolerance above twice it \
                                     stops forgiving the oracle's artifact and starts hiding this \
                                     engine's"
                                ));
                            }
                            if close(actual, expected, rel_tol) {
                                fail(format!(
                                    "declares an absolute tolerance for `{var}`, which passes at \
                                     the fixture's relative tolerance {rel_tol:e} (rel {:e}). \
                                     Delete the absolute entry rather than leaving a dead channel \
                                     in the file.",
                                    rel_diff(actual, expected)
                                ));
                            } else if diff > abs_tol {
                                fail(format!(
                                    "`{var}` = {actual} but Java got {expected} (abs {diff:e}, \
                                     declared absolute tolerance {abs_tol:e})"
                                ));
                            }
                        }
                    },
                }
            }
            for var in actual_vars.keys() {
                if !golden_vars.contains_key(var) {
                    fail(format!("extra variable `{var}` not in the golden fixture"));
                }
            }
            // A variable named in the `absolute` section that the golden does
            // not have grades nothing at all, silently. That is the same
            // failure mode the dead-entry guards exist to prevent, one level
            // down: the scope of an absolute entry is its variable names, so a
            // typo there is a scope error.
            for var in covered.into_iter().flatten().map(|(var, _)| var) {
                if !golden_vars.contains_key(var) {
                    fail(format!(
                        "declares an absolute tolerance for `{var}`, which is not a variable of \
                         the golden fixture"
                    ));
                }
            }

            // display_names is compared EXACTLY: the Java engine records the
            // spelling of each variable's first appearance, and the dumper
            // wrote that map into the fixture verbatim.
            let golden_names: BTreeMap<String, String> = expect["display_names"]
                .as_object()
                .expect("fixture has display_names")
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.as_str().expect("display name is a string").to_string(),
                    )
                })
                .collect();
            if solution.display_names != golden_names {
                fail(format!(
                    "display_names {:?} but Java recorded {golden_names:?}",
                    solution.display_names
                ));
            }

            let expected_blocks = expect["block_count"].as_u64().unwrap_or(0) as usize;
            if solution.blocks.len() != expected_blocks {
                fail(format!(
                    "block_count {} but Java got {expected_blocks}",
                    solution.blocks.len()
                ));
            }

            compare_ode_tables(
                &expect["ode_tables"],
                &solution.ode_tables,
                rel_tol,
                &mut worst,
                &mut fail,
            );

            // The dead-tolerance guard runs LAST, once `worst` has seen every
            // numeric comparison the fixture grades — variables and ODE
            // tables. A transient fixture's divergence usually lives in its
            // table, and a guard that read only `variables` would kill every
            // transient entry as dead on arrival.
            if tolerances.contains_key(&name) {
                if worst <= REL_TOL {
                    fail(format!(
                        "fixtures/{TOLERANCE_FILE} relaxes this fixture to {rel_tol:e}, but it \
                         matches the oracle to {worst:e} — at or under the {REL_TOL:e} default. \
                         Delete the entry rather than leaving a dead tolerance in the file."
                    ));
                } else {
                    used.insert(name.clone());
                }
            }
        }
        Err(err) => {
            if expected_error.is_null() {
                fail(format!("Java solved but Rust failed: {err}"));
            } else {
                let java_type = expected_error["type"].as_str().unwrap_or("?");
                if !error_matches(java_type, &err.error) {
                    fail(format!(
                        "Java failed with {java_type} but Rust failed differently: {err}"
                    ));
                }
            }
        }
    }
}

#[test]
fn golden_corpus_parity() {
    if !CORPUS_IS_SERVABLE {
        panic!("{WRONG_BACKEND}");
    }

    let dir = golden_dir();
    let mut paths: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .filter_map(|entry| {
            let p = entry.ok()?.path();
            (p.extension().and_then(|x| x.to_str()) == Some("json")).then_some(p)
        })
        .collect();
    paths.sort();

    assert!(
        !paths.is_empty(),
        "no golden fixtures in {} — the parity harness is not wired",
        dir.display()
    );

    // The shard this process replays (see "Sharding" in the module docs).
    // `paths` deliberately stays WHOLE: the stale-declaration sweep at the
    // bottom is a property of the corpus, not of this slice, and it is what
    // makes the union of the shards as strong as one run. Only the replay loop
    // is strided.
    let shard = declared_shard();
    let mine: Vec<&PathBuf> = paths
        .iter()
        .enumerate()
        .filter(|(i, _)| *i % shard.count == shard.index)
        .map(|(_, p)| p)
        .collect();
    assert!(
        !mine.is_empty(),
        "shard {}/{} selected 0 of the {} golden fixtures in {}. A shard that replays \
         nothing must fail rather than report green — reduce {SHARD_COUNT_VAR}.",
        shard.index,
        shard.count,
        paths.len(),
        dir.display()
    );
    // This shard's fixture stems, for scoping the one guard that cannot be
    // whole-corpus (the unreached-absolute-entry sweep below).
    let mine_stems: BTreeSet<String> = mine
        .iter()
        .filter_map(|p| p.file_stem()?.to_str().map(str::to_string))
        .collect();

    let tolerances = declared_tolerances();
    let floors = declared_solver_floors();
    let absolutes = declared_absolutes();
    let mut used = BTreeSet::new();
    let mut used_floors = BTreeSet::new();
    let mut used_absolutes = BTreeSet::new();
    let mut failures = Vec::new();
    for path in &mine {
        replay(
            path,
            &tolerances,
            &floors,
            &absolutes,
            &mut used,
            &mut used_floors,
            &mut used_absolutes,
            &mut failures,
        );
    }

    // An absolute declaration the replay never reached grades nothing — the
    // fixture failed before its variables were compared, or the variable does
    // not exist. Either way it is dead, and dead entries do not accumulate.
    //
    // This is the one guard that had to become shard-local: an entry whose
    // fixture is not in this shard was never *offered* to the replay, which is
    // not the same as unreachable. It belongs to exactly one shard and is
    // graded there. An entry whose fixture is in no shard at all — because it
    // has no fixture — falls to the whole-corpus sweep below, which every shard
    // runs.
    for (fixture, vars) in absolutes.iter().filter(|(f, _)| mine_stems.contains(*f)) {
        for var in vars.keys() {
            if !used_absolutes.contains(&(fixture.clone(), var.clone())) {
                failures.push(Failure {
                    fixture: fixture.clone(),
                    detail: format!(
                        "declares an absolute tolerance for `{var}` in \
                         fixtures/{TOLERANCE_FILE}, which the replay never reached"
                    ),
                });
            }
        }
    }

    // A declaration for a fixture that is not in the corpus is a stale entry, and
    // the "dead entry" guards above cannot see it — nothing replays it.
    //
    // Whole-corpus on purpose, in EVERY shard: it is checked against `paths`,
    // not `mine`, so a stale entry cannot hide in the slice this process did
    // not take. It reads the directory listing and no fixture, so running it
    // `count` times is free.
    for (section, name) in tolerances
        .keys()
        .map(|n| ("fixtures", n))
        .chain(floors.keys().map(|n| ("solver_floor", n)))
        .chain(absolutes.keys().map(|n| ("absolute", n)))
    {
        if !paths
            .iter()
            .any(|p| p.file_stem().and_then(|s| s.to_str()) == Some(name.as_str()))
        {
            failures.push(Failure {
                fixture: name.clone(),
                detail: format!(
                    "declared in fixtures/{TOLERANCE_FILE} ({section}) but has no fixture in \
                     fixtures/golden/"
                ),
            });
        }
    }

    if !failures.is_empty() {
        let scope = if shard.count == 1 {
            String::new()
        } else {
            format!(
                " (shard {}/{} of {})",
                shard.index,
                shard.count,
                paths.len()
            )
        };
        let mut report = format!(
            "\n{}/{} fixtures diverged from the Java oracle{scope}:\n",
            failures
                .iter()
                .map(|f| f.fixture.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            mine.len()
        );
        for f in &failures {
            report.push_str(&format!("  [{}] {}\n", f.fixture, f.detail));
        }
        panic!("{report}");
    }

    // The census line, machine-readable and printed on every green run: summing
    // `replayed` across the shards must give `corpus`. It is how the union is
    // checked rather than assumed — see "Sharding" in the module docs.
    println!(
        "parity-shard: index={} count={} replayed={} corpus={}",
        shard.index,
        shard.count,
        mine.len(),
        paths.len()
    );
    println!(
        "parity: {} fixtures match the Java oracle through {} \
         ({} at a declared tolerance from fixtures/{TOLERANCE_FILE}: {}) \
         ({} at a declared stop-criterion floor: {}) \
         ({} variable(s) on the absolute channel: {})",
        mine.len(),
        frees_core::props::propfun::backend_description(),
        used.len(),
        used.iter().cloned().collect::<Vec<_>>().join(", "),
        used_floors.len(),
        used_floors.iter().cloned().collect::<Vec<_>>().join(", "),
        used_absolutes.len(),
        used_absolutes
            .iter()
            .map(|(fixture, var)| format!("{fixture}:{var}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
}
