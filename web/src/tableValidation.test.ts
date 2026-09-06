import { expect, it } from 'vitest'
import { newFunctionTable, newParamTable, normalizeTables, toFunctionTableDtos, duplicateAsEditable, duplicateAsSnapshot } from './tables'
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
it('freezes solved outputs in a snapshot copy and leaves an editable copy input-only', () => {
  const table = newParamTable([])
  table.vars = ['T', 'eta']
  table.rows = [{ id: 'r1', values: { T: '300', eta: '' } }]
  table.results = [{ success: true, values: { T: 300, eta: 0.42 }, error: null }]
  const editable = duplicateAsEditable(table)
  expect(editable.kind).toBe('parametric')
  if (editable.kind !== 'parametric') return
  expect(editable.rows[0].values.eta).toBe('')
  expect(editable.results).toEqual([])
  const snapshot = duplicateAsSnapshot(table)
  expect(snapshot.kind).toBe('parametric')
  if (snapshot.kind !== 'parametric') return
  expect(snapshot.rows[0].values).toEqual({ T: '300', eta: '0.42' })
  expect(snapshot.results).toEqual([])
})
