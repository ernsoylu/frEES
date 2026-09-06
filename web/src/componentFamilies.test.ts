import { describe, expect, it } from 'vitest'
import { familyOf, MODEL_FAMILIES } from './componentFamilies'
import { COMPONENT_CATALOG } from './componentCatalog'

describe('MODEL_FAMILIES', () => {
  it('groups Fan / FanCurve / FanMap by required data', () => {
    const fan = familyOf('Fan')
    expect(fan?.members.map((m) => m.type)).toEqual(['Fan', 'FanCurve', 'FanMap'])
    expect(fan?.members.find((m) => m.type === 'FanMap')?.requiredData).toMatch(/map\$/)
    expect(fan?.members.find((m) => m.type === 'Fan')?.energy).toMatch(/enthalpy/)
  })

  it('keeps compressor and pump alternatives searchable without renaming types', () => {
    expect(familyOf('CompressorMap')?.id).toBe('compressor')
    expect(familyOf('TwoPhaseCompressor')?.id).toBe('compressor')
    expect(familyOf('PumpMap')?.id).toBe('pump')
    expect(familyOf('LiquidPumpMap')?.id).toBe('liquid-pump')
  })

  it('names only catalogued types', () => {
    const types = new Set(COMPONENT_CATALOG.map((c) => c.type))
    for (const family of MODEL_FAMILIES) {
      for (const member of family.members) {
        expect(types.has(member.type), member.type).toBe(true)
      }
    }
  })
})
