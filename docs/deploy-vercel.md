# Deploying frees to Vercel

frees is a **fully static SPA** — the engine is WebAssembly inside the bundle,
there is no backend, no serverless functions, and no `/api/*` (those routes must
**404**, and the `vercel.json` rewrite deliberately excludes them). Two hard
constraints frame everything below:

1. **D3 (`docs/decisions/0003-threading-model.md`): plain static files, NO
   COOP/COEP.** Never add `Cross-Origin-Opener-Policy` or
   `Cross-Origin-Embedder-Policy` headers, and never enable any Vercel
   "cross-origin isolation" toggle or integration. Vercel does not send these
   headers by default — keep it that way. The worker pool and the PWA depend on
   their absence.
2. **The service worker must revalidate on every load.** `vercel.json` sends
   `Cache-Control: no-cache` for everything outside `/assets/` (matching
   `web/nginx.conf.template`); if `sw.js` ever becomes HTTP-cacheable, deploys
   stop reaching users.

Everything Vercel needs is in three checked-in files:

| File | Role |
|---|---|
| `/vercel.json` | build commands, output dir, headers (security + caching), SPA rewrite |
| `/tools/vercel-build.sh` | idempotent toolchain bootstrap (rustup + wasm32 + wasm-pack 0.13.1) + the two-stage build |
| `/.vercelignore` | trims CLI uploads (target/, node_modules, fixtures, docs, …) |

---

## Path A — Vercel builds everything

**Dashboard settings** (Project → Settings → Build & Development Settings):

- **Framework Preset: "Other"** (`vercel.json` sets `"framework": null`; do not
  pick "Vite" — its preset assumes `package.json` at the project root, and ours
  is in `web/`).
- **Root Directory: leave as the repository root.** The build needs `crates/`
  and `Cargo.toml`, which live above `web/`.
- **Node.js Version: 22.x.** The usual pin is `"engines"` in `package.json`,
  which this repo deliberately does not use (the frontend tree is a vendored
  sync; see `web/WASM-PORT.md`), so the dashboard setting is the pin.
  `web/.nvmrc` says 22; the build script fails below Node 20 and warns below 22.
- Build Command / Output Directory / Install Command: leave empty —
  `vercel.json` overrides them (`bash tools/vercel-build.sh`, `web/dist`,
  `cd web && npm ci --ignore-scripts`).

Then connect the git repo (or run `vercel` from the repo root — not done here).

**What the build does and costs.** The Vercel build container has Node but no
Rust, so `tools/vercel-build.sh` first installs rustup (honouring
`rust-toolchain.toml`: stable + wasm32-unknown-unknown, minimal profile, roughly
a few hundred MB of toolchain download), drops in the prebuilt
**wasm-pack v0.13.1** binary (the CI-pinned version; seconds, not a `cargo
install`), then runs

```
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg
cd web && npm run build
```

Expect **~10–20 minutes per deploy**: a cold `--release` compile of the engine
workspace dominates, and **Vercel's build cache does not persist `~/.cargo` or
`target/`**, so every deploy recompiles Rust from scratch. This fits the
45-minute build limit but burns build minutes. That cost is the main reason
Path B is recommended for routine deploys; keep Path A as the "a git push is a
deploy" convenience if the minutes don't bother you.

Expected log noise, all harmless: npm peer-dependency notes silenced by
`web/.npmrc` (`legacy-peer-deps=true`, picked up because `npm ci` runs inside
`web/`). The build carries **zero** Rust future-incompat warnings — the
`meval → nom 1.2.4` debt left with MDF4 (decision D6).

## Path B — prebuilt (recommended)

Build `web/dist` locally or in CI (where the Rust toolchain already exists and
`target/` is warm — incremental rebuilds are minutes, not tens of minutes), then
hand Vercel the finished output.

### B1: `vercel build` + `vercel deploy --prebuilt` (keeps vercel.json semantics)

```bash
cd /Users/erensoylu/homecloud/dev/frees

# one-time per machine: link the project (interactive)
vercel link

# pull the project settings, build locally using vercel.json
# (this runs tools/vercel-build.sh on YOUR machine — idempotent, skips
#  toolchain installs that already exist), then upload only the output:
vercel pull --environment=production
vercel build --prod            # produces .vercel/output/ from web/dist + vercel.json
vercel deploy --prebuilt --prod
```

`vercel build` bakes the headers and the SPA rewrite from `vercel.json` into
`.vercel/output/config.json`, so the deployed behaviour is identical to Path A —
just without spending Vercel build minutes. This is the recommended production
path.

If you want to reuse an existing `web/dist` (e.g. CI already ran
`tools/vercel-build.sh`), `vercel build` will still invoke the buildCommand;
the script's idempotence makes that cheap everywhere except `npm run build`
itself.

### B2: drag-and-drop (no CLI at all)

The dashboard's "Deploy" drag-and-drop treats the dropped folder as the project
root — which means the repo-root `vercel.json` is **not** part of the upload and
its headers/rewrites would be lost. To use this path:

```bash
bash tools/vercel-build.sh          # produce web/dist
# give the dropped folder its own config: headers + rewrite ONLY —
# no buildCommand/outputDirectory (there is nothing to build in dist/):
python3 - <<'EOF'
import json
cfg = json.load(open('vercel.json'))
json.dump({k: cfg[k] for k in ('headers', 'rewrites')},
          open('web/dist/vercel.json', 'w'), indent=2)
EOF
```

then drop `web/dist` onto vercel.com/new. Caveats: the generated
`web/dist/vercel.json` is served as a public static file (it contains no
secrets, only the header table), and it must be regenerated after every build
because `npm run build` wipes `dist/`. Prefer B1.

---

## After the first deploy — verification checklist

Replace `https://<app>` with the deployment URL.

**Routes**

```bash
curl -s -o /dev/null -w '%{http_code}\n' https://<app>/            # 200
curl -s -o /dev/null -w '%{http_code}\n' https://<app>/help        # 200 — SPA fallback; body is index.html
curl -s -o /dev/null -w '%{http_code}\n' https://<app>/no/such/page  # 200 — extensionless → shell (app decides)
curl -s -o /dev/null -w '%{http_code}\n' https://<app>/api/solve   # 404 — /api/* is intentionally dead
curl -s -o /dev/null -w '%{http_code}\n' https://<app>/assets/nope.js  # 404 — missing assets must NOT get HTML
```

In the browser, open `https://<app>/help` directly (not via in-app navigation):
`web/src/main.tsx` routes on `pathname === '/help'`, so a hard load must render
the help view, not the workspace.

**Headers**

```bash
curl -sI https://<app>/sw.js | grep -i cache-control        # no-cache
curl -sI https://<app>/manifest.webmanifest | grep -i cache-control  # no-cache
curl -sI https://<app>/ | grep -iE 'cache-control|content-security|frame-options'
# no-cache + the CSP from web/security-headers.conf + SAMEORIGIN
ASSET=$(curl -s https://<app>/ | grep -o 'assets/index-[^"]*\.js' | head -1)
curl -sI "https://<app>/$ASSET" | grep -i cache-control     # public, max-age=31536000, immutable
# and the D3 invariant — BOTH must print nothing:
curl -sI https://<app>/ | grep -i 'cross-origin-opener\|cross-origin-embedder'
```

Vercel adds its own infrastructure headers (`x-vercel-id`, `x-vercel-cache`,
`etag`, `age`, sometimes `x-frame-options` is joined by ours — duplicates of the
same value are fine). Its default `cache-control` for static files
(`public, max-age=0, must-revalidate`) is overridden by the explicit rules for
every path class we care about.

**PWA / service worker** (Chrome)

1. First load: DevTools → Application → Service Workers shows `sw.js`
   activated at scope `/`; Cache Storage populates ~**334 entries /
   ~30 MB** — including the ~3 MB wasm engine.
2. Solve something (F2) — baseline.
3. DevTools → Network → **Offline**, then reload: the app shell must render and
   a solve must still succeed (the engine comes out of the precache; this is
   the Phase 11 browser proof, now on the CDN).
4. Update flow: push any second deploy, go back online, reload — the
   registration re-fetches `sw.js` (this is what `no-cache` buys), and the
   in-app update prompt appears rather than assets vanishing under the tab.

---

## Surprises to expect on first deploy

- **The precache downloads ~30 MB on first visit** (334 entries, everything
  including the wasm engine and all lazy chunks). On a metered connection that
  is a real cost, and `docs/status-phase11.md` lists the missing opt-out as a
  known gap. First *paint* does not wait for it, but the offline promise only
  holds once it finishes.
- **Path A burns 10–20 build minutes per deploy** (cold Rust toolchain +
  release compile, no Rust cache between builds). Prefer Path B for iteration.
- **`/api/*` 404s by design.** Any lingering client code that calls a live
  endpoint is a bug in the static build, not a deployment problem
  (`web/src/api.ts` stubs everything unported).
- **HSTS is live from the first response** (`max-age=15552000;
  includeSubDomains`) — inherent to the `*.vercel.app` domain anyway, but if
  you attach a custom apex domain, every subdomain of it must be HTTPS-capable
  from then on.
- **The CSP has `connect-src 'self'`.** The optional remote-compute adapter
  (`VITE_API_BASE`) cannot talk to a cross-origin backend until that directive
  is deliberately widened — noted in `docs/status-phase11.md`.
- **Do not set the Vite framework preset later.** Switching the preset to
  "Vite" makes Vercel look for a root `package.json`/`index.html` and can
  silently replace the SPA-fallback semantics; "Other" + `vercel.json` is the
  supported configuration.
