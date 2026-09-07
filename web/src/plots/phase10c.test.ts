import { describe, expect, it } from 'vitest'
import {
  buildPropertyFigure,
  buildXYFigure,
  buildBodeFigure,
} from './figure'
import { buildFigure, type FigureInputs } from './PlotCard'
import { validatePlotFormat } from './PlotConfigModal'
import { prepareFigureForExport } from './exportPlot'
import { defaultFormat, newPlotSpec } from './types'
import { isOffsetUnit } from './units'
import type { DiagramResponse, StateTableDto, VariableResult } from '../api'
import type { StateTable } from './stateTable'

describe('Phase 10C: Units and format validation', () => {
  it('detects offset units correctly', () => {
    expect(isOffsetUnit('T', '°C')).toBe(true)
    expect(isOffsetUnit('T', '°F')).toBe(true)
    expect(isOffsetUnit('T', 'K')).toBe(false)
    expect(isOffsetUnit('T', 'R')).toBe(false)
    expect(isOffsetUnit('P', 'psig')).toBe(true)
    expect(isOffsetUnit('P', 'kPa')).toBe(false)
    expect(isOffsetUnit('P', 'bar')).toBe(false)
    expect(isOffsetUnit('h', 'kJ/kg')).toBe(false)
  })

  it('rejects nonpositive bounds and offset units on log axes in validatePlotFormat', () => {
    const spec = newPlotSpec('property', 'test')
    spec.property = {
      diagram: 'P-h',
      fluid: 'Water',
      quality: false,
      isolines: false,
      overlayStates: false,
      connectStates: false,
      closeCycle: false,
      stateTable: null,
    }
    spec.format.yLog = true // P axis
    spec.format.yMin = -10
    spec.format.yMax = 100
    spec.format.yUnit = 'psig' // offset unit on log axis

    const errors = validatePlotFormat(spec)
    expect(errors.yMin).toContain('greater than zero')
    expect(errors.yUnit).toContain('offset units')

    // Fix bounds and unit to absolute
    spec.format.yMin = 10
    spec.format.yUnit = 'kPa'
    const cleanErrors = validatePlotFormat(spec)
    expect(cleanErrors.yMin).toBeUndefined()
    expect(cleanErrors.yUnit).toBeUndefined()
  })

  it('validates min < max order', () => {
    const spec = newPlotSpec('xy', 'test')
    spec.format.xMin = 100
    spec.format.xMax = 50
    const errors = validatePlotFormat(spec)
    expect(errors.xMin).toContain('less than max')
  })

  it('skips axis bound checks for pie charts', () => {
    const spec = newPlotSpec('xy', 'pie-test')
    spec.xy.chartType = 'pie'
    spec.format.xMin = -10
    spec.format.xMax = -20
    const errors = validatePlotFormat(spec)
    expect(Object.keys(errors).length).toBe(0)
  })
})

describe('Phase 10C: Markers and stable trace identities', () => {
  const dummyDiagram: DiagramResponse = {
    fluid: 'Water',
    kind: 'ph',
    xProperty: 'h',
    yProperty: 'P',
    xLog: false,
    yLog: true,
    dome: [],
    isolines: [],
    markers: [{ label: 'Critical point', x: 2000000, y: 22064000 }],
  }

  const dummyStates: StateTable = {
    indices: [1, 2],
    columns: ['h', 'P'],
    values: {
      1: { h: 100000, P: 100000 },
      2: { h: 500000, P: 100000 },
    },
  }

  it('renders critical markers with diamond symbol and uid', () => {
    const fig = buildPropertyFigure(
      dummyDiagram,
      {
        diagram: 'ph',
        fluid: 'Water',
        quality: false,
        isolines: false,
        overlayStates: true,
        connectStates: true,
        closeCycle: false,
        stateTable: null,
      },
      defaultFormat('property'),
      dummyStates,
      'dark',
      undefined,
      42,
    )

    const markerTrace = fig.data.find((t) => t.uid === 'critical_markers')
    expect(markerTrace).toBeDefined()
    expect(markerTrace?.name).toBe('Critical point')
    expect(markerTrace?.text).toEqual(['Critical point'])

    const connTrace = fig.data.find((t) => t.uid === 'schematic_connections')
    expect(connTrace).toBeDefined()
    expect(connTrace?.name).toBe('Schematic Connections')

    // Revision is reflected in uirevision
    expect(fig.layout.uirevision).toContain('42')
  })

  it('assigns stable uid to XY series across chart types', () => {
    const series = [{ name: 'power', x: [1, 2], y: [10, 20] }]
    const lineFig = buildXYFigure(series, defaultFormat('xy'), 'time', 'power', 'dark')
    expect(lineFig.data[0].uid).toBe('power')

    const barFig = buildXYFigure(series, defaultFormat('xy'), 'time', 'power', 'dark', { chartType: 'bar', xVar: 'time', yVars: ['power'] })
    expect(barFig.data[0].uid).toBe('power')

    const histFig = buildXYFigure(series, defaultFormat('xy'), '', 'power', 'dark', { chartType: 'histogram', xVar: null, yVars: ['power'] })
    expect(histFig.data[0].uid).toBe('power')
  })

  it('sets uirevision on Bode figures', () => {
    const fig = buildBodeFigure([1, 10], [0, -20], [0, -90], defaultFormat('bode'), 'dark', 7)
    expect(fig.layout.uirevision).toBe('bode_7')
  })
})

describe('Phase 10C: Circuit binding and missing state sources', () => {
  const stateTableDefs: StateTableDto[] = [
    {
      name: 'CircuitA',
      fluid: 'Water',
      variables: ['h_a_1', 'P_a_1'],
    },
    {
      name: 'CircuitB',
      fluid: 'R134a',
      variables: ['h_b_1', 'P_b_1'],
    },
  ]

  const variables: VariableResult[] = [
    { name: 'h_a_1', value: 100000, units: 'J/kg' },
    { name: 'P_a_1', value: 100000, units: 'Pa' },
    { name: 'h_b_1', value: 200000, units: 'J/kg' },
    { name: 'P_b_1', value: 500000, units: 'Pa' },
  ]

  const baseInputs: FigureInputs = {
    states: { indices: [1], columns: ['h', 'P'], values: { 1: { h: 100000, P: 100000 } } },
    cyclePath: [{ h: 100000, P: 100000 }, { h: 500000, P: 100000 }],
    tableRows: [],
    tableResults: [],
    variables,
    stateTableDefs,
    diagram: {
      fluid: 'Water',
      kind: 'ph',
      xProperty: 'h',
      yProperty: 'P',
      xLog: false,
      yLog: true,
      dome: [],
      isolines: [],
      markers: [],
    },
    psychart: null,
    theme: 'dark',
  }

  it('does not fall back to all detected states when a named circuit is missing', () => {
    const spec = newPlotSpec('property', 'circuit-test')
    spec.property = {
      diagram: 'ph',
      fluid: 'Water',
      quality: false,
      isolines: false,
      overlayStates: true,
      connectStates: true,
      closeCycle: false,
      stateTable: 'NonExistentCircuit',
    }

    const fig = buildFigure(spec, baseInputs)!
    expect(fig).toBeDefined()
    // Since NonExistentCircuit does not exist, states trace has 0 points
    const statesTrace = fig.data.find((t) => t.uid === 'states')
    expect(statesTrace).toBeUndefined()
  })

  it('suppresses cyclePath when circuit fluid does not match diagram fluid', () => {
    const spec = newPlotSpec('property', 'fluid-mismatch')
    spec.property = {
      diagram: 'ph',
      fluid: 'Water',
      quality: false,
      isolines: false,
      overlayStates: true,
      connectStates: true,
      closeCycle: false,
      stateTable: 'CircuitB', // R134a on a Water diagram!
    }

    const fig = buildFigure(spec, baseInputs)!
    const cyclePathTrace = fig.data.find((t) => t.uid === 'cycle_path')
    expect(cyclePathTrace).toBeUndefined()
  })

  it('suppresses cyclePath when multiple circuits have differing fluids and no circuit is selected', () => {
    const spec = newPlotSpec('property', 'ambiguous-multi-circuit')
    spec.property = {
      diagram: 'ph',
      fluid: 'Water',
      quality: false,
      isolines: false,
      overlayStates: true,
      connectStates: true,
      closeCycle: false,
      stateTable: null, // ambiguous
    }

    const fig = buildFigure(spec, baseInputs)!
    const cyclePathTrace = fig.data.find((t) => t.uid === 'cycle_path')
    expect(cyclePathTrace).toBeUndefined()
  })

  it('allows cyclePath when circuit fluid matches diagram fluid', () => {
    const spec = newPlotSpec('property', 'fluid-match')
    spec.property = {
      diagram: 'ph',
      fluid: 'Water',
      quality: false,
      isolines: false,
      overlayStates: true,
      connectStates: true,
      closeCycle: false,
      stateTable: 'CircuitA', // Water on Water diagram
    }

    const fig = buildFigure(spec, baseInputs)!
    const cyclePathTrace = fig.data.find((t) => t.uid === 'cycle_path')
    expect(cyclePathTrace).toBeDefined()
  })
})

describe('Phase 10C: Export preparation and autoscale', () => {
  it('autoscales axes in full view mode', () => {
    const dummyFigure = {
      data: [],
      layout: {
        xaxis: { range: [10, 50], autorange: false },
        yaxis: { range: [100, 500], autorange: false },
      },
    }

    const fullExport = prepareFigureForExport(dummyFigure, {
      viewMode: 'full',
      background: '#ffffff',
    })

    expect(fullExport.layout.paper_bgcolor).toBe('#ffffff')
    expect(fullExport.layout.plot_bgcolor).toBe('#ffffff')
    expect(fullExport.layout.xaxis?.autorange).toBe(true)
    expect(fullExport.layout.xaxis?.range).toBeUndefined()
    expect(fullExport.layout.yaxis?.autorange).toBe(true)
    expect(fullExport.layout.yaxis?.range).toBeUndefined()

    const currentExport = prepareFigureForExport(dummyFigure, {
      viewMode: 'current',
    })
    expect(currentExport.layout.xaxis?.range).toEqual([10, 50])
  })
})
