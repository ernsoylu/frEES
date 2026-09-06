import { expect, it } from 'vitest'
import { duplicateAsEditable, fillMissingCells, functionTableFromDigitizer, mergeCodeTables, toFunctionTableDtos } from './tables'
import { csvValuesFor } from './tablesGrid/tableGridModel'

it('retains close knots, first exact duplicates and full precision through adapters and copies', () => {
  const points = [[1.000001, 1.2345678901234567], [1.000002, 2.345678901234567], [1.000001, 99]]
  const [code] = mergeCodeTables([], [{ name: 'curve', argNames: ['x'], xLog: false, yLog: false, curves: [{ param: null, points }] }])
  const copy = duplicateAsEditable(code)
  expect(toFunctionTableDtos([copy])[0].curves[0].points).toEqual(points.slice(0, 2))
  expect(csvValuesFor(copy).flat()).toContain(String(points[0][1]))
  const digitized = functionTableFromDigitizer({ existing: [], xName: 'x', yName: 'y', xLog: false, yLog: false, curves: [{ param: '', points: points.map(([x, y]) => ({ x, y })) }] })
  expect(digitized.rows).toEqual(code.rows)
  const [ode] = mergeCodeTables([], [], [], [{ name: 'trace', units: [], events: [], method: 'rk45', stopped: false, endTime: 1, vars: ['t', 'y'], rows: [points[0]] }])
  expect(ode.rows[0]).toMatchObject({ values: { t: String(points[0][0]), y: String(points[0][1]) } })
  const filled = fillMissingCells({ ...digitized, rows: [{ x: '0', ys: ['0'] }, { x: '1', ys: [''] }, { x: '3', ys: ['1'] }] })
  expect(filled.rows[1].ys[0]).toBe(String(1 / 3))
})

it('rejects invalid log domains atomically and records valid interpolation without extrapolation', () => {
  const table = functionTableFromDigitizer({ existing: [], xName: 'x', yName: 'y', xLog: true, yLog: true, curves: [{ param: '', points: [{ x: 1, y: 1 }, { x: 100, y: 10000 }] }] })
  const rows = [{ x: '1', ys: ['1'] }, { x: '10', ys: [''] }, { x: '100', ys: ['10000'] }, { x: '1000', ys: [''] }]
  const filled = fillMissingCells({ ...table, rows })
  expect(Number(filled.rows[1].ys[0])).toBeCloseTo(100, 12)
  expect(filled.rows[3].ys[0]).toBe('')
  expect(filled.interpolatedCells).toEqual([{ row: 1, column: 0 }])
  expect(() => fillMissingCells({ ...table, rows: [{ x: '1', ys: ['-1'] }, ...rows.slice(1)] })).toThrow('row 1, curve 1')
  expect(rows[1].ys[0]).toBe('')
})
