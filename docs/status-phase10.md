# Phase 10 — measured data, and the upload that stopped happening

> **This document is history: the surface it describes no longer exists
> (2026-08-24).** [D6](decisions/0006-remove-mdf4.md) removed the `.mf4`
> reader; [D11](decisions/0011-remove-analyzer.md) removed the Data Analyzer
> and with it the whole engine measurement stack —
> `crates/frees-core/src/measurement/` (3,251 lines),
> `crates/frees/src/measurement.rs` (1,184 lines), the `measurement_calc`
> export, and the `measurement_parity` / `measurement_robustness` suites.
> Measured data now enters a document as a CSV-imported **function table**
> (Wave H), callable in equations. Nothing below describes the current engine.
> The body is left exactly as written on 2026-08-01, because the gap list and
> the fifteen-defect inventory are the record of what was removed and why it
> was affordable to remove. Ledger items 26–30 in
> [`status-phase1.md`](status-phase1.md) carry the same annotation; item 38
> records the removal.

**Read this after [`status-phase9.md`](status-phase9.md).** Phase 10 ports
`backend/core/src/main/java/com/frees/backend/measurement/` (11 files, ~1.1k
LOC) plus the two web controllers over it, and wires the result to the Data
Analyzer that was already in the shipped frontend. `.mf4` reading, resampling,
envelope decimation, raster construction and calculated signals all run in the
tab.

The headline is not a feature. It is a **deletion**: the Java tier uploaded a
measurement recording to `/api/measurements`, indexed it on disk under a TTL,
and read it back as windowed envelopes — and the *only* reason it did that was
that the parser lived on the server. With the parser in wasm the upload buys
nothing and costs everything, because measurement recordings (vehicle logs, rig
runs, customer acceptance traces) are confidential in a way an equation document
is not.

> **A `.mf4` opened in frees now never leaves the machine.** It is read by
> `FileReader` and handed straight to wasm. Nothing is transmitted, nothing is
> written to a server disk, and there is no TTL sweep to prove it was forgotten.
> This is the most user-visible consequence of the entire port.

The price of that is stated in full [below](#what-phase-10-did-not-deliver--ranked):
the Java's `Mf4Parser` → `FallbackMeasurementParser` → Python **`mdf-sidecar`**
ladder collapses to one rung, and **the sidecar has no successor.** Deflate,
ZSTD and LZ4 recordings that the sidecar read are now refused.

---

## Gate numbers, all raw

Every number below was produced by running the command through the absolute
binary and redirecting **both** streams to a file (`rtk` swallows clippy
warnings and truncates output; `cargo fmt --check` writes its diff to *stdout*,
so `2>` alone looks falsely clean).

| Gate | Result |
|---|---|
| `cargo test --release --workspace` | **3155 passed, 0 failed, 6 ignored**, exit 0 (Phase 9: 2933/0/4) |
| `cargo test -p frees-core --test parity` | `golden_corpus_parity` **ok** — all **531** fixtures match the Java oracle, 17.9 s |
| `cargo test -p frees-core --test measurement_parity` | **16 passed**, 0.03 s |
| `cargo test -p frees-core --test measurement_robustness` | **31 passed**, 0.89 s |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no lints |
| `cargo clippy --workspace --target wasm32-unknown-unknown --all-targets -- -D warnings` | exit 0, no lints |
| `cargo fmt --all --check` | exit 0, **zero bytes** on stdout and stderr |
| `./node_modules/.bin/vitest run` (Node 22.23.2) | **38 files, 369 passed**, exit 0 (Phase 9: 37/352) |
| `npm run build` | exit 0 |
| `wasm-pack build --release` | **2944 KiB raw / 1390 KiB gzipped — 95.8 % of the 3072 KiB budget. GREEN.** See [Bundle](#bundle-the-gate-is-green-and-the-phase-9-doc-is-stale). |

The only warning anywhere in any Rust build is
`the following packages contain code that will be rejected by a future version
of Rust: nom v1.2.4`, which is [accepted debt](#the-nom-124-debt-recorded-not-hidden).

Test totals by suite, since the workspace number is now large enough to hide
things: `frees-core` lib **2444** (of which `measurement::` is **121** —
`mdf4` 36, `raster` 25, `decimate` 21, `calc` 19, `series` 18, `window` 2),
`frees` lib **113** (of which `measurement::` is **56**),
`measurement_parity` 16, `measurement_robustness` 31.

**The delta reconciles exactly**, which is worth showing rather than asserting:
2933 (Phase 9) − **2** reclassified as `#[ignore]` + **224** new measurement
tests = **3155**. The two reclassified are `cas_control_robustness`'s slow
characterisation tests (`apart_at_a_fifty_fold_repeated_pole_answers` ~20 s,
`two_hundred_block_chains_stay_bounded` ~96 s), which is also where the ignored
count's 4 → 6 comes from. Nothing regressed; nothing else moved.

> **One caveat, stated because it would otherwise be assumed.** That identity
> closing is evidence, not proof — this phase was written by six agents working
> concurrently in one tree, and an addition and a deletion elsewhere would cancel
> invisibly. Every number above was run by me, raw, against the tree as it stood
> at the end; `fmt` and both `clippy` invocations were re-run after the last
> source edit, not before it.

---

## What shipped, by area

### 1. The contract (`measurement/mod.rs`, 183 lines)

`MeasurementError`, `ChannelKind`, `ChannelInfo`, `GroupInfo`,
`MeasurementMetadata`, `ChannelData`, `MeasurementSource` and `Result<T>` — the
fixed interface between the submodules and the wasm boundary, mirroring the Java
records so the existing frontend DTOs parse unchanged. Two rules are stated
there and enforced everywhere below: **time is always seconds, values are always
`f64`, and a gap is `NaN` — never an absent sample**; and **interpolation must
not bridge a `NaN`.**

`MeasurementError::code()` is what the frontend switches on:
`MEASUREMENT_PARSE_FAILED`, `CHANNEL_NOT_FOUND`, `FORMULA_ERROR`,
`RASTER_CAP_EXCEEDED`.

### 2. `mdf4.rs` (2024 lines) — ASAM MDF4 reading, from bytes

Port of `Mf4Parser.java` (254 LOC, over mdf4j). mdf4j memory-maps and streams
records lazily; there is no file to map in a browser, so the backing crate is
**`mf4-rs` 3.6** (MIT) and the recording is the `Vec<u8>` the caller already
holds. `Mdf4Source` owns those bytes for its whole life.

`mf4-rs` was chosen for one reason no other Rust MDF4 crate meets: it `#[cfg]`s
its memmap backing store down to a plain `Vec<u8>` on `wasm32-unknown-unknown`,
so `MDF::from_bytes` is a real entry point rather than one that traps on a
missing syscall. It is pulled with `default-features = false`, which keeps its
optional `wasm` feature (a wasm-bindgen layer) **off** — `frees-core` must never
depend on wasm-bindgen.

Format coverage against the Java baseline is [the gaps
list](#what-phase-10-did-not-deliver--ranked)'s first entry. Everything
structural is checked at `open`, before any sample is decoded, so after `open`
returns `Ok` the metadata call cannot fail for structural reasons — and a
compressed file is refused **at load, naming the remedy**, rather than listing
its channels happily and then failing on every single read.

`MdfError`'s own `Display` interpolates `file!()`/`line!()`, which would print
the build machine's cargo-registry path into a browser error message; every
variant is spelled out by hand instead, in an exhaustive match, so a future
`mf4-rs` variant stops the build rather than leaking a path.

### 3. `series.rs` (327) + `decimate.rs` (440) — the two hot primitives

Ports of `SampledSeries.java` and `EnvelopeDecimator.java`, branch for branch:

* `SampledSeries::at` reproduces the Java exactly, including the branch a
  careful reading gets wrong — a query *before* the first sample is `NaN`, not
  the first value (`i = lb - 1; if (i < 0) return NaN`) — and including the
  `if (v0.isNaN() || v1.isNaN()) return NaN` gate that is the load-bearing
  "gaps stay gaps".
* `lower_bound` is written as the Java's explicit `lo + (hi - lo) / 2` loop
  rather than `partition_point`, because the predicate is **not** partitioned
  when the time master contains a `NaN` — where `partition_point` is
  unspecified and this loop is deterministic and matches both the Java and the
  frontend's `decimate.ts`.
* `min_max` computes bucket edges as `(b as u64 * n as u64 / m as u64)`. The
  `u64` is not decoration: `usize` is **32-bit on wasm32**, and `b * n` wraps
  past ~65 k samples, which in release would silently scramble bucket
  boundaries. A regression test asserts the `u32` product differs from `edge`.
  An all-`NaN` bucket reports `NaN`/`NaN`, never `±∞`, and the bucket's
  representative time is `t[start + (end - start) / 2]` — the **midpoint
  index**, not the mean time.

### 4. `raster.rs` (895) — where the output grid comes from

Port of `MergedRaster.java`. `union` (merge every input's timestamps),
`fixed` (a constant `dt`) and `suggest_dt` (the 1-2-5 ladder the frontend
renders on its "use this dt instead" button). `union` is a streaming k-way merge
rather than the Java's concatenate-sort-dedupe, and a randomised test
(`union_matches_the_concatenating_reference_on_random_bases`, 300 trials × 3
caps, compared **bitwise** so `NaN` and `-0.0`/`0.0` cannot pass vacuously)
pins the two to the same answer.

`pow10` is a literal 61-entry decade table rather than `libm::pow`, for the
reason in [the sweep](#defect-3--suggest_dt-offered-half-the-resolution-the-java-does).

### 5. `calc.rs` (1497) + `window.rs` (140) — calculated signals

Port of `TimeSeriesEvaluator.java` (374) and `ChannelWindowDto.java` (62).

The formula language is **the frees expression language**, not a bespoke calc
dialect, and that is the point: `enthalpy(R134a, T=t_evap, P=p_rail)` over a
measured channel is a real property call through the same `props` backend a
document uses — something conventional measurement tools, whose calc engines are
C-like arithmetic, cannot do. `parse_formula` enters at
`parser::parse_bool_expr`, so `speed > 25 AND gear = 3` parses and Event-List
boolean channels survive.

A raster is routinely 10⁵–10⁶ points, so the formula is **compiled once** into a
tree whose variable read is an array index into a reused slot buffer. The
compiled tree is an enum rather than `Box<dyn Fn>`: the Java closes over one
mutable `HashMap` (which under `Fn` would need `Rc<RefCell<…>>`) and throws for
errors (which a closure returning `f64` cannot do without boxing a
`FreesError` into every leaf). An enum takes both as ordinary parameters and
still has no per-point allocation and no per-point map lookup.

`delta`, `integral`, `movavg` and `delay` are functions of a *series*, not of a
value; a pre-pass replaces each with a synthetic input computed once over the
whole raster, so per-point evaluation stays pure and order-free.

**Arithmetic here is IEEE, deliberately unlike the document evaluator.** `1/0`
in a document is an error, because a residual that silently became `inf` would
poison a Newton block; in a calculated signal it is `inf`, as in the Java, whose
compiled `/` is a bare `l / r`. Measured data has zeros in it — a stopped
engine, a closed valve — and a 500 000-point channel must not fail wholesale
because one sample divided by zero. Inside a function-call argument the document
semantics apply again, because the whole subtree goes to `eval::eval`; the Java
splits the same way for the same reason.

### 6. `crates/frees/src/measurement.rs` (2735) — the boundary

Four exports, replacing four REST routes:

```rust
#[wasm_bindgen] pub fn measurement_open(bytes: Vec<u8>, name: &str) -> String
#[wasm_bindgen] pub fn measurement_channel_window(request_json: &str) -> String
#[wasm_bindgen] pub fn measurement_calc(request_json: &str) -> String
#[wasm_bindgen] pub fn measurement_close(measurement_id: &str)
```

`MeasurementStore.getWindow`, `sniffKind` and
`MeasurementCalcController.buildRaster` are transcribed statement for statement,
including `i0 = max(0, lowerBound(from) - 1)`, `count <= maxPoints`,
`buckets = max(1, maxPoints/2)` and the NaN-rejecting `!(dt > 0)` guard. The
caps are the Java's own, read out of `../frEES`: `MAX_RASTER = 1_000_000`,
`MAX_RASTER_WITH_CALLS = 100_000`, `2400` default points clamped to
`[2, 20_000]`.

Three of the Java store's four jobs are deleted outright — the upload, the temp
directory with its owner-only permissions and TTL sweep, and the
`202 Accepted` + `jobId` + poll path for call-bearing calc requests (which
existed to move a slow evaluation onto the compute tier; we are already *on* the
worker thread).

`bytes: Vec<u8>` rather than `&[u8]` was verified rather than assumed: both were
built and the emitted `frees.js` is byte-for-byte identical
(`passArray8ToWasm0`, no `__wbindgen_free` of the buffer — ownership transfers),
and the `&[u8]` module is **133 bytes larger**, which is the `.to_vec()`. The
move survives down to `mf4-rs`'s wasm32 arm, which stores the `Vec` as its
backing store: **exactly one copy of the recording in linear memory.**

The registry is a `thread_local! RefCell<HashMap>` with a monotonic id counter
doubling as the LRU clock (no `getrandom` in an over-budget bundle, no
`Date.now()` millisecond collisions), bounded by `RETAINED_BYTES_BUDGET = 512 MiB`
and `MAX_OPEN_FILES = 256`, with eviction naming the dropped ids in `evicted`.
Every `f64` on the wire goes through `finite_or_null`.

### 7. `web/src/analyzer/measurementApi.ts` — the seam, and six consumers untouched

Same exports, same types, same `MeasurementApiError` with its `status` and
`payload`; the `/api/` strings that remain are comments recording which route
each call replaces. `calc.ts`, `channelStore.ts`, `SignalBrowser.tsx`,
`CalcSignalModal.tsx`, `DataAnalyzerTab.tsx` and `calc.test.ts` compile
unchanged. HTTP status codes are synthesised from `MeasurementError::code()`
(`…PARSE_FAILED` → 400, `CHANNEL_NOT_FOUND` → 404, `FORMULA_ERROR` and
`RASTER_CAP_EXCEEDED` → 422) and the nested error body is flattened so
`calc.ts::parseOverCap` still reads `payload.suggestedDt` off the top level.

`EngineRequest` gained one non-string field, `bytes?: Uint8Array`, and
`engineClient.call` puts `bytes.buffer` in `postMessage`'s **transfer list** —
narrowed with `instanceof ArrayBuffer`, because the DOM lib types `.buffer` as
possibly `SharedArrayBuffer`, which is not transferable. A test asserts
`post.transfer === [buffer]` against the real client with a fake `Worker`; a
mocked client cannot distinguish transfer from clone.

---

## What the adversarial sweep found

Three sweeps ran over this surface — an MDF4 malformed-input fuzz, a bounds
audit, and a numeric-parity comparison against a live JDK build of the reference
— and between them they found **fifteen confirmed defects**. That is a far higher
yield than Phase 9's one, and the reason is structural rather than a comment on
the code: a measurement file is **the most hostile input in the product** —
arbitrary bytes from an arbitrary tool, parsed as a graph of self-describing
blocks whose every length and link address comes out of the file itself — and the
wasm release profile is `panic = "abort"`, so a panic is not a diagnostic the
shim can render. It is the tab, and with it the recording the user was promised
would never leave it.

The regressions live in `crates/frees-core/tests/measurement_robustness.rs`
(31 tests) and `crates/frees-core/tests/measurement_parity.rs` (16). The count is
listed here so it is checkable rather than asserted:

| # | Defect | Class | Guard |
|---|---|---|---|
| 1 | a corrupt `##CC` link aborts inside `mf4-rs` | **abort** | `mdf4::validate_block_graph` |
| 2 | an implausible `cn_cycle_count` reaches `Vec::with_capacity` and aborts | **abort** | `mdf4::probe_records` |
| 3 | `MAX_RECORDS` tested `min(declared, physical)`; the *declared* count is what allocates | **abort** (8 GiB) | test the declared count |
| 4 | `MAX_BLOCKS` bounded visits, not cost — the visited list was scanned linearly | wedge (~11 min) | `mdf4::visit` on a `HashSet` |
| 5 | nothing bounded the per-group sample-storage walk; the cost is groups × `##DL` | unbounded | `mdf4::check_storage_chain` |
| 6 | `calc::evaluate`'s working set is `raster × inputs`; both factors capped, the product not | **abort** (1 044 MB) | `calc::MAX_INPUT_COLUMN_SAMPLES` |
| 7 | same product from the formula: one column per time-operator *occurrence* | **abort** (1 616 MB) | `calc::MAX_SYNTHETIC_SAMPLES` |
| 8 | a formula's depth was bounded and its **node count** was not | wedge (51 s) + 781 MB | `calc::MAX_FORMULA_NODES`, and the `calls × slots` compile quadratic fixed outright |
| 9 | a `NaN` span answered an **empty raster** instead of refusing | **silent wrong answer** | `raster::fixed` |
| 10 | `movavg` poisoned the rest of the channel after one `±∞`; **the first fix did not close it** | **silent wrong answer** | `calc::movavg`, gated on the window's `±∞` population |
| 11 | `suggest_dt` skipped a rung of the 1-2-5 ladder — half the resolution the Java offers | parity | `raster::pow10`, a literal 61-entry table |
| 12 | `and`/`or` did not short-circuit; Java's compiled forms are `&&`/`\|\|` | parity | `calc::Compiled::Logical` |
| 13 | `^` was C's `pow`: `pow(1, NaN)` = `1`, Java's `Math.pow` = `NaN` | parity, invents a sample | `calc::java_pow` |
| 14 | the same rule, **second site** — a call argument goes to the document evaluator | parity, invents a sample | `eval.rs::apply_binop` (engine-wide) |
| 15 | `fixed`'s `+ 1` was applied in `f64`, a no-op past 2⁵³ | parity | `raster::fixed` |

### Defects 1–7: the ones that end the session — five aborts and two unbounded walks

#### Defects 1 and 2 — a corrupt `##CC` link, and an implausible `cn_cycle_count`, aborted inside `mf4-rs`

Found by fuzzing mutations of a real recording. Both are closed *ahead* of
`mf4-rs`, in `validate_block_graph` (which walks and bounds the block graph
before any parse) and the `cn_cycle_count` plausibility test in `probe_records`.
Neither could have been caught after the fact.

#### Defects 3, 4 and 5 — three bounds that existed and were **stated in the wrong unit**

The shape that recurs everywhere in this phase: a ceiling that a file can
satisfy while still costing unbounded time or memory. One of the three is an
allocation abort; the other two are wedges, which under `panic = "abort"` is the
only difference that matters between them.

| Bound | What it actually bounded | Measured damage | Guard |
|---|---|---|---|
| `MAX_RECORDS` | tested against `min(declared, physical)`, but `mf4-rs` passes the **declared** `cn_cycle_count` to `Vec::with_capacity` | a **4 195 160-byte file holding 80 samples** produced two vectors of 4 195 160 capacity; at the boundary's 512 MiB file limit the same shape asks for **8 GiB** against a 4 GiB address space | test against the declared count |
| `MAX_BLOCKS` | how many blocks the pre-flight *visits* — but the visited list was scanned linearly, so the **cost** was that ceiling squared | 10 000 blocks 0.11 s, 40 000 1.0 s, 160 000 **17.6 s**; about **eleven minutes** at the ceiling, worker wedged, nothing able to cancel | `visit` on a `HashSet` |
| *(nothing)* | the sample-storage walk, which `mf4-rs` repeats once per channel group; groups and `##DL` blocks are both cheap and the cost is their **product** | unbounded | `check_storage_chain` charges a shared budget |

#### Defects 6 and 7 — `calc::evaluate`'s working set was `raster × inputs`, and **both factors were capped while their product was not**

The boundary's `MAX_INPUTS` counts inputs (128) and its `MAX_INPUT_SAMPLES`
counts *source* samples, so 128 one-point inline series satisfies both and still
asks for 128 full-length columns. Measured with a counting global allocator
through `measurement_calc`: a **5 604-byte request body peaked at 1 044 MB** — a
186 000× amplification, at a `collect()`, which under `panic = "abort"` is not a
diagnostic. The same product reached from the formula instead of the input list:
200 `delta(x)` terms over a million points peaked at **1 616 MB** from one input
and 1.6 kB of formula text.

Closed by `calc::MAX_INPUT_COLUMN_SAMPLES` and `calc::MAX_SYNTHETIC_SAMPLES`,
both 16 777 216 samples = 128 MiB, deliberately the same number as
`mdf4::MAX_RECORDS` so there is one ceiling to reason about rather than three.

### Defects 9 and 10: the two **silent wrong answers**

#### Defect 9 — a `NaN` span answered an **empty raster** instead of refusing

`inf - inf` is `NaN`, which slips past `t1 >= t0`, and `(NaN + 1.0) as u64`
saturates to zero. `raster::fixed` returned an empty grid, so the whole
calculated signal came back as a *successful, empty column*.
(`a_span_between_two_infinities_is_refused_rather_than_answered_empty`.)

#### Defect 10 — `movavg` poisoned the rest of the channel, and the first fix did not close it

`movavg`'s running sum is a one-way door: one `±∞` sample — or two large finite
ones — poisoned the accumulator, and **every later point came back `NaN`, a gap
over data that was fine.** This is genuine Java behaviour
(`TimeSeriesEvaluator.movavg` is bit-identical to the pre-fix Rust), so
diverging is a judgement call, made because a fabricated gap in a measurement
tool is worse than a parity difference.

**The interesting part is that the first repair did not work and was caught by
verifying it rather than trusting it.** The initial fix recomputed the window on
every non-finite accumulator, *including* the points where the offending sample
is still inside the window and the recompute is arithmetically guaranteed to
fail again. Those hopeless passes cost `Σ span ≈ W²/2`, so the repair budget was
exhausted before the one repair that mattered. Its regression test used a
10-point raster with two samples in the window; scaled to anything realistic it
did nothing:

| channel | window | tail points still fabricated as `NaN` |
|---|---|---|
| 200 000 pts @ 1 kHz, one `+∞` at sample 0 | 2 s | **195 998 of 195 998** |
| same | 10 s | 179 998 / 179 998 |
| 500 000 pts, `+∞` at sample 250 000 | 2 s | 245 998 / 245 998 |

All inside the boundary's own `MAX_RASTER = 1_000_000`, so all reachable from the
browser — and the motivating scenario ("on a 500 000-point channel it is the
whole rest of the recording") was exactly the case that stayed broken. The
shipped fix tracks the window's `±∞` **population** with the same
add-on-entry/subtract-on-exit bookkeeping as `count` and lets it decide the
answer outright while non-zero; the hopeless passes disappear, and what remains
is provably `O(n)` in total because two successful repairs are separated by a
whole window and their spans telescope.

### Defect 8: the one that wedged the worker on ordinary-looking input

#### Defect 8 — a formula's **depth** was bounded and its **node count** was not

`(A + A)` doubled *k* times is a tree of depth *k* with 2^k leaves, so the depth
budget is no constraint at all on width — and every cost in `calc.rs` is
`nodes × something`, so a shallow enormous formula was unbounded in three
directions at once. Measured, in release: **51 s** to evaluate a 24 kB call-free
formula over a million-point raster (about fourteen minutes for a megabyte);
**6.2 s** to *compile* a 90 kB formula over a **four-point** raster, which is
below every byte-counting cap there could be because it is not a memory problem;
and **781 MB** of synthetic columns from 14 kB of formula.

The 6.2 s compile was fixed outright rather than bounded — every `Expr::Call`
node was building and sorting its own copy of the whole slot table, so the cost
was `calls × slots` and both factors grow with the formula; `evaluate` now builds
that binding table once. `MAX_FORMULA_NODES = 1024` is what makes the other two
unreachable.

### Defects 11–15: the **numeric-parity** divergences, found against a live JDK oracle

`measurement_parity.rs` compared about **21 000 values** across four surfaces
against the reference classes compiled out of
`../frEES/backend/core/build/classes/java/main` and driven by a probe program on
this machine — 4 896 `suggestDt` calls covering every decade in `[-25, 25]` at
every rung of the 1-2-5 ladder and one ULP either side, 847 window slices,
14 319 per-sample results from 43 formulas over a channel pair carrying gaps,
infinities and zeros, and the full branch table of `at`, `lowerBound` and
`minMax`. **Four divergences survived scrutiny on that sweep; a fifth
(defect 15) came from the same comparison run against `fixed`. All five are
fixed.**

That the values were *run* rather than *read* earned its keep: four of the
expected tables contradict what a careful reading of the Java predicted, most
sharply the case where a `NaN` in the time master corrupts the binary search
rather than the arithmetic.

#### Defect 11 — `suggest_dt` offered half the resolution the Java does

`suggest_dt` built its 1-2-5 ladder on `libm::pow(10.0, k)`, which is **not**
correctly rounded — one ULP out at eight of the sixty-one decades in `[-30, 30]`
(`k = -29, -24, -21, -20, -17, -11, -5, 29`), where Java's `Math.pow` is exact at
every one. Two of those errors point *downwards*, and the ladder tests
`1 * decade >= raw`, so at `k = -5` and `k = -17` the rung that should have
matched failed and the answer jumped to the next. Measured:
`suggest_dt(0, 1e-4, 11)` returned **2 × 10⁻⁵ where the Java says 10⁻⁵**. That
number is rendered on the frontend's "use this dt instead" button, so the user
was being offered half the resolution they were entitled to, spelled unroundably.
Fixed with a literal 61-entry decade table (`raster::pow10`).

#### Defect 12 — `and`/`or` did not short-circuit

The Java's compiled `and`/`or` *are* Java's `&&`/`||`. This port evaluated both
operands — invisible until the right operand fails, and the entire reason to
write `p > 0 and enthalpy(…)` over measured data is that the property call is
undefined exactly where the guard is false. Measured:
`x > 5 and nosuchfn(x) > 0` returned `[0, 0, 0]` from the Java and failed the
whole channel here.

#### Defects 13 and 14 — `^` invented a sample nobody recorded, at **two** sites

Java's `^` is `Math.pow`, which is **not** C's `pow`: `pow(1, NaN)` and
`pow(±1, ±∞)` are `NaN` in Java and `1` in C. A `NaN` exponent is a dropout in
the exponent channel, so C's answer invents a value wherever the base sits at
exactly 1 — breaking this module's headline rule, not merely its parity.

Defect 14 was found while *verifying* the fix for defect 13, which had corrected
`^` only in the compiled calc tree. A function call is not compiled: the whole
subtree goes to the document evaluator, which had C's `pow` too, so `abs(b ^ e)`
re-invented the same `1.0` that had just been removed — and `abs(b ^ inf)` was
wrong at *every* sample rather than only at the gap. One rule, two sites; the
second fix is in `eval.rs::apply_binop` and is therefore **engine-wide**, which
is correct, because `ast/Evaluator` uses `Math.pow` exactly as
`TimeSeriesEvaluator` does.

#### Defect 15 — a point count past 2⁵³ was swallowed by its own `+ 1`

`raster::fixed` added its `+ 1` in `f64`, where it is a no-op past 2⁵³, so the
count quoted in a cap refusal was one short of the Java's.

### What held, having been attacked hard enough that saying so means something

`SampledSeries::at` on every branch in both interpolation modes; `lower_bound`
including `NaN` probes and runs of equal timestamps (plus a 400-point randomised
agreement check against a linear scan); `min_max`'s bucket edges and its
midpoint-index representative time at every bucket count from one to past the
sample count, and at **three million samples**; the trapezoid `integral`,
`delta`, the trailing-window `movavg` (including which side of `t - window` is
inclusive) and `delay` over regular, irregular, duplicate-timestamped and
`NaN`-mastered rasters; `union`'s sort/dedupe including signed zero and `NaN`;
`fixed`'s accumulation drift; truncated and lying block headers; ragged
channels; descending and stalled time masters; degenerate windows; and formulas
at the parser's depth ceiling.

### Three divergences measured and **left in place**, as a record

Inside a function-call argument the calc path defers to the document evaluator,
which *refuses* three things the Java's `ast/Evaluator` answers: division by zero
(`±∞`), a negative base raised to a non-integer power (`NaN`), and zero raised to
a negative power (`+∞`). Those guards are engine-wide and load-bearing for
Newton, so they are out of this module's reach. Note the shape they share with
the fixed entries and *not* with each other: a guard **fails loudly**, where C's
`pow` returned a plausible wrong number.

Also measured and deliberately not treated as a defect: `libm::pow` and the
JVM's `Math.pow` intrinsic disagree by up to one ULP on ordinary arguments. On
every disagreeing case checked, **this host's own `f64::powf` agrees with the JVM,
not with `libm`** — so `libm` is the least accurate of the three, and it is
chosen anyway because it is the only one that makes a native run and a wasm run
agree bit for bit, which is a crate-wide rule. Accuracy is what is being traded,
deliberately.

---

## Browser proof

Rebuilt `web/src/wasm/pkg`, built `web/dist`, served it with
`python3 tools/serve-dist.py web/dist 8911`, drove it with Playwright.

| Step | Result |
|---|---|
| Open `fixtures/measurement/a_small_uncompressed.mf4` (26 KB, genuine **asammdf** output — `##ID` says `amdf8.8.`) through *Analyzer → Import CSV/MF4* | **`1,000 samples · 3 channels`**, listed as `speed [m/s]`, `torque [Nm]`, `valve_open [-]` — the Java oracle's own `gate1MetadataEnumerates` assertions, in a browser |
| Add `speed` to a scope strip | trace renders: a ~20 ± 10 oscillation (the generator's `20 + 10·sin(2t)`) with the single-sample **99.5 spike at t = 5.00 s** preserved through the envelope decimation, which is exactly what the fixture exists to test |
| **Calculated signal** `c1 = movavg(x, 0.5) + integral(x)`, `x → speed`, merged raster | **computed**, `1,000 samples · 1 channels`, added to the signal browser and the strip |
| Open a `##DZ`-marked file (`a_small_uncompressed.mf4` with its `##DT` id overwritten) | refusal renders verbatim: *"Channel group 0 stores its samples in compressed (##DZ) data blocks. This reader has no decompressor — re-export the recording uncompressed."* |
| …and the worker survived it | yes — `torque` was added to the strip afterwards and the first file's signals were untouched |
| **`/api/` requests** | **ZERO.** 36 requests total, all static assets plus `frees_bg-BvITCa3t.wasm`. The single URL matching `/api/` is `/assets/api-Cj-Mc6sj.js`, the bundled fetch→RPC shim served as a file. **No POST of any kind**, so the 26 KB recording provably never went anywhere. Two console errors, both pre-existing benign 404s (`build-info.js`, `favicon.ico`) |

Screenshots in the session scratchpad as `/tmp/p10-proof/p10-analyzer.png` and
`/tmp/p10-proof/p10-browser-proof.png`.

---

## Fixtures

**Promoted: none, and none were expected.** The golden corpus is a *document*
corpus driven by `frees-cli`; nothing in Phase 10 is reachable from a `.frees`
document, because measurement is an API surface, not a language feature. The
corpus is unchanged at **531/531** and `fixtures/corpus-pending/` is unchanged at
**11**.

**One binary fixture was added**:
`fixtures/measurement/a_small_uncompressed.mf4`, copied from
`../frEES/backend/core/src/test/resources/measurement/` (md5 verified identical;
`../frEES` was not modified). It matters more than its size suggests — it is
**third-party bytes**, written by asammdf, and it is the same file the Java
oracle asserts against, so five tests transcribe `Mf4SpikeTest.gate1*`'s
assertions rather than re-deriving them. It sits behind `#[cfg(test)]`, verified
empirically: `grep amdf8` on the release `wasm32` rlib finds nothing, so it costs
the bundle zero.

> Whoever commits this phase **must include that path**. The core test build
> fails without it, and it is the only non-`.rs` file Phase 10 adds.

---

## Bundle: the gate is green, and the Phase 9 doc is stale

```
wasm-pack build crates/frees --release --target web
  →  3,014,829 bytes  =  2944 KiB raw  /  1390 KiB gzipped
     budget            =  3072 KiB raw
     95.8 % of budget, 128 KiB of headroom
```

**`WASM_BUDGET_KB` was not raised, because it did not need to be.** A raise was
authorised for this round; the gate is green, and `.github/workflows/ci.yml`
says in terms *"Do not raise this again without doing one of them first"*, so
raising it to make room that already exists would spend the only discipline that
file has. **`.github/workflows/ci.yml` is therefore untouched by this phase** —
including its comment block, whose newest recorded measurement is still Phase
6's 2184.5 KiB. See gap 9.

### Why it is green when `status-phase9.md` says 3336 KiB and "the one red gate"

**That document is stale relative to its own commit.** Phase 9 measured
`opt-level = "s"` as a 539 KiB saving, wrote it up as *"the measured lever, not
taken"*, and then **took it**: `[profile.release]` in commit `9740aa2` carries
`opt-level = "s"` + `lto = true` + `codegen-units = 1`. Re-measured here, to
confirm the mechanism rather than infer it:

| build | raw | gzipped |
|---|---|---|
| HEAD (no Phase 10) at `opt-level = 3` | **3304 KiB** | 1488 KiB |
| HEAD (no Phase 10) as committed, `opt-level = "s"` | **2769 KiB** | 1318 KiB |

535 KiB, which is Phase 9's own measured 539 KiB. The 3336 KiB in
`status-phase9.md` and in `CLAUDE.md`'s status paragraph is a pre-commit number
and both are corrected by this document.

### The Phase 10 delta, and the `mf4-rs` share of it, measured

Three builds, each from a clean tree with its own `CARGO_TARGET_DIR`. The middle
one is the shipping tree with `mdf4.rs` replaced by an API-compatible stub and
`mf4-rs` removed from both manifests, built in a scratch copy — the working tree
was not touched.

| build | bytes | raw | gzipped |
|---|---|---|---|
| HEAD, no Phase 10 | 2 835 456 | 2769 KiB | 1318 KiB |
| Phase 10 with `mdf4.rs` stubbed, no `mf4-rs` | 2 924 353 | 2855 KiB | 1352 KiB |
| **Phase 10, shipping** | **3 014 829** | **2944 KiB** | **1390 KiB** |

| | raw | gzipped |
|---|---|---|
| **Phase 10 total** | **+175.2 KiB** | +71.5 KiB |
| — of which **`mf4-rs` + `mdf4.rs`** | **+88.4 KiB** | +38.0 KiB |
| — the rest (raster, series, decimate, calc, window, the boundary) | +86.8 KiB | +33.5 KiB |

By wasm section: **code +158.8 KiB, data +15.7 KiB** (the data growth is the
diagnostic strings — every refusal in this phase names its remedy).

`wasm-opt -Oz` **is** being applied, verified rather than assumed: re-running
`-Oz --enable-bulk-memory --enable-nontrapping-float-to-int` on the shipped
artifact saves a further 2 537 bytes (0.08 %), which is a no-op.

### The debt, unchanged

Both entries in `.github/workflows/ci.yml` are still open and neither was paid:
the ~526 KB of property tables are still **linked** rather than fetched, and the
engine is still **one chunk**. Headroom is now 128 KiB — a fifth of the 622 KiB
Phase 7–8 had, and **less than Phase 10 itself cost (175 KiB)**. The next phase
of comparable size breaches the budget, and the two debts above are the answer,
not another profile flag; the profile lever has been spent.

One cheap, measured item is also outstanding and is *not* free: enabling
`serde_json`'s `float_roundtrip` feature costs **+19.2 KiB**. See gap 6.

---

## The `nom 1.2.4` debt, recorded not hidden

```
$ cargo tree -i nom@1.2.4
nom v1.2.4
└── meval v0.2.0
    └── mf4-rs v3.6.0
        └── frees-core v0.1.0
            ├── frees-cli v0.1.0
            └── frees v0.1.0
```

`cargo` flags `nom v1.2.4` as *"contains code that will be rejected by a future
version of Rust"*. The specific lint, from
`cargo report future-incompatibilities`, is **`trailing semicolon in macro used
in expression position`** at `nom-1.2.4/src/macros.rs:482`. It is the **only**
future-incompat warning in the entire build, native or wasm.

**Accepted, with the reason.** MDF4 is a large binary specification with a
self-describing block graph, versioned headers, several storage layouts and a
dozen conversion types; the alternative to a crate was hand-rolling a parser for
it, against fixtures we mostly cannot generate. `mf4-rs` is the only Rust MDF4
crate that genuinely supports `wasm32-unknown-unknown`. Taking it takes its tree.

**The exit is narrower than it looks**, and worth writing down because it is not
obvious from the tree above: **`mf4-rs`'s own parser is on `nom` 8.0.0**, which
is current and unflagged. `nom 1.2.4` arrives *only* through `meval`, and
`meval` is used at exactly one call site — `blocks/conversion/linear.rs`, which
evaluates the MCD-2 MC text formula of an **algebraic (type 3) conversion**.
That is a conversion class the Java baseline did not support at all. So:

1. **Fork `mf4-rs` and drop the algebraic-conversion path** — one file, one
   dependency edge, and it removes `meval` → `nom 1.2.4` outright while
   *increasing* parity with the Java (which read identity + linear only).
2. Or **vendor a narrowed reader**: this port uses a small slice of `mf4-rs`
   (`MDF::from_bytes`, channel-group and channel iteration, `ChannelBlock`
   fields, `ConversionType`, `RawChannel`), and vendoring that slice also sheds
   `memmap2` → `libc`, which is dead weight in a browser.

Neither was done here. Nothing is broken today; this is a supply-chain item on a
repo whose recent history is specifically about supply-chain scope, and it should
be a deliberate decision rather than a discovery.

---

## What Phase 10 did **not** deliver — ranked

1. **Compressed recordings are refused, and the `mdf-sidecar` has no successor.
   This is the phase's real cost.** The Java ran a three-rung ladder:
   `Mf4Parser` (mdf4j, in-process) → `FallbackMeasurementParser` → the Python
   **`mdf-sidecar`** (FastAPI + **asammdf**, `../frEES/mdf-sidecar/app.py`), whose
   own docstring says it exists for *"DZ-compressed data blocks
   (deflate/ZSTD/LZ4), **the norm for OEM recordings**"*. In the browser there is
   no second process, and nothing to fall back *for*. Measured against three
   baselines rather than two:

   | | Java rung 1 (mdf4j 0.2.0) | Java rung 3 (asammdf sidecar) | **here (`mf4-rs` 3.6)** |
   |---|---|---|---|
   | uncompressed `##DT`/`##DV`, `##DL` chains | yes | yes | **yes** |
   | **deflate `##DZ`** | **yes** | yes | **NO — refused at `open`** |
   | **4.30 ZSTD / LZ4 `##DZ`** | no | **yes** | **NO** |
   | **VLSD (string) storage** | no | **yes** | **NO** |
   | **unsorted / multi-group data groups** | no | **yes** | **NO** |
   | conversions | identity + linear | full | identity, linear, rational, algebraic, table lookups |
   | value-to-text channels | raw numbers | text | raw numbers |
   | MDF version range | **≤ 4.20** (javadoc) | 4.x | **≥ 4.10**, refused below with the version spelled out |

   So this reader **beats** mdf4j on conversions and **loses to it** on deflate,
   and it loses to the sidecar on everything the sidecar existed for. `mf4-rs`
   has **no decompressor in its dependency tree at all** — verified: its
   `Cargo.toml` contains no `flate`/`zstd`/`lz4`/`miniz`/`zlib` edge of any kind,
   and its reader accepts only `##DT`/`##DV`/`##DL`. Adding deflate means a zlib
   crate in a bundle with 128 KiB of headroom; ZSTD and LZ4 mean two more. That
   is a budget decision, not a coding one. Until then a compressed recording must
   be re-exported uncompressed, and the refusal says exactly that, at load,
   rather than letting the user discover it one channel at a time.

   **The honest summary for a user: an OEM recording will probably not open.**

2. **Only one of the six MDF4 fixtures is real, and the other five cannot be
   regenerated here.** `../frEES`'s `generate_mdf_fixtures.py` produces six
   fixtures — `a_small_uncompressed`, `b_zstd`, `c_lz4`, `d_vlsd`,
   `e_multigroup` (deflate + value-to-text), `f_large` (~100 MB) — **at test
   time, and commits only (a)**. So the positive-path evidence over genuine
   third-party bytes covers exactly one uncompressed 4.10 file: value-to-text,
   VLSD, linear conversions and multi-group are tested **only against synthetics
   written by `mf4-rs`'s own writer**, which is a real round trip but a
   self-consistent one — a shared misreading of the spec would pass every such
   test. And **the 4.30 ZSTD/LZ4 rejection is untested on genuine bytes**;
   re-generating them needs `pip install asammdf`, which was not done. This is
   the largest evidence gap in the phase.

3. **Lazy extraction is gone, architecturally, and the Java's Gate 3 has no
   equivalent.** mdf4j memory-mapped the file and streamed records; the Java's
   `Mf4SpikeTest.gate3Scale100Mb` asserts a **retained-heap delta below 60 MB
   after extracting one channel of a 100 MB file**, with the comment *"lazy
   extraction must not materialize the whole file on the heap"*. A browser has
   bytes, not a file, so `Mdf4Source` holds the whole recording *plus* the
   decoded `f64` columns. `MAX_RECORDS` (16.7 M samples, 256 MiB per channel)
   and `RETAINED_BYTES_BUDGET` (512 MiB across all open files, LRU-evicted) are
   the ceilings that replace it, and a 100 MB recording still fits — the failure
   mode has moved from "slow" to "refused". **No test in this repo opens a file
   anywhere near that size.** In fairness the Java's own Gate 3 is
   `assumeTrue`-gated on `f_large.mf4`, which is not committed either, so
   *neither* side runs that check in CI — but only one side needs it.

4. **There is no end-to-end test that reads a real `.mf4` through the wasm
   boundary.** `frees`'s 56 measurement tests drive a `FakeSource`;
   `Mdf4Source` is proven against genuine asammdf bytes in `frees-core`'s own
   tests. The two halves are each tested and never together — except in the
   browser proof above, which is a manual Playwright session, not a gate.

5. **Virtual master channels (`cn_type` 3) are refused, and it is unknown
   whether the Java accepted them.** `Mf4Parser` delegates to mdf4j's opaque
   `isTimeMaster()`, so the reference source does not reveal it. Refusing is
   deliberate — a silently wrong time axis is the worst possible failure in a
   measurement tool — but it is flagged here as a *possible* narrowing rather
   than a proven match.

6. **`serde_json` parses floats to within 1 ULP, and two tests assert that the
   defect exists.** The exact parser is behind the `float_roundtrip` feature,
   which this workspace does not enable. Numbers the boundary *emits* are exact
   (`ryu`); every number it *reads* can shift by one bit — `1.4000000000000001`,
   which is what `JSON.stringify` writes for `14 × 0.1`, comes back as `1.4`.
   Two rules in this phase are exact `f64` equality (`SampledSeries::at`'s
   exact-hit branch and `raster::union`'s dedupe), so an inline calc series and
   a series read from a file **can fail to merge on a timestamp that is the same
   number**. The cost of the fix is measured — **+19.2 KiB** against 128 KiB of
   headroom — and it was not applied because the workspace manifest belongs to no
   one agent and because landing it *inverts* the two tests, which were written
   to assert the defect deliberately.

7. **`ChannelKind::Boolean` is close to dead code on real files.** It needs
   `cn_flags` bit 3 or a 1-bit integer, and asammdf sets neither: the fixture's
   `valve_open` is a genuine 0/1 `uint8` and reports as `Analog`. That *matches*
   the Java (which had no boolean case at all), and a test now pins it — but the
   Events tab's boolean affordances will rarely light up from an MDF4 file.

8. **A stale claim survives in `crates/frees/src/measurement.rs`'s header**:
   it says *"`measurementApi.ts::calcSignal` still has the polling branch"*. That
   branch was deleted in this same phase. The file belongs to another agent and
   the line is documentation, not behaviour, but it should be corrected.

9. **`.github/workflows/ci.yml`'s newest recorded measurement is Phase 6's**
   (2184.5 KiB). Three phases have passed. The file's discipline — a raise must
   carry the number, the date and the reason — is good, and it is being eroded
   by *not writing measurements down* rather than by bad raises. This document
   carries Phase 10's; nobody carried Phases 7–9's into that file.

10. **No property-based or coverage-guided fuzzing, again.** Everything above is
    hand-chosen adversarial cases plus targeted mutation testing. Fifteen defects
    from three sweeps is a high yield, and the honest reading of a high yield is
    that the surface has more. There is still no `proptest` and no `cargo-fuzz`
    target anywhere in the tree, and this is the surface that most deserves one:
    the input is untrusted bytes with a documented structure, which is the
    textbook case for a structure-aware fuzzer.

11. **Everything Phase 8 did not deliver is still not delivered.** The optimizer,
    NSGA-II, curve fitter, Monte Carlo, parameter fit and all-roots solver remain
    unreachable from the boundary; the Min/Max, Curve Fit and PID Tuner buttons
    in the shipped UI still have no engine behind them. Phase 10 did not touch
    this.

---

## Divergences opened by this pass

Recorded in the ledger at
[`status-phase1.md`](status-phase1.md#opened-by-phase-10-2026-08-01) as items
26–30.
