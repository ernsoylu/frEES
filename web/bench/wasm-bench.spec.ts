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
  ['large_component_network', doc('ev-thermal-management')],
  ['stiff_thermofluid_transient', doc('pressure-cooker')],
]

function doc(name: string): string {
  return readFileSync(
    join(HERE, '..', '..', 'fixtures', 'corpus', `${name}.frees`),
    'utf8',
  )
}

test('wasm cold initialization versus warm solve', async ({ page }) => {
  page.on('console', (m) => console.log(`[page] ${m.text()}`))
  await page.goto('/web/bench/blank.html')

  const res = await page.evaluate(async ({ scalar }) => {
    // Cold initialization: time to import module + instantiate WASM + first solve
    const t0 = performance.now()
    const mod = await import('/web/src/wasm/pkg/frees.js')
    await mod.default('/web/src/wasm/pkg/frees_bg.wasm')
    const tInit = performance.now() - t0

    const tFirstSolve0 = performance.now()
    const firstProbe = JSON.parse(mod.solve(scalar, ''))
    const tFirstSolve = performance.now() - tFirstSolve0

    if (!firstProbe.success) return { error: 'first solve failed' }

    // Warm execution: repeated solve of scalar
    const warmSamples: number[] = []
    for (let i = 0; i < 50; i++) {
      const tWarm0 = performance.now()
      mod.solve(scalar, '')
      warmSamples.push(performance.now() - tWarm0)
    }
    warmSamples.sort((a, b) => a - b)
    const warmMedian = warmSamples[Math.floor(warmSamples.length / 2)]

    return {
      tInit,
      tFirstSolve,
      totalCold: tInit + tFirstSolve,
      warmMedian,
    }
  }, { scalar: SCALAR })

  expect(res).not.toHaveProperty('error')
  const r = res as { tInit: number; tFirstSolve: number; totalCold: number; warmMedian: number }
  console.log(`\nCold Initialization: ${r.tInit.toFixed(2)} ms (WASM load/init) + ${r.tFirstSolve.toFixed(2)} ms (first solve) = ${r.totalCold.toFixed(2)} ms total cold`)
  console.log(`Warm Execution: median ${r.warmMedian.toFixed(3)} ms`)
})

test('wasm solve benchmark over the benchmark suite documents', async ({ page }) => {
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
        while ((elapsed < 2000 || samples.length < 3) && samples.length < 100) {
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

test('wasm constrained optimization benchmark', async ({ page }) => {
  page.on('console', (m) => console.log(`[page] ${m.text()}`))
  await page.goto('/web/bench/blank.html')

  const res = await page.evaluate(async () => {
    const mod = await import('/web/src/wasm/pkg/frees.js')
    await mod.default('/web/src/wasm/pkg/frees_bg.wasm')
    const optSource = 'f = (x - 10)^2 + (y - 10)^2 + (z - 10)^2\n'
    const optReq = JSON.stringify({
      objective: 'f',
      decisions: ['x', 'y', 'z'],
      lowers: [0.0, 0.0, 0.0],
      uppers: [15.0, 15.0, 15.0],
      constraints: ['x + y <= 8', 'y + z <= 8', 'x >= 1'],
      method: 'nelder-mead',
      maximize: false,
    })

    const probe = JSON.parse(mod.optimize(optSource, optReq))
    if (!probe.success) return { error: probe.warning || 'optimization did not converge' }

    // 3 warmup iterations
    for (let i = 0; i < 3; i++) mod.optimize(optSource, optReq)

    const optTimes: number[] = []
    for (let i = 0; i < 20; i++) {
      const tStart = performance.now()
      mod.optimize(optSource, optReq)
      optTimes.push(performance.now() - tStart)
    }
    optTimes.sort((a, b) => a - b)
    return {
      medianMs: optTimes[optTimes.length >> 1],
      minMs: optTimes[0],
      n: optTimes.length,
    }
  })

  expect(res).not.toHaveProperty('error')
  const r = res as { medianMs: number; minMs: number; n: number }
  console.log(`constrained_optimization: median ${r.medianMs.toFixed(3)} ms, min ${r.minMs.toFixed(3)} ms, n=${r.n}`)
})

test('wasm 1,000-row sweep benchmark', async ({ page }) => {
  page.on('console', (m) => console.log(`[page] ${m.text()}`))
  await page.goto('/web/bench/blank.html')

  const res = await page.evaluate(async () => {
    const mod = await import('/web/src/wasm/pkg/frees.js')
    await mod.default('/web/src/wasm/pkg/frees_bg.wasm')
    const sweepSource = 'y = 2 * x + 1\n'
    const rows = Array.from({ length: 1000 }, (_, i) => ({ x: i + 1 }))
    const sweepReq = JSON.stringify({
      table: {
        variables: ['x', 'y'],
        rows,
      },
    })

    const probe = JSON.parse(mod.solve_table(sweepSource, sweepReq))
    if (!probe.stats || probe.stats.solved !== 1000) {
      return { error: `sweep failed: ${probe.stats?.solved} solved of 1000` }
    }

    const sweepDurations: number[] = []
    for (let count = 0; count < 10; count++) {
      const mark = performance.now()
      mod.solve_table(sweepSource, sweepReq)
      sweepDurations.push(performance.now() - mark)
    }
    sweepDurations.sort((x, y) => x - y)
    return {
      medianMs: sweepDurations[sweepDurations.length >> 1],
      minMs: sweepDurations[0],
      n: sweepDurations.length,
    }
  })

  expect(res).not.toHaveProperty('error')
  const r = res as { medianMs: number; minMs: number; n: number }
  console.log(`sweep_1000_rows: median ${r.medianMs.toFixed(3)} ms, min ${r.minMs.toFixed(3)} ms, n=${r.n}`)
})
