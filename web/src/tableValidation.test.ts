import { expect, it } from 'vitest'
import { newFunctionTable, normalizeTables, toFunctionTableDtos, duplicateAsEditable } from './tables'
import { tableInputIssues } from './tableValidation'
it('retains invalid drafts but refuses solver conversion with the cell location', () => {
  const table = newFunctionTable([], true)
  table.rows[0] = { x: '12e', ys: ['3'] }
  expect(() => toFunctionTableDtos([table])).toThrow('Row 1, x')
  expect(table.rows[0].x).toBe('12e')
  table.rows[0].x = ''
  expect(tableInputIssues(table)).toEqual([])
})
it('retains malformed imports for recovery and generates distinct copy identities and names', () => {
  const raw = { name: 'broken', rows: 'wrong' }
  const recovered = normalizeTables([raw])
  expect(recovered[0].recoveredContent).toEqual(raw)
  expect(tableInputIssues(recovered[0]).join()).toContain('Malformed')
  const table = newFunctionTable([], true)
  const first = duplicateAsEditable(table)
  const second = duplicateAsEditable(table, [table, first])
  expect(second.name).not.toBe(first.name)
  expect(second.id).not.toBe(first.id)
})
