import { describe, it, expect } from 'vitest'
import { activeCallAt, highlightArgIndex, usageArgParts } from './signatureHelp'

const caret = (src: string): { doc: string; at: number } => {
  const at = src.indexOf('|')
  return { doc: src.replace('|', ''), at }
}

describe('activeCallAt', () => {
  it('resolves Type Instance( to the type, not the instance name', () => {
    const { doc, at } = caret('Compressor CMP(fluid$=R134a, eta=|')
    const call = activeCallAt(doc, at)
    expect(call).toMatchObject({ name: 'Compressor', instanceName: 'CMP', namedArg: 'eta' })
  })

  it('keeps a function call as the callee', () => {
    const { doc, at } = caret('y = sin(|')
    expect(activeCallAt(doc, at)).toMatchObject({ name: 'sin', argIndex: 0 })
  })

  it('survives a multiline argument list', () => {
    const { doc, at } = caret('Compressor CMP(\n  fluid$=R134a,\n  eta=|\n)')
    const call = activeCallAt(doc, at)
    expect(call?.name).toBe('Compressor')
    expect(call?.namedArg).toBe('eta')
  })

  it('does not count commas inside nested calls or strings', () => {
    const { doc, at } = caret("Pipe LINE(fluid$='A, B', L=sin(1, 2), D=|")
    const call = activeCallAt(doc, at)
    expect(call).toMatchObject({ name: 'Pipe', namedArg: 'D', argIndex: 2 })
  })

  it('ignores commas inside comments', () => {
    const { doc, at } = caret('Compressor CMP(eta=0.7, { unused, list } fluid$=|')
    const call = activeCallAt(doc, at)
    expect(call?.namedArg).toBe('fluid$')
    expect(call?.argIndex).toBe(1)
  })

  it('stays quiet inside a comment or string', () => {
    const inside = caret('Compressor CMP(eta={ 1, |')
    expect(activeCallAt(inside.doc, inside.at)).toBeNull()
  })
})

describe('highlightArgIndex', () => {
  const usage = 'Compressor inst(eta, fluid$, model$)'

  it('uses comma position for positional arguments', () => {
    expect(highlightArgIndex(usage, { name: 'Compressor', argIndex: 1 })).toBe(1)
  })

  it('highlights a reordered named argument by name', () => {
    expect(highlightArgIndex(usage, { name: 'Compressor', argIndex: 0, namedArg: 'model$' })).toBe(2)
    expect(highlightArgIndex(usage, { name: 'Compressor', argIndex: 2, namedArg: 'eta' })).toBe(0)
  })

  it('splits usage args on top-level commas only', () => {
    expect(usageArgParts('f(a, g(b, c), d)')).toEqual(['a', ' g(b, c)', ' d'])
  })
})
