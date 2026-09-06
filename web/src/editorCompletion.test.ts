import { describe, expect, it } from 'vitest'
import {
  completionsForPrefix,
  localComponentNames,
  localSignature,
  localTypeCompletions,
  namedArgsAlreadyPresent,
  parameterCompletions,
} from './editorCompletion'

const DOC = `COMPONENT Heater(in, out)
  PARAM fluid$, Q
  out.mdot = in.mdot
END
Source SUP(fluid$=Water, mdot=1, P=2e5, T=300)
Pipe LINE(fluid$=Water, L=1, D=0.05, rough=1e-4)
Heater H1(fluid$=Water, Q=1000)
`

describe('completionsForPrefix', () => {
  it('completes ports after an instance dot', () => {
    const items = completionsForPrefix(DOC, 'LINE.')
    expect(items?.map((i) => i.label).sort()).toEqual(['in', 'out'])
    expect(items?.every((i) => i.apply === i.label)).toBe(true)
  })

  it('completes members after instance.port.', () => {
    const items = completionsForPrefix(DOC, 'LINE.in.')
    expect(items?.map((i) => i.label)).toEqual(expect.arrayContaining(['P', 'h', 'mdot']))
  })

  it('includes local component types', () => {
    expect(localComponentNames(DOC).map((c) => c.name)).toEqual(['Heater'])
    expect(localTypeCompletions(DOC)[0]).toMatchObject({
      label: 'Heater',
      info: 'Local component definition',
    })
  })

  it('inserts only the missing parameter name', () => {
    const items = parameterCompletions(DOC, 'Heater', namedArgsAlreadyPresent('fluid$=Water, '))
    expect(items.map((i) => i.apply)).toEqual(['Q='])
  })

  it('describes a local signature from PARAM and ports', () => {
    expect(localSignature(DOC, 'Heater')).toMatchObject({
      usage: 'Heater Instance(in, out, fluid$=, Q=)',
      detail: 'Local component definition',
    })
  })
})
