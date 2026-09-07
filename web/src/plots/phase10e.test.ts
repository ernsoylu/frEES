import { describe, expect, it, vi, beforeEach } from 'vitest'
import {
  buildXYFigure,
  decimateMonotonicSeries,
  isMonotonicX,
  type XYSeries,
} from './figure'
import { defaultFormat } from './types'
import { applyCellEdits } from '../tablesGrid/tableGridModel'
import {
  type ParamTableSpec,
  type FunctionTableSpec,
  type ParamRow,
} from '../tables'
import { parseCsvPreview, splitCsvRows, parseCsvTable } from '../tablesGrid/csv'
import { clearThermoCache, getPropertyDiagram } from '../api'
import { wasmPropertyDiagram } from '../wasm/engineClient'

vi.mock('../wasm/engineClient', () => ({
  wasmFluids: vi.fn(),
  wasmPropertyDiagram: vi.fn(),
  wasmPsychrometricChart: vi.fn(),
  wasmSolve: vi.fn(),
  wasmCheck: vi.fn(),
  wasmSolveTable: vi.fn(),
}))

const diagramMock = vi.mocked(wasmPropertyDiagram)

describe('Phase 10E: Plot performance, dense marker suppression, and decimation', () => {
  it('detects monotonic vs non-monotonic series correctly', () => {
    expect(isMonotonicX([1, 2, 3, 4, 5])).toBe(true)
    expect(isMonotonicX([1, 1, 2, 2, 3])).toBe(true)
    expect(isMonotonicX([1, null, 2, 3, null, 4])).toBe(true)
    // Non-monotonic (cyclic, loops, unordered)
    expect(isMonotonicX([1, 2, 1, 2])).toBe(false)
    expect(isMonotonicX([5, 4, 3, 2, 1])).toBe(false)
    expect(isMonotonicX([0, 1, 0, -1, 0])).toBe(false)
  })

  it('never decimates non-monotonic, cyclic, or short series', () => {
    const cyclicX = [0, 1, 2, 1, 0, -1, -2, -1, 0]
    const cyclicY = [0, 1, 0, -1, 0, 1, 0, -1, 0]
    const result = decimateMonotonicSeries(cyclicX, cyclicY, undefined, 4)
    expect(result.x).toEqual(cyclicX)
    expect(result.y).toEqual(cyclicY)

    // Series below maxPoints is left intact
    const shortX = [1, 2, 3]
    const shortY = [10, 20, 30]
    const shortResult = decimateMonotonicSeries(shortX, shortY, undefined, 100)
    expect(shortResult.x).toEqual(shortX)
    expect(shortResult.y).toEqual(shortY)
  })

  it('preserves exact local extrema, endpoints, and gaps in monotonic series', () => {
    const N = 10000
    const x: (number | null)[] = []
    const y: (number | null)[] = []
    const sampleIds: (string | undefined)[] = []

    for (let i = 0; i < N; i++) {
      if (i === 5000) {
        // Gap
        x.push(null)
        y.push(null)
        sampleIds.push(undefined)
      } else {
        x.push(i)
        // High frequency oscillation with huge spikes
        const val = i === 2500 ? 99999 : i === 7500 ? -99999 : Math.sin(i * 0.1)
        y.push(val)
        sampleIds.push(`sample_${i}`)
      }
    }

    const decimated = decimateMonotonicSeries(x, y, sampleIds, 500)
    // Decimated length should be drastically smaller than 10,000
    expect(decimated.x.length).toBeLessThanOrEqual(600)
    expect(decimated.x.length).toBeGreaterThan(100)

    // Extrema must be preserved exactly
    expect(decimated.y).toContain(99999)
    expect(decimated.y).toContain(-99999)

    // Endpoints of segments must be preserved
    expect(decimated.x).toContain(0)
    expect(decimated.x).toContain(4999)
    expect(decimated.x).toContain(5001)
    expect(decimated.x).toContain(9999)

    // Gap (null) must be preserved
    expect(decimated.x).toContain(null)
    expect(decimated.y).toContain(null)
  })

  it('suppresses markers on dense line series (> 300 points) unless explicitly configured', () => {
    const N = 500
    const denseSeries: XYSeries[] = [
      {
        name: 'T',
        x: Array.from({ length: N }, (_, i) => i),
        y: Array.from({ length: N }, (_, i) => i * 2),
      },
    ]

    // Without explicit marker style -> mode is 'lines'
    const fig1 = buildXYFigure(denseSeries, defaultFormat('xy'), 'time', 'T', 'dark')
    expect(fig1.data[0].mode).toBe('lines')

    // With explicit marker style -> mode is 'lines+markers'
    const fig2 = buildXYFigure(
      denseSeries,
      { ...defaultFormat('xy'), traceStyles: { T: { markerSymbol: 'diamond' } } },
      'time',
      'T',
      'dark',
    )
    expect(fig2.data[0].mode).toBe('lines+markers')

    // Short series (<= 300 points) keeps 'lines+markers'
    const shortSeries: XYSeries[] = [
      {
        name: 'T',
        x: [1, 2, 3],
        y: [10, 20, 30],
      },
    ]
    const fig3 = buildXYFigure(shortSeries, defaultFormat('xy'), 'time', 'T', 'dark')
    expect(fig3.data[0].mode).toBe('lines+markers')
  })
})

describe('Phase 10E: Batch edits and table commit performance', () => {
  it('applies 1,000 cell edits to a 1,000-row table in under 50ms with correct data', () => {
    const rowCount = 1000
    const rows: ParamRow[] = Array.from({ length: rowCount }, (_, i) => ({
      id: `row_${i + 1}`,
      values: { a: String(i), b: String(i * 2) },
    }))
    const spec: ParamTableSpec = {
      id: 'table1',
      name: 'table1',
      source: 'gui',
      kind: 'parametric',
      vars: ['a', 'b'],
      rows,
      results: [],
      stats: null,
      checkResult: null,
      checkMessage: '',
    }

    const edits = Array.from({ length: 1000 }, (_, i) => ({
      gridRow: i,
      col: 1, // 'a'
      text: `${i + 100}`,
    }))

    const t0 = performance.now()
    const result = applyCellEdits(spec, edits)
    const dt = performance.now() - t0

    expect(dt).toBeLessThan(50) // Candidate target < 50ms
    expect(result.changed).toBe(true)
    const updated = result.spec as ParamTableSpec
    expect(updated.rows[0].values['a']).toBe('100')
    expect(updated.rows[500].values['a']).toBe('600')
    expect(updated.rows[999].values['a']).toBe('1099')
    // Untouched column 'b' preserved
    expect(updated.rows[0].values['b']).toBe('0')
    expect(updated.rows[500].values['b']).toBe('1000')
  })

  it('applies batch edits to function tables accurately', () => {
    const rows = Array.from({ length: 100 }, (_, i) => ({
      x: String(i),
      ys: [String(i * 10)],
    }))
    const spec: FunctionTableSpec = {
      id: 'func1',
      name: 'func1',
      source: 'gui',
      kind: 'function',
      argName: 'x',
      paramName: '',
      xLog: false,
      yLog: false,
      columns: ['y'],
      rows,
      is1D: true,
    }

    const edits = [
      { gridRow: 1, col: 0, text: '10.5' }, // x of row 0
      { gridRow: 2, col: 1, text: '25.0' }, // y of row 1
    ]

    const result = applyCellEdits(spec, edits)
    expect(result.changed).toBe(true)
    const updated = result.spec as FunctionTableSpec
    expect(updated.rows[0].x).toBe('10.5')
    expect(updated.rows[1].ys[0]).toBe('25.0')
  })
})

describe('Phase 10E: CSV parsing and fast preview', () => {
  it('parses preview rows without processing entire file', () => {
    // Generate a CSV with 500 rows
    const lines = ['time,voltage,current']
    for (let i = 0; i < 500; i++) {
      lines.push(`${i * 0.1},${Math.sin(i)},${Math.cos(i)}`)
    }
    const csvContent = lines.join('\n')

    const preview = parseCsvPreview(csvContent, 20)
    expect(preview.rowCount).toBeLessThanOrEqual(20)
    expect(preview.columns.map((c) => c.name)).toEqual(['time', 'voltage', 'current'])
    expect(preview.columns[0].values.length).toBeLessThanOrEqual(20)

    const full = parseCsvTable(csvContent)
    expect(full.rowCount).toBe(500)
  })

  it('correctly handles quoted fields and commas in slice-based parser', () => {
    const csv = 'Name,"Quoted, Value",Score\n"Alpha","Beta, Gamma",100\n"Delta","Epsilon",200'
    const rows = splitCsvRows(csv, ',')
    expect(rows).toEqual([
      ['Name', 'Quoted, Value', 'Score'],
      ['Alpha', 'Beta, Gamma', '100'],
      ['Delta', 'Epsilon', '200'],
    ])
  })
})

describe('Phase 10E: Thermo diagram caching and in-flight deduplication', () => {
  beforeEach(() => {
    clearThermoCache()
    vi.clearAllMocks()
  })

  const dummyDiagram = {
    fluid: 'Water',
    kind: 'TS',
    xProperty: 's',
    yProperty: 'T',
    xLog: false,
    yLog: true,
    dome: [],
    isolines: [],
    markers: [],
  }

  it('deduplicates concurrent requests and reuses cached diagram responses', async () => {
    diagramMock.mockResolvedValue(dummyDiagram)

    // Concurrent requests share one in-flight promise
    const [d1, d2, d3] = await Promise.all([
      getPropertyDiagram('Water', 'T-s'),
      getPropertyDiagram('Water', 'T-s'),
      getPropertyDiagram('water', 't-s'), // case-insensitive hit
    ])
    expect(d1).toEqual(dummyDiagram)
    expect(d2).toEqual(dummyDiagram)
    expect(d3).toEqual(dummyDiagram)
    expect(diagramMock).toHaveBeenCalledTimes(1)

    // Subsequent call hits cache
    const d4 = await getPropertyDiagram('WATER', 'T-S')
    expect(d4).toEqual(dummyDiagram)
    expect(diagramMock).toHaveBeenCalledTimes(1)
  })

  it('evicts failed diagram requests so retries invoke the backend again', async () => {
    diagramMock.mockRejectedValueOnce(new Error('backend timeout'))
    await expect(getPropertyDiagram('Water', 'T-s')).rejects.toThrow('backend timeout')

    diagramMock.mockResolvedValueOnce(dummyDiagram)
    const retryResult = await getPropertyDiagram('Water', 'T-s')
    expect(retryResult).toEqual(dummyDiagram)
    expect(diagramMock).toHaveBeenCalledTimes(2)
  })
})
