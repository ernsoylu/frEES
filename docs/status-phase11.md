# Phase 11 — the browser-native product layer

**Read this after [`status-phase10.md`](status-phase10.md).** Phase 11 is the
first phase that ports no Java: the engine is untouched, the corpus is
untouched, and every change is in `web/` and the deployment story. What it
delivers is the product half of PLAN.md's thesis — the app **installs as a
PWA, works fully offline including reload, keeps a library of projects in the
browser with no server**, and deploys as dumb static files with no COOP/COEP.

Three of the four exit criteria in PLAN.md's Phase 11 row are met and proven
in a real browser below. The fourth ("project open/save without a server")
turned out to be **already largely shipped** by the vendored frontend — see
"What was already there", which this document is deliberately explicit about,
because claiming it as Phase 11 work would inflate the phase.

---

## Gate numbers, all raw

Run on this machine (macOS, the recovered checkout whose parity was
re-verified against the oracle corpus before any Phase 11 work began).

| Gate | Result |
|---|---|
| `cargo test --release --workspace` | **3155 passed, 0 failed, 6 ignored**, exit 0 — identical to Phase 10, as it must be: no Rust changed |
| `cargo test -p frees-core --test parity` | `golden_corpus_parity` **ok** — all **531** fixtures match the Java oracle (40.9 s on this machine) |
| `cargo clippy` (native + wasm32 targets) | exit 0, no lints |
| `cargo fmt --all --check` | exit 0, zero bytes of diff |
| `./node_modules/.bin/vitest run` (Node 26.5.0) | **39 files, 388 passed**, exit 0 (Phase 10: 38/369; +1 file / +19 tests is `projectStore.test.ts`) |
| `npm run build` | exit 0; `tsc -b` clean; PWA plugin emits `dist/sw.js` + `dist/manifest.webmanifest`, **334 precache entries / 30 421.88 KiB** |
| wasm bundle | **untouched**: 3 015 181 bytes = 2944 KiB raw (95.8 % of the 3072 KiB budget). Gzip on this machine says 1393 KiB where Phase 10's Linux gzip said 1390 — same bytes, different gzip build; the raw number is the gated one |
| eslint over the touched files | 0 errors; the only warnings are pre-existing (`App.tsx` `any`s, `MobileLayout` effects, `main.tsx` fast-refresh) |

New dependencies: **zero runtime**, two dev — `vite-plugin-pwa` 1.3.0 (build
integration + generated Workbox worker) and `fake-indexeddb` 6.2.5 (tests
only). The IndexedDB wrapper itself is hand-rolled (~40 lines of
promise-wrapping); an `idb`-style runtime dependency was not worth its bytes.

---

## What was already there, stated plainly

Two of the phase's named deliverables predate it, shipped silently by the
vendored frontend and its earlier phases:

* **Share links** (`web/src/share.ts` + tests): `#share=` fragments carrying
  the lz-string-compressed document, a 16 000-char ceiling, boot-time import
  with a confirm guard, wired into both menus and Spotlight. Phase 11 verified
  it (its tests run in the 39-file suite) and deliberately kept its scope —
  the fragment carries the **document text only**. A whole-project share would
  smash the URL ceiling on the first embedded whiteboard image; the `.frees`
  file is the whole-project transport.
* **`.frees` file open/save** (`web/src/project.ts`): the unified project
  JSON, `showSaveFilePicker` with a download fallback, a hidden file input for
  open, sanitize-on-write and migrate-on-read. Phase 11 built *on* this rather
  than beside it.

What did **not** exist: any way to keep more than one project, any durable
autosave, any installability, any offline story beyond "the HTTP cache might
have it", and a deploy config that wasn't dragging a dead backend proxy.

---

## What shipped, by area

### 1. The project library (`projectStore.ts`, 232 lines + `ProjectLibraryModal.tsx`, 205) — decision D4

IndexedDB, two object stores: `projects` (keyed by **display name**, so "Save
to browser" has file semantics — same name overwrites) and `autosave` (one
row). The API is fully async and degrades to "the library is absent" wherever
IndexedDB is unavailable (private modes, partitioned iframes) — the
localStorage path this sits beside is untouched, so degradation means
*today's* behaviour, not a broken app.

Reads re-validate: `project.ts` gained one export, `normalizeStoredProject`
(migrate-then-sanitize), used by every IndexedDB read, because browser storage
is outside the trust boundary no matter which API it hides behind
(tssecurity:S8475 — the same argument `sanitizeProject` already makes for
localStorage writes).

The modal (File → Browser Projects…, also in the mobile menu and Spotlight)
lists name / saved-at / size, opens through App's existing dirty-check guard,
renames by re-keying (refusing collisions — silently merging two projects is
the one outcome worse than a failed rename), and deletes behind a two-click
arm-then-confirm that disarms after 3 s.

### 2. The durable autosave mirror, and the quota story

The debounced autosave now writes **both** halves from one `buildProject()`
output: localStorage (what boot reads synchronously — unchanged) and the
IndexedDB mirror. Because both halves share one `savedAt`, a healthy pair can
never differ; the mirror being **strictly newer** at boot is therefore a
*diagnosis* — the localStorage write has been failing, which is exactly what
happens past the ~5 MB quota once whiteboard images or spreadsheets grow. The
app then offers a restore (yellow notification, explicit button), never forces
it, because the user may have wanted the state they are looking at.
`applyProject`, New Project and Load Example all keep the mirror in step so a
stale mirror cannot masquerade as newer.

**Deliberately not done: making boot async.** `App.tsx` reads the autosave
synchronously before every `useState` initializer; moving boot to IndexedDB
would force a suspense/loading-gate rewrite of a file the port otherwise does
not own, for zero happy-path gain. D4 records the reasoning; revisit if
projects outgrow structured-clone comfort (~tens of MB), which is also the
point at which OPFS per-project files become the right shape.

### 3. The PWA (`vite-plugin-pwa`, `src/pwa.tsx`, `public/icons/`)

* **Manifest**: generated by the plugin — name, `display: standalone`, dark
  `theme_color` (#1a1b1e, matching the app's default dark scheme where the old
  hand-written manifest said white), and a real icon set (192/512 PNG, a
  maskable 512 with the glyph inside the safe zone, and the SVG source). The
  old `public/manifest.json` — `icons: []`, never installable, and (it turns
  out) **never even copied into the Docker image**, which did not `COPY
  public/` — is deleted.
* **Service worker**: Workbox `generateSW`, precaching `dist` in full — 334
  entries, 30.4 MB, including the 2.9 MB wasm engine, Plotly (4.8 MB) and the
  spreadsheet stack (5.7 MB) via an 8 MiB per-file ceiling. The scope call:
  "full offline session" for an engineering tool means the whole tool —
  property diagrams and spreadsheets included — not a shell that apologises
  once you open a plot. The cost is honest: the first visit downloads
  everything; every subsequent session costs zero network.
* **Update flow**: `registerType: 'prompt'`. A new deploy installs in the
  background and *waits*; an in-app notification offers "Reload to update"
  (`updateSW(true)` = skipWaiting + reload in one motion). Auto-activation
  was rejected specifically because purging old hashed chunks under a running
  tab recreates the stale-chunk failure `main.tsx`'s `vite:preloadError`
  handler exists to paper over — the SW must not manufacture the emergency
  its host is already guarding against.
* **Exclusions that matter**: `build-info.js` (server-stamped, unhashed) is
  not precached and not navigation-fallback'd; `/api/` navigations are
  denylisted so a hybrid deploy's endpoints fail honestly rather than serving
  the app shell.

### 4. The static deploy (`nginx.conf.template`, `Dockerfile`, `vite.config.ts`)

The nginx template drops from 105 lines to 41: all three `/api` proxy
locations, the `limit_req` zone and the `real_ip` machinery are gone with the
backend they guarded (`docs/dependency-map.md` has listed them under deleted
infrastructure since Phase 3; Phase 11 is where they actually die). What
remains: immutable caching for `/assets/`, `no-cache` for everything unhashed
— which now **matters** rather than being hygiene, because `sw.js` must
revalidate on every load or a deploy can never reach users. The Dockerfile
loses the resolver/entrypoint machinery, gains `COPY public ./public` (see
above) and a fail-fast guard: an image built from a tree without the generated
wasm pkg now errors with the exact `wasm-pack` command instead of shipping a
UI with no engine. The vite `/api` dev proxy is gone (nothing calls it); the
`VITE_API_BASE` build args stay, because the optional remote adapter
(PLAN.md scope decision 2) remains a supported opt-in, not a casualty.

No COOP/COEP anywhere, re-verified — D3's constraint holds through the PWA.

---

## Browser proof (ran, not imagined)

`web/dist` served by `tools/serve-dist.py` on 127.0.0.1:8911 — a dumb
loopback static server, no headers, no proxy — driven by Playwright over the
installed system Chrome (`channel: 'chrome'`; the bundled Chromium does not
support macOS 13).

| Step | Result |
|---|---|
| First load | service worker registers at scope `/`, activates; precache populates **329 entries including `frees_bg-<hash>.wasm`** |
| Manifest | fetched from the injected link: name, `standalone`, `start_url "."`, theme `#1a1b1e`, icons `192 / 512 / 512-maskable / any(svg)` — the installability surface Chrome requires |
| Solve online | F2 → Solved (sanity baseline) |
| File → Browser Projects… → Save | IndexedDB `frees/projects` holds `{name: "untitled", size: 3781}` with the document text — written through the real UI, read back through raw IDB |
| **Network cut** (`context.setOffline(true)`), reload | **app shell renders** |
| Solve, offline, after reload | **Solved** — the wasm engine came out of the precache |
| Browser Projects, offline | lists the saved project (IndexedDB is local) |
| Requests during the offline phase | 25, all satisfied by the service worker; zero reached a network |

Screenshot: the offline session with the Solved pill, the populated Variable
Explorer and the library modal listing the saved project, captured in the
session scratchpad as `p11-offline-proof.png`.

The precache count difference (334 built vs 329 cached) is the plugin's
manifest entries for the icons + manifest that Chrome had already fetched
outside the cache-storage API; every hashed asset is in.

---

## What Phase 11 did **not** deliver — ranked

1. **The remote-fallback adapter is still unwired, by choice.** `runCompute`/
   `pollJob` in `api.ts` remain the documented hybrid seam, gated on
   `VITE_API_BASE`, and nothing this phase exercised them. Wiring them means
   deciding job-shape questions (which calls are "oversized", what auth, what
   privacy story for a product whose headline is "nothing leaves the tab") —
   product questions, not code ones. Note for whoever wires it:
   `security-headers.conf` still says `connect-src 'self'`, which blocks any
   cross-origin backend until deliberately widened.
2. **No Save shortcut re-saves to the browser.** "Save Project" still means
   the file picker; saving to the browser is a separate explicit action. A
   unified Save that remembers where the project came from (file handle vs
   library row — `FileSystemFileHandle` is structured-cloneable and could
   live in the library) is the obvious next UX step and was cut for scope.
   *(Closed 2026-08-22, Wave E, in its minimal form: `App.tsx` tracks the
   project's provenance (`file` / `browser` / none), so Save re-saves a
   library-opened project to the library — with a confirming toast — and
   keeps meaning the file picker otherwise. The `FileSystemFileHandle`
   half, re-saving to the same file without a picker, stays open.)*
   *(Handle half closed 2026-08-24, Wave I: opening or saving through the
   FS Access API keeps the `FileSystemFileHandle` (Open now prefers
   `showOpenFilePicker` where it exists), and plain Save writes back to that
   file via `createWritable()` after `queryPermission`/`requestPermission` —
   falling back to the picker when the API is absent (Firefox/Safari) or
   permission is refused, with a toast either way. The decision is pure and
   vitest-graded (`src/saveTarget.ts`: `saveTarget(provenance, hasHandle,
   permission)` → library/handle/picker); the handle is structured-cloned
   into IndexedDB beside the autosave (`projectStore`'s file link), so a
   reload keeps Save pickerless after the browser's permission re-prompt.
   Save As always picks — and the picked file becomes the new home. The
   save-check dialog's Save now routes through the same decision; it used
   to always open the picker even for a library project.)*
3. **The precache-everything call has no opt-out.** A metered-connection user
   pays 30 MB on first visit. Workbox supports runtime-caching strategies
   that would precache the boot path (~4 MB) and cache the rest on use; that
   is a better product for constrained networks and a worse one for the
   offline promise. Revisit with real users.
4. **`/help` under a sub-path deploy is still broken** (pre-existing):
   `main.tsx` routes on `location.pathname === '/help'` exactly, and the
   build assumes base `/`. Deploying anywhere but the origin root breaks the
   Help route and the SW scope. Not a regression — but the phase that makes
   deployment a headline should have fixed it, and did not.
   *(Route half closed 2026-08-22, Wave E: `main.tsx` now resolves `/help`
   against `import.meta.env.BASE_URL`, trailing-slash tolerant, so a
   `vite build --base` sub-path deploy routes correctly. The service-worker
   scope under a sub-path remains unverified.)*
   *(SW-scope half verified 2026-08-24, Wave I — the PWA machinery **works
   under a sub-path**; one non-PWA fix was needed. Method: `vite build
   --base /frees-test/` into a scratch dist, served by a scratch server that
   404s everything outside `/frees-test/`, driven headless (chromium via
   Playwright). Measured: the SW registers at `/frees-test/sw.js` with scope
   `http://…/frees-test/`; the manifest gets `scope: "/frees-test/"` from the
   plugin and its relative `start_url: "."`/icon srcs resolve against the
   manifest URL; the precache (`workbox-precache-v2-…/frees-test/`, 99 cache
   keys from the 104-entry manifest — `includeAssets` icons dedupe into the
   glob) fills, and the app boots, reloads and renders **fully offline under
   the prefix** with zero same-origin requests escaping it. Workbox's
   relative precache URLs and `navigateFallback: 'index.html'` resolve
   against the SW script URL, so none of the PWA options needed changing.
   What WAS broken: `buildInfoPlugin` injected a hardcoded
   `/build-info.js` — the one root-absolute URL in the built page; it now
   prefixes the resolved `config.base` (vite.config.ts). One recorded
   nuance, not fixed: `navigateFallbackDenylist: [/^\/api\//]` is anchored
   at the origin root, so it denies nothing under a prefix — moot while the
   remote adapter (gap 1) stays unwired and `VITE_API_BASE` names an
   absolute origin, but whoever wires it under a sub-path must revisit.)*
5. **No multi-tab coordination for the library.** Two tabs can save under the
   same name; last write wins, silently. IndexedDB gives the primitives
   (`versionchange` is handled; `BroadcastChannel` is not used). The autosave
   mirror has the same property, mitigated by it being a mirror of a
   single-tab-authoritative localStorage key.
   *(Narrowed 2026-08-22, Wave E: `projectStore.ts` now posts every
   save/delete/rename on a `BroadcastChannel`, the library modal refreshes
   its listing live, and App warns when the browser-library project a tab has
   open is saved, renamed or deleted by another tab. Writes are still
   last-write-wins — this closed the *silently*, not the race. The autosave
   mirror is deliberately untouched: it mirrors a single-tab-authoritative
   key, so cross-tab posts for it would be noise.)*
6. **The PWA icons are unreviewed by any designer.** They are a
   correctly-sized, correctly-masked placeholder mark (italic *f* + equals),
   generated from SVG in-repo. Fine for install criteria; not a brand.
7. **`sw.js` is tested by grep and browser proof, not by a unit gate.** CI now
   asserts it exists, references the wasm, and ships with the manifest and
   icons — but nothing in CI *runs* it. A Playwright job doing the offline
   reload in CI would close the gap; it needs a browser in the runner and was
   deferred.
   *(Closed 2026-08-22, Wave D4: `web/e2e/offline.spec.ts` with a committed
   Playwright config — first-visit precache, an offline reload, an offline
   Solve graded against the parity golden's `x = 4.69401…`, and zero
   `/api/` requests — runs in the web CI job against the built `dist`
   through `tools/serve-dist.py`, chromium installed in the runner.
   Locally: `npm run build && npx playwright test`, ~8 s after the build.)*

---

## Divergences opened by this pass

Recorded in the ledger at
[`status-phase1.md`](status-phase1.md#opened-by-phase-11-2026-08-05) as items
31–33 (dual-written autosave; static-only deployment; full-precache PWA) —
all three are frontend/deployment divergences from the vendored upstream, not
engine divergences. The engine ledger is unchanged by this phase.
