// curveFitShared.test.ts — the pure pieces shared by CurveFitModal and the
// digitizer's fit dialog (Wave H): template-variable substitution, the
// fitted-model editor text, and the Phase 4.2 numeric-list parsing and band
// figure.

import { describe, expect, it } from 'vitest'
import {
  buildFitFigure,
  FIT_TEMPLATES,
  fittedModelInsertText,
  parseNumberList,
  templateModelFor,
  validateSigma,
} from './curveFitShared'
import { CURVE_FIT_FAILURE, CurveFitResponse } from './api'

describe('templateModelFor', () => {
  it('rewrites whole-word x and y to the caller names', () => {
    const linear = FIT_TEMPLATES[0]
    expect(templateModelFor(linear, 'Re', 'f_D')).toBe('f_D = a * Re + b')
  })

  it('does not touch the x inside exp()', () => {
    const exp = FIT_TEMPLATES.find((t) => t.name.startsWith('Exponential ('))
    expect(exp).toBeDefined()
    expect(templateModelFor(exp!, 'T', 'k')).toBe('k = a * exp(b * T)')
  })

  it('survives an x/y swap without cascading (single-pass substitution)', () => {
    const linear = FIT_TEMPLATES[0]
    expect(templateModelFor(linear, 'y', 'x')).toBe('x = a * y + b')
  })
})

describe('fittedModelInsertText', () => {
  it('emits the template comment, parameter lines (with units) and the model', () => {
    const text = fittedModelInsertText(
      'Linear (y = a * x + b)',
      'y = a * x + b',
      ['a', 'b'],
      [2, 3],
      { a: 'kPa/m', b: ' ' },
    )
    const lines = text.split('\n')
    expect(lines[0]).toBe('{ Fitted Model: Linear (y = a * x + b) }')
    expect(lines[1]).toBe('a = 2 [kPa/m]')
    // A blank unit adds no bracket.
    expect(lines[2]).toBe('b = 3')
    expect(lines[3]).toBe('y = a * x + b')
  })

  it('labels a custom model as Custom and defaults to no units', () => {
    const text = fittedModelInsertText('custom', ' q = a * t ', ['a'], [1.5])
    expect(text).toBe('{ Fitted Model: Custom }\na = 1.5\nq = a * t')
  })
})

describe('parseNumberList', () => {
  it('reads a comma-separated list of the expected length', () => {
    expect(parseNumberList(' 1, 2.5 ,-3 ', 3, 'Lower bounds')).toEqual([1, 2.5, -3])
  })

  it('treats blank as absent, not invalid — the fields are optional', () => {
    expect(parseNumberList('', 2, 'Lower bounds')).toBeUndefined()
    expect(parseNumberList('   ', 2, 'Lower bounds')).toBeUndefined()
  })

  it('names the field and the counts when the length is wrong', () => {
    const problem = parseNumberList('1, 2', 3, 'Upper bounds')
    expect(problem).toBe('Upper bounds needs one value per parameter (got 2, expected 3).')
  })

  it('refuses non-numeric entries', () => {
    expect(parseNumberList('1, oops', 2, 'Initial guesses')).toBe(
      'Initial guesses must be numbers, comma-separated.',
    )
  })
})

describe('validateSigma', () => {
  it('accepts absent and positive uncertainties', () => {
    expect(validateSigma(undefined)).toBeNull()
    expect(validateSigma([0.1, 2, 1e-9])).toBeNull()
  })

  it('refuses zero, negative and non-finite — the engine would too', () => {
    for (const bad of [[0.1, 0], [0.1, -1], [0.1, NaN], [0.1, Infinity]]) {
      expect(validateSigma(bad)).toBe('Uncertainties must be finite and greater than zero.')
    }
  })
})

describe('buildFitFigure', () => {
  const base: CurveFitResponse = {
    ...CURVE_FIT_FAILURE,
    error: null,
    success: true,
    parameterNames: ['a'],
    fittedParameters: [2],
    fittedValues: [2, 4, 6],
    residuals: [0, 0, 0],
    confidence: 0.95,
    confidenceBandLo: [1.5, 3.5, 5.5],
    confidenceBandHi: [2.5, 4.5, 6.5],
    predictionBandLo: [1, 3, 5],
    predictionBandHi: [3, 5, 7],
    parameterStdErrors: [0.1],
    atBound: [false],
  }

  it('draws both ribbons, the curve and the observations', () => {
    const figure = buildFitFigure(base, [1, 2, 3], 'x', 'y', [2, 4, 6])
    expect(figure).not.toBeNull()
    // two traces per ribbon, plus fit, plus observed
    expect(figure!.data).toHaveLength(6)
    const names = figure!.data.map((t) => (t as { name?: string }).name)
    expect(names).toContain('Prediction band')
    expect(names).toContain('Confidence band')
    expect(names).toContain('Fit')
    expect(names).toContain('Observed')
  })

  it('sorts by x so a ribbon cannot fold back over itself', () => {
    const figure = buildFitFigure(base, [3, 1, 2], 'x', 'y', [6, 2, 4])
    const observed = figure!.data.find((t) => (t as { name?: string }).name === 'Observed')
    expect((observed as { x: number[] }).x).toEqual([1, 2, 3])
    // The y values travel with their own x, not with their original slot.
    expect((observed as { y: number[] }).y).toEqual([2, 4, 6])
  })

  it('omits a band the fit could not support rather than drawing it flat', () => {
    const noBands = {
      ...base,
      confidenceBandLo: [null, null, null],
      confidenceBandHi: [null, null, null],
      predictionBandLo: [null, null, null],
      predictionBandHi: [null, null, null],
    }
    const figure = buildFitFigure(noBands, [1, 2, 3], 'x', 'y', [2, 4, 6])
    expect(figure!.data).toHaveLength(2)
  })

  it('returns null for a failed fit or a length mismatch', () => {
    expect(buildFitFigure({ ...base, success: false }, [1, 2, 3], 'x', 'y', [2, 4, 6])).toBeNull()
    expect(buildFitFigure(base, [1, 2], 'x', 'y', [2, 4])).toBeNull()
  })
})
