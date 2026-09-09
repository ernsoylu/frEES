// @vitest-environment node
// @ts-expect-error Node built-in module not declared in Vite client tsconfig
import { readFileSync } from 'node:fs'
import { expect, it } from 'vitest'
import { initSync, solve, solve_zerocopy, solve_table, solve_table_zerocopy } from './pkg/frees.js'

initSync({ module: readFileSync(new URL('./pkg/frees_bg.wasm', import.meta.url)) })

it('transfers real WASM sweep and trajectory data without detaching engine memory', () => {
  const source = 'y = sqrt(x)\nz = 2 * y'
  const request = JSON.stringify({ table: { variables: ['x', 'y'], rows: [{ x: 2 }, { x: -1 }, { x: 4 }] } })
  const legacy = JSON.parse(solve_table(source, request))
  const typed = solve_table_zerocopy(source, request)
  const envelope = JSON.parse(typed.envelope)
  const matrix = structuredClone(typed.matrix, { transfer: [typed.matrix.buffer] })
  expect(typed.matrix.byteLength).toBe(0)
  expect(matrix).toBeInstanceOf(Float64Array)
  expect(envelope.results[1].success).toBe(false)
  for (const [r, row] of legacy.results.entries()) {
    expect(envelope.results[r].values).toEqual({})
    for (const [c, name] of envelope.varNames.entries()) {
      expect(matrix[r * envelope.numCols + c]).toBe(row.values[name] ?? NaN)
    }
  }

  const dynamic = "y_final = FinalValue('y')\nDYNAMIC relax(method = ode45, time = 0 .. 1, points = 5)\n der(y) = -y / 2\n y(0) = 1\nEND"
  const expected = JSON.parse(solve(dynamic, '{}'))
  const trajectory = solve_zerocopy(dynamic, '{}')
  expect(JSON.parse(trajectory.envelope).odeTables[0].rows).toEqual([])
  const buffers = structuredClone(trajectory.odeBuffers, {
    transfer: trajectory.odeBuffers.map((buffer: Float64Array) => buffer.buffer),
  })
  expect(trajectory.odeBuffers[0].byteLength).toBe(0)
  expect(Array.from(buffers[0])).toEqual(expected.odeTables[0].rows.flat())
  expect(JSON.parse(solve('x = 3', '{}')).success).toBe(true)
  expect(JSON.parse(solve_table_zerocopy(source, '{').envelope).error).toMatch(/Invalid request/)
  expect(JSON.parse(solve_zerocopy('x =', '{}').envelope).success).toBe(false)
})
