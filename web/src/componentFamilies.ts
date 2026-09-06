// Related-model groupings for the Component Wizard. Search tags on the
// generated catalog remain the discovery index; this table is the comparison
// the wizard shows once a family member is selected. No runtime aliases.
//
// Rows live in one TSV blob so the comparison columns are not copy-pasted
// as object literals (Sonar new-code duplication).

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

// Family header: id<TAB>label (2 columns). Member: 7 columns.
const FAMILY_TSV = `\
fan	Fans
Fan	fluid$, dP0, Q0, eta	in, out	mass conserved; ΔP from (Q0, dP0)	shaft work added to enthalpy	steady	quadratic ΔP vs volumetric flow around the design point
FanCurve	rho, dP0, Q0	in, out	mass conserved; same quadratic ΔP	pressure only — no enthalpy rise	steady	constant density; no work term
FanMap	rho, map$ (ΔP vs Q)	in, out	mass conserved; ΔP from the table	pressure only — no enthalpy rise	steady	caller supplies a TABLE/FUNCTION map
compressor	Compressors
Compressor	fluid$, eta; model$ (isentropic / volumetric)	in, out	mass conserved; volumetric variant sets mdot from rpm	isentropic work / eta; named output W	steady	real-fluid isentropic path; volumetric needs disp and rpm
CompressorMap	fluid$, map_eta$ (eta vs pressure ratio)	in, out	mass conserved	same isentropic path; eta from the map	steady	TABLE/FUNCTION of eta vs out.P/in.P
TwoPhaseCompressor	fluid$, eta; model$ (isentropic / volumetric)	in, out (twophase)	mass conserved on refrigerant ports	isentropic work / eta; named output W	steady	two-phase connector family; same physics as Compressor
pump	Pumps (thermofluid)
Pump	fluid$, eta	in, out	mass conserved	v·ΔP / eta added to enthalpy; named output W	steady	incompressible work from specific volume at the inlet
PumpMap	rho, map$ (head vs Q)	in, out	mass conserved; ΔP = ρ g head(Q)	pressure only — no enthalpy rise	steady	constant density; tabulated head curve
liquid-pump	Pumps (liquid coolant)
LiquidPump	fluid$, eta	in, out (liquid)	mass conserved on liquid ports	pump work on the liquid stream	steady	single-phase coolant connector; TMS loops
LiquidPumpMap	rho, eta, map$	in, out (liquid)	mass conserved; head from the map	map head plus efficiency	steady	TABLE/FUNCTION map on a liquid connector
`

function parseFamilies(src: string): ModelFamily[] {
  const out: ModelFamily[] = []
  for (const line of src.trim().split('\n')) {
    const cols = line.split('\t')
    if (cols.length === 2) {
      out.push({ id: cols[0], label: cols[1], members: [] })
      continue
    }
    const last = out[out.length - 1]
    if (!last || cols.length < 7) continue
    last.members.push({
      type: cols[0],
      requiredData: cols[1],
      ports: cols[2],
      flow: cols[3],
      energy: cols[4],
      regime: cols[5],
      assumptions: cols[6],
    })
  }
  return out
}

export const MODEL_FAMILIES: ModelFamily[] = parseFamilies(FAMILY_TSV)

export function familyOf(type: string): ModelFamily | undefined {
  const key = type.toLowerCase()
  return MODEL_FAMILIES.find((f) => f.members.some((m) => m.type.toLowerCase() === key))
}
