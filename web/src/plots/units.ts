/**
 * Display unit choices per thermodynamic property. All stored values are SI;
 * a choice converts for display as value * scale + offset. Used by the State
 * Points table and the plot axis unit pickers.
 */

export interface UnitChoice {
  id: string
  label: string
  scale: number
  offset: number
}

const BTU_PER_LBM = 2326
const CAL_KG = 4186.8
const FT3_PER_LBM = 16.018463

function u(id: string, scale: number, offset = 0): UnitChoice {
  return { id, label: id, scale, offset }
}

export const PROPERTY_UNITS: Record<string, UnitChoice[]> = {
  T: [
    u('K', 1),
    u('°C', 1, -273.15),
    u('°F', 1.8, -459.67),
    u('R', 1.8),
  ],
  P: [
    u('Pa', 1),
    u('kPa', 1e-3),
    u('MPa', 1e-6),
    u('bar', 1e-5),
    u('atm', 1 / 101_325),
    u('psia', 1 / 6_894.757),
    u('psig', 1 / 6_894.757, -14.6959),
    u('mmHg', 1 / 133.322),
  ],
  h: [
    u('J/kg', 1),
    u('kJ/kg', 1e-3),
    u('MJ/kg', 1e-6),
    u('Btu/lbm', 1 / BTU_PER_LBM),
    u('kcal/kg', 1 / CAL_KG),
  ],
  s: [
    u('J/kg·K', 1),
    u('kJ/kg·K', 1e-3),
    u('J/kg-C', 1),
    u('kJ/kg-C', 1e-3),
    u('Btu/lbm·R', 1 / CAL_KG),
    u('kcal/kg·K', 1 / CAL_KG),
    u('kcal/kg-C', 1 / CAL_KG),
  ],
  v: [
    u('m³/kg', 1),
    u('L/kg', 1e3),
    u('ft³/lbm', FT3_PER_LBM),
  ],
  rho: [
    u('kg/m³', 1),
    u('g/cm³', 1e-3),
    u('lbm/ft³', 1 / FT3_PER_LBM),
  ],
  w: [
    u('kg/kg', 1),
    u('g/kg', 1e3),
    u('gr/lbm', 7000),
  ],
  x: [u('-', 1), u('%', 100)],
  rh: [u('-', 1), u('%', 100)],
}

// Internal energy, wet bulb and dew point share other properties' units.
PROPERTY_UNITS.u = PROPERTY_UNITS.h
PROPERTY_UNITS.Twb = PROPERTY_UNITS.T
PROPERTY_UNITS.Tdp = PROPERTY_UNITS.T

/** Default display unit on plots: textbook engineering units. */
export function defaultUnitId(property: string, celsius: boolean): string {
  switch (property) {
    case 'T':
    case 'Twb':
    case 'Tdp':
      return celsius ? '°C' : 'K'
    case 'P':
      return 'kPa'
    case 'h':
    case 'u':
      return 'kJ/kg'
    case 's':
      return 'kJ/kg·K'
    default:
      return PROPERTY_UNITS[property]?.[0]?.id ?? '-'
  }
}

function normalizeId(id: string): string {
  return id
    .replaceAll('°', '')
    .replaceAll('·', '-')
    .replace(/[\^³]/g, '3')
    .replaceAll('*', '')
    .trim()
    .toLowerCase()
}

/** Resolves a unit selection, falling back to the property default. */
export function resolveUnit(
  property: string,
  selectedId: string | null | undefined,
  celsius: boolean,
): UnitChoice {
  const choices = PROPERTY_UNITS[property] ?? [u('-', 1)]
  const id = selectedId ?? defaultUnitId(property, celsius)
  const normId = normalizeId(id)
  return choices.find((c) => normalizeId(c.id) === normId) ?? choices[0]
}

export function unitIdsFor(property: string): string[] {
  return (PROPERTY_UNITS[property] ?? []).map((c) => c.id)
}

/** Returns true if the resolved unit has a non-zero offset (e.g. °C, °F, psig). */
export function isOffsetUnit(
  property: string,
  unitId?: string | null,
  celsius = false,
): boolean {
  const unit = resolveUnit(property, unitId, celsius)
  return unit.offset !== 0
}
