// DTO-parity assertions for the property-plot seam: what the Rust boundary
// emits for GET /api/plot/fluids, POST /api/plot/propplot and
// POST /api/plot/psychart versus what api.ts promises its call sites.
//
// The three contracts under test:
//
//  1. getFluids() RESOLVES, always. Its three call sites (App.tsx, PlotTab,
//     HelpPage) consume it at boot with bare .then(), so a rejection would be
//     an unhandled promise. It also honours the engine's `available` flag: an
//     engine with no real-fluid backend reports an empty list, exactly like the
//     Java PlotController's `CoolProp.isAvailable() ? plotFluids() : List.of()`.
//  2. getPropertyDiagram()/getPsychrometricChart() REJECT with the engine's own
//     message — PlotCard has a catch and shows it in the card's error state, so
//     a silent empty chart would be a lie.
//  3. gaps survive: a point the property backend declined arrives as `null` in
//     x/y and must stay null (Plotly reads it as a line break). Nothing may
//     coerce it to 0.
import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('./wasm/engineClient', () => ({
  wasmSolveTable: vi.fn(),
  wasmMonteCarlo: vi.fn(),
  wasmOptimize: vi.fn(),
  wasmOptimizeMulti: vi.fn(),
  wasmCurveFit: vi.fn(),
  wasmParameterFit: vi.fn(),
  wasmPidTune: vi.fn(),
  wasmExtractPlant: vi.fn(),
  wasmFluids: vi.fn(),
  wasmPropertyDiagram: vi.fn(),
  wasmPsychrometricChart: vi.fn(),
}))

import { clearThermoCache, getFluids, getPropertyDiagram, getPsychrometricChart } from './api'
import {
  wasmFluids,
  wasmPropertyDiagram,
  wasmPsychrometricChart,
} from './wasm/engineClient'

const fluidsMock = vi.mocked(wasmFluids)
const diagramMock = vi.mocked(wasmPropertyDiagram)
const chartMock = vi.mocked(wasmPsychrometricChart)

// ── Fixtures: verbatim boundary output (crates/frees) ────────────────

/** `fluids()` with a real-fluid backend installed — the Java plotFluids() list. */
const FLUIDS_AVAILABLE = {
  available: true,
  fluids: ['Air', 'Ammonia', 'CO2', 'R134a', 'Water'],
  backend: '(P,h) split tables [Water, R134a]',
}

/** `fluids()` in a build with no property backend — this repo's current state. */
const FLUIDS_UNAVAILABLE = {
  available: false,
  fluids: [] as string[],
  backend: 'none (no real-fluid property backend installed)',
}

/** A T-s diagram with one gap in the dome, as `props::diagrams` emits it. */
const DIAGRAM = {
  fluid: 'Water',
  kind: 'TS',
  xProperty: 's',
  yProperty: 'T',
  xLog: false,
  yLog: true,
  dome: [
    {
      family: 'dome',
      label: 'Saturation',
      x: [1.0, null, 3.0],
      y: [300.0, null, 500.0],
    },
  ],
  isolines: [
    { family: 'quality', label: 'x = 0.5', x: [1.5], y: [400.0] },
    { family: 'isobar', label: '100 kPa', x: [2.0], y: [372.0] },
  ],
  markers: [{ label: 'Critical point', x: 4.4, y: 647.1 }],
}

const CHART = {
  pressure: 101325,
  tMin: 273.15,
  tMax: 323.15,
  curves: [
    { family: 'saturation', label: 'Saturation', x: [280.0], y: [0.006] },
    { family: 'rh', label: 'φ = 50%', x: [280.0, null], y: [0.003, null] },
  ],
}

beforeEach(() => {
  vi.resetAllMocks()
  clearThermoCache()
})

describe('getFluids', () => {
  it('returns the engine list when a property backend is available', async () => {
    fluidsMock.mockResolvedValue(FLUIDS_AVAILABLE)
    await expect(getFluids()).resolves.toEqual([
      'Air',
      'Ammonia',
      'CO2',
      'R134a',
      'Water',
    ])
  })

  it('returns an empty list when the engine has no property backend', async () => {
    fluidsMock.mockResolvedValue(FLUIDS_UNAVAILABLE)
    await expect(getFluids()).resolves.toEqual([])
  })

  it('never offers fluids the engine says are unavailable', async () => {
    // A backend that reports available:false but still names fluids must not
    // leak them into a dropdown whose every entry would fail to plot.
    fluidsMock.mockResolvedValue({ ...FLUIDS_AVAILABLE, available: false })
    await expect(getFluids()).resolves.toEqual([])
  })

  it('resolves to an empty list when the engine worker fails', async () => {
    fluidsMock.mockRejectedValue(new Error('worker died'))
    await expect(getFluids()).resolves.toEqual([])
  })
})

describe('getPropertyDiagram', () => {
  it('passes the DiagramResponse through with its gaps intact', async () => {
    diagramMock.mockResolvedValue(DIAGRAM)
    const diagram = await getPropertyDiagram('Water', 'T-s')
    expect(diagramMock).toHaveBeenCalledWith('Water', 'T-s')
    expect(diagram.fluid).toBe('Water')
    expect(diagram.kind).toBe('TS')
    expect(diagram.xProperty).toBe('s')
    expect(diagram.yProperty).toBe('T')
    expect(diagram.yLog).toBe(true)
    // The gap is preserved as null — never coerced to a number.
    expect(diagram.dome[0].x).toEqual([1.0, null, 3.0])
    expect(diagram.dome[0].y[1]).toBeNull()
    expect(diagram.dome[0].x.length).toBe(diagram.dome[0].y.length)
    expect(diagram.isolines.map((c) => c.family)).toEqual([
      'quality',
      'isobar',
    ])
    expect(diagram.markers[0]).toEqual({
      label: 'Critical point',
      x: 4.4,
      y: 647.1,
    })
  })

  it('rejects with the engine message so the card can show it', async () => {
    diagramMock.mockRejectedValue(
      new Error(
        'property error: Ttriple of Water needs a real-fluid property backend and none is installed.',
      ),
    )
    await expect(getPropertyDiagram('Water', 'T-s')).rejects.toThrow(
      /real-fluid property backend/,
    )
  })
})

describe('getPsychrometricChart', () => {
  it('passes the PsychartResponse through and forwards the window', async () => {
    chartMock.mockResolvedValue(CHART)
    const chart = await getPsychrometricChart(101325, 273.15, 323.15)
    expect(chartMock).toHaveBeenCalledWith(101325, 273.15, 323.15)
    expect(chart.pressure).toBe(101325)
    expect(chart.tMin).toBe(273.15)
    expect(chart.tMax).toBe(323.15)
    expect(chart.curves.map((c) => c.family)).toEqual(['saturation', 'rh'])
    expect(chart.curves[1].x[1]).toBeNull()
  })

  it('rejects on a window the engine refuses', async () => {
    chartMock.mockRejectedValue(
      new Error(
        'Psychrometric chart needs pressure > 1 kPa and tMax > tMin (SI units)',
      ),
    )
    await expect(getPsychrometricChart(500, 273.15, 323.15)).rejects.toThrow(
      /pressure > 1 kPa/,
    )
  })

  it('reuses in-flight requests and caches responses for identical queries', async () => {
    chartMock.mockResolvedValue(CHART)
    // Concurrent calls share in-flight promise
    const [c1, c2] = await Promise.all([
      getPsychrometricChart(101325, 273.15, 323.15),
      getPsychrometricChart(101325, 273.15, 323.15),
    ])
    expect(c1).toEqual(CHART)
    expect(c2).toEqual(CHART)
    expect(chartMock).toHaveBeenCalledTimes(1)

    // Subsequent call hits cache
    const c3 = await getPsychrometricChart(101325, 273.15, 323.15)
    expect(c3).toEqual(CHART)
    expect(chartMock).toHaveBeenCalledTimes(1)
  })

  it('evicts failed requests from cache so retrying invokes the engine again', async () => {
    chartMock.mockRejectedValueOnce(new Error('transient error'))
    await expect(getPsychrometricChart(101325, 273.15, 323.15)).rejects.toThrow('transient error')

    chartMock.mockResolvedValueOnce(CHART)
    const retry = await getPsychrometricChart(101325, 273.15, 323.15)
    expect(retry).toEqual(CHART)
    expect(chartMock).toHaveBeenCalledTimes(2)
  })
})
