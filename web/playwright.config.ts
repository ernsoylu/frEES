// The offline-PWA proof, promoted from Phase 11's one-off browser run into a
// repeatable gate (Wave D4). Serves the BUILT dist through the same dumb
// static server the original proof used — no Vite dev server, no HMR, so the
// service worker installs and the precache is what a real deploy ships.
//
// Run locally:   npm run build && npx playwright test
// The CI job builds dist first and runs against chromium only.
import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  timeout: 180_000,
  // The precache is ~30 MB; one worker keeps the static server honest.
  workers: 1,
  retries: process.env.CI ? 1 : 0,
  use: {
    baseURL: 'http://127.0.0.1:8931',
    trace: 'retain-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],
  webServer: {
    command: 'python3 ../tools/serve-dist.py dist 8931',
    url: 'http://127.0.0.1:8931',
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
})
