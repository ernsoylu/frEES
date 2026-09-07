// The Phase-11 offline story, as a repeatable test (Wave D4, closing that
// phase's gap 7: "no CI job runs the service worker offline"). Three claims,
// in one browser session because the second depends on the first's precache:
//
//   1. the service worker installs and precaches on first visit;
//   2. with the network cut, the app still boots from the precache;
//   3. an offline Solve of the boot document produces the engine's own
//      numbers — the wasm module came from the cache, not the wire —
//      and no /api/ request is ever attempted.
//
// The boot document's expected values are the parity golden's
// (fixtures/golden/default-boot-document.json): x = 4.694012391660914.
import { expect, test } from '@playwright/test'

test('the app boots and solves fully offline after one visit', async ({ page, context }) => {
  const apiRequests: string[] = []
  page.on('request', (request) => {
    if (new URL(request.url()).pathname.startsWith('/api/')) {
      apiRequests.push(request.url())
    }
  })

  // First visit: let the service worker install and finish precaching. The
  // Workbox precache runs during install, so `ready` (active) means the
  // ~30 MB manifest is in Cache Storage.
  await page.goto('/')
  await expect(page.getByRole('button', { name: 'Solve', exact: true })).toBeVisible({
    timeout: 60_000,
  })
  // First visit opens the Getting Started dialog; dismiss it (its shown-once
  // flag persists in localStorage, so it stays away for the offline reload).
  const welcome = page.getByRole('dialog').filter({ hasText: 'Welcome to frees' })
  if (await welcome.isVisible().catch(() => false)) {
    await page.keyboard.press('Escape')
    await expect(welcome).toBeHidden()
  }
  await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready
    if (!registration.active) throw new Error('service worker did not activate')
  })
  // A controlled reload so the page itself is served by the worker.
  await page.reload()
  await expect(page.getByRole('button', { name: 'Solve', exact: true })).toBeVisible({
    timeout: 60_000,
  })

  // Cut the network. Everything from here must come from the precache.
  await context.setOffline(true)
  await page.reload()
  await expect(page.getByRole('button', { name: 'Solve', exact: true })).toBeVisible({
    timeout: 60_000,
  })

  // Offline Solve of the boot document: the golden's x to the shown digits.
  await page.getByRole('button', { name: 'Solve', exact: true }).click()
  await expect(page.getByText(/4\.69401/).first()).toBeVisible({ timeout: 60_000 })

  expect(apiRequests, 'the app must never call /api/').toEqual([])
})

test('zero-network solve with all external network requests aborted', async ({ page }) => {
  // Abort all non-local network traffic to verify zero external calls
  await page.route('**/*', (route) => {
    const url = new URL(route.request().url())
    // Allow local test server (localhost / 127.0.0.1) and data/blob URIs only
    if (url.hostname === '127.0.0.1' || url.hostname === 'localhost' || url.protocol === 'data:' || url.protocol === 'blob:') {
      route.continue()
    } else {
      route.abort()
    }
  })

  await page.goto('/')
  await expect(page.getByRole('button', { name: 'Solve', exact: true })).toBeVisible({
    timeout: 60_000,
  })
  const welcome = page.getByRole('dialog').filter({ hasText: 'Welcome to frees' })
  if (await welcome.isVisible().catch(() => false)) {
    await page.keyboard.press('Escape')
    await expect(welcome).toBeHidden()
  }

  // Perform solve with all external routes blocked
  await page.getByRole('button', { name: 'Solve', exact: true }).click()
  await expect(page.getByText(/4\.69401/).first()).toBeVisible({ timeout: 60_000 })
})

