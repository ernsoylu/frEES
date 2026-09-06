import { describe, it, expect } from 'vitest'
import {
  alreadyConnected,
  connectStatement,
  groupByInstance,
  hasUnknownUnitWarning,
  instancesInDiagnosis,
  parseFreeQuantities,
  previewDomain,
  previewWire,
} from './wiring'

describe('previewDomain', () => {
  it('uses the catalog library for ordinary ports', () => {
    expect(previewDomain('Resistor', 'a')).toBe('electrical')
    expect(previewDomain('Pipe', 'in')).toBe('fluid')
    expect(previewDomain('Conduction', 'a')).toBe('heat')
  })

  it('overrides wall/heat ports on mixed devices', () => {
    expect(previewDomain('LiquidWallHX', 'in')).toBe('fluid')
    expect(previewDomain('LiquidWallHX', 'wall')).toBe('heat')
  })

  it('does not invent a domain for an unknown type', () => {
    expect(previewDomain('MyCustomThing', 'foo')).toBeNull()
  })
})

describe('previewWire', () => {
  it('rejects wiring a component to itself', () => {
    const r = previewWire(
      { instance: 'r1', port: 'a', label: 'R1', type: 'Resistor' },
      { instance: 'r1', port: 'b', label: 'R1', type: 'Resistor' },
    )
    expect(r.ok).toBe(false)
    if (!r.ok) expect(r.reason).toMatch(/different component/i)
  })

  it('names both endpoints of an electrical-to-fluid attempt', () => {
    const r = previewWire(
      { instance: 'r1', port: 'a', label: 'R1', type: 'Resistor' },
      { instance: 'line', port: 'in', label: 'LINE', type: 'Pipe' },
    )
    expect(r.ok).toBe(false)
    if (!r.ok) {
      expect(r.reason).toMatch(/electrical/)
      expect(r.reason).toMatch(/fluid/)
      expect(r.reason).toContain('R1.a')
      expect(r.reason).toContain('LINE.in')
    }
  })

  it('allows a valid fluid chain and a heat coupling', () => {
    expect(
      previewWire(
        { instance: 'sup', port: 'out', label: 'SUP', type: 'Source' },
        { instance: 'line', port: 'in', label: 'LINE', type: 'Pipe' },
      ).ok,
    ).toBe(true)
    expect(
      previewWire(
        { instance: 'hx', port: 'wall', label: 'HX', type: 'LiquidWallHX' },
        { instance: 'wall', port: 'a', label: 'W', type: 'Conduction' },
      ).ok,
    ).toBe(true)
  })

  it('allows unknown custom connectors rather than inventing physics', () => {
    expect(
      previewWire(
        { instance: 'a', port: 'foo', label: 'A', type: 'Mystery' },
        { instance: 'b', port: 'bar', label: 'B', type: 'Mystery' },
      ).ok,
    ).toBe(true)
  })

  it('rejects reconnecting endpoints already in one connection-set', () => {
    const r = previewWire(
      { instance: 'a', port: 'out', label: 'A', type: 'Source' },
      { instance: 'b', port: 'in', label: 'B', type: 'Pipe' },
      [{ domain: 'fluid', endpoints: ['a.out', 'b.in', 'c.in'] }],
    )
    expect(r.ok).toBe(false)
    if (!r.ok) expect(r.reason).toMatch(/already/)
  })

  it('allows a legal branch onto a third port of an existing node', () => {
    // A.out is on the node; C.out is not — this is a branch, not a duplicate.
    expect(
      previewWire(
        { instance: 'c', port: 'out', label: 'C', type: 'Source' },
        { instance: 'a', port: 'out', label: 'A', type: 'Source' },
        [{ domain: 'fluid', endpoints: ['a.out', 'b.in'] }],
      ).ok,
    ).toBe(true)
  })
})

describe('alreadyConnected', () => {
  it('matches case-insensitively and ignores raw-string formatting', () => {
    expect(
      alreadyConnected([{ domain: 'fluid', endpoints: ['SUP.out', 'LINE.in'] }], 'sup.out', 'line.in'),
    ).toBe(true)
    expect(
      alreadyConnected([{ domain: 'fluid', endpoints: ['SUP.out', 'LINE.in'] }], 'sup.out', 'ret.in'),
    ).toBe(false)
  })
})

describe('connectStatement', () => {
  it('uses display labels', () => {
    expect(
      connectStatement(
        { instance: 'sup', port: 'out', label: 'SUP' },
        { instance: 'line', port: 'in', label: 'LINE' },
      ),
    ).toBe('connect(SUP.out, LINE.in)')
  })
})

describe('diagnosis helpers', () => {
  it('extracts free quantities and groups them by instance', () => {
    const msg =
      'There are 5 equations and 7 variables. The problem is underspecified and cannot be solved. ' +
      'Free quantities (no defining relation): LINE.in.P, LINE.out.P, RET.in.mdot.'
    expect(parseFreeQuantities(msg)).toEqual(['LINE.in.P', 'LINE.out.P', 'RET.in.mdot'])
    expect(instancesInDiagnosis(msg).sort()).toEqual(['line', 'ret'])
    expect(groupByInstance(parseFreeQuantities(msg)).map((g) => g.instance).sort()).toEqual([
      'line',
      'ret',
    ])
  })

  it('detects unknown-unit warnings', () => {
    expect(hasUnknownUnitWarning(['unknown unit `bananas`: the literal was left unconverted'])).toBe(
      true,
    )
    expect(hasUnknownUnitWarning(['T_out: mixing K with C'])).toBe(false)
  })
})
