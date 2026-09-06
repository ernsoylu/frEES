# Parity fixtures

The correctness backbone of the port (`PLAN.md` §4). The Java engine has 1,237
JUnit tests; hand-translating 24,359 lines of test code would be the project's
biggest mistake. Instead both engines run the **same corpus** and are compared.

```
fixtures/corpus/*.frees        the documents (hand-authored + harvested)  — 1308
fixtures/corpus/*.tables.json  request-level Function Tables for the document
                               beside them (12 — the curvefn group; see below)
fixtures/corpus/*.request.json non-default solver settings / variable specs for
                               the document beside them (25; see below)
fixtures/golden/*.json         what the Java engine produced for each     — 1308
fixtures/corpus-pending/       the staging area: documents not yet promoted — 2
fixtures/proptables/      generated CoolProp (P,h) split tables (not a parity artifact)
fixtures/auxtables/       generated CoolProp FRAUX1 grids   (not a parity artifact)
tools/golden-dumper/      the Java side that generates fixtures/golden
tools/table-gen/          the Java side that generates fixtures/proptables
tools/aux-gen/            the Java side that generates fixtures/auxtables
```

**A `<name>.tables.json` sidecar carries the request-level Function Tables of
the document beside it** — the GUI channel (`SolveDtos.functionTables`, wired
end-to-end by Wave H / decision D10). The harvester writes it when the Java
test passed `extraDefs` into `solve(...)`; `tools/golden-dumper` installs the
tables on the Java side (`SolveDtos.functionDefsOf`'s rules: keyed by the
trimmed lowercased name, the 5-argument `FunctionTableDef` constructor) *and*
embeds the sidecar verbatim in the fixture as a top-level `function_tables`
field; `tests/parity.rs` replays that field through core's
`solve_with_tables`, the same merge position every solving endpoint uses. An
absent field is the empty slice — byte-for-byte the plain `solve` — so the
rest of the corpus replays exactly as before. These twelve fixtures are the
only oracle-graded coverage of the request-tables channel; everything else
about them (tolerances, promotion, error rules) is ordinary.

**A `<name>.request.json` sidecar carries the rest of the solve request** —
the stop criteria (complex mode included) and the per-variable
guesses/bounds/uncertainty the Java test handed
`EquationSystemSolver.solve(source, settings, specs, extraDefs)`. It exists
because **a document is only half of a solve**: `x^2 = 4` from a guess of −1
and from a guess of +1 are two different roots, and grading either at the
engine defaults asserts an answer the Java test never made. Wave Q (2026-08-25)
built it on the tables chain above, one channel over: the harvester evaluates
the two arguments into the sidecar, `tools/golden-dumper` rebuilds the same
`SolverSettings` and `Map<String, VariableSpec>` from it and records the golden
**under them**, then embeds the sidecar verbatim as a top-level `request`
field; `tests/parity.rs` turns it back into the `SolverSettings` +
`VariableOverride` pair `solve_with_tables` takes. An absent field is the
engine default and the empty override slice — byte-for-byte the previous
call — so the 1281 fixtures that predate the channel replay unchanged.

```json
{
  "stopCriteria": { "maxIterations": 250, "relativeResiduals": 1e-6,
                    "changeInVariables": 1e-9, "elapsedTimeSeconds": 3600.0,
                    "complexMode": true },
  "variableInfo": [ { "name": "x", "guess": 2.5, "lower": 0.0,
                      "upper": 4000.0, "uncertainty": 0.1 } ]
}
```

Four things about the shape are load-bearing, and each one is a decision:

* **The two objects are the wasm boundary's own DTOs** — `stopCriteria` is
  `SolverApiSupport.StopCriteriaDto` (`StopCriteria` in `api.ts`) and
  `variableInfo` is `VariableInfoDto` (`VariableInfo`). The fixture format
  invents nothing: the browser already speaks both, and
  `crates/frees/src/lib.rs::settings_of`/`overrides_of` are the same
  conversion the replay does.
* **`variableInfo` carries no `units` key.** A `VariableSpec` is the Java
  *engine*'s record, not its HTTP DTO, so everything the harvester reads off
  one is already SI and there is nothing to convert. The harvester never emits
  a unit and both sides pass `None`.
* **An absent `lower`/`upper` is ±∞** — the record's own default. JSON has no
  infinity literal and does not need one; `VariableOverride`'s `None` means
  exactly that, and the dumper substitutes `Double.NEGATIVE_INFINITY` /
  `POSITIVE_INFINITY`. An absent `guess` is `DEFAULT_GUESS` clamped into the
  bounds, which is the Java `VariableInfoDto.toSpec` rule that
  `engine::override_spec` already mirrors.
* **Two of the five stop criteria are read by the Java side only.**
  `changeInVariables` has no counterpart in this port (its Newton stop rule is
  residual-based) and `elapsedTimeSeconds` none in core (no clock on
  `wasm32-unknown-unknown` — the boundary installs the deadline). This is
  exactly how the boundary treats them, and a fixture whose Java answer
  depended on either is a divergence the replay *should* report rather than
  paper over.

The two sidecars are independent and can coexist on one document —
`curvefn-solves-inverse-problem-through-newton` carries a Function Table *and*
a guess, which is what its Java test does. Both are part of the harvester's
duplicate key: the same text under different request-level inputs is a
different fixture, which is what lets `eqsys-guess-value-selects-root-2` and
`-3` (the same equation, guesses 0.5 and 2.5) sit in the corpus as the two
roots they select.

## How properties are answered — read this before the tolerance sections

**The engine's real-fluid backend is [rustprop](https://github.com/ernsoylu/RustProp),
a pure-Rust port of CoolProp 8.0.0, linked as an ordinary Cargo dependency.**
Decision [D9](../docs/decisions/0009-rustprop-backend.md) made it the wasm
build's *only* in-bundle property source on 2026-08-17, implementing
[D8](../docs/decisions/0008-coolprop-wasm.md). Everything the parity gate grades
— native and browser alike — goes through it, and it is the reason a real-fluid
document can be graded at the corpus default `1e-9` at all.

`proptables/` and `auxtables/` are the odd ones out: they hold **inputs to the
Rust engine**, not expected outputs. The `.phtab` files are decision
[D1](../docs/decisions/0001-property-backend.md)'s precomputed `(P,h)` split
tables and the `.fraux` files are [D7](../docs/decisions/0007-auxiliary-property-grids.md)'s
three surfaces that geometry cannot carry — the incompressible glycols, air
transport, and transport on the saturation line. Formats and measured
tabulation error live in `tools/table-gen/README.md` and
`tools/aux-gen/README.md`. Both sets are regenerated by their `run.sh`, never
edited.

They are **no longer what the browser downloads.** D9 took the linked bytes out
of the wasm behind a `linked-tables` Cargo feature; what is left of D1 in the
browser is the decoders and the `install_from_bytes` fetch seam, for a host that
wants the offline path. Natively the artifacts are still linked (the feature is
on by default) and the `TableBackend` is still selectable, so these directories
are live inputs — just not the accuracy path. **Do not add a fluid to them**:
D8's moratorium stands, and rustprop supersedes an `air.phtab` and any fourth
`FRAUX1` grid.

## Regenerating

```bash
tools/golden-dumper/run.sh                    # corpus -> golden
tools/golden-dumper/run.sh <corpus> <out>     # explicit paths
tools/table-gen/run.sh                        # -> fixtures/proptables
```

**`golden-dumper/run.sh` regenerates every `.frees` in the directory it is
pointed at.** Running it over `corpus-pending/corpus` while another change is
mid-flight will mint goldens for documents that are not ready for one. Point it
at an explicit output directory and diff before keeping the result.

`classpath.sh` locates the newest `core-*.jar` under
`../frEES/backend/core/build/libs/` and assembles the dependency classpath from
the Gradle cache. It needs no Gradle run and **never writes to `../frEES`**.

Two classpath hazards it handles, both found the hard way:

* The cache holds `antlr4-runtime` 4.7.2 **and** 4.13.2. If the old one wins,
  every parse dies in class-initialisation with a version mismatch. Pinned jars
  go first and 4.7.x is filtered out.
* Multiple SLF4J providers are present; Logback is excluded so the binding is
  unambiguous.

Override with `FREES_HOME`, `FREES_CORE_JAR`, or `GRADLE_CACHE`.

## Fixture format

```json
{
  "name": "canonical",
  "source": "x^2 + y^3 = 77\nx/y = 1.23456\n",
  "expect": {
    "variables":     { "x": 4.694012391660914, "y": 3.802174371161316 },
    "display_names": { "x": "x", "y": "y" },
    "block_count": 1,
    "error": null
  },
  "oracle": { "engine": "frEES backend/core (Java)", "generated_by": "tools/golden-dumper" }
}
```

Two optional top-level fields sit beside `source`, each the verbatim body of
the like-named sidecar in `fixtures/corpus/` and each recorded *into* the
golden by the oracle that used it: **`function_tables`** (the request-level
Function Tables) and **`request`** (the stop criteria and variable specs) —
both described at the top of this file. Absent means the plain
`solve(source, DEFAULTS, Map.of(), Map.of())`, which is what 1281 of the 1306
fixtures are.

A document that **fails** is as valuable as one that solves — the Rust engine
must fail the same way:

```json
"error": { "type": "SolverException", "message": "There are 2 equations and 1 variables. …" }
```

## Comparison policy

Compare by **relative tolerance, never bit-equality**. WASM `f64` arithmetic is
IEEE-754 deterministic, but transcendentals (`exp`, `ln`, `pow`, `sin`) are
unspecified and differ between the JVM and Rust's libm.

| Field | Rule |
|---|---|
| `variables` | relative tolerance `1e-9`; absolute `1e-12` near zero. A named fixture may declare a looser *relative* tolerance in `fixtures/tolerances.json` — see below. A *named variable of a named fixture* may instead be graded by a declared **absolute** tolerance, for a quantity whose true value is a structurally exact zero — see *The absolute channel* below |
| `display_names` | exact |
| `block_count` | exact — a different blocking is a real divergence |
| `error` | `type` exact; `message` **not** compared verbatim (see below) |
| `ode_tables` | one entry per `DYNAMIC` block, in declaration order. `name`, `method`, `columns`, `stopped` and each event's `name` are **exact**; `end_time` and each event `time` take the same numeric tolerance as `variables`. Row cells take that tolerance **plus a scale anchor**: a cell also passes when `\|a − e\| ≤ rel_tol · scale`, where `scale` is the max `\|expected\|` over that signal's own trajectory (`time` excluded) — see *The decayed-signal measure* below |

### A solution must be finite — the replay's own blind spot (Wave T3)

Every fixture that solves is also checked for non-finite values, and a `NaN` or
an infinity in `Solution::values` fails the gate naming the fixture and the
variables.

**This is not implied by matching the golden**, which is exactly why it is
asserted separately. `close` and `rel_diff` treat `NaN` against `NaN` as
agreement and infinities as exact hits — deliberately, because a golden that
records the oracle's own non-finite answer has to be gradable at all — so a
fixture could match its golden while this engine produced garbage. Nothing else
in the corpus would notice.

It arrived here from `props_robustness`, where asserting it cost a **second**
whole-corpus solve in the CI job that was already the workflow's critical path
(946.90 s against the replay's 986.41 s, same debug profile, same box — the two
passes were the same size). The replay already solves every fixture, so here it
is free, and it is graded exactly once across the shards' union like every other
per-fixture guard.

One difference from the version that lived in `props_robustness` is deliberate.
That one always solved with `SolverSettings::default()`, so for the fixtures
carrying a `.request.json` it graded a configuration the document was never
meant to run under — and its `if let Ok(solution)` silently skipped any that
then failed to solve at all. Here each fixture is checked under the settings it
ships with.

### `ode_tables` — why a transient fixture needs it

**A solved `DYNAMIC` block puts nothing in `variables`.** The trajectory is a
first-class ODE Table, so a transient document's `variables` map holds only its
analytic parameters — `dyn_plain_ode` has exactly `{k, Tinf}` in it. A fixture
compared on `variables` alone therefore passes **vacuously**: the integration
could be wrong, or missing entirely, and the gate would stay green.

A golden dumped *before* the dumper grew this section has no `ode_tables` key at
all, which the replay treats as "not a claim" rather than "no tables" — but if
the Rust engine produces a table where the golden makes no claim, the fixture
**fails** with a "re-dump this fixture" message. Absence is never allowed to hide
a transient.

The comparison was validated the way the harness itself was in Phase 1, by
perturbing a golden and watching the gate go red. All four perturbation classes
were observed and then reverted:

| Perturbation | Reported as |
|---|---|
| a row cell `47.59095803046333` → `47.6` | ``ode_tables[0] `cooling` rows row 1 col `temp` = … but Java got 47.6 (rel 1.9e-4, tolerance 1e-9)`` |
| the whole table deleted | `Java recorded 0 ODE table(s), Rust produced 1` |
| `method` `ode45` → `ode23` | ``ode_tables[0] `ascent` method = "ode45" but Java got "ode23"`` |
| a stop event's `time` shifted by 0.5 s | ``ode_tables[0] `ascent` events hit 0 (`apogee`) fired at 156.74… but Java got 157.24… (rel 3.2e-3)`` |

#### The decayed-signal measure (Wave G1, 2026-08-23)

A pointwise relative measure has no denominator on a signal that decays
through zero: `pressure-cooker`'s `steel$port$qdot` falls from 1 497.86 W to
2.27e-10 W, where a 4.4e-5 W absolute agreement — 2.9e-8 of the signal — reads
as rel 5.7e-3 purely by denominator collapse. The digits a pointwise measure
demands there were never *controlled* by either engine: both integrators run
error control of the `atol + rtol·|y|` shape (the IDA path at rtol `1e-6` /
atol `1e-8`), so the trailing digits of a decayed tail are integration noise
on both sides. Anchoring the tolerance to the signal's own dynamic range —
a row cell also passes when `|a − e| ≤ rel_tol · max|expected|` over its
column — is the same measure the integrators themselves use, and changes
nothing for a healthy signal, where `|e| ≈ scale`. Three deliberate limits:
the anchor uses the **expected** side's range only, so a wrongly-huge Rust
value cannot widen its own gate; the `time` column is excluded (sample times
are grid structure, not integrated state); and a column whose golden is all
zeros keeps the pointwise measure. This is a *measure correction*, not a
relaxation — it was validated red the same way the row comparison itself was
(perturbing the `steel$port$qdot` tail to 0.02 fails at `scaled 1.3e-5` of the
column range, and the classic `47.59… → 47.6` perturbation above still fails
at `scaled 9.5e-5`; both observed in one 2/775 run on 2026-08-23, then
restored). The measure is what promoted `sysdesign-ex01-thermal-network-2`,
`ev-battery-cooling-pid` and `pressure-cooker` — and with it the dead-entry
guard now folds ODE cells into "passes at the default", because a transient
fixture's divergence lives in its table, not its variables.

### The tolerance files — the one relaxation, and its guards

Decision D1 resolved real-fluid properties from precomputed `(P,h)` tables whose
measured error against native CoolProp is `1e-7…1e-4` relative, while the
goldens hold values the native library produced. **No table-backed engine can
pass a `1e-9` gate on a document that calls a real-fluid property function** —
the gap is a property of the artifact, not a bug waiting to be found. D1 named
two honest ways out (a per-fixture tolerance, or shipping CoolProp itself as the
accuracy path). **Both have now been taken**, and taking the second is what
shrank the first: under rustprop that whole error class is gone, so thirteen of
the 23 entries below are dead and the file that grades what ships had ten.
*(Twelve since Wave G1, 2026-08-23: the first two **transient** entries —
`ev-battery-cooling-pid`, whose golden trajectory integrated the Java's
`(P,Hmass)→T` table into the RHS (the `oracle-ph-table` mechanism, leaf-probed
at the trajectory's own steady state), and `pressure-cooker`, graded by the
new `ida-adaptive-path` mechanism. Both `measured` values are scale-anchored
binding errors; the entries' reasons carry the evidence.)*

A tolerance entry relaxes the *numeric* tolerance for a named fixture and
nothing else —
`display_names`, `block_count` and the error classification are still exact for
every fixture in the corpus, and every fixture not named there is still held to
`1e-9`. Three guards stop it becoming a place to hide failures:

* a fixture named there but absent from `fixtures/golden/` **fails the gate**;
* a fixture named there that **passes at the default** fails too, so a tolerance
  that is no longer needed cannot sit in the file pretending it is — and since
  Wave G1 "passes at the default" is judged over everything numeric the fixture
  grades, ODE row cells (under the decayed-signal measure) included, so a
  transient entry dies as loudly as a scalar one;
* if the file catalogues its `mechanisms` — `tolerances-rustprop.json` does,
  since Wave-3 F5 — every entry must name a slug the catalogue defines **and
  every catalogued slug must be named by an entry**. The second direction is the
  one that earns its keep: a mechanism whose last instance dies leaves prose
  behind that still reads like a live description of the build, which is exactly
  how the retired stop-criterion mechanism survived a whole wave after its only
  fixture reached the engine default. A dead *explanation* now fails like a dead
  tolerance.

Each entry must record the *measured* error and a `reason` naming the mechanism
that produces it. The parity test prints which fixtures used a declared
tolerance on every run.

#### One file per backend (D9)

Since [D9](../docs/decisions/0009-rustprop-backend.md) there are **two** of these
files and the harness reads exactly one, selected by the same `rustprop-backend`
cfg that selects the property backend:

| file | grades | entries |
|---|---|---:|
| `tolerances.json` | the D1 `(P,h)` `TableBackend` | 23 |
| `tolerances-rustprop.json` | rustprop, the accuracy path — what the wasm ships | 65 relative + 10 absolute (11 variables) |

They cannot be merged, because the "dead tolerance" guard above is what makes
them backend-specific: thirteen of the 23 entries exist only because of the
tables' own interpolation error, and under rustprop — which *is* CoolProp 8.0.0
— those fixtures match at `1e-11…1e-16` and their entries would fail.

*(The rustprop file's count is a corpus-growth number, not a quality one: F5
re-baselined it at **ten**, Wave G1 added the first two transients, Wave G4's
component sweep added eleven, Wave A1 (2026-08-24) added fourteen from the
Wave-I two-phase-cycle harvest and Wave C1 one for CO2, Wave P2 the same day
added eighteen from the Wave-J class sweep, Wave P1 opened the `absolute`
section and brought eight documents in with both kinds of entry, and Wave Q1
(2026-08-25) closed the last two held documents — and then **Wave R2 killed
one**, the first entry this file has ever lost to an *engine* change rather
than to a re-measurement: `components_g4_radiator`'s worst variable went
1.2042e-6 → **1.2947e-14** once the ported `props_si` cache made a repeated
property call return the identical double, so its eleven-digit cancellation
became exact and the dead-tolerance guard retired the entry. **65 relative +
10 absolute**. Counted by mechanism: 56 relative and 9 absolute are
`oracle-ph-table`, i.e. the Java's own run-time `(P,Hmass)` interpolation
table; 5 are `ida-adaptive-path`, 3 `upstream-ps-flash-residual`, 1 the
`ode45-adaptive-path` Wave P2 catalogued and 1 the `smooth-clamp-regulariser`
Wave Q1 did. Each entry carries its own evidence in `reason`; A1's fourteen,
P2's nineteen, P1's eight and Q1's two were every one leaf-probed against the
CoolProp 8.0.0 wheel at the Java's own inputs before they were written.)*

#### Running the gate — the corpus has exactly one servable backend

Since Wave-3 F6/F8 the corpus is **rustprop-graded by construction**. Twelve of
the corpus documents (722 as of 2026-08-21) ask for things the `(P,h)`
`TableBackend` cannot serve *at all* — `HAPropsSI` (the seven humid-air documents), single-phase `(P,T)`
transport (`hx-correlations-fluid`), `CompressibilityFactor`
(`thermo-compliance`) and `Air` `Enthalpy` (the three pneumatic documents) — so
`tolerances.json` no longer has a configuration that replays the corpus.

Use either of these; both grade the whole corpus through rustprop:

```bash
cargo test --workspace --test parity                              # what CI runs
cargo test -p frees-core --features rustprop-backend --test parity
```

The first needs nothing turned on because `frees` requires the feature and
resolver-v2 unifies it onto `frees-core`; the run prints *"&lt;count&gt; fixtures
match the Java oracle through rustprop (CoolProp 8.0.0)"* with the live corpus
count.

#### Sharding the gate (Wave Q2, 2026-08-25)

The replay is the longest gate in the project and it grows with the corpus, so
it can be split across processes by two environment variables — `PARITY_SHARD_COUNT`
and `PARITY_SHARD_INDEX` (`0 <= index < count`):

```bash
PARITY_SHARD_COUNT=4 PARITY_SHARD_INDEX=1 cargo test --workspace --test parity -- --nocapture
```

With neither set the replay is exactly what it always was — one process, the
whole corpus. With both set, this process replays the fixtures at
`i % count == index` of the *sorted* golden listing, which is a partition: every
fixture is in exactly one shard and the union is the corpus. Every run prints a
census line, `parity-shard: index=… count=… replayed=… corpus=…`, so the union
is checked rather than assumed — sum `replayed` over the shards and it must
equal `corpus`. CI runs four shards and fails the workflow if any of them fails.

**The gate is not weaker across the union**, and `tests/parity.rs`'s module docs
carry the argument in full. In one paragraph: the "declared here but no such
fixture" sweep is a property of the corpus, so it still reads the whole
directory listing in *every* shard (it opens no fixture, so repeating it is
free), and the `mechanisms` catalogue check reads only the tolerance file and is
likewise whole-file everywhere. The guards that need a replayed value — a dead
relative tolerance, a dead `solver_floor`, a dead or misspelled `absolute`
entry — run where their fixture runs, which is exactly one shard, so each is
graded exactly once across the union. Every under-replaying configuration
(half-set variables, a zero or non-numeric count, an out-of-range index, a
stride that selects nothing) panics rather than reporting green.

**One fixture was 61 % of this gate, and then the lever was pulled.** Timed per
fixture on 2026-08-25 (release, 317.4 s inside the replay loop),
`ev-battery-cooling-pid` alone cost 193.0 s and the top 20 of 1281 accounted for
89.4 % — reproduced by a second independent run at 191.3 s of 316.8 s (60.4 %).
No partition splits a fixture, so 193 s was the floor for any shard count, and
the measured longest shard was 273.8 s at N=2, 227.7 s at N=4 and 202.7 s at
N=8. The closing sentence of that paragraph — *"if this job needs to get
genuinely faster, the lever is that one transient, not more runners"* — is what
Wave R2 then did.

**Re-measured 2026-08-26 at HEAD** (release, quiet box), after R2's `props_si`
cache and R3's slot-native prepared block: the whole corpus is **126.98 s**
unsharded, `ev-battery-cooling-pid`'s marginal cost inside the replay loop is
**45.86 s** (36 %, from 61 %), and the four shards timed one at a time are
**11.97 / 17.36 / 46.35 / 52.49 s**. Compare the two sets by share rather than
by seconds — they are different machines, which is what the 317.4 s against
126.98 s for the same corpus says — and the "no partition splits a fixture"
floor is now ~36 % of the gate where it was 61 %. Four shards are kept — the
growth-headroom argument above is the one that was always load-bearing, and in
CI's own debug profile the four legs of the S1 run took 0m57s / 1m27s / 1m35s /
3m43s.

**But sharding was never what decided this workflow's wall clock**, which is
the finding of 2026-08-26. The `native` job ran `cargo test --workspace`, and
`tests/parity.rs` is an ordinary integration test of `frees-core`, so that job
replayed the *whole* corpus, unsharded, in the same debug profile, in a **fifth
job** — 14m26s of a 15m09s step while the four shards were finishing in under
four minutes. It now skips the replay by name, and note what that does **not**
do, against the `required-features` rule below: the harness is unchanged,
`cargo test --workspace` still replays the corpus everywhere else including on
every developer's machine, and the four shards still replay it in the same
workflow with a census that must sum to `corpus`. The gate is skipped in one
*job*, not made skippable in the *test*.

**That was only half of what the job was paying**, and the other half went the
same way one commit later (Wave T3).
`props_robustness::no_promoted_fixture_solves_to_a_non_finite_value` solved
every document in `fixtures/corpus` a second time — one thread, same debug
profile — and it was the same size as the replay, not a rounding error:
**946.90 s against the replay's 986.41 s** in that profile on a dev box. The
assertion moved into this replay rather than being skipped or deleted, for
three reasons: the replay already solves every fixture, so it is free here; it
runs in the four shards, so it is still asserted exactly once across their
union; and it closes a blind spot the replay had on its own — see *A solution
must be finite* under **Comparison policy**. Nothing else walks the corpus and
solves it: `matrix_expansion` only parses, and `dynamics_robustness` only reads
`points =` out of the source text.

**The single-package form without the feature refuses, on purpose.**
`cargo test -p frees-core` (or `… --test parity`) does not unify anything, and
until Wave-4 F9 it failed on those twelve with an error that named nothing: a
property error inside Newton becomes a `NaN` residual — faithfully, that is what
the Java's `NewtonSolver.residuals()` does — so all twelve came back as *"Newton
iteration stalled after 0 iteration(s) … (norm NaN)"*, and neither `HAPropsSI`
nor "not a tabulated output" appeared anywhere in the output. `tests/parity.rs`
now checks the backend before it replays anything and prints the two commands
above. Only the corpus replay declines — measured with `--no-fail-fast`, the
other 24 `frees-core` test targets run and pass there, 2,986 tests.

It is a runtime check rather than `required-features` on the test target for one
reason, and it is this file's own reason: `required-features` would make
`cargo test -p frees-core` **skip** the gate and report green. The three guards
above exist to stop a gate quietly asserting nothing, and a gate that can
disappear from a run is the same failure one level up.

#### The ten that survive, and why none of them is port error

They have a different cause from the thirteen, and **Wave-3 F5 re-measured every
one of them and proved it**. Nine are the *golden* side:
`PropertyFunctions.java` asks its own `PhTableRegistry` before it asks CoolProp,
and that registry answers whenever the output is `T`/`Dmass`/`Smass` and the
input pair is `(P, Hmass)`, from a run-time 256-point saturation curve and a
96×48 grid gated only at `1e-4`. So those goldens are interpolated values, up to
`6.7e-6` from what CoolProp 8.0.0 actually returns at the same inputs — while
rustprop reproduces the wheel there to between 0 (bit-identical, in three of the
nine) and `2.1e-13`. The tenth, `refrigeration-vcr`, contains no table shape at
all: its golden *is* a CoolProp value, from a `(P, s)` flash that upstream
stopped `4.06e-9` (in pressure) short of the state that was asked for. Both
mechanisms, the evidence for each, and the per-fixture amplification factors are
in that file's `mechanisms` catalogue.

`tolerances-rustprop.json` also carries a second section, `solver_floor`, which
relaxes the Newton **stop criterion** for a named fixture rather than the
comparison tolerance. It exists for one mechanism: rustprop answers an inverse
property call with an iterative flash, so a residual like
`T_out = Temperature(fluid, P, h)` can advance in jumps rather than along a
slope, and a line search cannot descend a staircase. It carries the same two
guards as a numeric tolerance — a fixture that converges at the default, or an
entry with no fixture, fails the gate — and the *values* are still compared
normally, so relaxing it asserts strictly more than pinning the fixture as a
known divergence would. **It is currently empty**: its only instance died at
Wave-2 integration when upstream's Boost TOMS748 replaced the bisection stand-in
that produced the staircase.

The same applies to D7's `FRAUX1` grids, which have their own error class
(`tools/aux-gen/README.md`). Two entries **in `tolerances.json`** come from them.
Neither *mechanism* survives into the other file, because rustprop answers
`INCOMP::MEG` exactly: `sysdesign-ex11-liquid-cooling-loop` drops out
altogether, and `ev-thermal-management` is still listed but for a completely
different reason — 178× better at `5.039e-6`, now the Java oracle's own
`(P,h)` table on the *refrigerant* side, amplified 22× by an isentropic
compressor work term.

The `tolerances.json` figure is worth reading before it is copied as precedent:
`ev-thermal-management` sits there at
`2e-3` (measured `8.951e-4`) **not** because the glycol grid is that inaccurate
— its `Dmass` is `1.6e-5` — but because the document's `htc_1phase` call runs at
`Re = 2987`, dead centre of `nuSinglePhase`'s 2300..4000 laminar↔turbulent
blend, where Nu sweeps 3.66 → ~30 and a `5e-4` viscosity error is amplified onto
the steepest part of the correlation. The *operating point* sets that number:
the same grid grades `sysdesign-ex11-liquid-cooling-loop`, which sits off the
blend, at `1.310e-4`. A future fixture with a similar tolerance should be able
to point at a similarly specific mechanism, or it is hiding something.

#### The absolute channel — an exact zero has no denominator (Wave P1, 2026-08-24)

`tolerances-rustprop.json` carries a third graded section, `absolute`, and it
exists for a shape a relative measure cannot express at all: a quantity that is
**identically zero by physics**. A condenser outlet still inside the dome makes
`SC = Tcond − Temperature(P,h)` exactly 0; an evaporator outlet below `hg` makes
`SH = Temperature(P,h) − Tsat` exactly 0. In every one of the eight fixtures that
use the channel, the CoolProp 8.0.0 wheel returns *the same `f64`* for
`Temperature(P,h_out)` and for `T_sat(P)` — difference exactly `0.0`, not merely
small — and the golden's own `Tcond`/`tsat` is that same bit pattern. This engine returns `0` or one ulp of a
~310 K temperature (`±5.7e-14`). The Java answers the same call from its
`(P,Hmass)` interpolation table and returns `2.8e-7 … 1.5e-6` K, so **the golden
asserts the oracle's own table error**, every such variable reads `rel ≈ 1.0` by
denominator collapse, and no `relative` the harness accepts (`> 1e-9`, `< 1e-2`)
can pass it. Like the decayed-signal measure above this is a *measure
correction*, not a relaxation — it grades the thing that is actually being
compared.

It is deliberately **narrower than a relative entry in two ways**. It is **per
variable**: an entry names the variables it covers, so every other variable in
the document stays on the ordinary relative tolerance and a second divergence
elsewhere still fails. `chgclosed-condensing-pressure-floats-with-ambient-and-charge`
shows why that matters: `cnd.sc` is one of 21 variables there, and one number
applied to all 21 at the ceiling would forgive `1e-4` kg/m³ on `cnd.rho_in`,
whose **real** divergence is `4.48e-5` kg/m³ — the `(P,Hmass)→Dmass` gap the
fixture exists to grade, passing silently. And it grades `variables` **only**:
ODE row cells, `end_time` and event times keep the relative measure plus the
Wave-G1 scale anchor, which is the same idea applied to a trajectory. All ten
fixtures carry *both* kinds of entry — one variable absolute (two, in
`two-phase-distributed-…-multi-cell-coil`), everything else on a measured
`relative`.

**Wave Q1 (2026-08-25) added the other kind of exact zero, and the channel's
first entry not in kelvin.** Ten of the eleven covered variables are the `SC`/`SH`
shape above, where the zero comes from the *physics* and the golden is the
**oracle's** table error. `ev-tms-api-model-…-plain-solver`'s `rada.q_lat` is the
latent duty of a **dry** moist-air coil, where the zero comes from the *model*:
`MoistAirWallHX` clamps `out.W` at saturation with a C¹ smooth min,
`out.W = 0.5·(in.W + W_sat − √((in.W − W_sat)² + 1e-12))`, and on a dry coil the
exact min is `in.W`, so the true latent duty is `0` W and what both engines report
is the `1e-12` **regulariser** — `in.W − out.W = 2.5e-13 / (W_sat − in.W)`,
recovered by cancelling ~11 digits off a Newton-solved `out.W`. The two engines
differ there by exactly **16 ulps** of `out.W`, which `mdot·2.501e6` turns into
`6.247503e-11` W. The mechanism is catalogued as `smooth-clamp-regulariser`, the
unit is **W**, and the `2·|golden|` bound is not what limits it (the golden is
`1.14e-5` W, so the bound is `2.3e-5`) — the measured-×1.5 rule is. The `1e-4`
ceiling argued in kelvin below still binds it, six orders clear.

**And the same document's second dry coil is the sharpest thing the channel has
produced, because the channel refused it.** `conda.q_lat` is the identical shape,
and a first draft of Wave Q1 put it on the channel too. The gate rejected it —
*"declares an absolute tolerance for `conda.q_lat`, which passes at the fixture's
relative tolerance 3.9e-6 (rel 7.783161130893312e-7)"* — because in the **full
1283-fixture replay** it lands **one** ulp of `out.W` from the golden, while in a
fresh process replaying that document **alone** the same binary lands **eight**
(rel 6.2265e-6, a 8× swing with nothing changed but what ran before it).
The swing is what was **measured**; the identified candidate — not an isolated
cause — is `crates/frees-core/src/props/rustprop_warm.rs`, which seeds every
`(P,h)`/`(P,s)` flash from the previous answer, so a warm-started flash can
converge on a different last ulp than a cold one, and an eleven-digit cancellation
turns that into the leading digits of `q_lat`. That is Wave P2's *"a relative tolerance there would pin luck"*
reproduced, with the confound identified: it is the **run**, not the build — all
three build configurations agree when run the same way. `rada.q_lat` is 16 ulps in
**both** contexts, which is why it is the one graded absolutely and `conda.q_lat`
stays on the relative channel.

Its guards mirror the relative ones, with two the relative channel cannot have:

* an entry whose fixture is absent from `fixtures/golden/` fails; an entry naming
  a variable the golden does not have fails; so does one the replay never
  reaches;
* a covered variable that **passes at its fixture's relative tolerance** fails —
  a dead channel dies like a dead tolerance;
* `1e-12 < absolute < 1e-4`. Below the floor the harness already accepts the
  difference. The ceiling is argued in kelvin because that is the unit every
  instance is in: the smallest *legitimately* non-zero superheat this corpus
  grades is `0.2522` K
  (`chiller-higher-refrigerant-flow-delivers-more-cooling-2`, held open by a zone
  ramp and agreed to four figures by both engines), so `1e-4` K is 2 500× below
  the smallest real signal of this kind — and it also sits under the worst
  `(P,Hmass)→T` table error the corpus has measured (`1.53e-4 … 1.56e-4` K, the
  three chiller entries), so an entry needing more is claiming a bigger oracle
  artifact than any yet measured and owes fresh evidence, not a bigger number.
  The ten kelvin instances sit 44×…240× under it, and Wave Q1's watt instance
  (9.4e-11 W) a further four orders below that;
* `absolute ≤ 2 · |golden|` for that variable, checked at replay. This is what
  makes the channel self-limiting: where the true value is zero the golden **is**
  the oracle's error, so forgiving more than twice it stops forgiving the
  oracle's artifact and starts hiding this engine's — and the channel can never
  be pointed at a healthy variable in order to widen it. The measured-×1.5 rule
  leaves ~33 % of slack under the bound. One limit follows and is accepted: a
  golden of exactly `0.0` admits no entry at all, because if the *port* returns
  more than `1e-12` where the oracle returns a true zero, the artifact is on this
  side;
* a covered variable is kept **out of** the dead-relative-tolerance guard's
  `worst`, since its `rel ≈ 1.0` would keep a dead `relative` on the same fixture
  looking alive for ever.

Validated red before it was used, the way the `ode_tables` comparison and the G1
measure were: four deliberate breakages in one run on 2026-08-24 — a tolerance
tightened under the measured gap, one raised over the `2 · |golden|` bound, one
pointed at a healthy variable, one pointed at a variable that does not exist —
each producing its own message in one `4/1257` run, then restored.
`tests/parity.rs`'s module docs quote all four verbatim. The fourth also proved
the channel is load-bearing rather than decorative: with its entry pointed at a
misspelled name, the variable it should have covered fell through to the
ordinary measure and failed there —

```text
  [accomp-air-coil-cools-and-dehumidifies] `coil.ev.sh` = 0 but Java got
  -0.0000012156851880718025 (rel 1e0, tolerance 2e-7)
```

Error *messages* are not compared literally. The Java engine emits long,
domain-specific guidance ("A common cause: an element chain with no constitutive
law for that quantity…"). Matching that prose is not the goal; matching the
*classification* is. Assert the error type and that the message names the same
offending variables.

## Ground truth this corpus already pinned down

Behaviours that would otherwise have been guesses, now settled by running the
oracle rather than reading the source:

| Fixture | Establishes |
|---|---|
| `arithmetic` | `2^3^2 = 512` — exponentiation is **right**-associative. `-2^2 = -4` — unary minus binds **looser** than `^`. `-` and `/` are left-associative. |
| `units_negative_celsius` | `-10 [C]` = **263.15 K**, not −283.15. The unary sign folds into the literal *before* unit conversion (`AstBuilder.bareUnitLiteral`). |
| `units_pressure` | `140 [kPa]` = `140000.0` — SI conversion happens at **parse** time. |
| `units_temperature` | `25 [C]` = 298.15, `32 [F]` = 273.15 — additive offsets, `F` factor 5/9. |
| `intrinsics` | `sin(0)=0`, `cos(0)=1` — trig takes **radians**. |
| `constants` | `pi# = 3.141592653589793`, `R# = 8.314462618`, `g# = 9.80665`. |
| `case_insensitive` | `Tin`/`TIN`/`tin` are one variable; the result key keeps the **first-seen** spelling, and `display_names` maps lowercase → that spelling. |
| `sequential` | Blocks solve in **dependency** order, not source order (3 blocks). |
| `overdetermined` / `underdetermined` | Both are `SolverException`, and the message names the specific redundant relation / free quantity. |
| `empty` | A comment-only document is `SolverException: "No equations to solve."`, not success-with-nothing. |

## `fixtures/dae-oracle.json` — the API-level oracle

The odd one out in the other direction: not a document replay, but a dump of the
Java **DAE API** driven directly.

```bash
tools/dae-probe/run.sh            # -> fixtures/dae-oracle.json
```

It exists because a `.frees` document has no way to reach the implicit-DAE path
until the `DYNAMIC` grammar lands (`method = ida` on a `DYNAMIC` block is what
selects it), so `tools/golden-dumper` cannot produce ground truth for it. The
probe instead calls `IdaDaeSolver`, `DaeJacobian` and `SparseSteadyKlu` on
analytic problems — Newton cooling, a semi-explicit index-1 pair, Robertson
(with and without root finding), a 31-unknown C-R-C heat chain above the sparse
threshold, and a coupled algebraic loop — and records the trajectories,
consistent initial conditions, root times/directions, Jacobians, colourings and
sparse solves.

`crates/frees-core/src/dae/solver_tests.rs` and `dae/jacobian.rs` embed those
numbers as their `ORACLE_*` constants. **Re-run the probe rather than editing a
constant.**

> **It needs SUNDIALS ≥ 6.** `libsundials_ida.so.6` (the `SUNContext` API) must
> be loadable, plus `sunmatrixsparse`/`sunlinsolklu` for the KLU case. Without
> them the probe writes `"available": false` and exits 0 — the same graceful
> degradation `SundialsIda.isAvailable()` gives the Java engine — so a machine
> without SUNDIALS cannot silently mint an empty oracle over a real one.
>
> **Re-verified 2026-08-23 (Wave G6):** this machine's `libsundials-dev`
> 6.4.1 (Ubuntu 24.04, with `sunlinsolklu` + SuiteSparse `libklu.so.2`) runs
> the probe and reproduces the committed file **bit-identically** — the
> `ORACLE_*` constants are re-measurable, not frozen. The input space around
> these fixed cases is fuzzed by `crates/frees-core/tests/dae_robustness.rs`.

## Growing the corpus

The 17 Phase-1 seeds (scalar equations, units, precedence, blocking, error
paths) have been joined by the harvested example documents, the hand-authored
Phase-4 cases that passed promotion, and the adversarial probe documents written
to hunt Java divergences (matrix naming and determinants, PROCEDURE/FUNCTION/
MODULE scoping, TABLE log-space and family interpolation, `Integral` accuracy
and its variable pin, solver bounds/root selection, units, operator precedence).
See **Pending corpus** below for the staging area and what is still blocked.
Extend it further from, in rough order of value:

1. `web/src/examples.ts` + `web/src/defaultExample.ts` — **harvested and
   exhausted**: 47 documents (46 `Example.id`s plus `rankine-cycle-2`, the
   duplicate id), **44 promoted, 3 pending** — `pressure-cooker`,
   `estimator-gramian-balreal`, `ev-battery-cooling-pid`. *(Re-counted from the
   files on 2026-08-18. It read "41 promoted, 6 pending": two of those six,
   `hx-correlations-fluid` and `thermo-compliance`, were promoted by Wave-3 F6
   below, and the figure had already drifted by one before that.)*
2. `../frEES/frontend/src/docs/*.md` (documented snippets) — **harvested
   2026-08-22 (Wave D1 slice)**: 142 fenced blocks across 14 files, 129
   candidates after filtering (1 was already in the corpus verbatim), the
   oracle solved 44, **43 promoted** (`docs_*`), 1 pending
   (`docs_fluids_materials_03`, the CO2 linkage hold above). The harvest also
   caught two real engine divergences, both fixed the same day: the
   component-free early return skipped the Java's dotted-name mangling
   (`docs_components_05`), and display names were over-registered for
   `Convert`/`ConvertTemp` unit tokens and matrix-library temporaries
   (`docs_language_fundamentals_12`/`_13`, `docs_matrix_algebra_03`). The 85
   non-solving blocks classify as: 24 Python-style `#` prose comments, 21
   underspecified fragments (an equation shown without its inputs), 14
   display-form slice syntax (`v[1:3]` in prose the grammar rejects), 11
   declaration-only fragments, 10 other pseudo-syntax, 3 other solver
   refusals, 2 prose ellipses (`..` as continuation). They are doc
   *illustrations*, not documents — none is worth a fixture
3. the Java test classes — **harvested (Phase 12)**:
   `tools/harvest-java-tests/harvest.py` extracted 212 candidates from 13
   classes (both `"""` text blocks and `\n`-concatenated `solve(...)`
   arguments, resolving same-file constants and locals); 191 survived golden
   review, **170 promoted**, 21 pending with classified blockers *(as of the
   Phase-12 harvest; **9 of those 21 remain** — twelve have been promoted since,
   four on 2026-08-06 and eight by the rustprop backend, 5 in Wave-3 F6 and 3 in
   Wave-3 F8, all recorded at the end of this file)*. ~~The
   remaining Java documents are the ones the harvester cannot represent:
   tests that pass extra `solve(...)` arguments (complex mode, `ProcDef`
   function tables), `String.format` templates, and cross-file constants~~
   *(Re-harvested 2026-08-24, Wave I — the representable-document boundary,
   status-phase12 "did not deliver" item 1. The resolver grew: `.formatted`/
   `String.format` with literal arguments, in-file helper-method inlining
   (parameter binding per literal-argument call site), a cross-file
   `static final String` registry, and `ProcDef.FunctionTableDef` extra-defs
   evaluated into `.tables.json` sidecars (the `function_tables` chain above).
   An inventory over **all 197** test classes measured the blockers: of 540
   solve-call documents, 504 now resolve; blocker (b) was 8 direct-`formatted`
   + 44 helper-carried documents, blocker (a-tables) 11 call sites — all in
   `CurveFunctionTest` — and blocker **(c), cross-file constants, is empty**:
   the registry resolves nothing because no solve site references another
   class's constants. Unresolvable remainder: 36 call sites (12
   CoolProp-computed template arguments — `he0 = propsSI(...)` — 10 unknown
   idents, 14 builder loops). Ten classes joined `CLASSES` and 99 candidates
   staged (65 from classes + 34 validation resources, item 5 below):
   **77 promoted** — corpus 905 → 982 on this branch (983 merged beside Wave I's CabinZone promotion), including all 10 `curvefn` request-table
   fixtures at the corpus default, bit-exact on the Java tests' own asserted
   values — 18 pending (property-chain holds, see the pending table; **14 of
   the 18 promoted 2026-08-24, Wave A1**, at declared `oracle-ph-table`
   tolerances measured ×1.5 and leaf-probed against the CoolProp 8.0.0 wheel —
   corpus 983 → 997 — leaving the 4 whose true answer is a structurally exact
   zero) and 4
   dropped as new witnesses of ledger item 35 (`multiout-*-tilde-*`: the
   oracle's JVM-batch-global `~ignored~N` sink counter makes their goldens
   unreproducible — unfreezable by design, not pending). The remaining
   un-harvested test-class documents are now dominated by ~100 classes that
   were simply never in `CLASSES` (about 340 text blocks and 400+ resolvable
   solve-call docs tree-wide, many duplicated across both counts and against
   the existing corpus) — a sweep-sized follow-up, not a representability
   problem; the truly unrepresentable are the 9 a-complex sites (complex mode:
   count-only, out of scope), 28 a-specs sites (`VariableSpec` guess/bounds
   overrides — a fixture-format gap nothing currently needs), 10 a-settings
   sites, and the 36 unresolvable arguments above.)*

   **The sweep is done — 2026-08-24, Wave J. Corpus 983 → 1238.** Wave I's
   correction was right and its estimate was close: the real figures, from
   `harvest.py --inventory` over the whole tree, are **138 of the 197 test
   classes hold a harvestable document, 23 were named in `CLASSES`, and 115
   were never listed at all** (114 are now swept by name; `ValidationSuiteTest`
   is the one `SKIP_CLASSES` entry, because item 5 below already harvests its
   documents verbatim from the resource directory) — 540 solve-call sites
   (504 resolve) and 337
   text blocks tree-wide, of which 307 sites and 235 blocks sat outside
   `CLASSES`. So the class list stopped being the sweep: `swept_classes()`
   now walks `JAVA_TEST_ROOT` and harvests **every** class, `CLASSES` only
   pins the prefix (and extraction preference) for the pre-Wave-J names so a
   re-run cannot restage the promoted corpus under new stems, and a class
   derives `component-moist-air`-style prefixes from its own name otherwise.
   Four guards joined the existing ones. `SKIP_SITE_TAGS` drops the sites
   the Wave-I inventory classified as unrepresentable (a-complex / a-specs /
   a-settings — 37 sites; their default-settings golden would not be the
   answer the Java test asserts). `MARKUP_RE` drops a candidate whose first
   content line opens a tag (`VectorExportTest`'s asserted `<svg …>` reaches
   the fragment guard with an `=` in it — an XML attribute). `normalize`
   adds a layout-insensitive duplicate check on top of the exact one (fold
   case, collapse spaces *within* a line, never across lines — newlines are
   syntax here). And `MAX_DECLARED_POINTS` refuses a `DYNAMIC` grid denser
   than 2 000 samples, which is a **fixture** limit, not an engine one: the
   engine's ceiling is 100 000, but the golden stores every cell, and the
   three Wave-J documents above the line cost 1.6 MB, 3.1 MB and — at
   `points = 60001` — **139 MB**, against 640 KB for the largest golden the
   corpus had. They also broke
   `dynamics_robustness::the_corpus_sample_counts_are_far_below_the_ceiling`,
   which pins the whole corpus a factor of ten under that ceiling; 2 000
   keeps that property and excludes nothing else, because the densest
   document either side of the sweep declares 1 201. `DROPPED` records the
   four ledger-35 witnesses by name so golden review's permanent rejections
   are not re-minted every run.

   **285 candidates staged, 284 goldened, 255 promoted, 19 pending, 11
   classified below** — *18 of the 19 promoted 2026-08-24 by Wave P2, so this
   harvest is now 273 promoted and 1 pending; the probe, and why the last one
   is held, are written up under **Pending corpus**.* ***And 274 promoted with
   none pending since Wave Q1 (2026-08-25), which took the nineteenth — plus
   the last of the "5 not staged — superheat ≈ 0" set below, re-harvested and
   re-oracled, so that bullet is empty too. Nothing from this harvest is in
   `corpus-pending/` any more.*** Duplication against the existing corpus was far
   smaller than feared: only 352 exact + 2 near duplicates were skipped
   across the whole tree, so these really were documents nobody had
   harvested. The 255 are 204 solving documents (35 of them transient, with
   full `ode_tables` comparison), 34 `SolverException` goldens and 17
   `ParseException` goldens — the domain-separation refusals, the port-count
   and unknown-`model$` component rejections, the `DYNAMIC` header
   rejections, the high-index diagnoses, and fourteen "No equations to
   solve." documents that are a library or a `STATE TABLE`/`PARAMETRIC`
   declaration with nothing to solve. The 30 non-promoted classify as:

   * **19 numeric holds, staged in `corpus-pending/`** (the table below).
     Eighteen are the established `oracle-ph-table` signature — R134a /
     R1234yf / Water two-phase chains on `cmp.h_s`, `k.h_s`, `b1.q`,
     `cd.q_sc`, `ev.q_sh`, `rho_in` — with worst divergences (measured
     per fixture, uncapped, at the corpus default) spanning **1.60e-9**
     (`steady-by-integration-chiller-bridge-…`) to **4.71e-5**
     (`moving-boundary-hx-condenser-…`, on the subcool-zone duty). Three of
     the eighteen are transient and graded scale-anchored. The nineteenth,
     `dynamic-array-states-rod-with-ode45-also`, has **no property call at
     all**: a 4-node conduction rod through `ode45` whose 274 diverging
     table cells peak at 1.40e-7 of the column range — adaptive-step
     interpolant noise, `ida-adaptive-path`'s explicit sibling. **Three sit
     within 10× of the corpus default** (1.60e-9, 2.02e-9, 6.90e-9) and
     would very likely promote on a probe; none was probed here, because
     promotion needs `tolerances-rustprop.json`'s evidence discipline — a
     wheel-vs-rustprop-vs-golden measurement *per entry*.
     *(Probed 2026-08-24, Wave P2 — 18 of the 19 promoted, corpus 1253 →
     1271. Two of this paragraph's claims did not survive the probe and are
     left standing above so the correction is visible: the three within 10× of
     the default did **not** promote on the default — every one is above
     `1e-9` and needed an entry — and the count of `oracle-ph-table` documents
     is 13, not 18. Five of the eighteen have no intercepted `(P,Hmass)` shape
     anywhere in them: `real-fluid-properties-solves-vapor-compression-cycle`
     is `upstream-ps-flash-residual`, and `cooker-faithful`,
     `pressure-cooker-…-undersized-valve` and
     `steady-by-integration-chiller-bridge` are `ida-adaptive-path`. The rod
     needed a new catalogued mechanism, `ode45-adaptive-path`. The nineteenth,
     `ev-tms-api-model-…`, is held — its binding variable is not a property
     gap and is not determined by the model; the pending table says why.)*
     *(**The nineteenth promoted 2026-08-25, Wave Q1**, corpus 1281 → 1282.
     P2's diagnosis was confirmed independently and the fix was a *second* new
     catalogued mechanism, `smooth-clamp-regulariser`: `rada.q_lat` is the
     latent duty of a **dry** coil, whose true value is 0 W and whose golden is
     `MoistAirWallHX`'s own `1e-12` smooth-min regulariser,
     `2.5e-13/(W_sat − in.W)` scaled by `mdot·2.501e6`. The two engines differ
     by exactly **16 ulps** of a Newton-solved `out.W` — 6.247503e-11 W — so it
     went onto the **absolute** channel, in watts, while the document's ordinary
     `oracle-ph-table` compressor chain took a measured `relative` of 3.9e-6.
     The document's second dry coil, `conda.q_lat`, is the same shape and the
     channel **refused** it: it is 1 ulp out in the full-corpus replay and 8 in
     a single-document process, so it passes the relative tolerance in the gate
     and its absolute entry was dead. See *The absolute channel* above for that
     finding, and the pending table's Wave Q1 note for the leaf probes and the
     wet control group.)*
   * **5 not staged — superheat ≈ 0, denominator collapse.** `ev.sh` /
     `s1.sh` / `sen.sh` / `sh_start` are 0 here and 1.2e-6…1.5e-6 in the
     oracle, which reads `rel 1.0` however the tolerance is declared. They
     cannot pass as authored, so they are recorded here rather than parked
     in a staging area that means "close":
     `moving-boundary-hx-evaporator-superheat-zone-collapses-smoothly`,
     `moving-boundary-hx-undersized-evaporator-leaves-refrigerant-two-phase`,
     `moving-boundary-hx-transient-warmup-births-superheat-zone-on-ida`,
     `two-phase-distributed-temperature-glides-along-a-multi-cell-coil`,
     `two-phase-domain-two-phase-chain-computes-quality-void-and-pressure-glide`.

     ***Four of the five promoted at Wave P1 (2026-08-24)***, once
     `tests/parity.rs` had the **absolute channel** — see *The absolute
     channel* under **Comparison policy**. They were re-harvested with
     `tools/harvest-java-tests/harvest.py --out …` — it emits only what nothing
     has staged, which was these eleven on the day, and it rewrote
     `harvest-manifest.json` byte-identically — oracled with
     `tools/golden-dumper/run.sh` into a scratch directory, and
     leaf-probed against the CoolProp 8.0.0 wheel at the Java's own inputs.
     In every one the wheel's `Temperature(P,h)` and its own `T_sat(P)` are the
     same `f64`, difference exactly `0.0`, and the golden's own `tsat` is that
     bit pattern — outlet qualities 0.2023, 0.2046, 0.3028, 0.4701, 0.6390, so
     none is a dome-edge artifact. Each took an absolute entry plus a *very*
     tight relative one, because outside the zero these documents are almost
     bit-exact: `…-collapses-smoothly` and `…-undersized-evaporator` (`ev.sh`
     absolute 2.1e-6 K, measured 1.378899e-6; relative **7.4e-9**, measured
     4.956894e-9 on `ev.t_out` — the other 38 of their 40 variables agree with the
     oracle inside the harness's own `1e-12` absolute floor), `two-phase-distributed-…-multi-cell-coil` (`s1.sh` 2.3e-6 K
     and `s2.sh` 1.8e-6 K — **two** exact zeros, with the Java's tabulated
     error carrying *opposite signs* at the two neighbouring pressures;
     relative 8.0e-9, measured 5.306466e-9) and
     `two-phase-domain-…-pressure-glide` (`sen.sh` 1.9e-6 K, measured
     1.263309e-6; relative 6.6e-9, measured 4.380011e-9). Corpus 1257 → 1261.

     **The fifth is still not staged**, and the absolute channel alone does not
     reach it. `moving-boundary-hx-transient-warmup-births-superheat-zone-on-ida`
     puts its exact zero in a *variable* the channel would cover — `sh_start =
     MinValue('ev.sh')`, 0 here against 1.378899e-6 in the oracle — but its
     `ida` trajectory also diverges on `ev$sh` at **1.62e-5 scale-anchored**
     (row 11, ours 9.93351603859169 against 9.93330957310827), with `sh_final`
     at 8.12e-6 and `lsh_final` at 6.08e-8. That is two decades larger than the
     1.38e-6 K table error the algebraic siblings show, so it is not the same
     mechanism read through a trajectory; separating `ida-adaptive-path` from
     `oracle-ph-table` there needs its own measurement, which P1 did not do.

     ***The fifth promoted at Wave Q1 (2026-08-25), and the measurement says
     BOTH.*** It was re-harvested with `tools/harvest-java-tests/harvest.py
     --out …` (which again rewrote `harvest-manifest.json` byte-identically),
     re-oracled with `tools/golden-dumper/run.sh` into a scratch directory, and
     the `ev$sh` gap was **decomposed at all 40 rows** by asking the CoolProp
     8.0.0 wheel for `Temperature(R134a, 350 kPa, h)` at *both* sides' own
     recorded `ev$out$h` — property library held fixed, only the state moving —
     which splits it into a **table term** (wheel minus the golden's own
     tabulated `ev$t_out`, at the golden's own `h`) and a **state term** (wheel
     at our `h` minus wheel at the golden's `h`). The table term is systematic
     and one-signed, **+1.3410e-4 … +1.5755e-4 K** on every superheated row —
     the same `(P,Hmass)→T` error at the same 350 kPa that the algebraic sibling
     `moving-boundary-hx-evaporator-resolves-two-phase-and-superheat-zones`
     already grades at 1.54e-4 K. The state term **alternates sign row to row**
     and reaches **2.4494e-4 K** (row 20), larger than the table term at its own
     worst; over the settled tail (rows 30–39) it holds a steady −4.9e-5 K,
     which is the table error fed back through `Q_sh = U_sh·π·D·L_sh·(T_wall −
     0.5·(Tsat + T_out))·r_sh` at −7.5 W/K. A one-signed RHS bias cannot make an
     **integrated** state swing sign row to row; two accepted-step meshes can,
     and `wall$port$t` — the document's only differential state — does exactly
     that (2.3883e-7 relative at row 10, alternating). At the binding cell
     (`ev$sh`, row 11) the split is **1.386477e-4 K table (67.2 %)** and
     **6.869976e-5 K state (33.3 %)**; this engine's own `ev$t_out` is within
     2.916e-6 K of the wheel at its own `h` at every row, so no property call on
     this side is in question. Promoted with both entries: `relative` **2.4e-5**
     (measured 1.620292e-5, mechanism `ida-adaptive-path`, because the state term
     is what makes it need more than its algebraic sibling) and `sh_start` on the
     absolute channel at **2.1e-6 K** (measured 1.378899e-6, mechanism
     `oracle-ph-table`) — the *same* declaration the two algebraic siblings carry,
     because it is the same `f64`, and the one place in this transient where the
     state term is exactly `0.000e+00` so the table error is read undiluted.
     **The framing above does not survive the measurement**, and is left standing
     so the correction is visible: 1.38e-6 K is the table error at the
     **saturated-vapour boundary** (the collapsed-superheat siblings, and rows
     0–2 of this very trajectory), not at a superheated state — the superheated
     table error at this pressure is 1.5e-4 K and the corpus had already measured
     it, so 1.62e-5 was never "two decades over" anything. Corpus 1282 → 1283,
     and the "5 not staged — superheat ≈ 0" bullet is now empty.
   * **5 not staged — engine divergences, not tolerances.** Each is a real
     behavioural difference and a fixture would pin the difference, not the
     behaviour. Two are the Water/zone-HX guess-landscape family the Wave-I
     oracle verdict above already bounds:
     `component-multi-zone-hx-two-zone-counterflow-hx-transfers-heat-energy-balanced`
     (the **Java** stalls, this port solves) and
     `component-two-phase-three-zone-counterflow-hx-is-energy-balanced` (the
     Java solves, this port stalls in block 37 of 66 equations). Two are
     transient: `two-phase-distributed-transient-crc-relaxes-on-ida-and-migrates-charge`
     (this port's DAE integrator cannot take the first step at `t = 0`) and
     `zeotropic-blend-zeotropic-blend-shows-temperature-glide` (a NaN
     residual out of the blend property chain). The fifth is the sharpest
     and is new: **`solver-equilibration-coupled-block-with-twelve-orders-of-magnitude-scale-disparity-solves`**
     — `big = 2e6 - 1e12*small` / `small = 1e-6*sqrt(big/1e6)`, two
     equations, which the Java solves and this port refuses with "the
     Jacobian is singular" after one iteration. The Java's Newton
     equilibrates the block (its test class is literally
     `SolverEquilibrationTest`); this port does not. Ledger item 38.
     ***Closed and promoted 2026-08-24 (P3), so it is four not five.*** The
     class name misled the triage: this port already equilibrates by the
     Java's exact rule, and Commons Math's LU calls the equilibrated matrix
     singular too — what the Java has is the **SVD pseudo-inverse** its
     `solveLinear` falls back to in the `catch (SingularMatrixException)`
     arm. Transcribed; the document now solves at `big = 1000000.0`,
     `small = 1e-6` with both residuals exactly 0.0, and is staged in
     `corpus/` + `golden/` at the corpus default `1e-9` with no tolerance
     entry. The same change closed a second, unrecorded divergence and
     promoted `solver_merge_rung_rank_deficient_pair` with it (the retry
     ladder's merge rung had nothing to slide a rank-deficient pair with).
     Ledger item 40 has both sets of measurements.
   * **1 not staged — no golden exists.**
     `steady-by-integration-floating-cycle-by-control-volume-integration`
     does not terminate in the **oracle**: IDA reports `mxstep steps taken
     before reaching tout` at `t = 3.1062` and the Java retry ladder spins
     on it indefinitely (killed at 500 s; the other 284 candidates goldened
     in ~26 min total). Nothing to compare against.

   Four more never reached the 285, and are worth naming because a re-run
   must keep refusing them. Three are the `MAX_DECLARED_POINTS` casualties
   above (`ev-tms-transient-compressor-ramp-transient`,
   `scheduled-input-component-takes-scheduled-input`,
   `ev-tms-component-transient-full-network-transient-both-pressures-float`)
   — all three were goldened and the first two *passed*, so this is coverage
   deliberately declined on fixture-size grounds, not a divergence; a
   re-authored copy at a sane `points` would be welcome and would need its
   own oracle run. The fourth is
   `multiout-user-function-with-tilde-discard-2`, a fourth ledger-35
   `~ignored~N` witness the wider sweep surfaced, now in `DROPPED` with the
   three Wave-I ones.

   The replay cost is real: **353.3 s** for 1 238 fixtures in release (on a
   machine carrying a load average of 9–20 from a parallel agent, so read it
   as an upper bound), against ~145 s for 983. What is left un-harvested
   from the Java tests is now only
   what the guards say it is — ~~37 site-tag drops~~, 36 unresolvable arguments
   (12 CoolProp-computed template values, 10 unknown idents, 14 builder
   loops), 3 oversampled and the four `DROPPED` tilde documents.

   **The site-tag drops are mostly gone — 2026-08-25, Wave Q. Corpus
   1281 → 1306.** They were never a *document* problem: every one of those 47
   sites resolved its text perfectly, and was dropped because the fixture
   format could not carry what the test passed to
   `solve(source, settings, specs, defs)` — so the golden would have been the
   engine defaults' answer, which is not what the Java test asserts. The
   `.request.json` sidecar (top of this file) closes that, and it is **one**
   mechanism for all three classes, because all three are the same thing: the
   request carried something other than the defaults. `harvest.py` grew
   `_settings_of`/`_specs_of` (evaluating `new SolverSettings(...)` and
   `Map.of(k, new VariableSpec(...))`, through locals, with a whitelist of
   Java compile-time constants so `Double.NEGATIVE_INFINITY` resolves), and
   a site whose settings *and* specs both evaluate is now tagged `a-request`
   and staged instead of dropped. Half a request is never used: a site whose
   specs do not resolve keeps its skip tag even when its settings did, because
   grading one half against a golden built from both would be the exact
   failure `SKIP_SITE_TAGS` exists to prevent.

   The measured classification, from `harvest.py --inventory` before and
   after:

   | class | before | carried | residue | why the residue stays |
   |---|---|---|---|---|
   | a-complex | 9 | **8** | 0 | the 9th (`EquationSystemSolverTest:375`) is dropped by its specs, not its settings — see a-specs |
   | a-specs | 28 | **17** | 11 | 5 are not `EquationSystemSolver.solve` at all; 6 build the map imperatively |
   | a-settings | 10 | **2** | 8 | 6 are not that `solve` either; 2 have an unresolvable first argument anyway |

   **27 sites carried, 26 candidates staged** (two sites are the same document
   under the same request — `AllRootsSolverTest`'s and
   `EquationSystemSolverTest`'s `x^2 = 4` bounded to the positive half-plane —
   and the duplicate key catches it), **25 promoted at the corpus default
   `1e-9` with no tolerance entry**, 1 staged pending (below). Nothing needed
   a tolerance: 22 solve and 3 are error goldens (`tan` in complex mode,
   `Integral` in complex mode, and the 1-iteration budget the Java refuses
   with "did not converge within 1 iterations"). Ten carry `stopCriteria` and
   fifteen `variableInfo`; **none carries both**, because no Java test passes
   non-default settings *and* a resolvable spec map — the one that does
   (`solvesPowerFactorCorrectionComplex`'s second solve) is the `allOnes`
   builder loop in the residue. Eight are complex-mode documents — the first
   complex fixtures the corpus has ever graded against the oracle, and they
   matter because `SolverSettings::complex_mode` had no golden-corpus coverage
   at all before this. Two carry a request *and* a Function Table (the
   `curvefn` inverse-interpolation pair). The pairs are
   the point of the whole exercise, and the reason a `-2` suffix is on so many
   of them: **sixteen of the 25 are byte-identical in `source` to a fixture
   the corpus already had, and thirteen of those sixteen have a different
   golden** — the request is the only thing that differs, and it changes the
   answer. `eqsys-guess-value-selects-root`/`-2`/`-3` is one equation graded
   at both of its roots (guess 0.5 → 1.0, guess 2.5 → 2.0);
   `eqsys-bounds-select-root` solves to +2 and `-2` to −2 on a bound;
   `eqsys-complex-literals` is the Java refusing with *"enable Complex mode to
   solve them"* and `-2` is the same text solving to `z_r = 3, z_i = 4`;
   `eqsys-complex-solving` is Newton stalling on `z^2 = -4` and `-2` is
   `z_i = 2`; `eqsys-respects-iteration-limit-from-stop-criteria` reaches
   `x = √2` where `-2` fails with *"did not converge within 1 iterations"*;
   and the arrow points both ways — `eqsys-complex-unsupported-function-is-rejected`
   and `integral-rejects-integral-in-complex-mode` **solve** at the defaults
   and are refusals only in complex mode. A fixture format that could not
   carry the request would have had to pick one of each pair and call it the
   document's behaviour.

   The other three twins are the honest weak end, and they are weak for a
   reason worth recording. `curvefn-adjusts-outof-range-guesses-to-range-average-2`
   differs from its twin only in the *path* Newton takes (its Java test
   asserts that an out-of-range guess is pulled to the curve's range average —
   without the pull it would stall, so the fixture does grade the mechanism,
   just not through a different number). `eqsys-propagates-uncertainty-simple-2`
   and `-multiple-inputs-2` carry a `uncertainty` on their specs, and **the
   golden has no field for the propagated σ**: `Result.uncertainties()` is not
   one of the five things the dumper records, so those two grade the solve
   under the spec and nothing else. The third uncertainty document,
   `eqsys-evaluates-uncertainty-of-accessor-2`, does not have the problem
   because `UncertaintyOf(y)` puts the answer *in* `variables` — 0.15 against
   its twin's 0.0. Extending the golden to `uncertainties` would close the
   gap and would touch every fixture-format consumer; it was out of Wave Q's
   scope and is the obvious next step for this channel.

   **The residue is honest, and two thirds of it is a false positive of the
   harvester's own regex.** `CALL_RE` matches any `solve(`, and 11 of the 19
   remaining sites are a different `solve` entirely — `CasIdentity.solve(lhs,
   rhs, var)` (6) and the sparse `SparseSteadyKlu` `solve(double[])` (2), plus
   3 whose document argument does not resolve either. The inventory now says
   so out loud: they carry an `a-settings-alien` / `a-specs-alien` sub-tag,
   counted separately in the summary line, so a future reader does not go
   looking for a fixture-format gap that is not there. **The eight genuine
   ones all build their spec map imperatively** — `new HashMap<>()` then a
   `for` loop over parsed equation names
   (`ClosedLoopDiagnosisTest`, `ComponentMultiZoneHxTest`,
   `EquationSystemSolverTest.solvesPowerFactorCorrectionComplex`'s `allOnes`)
   or two `.put(...)` statements (`PropertyArgumentSeedingTest`). Carrying
   those needs statement-level evaluation of a Java method body, not
   expression evaluation, and it would buy at most six fixtures — three of
   which are the guess-landscape documents the Wave-I oracle verdict already
   records as engine divergences. It is deliberately not built.

   One site-tag drop remains that is *not* about specs at all:
   `RealFluidPropertiesTest:148` is a `check(source, java.util.Map.of())`
   whose document argument is unresolvable, and the dumper's oracle call is
   `solve`, so a `check` site's second argument is a classification only and
   never becomes a request.
4. `../frEES/backend/core/src/main/resources/components/*.frees` — all 295
   library components. **Swept 2026-08-23 (Wave G4).** A coverage inventory
   found 165 of the 295 component types already exercised by the corpus and
   130 not; four parallel authoring agents (one per domain slice: twophase+ac,
   fluid+liquid, signal+electrical+mechanical, moistair+hydraulic+pneumatic+
   heat+powertrain) wrote one minimal physically-meaningful document per
   uncovered type — 130/130 authored and solving on this engine, most with
   hand-computed anchors in comments. The Java oracle solved 129;
   **129 promoted** (`components_g4_*`): 118 at the corpus default with no
   tolerance entry, **11 at declared `oracle-ph-table` entries** (the R134a
   `(P,Hmass)` table again — worst 4.6e-5 on a superheat subtraction,
   `components_g4_twophaseenthalpysource`, the same SEN amplification the
   `components_family_twophase` entry probed; `tolerances-rustprop.json` has
   the per-fixture evidence). One authoring re-tune during promotion:
   `components_g4_aircoil`'s refrigerant flow was lowered so its superheat
   sits at 15.85 K instead of ~0, where the table error read rel 1.0 by
   denominator collapse. One drop: `CabinZone` — the ORACLE stalls at its
   default guesses (its humidity-ratio unknown seeds into `HAPropsSI`'s NaN
   region; this port solves the document) and no in-document guess syntax
   exists to steer it, so the component stays uncovered rather than pinning
   an oracle guess-landscape artifact as a golden. **Component coverage is
   now 294 of 295** *(closed 2026-08-24, Wave I — see the CabinZone note
   below; coverage is 295 of 295)*. Corpus 776 → 905. The agents' authoring
   insights (all-or-none port binding, one-connect-per-electrical-node, the
   der→0 steady branch carrying most stateful components, `time` pinning for
   steady signal rigs) are recorded in the G4 commit message — the batch
   `NOTES.md` files lived in a session scratchpad and are not preserved.

   **Oracle verdict on the Water zone-HX stall (2026-08-24, Wave I).** G4's
   one engine lead — TwoZoneHX/ThreeZoneHX/HeatExchanger "deterministically
   stall on Water: unsolved intermediate enthalpies start at the ~1 J/kg
   default guess, one joule above Water's triple-point property cliff" — was
   never put to the oracle. It has been now: water-to-water TwoZoneHX and
   ThreeZoneHX documents at the `components_g4_heatexchanger` conditions
   (360 K / 200 kPa hot, 300 K / 150 kPa cold, UA = 2500 W/K) were authored,
   confirmed to stall this engine (Newton non-convergence after 27.9 s /
   stall after 52.3 s of retry ladder), and run through golden-dumper —
   **the Java fails identically.** TwoZoneHX: `SolverException: Block 26 did
   not converge within 250 iterations (residual norm 13854.5)` against this
   engine's block 27 at norm 14078; ThreeZoneHX: `Newton iteration stalled
   in block 40` against this engine's stall in block 41. Both Java errors
   bottom out on the same cliff this port hits — CoolProp's
   `HSU_P_flash_singlephase_Brent … Hmolar is below the minimum value of
   2.46802437917 J/mol` on a `("P", "Hmass", "Water")` call, with the very
   `Tmin=273.144, Tmax=393.36` bracket rustprop's refusal quotes. **The
   stall is a faithful port of the Java guess landscape, not a seeding
   bug** — no engine change, and the Water documents stay unpromoted (an
   error-message golden would pin block numbering and residual noise, not
   behavior). A future *joint* seeding improvement remains possible but
   belongs upstream-of-parity; the G4 "a future seeding improvement could
   close this" note now carries this boundary.

   **CabinZone is covered (2026-08-24, Wave I): coverage 294 → 295, corpus
   905 → 906.** The G4 drop was the *steady* document — the oracle stalls
   when the der→0 branch makes `Wz` an algebraic unknown whose default guess
   seeds `HAPropsSI` into its NaN region. The transient hypothesis held: in
   a `DYNAMIC` document `Tz`/`Wz` are integrator states seeded from
   `init(T0/W0)`, so neither engine ever visits the guess landscape.
   `components_i_cabinzone_dynamic` (a 120 s ventilation pulldown, ode23s,
   21 points, wall coupled through `Convection` to a `ThermalSource`
   ambient) solves in both engines and the pair is **bit-identical** —
   every value of the 21×35 `ode_tables` grid, both `FinalValue` anchors
   (`t_end` 292.33696797339354 K, `w_end` 0.007298792069508541),
   `end_time` and `stopped` — promoted at the corpus default with no
   tolerance entry. `component_families.rs`'s measured group floor
   re-raised 262 → 263 per its never-lower rule.
5. `../frEES/backend/core/src/test/resources/validation/*.frees` — the
   reference's **published verification suite** (`ValidationSuiteTest`): 34
   closed-form physics/numerics problems whose `// EXPECT` directives are
   ordinary frees comments, so each file is a complete document as-is.
   **Harvested 2026-08-24 (Wave I)**, discovered during the Wave-I inventory:
   all 34 staged verbatim (`validation-<stem>`), the oracle solved all 34,
   **34 promoted at the corpus default with no tolerance entry** — including
   the four transient ones (`validation-ode-*`), whose ODE tables compare
   row-for-row. The one growth source that was free: the documents already
   existed and every one cleared

Add the document to `corpus/`, rerun `run.sh`, and commit both the source and
the generated golden file. **Review the generated fixture before committing** —
it encodes whatever the Java engine does, including any bug.

### The `components_*` group (Phase 6)

176 fixtures whose stem starts with `components_` (46 until Wave G4,
2026-08-23), in five provenance classes:

| Prefix | Count | Where it came from |
|---|---|---|
| `components_g4_<type>` | 129 | **The Wave G4 library sweep** (growth-source item 4 above): one minimal document per previously-uncovered built-in, authored by four parallel agents, oracle-solved and promoted under the standard rule — 118 at the corpus default, 11 with `oracle-ph-table` entries. Raised `component_families.rs`'s measured group-coverage floor 120 → 262 of 295. |
| `components_i_cabinzone_dynamic` | 1 | **Wave I (2026-08-24)** closing G4's one drop: the `DYNAMIC` CabinZone pulldown that sidesteps the oracle's steady-guess stall (item 4 above has the full story). Bit-identical to the golden; floor 262 → 263. |
| `components_family_<domain>` | 12 | **Hand-authored here**, one per domain family (`ac`, `electrical`, `fluid`, `heat`, `hydraulic`, `liquid`, `mechanical`, `moistair`, `pneumatic`, `powertrain`, `signal`, `twophase`). `control.frees` ships one component, `PIThermostat`, which rides the heat bond and is exercised from `components_family_heat`. |
| `components_<wave>_<name>` | 31 | **Harvested** from `../frEES/backend/core/src/test/resources/component-fixtures/`, renamed `<directory>_<stem>`. Only documents that reproduce the Java answer are here; the reference set's transient (`DYNAMIC`) documents are not, because the ODE engine is Phase 7/8. |
| `components_definition_*`, `components_user_defined_type` | 3 | Edge cases: a template that is never instantiated (empty document, and beside an unrelated scalar), and a user-declared `COMPONENT` shadowing nothing. |

Two things about this group differ from the rest of the corpus and are worth
knowing before extending it:

* **The golden's variable keys are display names.**
  `EquationSystemSolver.buildResult` keys `Result.variables()` with
  `displayNames.getOrDefault(name, name)`. For a scalar document that is the
  canonical name once lowercased, which is why folding the golden was enough for
  the pre-component corpus. For a component document it is not: the canonical
  name is `s2$p` and the display name is `s2.P`. `tests/parity.rs` therefore
  routes the *Rust* side through the same map before comparing — a no-op on
  every non-component fixture.
* **`crates/frees-core/tests/component_families.rs` measures coverage** over
  exactly this group, and pins two floors: how many of the 295 built-ins the
  corpus instantiates, and how many expand from a bare probe instantiation.
  Adding a document here can only raise those numbers.

Of the 92 documents in the reference component-fixture set, 51 are steady (the
rest need `DYNAMIC`); of those 51, **31 reproduce the Java answer** and are
promoted. The other 20 were held on a property-backend limit, not the component
layer — `Air`/`CO2`/`Hydrogen`/`INCOMP::MEG` not tabulated, `HAPropsSI` not
implemented, `Cpmass`/`viscosity` not stored by the split `(P,h)` table — and
every one of them named the missing capability in its error.

> **That triage predates the backend and has not been redone.** It was measured
> against D1's `(P,h)` tables. Since [D9](../docs/decisions/0009-rustprop-backend.md)
> the engine answers `HAPropsSI`, `Air`, `INCOMP::MEG`/`MPG`, `Cpmass` and
> transport directly, so several of those twenty limits no longer exist — the
> analogous held documents in `corpus-pending/` all cleared (Wave-3 F6/F8,
> twelve for twelve). `CO2` and `Hydrogen` are the exception and are a
> different kind of question: rustprop has both, but this build enabled four
> per-fluid Cargo features (`water`, `r134a`, `r1234yf`, `air`), so adding one
> is a bundle decision, not a porting one — ~26 KB of wasm per fluid by
> rustprop's own all-130-fluids measurement, unverified from here. *(Verified
> 2026-08-23, Wave G2: linking `carbondioxide` cost exactly +25.5 KiB raw —
> rustprop's per-fluid estimate holds on this build's toolchain. Five features
> now; `Hydrogen` remains unlinked and unasked-for.)*
> These twenty documents are not in `corpus-pending/`, so re-checking them
> means harvesting them first; nobody has, so the number that would come out is
> unknown.

### The `av_*` / `pv_*` group (Phase 8 adversarial verification)

110 documents written to *hunt* divergences in the dynamics and analysis paths
rather than to record behaviour already believed correct. Each was authored,
run through **both** engines, and diffed on `variables`, `display_names`,
`block_count`, `error` classification and — for every transient — the whole
`ode_tables` section. Only documents that reproduced the oracle were promoted.

| Prefix | Count | What it pins |
|---|---|---|
| `av_int_` | 15 | Integrator agreement: Van der Pol through `ode45`/`ode23`/`ode23s` at 201 samples, a stiff scalar through all of `ode15s`/`ode23s`/`ode45`, a coupled stiff pair through the BDF path, fixed-step `ode1`/`ode4`, the dense-output interpolant sampled between steps, `maxstep` clamping, `rtol = 1e-12`, step rejection against a narrow Gaussian pulse, and the degenerate/reversed spans |
| `av_ev_` | 17 | Events: `rising` / `falling` / bare / explicit `any`; a guard exactly zero at `t0` (both `record` and `stop`); two guards crossing inside one step and the same two far apart — together those bracket `earliestEvent`'s **one hit per step** rule; two `stop` guards racing; a tangential touch that never changes sign; `set` restarting integration 27 times (thermostat) and 11 times (elastic bounce); `set` then `stop`; guards on an auxiliary and on `time`; a crossing exactly at `tf` |
| `av_acc_` | 14 | The twenty accessors read off a solved table: all ten on one column, on `time` itself, `ODEValue`'s clamp outside the span, `TimeAt` on a row / never reached / first of two crossings, an auxiliary column, two blocks with distinct and with identical column names, a spurious second argument, a missing required one, the null-context defaults, and the `augmentAccessorDependencies` rule that an unknown living only inside the block never becomes an analytic variable |
| `av_live_` | 5 | The live second-solve pass: `FinalValue` / `MaxValue` / `TimeAt` / `ODEValue` targets solved for an ODE input, and a 2×2 block whose every residual costs an integration |
| `av_stor_` | 5 | Storage → `DYNAMIC`: one and two `ThermalMass` lumps integrating, an electrical `Capacitor` (the routing is domain-agnostic), an event on a mangled port member, and the **steady limit** — the same network with no `DYNAMIC` block recovers the operating point the transient converges to (446.667 / 406.667 K) |
| `av_mol_` | 1 | Method of lines: a `FOR` loop in the block body over an array state through `ode23s` |
| `av_lim_` | 15 | Header and failure edges: `points` = 0 / 1 / 2 / negative, `step` past the span, `rtol = 0`, `atol = 0`, an unknown method, an unknown option, finite-time blow-up, a NaN rhs, the `span/100` default `maxstep`, a stop event's row grid, a unit on the span, an expression initial condition |
| `av_unc_` | 22 | `UncertaintyOf(X) = expr`: the square law, root-sum-square of two sources, a three-deep chain, one source feeding two dependents, propagation through an implicit 2×2 block, through the component layer, and through an **ODE accessor**; an expression-valued spec; zero / negative / 1e-12 sigmas; a query that reads its own dependent; a string-literal target; redeclaration; and the two shapes that are *not* declarations (`expr = UncertaintyOf(X)`, and a two-argument call) |
| `av_lin_` | 6 | `LINEARIZE`: default matrix names, 2 in / 2 out / 2 states, the 1- and 2-subscript spellings of a single-column `B`, the identity `C` row, an unknown block, an unknown output |
| `pv_` | 10 | `PARAMETRIC` declarations: a basic sweep, ragged columns, `Log` endpoints, and the seven range rejections |

Two things this group establishes that no earlier fixture did:

* **The uncertainty pass reaches the engine.** Before this phase
  `crates/frees-core/src/analysis/uncertainty.rs` was complete and unit-tested
  but had **zero call sites** — `UncertaintyOf(X) = 0.1` stayed in the equation
  list and every uncertainty document failed as overspecified. The `av_unc_`
  documents are the regression wall for the wiring.
* **A `DYNAMIC` document's cost is `maxstep`-driven, not span-driven.** With no
  `maxstep` the cap is `span/100`, which pins ~100 steps whatever the span is,
  and each implicit step's Newton residual re-blocks the algebraic system. Van
  der Pol through `ode15s` at 0..20 / 201 points costs **28.6 s here and 32 s in
  the Java oracle** — comparable, so it is a shared algorithmic cost and not a
  divergence, but it is why the promoted BDF probes are small and why
  `av_int_stiff_pair_ode15s` uses a linear 2-state system.

### The `ctl-*` group (Phase 9 adversarial verification)

19 documents written to hunt divergences in the newly-wired control-systems
`CALL` surface. Between them they exercise **all 41 names** in
`control::flatten::CALL_NAMES` (40 + `mason`). Each was authored, run through
both engines and diffed on every variable; only exact agreement was promoted.

| Fixture | What it pins |
|---|---|
| `ctl-lqr_dblint`, `ctl-lqr_3state`, `ctl-lqr_mimo` | `lqr` for 2/3 states and 2 inputs, plus the closed-loop `A − BK` spectrum via `pole` |
| `ctl-lqr-stabilisable_*`, `ctl-lqr-marginal_uncontrollable` | LQR needs `(A,B)` **stabilisable**, not controllable: three rank-deficient controllability matrices whose unreachable mode decays, all of which must still solve. The complementary *unstabilisable* case is a recorded divergence, not a fixture — see below |
| `ctl-place_acker` | `place` and `acker` on one plant, and the poles they realise |
| `ctl-lyap_dare` | `lyap`, `dlyap`, `dare`, `dlqr` |
| `ctl-lqe_pidtune` | `lqe`; `pidtune` in both `PID` and `PI` form |
| `ctl-interconnect` | `series` / `parallel` / `feedback` over one pair |
| `ctl-tf_roundtrip_proper` | `tf2ss` → `ss2tf` on a **biproper** plant (non-zero `D`) |
| `ctl-ss2ss_ij` | `ss2tfij` at two output/input pairs and `ss2ss` under a similarity transform (both in `CALL` form — the sized-output assignment spelling is rejected by *both* engines) |
| `ctl-zp_roundtrip` | `tf2zp` → `zp2tf` |
| `ctl-ctrb_obsv_rank` | `ctrb`, `obsv`, `rank`, and `gram` in both `'c'` and `'o'` form |
| `ctl-mason_graph` | `mason` over a 3-node signal-flow graph |
| `ctl-c2d_d2c` | `c2d` → `d2c` |
| `ctl-pade_delay` | `pade` at orders 2 and 3 |
| `ctl-response_analysis` | `step`, `impulse`, `lsim`, `stepinfo`, `bode`, `nyquist`, `nichols`, `margin` — 101 variables |
| `ctl-residue_routh_rlocus` | `residue`, `routh`, `rlocus`, `errorconst` — 524 variables, the largest in the corpus |

**Three control results were deliberately *not* frozen**, because their outputs
are not stable enough to gate on — record them here instead of pretending they
are regressions:

* **Repeated roots.** `pole` on a polynomial with a root of multiplicity `m`
  produces spurious imaginary parts of order `eps^(1/m)`, and the two engines
  land on *different* noise. Measured: `(s+1)(s+2)²` → `±4.06e-8` (Java) vs
  `±4.32e-8` (Rust); `(s−1)³` → `±6.45e-6` vs `±6.49e-6`; `(s−1)⁴` → real parts
  `0.99984819` vs `0.99984296`. That is textbook ill-conditioning at the
  expected magnitude, present equally in both, and a fixture on it would be a
  flake generator.
* **`balreal` sign convention.** For `A = [0 1; −2 −3]`, `B = [0; 1]`,
  `C = [1 0]` the two balanced realisations differ by exactly `T = diag(1, −1)`:
  Java `Ab[1,2] = −0.9701425001453321`, Rust `+0.970142500145332`, and likewise
  for `Bb[2,1]` / `Cb[1,2]`. Verified to be a pure state-basis flip —
  `T·Aⱼ·T = Aᵣ`, `T·Bⱼ = Bᵣ`, `Cⱼ·T = Cᵣ` — with identical eigenvalues
  `{−1, −2}` and identical transfer function `1/(s²+3s+2)`. Both are valid; a
  balanced realisation is only unique up to that sign. *(Closed 2026-08-21:
  since `linalg::svd` became the Commons Math transcription, the port lands on
  the Java's basis exactly — this note stays as the record of why the two
  could legitimately differ before.)*
* **LQR on an unstabilisable plant.** `A = [2 1; 0 3]`, `B = [1; 1]` has
  `rank[B, AB] = 1` and the *unreachable* mode is `λ = +2`, so no stabilising
  gain exists. The Java returns one anyway — and it does not stabilise. Measured
  closed-loop spectra for the Java's `K`: `Q = I` → `{1.928, +2}`;
  `Q = 10·I` → `{−9.099, +2}`; `Q = 20·I` → `{−14.54, +2}`; and for `Q = 11·I`
  and `Q = 100·I` the Java instead errors "matrix is singular", i.e. its answer
  is decided by which side of Commons Math's `1e-11` LU pivot threshold the
  rounding falls on. This port refuses, and now names the mode
  (`control::design::unstabilisable_mode`). Pinned as a unit test —
  `design::tests::lqr_solves_stabilisable_plants_and_names_the_mode_when_it_cannot`
  — rather than as a corpus fixture, because the corpus compares *against* the
  Java and here the Java is wrong.

## Pending corpus

`corpus-pending/` is a staging area with the same shape as the promoted corpus
(`corpus-pending/corpus/*.frees` + `corpus-pending/golden/*.json`). Every
document there has a golden generated by the same oracle; none of them is
replayed by `crates/frees-core/tests/parity.rs`, so nothing here can turn the
gate red.

**The promotion rule:** run the document through the Rust engine
(`cargo run -qp frees-cli --features rustprop-backend -- solve <file>`) and
compare against its golden using the tolerance table above — `variables` by
relative tolerance, `display_names` and `block_count` exactly, `error` by
classification. (A document with a `.tables.json` **or** a `.request.json`
sidecar cannot be graded by the CLI one-liner — the CLI has neither channel,
and for a request-carrying document it would silently grade the *defaults*
against a golden the oracle produced under other settings, which is worse than
no answer; use the scratch `parity.rs` procedure below, which replays both
fields.) If it agrees, move **every** file of the document — `.frees`, its
sidecars, and the golden — into `corpus/` and `golden/`. If it diverges, leave
it here. A pending document that starts passing because someone fixed the
engine is the point.

### What is pending today — 2 documents

| Blocker | Count | Documents |
|---|---:|---|
| **The shooting residual is a staircase, and the equation it asks to satisfy has no solution** — measured 2026-08-25, Wave R1, which is also where Wave Q's `ode45-adaptive-path`-amplified-by-a-shooting-solve reading was measured and **refuted**. The document is a rocket ascent through `ode45` whose burn time is *sized* by the algebraic system so apogee reaches 100 km (`VariableSpec("t_burn", 30, 5, 55)` is the request it carries). Wave Q's numbers reproduce exactly: the trajectory passes at the bare `1e-9` — worst cell **7.374096e-11** scale-anchored (`drag`, row 86 of 500, the first row after thrust cut-off), worst absolute 7.176131e-6 m on `h`, `end_time` 3.543114e-11 (5.59e-9 s), the `apogee` event with it — while `t_burn` 27.14432258487798 against 27.14264495979557 and `m_burn` 244.29890326390182 against 244.28380463816015 both read **6.18039030873655e-5**. Q4's read that this is one divergence seen twice is **confirmed to the bit**: `9.0 * t_burn == m_burn` is bit-equal (0 ulps) on *both* sides and `m_burn/t_burn == 9.0` exactly on both, so the pair is one number, not two. **Its diagnosis is not.** The amplification story requires the observed `t_burn` gap to be a trajectory disagreement divided by `dh/dt_burn`; measured by secant over three spans that number is 9.044e3 (±0.02 s), 1.096e4 (±0.01 s) and 1.244e4 (±0.005 s) m/s, so the 1.6776250824e-3 s gap would demand **15.2 … 20.9 m** of apogee disagreement between the two engines. The measured apogee disagreement is **7.176131e-6 m** (100007.75987525134 golden, 100007.75988242748 here). Run backwards, that 7.18 µm implies a `t_burn` gap of 5.8e-10 … 7.9e-10 s, i.e. rel **2.1e-11 … 2.9e-11**, against the 6.18e-5 observed. Five orders out in both directions: the sensitivity path is not the mechanism. What is: **`t_burn` reaches the ODE right-hand side only through `If(time, t_burn, …)`** — only through the *sign* of `time − t_burn` at the Dormand–Prince stage points — so the whole integration, and therefore the residual `MaxValue('h') − h_target`, is **piecewise constant** in the unknown. Measured: the apogee is bit-identical at 100007.759882427476 m for every `t_burn` in **[27.1419101058221592, 27.1448242496660086]**, a plateau **2.9141438438e-3 s** wide (**1.073574e-4** relative), both edges bisected to the last representable step and 400 uniform interior samples all landing on the same bits. **Both engines' answers are inside that one plateau** — the Java's 25.22 % along it, this build's 82.79 %, their gap 57.6 % of its width — and feeding this engine the Java's own `t_burn` reproduces this engine's own trajectory **bit-for-bit**: 0 of 4 000 cells differ and `end_time` is bit-equal. (The probe reproduces the *fixture's* own solved table bit-for-bit too, 0 of 4 000, so the plateau is the fixture's and not an artifact of the reduced document.) The goldens corroborate it without any engine being run: the golden's own `m` ends at 355.70671449100365, i.e. 244.29328550899635 kg burned, an **effective** burn of 27.143698389888485 s — 1.0534e-3 s *more* than the `t_burn` the same fixture reports — while this build's effective burn is 27.143698390602644 s, 6.2419e-4 s *less* than its own. **The two engines agree on how much propellant actually burned to 2.6310e-11 and disagree on the nominal burn time by 6.18e-5.** And the equation is not merely flat near the answer, it is **unsatisfiable**: over `t_burn ∈ [27.130, 27.160]` at 5e-5 s resolution the apogee takes **61 distinct plateau values**, treads 5e-5 … 2.85e-3 s wide, non-monotone, changing sign about `h_target` **three times without ever attaining it** — the closest attained apogee in the whole window is 100001.007818705 m (+1.008 m) and the tread both engines stop on is **+7.760 m** out, a residual both report success with standing (the block is 1 equation in 1 unknown). **So there is no tolerance entry to write, and this is why** — not a missing measurement. The reported `t_burn` is a free parameter of the iteration, not an output of the model: with the fixture's own document and *only the initial guess* changed it ranges 27.1424015539609940 (guess 10) … 27.1463265028567555 (guess 40), a spread of 3.9249e-3 s = **1.4459e-4 relative**, 2.3× the divergence being graded, and `jacobian_epsilon` 1e-7 → 1e-8 moves it to 27.1376778256066515, 2.0e-4 relative away. (`rel_tolerance` 1e-6 … 1e-12 and `max_iterations` 10 … 200 do not move it at all, and the gate's own configuration — guess 30, defaults — reproduces 27.14432258487798 bit-for-bit in three separate processes, so a `relative` of 9.3e-5 *would* be green today.) It would also be worthless: `tolerances-rustprop.json`'s rule is that `reason` must let a future session tell a golden-side artifact from a regression, and on a flat residual **no** future movement of this number is classifiable as either — a harmless refactor of the Newton path and a real regression look identical, and the only response to either would be to re-measure and widen, which is the ratchet the file exists to prevent. The entry is therefore disqualified by the file's own standard, the same way Wave P2's `conda.q_lat` was. Neither side is nearer the truth because the document as discretised has no truth to be nearer to; this is not a port defect, not an oracle defect, and not a tolerance entry. **Do not catalogue a mechanism for it** — a catalogued mechanism that no entry cites fails the harness, so the name this earns (`stepped-over-switch-plateau`: an RHS switch the adaptive integrator steps over, flattening a shooting residual) lives here in prose and nowhere else. **Promoting it needs the document changed, not the gate**: give the switch an `EVENT` so the integrator lands on it exactly, and the residual becomes smooth and the unknown determined | 1 | `ode-rocket-trajectory-sizes-burn-time-so-apogee-reaches100km` |
| **Cost, not correctness** — solves **bit-identically** since 2026-08-21 (Wave A5: table rows exact, variables at ~1e-15, `block_count` and display names exact). Re-measured 2026-08-23 after Wave G3's per-step caches: **~5.6 min** (339 s, upper bound — the first 147 s shared the machine with a replay), from A5's ~12 min, converging to the same `dk` at rel ~8e-16; the whole replay is ~145 s, so one document still costs ~2.3× the gate and the hold stands. **Re-measured 2026-08-25 after Wave Q3** (the per-call block-equation list and the per-step `Scope` clone, −9.7 % instruction count on both stand-ins): **155.5 s** user on a quiet machine, from the same binary's **180.4 s** baseline, output byte-identical. Against a replay that was then **311 s for 1281 documents** (0.243 s/document average) this one was **640× the average**; promoting it would have taken the gate to ~466 s (**+50 %**) and made one document a third of it. **Re-measured 2026-08-25 after Wave R3** (the prepared block's `work_scope` becomes slot-native and its map stops being materialised per call, −17.9 % instruction count on both stand-ins): **134.9 s** user, the mean of two interleaved pairs (132.16 / 137.71) against the same baseline binary's **163.7 s** (168.23 / 159.24) — −17.6 % wall, matching the instruction count, and all four outputs share one MD5. Scaling Q3's quiet-machine anchor by the measured ratio puts it at **~128 s**. The hold still stands, and the margin is now argued against the gate CI actually runs. Unsharded the replay was **188 / 194 / 242 s for 1308 documents** over three runs on that box (0.144–0.185 s/document; the spread is the machine, not the build — a sibling agent compiles beside it), so this one was **730–940× the average**; promoting it would have taken the unsharded gate to ~325–375 s (**+56–72 %**). **Every shard number this row used to carry was invalidated one commit later, and is re-measured here — 2026-08-26, at HEAD, release, on a quiet box.** Wave R2's `props_si` cache landed *after* R3 and cut `ev-battery-cooling-pid` — the fixture that sets the longest shard — from ~81 s to **42.96 / 43.18 s** user (two runs), so the "~108–121 s longest shard" this row asserted was already wrong when it was written. What is true now: `dyn_accessor_live` standalone is **120.95 / 120.86 s** user (R3's 134.9 s was wall, on a box carrying a sibling build); the whole corpus unsharded is **126.98 s** for 1308 documents (0.0971 s/document), so this one document is **~1250× the average** — worse again than R3's 730–940×, and for the same reason, that the corpus keeps getting faster and this document does not; and the four shards CI actually pays, timed one at a time, are **11.97 / 17.36 / 46.35 / 52.49 s**, where the longest is shard 3 and `ev-battery-cooling-pid` (sorted index 855, marginal cost 45.86 s inside the replay) is **87 %** of it. Promotion still puts `dyn_accessor_live` at sorted index **727 → shard 3** and pushes `ev-battery-cooling-pid` to index 856 → **shard 0**, exactly as R3 computed; what changed is the size. Projected shards **≈ 54 / 22 / 37 / 135 s**: the longest goes **52.5 → ≈135 s, +157 %**, with one fixture at **90 % of its own shard**. *(Projection method: every fixture timed alone through the harness's own `PARITY_SHARD_COUNT=1308` mode, less the measured 0.0337 s per-process floor, then scaled ×0.944 so that today's modelled longest shard reproduces the 52.49 s actually measured.)* And the instability is now pinned rather than asserted: the two heavy fixtures sit 129 sorted indices apart and 129 ≡ 1 (mod 4), so **three** future fixtures landing *between* indices 727 and 856 collide them into one shard of **≈167 s (+218 %)**. A cost that large *and* that sensitive to where the next fixture happens to sort is what keeps the hold | 1 | `dyn_accessor_live` |

*(**The Wave-J class-sweep row left this table on 2026-08-25, Wave Q1** — its last document, `ev-tms-api-model-integrated-model-solves-with-plain-solver`, promoted with **two** kinds of entry. Its property chain is the ordinary `oracle-ph-table` P2 called it: the R1234yf compressor's suction entropy `Entropy(R1234yf, 350 kPa, h = 371807.7308757396)` — a `(P,Hmass)→Smass` call — sits **1.2896e-7** from the CoolProp 8.0.0 wheel in the golden and **5.6713e-13** in this engine, while the `(P,Smass)` leaf downstream is **bit-identical** between golden and wheel at each side's own `s_in`; the leaf walks to **2.6013e-6** on `cmp.W` (×20.2, `W = mdot·(out.h − in.h)` subtracting two enthalpies that differ by a tenth of either), which the declared `relative` **3.9e-6** grades. Its worst variable is not that chain, and P2's diagnosis is confirmed independently. `rada.q_lat` (5.4713e-6) is the latent duty of a **dry** coil, where `MoistAirWallHX`'s saturation clamp never engages — RH 0.1818 at the outlet, `W_sat/in.W` = 5.93 — so `in.W − out.W` is nothing but the smooth-min's `1e-12` regulariser, `2.5e-13/(W_sat − in.W)`, which through `mdot·2.501e6` **is** the 1.1418743867640246e-5 W the golden records. The true duty is 0 W, the two engines differ by exactly **16 ulps** of a Newton-solved `out.W`, and `0.9·2.501e6·|Δout.W|` reproduces the absolute gap **6.247503e-11 W** to the last digit. Its inputs are innocent — `in.W` is `0.01` on both sides exactly, `t_out`/`in.h`/`out.h`/`wall.T` are **bit-identical**, `w_sat` agrees to 22 ulps (2.5751e-15). It went onto the **absolute channel** under a new catalogued mechanism, `smooth-clamp-regulariser` — the exact-zero shape arriving from the *model's* own algebra rather than the oracle's property table, and the channel's first entry in watts. Two facts decide it without a third library: the closed form `2.5e-13/(W_sat − in.W)` is exact to 1.0e-10 here and **this engine is 46× nearer to it** than the golden (1.203e-7 against 5.592e-6); and the same document carries a **wet** coil, `EVCA`, whose clamp does engage and whose 1229.8257 W of real latent duty the two engines agree on to **3.0665e-12**. **P2's `conda.q_lat` variance reproduced, and the confound is the run rather than the build.** All three build configurations agree when run the same way; what moves the number is whether the document is replayed alone or inside the corpus. In the gate — 1283 fixtures in one process — `conda.q_lat` is **one** ulp of `out.W` out (3.904689e-12 W, rel 7.7832e-7, P2's own figure); alone in a fresh process the same binary is **eight** (3.123751e-11 W, rel 6.2265e-6). The swing is what was measured; the identified candidate is `props/rustprop_warm.rs`, which seeds each `(P,h)`/`(P,s)` flash from the previous answer, so a warm-started flash can converge on a different last ulp. So `conda.q_lat` cannot take an absolute entry — it passes the 3.9e-6 relative in the gate, and the channel's dead-entry guard said so out loud when a first draft of this wave tried. Corpus 1281 → 1282, pending 2 → 1.)*

*(**The structurally-exact-zero row — the last four of the Wave-I
property-chain 18 — left this table on 2026-08-24, Wave P1**, by a gate change
rather than an engine change: `tests/parity.rs` grew an **absolute** channel
(see *The absolute channel* under **Comparison policy** above), because the
quantity those four assert is identically zero and a relative measure has no
denominator against a zero. The four promoted with **two** entries each — one
variable on the absolute channel, everything else on a measured `relative`:
`accomp-air-coil-cools-and-dehumidifies` (`coil.ev.sh` absolute 1.8e-6 K,
measured 1.215685e-6; relative 2.0e-7, measured 1.309431e-7 on `coil.ev.q_sh`),
`chgclosed-charge-chain-is-well-posed` (`cnd.sc` absolute 4.2e-7 K, measured
2.797688e-7; relative 5.7e-6, measured 3.767336e-6 on `cnd.rho_in` — still the
largest single R134a `Dmass` leaf error the corpus has measured),
`chgclosed-condensing-pressure-floats-with-ambient-and-charge` (`cnd.sc`
absolute 1.8e-6 K, measured 1.183385e-6; relative 1.4e-6, measured 9.618416e-7)
and `tpcharge-charge-sets-condensing-pressure-and-subcooling` (`cond.sc`
absolute 8.5e-7 K, measured 5.685999e-7; relative 1.7e-6, measured 1.103548e-6).
Every one was leaf-probed against the CoolProp 8.0.0 wheel at the **Java's own**
inputs first, and in all four the wheel's `Temperature(P,h_out)` and its own
`T_sat(P)` are the same `f64` — difference exactly `0.0` — with the golden's own
`Tcond`/`tsat` that bit pattern too, so the golden's `2.8e-7 … 1.2e-6` K **is**
the Java table's error at that state. Outlet qualities 0.0093, 0.0121, 0.0241
and 0.935, so none of them is a dome-edge artifact either. `block_count` and
`display_names` were exact on all four throughout.
**The same channel also reached four documents the Wave-J sweep had left
unstaged** — the "5 not staged — superheat ≈ 0" bullet under *Growing the
corpus*, re-harvested and freshly oracled; that bullet carries their numbers
and says why the fifth, a transient, still needs a measurement the absolute
channel cannot supply. Corpus 1253 → 1261, pending 24 → 20.)*

*(**Fourteen of the Wave-I property-chain row's 18 left this table on
2026-08-24, Wave A1** — by probing, not by any engine change. Each was
re-checked with the scratch-harness procedure below (goldens from
`corpus-pending/golden`, tolerance path aimed at a non-existent file so the
grade is the bare `1e-9`), its worst variable traced to a leaf property call,
and that leaf asked of the **CoolProp 8.0.0 wheel** in rustprop's
`tools/golden-gen/.venv` at the *Java's own inputs*. In all fourteen rustprop
reproduces the wheel to between 0 (bit-identical — the compressors' `Dmass`
and `Smass`) and 1.1e-11 (the subcooled-liquid `Dmass` in `tpcharge-*-2`, the
one leaf near the dome edge) while the golden sits 4.8e-8…1.4e-6 away — four
to five orders at every leaf — so every entry is `oracle-ph-table` and the
port is the nearer of the two to the physics. The interception boundary shows up
inside the fixtures themselves: every `(P,Q)` call in them (`hf`, `hg`,
`T_sat`, the condensers' `Tcond`) and every `(P,Smass)` call (both
compressors' `h_s`, asked at the golden's own tabulated entropy) is
**bit-identical** between golden and wheel — only `(P,Hmass) → T/Dmass/Smass`
diverges. Declared tolerances, measured ×1.5: `accomp-exv-opening-sets-flow`
(+`-2`) and `accomp-txv-relaxes-toward-its-superheat-target` 1.1e-6 (measured
7.1497e-7, one shared R134a 900 kPa/`hf` `Dmass` leaf, undiluted);
`accomp-volumetric-compressor-scales-flow-with-rpm`(+`-2`) 3.1e-6 (2.0907e-6);
`chgclosed-condensing-pressure-floats-with-ambient-and-charge-2` 6.9e-6
(4.6118e-6); `chiller-chiller-cools-the-coolant-loop` 1.9e-5 (1.2487e-5);
`chiller-higher-refrigerant-flow-delivers-more-cooling` 9.5e-6 (6.3190e-6) and
`-2` 8.9e-4 (5.9320e-4 — one 1.5e-4 K tabulated-`T` error over a 0.252 K
superheat, the file's loosest entry and a denominator, not a defect);
`closedloop-measure-dof-6` 1.1e-5 (7.4616e-6); `closedloop-measure-dof-7`
3.3e-6 (2.1731e-6); `tpcharge-charge-sets-condensing-pressure-and-subcooling-2`
5.6e-5 (3.7321e-5); `tpcycle-cop-drops-versus-isobaric-baseline` 4.0e-6
(2.6898e-6); `tpcycle-non-isobaric-cycle-has-suction-below-evaporating-pressure-and-plausible-cop`
1.4e-5 (9.3646e-6). The TXV fixture is the wave's only transient and needs
**no** `ida-adaptive-path` component: only five of its table columns diverge
at all — `v$rho_in` and the four `mdot` copies that read √ρ of it — by the
same amount in all 40 rows, while the integrated states `SH_b`/`CdA` and the
`FinalValue`/`MinValue` scalars taken off them stay under the `1e-9` default.
Corpus 983 → 997, pending 19 → 5.)*

*(**Eighteen of the Wave-J row's nineteen left this table on 2026-08-24, Wave
P2** — by probing, not by any engine change; the nineteenth stayed, and moved
to the row above it. Each was re-checked with the scratch-harness procedure
below (goldens from `corpus-pending/golden`, tolerance path aimed at a file
that does not exist so the grade is the bare `1e-9`), re-measured
**bit-identically** under both `cargo test --release -p frees-core --features
rustprop-backend` and `--workspace`, then traced to its leaf and that leaf
asked of the **CoolProp 8.0.0 wheel** at the *Java's own inputs*. **None of the
three within 10× of the default passed outright** — 1.635745e-9, 2.023216e-9
and 6.936123e-9 are all above `1e-9`, so every one needed an entry; the earlier
"would very likely promote" was a guess and it was wrong. Corpus 1253 →
**1271**, pending 24 → 6.

Thirteen are `oracle-ph-table`, and the interception boundary is visible inside
the fixtures again: every `(P,Q)`, `(P,T)` and `(P,Smass)` call in them —
`hf`, `hg`, `T_sat`, `Enthalpy(Water, 8 MPa, 753.15 K)`, and all five
compressors' `h_s` asked at the golden's own tabulated entropy — is
**bit-identical** between golden and wheel, while every `(P,Hmass) →
T`/`Dmass`/`Smass` diverges by 3.0e-8 … 6.1e-6. rustprop reproduces the wheel
at those leaves to between 0 (bit-identical, three of them) and 9.0e-11, four
to five orders nearer than the golden. Declared tolerances, measured ×1.5:
the three Rankine documents 9.1e-6 (6.09863e-6 — **one shared operating
point**, `Volume(Water, 10 kPa, hf)`, identical to the last bit in all three);
`moving-boundary-hx-condenser` 7.1e-5 (4.7116e-5) and `-evaporator` 3.0e-5
(1.9800e-5), where a 3.0e-4 K / 1.5e-4 K tabulated-`T` error is spread over a
4.65 K subcooling and a 6.06 K superheat — denominators, not defects;
`new-library-components` 7.3e-6 (4.8539e-6); `ev-tms-cabin-steady` 4.6e-6
(3.0506e-6); `dual-evap-debug-check` 3.8e-6 (2.5016e-6);
`fluid-property-examples-example3-7` 1.7e-6 (1.1602e-6 — three lines and one
call, the cleanest instance of the mechanism in the corpus);
`component-cycles-refrigeration` 1.5e-6 (9.8592e-7);
`property-argument-seeding-subcritical` 1.2e-6 (8.1071e-7); the two
`component-variant-library-compressor` variants 4.0e-7 (2.6748e-7) and 1.3e-7
(8.7221e-8), one shared R134a (200 kPa, 283.15 K) suction state.

**Five of the eighteen are not that mechanism, and saying so is the point of
the wave.** `real-fluid-properties-solves-vapor-compression-cycle` is
`upstream-ps-flash-residual` at 3.0e-9 (2.023216e-9): it is the *same* R134a
`(P = 1016593.02212064, s = 1733.3507972613968)` flash `refrigeration-vcr`
already grades, and its 2.023216e-9 against that fixture's 2.023233e-9 is the
cross-check. Three more have no intercepted shape anywhere in them, so
`oracle-ph-table` would have been a lie: `cooker-faithful` 1.2e-7 (8.2728e-8),
`pressure-cooker-…-undersized-valve` 1.0e-8 (6.9361e-9) and
`steady-by-integration-chiller-bridge` 2.5e-9 (1.6357e-9) are
`ida-adaptive-path` — the vessels flash `(Dmass, Umass)`, which
`PhTableRegistry` does not intercept, and `EG50` gets no table at all because
`build()` needs a finite `pcrit` and `INCOMP::MEG`'s is `+inf`, checkable from
the wheel alone. All three localise the way an accepted-step artifact does:
rows 14–24 of 25 in the two cookers (after the relief valve cracks), rows 4–6
of 60 in the chiller bridge, whose four `FinalValue`/`MinValue` scalars agree
to 3.7e-12. For the cookers there is a sharper scale: at `cooker-faithful`'s
own recorded `(rho_cv, u_cv)` the wheel returns a `Quality` **8.8e-8** from the
`x_cv` that same golden row records, so the two engines are ~1300× closer to
each other than the golden is to CoolProp at its own state — which is what an
`atol` of `1e-4` on an `8e-4` variable buys.

The fifth, `dynamic-array-states-rod-with-ode45-also`, needed a **new
catalogued mechanism**, `ode45-adaptive-path`, at 2.2e-7 (1.4460e-7).
`ida-adaptive-path` describes a native-SUNDIALS golden against this build's IDA
transcription; the rod is the *same* `ode45` on both sides, with a linear,
constant-coefficient RHS and no property call at all. It was graded against a
60-digit closed-form oracle (`T(t) = Tss + exp(At)(T0 − Tss)`) rather than
against a property library: over the first 7 rows both engines sit 1.3729e-5 /
1.9433e-5 from the exact solution — the same figures to five digits — and over
the settled 93 the Java is 4.8322e-6 / 6.8338e-6 out while this build is
4.3470e-6 / 6.1475e-6, so **the port is the nearer of the two and their mutual
difference is smaller than either one's own error against the truth**. The
exact solution rises to 25 K monotonically; the golden exceeds 25 in 91 of
those 93 rows. The seed is `pow` in the step-size controller
(`ode/methods.rs:407`, `ode/integrator.rs:573`) — the transcendental this
file's own Comparison policy declares unspecified — which is why
`ida-adaptive-path`'s claim that the explicit transcriptions "reproduce the
Java's arithmetic bit-for-bit when the RHS does" now carries a dated amendment
in `tolerances-rustprop.json` rather than being left to read as fact.)*

*(The fluid-linkage row left this table on 2026-08-23, Wave G2: the
`carbondioxide` feature joined `rustprop-data`'s list in
`crates/frees-core/Cargo.toml` — measured at +25.5 KiB raw / +19.5 KiB
gzipped on the wire, 3092.6 of the 4096 KiB budget, the ci.yml header's
dated entry — and `docs_fluids_materials_03` promoted **bit-exact** on all
three variables at the corpus default with no tolerance entry.
`served_fluids`, the diagram picker, is deliberately unchanged: linking a
fluid's data and listing it in the picker are separate decisions. Corpus
775 → 776, pending 2 → 1.)*

> **The second of those decisions was taken on 2026-08-24 (Wave C1), by owner
> request: `CO2` is on `served_fluids` and in the picker.** The paragraph above
> is still the right rule — it is why the two changes are a wave apart — but its
> last-but-one sentence no longer describes the tree. Because G2 had already
> paid for the data, the picker entry cost **+142 bytes raw / +39 gzipped**.
> Wave C1 also promoted `props_realfluid_co2_transcritical` (corpus 983 → 984),
> which needs a `3.6e-9` entry in `tolerances-rustprop.json` on the
> `upstream-ps-flash-residual` mechanism — that mechanism's *second* instance,
> and the one where the residual shows in entropy rather than pressure. What was
> verified through the transcritical region, and the one diagram curve left
> absent on purpose, are in the Wave C1 amendment to
> [`docs/decisions/0009-rustprop-backend.md`](../docs/decisions/0009-rustprop-backend.md).

*(The decayed-through-zero row — three documents — left this table on
2026-08-23, Wave G1, via the decayed-signal measure under **Comparison
policy** above, not by any engine change: `sysdesign-ex01-thermal-network-2`
promoted at the corpus default (binding error 2.0e-15 — the ode45
transcription is bit-tight when the RHS is), `pressure-cooker` at a declared
1.1e-6 (measured 7.4435e-7 scale-anchored, mechanism `ida-adaptive-path`),
and `ev-battery-cooling-pid` at a declared 3.6e-6 (measured 2.4245e-6
scale-anchored, mechanism `oracle-ph-table` — the old claim that it "solves
and agrees, at `t_bat` rel 5.0e-13" was true of its *variables* only; the
G1 measurement found its ODE table carries a genuine ~2.4e-6 divergence on
the R134a evaporator chain, leaf-probed to the Java's `(P,Hmass)→T` table,
and its entry's reason records the probe). Corpus 772 → 775, pending 5 → 2.
The promotion costs the replay real time: `ev-battery-cooling-pid` alone
integrates ~100 s, taking the full release replay from ~82 s to ~200 s.
Wave G3's per-step caches took it back to ~145 s two days later.)*

*(Two rows left this table on 2026-08-21. The `CALL eigenvalues` / `eigen`
row: ledger item 34 closed, the three `eqsys-*` documents promoted, corpus
719 → 722. The `linalg::svd` column-sign row: ledger item 24 closed —
`linalg::svd` is now a line-faithful transcription of Commons Math 3.6.1's
JAMA-derived `SingularValueDecomposition`, whose Householder reflector signs
the goldens record, replacing the one-sided Jacobi kernel and its invented
largest-component-positive rule — and all six sign-blocked documents promoted
at the corpus default `1e-9` with no tolerance entry, corpus 722 → 728,
pending 11 → 5.)*

**No pending document is a property hold** — true again since Wave G2 linked
CO2 (2026-08-23). The 2026-08-22 docs harvest had briefly broken that: its
`docs_fluids_materials_03` was a *linkage* hold, not a backend gap (rustprop
served CO2 fine, this build just did not link its data), and closing it took
one Cargo feature. D8/D9 earned the property-hold-free set and the history
below records how. *(The pipeline-ordering row left 2026-08-21, Wave A4: MODULE instantiation moved to the expansion stage — after the unroller, at Java's own position — and `module_inside_for_loop` promoted exactly, display names and `block_count` included. Corpus 728 → 729.)* That is what
[D8](../docs/decisions/0008-coolprop-wasm.md) bought: it predicted twelve of the
then-26 would clear under a real CoolProp, and twelve of twelve did — Wave-3 F6
took nine and F8 the last three, corpus 707 → 719, pending 26 → 14, and
`tolerances-rustprop.json` was not touched by either. The per-document numbers
are in the two "Re-check 2026-08-18" sections near the end of this file; the
prose rows are in *Still pending* below.

> **How to re-check the whole staging area at once.** Copy `tests/parity.rs`,
> point `golden_dir()` at `corpus-pending/golden`, aim the tolerance path at a
> file that does not exist (so the grade is the corpus default `1e-9` with no
> exception available), and replace the panic at the end with a per-fixture
> `PROMOTE`/`HOLD` print. That reuses the gate's exact comparison logic instead
> of a hand-rolled approximation of it — which matters, because the `error` rule
> maps unrecognised Java exception types to "any Rust error", so a document both
> engines *refuse* scores green. Build with `--features rustprop-backend` or the
> copy will refuse to run at all. Delete the copy afterwards. **Never relax
> `tests/parity.rs` to make something pass.**
>
> The historical log below is chronological and long; the table above is the
> current state. **Most recent full re-check: 2026-08-18, Wave-3 F6** (26
> staged, 9 promoted), extended by **Wave-3 F8** (3 more) — both at the end of
> this file. The entries that follow run oldest-first from here.
>
> Re-check **2026-07-30, Phase 5 property dispatch — 51 staged, 19
> promoted, 32 remain.** Wiring `props::*` into `eval.rs` (every `prop$…`
> synthetic, the seven `eos_*`, the five `AdiabaticFlameTemp*` spellings, the
> seven `mix_*`, `eq_molefraction`, the nine `htc_*`/`dp_*`) unblocked
> `adiabatic-flame-temp`, `chem_equilibrium`, `chem_errors`, `chem_flame_temp`,
> `chem_heating_value`, `chem_idealgas`, `chem_mixture`, `chem_molar_mass`,
> `chem_nasa7`, `cubic-eos-properties`, `eos-cubic-spot-probe`,
> `eos-cubic-sweep`, `gas-transport-mixture`, `karman-rocket`,
> `material-conduction`, `multi-objective-beam`, `solid-materials`,
> `solid_absent_property_rejected` and `solid_unknown_material_rejected`.
> Seven of those needed one further fix: a fluid/material/formula **token**
> (`MolarMass(C8H18)`, `k_(Steel)`, `Enthalpy(CO2, T=…)`) was being registered
> as a display name, because the Rust parser reaches it through `parse_expr`
> while the Java grammar has a dedicated token rule. `parse_call_atom` now
> un-registers it, first-wins-preserving (`Cursor::forget_display_name_if_new`).
>
> The previous re-check's nine green-scoring holds are down to one: the eight
> CoolProp goldens now fail honestly (`… needs a real-fluid property backend and
> none is installed`), because `prop$` dispatch reaches a real call instead of
> refusing the family. Only the rank-deficient `solver_singular_linear_cycle`
> is still a deliberate withhold.
>
> **Re-check 2026-07-31, Phase 6 component expansion — 29 staged, 0 promoted.**
> Wiring the expander into `engine.rs` moved the three component-bearing
> documents past expansion but not past their *other* blocker, so none of them
> is promotable and the pending set is unchanged in size:
>
> | Document | Now blocked on |
> |---|---|
> | `ev-battery-cooling-pid` | `DYNAMIC` — Phase 7/8 |
> | `pressure-cooker` | `DYNAMIC` — Phase 7/8 |
> | `ev-thermal-management` | **expands and blocks correctly** (89-equation block 3), then fails on `no property table for fluid 'INCOMP::MEG[0.50]'` — the D1 table limit, not the component layer |
> | `thermo-compliance` | `Z(R134a, P=1, T=1)` outside the generated table — D1 |
> | `state-tables-multifluid` | `STATE TABLE` block type still unparsed |
> | `hx-correlations-fluid` | `viscosity` is not a tabulated output — D1 |
> | `heisler-transient` | string variables (`geom$ = 'wall'`) |
> | `solver_singular_linear_cycle` | still the deliberate rank-deficient withhold |
> | the other 21 | `PLOT` (5), `DYNAMIC` (5 more), `PARAMETRIC` (3), `SYMBOLIC` (1), CAS/control `CALL`s (6), `MODULE` inside `FOR` (1) |
>
> Component coverage grew instead through a **new** group of 46 `components_*`
> fixtures (see "The `components_*` group" above), not by promoting from here.
>
> **Re-check 2026-07-31, Phase 7 transient wiring — 52 staged, 29 promoted,
> 361 → 390.** Making `DYNAMIC`/`LINEARIZE` parse and routing them through
> `engine.rs` promoted:
>
> * the 19 `dyn_*` probes, minus `dyn_accessor_live`;
> * `damped-oscillator-ode`, `newton-cooling-transient`, `transient-heat-rod`,
>   `sounding-rocket-trajectory`, `engine-cycle-wiebe` — five of the eight named
>   `DYNAMIC` documents;
> * `linearize-thermal-siso`, `linearize-thermal-2x2`;
> * `damped-oscillator`, `driving-cycle-energy`, `projectile-trajectory`,
>   `solver_singular_linear_cycle` (the deliberate withhold now agrees).
>
> **Five goldens had to be re-dumped first.** `damped-oscillator-ode`,
> `newton-cooling-transient`, `transient-heat-rod`,
> `sounding-rocket-trajectory` and `engine-cycle-wiebe` were dumped *before* the
> dumper grew its `ode_tables` section, so their trajectories were not in the
> fixture at all. Re-running `tools/golden-dumper/run.sh` over them is what makes
> their promotion mean anything — otherwise the replay would compare `variables`
> (which a transient document barely populates) and pass vacuously.
>
> The 23 that remain, by blocker — **none is blocked by the transient path**:
>
> | Blocker | Count | Fixtures |
> |---|---|---|
> | control-systems `CALL`s not ported | 11 | `control-analysis-report`, `controller-design-lqr-pid`, `cruise-control`, `digital-control-c2d`, `estimator-gramian-balreal`, `inverse-laplace-residue`, `multi-output-destructuring`, `nichols-chart`, `root-locus-analysis`, `routh-stability`, `step-impulse-response` |
> | property-backend limits (D1) | 6 | `adv_moistair_W_passthrough`, `adv_moistair_dryair_three_way`, `ev-battery-cooling-pid`, `ev-thermal-management`, `hx-correlations-fluid`, `thermo-compliance` |
> | `SYMBOLIC` / `MODULE` inside `FOR` | 2 | `partial-fractions`, `module_inside_for_loop` |
> | string variables in a numeric position *(closed 2026-08-06)* | 1 | `heisler-transient` |
> | `method = ida` — the DAE path is assembled but not routed | 1 | `pressure-cooker` |
> | table-vs-CoolProp accuracy (worst 2.9e-6; needs a `tolerances.json` entry, deliberately not added) | 1 | `state-tables-multifluid` |
> | **cost, not correctness** — the live accessor converges to the oracle's `dk` to 4e-9 with a coarse `maxstep`, but does not finish in 7 min at the fixture's own `span/100` step cap. See `docs/status-phase7.md`. | 1 | `dyn_accessor_live` |
>
> **Re-check 2026-07-31, Phase 9 — the twelve holds that are *not* CAS or
> control, re-run one by one: 23 staged, 1 promoted, 22 remain.** The eleven
> control `CALL` documents and `partial-fractions` were left to Phase 9's own
> agents; every other pending document was replayed through
> `cargo run -qp frees-cli -- solve` and compared against its golden with the
> gate's own rules. **Nothing was found that Phases 5–7 had silently fixed**,
> but two of the twelve are real divergences rather than missing features, and
> one hold was withdrawn:
>
> | Document | Verdict | Evidence |
> |---|---|---|
> | `state-tables-multifluid` | **promoted** | Solves; the only gap is table-vs-CoolProp, worst 2.8963e-6 on `hw_1 = Enthalpy(Water, P=10 kPa, T=45 C)`. Phase 7 measured this and declined to add the `tolerances.json` entry; it is added now, because this is precisely the mechanism the file exists for and 17 fixtures already carry the same one. |
> | `heisler-transient` | **real divergence** *(closed 2026-08-06 — `parser/string_variables.rs`; promoted, corpus 701 → 702)* | Not `heisler_temp` — `props/heisler.rs` is ported. The missing piece is the Java's `parser/StringVariables.java`, run as the last line of `EquationParser.parseResult` (`EquationParser.java:336`): it *deletes* every `IDENT$ = 'literal'` equation from the numeric system and substitutes the literal at every use. The port's own pipeline docstring (`crates/frees-core/src/engine.rs:1372`) lists that line in the ported order, but no such module exists and nothing calls it, so `geom$ = 'wall'` reaches the blocker as an equation and dies with *"string literal 'wall' cannot be evaluated as a number"*. The golden proves the Java rule: `geom$` is in `display_names` but **not** in `variables`, and `block_count` is 14 for 14 numeric variables. ~130 LOC + one call site. |
> | `ev-battery-cooling-pid` | **real divergence** | Java solves it (`t_bat = 303.000000000087`, a 400-row `ode23s` table over 0..4000 s). Rust fails in under a second: *"Block 2 (2 equations) failed: Block 44 (79 equations) failed: `Dmass(R134a, P=1, Hmass=1)` is outside the generated property table"*. **Not a table-coverage limit** — every state the document really uses is servable (`Enthalpy(R134a, P=1.2 MPa, x=0)` = 265947.1989697985 against the golden's `hliq` 265947.2005481485, rel 5.8e-9; `Density(R134a, P=1.2 MPa, h=hliq)`, `Enthalpy`/`Density(R134a, P=350 kPa, x=1)` and `T_sat` all evaluate). `(P=1, Hmass=1)` is the **default initial guess** of the transient's 79-equation inner block, so the port never gets a finite first residual. See the note below. |
> | `adv_moistair_dryair_three_way`, `adv_moistair_W_passthrough` | blocked | `Enthalpy(AirH2O, T, P, W)` → `HAPropsSI`, which no shipped backend implements: `props::propfun::RealFluid::ha_props_si` has a *declining* default, and both overrides in the tree (`props/psychro.rs`'s `ToyHumidAir`, `props/propfun.rs`'s recorded-answer stub) live inside `mod tests`. Needs a humid-air model at CoolProp accuracy, not a stand-in. |
> | `hx-correlations-fluid` | blocked | D1. First line refused: `w_mu = Viscosity(Water, P=101325, T=320)` — *"'viscosity' is not a tabulated output"*. The `(P,h)` split table stores T, Dmass, Smass only. |
> | `thermo-compliance` | blocked | D1, and **only** `CompressibilityFactor`: probed directly, `Z_real = CompressibilityFactor(R134a, T=323.15 [K], P=1000000 [Pa])` → *"'Z' is not a tabulated output"*, while `Volume` at the identical state returns 0.02179627269999244. Everything else in the document (`T_crit`, `P_crit`, `StagnationTemp`, `StagnationPres`) evaluates. **Do not trust the message the document itself produces** — *"`Z(R134a, P=1, T=1)` is outside the generated property table"* — it names the wrong cause and quotes 1 Pa / 1 K, because `solve_block_with_fallback`'s merge rung resets every variable of the merged block to its initial guess before re-solving, and the *last* rung's error is what gets reported. |
> | ~~`ev-thermal-management`~~ | **promoted 2026-08-06** | Was: D1, `no property table for fluid 'INCOMP::MEG[0.50]'`. Closed by [D7](../docs/decisions/0007-auxiliary-property-grids.md)'s `FRAUX1` grids plus an `r1234yf.phtab`. It needed **four** capabilities, not one — the glycol, R1234yf, air transport for `htc_extair`, and saturation-line transport for `htc_evap`/`htc_cond`/`dp_2phase`. Grades at 8.951e-4 with a measured tolerance; see `tolerances.json` for why that number is an operating-point amplification and not a table defect. |
> | `pressure-cooker` | blocked | `method = ida`; the implicit-DAE path is assembled but not routed. |
> | `estimator-gramian-balreal` | blocked | Control, as expected: `CALL lqe` refuses first, then `gram` and `balreal` behind it. Phase 9's control suite owns it. |
> | `module_inside_for_loop` | blocked | Unchanged cluster-3 pipeline ordering: MODULE flattening must move past the `FOR` unroller. Refused loudly, which is the intended behaviour until it does. |
> | `dyn_accessor_live` | blocked (cost) | Re-timed: **no output after 420 s**, consistent with Phase 7's >7 min. |
>
> **Two findings the next agent should not have to re-derive.**
>
> 1. *`docs/status-phase7.md`'s follow-up hypothesis 1 is closed — negatively.*
>    It asks whether `try_univariate_bracketing_solve` is "gated too narrowly"
>    because it requires `uses_property_call`. It is not: the Java gates it the
>    same way, at `EquationSystemSolver.java:1148-1152`, with the reason spelled
>    out — *"Scope this resort to property inversions … For ordinary algebra a
>    bracketing rescue would bypass the user's Newton iteration-limit stop
>    criterion and could pick a different root than Newton's basin."*
>    `FinalValue('Temp') = 30` has no property call, so **neither** engine
>    brackets it. The remaining lever is hypothesis 2 (`solve_pinned` re-blocks
>    from scratch every step). Related and worth knowing: the Java's ladder is
>    wall-clock bounded (`config.deadlineNanos()`, checked inside the bracketing
>    sampler); wasm32 has no clock, so the port cannot inherit that escape hatch
>    and must be cheap rather than interruptible.
> 2. *A property failure at the initial guess is handled differently in the two
>    engines, and `ev-battery-cooling-pid` is where it shows.* Java
>    `NewtonSolver.residuals()` catches `PropertyEvaluationException` and writes
>    `NaN` — *"an invalid state point … is a bad region, not a fatal error"* —
>    so the line search and the retry ladder move on. The port matches that
>    inside Newton (`engine.rs::BlockProblem::residual` fills `NaN` for
>    `Property`/`Evaluation` errors) but **not before it**: `engine.rs::solve_block`
>    evaluates a pre-Newton probe whose error is returned, so such a block never
>    enters Newton at all and drops straight into `solve_block_with_fallback`.
>    At 1×1 that ladder rescues it — probe: `T_t = Temperature(R134a, P=P_x, h=250000)`
>    with `T_t = 280` starts at the out-of-box guess `P_x = 1` and still solves
>    to `P_x = 372708.40925159707` — but at 79 unknowns only transformed guesses
>    and a bidirectional merge apply, and neither fires. The Java engine was not
>    re-run on this document (its golden already records that it converges), so
>    the *exact* rung Java gets past t = 0 on is still unidentified; the two
>    candidates are this probe and a better-seeded ODE inner solve. Note that
>    even once it starts, this fixture carries the same live-accessor shape as
>    `dyn_accessor_live` (`t_bat = FinalValue('bp.t')` over a 400-point
>    `ode23s`), so it may hit the cost wall next.

```bash
tools/golden-dumper/run.sh fixtures/corpus-pending/corpus fixtures/corpus-pending/golden
```

The staged set began as the 48 whole example documents from `web/src/examples.ts`
\+ `web/src/defaultExample.ts` (extracted verbatim, one file per `Example.id`)
plus 56 hand-authored Phase-4 documents mirroring the Java unit tests named in
each document's header comment. Adversarial verification since then added ~70
more probe documents written specifically to try to make the two engines
disagree; the ones that agreed were promoted directly.

> The harvest also turned up a **duplicate `Example.id` in `web/src/examples.ts`**:
> two different Thermodynamics entries both claim `rankine-cycle`, so the second
> is staged as `rankine-cycle-2.frees`. Anything keying examples by id (deep
> links, the command palette, `share.ts`) can only reach the first.

### What blocks the pending set — the closed clusters, and why

*The live list is the table under "What is pending today" above; this section is
the cluster-by-cluster history, kept because each entry records what closed it.*

The `display_names` cluster (30 documents) and the unwired-kernel cluster
(`Integral`, `CALL Interp2`, `det$<n>` for n > 3) are **closed** — see the
engine-fix notes at the end of this section. So is the cluster that was the
largest of all:

**0. Property-backend limits (0).** *Closed 2026-08-18 by
[D9](../docs/decisions/0009-rustprop-backend.md)'s rustprop backend.* At its
peak this cluster held twelve documents — `HAPropsSI`, `Air` state points,
single-phase `(P,T)` transport, `CompressibilityFactor`, `INCOMP::MEG` — every
one of them a limit of D1's `(P,h)` tables rather than of the engine. All
twelve are promoted and none needed a tolerance entry. **The right response to a
new property hold is no longer a table: it is a rustprop question.**

**1. Block types the wasm engine still refuses (0).** *Closed by Phase 7.* Every
block form the grammar admits now parses into `Document` —
`parser/toplevel.rs::unsupported_construct` returns `None` for every token. The
`PLOT` / `PARAMETRIC` / `STATE TABLE` three landed in Phase 8's parser work and
`DYNAMIC` / `LINEARIZE` in Phase 7's.

**2. Library calls not ported (0 — closed 2026-08-21).** The eleven
control-systems `CALL`s (`lqr`, `lqe`, `c2d`, `routh`, `residue`, `tf2ss`,
`pole`, `nichols`, `rlocus`, `step`, `ss2tf`) landed in Phase 9, string
variables (`geom$`) on 2026-08-06 (`parser/string_variables.rs`), and the
material database (`E_`, `k_`), `MolarMass`, `eos_z` and `AdiabaticFlameTemp`
in Phase 5. The last member — `CALL eigenvalues` / `eigen`, which the Phase-12
harvest found unwired (ledger item 34) — was wired on 2026-08-21
(`parser/expand.rs::flatten_eigen` → `linalg::eval_intrinsic`), and the three
`eqsys-*` documents promoted at the corpus default `1e-9`.

**3. Pipeline-ordering deviation (0 — closed 2026-08-21, Wave A4).**
`module_inside_for_loop`: Java unrolls `FOR` *during* flattening, so
`CALL Twice(i : r[i])` inside a two-iteration loop produces two module
instances (`twice$1$…`, `twice$2$…`). This port used to flatten CALLs in a
pass before unrolling and refused the shape. The fix is what this paragraph
predicted: MODULE instantiation moved past the unroller — an in-FOR module
CALL now rides through `procedures::flatten_calls` intact and
`parser/expand.rs::flatten_module_call` (a transcription of
`EquationParser.flattenModuleCall`) instantiates it per iteration with the
loop variable bound, the shared instance counter re-based via
`flatten_calls_counted`. The fixture is promoted and matches Java exactly,
display names and `block_count` included.

**4. Ill-posed by construction (1).** `solver_singular_linear_cycle` is
structurally square but rank-deficient (`x = y+1`, `y = z+1`, `z = x-2` reduce
to `x = z+2` twice), so its solution set is a *line*. Both engines return a
point on that line with a residual at machine zero — Java `(2, 1, 0)`, Rust
`(2+6.6e-14, 1+6.6e-14, 6.6e-14)`. That agrees inside the tolerance table today,
but promoting it would freeze an arbitrary point of a continuum into the gate.
Held deliberately, for the same reason as the CoolProp eight.
*(Stale as written — kept for the reasoning. The Phase 7 re-check promoted it
after the two engines agreed inside tolerance; see the re-check note above.
This paragraph predates that and survived two audits; corrected 2026-08-05.)*

Separately, **8 documents have no usable golden**: the Java oracle refused them
with `IllegalStateException: The CoolProp native library is not available` on
this machine. Rust also refuses them, so a naive comparison scores them green —
that would bake a missing `.so` into the parity gate and turn it red the day
someone installs CoolProp. They are deliberately **not** promoted. Regenerate
their goldens with `COOLPROP_LIBRARY` set before trusting them.

> **Closed 2026-07-31.** Every one of those eight has since been re-dumped with
> `COOLPROP_LIBRARY` set — `run.sh` now exports it itself. No golden anywhere in
> `fixtures/` records an `IllegalStateException` any more: all 22 remaining
> pending goldens carry `expect.error = null` and real values. The paragraph
> above is kept because the hazard it describes is permanent — a golden dumped
> without the native library is a *recorded failure*, not a fixture.

### Engine fixes this staging area produced

Each of these was a genuine Rust-vs-Java divergence found by replaying a staged
or probe document through both engines, and each unblocked fixtures wholesale:

* **`display_names` was reconstructed, not recorded.** The port rebuilt the map
  with a lexer pass over the source and then filtered it to solved variables.
  That is wrong in both directions: unit spellings and TABLE column headers won
  the first-seen race against real variables (`cp = 1004 [J/kg-K]` bound
  `k -> "K"`, beating a later `k = 1.4`), while FUNCTION/MODULE body-locals,
  formals, integration variables, bare container names and the original casing
  of expanded elements were all missing. The map is now accumulated where Java
  accumulates it — `Cursor::record_display_name` at the two `AstBuilder` sites
  (`visitVarAtom`, `visitArrayAtom`), the CALL flattener's namespaced module
  variables, and an element rule replayed over the expanded system. 30 fixtures.
* **`det$<n>` was never dispatched.** `parser::expand` emits it for any `det(A)`
  larger than 3x3, and `linalg::eval_intrinsic` implements it (plus `qr$`,
  `chol$`, `expm$`, `svd$`) — but `eval::eval_synthetic` had no arm, so the whole
  module was unreachable from user text and every such document died with
  "not yet supported: det$4".
* **`CALL LUDecompose` and `CALL Interp2` were refused before their flatteners
  ran.** Java flattens PROCEDURE/MODULE calls and the matrix intrinsics in one
  pass; this port splits that in two, and stage 2 was refusing by name every
  intrinsic CALL — including the two whose flatteners live in stage 3. Both
  `expand::flatten_lu_decompose` and the new `expand::flatten_interp2` were
  dead code. `procedures::EXPANDED_CALL_TARGETS` now passes them through.
* **Ignored-output sinks leaked into the solution.** Omitting a trailing CALL
  output (`CALL LinFit(x, y : m, b)`) mints a hidden `~ignored~N` sink that the
  solver must determine but that Java never surfaces
  (`EquationSystemSolver.java:1888`). The port had `is_ignored_sink` but never
  called it on the result path, so `~ignored~0 = 1.0` showed up as a result row.
  `engine.rs` now filters it out of the values map, out of `check`'s variable
  list, and out of both reported counts (`surfacedVarCount` and
  `surfacedEqs = equations − (allVars − surfacedVars)`, so hiding a sink's
  variable also hides its equation). 2 fixtures.

### Authoring hazard: sink names carry a JVM-global counter

`EquationParser.IGNORED_SINK_SEQ` is a **static `AtomicLong`, never reset per
document**, so the `N` in `~ignored~N` depends on how many documents ran before
it *in the same JVM*. Running one document inside a batch produced
`~ignored~1`; running it alone produced `~ignored~0`, from the same engine and
the same source.

This is invisible for a **scalar** sink — Java hides it from `variables` *and*
`display_names`, so those fixtures are reproducible and are the ones frozen
(`call_linfit_omitted_r2`, `call_linfit_omitted_b_and_r2`). It is fatal for an
**array or matrix** sink: Java hides it from `variables` but **keeps** its
elements in `display_names` (`~ignored~1[1,1]` …), which `tests/parity.rs`
compares exactly. **Do not freeze a fixture whose golden mentions
`~ignored~N[`** — `[L] = LUDecompose(A)`, `CALL SVD(A : U, S)`,
`CALL QR(A : Q)`, `CALL FFT(re, im : out_re)` and the like. The Rust engine's
per-document counter is arguably better behaved and will never match the Java
batch value.

(A correction, found by the Phase 12 harvest: `~` **is** user-writable in the
destructuring form — `[whole, ~] = DivMod(17, 5)` parses — and an explicit
scalar `~` there is **not** hidden from `display_names` the way an
omitted-trailing scalar sink is. The golden records `~ignored~N` with the
JVM-batch-global counter value, so the "scalar sinks are safe" rule above
covers *omitted-trailing* sinks only. Do not freeze an explicit-`~`
destructuring fixture either; `multiout-user-function-with-tilde-discard` was
dropped for exactly this. In `CALL` argument lists `~` remains a
`ParseException`.)

### Still pending — the two, in detail

The summary is the table under *What is pending today*; this is the per-document
reason. Checked 2026-08-19 (then 14; the three `eqsys-*` and the six sign-hold
documents promoted 2026-08-21): all pending goldens carry `expect.error =
null`, i.e. **the Java engine solved every one of them** — none is pending
because the oracle refused it.

| Document | Why it is held |
|---|---|
| ~~`linalg-full-svd`~~, ~~`multiout-svd-discard-with-tilde`~~, ~~`ctldesign-balreal-invariants-integration`~~, ~~`ctldesign-bare-matrix-names-into-control-calls-resolve-shapes`~~, ~~`-2`~~, ~~`estimator-gramian-balreal`~~ | **promoted 2026-08-21** — was: `linalg::svd`'s column-sign convention (sign-only `U`/`V`/balancing-basis flips; ledger item 24). The convention proved **not statable as a normalisation** — in `linalg-full-svd`'s golden, V column 2's largest component is *negative*, contradicting any make-positive rule — so `linalg::svd` was replaced with a line-faithful transcription of Commons Math 3.6.1's JAMA-derived `SingularValueDecomposition`, whose Householder reflector signs reproduce the oracle's element-exact. All six grade at the default `1e-9` with no tolerance entry (the machine-zero Gramian off-diagonals pass under `ABS_TOL = 1e-12`). Corpus 722 → 728, pending 11 → 5. |
| ~~`eqsys-eigen-waits-for-matrix-entries-solved-elsewhere`~~, ~~`eqsys-solves-eigen-decomposition-with-vectors-and-downstream-equations`~~, ~~`eqsys-solves-eigenvalues-of-symmetric-matrix`~~ | **promoted 2026-08-21** — was: `CALL eigenvalues` / `eigen` not wired (ledger item 34). `parser/expand.rs::flatten_eigen` now emits the `eigen$val\|re\|im\|vec$…` synthetics and `linalg::eval_intrinsic` decodes them with the Java kernel's ascending (real, imag) sort and unit-norm/sign-fixed eigenvectors. All three grade at the default `1e-9` with no tolerance entry. Corpus 719 → 722, pending 14 → 11. |
| ~~`sysdesign-ex01-thermal-network-2`~~ | **promoted 2026-08-23 (Wave G1)** — was: asymptotic FP noise, `ode:m$port$qdot` decaying through zero with no denominator for the pointwise measure, and "only a larger `ABS_TOL` could express it". That framing missed the third option: the **decayed-signal measure** (see *Comparison policy*) anchors each cell to its own column's range instead of loosening anything globally, and under it this document grades at the corpus default with **no tolerance entry** — binding error 2.0e-15, the wave's proof that the ode45 transcription is bit-tight when the RHS is. Corpus 772 → 775 with the two rows below. |
| ~~`ev-battery-cooling-pid`~~ | **promoted 2026-08-23 (Wave G1)** at a declared 3.6e-6 (`tolerances-rustprop.json`, mechanism `oracle-ph-table`) — was: held for `ode:pid$e` decaying to ~8.7e-11. The old "solves and agrees (`t_bat` rel 5.0e-13)" was true of its **variables** only: the G1 measurement found the ODE table carries a genuine 2.4245e-6 scale-anchored divergence on the R134a evaporator chain, and the leaf probe pinned it to the Java's `(P,Hmass)→T` run-time table integrated into the golden's RHS (Java 320.7653749901327 vs rustprop 320.765512845656 at the trajectory's own steady state, `(P,x=1)` control bit-identical — the entry's `reason` has the full trail). Also the replay's new slowest promoted document: ~100 s of the full replay's ~200 s. |
| ~~`module_inside_for_loop`~~ | **promoted 2026-08-21 (Wave A4)** — was: `MODULE` flattening ran before `FOR` unrolling. `parser/expand.rs::flatten_module_call` now transcribes `EquationParser.flattenModuleCall` at the after-unroll position (loop vars bound, `r[i]` outputs resolving to element variables, per-iteration namespaces, `putIfAbsent` display names), with the instance numbering continued from stage 2 via `flatten_calls_counted`. One recorded residue: a top-level MODULE call written after a FOR containing MODULE calls would number differently from Java — no corpus document has that shape. |
| ~~`pressure-cooker`~~ | **promoted 2026-08-23 (Wave G1)** at a declared 1.1e-6 (`tolerances-rustprop.json`, the new `ida-adaptive-path` mechanism: native SUNDIALS in the oracle vs this build's IDA transcription, RHS ulps turned into a different accepted-step sequence, everything two decades inside the rtol 1e-6 both ran; measured 7.4435e-7 scale-anchored at `cook$vent$mdot`). The decayed `steel$port$qdot` that held it anchors at 2.9e-8. History: was `method = ida` — **the routing landed 2026-08-21 (Wave A3)**: `ode/dynamic.rs::solve_with_ida` ports `DynamicSolver.solveWithIda` (grid `max(points ?? 200, 2)`, no `maxstep` on this path, rows straight from IDA's state vector, the `calcConsistentIc → reinit` fallback, the root/set-event loop), and wiring the first real document flushed out **two transcription bugs against SUNDIALS v6.7.0's `ida.c`** — `IDARestore` un-scaling all of `phi[1..=kk]` instead of `ns..=kk`, and `IDASetCoeffs`' `alpha0` summed over `alpha[1..=kk]` instead of `alpha[0..kk-1]` (a ~4× inflated error constant at order 5) — which had stalled the run at t ≈ 696 s. Fixed, the document integrates all 1200 s and matches the oracle to ≤ 3.5e-8 on every real signal. The hold that remained after A3 — `steel$port$qdot` → 0 collapsing the relative denominator (rel up to 5.7e-3 on microwatt cells) — is exactly what the G1 measure closed without asserting nothing: the anchored gate still demands 1.1e-6 of the signal's 1 497.86 W range. |
| `dyn_accessor_live` | **Nothing missing — cost.** Re-measured 2026-08-21 (Wave A5): with the per-step structural cache (`engine.rs::PreparedPinnedSolver` — the blocking/spec/seeding half of `solve_pinned` hoisted out of the per-step loop) it now **finishes and grades perfectly** — ode_tables rows bit-identical, `dk` at rel 8e-16 — in **~12 minutes**, at the fixture's own `span/100` cap. The cost is structural: the outer Newton's wild iterates (`k` reaches 4.9e7 before bracketing back) each drive explicit ode45 into the 10⁶-step `MAX_STEPS` grind at stiffness ~5e7, exactly as the Java does — the Java is simply ~10× faster per micro-step. Promoting it would multiply the whole replay's wall clock (~82 s) ninefold, so the hold stands on cost alone. The same cache cut the full replay from ~122 s to ~82 s — the transient fixtures' per-step solves all ride it. *(Re-measured 2026-08-23, Wave G3, which went after that "~10× per micro-step" with a callgrind profile: ~46 % of the run was the allocator. `PinnedBlockCache` hoists the per-block symbolic derivatives, bounds and row dependencies out of the per-step loop, the pin source text rewrites through its existing buffer, the Newton scope writes stop cloning names, and the pin list is a slot-rewritten template. Per-step algebraic cost halves — native `transient_dyn` 30.2 → 14.9 ms, browser 60.3 → 23.9 ms — and this document lands at **~5.6 min** against the replay's **~145 s**. Still ~2.3× the whole gate for one document: the hold stands, smaller.)* |

*(Phase 9 promoted eleven documents out of an earlier version of this table on
2026-08-01 — `control-analysis-report`, `controller-design-lqr-pid`,
`cruise-control`, `digital-control-c2d`, `inverse-laplace-residue`,
`multi-output-destructuring`, `nichols-chart`, `root-locus-analysis`,
`routh-stability`, `step-impulse-response`, `partial-fractions` — all at the
default `1e-9` with no tolerance entry. The corpus was 512 fixtures then. The
property-blocked rows this table used to carry went at Wave-3 F6/F8; their
measurements are in the two "Re-check 2026-08-18" sections below.)*

**Re-check 2026-08-05, Phase 12 harvest — 212 extracted from 13 Java test
classes, 170 promoted, corpus 531 → 701, pending 11 → 32.** The harvester
(`tools/harvest-java-tests/harvest.py`, manifest beside it) and its full
triage are in `docs/status-phase12.md`. The 21 new pending rows, by blocker:

| Documents | Blocked on |
|---|---|
| ~~`linalg-full-svd`~~, ~~`multiout-svd-discard-with-tilde`~~, ~~`ctldesign-balreal-invariants-integration`~~, ~~`ctldesign-bare-matrix-names-into-control-calls-resolve-shapes`~~, ~~`-2`~~ | **promoted 2026-08-21** — was the recorded `linalg::svd` **column-sign convention** divergence (same mechanism as `estimator-gramian-balreal` above), closed by the Commons Math SVD transcription (ledger item 24) |
| ~~`sysdesign-ex16-moving-boundary-evaporator`~~ | **promoted 2026-08-06** — was the default-guess property probe divergence (`T(R134a, P=1, Hmass=…)`). Closed by porting `seedPropertyArgumentGuesses`, then by wiring it to the **main** solve path as well: it had first been added only to `solve_equation_list`, which the port's steady-state solve bypasses, so the transient documents were fixed and this one was not. Grades at 3.235e-6 — larger than its `(P,x)` siblings because it inverts `Temperature(R134a, P, h)`, the one shape the Java also tables. Corpus 704 → 705. |
| ~~`eqsys-eigen-waits-for-matrix-entries-solved-elsewhere`~~, ~~`eqsys-solves-eigen-decomposition-with-vectors-and-downstream-equations`~~, ~~`eqsys-solves-eigenvalues-of-symmetric-matrix`~~ | **promoted 2026-08-21** — was: `CALL eigenvalues` / `eigen` not wired, the one genuinely new gap the Phase-12 harvest found (ledger item 34, closed) |
| ~~`hvac-problem2-face-and-bypass`~~, ~~`hvac-problem3-psychrometric-balancing`~~, ~~`hvac-problem9-air-supply-wet-bulb`~~, ~~`sysdesign-ex12-moist-air-ahu`~~, ~~`sysdesign-ex13-humidifier`~~ | **promoted 2026-08-18 (Wave-3 F6)** — the `HAPropsSI` humid-air gap (as `adv_moistair_*` above) is closed by [D9](../docs/decisions/0009-rustprop-backend.md). All five grade at the default 1e-9 with no tolerance entry; worst per document 2.29e-15, 1.25e-15, 0, 0, 0. |
| ~~`sysdesign-ex06-pneumatic`~~, ~~`sysdesign-ex06-pneumatic-2`~~, ~~`sysdesign-ex07-pneumatic-servo`~~ | **promoted 2026-08-18 (Wave-3 F8)** — was: no **state** table for `Air`. [D7](../docs/decisions/0007-auxiliary-property-grids.md) added a `(P,T)` grid carrying air's `viscosity`/`conductivity`/`Cpmass`/`Dmass` — enough for `htc_extair`, not for these, which want `Enthalpy`. **Do not generate an `air.phtab`**: [D8](../docs/decisions/0008-coolprop-wasm.md) supersedes it, and [D9](../docs/decisions/0009-rustprop-backend.md) delivered it — rustprop serves `Air` `Enthalpy` and `Temperature(Air, P, h)` directly, through the pseudo-pure `HSU_P` flash Wave-2 R6/R7 ported. All three grade at the default 1e-9 with no tolerance entry. Corpus 716 → 719, pending 17 → 14. |
| ~~`sysdesign-ex11-liquid-cooling-loop`~~ | **promoted 2026-08-06** — the same `INCOMP::MEG` grid ([D7](../docs/decisions/0007-auxiliary-property-grids.md)) that unblocked `ev-thermal-management`. Grades an order of magnitude tighter (1.310e-4) because it sits off the laminar↔turbulent blend band. Corpus 703 → 704. |
| ~~`sysdesign-ex17-ac-expansion-valve`~~, ~~`sysdesign-ex20-zeotropic-blend`~~ | **promoted 2026-08-06** — D1 table-vs-CoolProp accuracy, measured at 7.8874e-8 and 2.8901e-8 and entered in `tolerances.json` at 2e-7 / 1e-7. Both reach R134a saturation states through `(P,x)`, which `PropertyFunctions.java` does *not* intercept, so the goldens are bit-exact CoolProp and the gap is this build's `.phtab` interpolation alone. Corpus 705 → 707. |
| ~~`sysdesign-ex01-thermal-network-2`~~ | **promoted 2026-08-23 (Wave G1)** at the corpus default, via the decayed-signal measure (see *Comparison policy*) — the "only a larger `ABS_TOL`" framing above missed the per-column anchor. Was: asymptotic FP noise on `ode:m$port$qdot` decaying through zero. |
| ~~`ev-battery-cooling-pid`~~ | **promoted 2026-08-23 (Wave G1)** at a declared 3.6e-6 (mechanism `oracle-ph-table` — the G1 measurement found real table-integrated property divergence beyond the decayed `pid$e` that held it; details in the *Still pending* table above). |

Two authoring hazards found the hard way, both worth repeating:

* **Brace comments do not nest.** A `{` inside a `{ … }` comment ends the
  comment at the first `}`, and the rest of the line is parsed as equations.
  `sum_{i,j=1..3}` in a header comment turned a working document into a syntax
  error and the golden dutifully recorded it. An unexpected `ParseException` in
  a generated golden usually means the *fixture* is wrong, not the engine.
* **`#` marks a built-in constant, not a user name.** `E# = 72000` is an attempt
  to redefine a built-in and makes the document overspecified; the activation
  energy has to be `Ea`.

**Re-check 2026-08-18, Wave-3 F6 — the property-blocked holds, under the
rustprop backend: 26 staged, 9 promoted, corpus 707 → 716, pending 26 → 17.**
[D8](../docs/decisions/0008-coolprop-wasm.md) predicted that a real CoolProp
backend would clear **twelve** of the then-26 pending fixtures — humid air 7,
`Air` `(P,h)` state 3, `(P,T)` transport 1 (`hx-correlations-fluid`), `Z` 1
(`thermo-compliance`). [D9](../docs/decisions/0009-rustprop-backend.md) shipped
that backend. Each of the nine in D8's humid-air / transport / `Z` groups was
replayed with `tests/parity.rs`'s own comparison logic pointed at
`corpus-pending/golden` and **no** tolerance file, so the grade is the corpus
default `1e-9` with no exception available:

| Document | Worst variable deviation | Verdict |
|---|---:|---|
| `adv_moistair_W_passthrough` | 0 | promoted |
| `adv_moistair_dryair_three_way` | 0 | promoted |
| `hvac-problem9-air-supply-wet-bulb` | 0 | promoted |
| `sysdesign-ex12-moist-air-ahu` | 0 | promoted |
| `sysdesign-ex13-humidifier` | 0 | promoted |
| `hvac-problem3-psychrometric-balancing` | 1.25e-15 | promoted |
| `hvac-problem2-face-and-bypass` | 2.29e-15 | promoted |
| `hx-correlations-fluid` | 1.24e-13 | promoted |
| `thermo-compliance` | 1.31e-11 | promoted |

A **0** here is the harness's own `rel_diff`, which returns zero when the two
values differ by no more than the `1e-12` absolute band — for a psychrometric
enthalpy of ~5e4 J/kg that is agreement to the last few bits, not a rounded
report. The seven humid-air documents are all at or under **2.29e-15**; the two
loosest are the two that are not humid-air documents at all —
`thermo-compliance`, whose worst variable is `Z_real` **alone** at 1.31e-11
(`v_real` at the identical `(P,T)` R134a state is bit-identical, so the residue
is in how `Z` itself is formed, not in the density), and
`hx-correlations-fluid`, which drives 38 transport and correlation outputs
across water, air and both R134a saturation branches. **No entry was added to
`fixtures/tolerances-rustprop.json`** — nine documents joined the corpus and the
file still had exactly the ten entries F5 re-baselined. *(Ten until Wave G1,
2026-08-23, which added the file's first two transient entries — see the
pending table.)*

Two findings beyond the nine:

* **D8's remaining three clear as well.** `sysdesign-ex06-pneumatic`,
  `sysdesign-ex06-pneumatic-2` and `sysdesign-ex07-pneumatic-servo` — the
  `Air` **state**-table group — grade at 1.43e-14, 0 and 0 under the same
  replay. They are left staged only because F6's scope was the nine; promoting
  them is a file move and a table edit, and takes pending 17 → 14. That would
  make D8's prediction exact: twelve for twelve.
  *(Done — **Wave-3 F8**, below.)*
* **The other twelve holds are unchanged and none of them is a property
  blocker.** Re-measured in the same run: the five `linalg::svd` column-sign
  documents plus `estimator-gramian-balreal` still differ by sign only
  (relative deviation ~2.0 by construction) *(re-checked 2026-08-21: no
  longer — the Commons Math SVD transcription closed ledger item 24 and all
  six promoted)*; the three `eqsys-*` still refuse
  with "CALL `eigenvalues`/`eigen` is not yet supported" *(re-checked
  2026-08-21: no longer — wired and promoted)*; `module_inside_for_loop`
  still refuses at parse; `pressure-cooker` still refuses `method = ida`; and
  `sysdesign-ex01-thermal-network-2` still fails on `ode:m$port$qdot` decaying
  through zero (worst 9.4e-7 on a cell of magnitude 4.8e-6). `dyn_accessor_live`
  and `ev-battery-cooling-pid` were not re-timed — both are recorded as cost /
  decayed-signal holds, not property holds. *(Re-checked 2026-08-23:
  `module_inside_for_loop` was promoted by Wave A4, `pressure-cooker` routed by
  A3 and promoted by G1, and the two decayed-signal holds promoted by G1's
  measure — see the pending table for all three.)*

**Re-check 2026-08-18, Wave-3 F8 — the `Air` state group: 3 staged, 3 promoted,
corpus 716 → 719, pending 17 → 14.** This closes D8's prediction at twelve for
twelve. Same replay method as F6 — `tests/parity.rs`'s own comparison logic
pointed at `corpus-pending/golden` with the tolerance path aimed at a
nonexistent file, so the grade is the corpus default `1e-9` with no exception
available to fall back on:

| Document | Graded worst | Raw worst relative | Verdict |
|---|---:|---:|---|
| `sysdesign-ex06-pneumatic` | 0 | 2.46e-15 (`ori.t_in`) | promoted |
| `sysdesign-ex07-pneumatic-servo` | 0 | 2.46e-15 (`sv.t_in`) | promoted |
| `sysdesign-ex06-pneumatic-2` | 2.50e-14 | 4.63e-11 (`ode` `vol$in$mdot`) | promoted |

Two columns, because one number is not enough for a transient. **Graded worst**
is `parity.rs`'s own `rel_diff`, which returns 0 inside a `1e-12` *absolute*
band; **raw worst relative** drops that band. For the two steady documents the
graded 0 is `t_in = 300.00000000000097` against the oracle's
`300.0000000000002` — 13 ulp on 300 K, or 2.46e-15 relative, i.e.
`Temperature(Air, P, h)` answering to the last few bits.

`sysdesign-ex06-pneumatic-2` needs both columns, and it also corrects F6's
figure for it: F6's 1.43e-14 was measured over `variables`, and this document
has exactly **one** variable (`Pf`) with its whole trajectory in a 100x20
`ode_tables` entry that the number never touched. Swept over the trajectory as
well, the graded worst is **2.50e-14** and the raw worst is **4.63e-11**, and
they are different cells rather than two views of one: `cap$port$p` at 652 kPa
(absolute gap 1.6e-8 Pa, outside the band) and `vol$in$mdot` at 6.09e-5 kg/s
(absolute gap 2.8e-19 kg/s, well inside it). Both are comfortably under the
`1e-9` default, and the ODE cells are compared by `parity.rs` either way — F6's
verdict was right, its measurement just under-reported. Nothing was added to
`fixtures/tolerances-rustprop.json`; it still carried exactly the ten entries F5
re-baselined *(twelve since Wave G1, 2026-08-23 — the two transient entries)*.

> **`Air`'s `(P,h)` failure window is real, and these three are nowhere near
> it.** Measured at the same time, directly through `rustprop::props_si`, by
> sweeping `h = h_L + q·(h_V − h_L)` at `q ∈ {0, 1e-6, 1e-4, 1e-3, 0.01, 0.05,
> 0.1, 0.3, 0.5, 0.9, 1}`. Inside the dome at 1 bar (79 K),
> `Temperature(Air, P, Hmass)` answers at `q = 0` and `1e-6`, **refuses at
> `1e-4`, `1e-3` and `0.01`** — "unable to bracket the (p,X) solution in
> [59.77, 78.80] (residuals −1.05e3, −3.39e-2) … the derivative path is not
> ported" — and answers again from `0.05` up. At 7 bar (101 K) the refusals are
> `1e-3` and `0.01` only. So the window is a low-quality sliver on the bubble
> side, bounded above somewhere in `(0.01, 0.05]`. The three fixtures ask at
> `h = 424 950 J/kg` and 1–7 bar, which is `T = 298.7…300.0 K` and `Q = -1` —
> single-phase gas, some 167 K above Air's critical temperature. Note what the
> refusal message says about itself: an **unported derivative path**, i.e. a
> rustprop-side gap rather than upstream behaviour being reproduced. Not
> verified against upstream from here — the wording is rustprop's own. Out of
> scope for this repo (no fixture reaches it) but not a closed question.

**Nothing in this repo still excludes `Air`.** F8 checked both of the places
that once did. `RealFluid::served_fluids` (`props/rustprop_backend.rs`) lists
it — F3 added it, and the D6 amendment to
[D9](../docs/decisions/0009-rustprop-backend.md) explicitly kept it there while
retiring the warm adapter's pseudo-pure path. The property-diagram picker is
`plot_fluids_available`, which is *derived* from that list rather than being a
second list, so it followed automatically; `crates/frees/src/lib.rs`
asserts the published picker is exactly `["Air", "R1234yf", "R134a", "Water"]`.
The one place that still narrows `Air` out is `TableBackend::served_fluids`
(`props/propfun.rs`), and that is **correct and must stay**: the `(P,h)` table
build has a `(P,T)` transport grid for air and no state table, so a picker entry
there would fail at every plot point. It is not the backend this gate runs.
