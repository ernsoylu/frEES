import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { visualizer } from 'rollup-plugin-visualizer'
import { VitePWA } from 'vite-plugin-pwa'
import pkg from './package.json'

// Inject the runtime build-info.js script into the production HTML.
// This classic (non-module) script sets window.__BUILD_COMMIT__ and is written
// at container start by the nginx entrypoint (see docker-entrypoint.d/).
// Injecting it via transformIndexHtml avoids Vite's "can't be bundled without
// type=module" warning that occurs when the tag is in the static index.html.
// The src is prefixed with the resolved base: injected markup bypasses Vite's
// own base rewriting, and a hardcoded '/build-info.js' was the one root-
// absolute URL a `vite build --base /prefix/` deploy still emitted (found by
// the Wave I sub-path verification; everything else derives from base).
function buildInfoPlugin() {
  let base = '/'
  return {
    name: 'inject-build-info',
    configResolved(config: { base: string }) {
      base = config.base
    },
    transformIndexHtml(html: string) {
      return html.replace(
        '</head>',
        `  <script src="${base}build-info.js"></script>\n  </head>`,
      )
    },
  }
}

// KaTeX's CSS lists every font as woff2 → woff → ttf; browsers that support
// woff2 (all of them, since ~2016) never request the fallbacks, so the woff
// and ttf copies are ~900 KB of dead weight in dist. Drop them at emit time —
// the CSS keeps its (now dangling) fallback URLs, which no supported browser
// resolves.
function stripLegacyKatexFontsPlugin() {
  return {
    name: 'strip-legacy-katex-fonts',
    generateBundle(_opts: unknown, bundle: Record<string, unknown>) {
      for (const name of Object.keys(bundle)) {
        if (/KaTeX_[^/]*\.(ttf|woff)$/.test(name)) delete bundle[name]
      }
    },
  }
}

// Phase 11: installable PWA + full offline session. The service worker
// precaches every built asset — including the ~3 MB wasm engine and the large
// lazy chunks (Plotly, the grids) — because "offline" for an
// engineering tool means the whole tool, not just the shell that was visited
// while online. `registerType: 'prompt'` keeps the old precache serving the
// running tab and surfaces an in-app "update ready" notification (src/pwa.tsx)
// instead of letting a background activation yank hashed chunks out from under
// a live session (which is exactly the stale-chunk condition main.tsx's
// vite:preloadError reload guards against).
function pwaPlugin() {
  return VitePWA({
    registerType: 'prompt',
    includeAssets: ['icons/icon.svg', 'icons/apple-touch-icon.png'],
    manifest: {
      name: 'frees Equation Solver',
      short_name: 'frees',
      description:
        'Declarative equation solving, acausal system modeling and measurement analysis — entirely in the browser.',
      start_url: '.',
      display: 'standalone',
      background_color: '#1a1b1e',
      theme_color: '#1a1b1e',
      icons: [
        { src: 'icons/icon-192.png', sizes: '192x192', type: 'image/png' },
        { src: 'icons/icon-512.png', sizes: '512x512', type: 'image/png' },
        { src: 'icons/icon-maskable-512.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
        { src: 'icons/icon.svg', sizes: 'any', type: 'image/svg+xml' },
      ],
    },
    workbox: {
      // woff2 only: KaTeX's woff/ttf fallbacks are stripped from the bundle
      // (stripLegacyKatexFontsPlugin) and no other dependency ships pre-woff2
      // fonts worth caching.
      globPatterns: ['**/*.{js,css,html,wasm,svg,png,woff2,json}'],
      // The ~3 MB wasm engine and Plotly are above workbox's 2 MiB default;
      // the point of the PWA is that they work offline too. 4 MiB (D10 —
      // down from the 8 MiB the removed 5.1 MB Univer chunk forced): the
      // wasm module is the largest precached file and a new chunk past this
      // cap silently falls out of the offline set, which the offline e2e
      // gate would then catch.
      maximumFileSizeToCacheInBytes: 4 * 1024 * 1024,
      navigateFallback: 'index.html',
      // /api/ is the (optional, unwired) remote-fallback adapter's namespace —
      // a navigation there must fail honestly, not serve the app shell.
      navigateFallbackDenylist: [/^\/api\//],
      cleanupOutdatedCaches: true,
    },
  })
}

export default defineConfig({
  plugins: [
    react(),
    buildInfoPlugin(),
    stripLegacyKatexFontsPlugin(),
    pwaPlugin(),
    visualizer({ open: false, filename: 'stats.html' }),
  ],
  define: {
    // The app version from package.json, baked in at build time so the REPL
    // banner and About dialog show "v0.1.0" without a runtime lookup. Paired
    // with the commit hash (VITE_COMMIT_HASH / window.__BUILD_COMMIT__) it
    // gives a full "frees v0.1.0 (abcdefg)" identity.
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  build: {
    // The editor core ("App" chunk) sits around ~260 kB gzipped after the
    // feature tabs (Diagram, Digitizer, Plots, formatted report), modals, the
    // Help page and Plotly are all code-split into their own lazily-loaded
    // chunks. Bump the warning limit above the core so the build only flags a
    // genuine regression there; the one chunk that still exceeds it is Plotly
    // (~4.8 MB), which is intentionally isolated and dynamically imported in
    // PlotlyChart, so it never blocks first paint.
    chunkSizeWarningLimit: 1000,
    // Split the big shared libraries into their own cached vendor chunks so the
    // editor and Help-page chunks stay small.
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          // Rollup's virtual CommonJS-interop helpers are shared by every
          // CJS-converted vendor module. Left to auto-placement they can land
          // in a vendor chunk that itself imports react (e.g. mantine),
          // creating a mantine <-> react chunk cycle whose init order breaks
          // at runtime (React undefined inside Mantine). Pin them to the
          // react chunk, which everything loads first and imports nothing back.
          if (id.includes('commonjsHelpers')) return 'react'
          // The boot path needs exactly one string out of this family — App.tsx
          // imports DEFAULT_EXAMPLE_TEXT from defaultExample.ts (9 kB). Left to
          // Rollup's auto-placement that module landed in `docs-data` (because
          // examples.ts also imports it), which made the App chunk statically
          // import 1068 kB of reference documentation on every cold start.
          // Pinning it to its own chunk cuts that to 9 kB.
          if (id.includes('defaultExample.ts')) return 'default-example'
          // Likewise, ExamplesModal (3 kB, lazy) needs only examples.ts (51 kB);
          // sharing a chunk with referenceCatalog.ts made opening the Examples
          // dialog a 1 MB download. Keep the two apart — the whole reference
          // corpus is reachable only from the (lazy) Help page.
          if (id.includes('examples.ts')) return 'examples-data'
          if (id.includes('docsCatalog.ts') || id.includes('referenceCatalog.ts') || id.includes('searchIndex.ts')) {
            return 'docs-data'
          }
          if (!id.includes('node_modules')) return undefined
          if (id.includes('@mantine')) return 'mantine'
          if (id.includes('katex')) return 'katex'
          // CodeMirror (+ its @lezer runtime) changes far less often than app
          // code: isolating it keeps ~300 kB of the entry cacheable across
          // deploys and shrinks the frequently-invalidated App chunk.
          if (id.includes('@codemirror') || id.includes('@lezer') || id.includes('@uiw/react-codemirror')) {
            return 'codemirror'
          }
          // Match only the real react/react-dom/scheduler packages. A loose
          // `/react/` also caught e.g. @floating-ui/react (a Mantine dep),
          // dragging it into this chunk and creating a mantine <-> react
          // chunk cycle (tslib landed in mantine, @floating-ui/react needs it,
          // mantine needs @floating-ui/react) that broke init order at runtime.
          if (/node_modules\/(react|react-dom|scheduler)\//.test(id)) {
            return 'react'
          }
          return undefined
        },
      },
    },
  },
  // No /api dev proxy: nothing in src/ issues an /api request — the engine is
  // in-bundle wasm (the former proxy served the retired Spring backend).
  server: {
    port: 5173,
  },
})
