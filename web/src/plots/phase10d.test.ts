import { describe, expect, it } from 'vitest'
import { buildFigure, computeTraceStats, formatPlotValue } from './PlotCard'
import { buildXYFigure, type XYSeries } from './figure'
import { resolvePlotSource } from './sources'
import { defaultFormat, newPlotSpec, PlotSpec } from './types'
import { FunctionTableSpec, mergeCodeTables, ParamTableSpec } from '../tables'

describe('Phase 10D: Plot inspection, statistics, and cursor measurement', () => {
  describe('computeTraceStats', () => {
    it('computes count, valid, missing, min, max, mean, and sum from raw data', () => {
      const trace = {
        name: 'Pressure',
        x: [0, 1, 2, 3, 4],
        y: [10, 20, Number.NaN, 40, 50],
      }
      const stats = computeTraceStats(trace)
      expect(stats).not.toBeNull()
      expect(stats!.traceName).toBe('Pressure')
      expect(stats!.count).toBe(5)
      expect(stats!.valid).toBe(4)
      expect(stats!.missing).toBe(1)
      expect(stats!.minX).toBe(0)
      expect(stats!.maxX).toBe(4)
      expect(stats!.minY).toBe(10)
      expect(stats!.maxY).toBe(50)
      expect(stats!.sumY).toBe(120)
      expect(stats!.meanY).toBe(30)
    })

    it('handles traces with all invalid/NaN points gracefully', () => {
      const trace = {
        name: 'Empty',
        x: [Number.NaN, Number.NaN],
        y: [Number.NaN, Number.NaN],
      }
      const stats = computeTraceStats(trace)
      expect(stats).not.toBeNull()
      expect(stats!.count).toBe(2)
      expect(stats!.valid).toBe(0)
      expect(stats!.missing).toBe(2)
      expect(Number.isNaN(stats!.meanY)).toBe(true)
    })

    it('returns null for non-array traces', () => {
      expect(computeTraceStats(null)).toBeNull()
      expect(computeTraceStats({})).toBeNull()
    })
  })

  describe('formatPlotValue', () => {
    it('formats standard floats with local formatting', () => {
      expect(formatPlotValue(12.3456)).toBe('12.3456')
      expect(formatPlotValue(0)).toBe('0')
    })

    it('formats large numbers and very small non-zero numbers in exponential notation', () => {
      expect(formatPlotValue(1000000)).toBe('1.0000e+6')
      expect(formatPlotValue(0.0000123)).toBe('1.2300e-5')
    })

    it('returns a dash for null, undefined, or non-finite values', () => {
      expect(formatPlotValue(null)).toBe('—')
      expect(formatPlotValue(undefined)).toBe('—')
      expect(formatPlotValue(Number.NaN)).toBe('—')
      expect(formatPlotValue(Number.POSITIVE_INFINITY)).toBe('—')
    })
  })

  describe('cursor measurement delta and slope', () => {
    it('calculates slope correctly between two points', () => {
      const c1 = { x: 2, y: 10 }
      const c2 = { x: 6, y: 30 }
      const dx = c2.x - c1.x
      const dy = c2.y - c1.y
      const slope = dx !== 0 ? dy / dx : null
      expect(dx).toBe(4)
      expect(dy).toBe(20)
      expect(slope).toBe(5)
    })

    it('handles vertical slope when dx is zero', () => {
      const c1 = { x: 5, y: 10 }
      const c2 = { x: 5, y: 25 }
      const dx = c2.x - c1.x
      const dy = c2.y - c1.y
      const slope = dx !== 0 ? dy / dx : null
      expect(dx).toBe(0)
      expect(slope).toBeNull()
    })
  })

  describe('per-trace styles and reference annotations in buildXYFigure', () => {
    it('applies dash styles and marker symbols from format.traceStyles', () => {
      const series: XYSeries[] = [
        {
          name: 'temp',
          x: [1, 2, 3],
          y: [10, 20, 30],
        },
      ]
      const format = {
        ...defaultFormat('xy'),
        traceStyles: {
          temp: {
            dash: 'dash' as const,
            markerSymbol: 'square',
          },
        },
      }
      const fig = buildXYFigure(series, format, 'X', 'Y', 'dark', { chartType: 'line', xVar: 'x', yVars: ['temp'] })
      expect(fig.data).toHaveLength(1)
      const trace = fig.data[0] as { line?: { dash?: string }; marker?: { symbol?: string } }
      expect(trace.line?.dash).toBe('dash')
      expect(trace.marker?.symbol).toBe('square')
    })

    it('inserts reference threshold lines (hline/vline) into layout.shapes and layout.annotations', () => {
      const series: XYSeries[] = [
        {
          name: 'temp',
          x: [1, 2, 3],
          y: [10, 20, 30],
        },
      ]
      const format = {
        ...defaultFormat('xy'),
        annotations: [
          {
            id: 'ann-1',
            type: 'hline' as const,
            value: 25,
            text: 'Max Temperature',
            color: '#ff6b6b',
            dash: 'dot' as const,
          },
          {
            id: 'ann-2',
            type: 'vline' as const,
            value: 2,
            text: 'Trip Time',
            color: '#4dabf7',
          },
        ],
      }
      const fig = buildXYFigure(series, format, 'X', 'Y', 'dark', { chartType: 'line', xVar: 'x', yVars: ['temp'] })
      expect(fig.layout.shapes?.length).toBe(2)
      expect(fig.layout.annotations?.length).toBe(2)

      const hline = fig.layout.shapes?.[0] as { type?: string; y0?: number; y1?: number; line?: { dash?: string } }
      expect(hline.type).toBe('line')
      expect(hline.y0).toBe(25)
      expect(hline.y1).toBe(25)
      expect(hline.line?.dash).toBe('dot')

      const vline = fig.layout.shapes?.[1] as { type?: string; x0?: number; x1?: number }
      expect(vline.type).toBe('line')
      expect(vline.x0).toBe(2)
      expect(vline.x1).toBe(2)

      expect(fig.layout.annotations?.[0].text).toBe('Max Temperature')
      expect(fig.layout.annotations?.[1].text).toBe('Trip Time')
    })
  })

  describe('resolvePlotSource with function and parametric tables', () => {
    it('resolves source for a function table matching argName and columns', () => {
      const fnTable: FunctionTableSpec = {
        id: 'fn-1',
        kind: 'function',
        name: 'lookup1',
        argName: 'Re',
        paramName: '',
        columns: ['Cd'],
        xLog: false,
        yLog: false,
        rows: [{ x: '100', ys: ['0.5'] }],
      }
      const plot: PlotSpec = {
        ...newPlotSpec('xy', 'Drag'),
        xy: { xVar: 'Re', yVars: ['Cd'] },
      }
      const src = resolvePlotSource(plot, [fnTable], [])
      expect(src).toEqual({ kind: 'table', tableId: 'fn-1', data: 'inputs' })
    })

    it('resolves source for a parametric table matching vars', () => {
      const paramTable: ParamTableSpec = {
        id: 'param-1',
        kind: 'parametric',
        name: 'Sweep',
        vars: ['time', 'speed'],
        rows: [{ id: 'r1', values: { time: '1', speed: '10' } }],
        results: [{ success: true, values: { time: 1, speed: 10 }, error: null }],
        stats: null,
        checkResult: null,
        checkMessage: '',
      }
      const plot: PlotSpec = {
        ...newPlotSpec('xy', 'Speed vs Time'),
        xy: { xVar: 'time', yVars: ['speed'] },
      }
      const src = resolvePlotSource(plot, [paramTable], [])
      expect(src).toEqual({ kind: 'table', tableId: 'param-1', data: 'solved' })
    })

    it('resolves source for ODE table with dot vs dollar matching', () => {
      const odeTable: ParamTableSpec = {
        id: 'code-ode-cool',
        kind: 'parametric',
        name: 'cool',
        vars: ['time', 'bp$soc', 'bp$t'],
        rows: [{ id: 'r1', values: { time: '0', 'bp$soc': '0.9', 'bp$t': '306' } }],
        results: [],
        stats: null,
        checkResult: null,
        checkMessage: '',
        source: 'code',
        origin: 'ode',
      }
      const plot: PlotSpec = {
        ...newPlotSpec('xy', 'SOC vs Time'),
        xy: { xVar: 'time', yVars: ['bp.soc'] },
      }
      const src = resolvePlotSource(plot, [odeTable], [])
      expect(src).toEqual({ kind: 'table', tableId: 'code-ode-cool', data: 'inputs' })
    })
  })

  describe('mergeCodeTables ODE preservation on check', () => {
    it('preserves existing code ODE tables when odeDtos is undefined (check response)', () => {
      const existingOde: ParamTableSpec = {
        id: 'code-ode-cooling',
        kind: 'parametric',
        name: 'cooling',
        vars: ['time', 'temp'],
        rows: [{ id: 'r1', values: { time: '0', temp: '95' } }],
        results: [],
        stats: null,
        checkResult: null,
        checkMessage: '',
        source: 'code',
        origin: 'ode',
      }
      const merged = mergeCodeTables([existingOde], [], [])
      expect(merged).toHaveLength(1)
      expect(merged[0].id).toBe('code-ode-cooling')
      expect((merged[0] as ParamTableSpec).origin).toBe('ode')
    })
  })

  describe('ODE trajectory plotting with component dot/dollar naming in buildFigure', () => {
    it('renders valid points when plot references dot name and row values use dollar name', () => {
      const spec: PlotSpec = {
        ...newPlotSpec('xy', 'X-Y 1'),
        source: { kind: 'table', tableId: 'code-ode-cool', data: 'inputs' },
        xy: { xVar: 'time', yVars: ['bp.soc'] },
      }
      const inputs = {
        states: { indices: [], columns: [], values: {} },
        tableRows: [
          { id: 'r1', values: { time: '0', 'bp$soc': '0.9' } },
          { id: 'r2', values: { time: '10', 'bp$soc': '0.8986' } },
        ],
        tableResults: [],
        variables: [],
        tableUnits: { 'bp$soc': '%' },
        theme: 'dark' as const,
      }
      const fig = buildFigure(spec, inputs)
      expect(fig).not.toBeNull()
      expect(fig!.data).toHaveLength(1)
      const trace = fig!.data[0] as { x: number[]; y: number[]; name: string }
      expect(trace.name).toBe('bp.soc')
      expect(trace.x).toEqual([0, 10])
      expect(trace.y).toEqual([0.9, 0.8986])
      expect(fig!.layout.yaxis?.title).toEqual({ text: 'bp.soc [%]' })
    })

    it('renders valid points even if spec.source.data is solved on an ODE table with empty tableResults', () => {
      const spec: PlotSpec = {
        ...newPlotSpec('xy', 'X-Y 1'),
        source: { kind: 'table', tableId: 'code-ode-cool', data: 'solved' },
        xy: { xVar: 'time', yVars: ['bp.soc'] },
      }
      const inputs = {
        states: { indices: [], columns: [], values: {} },
        tableRows: [
          { id: 'r1', values: { time: '0', 'bp$soc': '0.9' } },
          { id: 'r2', values: { time: '10', 'bp$soc': '0.8986' } },
        ],
        tableResults: [],
        variables: [],
        theme: 'dark' as const,
      }
      const fig = buildFigure(spec, inputs)
      expect(fig).not.toBeNull()
      const trace = fig!.data[0] as { x: number[]; y: number[] }
      expect(trace.x).toEqual([0, 10])
      expect(trace.y).toEqual([0.9, 0.8986])
    })
  })
})
