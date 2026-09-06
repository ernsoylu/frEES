// End-to-end DTO parity for the Phase-6 component surface: the two
// `SolveResponse` fields that were typed in `api.ts` but never populated —
// `components` (the datasheet) and `cyclePath` (the property-plot overlay).
//
// Both fixtures below are the **verbatim** output of the Rust boundary
// (`frees::solve`), not hand-written JSON, so a change on either side of
// the seam fails here rather than silently rendering nothing. They were
// captured by running the sources in the comments through
// `crates/frees` and pasting the result.
import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('./wasm/engineClient', () => ({
  wasmSolve: vi.fn(),
  wasmSolveTable: vi.fn(),
  wasmMonteCarlo: vi.fn(),
  wasmOptimize: vi.fn(),
  wasmOptimizeMulti: vi.fn(),
  wasmCurveFit: vi.fn(),
  wasmParameterFit: vi.fn(),
  wasmPidTune: vi.fn(),
  wasmExtractPlant: vi.fn(),
  wasmCheck: vi.fn(),
}))

import { solve, DEFAULT_STOP_CRITERIA } from './api'
import { wasmSolve } from './wasm/engineClient'
import { groupComponents } from './Workspace'
import { buildPropertyFigure } from './plots/figure'
import type { DiagramResponse } from './api'
import type { StateTable } from './plots/stateTable'
import { defaultFormat, type PropertyConfig } from './plots/types'

const solveMock = vi.mocked(wasmSolve)

// Source:
//   VoltageSource V1(E = 12)
//   Resistor      R1(R = 10)
//   Resistor      R2(R = R_load)
//   Ground        G1()
//   connect(V1.p, R1.a)
//   connect(R1.b, R2.a)
//   connect(R2.b, V1.n, G1.port)
//   R_load = 20
const SOLVE_COMPONENTS = '{"blocks": [{"equations": ["COMPONENT ground g1: port.V = 0"], "index": 0, "variables": ["g1.port.v"]}, {"equations": ["CONNECT r2.b, v1.n, g1.port: r2$b.v = g1$port.v"], "index": 1, "variables": ["r2.b.v"]}, {"equations": ["CONNECT r2.b, v1.n, g1.port: r2$b.v = v1$n.v"], "index": 2, "variables": ["v1.n.v"]}, {"equations": ["COMPONENT voltagesource v1: p.V - n.V = E"], "index": 3, "variables": ["v1.p.v"]}, {"equations": ["R_load = 20"], "index": 4, "variables": ["R_load"]}, {"equations": ["CONNECT v1.p, r1.a: v1$p.v = r1$a.v"], "index": 5, "variables": ["r1.a.v"]}, {"equations": ["COMPONENT resistor r1: a.V - b.V = R * a.I", "COMPONENT resistor r1: a.I + b.I = 0", "COMPONENT resistor r2: a.V - b.V = R * a.I", "CONNECT r1.b, r2.a: r1$b.v = r2$a.v", "CONNECT r1.b, r2.a: sum(i) = 0"], "index": 6, "variables": ["r1.a.i", "r1.b.i", "r1.b.v", "r2.a.i", "r2.a.v"]}, {"equations": ["CONNECT v1.p, r1.a: sum(i) = 0"], "index": 7, "variables": ["v1.p.i"]}, {"equations": ["COMPONENT voltagesource v1: p.I + n.I = 0"], "index": 8, "variables": ["v1.n.i"]}, {"equations": ["COMPONENT resistor r2: a.I + b.I = 0"], "index": 9, "variables": ["r2.b.i"]}, {"equations": ["CONNECT r2.b, v1.n, g1.port: sum(i) = 0"], "index": 10, "variables": ["g1.port.i"]}], "components": [{"name": "V1", "params": [{"name": "e", "ref": "12", "units": null, "value": 12.0}], "type": "VoltageSource"}, {"name": "R1", "params": [{"name": "r", "ref": "10", "units": null, "value": 10.0}], "type": "Resistor"}, {"name": "R2", "params": [{"name": "r", "ref": "R_load", "units": "\\u03a9", "value": 20.0}], "type": "Resistor"}, {"name": "G1", "params": [], "type": "Ground"}], "cyclePath": [], "error": null, "errorLine": null, "failedBlockIndex": null, "residuals": [{"equation": "COMPONENT voltagesource v1: p.V - n.V = E", "value": 0.0}, {"equation": "COMPONENT voltagesource v1: p.I + n.I = 0", "value": 0.0}, {"equation": "COMPONENT resistor r1: a.V - b.V = R * a.I", "value": 0.0}, {"equation": "COMPONENT resistor r1: a.I + b.I = 0", "value": 0.0}, {"equation": "COMPONENT resistor r2: a.V - b.V = R * a.I", "value": 0.0}, {"equation": "COMPONENT resistor r2: a.I + b.I = 0", "value": 0.0}, {"equation": "COMPONENT ground g1: port.V = 0", "value": 0.0}, {"equation": "CONNECT v1.p, r1.a: v1$p.v = r1$a.v", "value": 0.0}, {"equation": "CONNECT v1.p, r1.a: sum(i) = 0", "value": 0.0}, {"equation": "CONNECT r1.b, r2.a: r1$b.v = r2$a.v", "value": 0.0}, {"equation": "CONNECT r1.b, r2.a: sum(i) = 0", "value": 0.0}, {"equation": "CONNECT r2.b, v1.n, g1.port: r2$b.v = v1$n.v", "value": 0.0}, {"equation": "CONNECT r2.b, v1.n, g1.port: r2$b.v = g1$port.v", "value": 0.0}, {"equation": "CONNECT r2.b, v1.n, g1.port: sum(i) = 0", "value": 0.0}, {"equation": "R_load = 20", "value": 0.0}], "solutions": [{"maxResidual": 0.0, "variables": [{"name": "g1.port.i", "units": "", "value": 0.0}, {"name": "g1.port.v", "units": "", "value": 0.0}, {"name": "r1.a.i", "units": "", "value": 0.4}, {"name": "r1.a.v", "units": "", "value": 12.0}, {"name": "r1.b.i", "units": "", "value": -0.4}, {"name": "r1.b.v", "units": "", "value": 8.0}, {"name": "r2.a.i", "units": "", "value": 0.4}, {"name": "r2.a.v", "units": "", "value": 8.0}, {"name": "r2.b.i", "units": "", "value": -0.4}, {"name": "r2.b.v", "units": "", "value": 0.0}, {"name": "R_load", "units": "\\u03a9", "value": 20.0}, {"name": "v1.n.i", "units": "", "value": 0.4}, {"name": "v1.n.v", "units": "", "value": 0.0}, {"name": "v1.p.i", "units": "", "value": -0.4}, {"name": "v1.p.v", "units": "", "value": 12.0}]}], "stats": {"blocks": 11, "elapsedMillis": 103, "equations": 15, "iterations": 14, "maxResidual": 0.0, "unknowns": 15}, "success": true, "unitWarnings": ["COMPONENT resistor r1: a.V - b.V = R * a.I: the units of the left side [kg m^2 s^-3 A^-1] do not match the right side [A]."], "variables": [{"name": "g1.port.i", "units": "", "value": 0.0}, {"name": "g1.port.v", "units": "", "value": 0.0}, {"name": "r1.a.i", "units": "", "value": 0.4}, {"name": "r1.a.v", "units": "", "value": 12.0}, {"name": "r1.b.i", "units": "", "value": -0.4}, {"name": "r1.b.v", "units": "", "value": 8.0}, {"name": "r2.a.i", "units": "", "value": 0.4}, {"name": "r2.a.v", "units": "", "value": 8.0}, {"name": "r2.b.i", "units": "", "value": -0.4}, {"name": "r2.b.v", "units": "", "value": 0.0}, {"name": "R_load", "units": "\\u03a9", "value": 20.0}, {"name": "v1.n.i", "units": "", "value": 0.4}, {"name": "v1.n.v", "units": "", "value": 0.0}, {"name": "v1.p.i", "units": "", "value": -0.4}, {"name": "v1.p.v", "units": "", "value": 12.0}]}'

// Source (request {"fillMissing": true}):
//   T1 = 320 [K]
//   P1 = 101325 [Pa]
//   T2 = 400 [K]
//   P2 = 101325 [Pa]
const SOLVE_CYCLE = '{"blocks": [{"equations": ["T1 = 320 [K]"], "index": 0, "variables": ["T1"]}, {"equations": ["P1 = 101325 [Pa]"], "index": 1, "variables": ["P1"]}, {"equations": ["T2 = 400 [K]"], "index": 2, "variables": ["T2"]}, {"equations": ["P2 = 101325 [Pa]"], "index": 3, "variables": ["P2"]}], "components": [], "cyclePath": [{"P": 101325.0, "T": 319.99999998097184, "h": 196248.22335783625, "s": 662.8125706348566, "v": 0.0010106861445603906}, {"P": 101325.0, "T": 337.9064358181873, "h": 271158.9274119367, "s": 890.5921188110644, "v": 0.0010196974652496727}, {"P": 101325.0, "T": 356.77121087315606, "h": 350257.0725103925, "s": 1118.3716669872724, "v": 0.0010314589705849124}, {"P": 101325.0, "T": 373.12429215893866, "h": 433695.5568654983, "s": 1346.1512151634804, "v": 0.011890824906032677}, {"P": 101325.0, "T": 373.12429215893866, "h": 518685.63482237177, "s": 1573.9307633396884, "v": 0.0748727155069778}, {"P": 101325.0, "T": 373.12429215893866, "h": 603675.712386918, "s": 1801.7103115158961, "v": 0.13785460581718886}, {"P": 101325.0, "T": 373.12429215893866, "h": 688665.790343792, "s": 2029.4898596921043, "v": 0.2008364964181343}, {"P": 101325.0, "T": 373.12429215893866, "h": 773655.8683006654, "s": 2257.269407868312, "v": 0.26381838701907945}, {"P": 101325.0, "T": 373.12429215893866, "h": 858645.9456690485, "s": 2485.04895604452, "v": 0.3268002771839238}, {"P": 101325.0, "T": 373.12429215893866, "h": 943636.0236259219, "s": 2712.8285042207276, "v": 0.3897821677848689}, {"P": 101325.0, "T": 373.12429215893866, "h": 1028626.1015827956, "s": 2940.608052396936, "v": 0.4527640583858142}, {"P": 101325.0, "T": 373.12429215893866, "h": 1113616.1795396688, "s": 3168.3876005731436, "v": 0.5157459489867592}, {"P": 101325.0, "T": 373.12429215893866, "h": 1198606.2574965423, "s": 3396.167148749352, "v": 0.5787278395877042}, {"P": 101325.0, "T": 373.12429215893866, "h": 1283596.3354534162, "s": 3623.9466969255595, "v": 0.6417097301886497}, {"P": 101325.0, "T": 373.12429215893866, "h": 1368586.4134102901, "s": 3851.7262451017677, "v": 0.7046916207895951}, {"P": 101325.0, "T": 373.12429215893866, "h": 1453576.4905825085, "s": 4079.5057932779755, "v": 0.7676735108090716}, {"P": 101325.0, "T": 373.12429215893866, "h": 1538566.5685393824, "s": 4307.285341454183, "v": 0.830655401410017}, {"P": 101325.0, "T": 373.12429215893866, "h": 1623556.6461039288, "s": 4535.064889630392, "v": 0.8936372917202283}, {"P": 101325.0, "T": 373.12429215893866, "h": 1708546.7240608027, "s": 4762.844437806599, "v": 0.9566191823211737}, {"P": 101325.0, "T": 373.12429215893866, "h": 1793536.8020176766, "s": 4990.623985982807, "v": 1.0196010729221192}, {"P": 101325.0, "T": 373.12429215893866, "h": 1878526.8799745496, "s": 5218.403534159015, "v": 1.0825829635230637}, {"P": 101325.0, "T": 373.12429215893866, "h": 1963516.9579314226, "s": 5446.183082335223, "v": 1.1455648541240087}, {"P": 101325.0, "T": 373.12429215893866, "h": 2048507.0358882966, "s": 5673.962630511431, "v": 1.2085467447249543}, {"P": 101325.0, "T": 373.12429215893866, "h": 2133497.1138451714, "s": 5901.742178687639, "v": 1.2715286353259003}, {"P": 101325.0, "T": 373.12429215893866, "h": 2218487.1918020444, "s": 6129.521726863847, "v": 1.334510525926845}, {"P": 101325.0, "T": 373.12429215893866, "h": 2303477.2697589174, "s": 6357.301275040055, "v": 1.3974924165277898}, {"P": 101325.0, "T": 373.12429215893866, "h": 2388467.3477157913, "s": 6585.080823216263, "v": 1.4604743071287352}, {"P": 101325.0, "T": 373.12429215893866, "h": 2473457.4256726643, "s": 6812.860371392471, "v": 1.52345619772968}, {"P": 101325.0, "T": 373.12429215893866, "h": 2558447.502060229, "s": 7040.639919568679, "v": 1.5864380871676886}, {"P": 101325.0, "T": 373.12429215893866, "h": 2643437.580017102, "s": 7268.419467744887, "v": 1.6494199777686338}, {"P": 101325.0, "T": 399.9999997104144, "h": 2730301.0230699033, "s": 7496.1990159210945, "v": 1.801983072561103}, {"P": 101325.0, "T": 373.12429215893866, "h": 2643437.580017102, "s": 7268.419467744887, "v": 1.6494199777686338}, {"P": 101325.0, "T": 373.12429215893866, "h": 2558447.502060229, "s": 7040.639919568679, "v": 1.5864380871676886}, {"P": 101325.0, "T": 373.12429215893866, "h": 2473457.4256726643, "s": 6812.86037139247, "v": 1.52345619772968}, {"P": 101325.0, "T": 373.12429215893866, "h": 2388467.3477157913, "s": 6585.080823216263, "v": 1.4604743071287352}, {"P": 101325.0, "T": 373.12429215893866, "h": 2303477.2697589174, "s": 6357.301275040055, "v": 1.3974924165277898}, {"P": 101325.0, "T": 373.12429215893866, "h": 2218487.1918020444, "s": 6129.521726863847, "v": 1.334510525926845}, {"P": 101325.0, "T": 373.12429215893866, "h": 2133497.1138451714, "s": 5901.742178687638, "v": 1.2715286353259003}, {"P": 101325.0, "T": 373.12429215893866, "h": 2048507.0358882966, "s": 5673.962630511431, "v": 1.2085467447249543}, {"P": 101325.0, "T": 373.12429215893866, "h": 1963516.9579314226, "s": 5446.183082335223, "v": 1.1455648541240087}, {"P": 101325.0, "T": 373.12429215893866, "h": 1878526.8799745496, "s": 5218.403534159015, "v": 1.0825829635230637}, {"P": 101325.0, "T": 373.12429215893866, "h": 1793536.8020176766, "s": 4990.623985982807, "v": 1.0196010729221192}, {"P": 101325.0, "T": 373.12429215893866, "h": 1708546.7240608027, "s": 4762.8444378066, "v": 0.9566191823211737}, {"P": 101325.0, "T": 373.12429215893866, "h": 1623556.6461039288, "s": 4535.064889630392, "v": 0.8936372917202283}, {"P": 101325.0, "T": 373.12429215893866, "h": 1538566.5685393824, "s": 4307.285341454183, "v": 0.830655401410017}, {"P": 101325.0, "T": 373.12429215893866, "h": 1453576.4905825085, "s": 4079.5057932779755, "v": 0.7676735108090716}, {"P": 101325.0, "T": 373.12429215893866, "h": 1368586.4134102901, "s": 3851.7262451017677, "v": 0.7046916207895951}, {"P": 101325.0, "T": 373.12429215893866, "h": 1283596.3354534162, "s": 3623.9466969255595, "v": 0.6417097301886497}, {"P": 101325.0, "T": 373.12429215893866, "h": 1198606.2574965423, "s": 3396.1671487493522, "v": 0.5787278395877042}, {"P": 101325.0, "T": 373.12429215893866, "h": 1113616.1795396688, "s": 3168.3876005731436, "v": 0.5157459489867592}, {"P": 101325.0, "T": 373.12429215893866, "h": 1028626.1015827956, "s": 2940.608052396936, "v": 0.4527640583858142}, {"P": 101325.0, "T": 373.12429215893866, "h": 943636.0236259219, "s": 2712.828504220728, "v": 0.3897821677848689}, {"P": 101325.0, "T": 373.12429215893866, "h": 858645.9456690485, "s": 2485.0489560445203, "v": 0.3268002771839238}, {"P": 101325.0, "T": 373.12429215893866, "h": 773655.8683006654, "s": 2257.2694078683116, "v": 0.26381838701907945}, {"P": 101325.0, "T": 373.12429215893866, "h": 688665.790343792, "s": 2029.489859692104, "v": 0.2008364964181343}, {"P": 101325.0, "T": 373.12429215893866, "h": 603675.712386918, "s": 1801.7103115158961, "v": 0.13785460581718886}, {"P": 101325.0, "T": 373.12429215893866, "h": 518685.63482237177, "s": 1573.9307633396884, "v": 0.0748727155069778}, {"P": 101325.0, "T": 373.12429215893866, "h": 433695.5568654983, "s": 1346.1512151634797, "v": 0.011890824906032677}, {"P": 101325.0, "T": 356.77121087315606, "h": 350257.0725103925, "s": 1118.371666987272, "v": 0.0010314589705849124}, {"P": 101325.0, "T": 337.9064358181873, "h": 271158.9274119367, "s": 890.5921188110642, "v": 0.0010196974652496727}, {"P": 101325.0, "T": 319.99999998097184, "h": 196248.22335783625, "s": 662.8125706348565, "v": 0.0010106861445603906}], "error": null, "errorLine": null, "failedBlockIndex": null, "residuals": [{"equation": "T1 = 320 [K]", "value": 0.0}, {"equation": "P1 = 101325 [Pa]", "value": 0.0}, {"equation": "T2 = 400 [K]", "value": 0.0}, {"equation": "P2 = 101325 [Pa]", "value": 0.0}], "solutions": [{"maxResidual": 0.0, "variables": [{"name": "h1", "units": "J/kg", "value": 196248.22335783625}, {"name": "h2", "units": "J/kg", "value": 2730301.0230699033}, {"name": "P1", "units": "Pa", "value": 101325.0}, {"name": "P2", "units": "Pa", "value": 101325.0}, {"name": "rho1", "units": "kg/m^3", "value": 989.4268417372649}, {"name": "rho2", "units": "kg/m^3", "value": 0.5549441696911896}, {"name": "s1", "units": "J/kg-K", "value": 662.8125706348566}, {"name": "s2", "units": "J/kg-K", "value": 7496.1990159210945}, {"name": "T1", "units": "K", "value": 320.0}, {"name": "T2", "units": "K", "value": 400.0}, {"name": "u1", "units": "J/kg", "value": 196145.81558423868}, {"name": "u2", "units": "J/kg", "value": 2547715.0882426496}, {"name": "v1", "units": "m^3/kg", "value": 0.0010106861445603906}, {"name": "v2", "units": "m^3/kg", "value": 1.801983072561103}, {"name": "x1", "units": "", "value": -0.09874242738787939}, {"name": "x2", "units": "", "value": 1.0242732092948967}]}], "stats": {"blocks": 4, "elapsedMillis": 11, "equations": 4, "iterations": 4, "maxResidual": 0.0, "unknowns": 16}, "success": true, "unitWarnings": [], "variables": [{"name": "h1", "units": "J/kg", "value": 196248.22335783625}, {"name": "h2", "units": "J/kg", "value": 2730301.0230699033}, {"name": "P1", "units": "Pa", "value": 101325.0}, {"name": "P2", "units": "Pa", "value": 101325.0}, {"name": "rho1", "units": "kg/m^3", "value": 989.4268417372649}, {"name": "rho2", "units": "kg/m^3", "value": 0.5549441696911896}, {"name": "s1", "units": "J/kg-K", "value": 662.8125706348566}, {"name": "s2", "units": "J/kg-K", "value": 7496.1990159210945}, {"name": "T1", "units": "K", "value": 320.0}, {"name": "T2", "units": "K", "value": 400.0}, {"name": "u1", "units": "J/kg", "value": 196145.81558423868}, {"name": "u2", "units": "J/kg", "value": 2547715.0882426496}, {"name": "v1", "units": "m^3/kg", "value": 0.0010106861445603906}, {"name": "v2", "units": "m^3/kg", "value": 1.801983072561103}, {"name": "x1", "units": "", "value": -0.09874242738787939}, {"name": "x2", "units": "", "value": 1.0242732092948967}]}'

function engineReturns(payload: string) {
  solveMock.mockResolvedValueOnce(JSON.parse(payload) as never)
}

function runSolve(fillMissing = false) {
  return solve('', DEFAULT_STOP_CRITERIA, [], false, 'SI', fillMissing)
}

beforeEach(() => {
  solveMock.mockReset()
})

describe('SolveResponse.components', () => {
  it('arrives typed from the boundary and survives mapSolveData', async () => {
    engineReturns(SOLVE_COMPONENTS)
    const r = await runSolve()

    expect(r.success).toBe(true)
    expect(r.components).toHaveLength(4)
    // ComponentResult: {name, type, params} with the SOURCE spelling of both
    // identities (the AST lowercases them for registry lookup).
    expect(r.components!.map((c) => c.name)).toEqual(['V1', 'R1', 'R2', 'G1'])
    expect(r.components!.map((c) => c.type)).toEqual([
      'VoltageSource', 'Resistor', 'Resistor', 'Ground',
    ])
    // ComponentParamResult: a symbolic binding shows the symbol AND its value.
    expect(r.components![2].params).toEqual([
      { name: 'r', ref: 'R_load', value: 20, units: 'Ω' },
    ])
    // A literal binding resolves to its own number and has no unit.
    expect(r.components![1].params[0]).toEqual({ name: 'r', ref: '10', value: 10, units: null })
    // A component with no parameters still reports an empty list, never null.
    expect(r.components![3].params).toEqual([])
  })

  it('renders as the Variable Explorer datasheet', async () => {
    engineReturns(SOLVE_COMPONENTS)
    const r = await runSolve()

    // This is what Workspace.tsx does with the two payload halves.
    const { plain, components } = groupComponents(r.variables, r.components ?? [])

    // Every expanded port member is grouped under its instance, prefix stripped.
    expect(components.map((c) => c.name)).toEqual(['G1', 'R1', 'R2', 'V1'])
    const r1 = components.find((c) => c.name === 'R1')!
    expect(r1.type).toBe('Resistor')
    expect(r1.members.map((m) => m.label)).toEqual(['a.i', 'a.v', 'b.i', 'b.v'])
    expect(r1.params.map((p) => p.ref)).toEqual(['10'])
    // …and the document's own scalar stays out of the component groups.
    expect(plain.map((p) => p.name)).toEqual(['R_load'])
  })

  it('is an empty array, not undefined, for a document with no components', async () => {
    engineReturns(SOLVE_CYCLE)
    const r = await runSolve(true)
    expect(r.components).toEqual([])
    expect(groupComponents(r.variables, r.components ?? []).components).toEqual([])
  })
})

describe('SolveResponse.cyclePath', () => {
  it('arrives populated when fillMissing is requested', async () => {
    engineReturns(SOLVE_CYCLE)
    const r = await runSolve(true)

    expect(r.success).toBe(true)
    expect(r.cyclePath!.length).toBeGreaterThan(2)
    // Every point is a {property: number} map on the plot's own property names.
    for (const point of r.cyclePath!) {
      for (const key of ['T', 'P', 'h', 's', 'v']) {
        expect(typeof point[key]).toBe('number')
        expect(Number.isFinite(point[key])).toBe(true)
      }
    }
    // Fill-missing also injected the state properties as ordinary rows, with
    // the units stamped from the property identity.
    const h1 = r.variables.find((v) => v.name === 'h1')!
    expect(h1.units).toBe('J/kg')
    expect(h1.value).toBeGreaterThan(0)
  })

  it('drives the property figure overlay as a "Cycle Path" line', async () => {
    engineReturns(SOLVE_CYCLE)
    const r = await runSolve(true)

    // A minimal P–h diagram: the figure builder only needs the axis identity
    // and (empty) curve arrays to place a state overlay.
    const diagram: DiagramResponse = {
      fluid: 'Water',
      kind: 'ph',
      xProperty: 'h',
      yProperty: 'P',
      xLog: false,
      yLog: true,
      dome: [],
      isolines: [],
      markers: [],
    }

    const states: StateTable = {
      indices: [1, 2],
      columns: ['T', 'P', 'h'],
      values: {
        1: { T: 320, P: 101325, h: 196248 },
        2: { T: 400, P: 101325, h: 532000 },
      },
    }
    const config: PropertyConfig = {
      fluid: 'Water',
      diagram: 'ph',
      quality: false,
      isolines: false,
      overlayStates: true,
      connectStates: true,
      closeCycle: false,
    }
    const figure = buildPropertyFigure(diagram, config, defaultFormat('property'), states, 'dark', r.cyclePath)
    const path = figure.data.find((t) => t.name === 'Cycle Path')
    expect(path, JSON.stringify(figure.data.map((t) => t.name))).toBeTruthy()
    expect(path!.x!.length).toBe(r.cyclePath!.length)
    // Without the payload the same call falls back to the straight-line
    // "Cycle Connections" between the two state markers.
    const bare = buildPropertyFigure(diagram, config, defaultFormat('property'), states, 'dark', undefined)
    expect(bare.data.find((t) => t.name === 'Cycle Path')).toBeFalsy()
  })
})
