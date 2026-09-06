import { expect, it } from 'vitest'
import { buildFigure, type FigureInputs } from './PlotCard'
import { newPlotSpec } from './types'

const inputs: FigureInputs = { states: { indices: [], columns: [], values: {} }, tableRows: [], tableResults: [], diagram: null, psychart: null, theme: 'dark' }
it('preserves traversal order, failed rows and original row identities', () => {
  const spec = newPlotSpec('xy', 'path')
  spec.xy = { xVar: 'x', yVars: ['y'] }
  const figure = buildFigure(spec, { ...inputs, tableRows: [3, 1, 2].map((x) => ({ id: String(x), values: { x: String(x), y: '4' } })), tableResults: [{ success: true, values: { x: 3, y: 4 }, error: null }, { success: false, values: {}, error: 'failed' }, { success: true, values: { x: 2, y: 4 }, error: null }] })!
  expect(figure.data[0]).toMatchObject({ x: [3, Number.NaN, 2], customdata: ['3', '1', '2'], connectgaps: false })
})

it('renders Y-only histogram samples and aligns array mesh channels by index', () => {
  const spec = newPlotSpec('xy', 'samples')
  spec.xy = { xVar: null, yVars: ['y'], chartType: 'histogram' }
  const variables = Object.entries({ 'x[1]': 0, 'x[2]': 1, 'x[3]': 0, 'y[1]': 0, 'y[2]': 0, 'y[3]': 1, 'z[1]': 2, 'z[2]': 3, 'z[3]': 4 }).map(([name, value]) => ({ name, value, units: '' }))
  expect(buildFigure(spec, { ...inputs, variables })!.data[0]).toMatchObject({ type: 'histogram', x: [0, 0, 1] })
  spec.xy = { xVar: 'x', yVars: ['y'], zVar: 'z', chartType: 'surface3d' }
  expect(buildFigure(spec, { ...inputs, variables })!.data[0]).toMatchObject({ type: 'mesh3d', x: [0, 1, 0], y: [0, 0, 1], z: [2, 3, 4] })
  expect(buildFigure(spec, { ...inputs, variables: variables.filter((v) => v.name !== 'z[2]') })!.data).toHaveLength(0)
})
