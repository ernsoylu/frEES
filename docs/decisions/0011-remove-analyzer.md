# D11 — Remove the Data Analyzer; measured data enters through Tables

**Date** 2026-08-24 · **Status** accepted, in implementation (Wave J)

## Context

The Data Analyzer is the last survivor of the measured-data line. Its history
is a steady retreat:

* **Phase 10** shipped it against `.mf4` recordings, with an engine-side
  measurement stack (`crates/frees-core/src/measurement/`, 3 251 lines:
  sampled series, envelope decimation, raster construction, calculated
  signals) and its wasm boundary (`crates/frees/src/measurement.rs`,
  1 184 lines, the `measurement_calc` export).
* **[D6](0006-remove-mdf4.md)** removed MDF4 outright, leaving the Analyzer
  **CSV-only** and `measurement_calc` alive but stateless.
* **Wave H** then gave measured data a second, better route: CSV → function
  table, callable in equations.

What remains is a six-instrument oscilloscope UI (~6 000 lines across 31
files, plus `uplot` and `papaparse`) whose plotting job the Plots feature
already does for everything the solver produces, and whose one irreplaceable
capability — getting measured numbers *into* a document — is now served more
directly by a function table.

## Decision

1. **Remove the Data Analyzer**: `web/src/analyzer/**` (all instruments,
   channel store, CSV worker, decimation, stats, compare, offsets, calc
   signals), its dock windows and Inspector panel, the `analyzers` state and
   its project-file persistence, and the `uplot` + `papaparse` dependencies.
2. **Relocate CSV import into the Tables workbook**: an "Import CSV…" action
   producing a **function table** through Wave H's existing
   `composeTables` path (x column, y column, name, the 5 000-row decimation
   guard). Measured data stays importable *and* becomes callable in
   equations — the capability the Analyzer's own CSV path only half
   provided.
3. **Remove the engine measurement stack too**: `measurement_calc`,
   `crates/frees/src/measurement.rs`, `crates/frees-core/src/
   measurement/`, and their `measurement_parity` / `measurement_robustness`
   suites. The Analyzer is their only consumer; with it gone they are
   unreachable code carrying wasm bytes. This completes what D6 began.

## What is lost, explicitly

The oscilloscope workflow: multi-channel strip charts with cursors, the six
instruments (compare, histogram, scatter, statistics, table, event list),
calculated signals (frees formulas evaluated over sampled series), per-signal
offsets/relocation, envelope decimation of large recordings, CSV export of
channel data, and importing a *solved table back into* the analyzer
(`tableImport.ts`, the oscilloscope-parity path). None has a Plots
equivalent; the judgement is that none is worth its cost now that measured
data reaches documents through Tables.

## Compatibility policy

Following D10's precedent exactly: an existing `.frees` project's
`analyzers` array is **parsed and re-serialized inert** — never destroyed —
and a one-time load notice says the feature was removed and the data is
preserved in the file. No migration attempts to convert analyzer sessions
into tables; the shapes are too different to guess at.

## Consequences

`web/` is a vendored tree, so this is a second permanent fork beside D10's:
`WASM-PORT.md`'s re-sync procedure gains the Analyzer to its re-delete list.
Engine-side this is the first removal of *ported parity code* — the
measurement module was a faithful Phase-10 port with its own oracle suites,
so the decision is recorded here rather than in a status doc, and the corpus
is untouched (no measurement fixture ever entered `fixtures/corpus`).

### Measured at the web purge step (2026-08-24)

Both builds on the same tree at the same commit base, so the deltas are the
purge's alone. The wasm module is unchanged by this branch — the engine-side
removal is measured on its own branch.

|  | before | after | delta |
|---|---|---|---|
| `dist` total | 9,718,871 B (9.27 MiB), 101 files | 9,529,490 B (9.09 MiB), 94 files | **−189,381 B (−1.95 %)**, −7 files |
| precache manifest | 104 entries (9,469.01 KiB) | 97 entries (9,284.44 KiB) | −7 entries, **−184.57 KiB (−1.95 %)** |
| `node_modules` | 638 MB | 635 MB | **−3 MB** (5 packages: `uplot`, `papaparse`, `@types/papaparse` + transitives; `package-lock.json` −43 lines) |
| vitest | 42 files / 448 tests | 32 files / 391 tests | −11 analyzer suites (−81 tests), +1 CSV-reader suite (+24 tests) |

The seven dist files that left, by name: `DataAnalyzerTab.js` (108.12 kB raw
/ 40.62 kB gzip), `SignalBrowser.js` (37.29 / 14.15), `csvImport.worker.js`
(27.29), `DataAnalyzerTab.css` (1.65 / 0.71), and the three small chunks that
existed only to serve them — `offsets.js` (0.70), `IconWand.js` (0.55),
`IconChevronLeft.js` (0.33). **174.4 kB of the 189.4 kB delta is those
chunks**; the rest is the two deleted help topics leaving `docs-data`
(−4.9 kB), the App chunk shedding its analyzer wiring (−6.8 kB) and Mantine
tree-shaking the launcher path (−7.2 kB), less the +5.9 kB the relocated CSV
reader and Import dialog add to `TablesGridTab`.

The honest reading of that number: **the Analyzer was never a bundle
problem.** It was ~2 % of the app on the wire, and the case for removing it
was always its ~6 000 lines of maintained UI surface and the engine-side
measurement stack behind it, not its bytes. No dist file contains `uplot`
or `papaparse` after the purge.

**wasm module** (measured 2026-08-24 on `j-engine`, `wasm-pack build crates/frees --release --target web`, gzip -9): **3114.7 KiB raw / 1269.8 KiB gzipped → 3053.6 / 1245.9**, i.e. **−61.1 KiB raw (−1.96 %) and −23.9 KiB gzipped (−1.88 %)**; headroom against the 4096 KiB budget is 1042.4 KiB.
