// Wave G5: the browser-side benchmark — the same five documents as
// crates/frees-core/benches/solve_bench.rs, timed end-to-end through the
// wasm boundary's `solve` export in a real chromium, on the page's main
// thread. Closes docs/status-phase12.md's "did not deliver" item 3 ("no
// browser-side benchmark; the wasm factor is inferred").
//
// Method, chosen to be comparable with the native table rather than clever:
// per document, 3 untimed warmup calls (JIT/lazy-init settle, and the first
// call pays `install_builtin_once`), then timed single calls until 2 s of
// samples or 200 iterations accumulate (min 5). The reported number is the
// MEDIAN; min and n are printed beside it so the spread is visible. Solving
// happens synchronously on the page's main thread — no worker round-trip —
// exactly as the native criterion bench times the public `solve` alone. The
// worker adds one postMessage each way (~µs–ms), which is product overhead,
// not engine cost.
//
// KEEP THE DOCUMENT LIST IN SYNC with solve_bench.rs — same rule as its own
// header states for the JVM oracle directory.
import { readFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { test, expect } from '@playwright/test'

// "type": "module" — no __dirname in ESM specs.
const HERE = dirname(fileURLToPath(import.meta.url))

// solve_bench.rs's SCALAR, verbatim.
const SCALAR = 'x = 4 [m] - y\ny = x / 2\na = 2 * x\n'

const CASES: Array<[string, string]> = [
  ['scalar_two_block', SCALAR],
  ['rankine_cycle', doc('rankine-cycle')],
  ['component_mvem', doc('components_bsweep_mvem_wotmap')],
  ['transient_dyn', doc('dyn_accessor_read')],
  ['control_lqr', doc('ctl-lqr_3state')],
]

function doc(name: string): string {
  return readFileSync(
    join(HERE, '..', '..', 'fixtures', 'corpus', `${name}.frees`),
    'utf8',
  )
}

test('wasm solve benchmark over the five phase-12 documents', async ({ page }) => {
  page.on('console', (m) => console.log(`[page] ${m.text()}`))
  await page.goto('/web/bench/blank.html')

  const rows: Array<{
    name: string
    medianMs: number
    minMs: number
    n: number
  }> = []

  for (const [name, source] of CASES) {
    const r = await page.evaluate(
      async ({ source }) => {
        const w = window as unknown as {
          __frees?: { solve: (s: string, r: string) => string }
        }
        if (!w.__frees) {
          const mod = await import('/web/src/wasm/pkg/frees.js')
          await mod.default('/web/src/wasm/pkg/frees_bg.wasm')
          w.__frees = mod
        }
        const solve = w.__frees.solve

        // Fail loudly outside the timer if the document stops solving — a
        // bench that times an error path reports a fantasy speedup
        // (solve_bench.rs's own rule).
        const probe = JSON.parse(solve(source, ''))
        if (probe.error) return { error: String(probe.error.message ?? probe.error) }

        for (let i = 0; i < 3; i++) solve(source, '')

        const samples: number[] = []
        let elapsed = 0
        while ((elapsed < 2000 || samples.length < 5) && samples.length < 200) {
          const t0 = performance.now()
          solve(source, '')
          const dt = performance.now() - t0
          samples.push(dt)
          elapsed += dt
        }
        samples.sort((a, b) => a - b)
        return {
          medianMs: samples[Math.floor(samples.length / 2)],
          minMs: samples[0],
          n: samples.length,
        }
      },
      { source },
    )
    expect(r, `${name} solves in the browser`).not.toHaveProperty('error')
    const row = r as { medianMs: number; minMs: number; n: number }
    rows.push({ name, ...row })
    console.log(
      `${name}: median ${row.medianMs.toFixed(3)} ms, min ${row.minMs.toFixed(3)} ms, n=${row.n}`,
    )
  }

  console.log('\n| document | wasm (chromium) median | min | n |')
  console.log('|---|---|---|---|')
  for (const r of rows) {
    console.log(
      `| ${r.name} | ${r.medianMs.toFixed(3)} ms | ${r.minMs.toFixed(3)} ms | ${r.n} |`,
    )
  }
})
