// Wave G5: the in-browser benchmark's own Playwright config. A SEPARATE
// config, not an env-gated spec in e2e/, so the CI web job's plain
// `npx playwright test` (playwright.config.ts, testDir ./e2e) can never pick
// the benchmark up by accident — benchmarks on shared CI runners are noise,
// which is exactly why the native suite is not a CI gate either
// (docs/status-phase12.md §"did not deliver", item 5).
//
// Run:
//   wasm-pack build crates/frees --release --target web \
//     --out-dir ../../web/src/wasm/pkg
//   cd web && npx playwright test -c playwright.bench.config.ts
//
// Serves the REPO ROOT (not dist): the page imports the wasm-pack output at
// /web/src/wasm/pkg/frees.js directly and reads the bench documents
// from /fixtures/corpus/, so what is timed is the engine the app ships, with
// no worker round-trip and no app chrome — comparable to the native
// criterion bench, which also times the public `solve` alone.
import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './bench',
  timeout: 900_000,
  workers: 1,
  retries: 0,
  use: {
    baseURL: 'http://127.0.0.1:8933',
  },
  webServer: {
    command: 'python3 -m http.server 8933 --bind 127.0.0.1 --directory ..',
    url: 'http://127.0.0.1:8933',
    reuseExistingServer: true,
    timeout: 30_000,
  },
})
