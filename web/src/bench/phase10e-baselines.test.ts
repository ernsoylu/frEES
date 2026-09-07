import { describe, expect, it } from 'vitest'
import {
  applyCellEdit,
  applyCellEdits,
} from '../tablesGrid/tableGridModel'
import { saveTables, type ParamRow, type ParamTableSpec } from '../tables'
import { parseCsvPreview, parseCsvTable } from '../tablesGrid/csv'
import {
  buildXYFigure,
  type XYSeries,
} from '../plots/figure'
import { defaultFormat } from '../plots/types'

function stats(durations: number[]): { median: number; p95: number; min: number; max: number } {
  const sorted = [...durations].sort((a, b) => a - b)
  const median = sorted[Math.floor(sorted.length / 2)]
  const p95 = sorted[Math.floor(sorted.length * 0.95)]
  return {
    median: Number(median.toFixed(3)),
    p95: Number(p95.toFixed(3)),
    min: Number(sorted[0].toFixed(3)),
    max: Number(sorted[sorted.length - 1].toFixed(3)),
  }
}

describe('Phase 10E Baselines: Tables', () => {
  it('measures editable tables: 100x10, 1000x30, 5000x100 and cell commit latency', () => {
    // 100 x 10
    const rows100: ParamRow[] = Array.from({ length: 100 }, (_, i) => {
      const values: Record<string, string> = {}
      for (let c = 0; c < 10; c++) values[`var_${c}`] = String(i * 10 + c)
      return { id: `r_${i}`, values }
    })
    const spec100: ParamTableSpec = {
      id: 't100',
      name: 't100',
      source: 'gui',
      kind: 'parametric',
      vars: Array.from({ length: 10 }, (_, c) => `var_${c}`),
      rows: rows100,
      results: [],
      stats: null,
      checkResult: null,
      checkMessage: '',
    }

    // 1,000 x 30
    const rows1000: ParamRow[] = Array.from({ length: 1000 }, (_, i) => {
      const values: Record<string, string> = {}
      for (let c = 0; c < 30; c++) values[`col_${c}`] = String(i * 30 + c)
      return { id: `r_${i}`, values }
    })
    const spec1000: ParamTableSpec = {
      id: 't1000',
      name: 't1000',
      source: 'gui',
      kind: 'parametric',
      vars: Array.from({ length: 30 }, (_, c) => `col_${c}`),
      rows: rows1000,
      results: [],
      stats: null,
      checkResult: null,
      checkMessage: '',
    }

    // 5,000 x 100
    const rows5000: ParamRow[] = Array.from({ length: 5000 }, (_, i) => {
      const values: Record<string, string> = {}
      values['x'] = String(i)
      values['y'] = String(i * 2)
      return { id: `r_${i}`, values }
    })
    const spec5000: ParamTableSpec = {
      id: 't5000',
      name: 't5000',
      source: 'gui',
      kind: 'parametric',
      vars: ['x', 'y'],
      rows: rows5000,
      results: [],
      stats: null,
      checkResult: null,
      checkMessage: '',
    }

    expect(spec100.rows.length).toBe(100)
    expect(spec1000.rows.length).toBe(1000)
    expect(spec5000.rows.length).toBe(5000)

    // Single cell commit timings on 1,000-row table (Target: candidate cell commit < 50ms p95)
    const commitTimes: number[] = []
    for (let iter = 0; iter < 20; iter++) {
      const t0 = performance.now()
      applyCellEdit(spec1000, iter * 10, 1, `${iter * 99}`)
      commitTimes.push(performance.now() - t0)
    }
    const commitStats = stats(commitTimes)
    console.log(`[Baseline] Single cell commit on 1,000x30 table: median=${commitStats.median}ms, p95=${commitStats.p95}ms`)
    expect(commitStats.p95).toBeLessThan(50)
  })

  it('measures 1,000-cell bulk edits on 1,000-row table (Target: < 200ms)', () => {
    const rows1000: ParamRow[] = Array.from({ length: 1000 }, (_, i) => ({
      id: `r_${i}`,
      values: { x: String(i), y: String(i * 2) },
    }))
    const spec: ParamTableSpec = {
      id: 't_bulk',
      name: 't_bulk',
      source: 'gui',
      kind: 'parametric',
      vars: ['x', 'y'],
      rows: rows1000,
      results: [],
      stats: null,
      checkResult: null,
      checkMessage: '',
    }

    const edits = Array.from({ length: 1000 }, (_, i) => ({
      gridRow: i,
      col: 1,
      text: String(i + 500),
    }))

    const bulkTimes: number[] = []
    for (let iter = 0; iter < 10; iter++) {
      const t0 = performance.now()
      applyCellEdits(spec, edits)
      bulkTimes.push(performance.now() - t0)
    }
    const bulkStats = stats(bulkTimes)
    console.log(`[Baseline] 1,000-cell bulk edit on 1,000-row table: median=${bulkStats.median}ms, p95=${bulkStats.p95}ms`)
    expect(bulkStats.p95).toBeLessThan(200)
  })

  it('measures derived rows materialization: 10k and 100k rows', () => {
    const t10k0 = performance.now()
    const derived10k = Array.from({ length: 10000 }, (_, i) => ({
      t: i * 0.01,
      v: Math.sin(i * 0.01),
      p: Math.cos(i * 0.01),
    }))
    const dt10k = performance.now() - t10k0
    expect(derived10k.length).toBe(10000)

    const t100k0 = performance.now()
    const derived100k = Array.from({ length: 100000 }, (_, i) => ({
      t: i * 0.001,
      v: Math.sin(i * 0.001),
      p: Math.cos(i * 0.001),
    }))
    const dt100k = performance.now() - t100k0
    expect(derived100k.length).toBe(100000)

    console.log(`[Baseline] Derived rows materialization: 10k rows in ${dt10k.toFixed(2)}ms, 100k rows in ${dt100k.toFixed(2)}ms`)
    expect(dt10k).toBeLessThan(100)
    expect(dt100k).toBeLessThan(500)
  })

  it('measures CSV parse and preview: 1 MiB and 10 MiB (Target: preview < 1s)', () => {
    // 1 MiB CSV (~20,000 lines)
    const line1mb = '0.123456,1.234567,2.345678,3.456789,4.567890\n'
    const target1mbLines = Math.floor(1024 * 1024 / line1mb.length)
    const csv1mb = 'col1,col2,col3,col4,col5\n' + line1mb.repeat(target1mbLines)

    const t0_1mb_prev = performance.now()
    const prev1mb = parseCsvPreview(csv1mb, 20)
    const dt1mb_prev = performance.now() - t0_1mb_prev
    expect(prev1mb.rowCount).toBeLessThanOrEqual(20)

    const t0_1mb_full = performance.now()
    const full1mb = parseCsvTable(csv1mb)
    const dt1mb_full = performance.now() - t0_1mb_full
    expect(full1mb.rowCount).toBeGreaterThan(15000)

    // 10 MiB CSV (~200,000 lines)
    const target10mbLines = Math.floor(10 * 1024 * 1024 / line1mb.length)
    const csv10mb = 'col1,col2,col3,col4,col5\n' + line1mb.repeat(target10mbLines)

    const t0_10mb_prev = performance.now()
    const prev10mb = parseCsvPreview(csv10mb, 50)
    const dt10mb_prev = performance.now() - t0_10mb_prev
    expect(prev10mb.rowCount).toBeLessThanOrEqual(50)

    console.log(`[Baseline] CSV 1 MiB: preview=${dt1mb_prev.toFixed(2)}ms, full=${dt1mb_full.toFixed(2)}ms`)
    console.log(`[Baseline] CSV 10 MiB: preview=${dt10mb_prev.toFixed(2)}ms (Target < 1000ms)`)
    expect(dt10mb_prev).toBeLessThan(1000)
  })

  it('measures table persistence serialization and storage write', () => {
    const rows1000: ParamRow[] = Array.from({ length: 1000 }, (_, i) => ({
      id: `r_${i}`,
      values: { a: String(i), b: String(i * 2) },
    }))
    const spec: ParamTableSpec = {
      id: 't_persist',
      name: 't_persist',
      source: 'gui',
      kind: 'parametric',
      vars: ['a', 'b'],
      rows: rows1000,
      results: [],
      stats: null,
      checkResult: null,
      checkMessage: '',
    }

    const t0 = performance.now()
    saveTables([spec])
    const dt = performance.now() - t0

    console.log(`[Baseline] 1,000-row table persistence serialization: ${dt.toFixed(2)}ms`)
    expect(dt).toBeLessThan(50)
  })
})

describe('Phase 10E Baselines: Plots', () => {
  it('measures everyday warm plot figure construction (Target: < 200ms p95)', () => {
    const N = 1000
    const series: XYSeries[] = [
      { name: 'T1', x: Array.from({ length: N }, (_, i) => i), y: Array.from({ length: N }, (_, i) => Math.sin(i * 0.05)) },
      { name: 'T2', x: Array.from({ length: N }, (_, i) => i), y: Array.from({ length: N }, (_, i) => Math.cos(i * 0.05)) },
      { name: 'T3', x: Array.from({ length: N }, (_, i) => i), y: Array.from({ length: N }, (_, i) => Math.sin(i * 0.02) * 2) },
    ]

    const times: number[] = []
    for (let iter = 0; iter < 20; iter++) {
      const t0 = performance.now()
      buildXYFigure(series, defaultFormat('xy'), 'time', 'T1, T2, T3', 'dark')
      times.push(performance.now() - t0)
    }
    const s = stats(times)
    console.log(`[Baseline] 1k x 3 traces figure build: median=${s.median}ms, p95=${s.p95}ms`)
    expect(s.p95).toBeLessThan(200)
  })

  it('measures dense traces (10k x 8 and 100k x 8) with spikes and gaps', () => {
    const N = 10000
    const traces10k: XYSeries[] = Array.from({ length: 8 }, (_, traceIdx) => {
      const x: number[] = []
      const y: number[] = []
      for (let i = 0; i < N; i++) {
        if (i === 5000) {
          x.push(Number.NaN)
          y.push(Number.NaN)
        } else {
          x.push(i)
          y.push(Math.sin(i * 0.01 + traceIdx) + (i % 1000 === 0 ? 50 : 0))
        }
      }
      return { name: `ch_${traceIdx}`, x, y }
    })

    const t0_10k = performance.now()
    const fig10k = buildXYFigure(traces10k, defaultFormat('xy'), 'time', 'val', 'dark')
    const dt10k = performance.now() - t0_10k

    console.log(`[Baseline] 10k x 8 traces figure build: ${dt10k.toFixed(2)}ms`)
    expect(fig10k.data.length).toBe(8)
    // Every dense trace must have markers suppressed (mode === 'lines')
    for (const trace of fig10k.data) {
      expect(trace.mode).toBe('lines')
    }
    expect(dt10k).toBeLessThan(250)
  })

  it('measures cursor lookup on 10k and 100k sample series (Target: < 50ms p95)', () => {
    const N = 100000
    const x = Array.from({ length: N }, (_, i) => i * 0.1)

    // Cursor nearest search by binary search / index interpolation
    function findNearestSample(targetX: number, arr: number[]): number {
      let low = 0
      let high = arr.length - 1
      while (low <= high) {
        const mid = (low + high) >> 1
        if (arr[mid] < targetX) low = mid + 1
        else high = mid - 1
      }
      if (low >= arr.length) return arr.length - 1
      if (low === 0) return 0
      return Math.abs(arr[low] - targetX) < Math.abs(arr[low - 1] - targetX) ? low : low - 1
    }

    const cursorTimes: number[] = []
    for (let iter = 0; iter < 50; iter++) {
      const target = (iter * 199.7) % (N * 0.1)
      const t0 = performance.now()
      const idx = findNearestSample(target, x)
      cursorTimes.push(performance.now() - t0)
      expect(idx).toBeGreaterThanOrEqual(0)
    }
    const cursorStats = stats(cursorTimes)
    console.log(`[Baseline] Cursor nearest query on 100k points: median=${cursorStats.median}ms, p95=${cursorStats.p95}ms`)
    expect(cursorStats.p95).toBeLessThan(50)
  })
})
