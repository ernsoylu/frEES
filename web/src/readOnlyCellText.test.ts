import { describe, expect, it } from 'vitest'
import { readOnlyCellText } from './tables'
import type { ParamRow } from './tables'
import type { TableRowResult } from './api'

/**
 * A code PARAMETRIC table used to paint from its input rows alone, so running
 * "Solve Table" reported "121/121 runs solved" while every solved column stayed
 * blank — the sweep column was the only thing the grid had ever been given.
 */
describe('readOnlyCellText', () => {
  const row = (values: Record<string, string>): ParamRow => ({ id: 'r1', values })
  const solved = (values: Record<string, number>): TableRowResult => ({
    success: true,
    values,
    error: null,
  })

  it('shows the solved value in a column the user left blank', () => {
    expect(readOnlyCellText(row({ t: '0.05', x: '' }), solved({ t: 0.05, x: 0.0969922 }), 'x'))
      .toBe('0.0969922')
  })

  it('keeps the input the user supplied instead of the solver echo', () => {
    // The solver returns every variable, including the ones it was given. Those
    // must not overwrite the cell the user typed, or editing a sweep value would
    // appear to be ignored.
    expect(readOnlyCellText(row({ t: '0.05' }), solved({ t: 0.05000000001 }), 't')).toBe('0.05')
  })

  it('leaves the cell blank when the row failed to solve', () => {
    const failed: TableRowResult = { success: false, values: {}, error: 'singular' }
    expect(readOnlyCellText(row({ t: '0.05', x: '' }), failed, 'x')).toBe('')
  })

  it('leaves the cell blank before the table has been run at all', () => {
    expect(readOnlyCellText(row({ t: '0.05', x: '' }), undefined, 'x')).toBe('')
  })

  it('ignores a non-finite result rather than painting NaN or Infinity', () => {
    expect(readOnlyCellText(row({ x: '' }), solved({ x: Number.NaN }), 'x')).toBe('')
    expect(readOnlyCellText(row({ x: '' }), solved({ x: Number.POSITIVE_INFINITY }), 'x')).toBe('')
  })

  it('retains full precision for clipboard data', () => {
    expect(readOnlyCellText(row({ x: '' }), solved({ x: 4.898979485566356 }), 'x')).toBe('4.898979485566356')
  })

  it('is blank for a variable the solve did not produce', () => {
    expect(readOnlyCellText(row({ x: '' }), solved({ y: 1 }), 'x')).toBe('')
  })
})
