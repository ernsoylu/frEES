# WASM port notes for this vendored frontend

This tree is vendored from `../frEES/frontend` (rsync snapshot, 2026-07-30,
excluding `node_modules/`, `dist/`, `.vite/`). Solve and Check now run
**in-browser** through the WASM engine; the remaining `/api/*` endpoints are
stubbed in `src/api.ts` until their engine features port.

## The generated wasm package (`src/wasm/pkg/`)

`src/wasm/pkg/` is **generated output** (gitignored) and must exist before
`npm run dev`/`npm run build`. Rebuild it whenever `crates/` changes:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
wasm-pack build ../crates/frees --release --target web --out-dir ../web/src/wasm/pkg
```

(paths relative to `web/`; absolute paths work too). The Dockerfile guards
this: an image build fails fast when the pkg is absent instead of shipping a
UI with no engine.

## Files the WASM port has touched

- `src/api.ts` — **done**: `solve`/`check` call the engine worker via
  `src/wasm/engineClient.ts` and keep their exported signatures; every other
  endpoint is a stub (neutral boot values or a "not yet available in the
  browser engine" rejection). `runCompute`/`pollJob` are kept, unwired, for a
  future hybrid remote path (opt-in via `VITE_API_BASE`).
- `src/wasm/` — **done**: `engine.worker.ts` (module worker hosting the wasm
  engine, `{id, method, args}` → `{id, ok, result|error}`) and
  `engineClient.ts` (lazy singleton, id correlation, typed
  `wasmSolve`/`wasmCheck`/`wasmVersion`).
- `src/defaultExample.ts` — **done**: boots a document the browser engine
  solves; the EV COMPONENT model is kept as `EV_THERMAL_EXAMPLE_TEXT` for
  Phase 6.
- `src/analyzer/` — **gone (D11)**. Phase 10 ported `measurementApi.ts` onto
  the wasm boundary, D6 narrowed it to CSV-only, and D11 deleted the whole
  Data Analyzer. See the Analyzer purge section below.
- `vite.config.ts` — forked: `buildInfoPlugin`, the vendor `manualChunks`
  split, `rollup-plugin-visualizer`, and (Phase 11) `vite-plugin-pwa`. The
  `/api` dev proxy is gone — nothing in `src/` calls a live endpoint.

## Files Phase 11 added or forked (browser-native product layer)

- `src/projectStore.ts` + `src/ProjectLibraryModal.tsx` — the IndexedDB
  project library and durable autosave mirror (decision D4);
  `src/project.ts` gains `normalizeStoredProject` so both storage backends
  share one sanitize/migrate trust boundary. Wired into `src/App.tsx`,
  `src/WorkspaceChrome.tsx`, `src/MobileLayout.tsx`.
- `src/pwa.tsx` + `src/main.tsx` — service-worker registration with a
  prompt-style update flow (a background activation must not yank hashed
  chunks from under a live tab; see the comment in `vite.config.ts`).
- `index.html` — theme-color, SVG favicon, apple-touch-icon; the manifest
  link is injected by the PWA plugin.
- `public/icons/` — the app icon set (SVG sources + rasterized PNGs).
- `nginx.conf.template` + `Dockerfile` — static-only: the `/api` proxy
  blocks, rate limiting and real-ip machinery went with the server.

## The feature clip (decision D5, 2026-08-05)

The Min/Max, Curve Fit, PID Tuner, Monte Carlo and Parameter Estimation
modals and the PDF/EPS export options are **deliberately removed**, not
missing from the vendor sync: their engine features exist only as
`NOT_IN_BROWSER_ENGINE` stubs, and this build's UI promises only what its
engine does. Files deleted: the five `*Modal.tsx` components. Files forked
for the clip: `App.tsx`, `WorkspaceChrome.tsx`, `Workspace.tsx`,
`plots/exportPlot.ts`. The `api.ts` stubs and the `pidLoop`/`pidGainRewrite`
helpers stay as the wiring seam. When re-syncing from upstream, re-apply the
clip; when wiring a feature, restore its modal from git history and delete
this paragraph's entry for it.

## The Tables grid + the spreadsheet purge (decision D10, Wave H)

The Tables workbook is a **native glide-data-grid implementation**:
`src/tablesGrid/TablesGridTab.tsx` renders TableSpecs directly and
`src/tablesGrid/tableGridModel.ts` carries the binding rules retargeted from
the old `src/spreadsheet/tableBinding.ts` (computed-cell visibility, the
5000-row cap, error-literal sanitization, paste-region clipping, read-only
surfacing of stored `spec.formulas`). `src/App.tsx` lazy-loads it in place
of the Univer `TablesWorkbookTab`, and the AlterValuesModal application
routes through the model's `applyColumnFill`. The dock window id stays
`table:univer-workbook` (persisted in saved layouts; also hardcoded in
`MobileLayout.tsx`).

**Phase 2 (the purge) removed the spreadsheet feature and Univer entirely.**
`src/spreadsheet/` is deleted; `tablesWorkbookBridge.ts` and `csv.ts`
(both Univer-free, needed by the grid) moved into `src/tablesGrid/`; the
`SpreadsheetSpec` type moved into `src/project.ts`, where the `spreadsheets`
array of a `.frees` file is still parsed and re-serialized — **inert
retention**: App carries a loaded project's spreadsheet data through to save
without ever showing or destroying it, and shows a one-time notice when the
array is non-empty. `@univerjs/preset-sheets-core`, `@univerjs/presets`,
`rxjs` and the `lodash-es` override left `package.json` in the same commit
as `vite.config.ts` lost the two Univer strip plugins and the hyphenation
glob machinery (that config `readdirSync`s the package at eval time — the
D10 trap), and `maximumFileSizeToCacheInBytes` dropped 8 → 4 MiB.

Additional forks the purge made (all UI-side removals of the dead feature):
`src/StatesTab.tsx` and `src/TablesTab.tsx` (the "Open in Spreadsheet"
snapshot actions), `src/Workspace.tsx` ("Export to Spreadsheet"),
`src/WorkspaceChrome.tsx` (the rail entry + launcher), `src/PlotTab.tsx`,
`src/plots/PlotCard.tsx`, `src/plots/PlotConfigModal.tsx` (the
`spreadsheet:id!Range` plot data source), `src/helpReference.ts` and
`src/functionCatalog.ts` (the spreadsheet cell-reference function rows —
documents calling it now fail loudly at parse), and `src/project.ts`.

**Future vendor rsyncs must: re-delete `src/spreadsheet/`, skip the removed
dependencies (`@univerjs/*`, `rxjs`, the `lodash-es` override), keep
`src/tablesGrid/` with the `App.tsx` Tables routing, and re-apply the
forks listed above.** Measurements (−5.19 MiB dist, −265 MB node_modules)
are in docs/decisions/0010-remove-spreadsheet.md's Consequences.

## The Analyzer purge (decision D11, Wave J)

**`src/analyzer/` is deleted in full** — 37 files (six instruments, the
channel store, the papaparse CSV worker, decimation/stats/compare/offsets,
calculated signals, the `.frees` table-import path and eleven vitest suites).
With it went its dock windows, the Inspector `SignalBrowser` panel, the left
rail's Analyzer launcher, the `analyzers` React state and the `uplot` +
`papaparse` (+ `@types/papaparse`) dependencies. The engine side —
`measurement_calc`, `crates/frees/src/measurement.rs`,
`crates/frees-core/src/measurement/` — goes with it in the same wave.

**The one capability that moved rather than died is CSV import.** It now
produces a *function table*, which is strictly more than the Analyzer offered
because the result is callable from the equations:
`src/tablesGrid/ImportCsvModal.tsx` (Import CSV… in the Tables workbook's `+`
menu, and the `import-csv` Spotlight action) reads a file through
`src/tablesGrid/csv.ts::parseCsvTable` — a ~200-line reader written for this,
**not** papaparse — and converts it with the same
`composeTables.functionSpecFromXY` Wave H's `ChannelFunctionModal` used, so
the 5 000-row decimation guard, the duplicate-x rule, the skipped-pair count
and the `FunctionNameHints`/`FunctionPrecedenceNote` collision notices are
unchanged behaviour. `tablesWorkbookBridge.ts` gained
`requestTablesCsvImport()` so the Spotlight action survives the workbook's
lazy chunk load.

**Inert retention, exactly as D10 did for spreadsheets**: the `analyzers`
array of a `.frees` file is still parsed and re-serialized untouched, the
`AnalyzerSpec` type moved into `src/project.ts` as an opaque
`{id, name, [key: string]: unknown}`, App carries it in a ref through to
save, and `analyzerNotice()` shows a one-time "the data is preserved in the
file but not shown" line through the same `loadNotice` mechanism.
`src/project.test.ts`'s analyzer group is the contract, now asserting deep
equality of a full session rather than spot-checking fields.

Other forks the purge made: `src/App.tsx` (lazy imports, state, per-instance
panels, dock kind, Spotlight entries, Inspector panel, boot notice),
`src/WorkspaceChrome.tsx` (rail entry + launcher),
`src/workspace/WorkspaceDock.tsx` (the `analyzer` tab icon),
`src/ParameterFitModal.tsx` (its measured source is a 1-D function table now,
not an analyzer channel), `src/HelpPage.tsx` and `src/docs/data_analyzer.md`
(deleted — the `analyzer` and `calc-signals` help topics went with the
feature; `src/docs/programming_logic.md` gained the "Tables you don't type"
section in their place). `docsCatalog.ts` / `docsTopics.ts` /
`referenceCatalog.ts` are **generated** by `npm run compile-docs` — edit the
markdown, never those.

**Future vendor rsyncs must: re-delete `src/analyzer/`, skip `uplot`,
`papaparse` and `@types/papaparse`, keep `src/tablesGrid/csv.ts` +
`ImportCsvModal.tsx` with the `+`-menu and Spotlight routing, re-delete
`src/docs/data_analyzer.md` (and recompile the docs), and re-apply the forks
listed above.** Measurements are in
docs/decisions/0011-remove-analyzer.md's Consequences.

## Everything else stays in sync with upstream

Do not fork other files. When upstream `../frEES/frontend` changes, re-run the
vendor rsync (same excludes) and re-apply only the files listed above; any
other local edit here is a bug.

## Node version

`web/.nvmrc` pins Node 22 (`nvm use` in this directory). Node 20.x builds the
app fine but **cannot run the vitest suite**: jsdom@30 pulls undici@8, whose
`CacheStorage` constructor calls `webidl.util.markAsUncloneable` — added in
Node 22 — so every test file fails at environment setup with
`TypeError: webidl.util.markAsUncloneable is not a function`. CI reads the
`.nvmrc` via `setup-node`'s `node-version-file` and runs `npx vitest run`
before the build. Verified locally: 39 files / 388 tests pass on v26.5.0
(Phase 11; the Phase 10 run was 38/369 on v22.23.2).
