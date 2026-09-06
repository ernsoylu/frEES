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
