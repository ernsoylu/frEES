// Related-model groupings for the Component Wizard. Search tags on the
// generated catalog remain the discovery index; this table is the comparison
// the wizard shows once a family member is selected. No runtime aliases.

export interface ModelChoice {
  type: string
  requiredData: string
  ports: string
  flow: string
  energy: string
  regime: string
  assumptions: string
}

export interface ModelFamily {
  id: string
  label: string
  members: ModelChoice[]
}

export const MODEL_FAMILIES: ModelFamily[] = [
  {
    id: 'fan',
    label: 'Fans',
    members: [
      {
        type: 'Fan',
        requiredData: 'fluid$, dP0, Q0, eta',
        ports: 'in, out',
        flow: 'mass conserved; ΔP from (Q0, dP0)',
        energy: 'shaft work added to enthalpy',
        regime: 'steady',
        assumptions: 'quadratic ΔP vs volumetric flow around the design point',
      },
      {
        type: 'FanCurve',
        requiredData: 'rho, dP0, Q0',
        ports: 'in, out',
        flow: 'mass conserved; same quadratic ΔP',
        energy: 'pressure only — no enthalpy rise',
        regime: 'steady',
        assumptions: 'constant density; no work term',
      },
      {
        type: 'FanMap',
        requiredData: 'rho, map$ (ΔP vs Q)',
        ports: 'in, out',
        flow: 'mass conserved; ΔP from the table',
        energy: 'pressure only — no enthalpy rise',
        regime: 'steady',
        assumptions: 'caller supplies a TABLE/FUNCTION map',
      },
    ],
  },
  {
    id: 'compressor',
    label: 'Compressors',
    members: [
      {
        type: 'Compressor',
        requiredData: 'fluid$, eta; model$ (isentropic / volumetric)',
        ports: 'in, out',
        flow: 'mass conserved; volumetric variant sets mdot from rpm',
        energy: 'isentropic work / eta; named output W',
        regime: 'steady',
        assumptions: 'real-fluid isentropic path; volumetric needs disp and rpm',
      },
      {
        type: 'CompressorMap',
        requiredData: 'fluid$, map_eta$ (eta vs pressure ratio)',
        ports: 'in, out',
        flow: 'mass conserved',
        energy: 'same isentropic path; eta from the map',
        regime: 'steady',
        assumptions: 'TABLE/FUNCTION of eta vs out.P/in.P',
      },
      {
        type: 'TwoPhaseCompressor',
        requiredData: 'fluid$, eta; model$ (isentropic / volumetric)',
        ports: 'in, out (twophase)',
        flow: 'mass conserved on refrigerant ports',
        energy: 'isentropic work / eta; named output W',
        regime: 'steady',
        assumptions: 'two-phase connector family; same physics as Compressor',
      },
    ],
  },
  {
    id: 'pump',
    label: 'Pumps (thermofluid)',
    members: [
      {
        type: 'Pump',
        requiredData: 'fluid$, eta',
        ports: 'in, out',
        flow: 'mass conserved',
        energy: 'v·ΔP / eta added to enthalpy; named output W',
        regime: 'steady',
        assumptions: 'incompressible work from specific volume at the inlet',
      },
      {
        type: 'PumpMap',
        requiredData: 'rho, map$ (head vs Q)',
        ports: 'in, out',
        flow: 'mass conserved; ΔP = ρ g head(Q)',
        energy: 'pressure only — no enthalpy rise',
        regime: 'steady',
        assumptions: 'constant density; tabulated head curve',
      },
    ],
  },
  {
    id: 'liquid-pump',
    label: 'Pumps (liquid coolant)',
    members: [
      {
        type: 'LiquidPump',
        requiredData: 'fluid$, eta',
        ports: 'in, out (liquid)',
        flow: 'mass conserved on liquid ports',
        energy: 'pump work on the liquid stream',
        regime: 'steady',
        assumptions: 'single-phase coolant connector; TMS loops',
      },
      {
        type: 'LiquidPumpMap',
        requiredData: 'rho, eta, map$',
        ports: 'in, out (liquid)',
        flow: 'mass conserved; head from the map',
        energy: 'map head plus efficiency',
        regime: 'steady',
        assumptions: 'TABLE/FUNCTION map on a liquid connector',
      },
    ],
  },
]

export function familyOf(type: string): ModelFamily | undefined {
  const key = type.toLowerCase()
  return MODEL_FAMILIES.find((f) => f.members.some((m) => m.type.toLowerCase() === key))
}
