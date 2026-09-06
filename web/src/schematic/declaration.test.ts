import { describe, expect, it } from 'vitest'
import {
  declaredComponentTypes,
  declaredInstances,
  declarationLine,
  declarationLineFromCheck,
  instanceTypes,
  stripComments,
} from './declaration'

const DOC = `// A two-loop network
Pump P1(eta=0.7)
TwoPhaseEvaporator EVAP(UA=1200)
connect(P1.out, EVAP.in)
q = P1.out.mdot * 2
`

describe('declarationLine', () => {
  it('finds the line that declares an instance', () => {
    expect(declarationLine(DOC, 'P1')).toBe(2)
    expect(declarationLine(DOC, 'EVAP')).toBe(3)
  })

  it('is case-insensitive like the language', () => {
    expect(declarationLine(DOC, 'p1')).toBe(2)
    expect(declarationLine(DOC, 'evap')).toBe(3)
  })

  it('ignores references in connects and equations', () => {
    // P1 appears on lines 4 and 5 too; the declaration is still line 2.
    expect(declarationLine('connect(P1.out, EVAP.in)\nPump P1(eta=0.7)', 'P1')).toBe(2)
  })

  it('skips commented-out declarations', () => {
    expect(declarationLine('// Pump P1(eta=0.7)\nPump P1(eta=0.9)', 'P1')).toBe(2)
  })

  it('returns null when the instance is absent', () => {
    expect(declarationLine(DOC, 'NOPE')).toBeNull()
    expect(declarationLine('', 'P1')).toBeNull()
  })

  it('does not treat a name that merely prefixes another as a match', () => {
    expect(declarationLine('Pump P10(eta=0.7)', 'P1')).toBeNull()
  })
})

describe('declarationLineFromCheck', () => {
  const instances = [
    { name: 'a', label: 'A', line: 6 },
    { name: 'a.inner', label: 'inner', line: 6 },
    { name: 'b', label: 'B', line: 7 },
    { name: 'b.inner', label: 'inner', line: 7 },
  ]

  it('navigates a repeated child name to the correct parent scope', () => {
    expect(declarationLineFromCheck(instances, 'a.inner')).toBe(6)
    expect(declarationLineFromCheck(instances, 'b.inner')).toBe(7)
  })
})

describe('instanceTypes', () => {
  it('reads instance declarations from the document', () => {
    const types = instanceTypes('LiquidSource SRC(fluid$=Water)\nLiquidPump PMP(eta=0.7)\n')
    expect(types.get('src')).toBe('LiquidSource')
    expect(types.get('pmp')).toBe('LiquidPump')
  })

  it('ignores connects, blocks and comments', () => {
    const types = instanceTypes([
      '// Pump P1(eta=0.7)',
      'connect(SRC.out, PMP.in)',
      'FUNCTION f(x)',
      'TABLE t(x)',
      'DYNAMIC d(t = 0 .. 1)',
      'Pump P1(eta=0.7)',
    ].join('\n'))
    expect([...types.keys()]).toEqual(['p1'])
  })

  it('is empty for a component-free document', () => {
    expect(instanceTypes('x = 2\ny = x + 1').size).toBe(0)
  })

  // The regression the rendered EV example showed: prose inside a multi-line
  // `{ … }` comment drew `line`, `pipe` and `hx_eta_surf` nodes on the canvas.
  it('does not read component instances out of a multi-line comment', () => {
    const doc = [
      '{ A complete EV thermal-management system.',
      '',
      '  Coolant line (EG50): the pump feeds a wide-branch split.',
      '  A discretized radiator pipe (three wall-HX cells) rejects heat.',
      '  Derated by the surface efficiency hx_eta_surf(.., fin_efficiency(mL)). }',
      'LiquidPump PUMP(eta=0.6)',
    ].join('\n')

    expect([...instanceTypes(doc).keys()]).toEqual(['pump'])
  })

  it('rejects a prose phrase whose first word happens to name a real component', () => {
    // `radiator` IS a catalog type, so comment-stripping alone is not the only
    // guard — an unknown-type filter has to hold the line too.
    const types = instanceTypes('a radiator pipe (three cells) rejects heat', new Set(['liquidpump']))
    expect(types.size).toBe(0)
  })

  it('keeps declarations whose type the document defines itself', () => {
    const doc = 'COMPONENT MyCell(a, b)\nEND\nMyCell C1(UA=10)'
    const known = new Set([...declaredComponentTypes(doc)])
    expect(instanceTypes(doc, known).get('c1')).toBe('MyCell')
  })
})

describe('declaredInstances', () => {
  it('keeps the spelling the document used', () => {
    // A solve reports the written name, but a document that has only been
    // checked has not produced one — and labelling the pump `pump` until you
    // solve looks like the drawing lost information it never had.
    const types = declaredInstances('LiquidPump PUMP(eta=0.6)\nTwoPhaseCompressor CMP(eta=0.7)')
    expect(types.get('pump')).toEqual({ label: 'PUMP', type: 'LiquidPump' })
    expect(types.get('cmp')?.label).toBe('CMP')
  })

  it('keys canonically, so it joins with the lowercase wire names', () => {
    expect([...declaredInstances('LiquidPump Pmp()').keys()]).toEqual(['pmp'])
  })
})

describe('stripComments', () => {
  it('blanks a block comment while preserving line numbers', () => {
    const out = stripComments('a = 1\n{ two\nlines }\nb = 2')
    expect(out.split('\n')).toHaveLength(4)
    expect(out.split('\n')[3]).toBe('b = 2')
    expect(out.split('\n')[1].trim()).toBe('')
  })

  it('keeps code that follows a closing brace on the same line', () => {
    expect(stripComments('{ note } x = 1').trim()).toBe('x = 1')
  })

  it('drops a trailing line comment', () => {
    expect(stripComments('x = 1 // note')).toBe('x = 1 ')
  })
})
