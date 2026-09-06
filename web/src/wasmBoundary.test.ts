import { describe, it, expect, beforeAll } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import {
  initSync,
  solve,
  check,
  optimize,
  optimize_multi,
} from './wasm/pkg/frees.js'

beforeAll(() => {
  const wasmPath = resolve(__dirname, './wasm/pkg/frees_bg.wasm')
  const bytes = readFileSync(wasmPath)
  initSync({ module: bytes })
})

describe('WASM Real Boundary: Overrides', () => {
  it('applies overrides to Solve, updating dependent equations', () => {
    const source = 'x = 2\ny = x^2\n'
    const request = JSON.stringify({
      overrides: ['x = 3'],
    })
    const res = JSON.parse(solve(source, request))
    expect(res.success).toBe(true)
    const vars = Object.fromEntries(res.variables.map((v: { name: string; value: number }) => [v.name, v.value]))
    expect(vars['x']).toBe(3)
    expect(vars['y']).toBe(9)
  })

  it('handles case-insensitive overrides', () => {
    const source = 'a = 10\nb = a * 2\n'
    const request = JSON.stringify({
      overrides: ['A = 5'],
    })
    const res = JSON.parse(solve(source, request))
    expect(res.success).toBe(true)
    const vars = Object.fromEntries(res.variables.map((v: { name: string; value: number }) => [v.name.toLowerCase(), v.value]))
    expect(vars['a']).toBe(5)
    expect(vars['b']).toBe(10)
  })

  it('converts units in overrides through the solver', () => {
    const source = 'P = 100 [kPa]\n'
    const request = JSON.stringify({
      overrides: ['P = 250 [kPa]'],
    })
    const res = JSON.parse(solve(source, request))
    expect(res.success).toBe(true)
    const p = res.variables.find((v: { name: string; value: number }) => v.name === 'P')!
    expect(p.value).toBe(250000)
  })

  it('applies overrides to Check evaluation', () => {
    // Without overrides, y = x^2 has 1 equation and 2 variables (unsolvable)
    const source = 'y = x^2\n'
    const uncheck = JSON.parse(check(source, ''))
    expect(uncheck.solvable).toBe(false)

    // With override x = 3, Check recognises the degree of freedom is grounded
    const checked = JSON.parse(check(source, JSON.stringify({ overrides: ['x = 3'] })))
    expect(checked.solvable).toBe(true)
  })
})

describe('WASM Real Boundary: Multiple Roots', () => {
  it('solves all roots bounded up to 32 solutions when findAllSolutions is true', () => {
    const source = 'x^2 = 4\ny = x + 10\n'
    const request = JSON.stringify({ findAllSolutions: true })
    const res = JSON.parse(solve(source, request))
    expect(res.success).toBe(true)
    expect(res.solutions).toBeDefined()
    expect(res.solutions.length).toBe(2)

    const roots = res.solutions.map((s: { variables: Array<{ name: string; value: number }> }) => {
      const x = s.variables.find(v => v.name === 'x')!
      return Math.round(x.value)
    }).sort()

    expect(roots).toEqual([-2, 2])
    // Backwards compatibility: variables agrees with solutions[0].variables
    expect(res.variables).toEqual(res.solutions[0].variables)
  })

  it('solves a single root when findAllSolutions is false or omitted', () => {
    const source = 'x^2 = 4\n'
    const resDefault = JSON.parse(solve(source, ''))
    expect(resDefault.success).toBe(true)
    expect(resDefault.solutions.length).toBe(1)

    const resFalse = JSON.parse(solve(source, JSON.stringify({ findAllSolutions: false })))
    expect(resFalse.success).toBe(true)
    expect(resFalse.solutions.length).toBe(1)
  })
})

describe('WASM Real Boundary: Stopping Controls & Settings', () => {
  it('terminates with iteration error when maxIterations is severely capped', () => {
    // Stiff nonlinear problem that requires several Newton iterations
    const source = 'exp(x) + x = 10\n'
    const request = JSON.stringify({
      stopCriteria: {
        maxIterations: 1,
      },
    })
    const res = JSON.parse(solve(source, request))
    expect(res.success).toBe(false)
    expect(res.error).toBeDefined()
  })

  it('handles complexMode toggle in stopCriteria', () => {
    const source = 'x^2 = -4\n'
    const reqReal = JSON.stringify({ stopCriteria: { complexMode: false } })
    const resReal = JSON.parse(solve(source, reqReal))
    // In real mode, x^2 = -4 has no real root
    expect(resReal.success).toBe(false)

    const reqComplex = JSON.stringify({ stopCriteria: { complexMode: true } })
    const resComplex = JSON.parse(solve(source, reqComplex))
    expect(resComplex.success).toBe(true)
  })
})

describe('WASM Real Boundary: Imported Function Tables', () => {
  it('evaluates imported function table in solve with points format', () => {
    const source = 'y = lookup(2.0)\n'
    const request = JSON.stringify({
      functionTables: [
        {
          name: 'lookup',
          argNames: ['x'],
          curves: [
            {
              param: null,
              points: [
                [1.0, 10.0],
                [2.0, 20.0],
                [3.0, 30.0],
              ],
            },
          ],
        },
      ],
    })
    const res = JSON.parse(solve(source, request))
    expect(res.success).toBe(true)
    const y = res.variables.find((v: { name: string; value: number }) => v.name === 'y')!
    expect(y.value).toBe(20.0)
  })

  it('evaluates imported function table in solve with xs/ys sidecar format', () => {
    const source = 'y = sidecar_fn(2.0)\n'
    const request = JSON.stringify({
      functionTables: [
        {
          name: 'sidecar_fn',
          argNames: ['x'],
          curves: [
            {
              param: null,
              xs: [1.0, 2.0, 3.0],
              ys: [15.0, 25.0, 35.0],
            },
          ],
        },
      ],
    })
    const res = JSON.parse(solve(source, request))
    expect(res.success).toBe(true)
    const y = res.variables.find((v: { name: string; value: number }) => v.name === 'y')!
    expect(y.value).toBe(25.0)
  })

  it('evaluates imported function table in single-objective optimizer', () => {
    const source = 'f = (x - tab(1.0))^2\n'
    const req = JSON.stringify({
      objective: 'f',
      decision: 'x',
      lower: 0.0,
      upper: 10.0,
      maximize: false,
      functionTables: [
        {
          name: 'tab',
          argNames: ['a'],
          curves: [
            {
              param: null,
              points: [
                [0.0, 0.0],
                [1.0, 3.5],
                [2.0, 7.0],
              ],
            },
          ],
        },
      ],
    })
    const res = JSON.parse(optimize(source, req))
    expect(res.success).toBe(true)
    expect(Math.abs(res.decision.value - 3.5)).toBeLessThan(1e-2)
  })

  it('evaluates imported function table in multi-objective Pareto optimizer', () => {
    const source = 'f = (x - tab(1.0))^2\ng = (x - 6.0)^2\n'
    const req = JSON.stringify({
      objectives: ['f', 'g'],
      maximize: [false, false],
      decisions: ['x'],
      lowers: [0.0],
      uppers: [10.0],
      populationSize: 8,
      generations: 4,
      functionTables: [
        {
          name: 'tab',
          argNames: ['a'],
          curves: [
            {
              param: null,
              points: [
                [0.0, 0.0],
                [1.0, 2.0],
                [2.0, 4.0],
              ],
            },
          ],
        },
      ],
    })
    const res = JSON.parse(optimize_multi(source, req))
    expect(res.success).toBe(true)
    expect(res.front.length).toBeGreaterThan(0)
  })
})

describe('WASM Real Boundary: Error Classifications', () => {
  it('reports syntax error with errorLine', () => {
    const source = 'x = 1\ny = = 2\n'
    const res = JSON.parse(solve(source, ''))
    expect(res.success).toBe(false)
    expect(res.error).toBeDefined()
    expect(res.errorLine).toBe(2)
  })

  it('reports degrees of freedom error when underdetermined', () => {
    const source = 'x + y = 10\n'
    const res = JSON.parse(solve(source, ''))
    expect(res.success).toBe(false)
    expect(res.error).toBeDefined()
  })
})
