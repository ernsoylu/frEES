import { expect, it } from 'vitest'
import { resolvePlotSource } from './sources'
import { newPlotSpec } from './types'
import { newParamTable } from '../tables'
it('migrates only unambiguous sources and never rebinds a saved source after deletion or reorder', () => {
  const a = { ...newParamTable([]), id: 'a', vars: ['x', 'y'] }
  const b = { ...a, id: 'b' }
  const plot = newPlotSpec('xy', 'curve')
  plot.xy = { xVar: 'x', yVars: ['y'] }
  expect(resolvePlotSource(plot, [a, b], [])).toBeUndefined()
  const source = resolvePlotSource(plot, [a], [])!
  expect(source).toMatchObject({ kind: 'table', tableId: 'a' })
  expect(resolvePlotSource({ ...plot, source }, [b, a], [])).toEqual(source)
  expect(resolvePlotSource({ ...plot, source }, [b], [])).toEqual(source)
})
