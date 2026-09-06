import { describe, expect, it } from 'vitest'
import { formatGuessDirective, readGuessDirectives, writeGuessDirectives } from './guessDirectives'

describe('readGuessDirectives', () => {
  it('reads guess, bounds, and both', () => {
    const out = readGuessDirectives('GUESS x = 2\nGUESS y [0, 10]\nGUESS z = 5 [1, 9]\n')
    expect(out).toEqual([
      { name: 'x', guess: 2, lower: null, upper: null, line: 0 },
      { name: 'y', guess: null, lower: 0, upper: 10, line: 1 },
      { name: 'z', guess: 5, lower: 1, upper: 9, line: 2 },
    ])
  })

  it('accepts the language spellings: case, signs, exponents', () => {
    const out = readGuessDirectives('guess T = -3.5e2 [-1e3, 0]')
    expect(out[0]).toMatchObject({ name: 'T', guess: -350, lower: -1000, upper: 0 })
  })

  it('ignores a bare GUESS, which the parser rejects anyway', () => {
    expect(readGuessDirectives('GUESS x')).toEqual([])
  })

  it('ignores equations that merely mention the word', () => {
    expect(readGuessDirectives('guess_count = 3\nx = 2')).toEqual([])
  })

  it('reads a public member path', () => {
    const out = readGuessDirectives('GUESS HX.in.P = 200000 [1e5, 5e5]\n')
    expect(out[0]).toMatchObject({ name: 'HX.in.P', guess: 200000, lower: 1e5, upper: 5e5 })
  })
})

describe('formatGuessDirective', () => {
  it('renders each shape', () => {
    expect(formatGuessDirective('x', 2, null, null)).toBe('GUESS x = 2')
    expect(formatGuessDirective('y', null, 0, 10)).toBe('GUESS y [0, 10]')
    expect(formatGuessDirective('z', 5, 1, 9)).toBe('GUESS z = 5 [1, 9]')
  })

  it('is null when there is nothing to declare', () => {
    expect(formatGuessDirective('x', null, null, null)).toBeNull()
    expect(formatGuessDirective('x', null, 0, null)).toBeNull()
    expect(formatGuessDirective('x', Number.NaN, null, null)).toBeNull()
  })
})

describe('writeGuessDirectives', () => {
  it('appends new directives as a block, leaving the document intact', () => {
    const out = writeGuessDirectives('x^2 = 4\ny = x + 1\n', [
      { name: 'x', guess: -3, lower: null, upper: null },
    ])
    expect(out).toBe('x^2 = 4\ny = x + 1\n\nGUESS x = -3\n')
  })

  it('replaces an existing directive in place, preserving order and comments', () => {
    const doc = '// header\nGUESS x = 1\nx^2 = 4\n'
    const out = writeGuessDirectives(doc, [{ name: 'x', guess: -3, lower: 0, upper: 9 }])
    expect(out).toBe('// header\nGUESS x = -3 [0, 9]\nx^2 = 4\n')
  })

  it('removes a directive that no longer declares anything', () => {
    const doc = 'GUESS x = 1\nx^2 = 4\n'
    const out = writeGuessDirectives(doc, [{ name: 'x', guess: null, lower: null, upper: null }])
    expect(out).toBe('x^2 = 4\n')
  })

  it('matches names case-insensitively, like the language', () => {
    const out = writeGuessDirectives('GUESS Eta = 0.7\n', [
      { name: 'eta', guess: 0.9, lower: null, upper: null },
    ])
    expect(out).toBe('GUESS eta = 0.9\n')
  })

  it('round-trips a member path with Variable Information', () => {
    const entries = [{ name: 'HX.in.P', guess: 200000, lower: 1e5, upper: 5e5 }]
    const doc = writeGuessDirectives('LiquidWallHX HX(fluid$=Water, UA=10)\n', entries)
    expect(doc).toContain('GUESS HX.in.P = 200000 [100000, 500000]')
    const read = readGuessDirectives(doc)
    expect(read[0].name).toBe('HX.in.P')
    expect(read[0].guess).toBe(200000)
  })

  it('round-trips: what is written reads back identically', () => {
    const entries = [
      { name: 'x', guess: -3, lower: null, upper: null },
      { name: 'y', guess: null, lower: 0, upper: 10 },
      { name: 'z', guess: 5, lower: 1, upper: 9 },
    ]
    const doc = writeGuessDirectives('x^2 = 4\n', entries)
    const read = readGuessDirectives(doc)
    expect(read.map((d) => ({ name: d.name, guess: d.guess, lower: d.lower, upper: d.upper })))
      .toEqual(entries)
  })

  it('leaves the document untouched when nothing is declared', () => {
    const doc = 'x = 1\ny = 2\n'
    expect(writeGuessDirectives(doc, [{ name: 'x', guess: null, lower: null, upper: null }])).toBe(doc)
  })
})
