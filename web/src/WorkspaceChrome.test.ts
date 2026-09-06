import { expect, it } from 'vitest'
import { checkPill } from './WorkspaceChrome'
import type { CheckResponse } from './api'

const checked: CheckResponse = {
  solvable: true, equations: 12, unknowns: 12, variables: [], unitWarnings: [],
  inferredUnits: {}, errors: [], errorLine: null,
  message: 'No syntax errors were detected. There are 12 equations and 12 variables.',
}

it('shows successful checks as structurally solvable, preserving warnings and actual errors', () => {
  expect(checkPill(checked)).toMatchObject({ color: 'green', label: 'Structurally solvable', message: checked.message })
  expect(checkPill({ ...checked, unitWarnings: ['Unknown unit foo'] }).label).toBe('Structurally solvable · unconverted units')
  expect(checkPill({ ...checked, solvable: false, message: 'Underspecified. No syntax errors were detected.' }).label).toBe('Underspecified')
  expect(checkPill({ ...checked, solvable: false, errors: [{ line: 1, column: 2, message: 'Expected expression' }] }).label).toBe('Syntax error')
  expect(checkPill({ ...checked, solvable: false, errors: undefined, message: 'Syntax error: expected expression' }).label).toBe('Syntax error')
})
