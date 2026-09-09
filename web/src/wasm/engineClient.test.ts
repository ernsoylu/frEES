// Phase 12: the worker-death path, exercised for real.
//
// Every other suite that touches engineClient mocks the module away; this one
// drives the real singleton with a fake Worker, because the fail()/respawn
// path is load-bearing for the whole product: the shipped wasm is
// panic = "abort", so an engine defect kills the worker script, and the ONLY
// recovery is that engineClient rejects everything in flight and spawns a
// fresh worker on the next call. Nothing tested that until now.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

class FakeWorker {
  static instances: FakeWorker[] = []
  onmessage: ((e: { data: unknown }) => void) | null = null
  onerror: ((e: { message?: string }) => void) | null = null
  onmessageerror: (() => void) | null = null
  posted: { id: number; method: string; args: string[] }[] = []
  terminated = false

  constructor() {
    FakeWorker.instances.push(this)
  }

  postMessage(msg: unknown) {
    this.posted.push(msg as (typeof this.posted)[number])
  }

  terminate() {
    this.terminated = true
  }
}

beforeEach(() => {
  FakeWorker.instances = []
  vi.stubGlobal('Worker', FakeWorker)
  // The singleton lives at module scope; a fresh module per test.
  vi.resetModules()
})

afterEach(() => {
  vi.unstubAllGlobals()
})

const client = () => import('./engineClient')

describe('engineClient worker lifecycle', () => {
  it('rejects dispatch failures and accepts the next request', async () => {
    const { wasmVersion } = await client()
    vi.spyOn(FakeWorker.prototype, 'postMessage').mockImplementationOnce(() => {
      throw new Error('dispatch failed')
    })
    await expect(wasmVersion()).rejects.toThrow('dispatch failed')
    const next = wasmVersion()
    const w = FakeWorker.instances[0]
    w.onmessage?.({ data: { id: w.posted[0].id, ok: true, result: 'ok' } })
    await expect(next).resolves.toBe('ok')
    vi.restoreAllMocks()
  })

  it('ignores late errors from retired workers after a new request starts', async () => {
    const { wasmVersion, wasmStop } = await client()
    const first = wasmVersion()
    const old = FakeWorker.instances[0]
    wasmStop()
    await expect(first).rejects.toThrow('Operation stopped')
    const next = wasmVersion()
    const current = FakeWorker.instances[1]
    old.onerror?.({ message: 'late crash' })
    old.onmessageerror?.()
    expect(current.terminated).toBe(false)
    current.onmessage?.({ data: { id: current.posted[0].id, ok: true, result: 'ok' } })
    await expect(next).resolves.toBe('ok')
  })

  it('spawns exactly one worker across many calls and correlates by id', async () => {
    const { wasmVersion, wasmCheck } = await client()
    const p1 = wasmVersion()
    const p2 = wasmCheck('x = 2', '{}')
    expect(FakeWorker.instances.length).toBe(1)
    const w = FakeWorker.instances[0]
    expect(w.posted.map((m) => m.method)).toEqual(['version', 'check'])
    // Answer out of order — correlation is by id, not arrival.
    w.onmessage?.({ data: { id: w.posted[1].id, ok: true, result: '{"solvable":true}' } })
    w.onmessage?.({ data: { id: w.posted[0].id, ok: true, result: '0.1.0' } })
    await expect(p2).resolves.toEqual({ solvable: true })
    await expect(p1).resolves.toBe('0.1.0')
  })

  it('a dead worker rejects everything in flight and the next call respawns', async () => {
    const { wasmVersion, wasmSolve } = await client()
    const p1 = wasmVersion()
    const p2 = wasmSolve('x = 2', '{}')
    const first = FakeWorker.instances[0]

    // The worker script dies (what a wasm abort looks like from outside).
    first.onerror?.({ message: 'RuntimeError: unreachable' })

    await expect(p1).rejects.toThrow('RuntimeError: unreachable')
    await expect(p2).rejects.toThrow('RuntimeError: unreachable')
    expect(first.terminated).toBe(true)

    // The next call must not hang on the corpse: a fresh worker spawns.
    const p3 = wasmVersion()
    expect(FakeWorker.instances.length).toBe(2)
    const second = FakeWorker.instances[1]
    second.onmessage?.({ data: { id: second.posted[0].id, ok: true, result: '0.1.0' } })
    await expect(p3).resolves.toBe('0.1.0')
  })

  it('an unreadable message from the worker also fails over', async () => {
    const { wasmVersion } = await client()
    const p = wasmVersion()
    FakeWorker.instances[0].onmessageerror?.()
    await expect(p).rejects.toThrow('unreadable')
    expect(FakeWorker.instances[0].terminated).toBe(true)
  })

  it('a response for an unknown id is ignored, not a crash', async () => {
    const { wasmVersion } = await client()
    const p = wasmVersion()
    const w = FakeWorker.instances[0]
    w.onmessage?.({ data: { id: 999, ok: true, result: 'stray' } })
    w.onmessage?.({ data: { id: w.posted[0].id, ok: true, result: '0.1.0' } })
    await expect(p).resolves.toBe('0.1.0')
  })

  it('an {ok:false} message rejects that one call and keeps the worker', async () => {
    const { wasmCheck, wasmVersion } = await client()
    const bad = wasmCheck('nonsense', '{}')
    const w = FakeWorker.instances[0]
    w.onmessage?.({ data: { id: w.posted[0].id, ok: false, error: 'parse failed' } })
    await expect(bad).rejects.toThrow('parse failed')
    expect(w.terminated).toBe(false)

    const good = wasmVersion()
    expect(FakeWorker.instances.length).toBe(1) // same worker, no respawn
    w.onmessage?.({ data: { id: w.posted[1].id, ok: true, result: '0.1.0' } })
    await expect(good).resolves.toBe('0.1.0')
  })
})

// Wave T5. The bar the Solve button paints is driven from *inside* a blocking
// wasm call, so the one thing that must not happen is a progress message
// settling or dropping the request it belongs to.
describe('engineClient solve progress', () => {
  it('delivers progress without settling the request', async () => {
    const { wasmSolve } = await client()
    const seen: number[] = []
    const p = wasmSolve('x = 1', '{}', (f) => seen.push(f))
    const w = FakeWorker.instances[0]
    const id = w.posted[0].id

    w.onmessage?.({ data: { id, progress: 0.25 } })
    w.onmessage?.({ data: { id, progress: 0.75 } })
    expect(seen).toEqual([0.25, 0.75])

    // Still pending: only an ok/error message settles it.
    w.onmessage?.({ data: { id, ok: true, result: '{"success":true}' } })
    await expect(p).resolves.toEqual({ success: true })
  })

  it('ignores progress for a request that has already settled', async () => {
    const { wasmSolve } = await client()
    const seen: number[] = []
    const p = wasmSolve('x = 1', '{}', (f) => seen.push(f))
    const w = FakeWorker.instances[0]
    const id = w.posted[0].id
    w.onmessage?.({ data: { id, ok: true, result: '{"success":true}' } })
    await expect(p).resolves.toEqual({ success: true })

    // A late frame from a solve that already answered must not reach a listener
    // whose UI has moved on.
    w.onmessage?.({ data: { id, progress: 0.5 } })
    expect(seen).toEqual([])
  })

  it('a throwing progress listener does not break the request', async () => {
    const { wasmSolve } = await client()
    const p = wasmSolve('x = 1', '{}', () => {
      throw new Error('render blew up')
    })
    const w = FakeWorker.instances[0]
    const id = w.posted[0].id
    w.onmessage?.({ data: { id, progress: 0.5 } })
    w.onmessage?.({ data: { id, ok: true, result: '{"success":true}' } })
    await expect(p).resolves.toEqual({ success: true })
    expect(w.terminated).toBe(false)
  })

  it('a solve with no listener is unaffected by progress messages', async () => {
    const { wasmSolve } = await client()
    const p = wasmSolve('x = 1', '{}')
    const w = FakeWorker.instances[0]
    const id = w.posted[0].id
    w.onmessage?.({ data: { id, progress: 0.5 } })
    w.onmessage?.({ data: { id, ok: true, result: '{"success":true}' } })
    await expect(p).resolves.toEqual({ success: true })
  })
})

describe('engineClient worker pool for independent sweeps (Phase 7)', () => {
  it('detects parametric accessors with exact parity to Rust engine tests', async () => {
    const { mentionsParametricAccessor } = await client()

    // Hits from crates/frees-core/src/analysis/parametric.rs
    const positiveCases = [
      'r = TableRun#()',
      'r = TableRun()',
      'v = TableValue(1, 2)',
      "s = tablesum('p')",
      "s = TABLESTDDEV('p')",
      "a = TableAvg ('p')",
      "a = TableAvg\t('p')",
      "x = 1 + TableMin('p')",
      "{ note: TableMax('p') }",
      "— TableSum('p')",
    ]
    for (const text of positiveCases) {
      expect(mentionsParametricAccessor(text), `Expected hit for: ${text}`).toBe(true)
    }

    // Misses from crates/frees-core/src/analysis/parametric.rs
    const negativeCases = [
      'P = t * 3',
      "x = MyTableSum('p')",
      "x = TableSummary('p')",
      'TableSum = 3',
      'x = TableSum',
      '',
      'µ = 1 [kg]',
    ]
    for (const text of negativeCases) {
      expect(mentionsParametricAccessor(text), `Expected miss for: ${text}`).toBe(false)
    }
  })

  it('bounds and clamps worker pool concurrency', async () => {
    const {
      getWorkerPoolConcurrency,
      setWorkerPoolConcurrency,
      resetWorkerPoolConcurrency,
      MAX_WORKER_POOL_SIZE,
    } = await client()

    expect(MAX_WORKER_POOL_SIZE).toBe(4)
    setWorkerPoolConcurrency(1)
    expect(getWorkerPoolConcurrency()).toBe(1)
    setWorkerPoolConcurrency(3)
    expect(getWorkerPoolConcurrency()).toBe(3)
    setWorkerPoolConcurrency(100) // clamped to MAX_WORKER_POOL_SIZE
    expect(getWorkerPoolConcurrency()).toBe(4)
    setWorkerPoolConcurrency(0) // clamped to min 1
    expect(getWorkerPoolConcurrency()).toBe(1)
    setWorkerPoolConcurrency(Number.NaN)
    expect(getWorkerPoolConcurrency()).toBe(1)
    resetWorkerPoolConcurrency()
  })

  it('keeps accessor-dependent sweeps serial on a single worker even when concurrency is high', async () => {
    const { wasmSolveTable, setWorkerPoolConcurrency } = await client()
    setWorkerPoolConcurrency(4)

    const source = "avg = TableAvg('y')\ny = 2 * x\n"
    const req = JSON.stringify({
      table: {
        variables: ['x', 'y'],
        rows: [{ x: 1 }, { x: 2 }, { x: 3 }, { x: 4 }],
      },
    })

    const p = wasmSolveTable(source, req)
    expect(FakeWorker.instances.length).toBe(1)
    const w = FakeWorker.instances[0]
    expect(w.posted).toHaveLength(1)
    expect(w.posted[0].method).toBe('solveTable')

    const responsePayload = JSON.stringify({
      results: [
        { success: true, values: { x: 1, y: 2, avg: 5 }, error: null },
        { success: true, values: { x: 2, y: 4, avg: 5 }, error: null },
        { success: true, values: { x: 3, y: 6, avg: 5 }, error: null },
        { success: true, values: { x: 4, y: 8, avg: 5 }, error: null },
      ],
      stats: {
        converged: true,
        passes: 2,
        termination: 'completed',
        accessor: true,
        runs: 4,
        solved: 4,
        failed: 0,
        notRun: 0,
        equations: 2,
        unknowns: 2,
        iterations: 8,
        elapsedMillis: 15,
        maxResidual: 0,
      },
      variables: [{ name: 'y', value: 8 }],
    })
    w.onmessage?.({ data: { id: w.posted[0].id, ok: true, result: responsePayload } })

    const result = await p
    expect(result.stats?.accessor).toBe(true)
    expect(result.results).toHaveLength(4)
  })

  it('chunks independent sweeps across multiple pool workers and merges results in order', async () => {
    const { wasmSolveTable, setWorkerPoolConcurrency } = await client()
    setWorkerPoolConcurrency(2)

    const source = 'y = 2 * x\n'
    const req = JSON.stringify({
      table: {
        variables: ['x', 'y'],
        rows: [{ x: 1 }, { x: 2 }, { x: 3 }, { x: 4 }],
      },
    })

    const progressReports: number[] = []
    const p = wasmSolveTable(source, req, (f) => progressReports.push(f))

    // Must spawn 2 workers for 2 chunks (2 rows each)
    expect(FakeWorker.instances.length).toBe(2)
    const [w0, w1] = FakeWorker.instances
    expect(w0.posted).toHaveLength(1)
    expect(w1.posted).toHaveLength(1)

    // Verify chunk payloads
    const req0 = JSON.parse(w0.posted[0].args[1])
    const req1 = JSON.parse(w1.posted[0].args[1])
    expect(req0.table.rows).toEqual([{ x: 1 }, { x: 2 }])
    expect(req1.table.rows).toEqual([{ x: 3 }, { x: 4 }])

    // Progress aggregation test
    w0.onmessage?.({ data: { id: w0.posted[0].id, progress: 0.5 } })
    expect(progressReports).toContain(0.25) // 0.5 * 2/4 = 0.25
    w1.onmessage?.({ data: { id: w1.posted[0].id, progress: 1.0 } })
    expect(progressReports).toContain(0.75) // 0.25 + 1.0 * 2/4 = 0.75

    // Resolve workers
    const chunk0Res = JSON.stringify({
      results: [
        { success: true, values: { x: 1, y: 2 }, error: null },
        { success: true, values: { x: 2, y: 4 }, error: null },
      ],
      stats: {
        converged: true,
        passes: 1,
        termination: 'completed',
        accessor: false,
        runs: 2,
        solved: 2,
        failed: 0,
        notRun: 0,
        equations: 2,
        unknowns: 2,
        iterations: 4,
        elapsedMillis: 5,
        maxResidual: 1e-12,
      },
      variables: [{ name: 'y', value: 4 }],
    })
    const chunk1Res = JSON.stringify({
      results: [
        { success: true, values: { x: 3, y: 6 }, error: null },
        { success: true, values: { x: 4, y: 8 }, error: null },
      ],
      stats: {
        converged: true,
        passes: 1,
        termination: 'completed',
        accessor: false,
        runs: 2,
        solved: 2,
        failed: 0,
        notRun: 0,
        equations: 2,
        unknowns: 2,
        iterations: 4,
        elapsedMillis: 6,
        maxResidual: 2e-12,
      },
      variables: [{ name: 'y', value: 8 }],
    })

    w0.onmessage?.({ data: { id: w0.posted[0].id, ok: true, result: chunk0Res } })
    w1.onmessage?.({ data: { id: w1.posted[0].id, ok: true, result: chunk1Res } })

    const res = await p
    expect(res.results).toHaveLength(4)
    expect(res.results!.map((r) => r.values.y)).toEqual([2, 4, 6, 8])
    expect(res.stats?.runs).toBe(4)
    expect(res.stats?.solved).toBe(4)
    expect(res.stats?.failed).toBe(0)
    expect(res.stats?.iterations).toBe(8)
    expect(res.stats?.maxResidual).toBe(2e-12)
    // Variables must come from the last successful chunk (row 4)
    expect(res.variables).toEqual([{ name: 'y', value: 8 }])
  })

  it('wasmStop terminates all workers in the pool during an in-flight sweep', async () => {
    const { wasmSolveTable, setWorkerPoolConcurrency, wasmStop } = await client()
    setWorkerPoolConcurrency(2)

    const source = 'y = 2 * x\n'
    const req = JSON.stringify({
      table: {
        variables: ['x', 'y'],
        rows: [{ x: 1 }, { x: 2 }, { x: 3 }, { x: 4 }],
      },
    })

    const p = wasmSolveTable(source, req)
    expect(FakeWorker.instances.length).toBe(2)
    const [w0, w1] = FakeWorker.instances

    wasmStop()

    expect(w0.terminated).toBe(true)
    expect(w1.terminated).toBe(true)
    await expect(p).rejects.toThrow('Operation stopped')
  })

  it('a fatal error in any pool worker terminates all workers and rejects in flight', async () => {
    const { wasmSolveTable, setWorkerPoolConcurrency } = await client()
    setWorkerPoolConcurrency(2)

    const source = 'y = 2 * x\n'
    const req = JSON.stringify({
      table: {
        variables: ['x', 'y'],
        rows: [{ x: 1 }, { x: 2 }, { x: 3 }, { x: 4 }],
      },
    })

    const p = wasmSolveTable(source, req)
    expect(FakeWorker.instances.length).toBe(2)
    const [w0, w1] = FakeWorker.instances

    w1.onerror?.({ message: 'Wasm worker crash' })

    expect(w0.terminated).toBe(true)
    expect(w1.terminated).toBe(true)
    await expect(p).rejects.toThrow('Wasm worker crash')
  })

  it('retireExtraWorkers terminates workers 1..N-1 while keeping worker 0 alive', async () => {
    const { wasmSolveTable, setWorkerPoolConcurrency, retireExtraWorkers } = await client()
    setWorkerPoolConcurrency(2)

    const source = 'y = 2 * x\n'
    const req = JSON.stringify({
      table: {
        variables: ['x', 'y'],
        rows: [{ x: 1 }, { x: 2 }, { x: 3 }, { x: 4 }],
      },
    })

    const p = wasmSolveTable(source, req)
    const [w0, w1] = FakeWorker.instances
    const chunkRes = JSON.stringify({
      results: [{ success: true, values: { x: 1, y: 2 }, error: null }],
      stats: { solved: 1, runs: 1, iterations: 1, converged: true, termination: 'completed' },
      variables: [],
    })
    w0.onmessage?.({ data: { id: w0.posted[0].id, ok: true, result: chunkRes } })
    w1.onmessage?.({ data: { id: w1.posted[0].id, ok: true, result: chunkRes } })
    await p

    retireExtraWorkers()
    expect(w0.terminated).toBe(false)
    expect(w1.terminated).toBe(true)
  })

  it('retiring busy extras settles the sweep and preserves primary requests', async () => {
    const { wasmSolveTable, wasmVersion, setWorkerPoolConcurrency, retireExtraWorkers } = await client()
    setWorkerPoolConcurrency(2)
    const sweep = wasmSolveTable('y = x', JSON.stringify({
      table: { variables: ['x', 'y'], rows: [{ x: 1 }, { x: 2 }] },
    }))
    const version = wasmVersion()
    const [primary, extra] = FakeWorker.instances
    retireExtraWorkers()
    expect(primary.terminated).toBe(false)
    expect(extra.terminated).toBe(true)
    await expect(sweep).rejects.toThrow('Operation stopped')
    primary.onmessage?.({ data: { id: primary.posted[1].id, ok: true, result: 'ok' } })
    await expect(version).resolves.toBe('ok')
  })

  it('produces identical row results, stats, and variable lists across worker counts (1 vs 2 vs 4)', async () => {
    const { mergeSolveTableResponses } = await client()

    // 8-row workload with 7 solved rows and 1 failed row (Row 5 fails: 1/0)
    const perRowData = [
      { i: 1, x: 1, y: 2, success: true, error: null },
      { i: 2, x: 2, y: 4, success: true, error: null },
      { i: 3, x: 3, y: 6, success: true, error: null },
      { i: 4, x: 4, y: 8, success: true, error: null },
      { i: 5, x: 5, y: 0, success: false, error: 'division by zero' },
      { i: 6, x: 6, y: 12, success: true, error: null },
      { i: 7, x: 7, y: 14, success: true, error: null },
      { i: 8, x: 8, y: 16, success: true, error: null },
    ]

    // Simulate what the Rust engine emits for any slice of rows
    const simulateEngineChunk = (slice: typeof perRowData) => {
      const results = slice.map((r) => ({
        success: r.success,
        values: (r.success ? { x: r.x, y: r.y } : {}) as Record<string, number>,
        error: r.error,
      }))
      const solved = slice.filter((r) => r.success).length
      const failed = slice.length - solved
      const lastSolved = slice.filter((r) => r.success).slice(-1)[0]
      return {
        results,
        stats: {
          converged: true,
          passes: 1,
          termination: 'completed',
          accessor: false,
          runs: slice.length,
          solved,
          failed,
          notRun: 0,
          equations: lastSolved ? 2 : 0,
          unknowns: lastSolved ? 2 : 0,
          iterations: solved * 2,
          elapsedMillis: 10,
          maxResidual: lastSolved ? 1e-12 : 0,
        },
        variables: lastSolved ? [{ name: 'y', value: lastSolved.y }] : [],
      }
    }

    // 1 Worker execution (single chunk of 8 rows)
    const out1Worker = mergeSolveTableResponses([simulateEngineChunk(perRowData)], 8, 20)

    // 2 Workers execution (2 chunks of 4 rows)
    const out2Workers = mergeSolveTableResponses(
      [simulateEngineChunk(perRowData.slice(0, 4)), simulateEngineChunk(perRowData.slice(4, 8))],
      8,
      12,
    )

    // 4 Workers execution (4 chunks of 2 rows)
    const out4Workers = mergeSolveTableResponses(
      [
        simulateEngineChunk(perRowData.slice(0, 2)),
        simulateEngineChunk(perRowData.slice(2, 4)),
        simulateEngineChunk(perRowData.slice(4, 6)),
        simulateEngineChunk(perRowData.slice(6, 8)),
      ],
      8,
      8,
    )

    // Verify exact equality of numerical results and row order across worker counts
    expect(out2Workers.results).toEqual(out1Worker.results)
    expect(out4Workers.results).toEqual(out1Worker.results)

    // Verify exact equality of variable list (last successful row: Row 8 with y=16)
    expect(out1Worker.variables).toEqual([{ name: 'y', value: 16 }])
    expect(out2Workers.variables).toEqual(out1Worker.variables)
    expect(out4Workers.variables).toEqual(out1Worker.variables)

    // Verify stats equivalence (runs, solved, failed, equations, unknowns, iterations, maxResidual)
    for (const out of [out2Workers, out4Workers]) {
      expect(out.stats?.runs).toBe(out1Worker.stats?.runs)
      expect(out.stats?.solved).toBe(out1Worker.stats?.solved)
      expect(out.stats?.failed).toBe(out1Worker.stats?.failed)
      expect(out.stats?.equations).toBe(out1Worker.stats?.equations)
      expect(out.stats?.unknowns).toBe(out1Worker.stats?.unknowns)
      expect(out.stats?.iterations).toBe(out1Worker.stats?.iterations)
      expect(out.stats?.maxResidual).toBe(out1Worker.stats?.maxResidual)
      expect(out.stats?.converged).toBe(out1Worker.stats?.converged)
      expect(out.stats?.termination).toBe(out1Worker.stats?.termination)
      expect(out.stats?.accessor).toBe(false)
    }
  })

  it('forwards top-level chunk errors directly', async () => {
    const { wasmSolveTable, setWorkerPoolConcurrency } = await client()
    setWorkerPoolConcurrency(2)

    const source = 'nonsense document\n'
    const req = JSON.stringify({
      table: {
        variables: ['x', 'y'],
        rows: [{ x: 1 }, { x: 2 }],
      },
    })

    const p = wasmSolveTable(source, req)
    const [w0, w1] = FakeWorker.instances

    const errPayload = JSON.stringify({
      results: [],
      stats: null,
      variables: [],
      error: 'Syntax error: unexpected token',
    })
    w0.onmessage?.({ data: { id: w0.posted[0].id, ok: true, result: errPayload } })
    w1.onmessage?.({ data: { id: w1.posted[0].id, ok: true, result: errPayload } })

    const res = await p
    expect(res.error).toBe('Syntax error: unexpected token')
    expect(res.results).toEqual([])
  })

  it('wasmSolve handles zero-copy odeBuffers, reconstructing rows and attaching matrix', async () => {
    const { wasmSolve } = await client()
    const p = wasmSolve('dummy ODE source', '{}')
    expect(FakeWorker.instances).toHaveLength(1)
    const w = FakeWorker.instances[0]

    const envelope = JSON.stringify({
      success: true,
      variables: [],
      blocks: [],
      residuals: [],
      stats: null,
      solutions: [],
      unitWarnings: [],
      error: null,
      odeTables: [
        {
          name: 'ode1',
          vars: ['t', 'x'],
          units: ['s', 'm'],
          rows: [],
          events: [],
          method: 'ode45',
          stopped: false,
          endTime: 1.0,
        },
      ],
    })
    const odeBuf = new Float64Array([0.0, 10.0, 0.5, 15.0, 1.0, 20.0])
    w.onmessage?.({
      data: {
        id: w.posted[0].id,
        ok: true,
        result: envelope,
        odeBuffers: [odeBuf],
      },
    })

    const res = await p
    expect(res.success).toBe(true)
    expect(res.odeTables).toBeDefined()
    expect(res.odeTables![0].matrix).toBe(odeBuf)
    expect(res.odeTables![0].rows).toEqual([
      [0.0, 10.0],
      [0.5, 15.0],
      [1.0, 20.0],
    ])
  })

  it('wasmSolveTable hydrates empty values and attaches matrix from zero-copy worker reply', async () => {
    const { wasmSolveTable, setWorkerPoolConcurrency } = await client()
    setWorkerPoolConcurrency(1)

    const source = 'y = 2 * x\n'
    const req = JSON.stringify({
      table: {
        variables: ['x', 'y'],
        rows: [{ x: 1 }, { x: 2 }],
      },
    })

    const p = wasmSolveTable(source, req)
    const w = FakeWorker.instances[0]

    const envelope = JSON.stringify({
      results: [
        { success: true, values: {}, error: null },
        { success: true, values: {}, error: null },
      ],
      stats: {
        converged: true,
        passes: 1,
        termination: 'completed',
        accessor: false,
        runs: 2,
        solved: 2,
        failed: 0,
        notRun: 0,
        equations: 1,
        unknowns: 1,
        iterations: 2,
        elapsedMillis: 5,
        maxResidual: 0,
      },
      variables: [{ name: 'y', value: 4 }],
      varNames: ['x', 'y'],
      numRows: 2,
      numCols: 2,
    })
    const matrix = new Float64Array([1, 2, 2, 4])
    w.onmessage?.({
      data: {
        id: w.posted[0].id,
        ok: true,
        result: envelope,
        matrix,
      },
    })

    const raw = await p
    const parsed = raw
    // Row values should be hydrated
    expect(parsed.results![0].values).toEqual({ x: 1, y: 2 })
    expect(parsed.results![1].values).toEqual({ x: 2, y: 4 })
    // The response retains the transferred matrix
    expect(raw.matrix).toBe(matrix)
  })

  it('mergeSolveTableResponses merges chunk typed array matrices into a single contiguous matrix', async () => {
    const { mergeSolveTableResponses } = await client()

    const chunk1 = {
      results: [
        { success: true, values: { x: 1, y: 2 }, error: null },
        { success: true, values: { x: 2, y: 4 }, error: null },
      ],
      matrix: new Float64Array([1, 2, 2, 4]),
      varNames: ['x', 'y'],
    }
    const chunk2 = {
      results: [
        { success: true, values: { x: 3, y: 6 }, error: null },
        { success: true, values: { x: 4, y: 8 }, error: null },
      ],
      matrix: new Float64Array([3, 6, 4, 8]),
      varNames: ['x', 'y'],
    }

    const merged = mergeSolveTableResponses([chunk1, chunk2], 4, 15)
    expect(merged.matrix).toBeDefined()
    expect(merged.matrix!.length).toBe(8)
    expect(Array.from(merged.matrix!)).toEqual([1, 2, 2, 4, 3, 6, 4, 8])
    expect(merged.varNames).toEqual(['x', 'y'])
  })
  it('aligns different chunk columns and keeps failed cells missing', async () => {
    const { mergeSolveTableResponses } = await client()
    const merged = mergeSolveTableResponses([
      { results: [{ success: false, values: {}, error: 'failed' }],
        varNames: ['x'], matrix: new Float64Array([NaN]) },
      { results: [{ success: true, values: { x: 2, y: 4 }, error: null }],
        varNames: ['y', 'x'], matrix: new Float64Array([4, 2]) },
    ], 2, 0)
    expect(merged.varNames).toEqual(['x', 'y'])
    expect(Array.from(merged.matrix!)).toEqual([NaN, NaN, 2, 4])
  })

})
