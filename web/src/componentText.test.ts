import { describe, it, expect } from 'vitest'
import type { ComponentSpec } from './componentCatalog'
import {
  generateComponentText,
  isValidInstanceName,
  suggestInstanceName,
  instanceNameError,
  missingRequiredParams,
  activeParams,
  selectedVariant,
  assembleBlock,
  formatStringValue,
  isPlainNumericLiteral,
  paramValueError,
  paramValueErrors,
  inactiveDraftParams,
} from './componentText'

const param = (over: Partial<ComponentSpec['params'][number]> & { name: string }) => ({
  isString: over.name.endsWith('$'),
  isSelector: false,
  isMap: false,
  unit: '',
  description: '',
  required: true,
  values: [],
  variants: [],
  defaultValue: '',
  ...over,
})

function spec(
  type: string,
  library: string,
  ports: string[],
  params: ComponentSpec['params'],
  variants: ComponentSpec['variants'] = [],
): ComponentSpec {
  return { type, library, summary: '', tags: [], ports, params, variants }
}

const selector = (values: string[], defaultValue = '') =>
  param({ name: 'model$', isSelector: true, required: false, values, defaultValue })

const chiller = spec('Chiller', 'ac', ['ref_in', 'ref_out'], [
  param({ name: 'ref$' }),
  param({ name: 'cool$' }),
  param({ name: 'U_tp', isString: false, unit: 'W/m^2-K' }),
  param({ name: 'eps_zone', isString: false, unit: '' }),
  selector(['isentropic', 'volumetric']),
])

const hx = spec('LiquidWallHX', 'liquid', ['in', 'out', 'wall'], [
  param({ name: 'fluid$' }),
  param({ name: 'UA', isString: false, unit: 'W/K' }),
])

const compressor = spec(
  'Compressor',
  'fluid',
  ['in', 'out'],
  [
    param({ name: 'eta', isString: false }),
    param({ name: 'fluid$' }),
    selector(['isentropic', 'volumetric'], 'isentropic'),
    param({ name: 'eta_v', isString: false, variants: ['volumetric'] }),
    param({ name: 'disp', isString: false, variants: ['volumetric'] }),
    param({ name: 'rpm', isString: false, variants: ['volumetric'] }),
  ],
  [
    { name: 'isentropic', requires: ['eta'] },
    { name: 'volumetric', requires: ['eta_v', 'disp', 'rpm'] },
  ],
)

const orifice = spec(
  'HydraulicOrifice',
  'hydraulic',
  ['in', 'out'],
  [
    param({ name: 'CdA', isString: false }),
    selector(['laminar', 'turbulent'], 'turbulent'),
    param({ name: 'nu', isString: false, variants: ['laminar'] }),
  ],
  [
    { name: 'laminar', requires: ['nu'] },
    { name: 'turbulent', requires: [] },
  ],
)

function hxLine(ua: string, fluid = 'Water') {
  return generateComponentText(hx, 'HX', { fluid$: fluid, UA: ua })
}

describe('generateComponentText', () => {
  it('writes string params unquoted with their $ and units on numerics', () => {
    const text = generateComponentText(chiller, 'CHLR1', {
      ref$: 'R1234yf',
      cool$: 'EG50',
      U_tp: '3000',
      eps_zone: '0.01',
    })
    expect(text).toBe('Chiller CHLR1(ref$=R1234yf, cool$=EG50, U_tp=3000 [W/m^2-K], eps_zone=0.01)')
  })

  it('omits empty params (optional selector left unset)', () => {
    const text = generateComponentText(chiller, 'C1', { ref$: 'Water', cool$: 'Water', U_tp: '1', eps_zone: '0' })
    expect(text).not.toContain('model$')
  })

  it('includes the selector when a variant is chosen', () => {
    const text = generateComponentText(chiller, 'C1', { ref$: 'Water', cool$: 'Water', U_tp: '1', eps_zone: '0', model$: 'volumetric' })
    expect(text).toContain('model$=volumetric')
  })

  it('falls back to a suggested name when none is given', () => {
    const text = generateComponentText(chiller, '   ', { ref$: 'Water', cool$: 'Water', U_tp: '1', eps_zone: '0' })
    expect(text.startsWith('Chiller CHIL(')).toBe(true)
  })

  it('formats UA and fluid$ without inventing extra unknowns', () => {
    const cases: Array<[string, string, string]> = [
      ['10', 'Water', 'LiquidWallHX HX(fluid$=Water, UA=10 [W/K])'],
      ['conductance', 'Water', 'LiquidWallHX HX(fluid$=Water, UA=conductance)'],
      ['10 [W/K]', 'Water', 'LiquidWallHX HX(fluid$=Water, UA=10 [W/K])'],
      ['-1.5e3', 'Water', 'LiquidWallHX HX(fluid$=Water, UA=-1.5e3 [W/K])'],
      ['+8.0E-2', 'Water', 'LiquidWallHX HX(fluid$=Water, UA=+8.0E-2 [W/K])'],
      ['2 * 400', 'Water', 'LiquidWallHX HX(fluid$=Water, UA=2 * 400)'],
      ['10', 'INCOMP::MEG[0.50]', "LiquidWallHX HX(fluid$='INCOMP::MEG[0.50]', UA=10 [W/K])"],
      ['10', "'Water'", "LiquidWallHX HX(fluid$='Water', UA=10 [W/K])"],
    ]
    for (const [ua, fluid, want] of cases) {
      expect(hxLine(ua, fluid), `${fluid} UA=${ua}`).toBe(want)
    }
  })

  it('preserves offset-temperature annotations as written', () => {
    const tSpec = spec('LiquidWallHX', 'liquid', hx.ports, [
      param({ name: 'fluid$' }),
      param({ name: 'T', isString: false, unit: 'K' }),
    ])
    expect(generateComponentText(tSpec, 'S', { fluid$: 'Water', T: '20 [degC]' })).toBe(
      'LiquidWallHX S(fluid$=Water, T=20 [degC])',
    )
  })
})

describe('instance name helpers', () => {
  it('accepts legal identifiers and rejects illegal ones', () => {
    expect(isValidInstanceName('CHLR1')).toBe(true)
    expect(isValidInstanceName('_x')).toBe(true)
    expect(isValidInstanceName('1bad')).toBe(false)
    expect(isValidInstanceName('has space')).toBe(false)
    expect(isValidInstanceName('')).toBe(false)
  })

  it('suggests capitals first, else the first four chars', () => {
    expect(suggestInstanceName('MovingBoundaryEvaporator')).toBe('MBE')
    expect(suggestInstanceName('Chiller')).toBe('CHIL')
  })

  it('adds a suffix when the base name is already taken, case-insensitively', () => {
    expect(suggestInstanceName('Resistor', ['RESI'])).toBe('RESI2')
    expect(suggestInstanceName('Resistor', ['resi', 'RESI2'])).toBe('RESI3')
    expect(suggestInstanceName('Chiller', ['CHIL'])).toBe('CHIL2')
    expect(suggestInstanceName('MovingBoundaryEvaporator', ['MBE'])).toBe('MBE2')
  })

  it('rejects a case-insensitive collision', () => {
    expect(instanceNameError('r1', ['R1'])).toMatch(/collides/i)
    expect(instanceNameError('HX', ['LINE'])).toBeNull()
    expect(instanceNameError('1bad')).toMatch(/identifier/)
  })
})

describe('missingRequiredParams', () => {
  it('lists only empty required params, ignoring optional selectors', () => {
    expect(missingRequiredParams(chiller, { ref$: 'Water', U_tp: '1' })).toEqual(['cool$', 'eps_zone'])
    expect(missingRequiredParams(chiller, { ref$: 'W', cool$: 'W', U_tp: '1', eps_zone: '0' })).toEqual([])
  })
})

describe('field validation', () => {
  const ua = hx.params.find((p) => p.name === 'UA')!
  const fluid = hx.params.find((p) => p.name === 'fluid$')!

  it('accepts numerics, annotated literals, variables, and arithmetic', () => {
    expect(paramValueError(ua, '10')).toBeNull()
    expect(paramValueError(ua, '-1.5e3')).toBeNull()
    expect(paramValueError(ua, '10 [W/K]')).toBeNull()
    expect(paramValueError(ua, 'conductance')).toBeNull()
    expect(paramValueError(ua, '2 * 400')).toBeNull()
  })

  it('rejects malformed expressions so Add cannot look valid', () => {
    expect(paramValueError(ua, '10 [W/K] [')).toMatch(/valid/)
    expect(paramValueError(ua, '10 +')).toMatch(/valid/)
    expect(paramValueError(ua, '')).toBe('Required.')
  })

  it('accepts custom fluid names that generation will quote', () => {
    expect(paramValueError(fluid, 'INCOMP::MEG[0.50]')).toBeNull()
    expect(paramValueError(fluid, "'Water'")).toBeNull()
  })

  it('blocks Add when any active field is malformed', () => {
    const errors = paramValueErrors(hx, { fluid$: 'Water', UA: '10 [' })
    expect(errors.UA).toBeTruthy()
  })
})

describe('variant gating', () => {
  it('uses the declared model$ default, not the first documented variant', () => {
    expect(selectedVariant(orifice, {})).toBe('turbulent')
    expect(selectedVariant(orifice, { model$: 'laminar' })).toBe('laminar')
    expect(selectedVariant(compressor, {})).toBe('isentropic')
  })

  it('hides variant params unless their variant is active', () => {
    const isentropic = activeParams(compressor, {}).map((p) => p.name)
    expect(isentropic).not.toContain('eta_v')
    const volumetric = activeParams(compressor, { model$: 'volumetric' }).map((p) => p.name)
    expect(volumetric).toEqual(expect.arrayContaining(['eta_v', 'disp', 'rpm']))
  })

  it('requires variant params only when that variant is selected', () => {
    expect(missingRequiredParams(compressor, { eta: '0.7', fluid$: 'R134a' })).toEqual([])
    expect(missingRequiredParams(compressor, { eta: '0.7', fluid$: 'R134a', model$: 'volumetric' }))
      .toEqual(['eta_v', 'disp', 'rpm'])
  })

  it('drops stale inactive-variant values from the generated text', () => {
    const text = generateComponentText(compressor, 'C1', { eta: '0.7', fluid$: 'R134a', eta_v: '0.9' })
    expect(text).not.toContain('eta_v') // isentropic active → eta_v inactive
    expect(inactiveDraftParams(compressor, { eta: '0.7', fluid$: 'R134a', eta_v: '0.9' })).toEqual(['eta_v'])
  })
})

describe('assembleBlock', () => {
  it('joins preamble lines above the component line, dropping blanks', () => {
    expect(assembleBlock(['UA_x = ua_hx(1,2,3,4,5)', ''], 'Chiller C1(UA=UA_x)'))
      .toBe('UA_x = ua_hx(1,2,3,4,5)\nChiller C1(UA=UA_x)')
  })
})

describe('formatStringValue', () => {
  it('leaves identifiers and already-quoted values alone', () => {
    expect(formatStringValue('Water')).toBe('Water')
    expect(formatStringValue("'R134a'")).toBe("'R134a'")
  })

  it('quotes values that would be misparsed unquoted', () => {
    expect(formatStringValue('INCOMP::MEG[0.50]')).toBe("'INCOMP::MEG[0.50]'")
  })
})

describe('isPlainNumericLiteral', () => {
  it('accepts signed and scientific forms the lexer would', () => {
    expect(isPlainNumericLiteral('10')).toBe(true)
    expect(isPlainNumericLiteral('-2')).toBe(true)
    expect(isPlainNumericLiteral('.5')).toBe(true)
    expect(isPlainNumericLiteral('1e-3')).toBe(true)
    expect(isPlainNumericLiteral('conductance')).toBe(false)
    expect(isPlainNumericLiteral('10 [W/K]')).toBe(false)
  })
})
