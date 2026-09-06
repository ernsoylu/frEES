// GENERATED FILE - DO NOT EDIT DIRECTLY.
// Compiled from src/docs/reference/components/**/*.md by scripts/compile-docs.js
// (npm run compile-docs). Structured specs for the Component Browser/Wizard.

export interface ComponentParam {
  name: string;          // e.g. "U_tp", "fluid$"
  isString: boolean;     // name ends in "$"
  isSelector: boolean;   // a model$ variant selector
  isMap: boolean;        // a string param naming a TABLE/FUNCTION map (map$, map_eta$)
  unit: string;          // frees-safe unit token, or "" if none/dimensionless
  description: string;
  required: boolean;
  values: string[];      // selector option values (model variants), else []
  variants: string[];    // variants that require this param; [] = shared/always-shown
  defaultValue?: string; // engine-declared default (PARAM model$ = isentropic), else ""
}

export interface ComponentVariant {
  name: string;          // model$ value, e.g. "volumetric"
  requires: string[];    // params this variant requires
}

export interface ComponentSpec {
  type: string;          // "Chiller"
  library: string;       // "ac"
  summary: string;
  tags: string[];
  ports: string[];       // ["ref_in","ref_out","cool_in","cool_out"]
  params: ComponentParam[]; // in Usage (canonical) order
  variants: ComponentVariant[]; // model$ variants, [] if none
}

export const COMPONENT_CATALOG: ComponentSpec[] = [
  {
    type: `AirCoil`,
    library: `ac`,
    summary: `An air-to-refrigerant coil (the air side of an evaporator or condenser).`,
    tags: [`aircoil`, `component`, `ac`, `acausal`],
    ports: [`ref_in`, `ref_out`, `air_in`, `air_out`],
    params: [
      { name: `ref$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Refrigerant name (e.g. R134a, R1234yf).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U_tp`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Two-phase-zone overall coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U_sh`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Superheat-zone overall coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_zone`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Zone-collapse smoothing width.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_air`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Air-side effectiveness.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Chiller`,
    library: `ac`,
    summary: `A refrigerant-to-coolant chiller transferring heat between the two loops.`,
    tags: [`chiller`, `component`, `ac`, `acausal`],
    ports: [`ref_in`, `ref_out`, `cool_in`, `cool_out`],
    params: [
      { name: `ref$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Refrigerant name (e.g. R134a, R1234yf).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cool$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Coolant name (e.g. EG50, Water).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U_tp`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Two-phase-zone overall coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U_sh`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Superheat-zone overall coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_zone`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Zone-collapse smoothing width.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA_cool`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Coolant-side conductance [W/K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `EXV`,
    library: `ac`,
    summary: `An electronic expansion valve with a commanded opening.`,
    tags: [`exv`, `component`, `ac`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `CdA_max`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Maximum Cd·A [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `u`, isString: false, isSelector: false, isMap: false, unit: `J/kg`, description: `Specific internal energy [J/kg].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `EXVCmd`,
    library: `ac`,
    summary: `Acausal ac-domain component EXVCmd with ports in, out, u.`,
    tags: [`exvcmd`, `component`, `ac`, `acausal`],
    ports: [`in`, `out`, `u`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `CdA_max`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `HeaterCore`,
    library: `ac`,
    summary: `Acausal ac-domain component HeaterCore with ports cool_in, cool_out, air_in, air_out.`,
    tags: [`heatercore`, `component`, `ac`, `acausal`],
    ports: [`cool_in`, `cool_out`, `air_in`, `air_out`],
    params: [
      { name: `cool$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA_cool`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_air`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Radiator`,
    library: `ac`,
    summary: `Acausal ac-domain component Radiator with ports cool_in, cool_out, air_in, air_out.`,
    tags: [`radiator`, `component`, `ac`, `acausal`],
    ports: [`cool_in`, `cool_out`, `air_in`, `air_out`],
    params: [
      { name: `cool$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA_cool`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_air`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TXV`,
    library: `ac`,
    summary: `A thermostatic expansion valve that meters refrigerant to hold a target superheat.`,
    tags: [`txv`, `component`, `ac`, `acausal`],
    ports: [`in`, `out`, `bulb`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Kv`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Flow coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `SH_set`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Target superheat [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `CdA0`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Reference Cd·A [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `tau_valve`, isString: false, isSelector: false, isMap: false, unit: `s`, description: `Valve time constant [s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `tau_bulb`, isString: false, isSelector: false, isMap: false, unit: `s`, description: `Bulb time constant [s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `PIThermostat`,
    library: `control`,
    summary: `A proportional–integral thermostat controller driving an actuator to a setpoint.`,
    tags: [`pithermostat`, `component`, `control`, `acausal`],
    ports: [`port`],
    params: [
      { name: `Kp`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Proportional gain.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Ki`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Integral gain.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tref`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Reference (setpoint) temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Battery`,
    library: `electrical`,
    summary: `An electrical battery modeled as an EMF in series with an internal resistance.`,
    tags: [`battery`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `Voc`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Open-circuit voltage [V].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R0`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Series (ohmic) resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Battery2RC`,
    library: `electrical`,
    summary: `A battery with two RC branches for second-order transient terminal behavior.`,
    tags: [`battery2rc`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `Voc`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Open-circuit voltage [V].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R0`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Series (ohmic) resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R1`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `First RC-branch resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C1`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `First RC-branch capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R2`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Second RC-branch resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C2`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Second RC-branch capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vrc1_0`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Initial first-RC voltage [V].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vrc2_0`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Initial second-RC voltage [V].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `BatteryCellMap`,
    library: `electrical`,
    summary: `Acausal electrical-domain component BatteryCellMap with ports p, n, heat.`,
    tags: [`batterycellmap`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `heat`],
    params: [
      { name: `ocv$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dudt$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R0ref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Ea`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Q0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C_th`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `SOC0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `k_age`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`aging`, `aging_cal`], defaultValue: `` },
      { name: `R1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`rc_map`], defaultValue: `` },
      { name: `C1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`rc_map`], defaultValue: `` },
      { name: `k_cal`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`aging_cal`], defaultValue: `` },
      { name: `Ea_cal`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`aging_cal`], defaultValue: `` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`static`, `aging`, `rc_map`, `aging_cal`], variants: [], defaultValue: `static` }
    ],
    variants: [{ name: `static`, requires: [] }, { name: `aging`, requires: [`k_age`] }, { name: `rc_map`, requires: [`R1`, `C1`] }, { name: `aging_cal`, requires: [`k_age`, `k_cal`, `Ea_cal`] }],
  },
  {
    type: `BatteryPack`,
    library: `electrical`,
    summary: `Acausal electrical-domain component BatteryPack with ports p, n, heat.`,
    tags: [`batterypack`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `heat`],
    params: [
      { name: `Ns`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Np`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `ocv$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dudt$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R0ref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Ea`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Q0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C_th`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `SOC0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `BatteryRC`,
    library: `electrical`,
    summary: `A battery with one RC branch for first-order transient terminal behavior.`,
    tags: [`batteryrc`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `Voc`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Open-circuit voltage [V].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R0`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Series (ohmic) resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R1`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `First RC-branch resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C1`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `First RC-branch capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vrc0`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Initial RC-branch voltage [V].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `BatteryThermal`,
    library: `electrical`,
    summary: `A battery with a coupled thermal model relating losses to temperature.`,
    tags: [`batterythermal`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `heat`],
    params: [
      { name: `Voc`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Open-circuit voltage [V].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R0`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Series (ohmic) resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `BatteryTransient`,
    library: `electrical`,
    summary: `A transient battery model carrying state-of-charge dynamics.`,
    tags: [`batterytransient`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `heat`],
    params: [
      { name: `Voc`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Open-circuit voltage [V].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R0`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Series (ohmic) resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Q0`, isString: false, isSelector: false, isMap: false, unit: `W`, description: `Reference heat [W].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C_th`, isString: false, isSelector: false, isMap: false, unit: `J/K`, description: `Thermal capacitance [J/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `SOC0`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Initial state of charge (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Reference/initial temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Capacitor`,
    library: `electrical`,
    summary: `A capacitor storing charge, with i = C dV/dt.`,
    tags: [`capacitor`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V0`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Initial voltage / volume.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ChargerCCCV`,
    library: `electrical`,
    summary: `Acausal electrical-domain component ChargerCCCV with ports p, n.`,
    tags: [`chargercccv`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `Imax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vmax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsV`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `CurrentSource`,
    library: `electrical`,
    summary: `An ideal current source.`,
    tags: [`currentsource`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `I`, isString: false, isSelector: false, isMap: false, unit: `A`, description: `Current [A].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `DCDCConverter`,
    library: `electrical`,
    summary: `Acausal electrical-domain component DCDCConverter with ports in_p, in_n, out_p, out_n.`,
    tags: [`dcdcconverter`, `component`, `electrical`, `acausal`],
    ports: [`in_p`, `in_n`, `out_p`, `out_n`],
    params: [
      { name: `ratio`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `DCMotor`,
    library: `electrical`,
    summary: `A DC motor — an electrical-to-mechanical transducer (back-EMF and torque constants).`,
    tags: [`dcmotor`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `shaft`],
    params: [
      { name: `Kt`, isString: false, isSelector: false, isMap: false, unit: `N-m/A`, description: `Torque constant [N·m/A].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Ke`, isString: false, isSelector: false, isMap: false, unit: `V-s/rad`, description: `Back-EMF constant [V·s/rad].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Diode`,
    library: `electrical`,
    summary: `A nonlinear diode with an exponential current–voltage characteristic.`,
    tags: [`diode`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `Gon`, isString: false, isSelector: false, isMap: false, unit: `S`, description: `On-state conductance [S].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Effectiveness / roughness.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Electrolyzer`,
    library: `electrical`,
    summary: `Acausal electrical-domain component Electrolyzer with ports p, n, heat.`,
    tags: [`electrolyzer`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `heat`],
    params: [
      { name: `ncells`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `i0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Rohm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `E0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `alpha`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Eth`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ElectrolyzerThermal`,
    library: `electrical`,
    summary: `Acausal electrical-domain component ElectrolyzerThermal with ports p, n, cool_in, cool_out.`,
    tags: [`electrolyzerthermal`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `cool_in`, `cool_out`],
    params: [
      { name: `ncells`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `i0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Rohm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `E0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `alpha`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Eth`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `FuelCellStack`,
    library: `electrical`,
    summary: `A PEM fuel-cell stack producing voltage from its polarization curve.`,
    tags: [`fuelcellstack`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `heat`],
    params: [
      { name: `ncells`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Number of cells.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Area [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `i0`, isString: false, isSelector: false, isMap: false, unit: `A`, description: `Initial current [A].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `ilim`, isString: false, isSelector: false, isMap: false, unit: `A`, description: `Current limit [A].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Rohm`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Ohmic resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `E0`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `Reference EMF [V].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `alpha`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Void fraction / coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Eth`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Activation/threshold energy.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `FuelCellStackCooled`,
    library: `electrical`,
    summary: `Acausal electrical-domain component FuelCellStackCooled with ports p, n, cool_in, cool_out.`,
    tags: [`fuelcellstackcooled`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `cool_in`, `cool_out`],
    params: [
      { name: `ncells`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `i0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `ilim`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Rohm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `E0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `alpha`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Eth`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Ground`,
    library: `electrical`,
    summary: `The electrical reference node (V = 0).`,
    tags: [`ground`, `component`, `electrical`, `acausal`],
    ports: [`port`],
    params: [

    ],
    variants: [],
  },
  {
    type: `HarnessResistance`,
    library: `electrical`,
    summary: `Acausal electrical-domain component HarnessResistance with ports a, b, heat.`,
    tags: [`harnessresistance`, `component`, `electrical`, `acausal`],
    ports: [`a`, `b`, `heat`],
    params: [
      { name: `R20`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `alphaT`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `HeatingResistor`,
    library: `electrical`,
    summary: `A resistor that dissipates its electrical power as heat (electrical→thermal transducer).`,
    tags: [`heatingresistor`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `heat`],
    params: [
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Inductor`,
    library: `electrical`,
    summary: `An inductor storing magnetic energy, with V = L di/dt.`,
    tags: [`inductor`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `I0`, isString: false, isSelector: false, isMap: false, unit: `A`, description: `Saturation current [A].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `InverterLoss`,
    library: `electrical`,
    summary: `Acausal electrical-domain component InverterLoss with ports in_p, out_p, heat.`,
    tags: [`inverterloss`, `component`, `electrical`, `acausal`],
    ports: [`in_p`, `out_p`, `heat`],
    params: [
      { name: `V0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `r`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Esw`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `fsw`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Iref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vnom`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsI`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `MotorMap`,
    library: `electrical`,
    summary: `Acausal electrical-domain component MotorMap with ports p, n, shaft, heat, u.`,
    tags: [`motormap`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `shaft`, `heat`, `u`],
    params: [
      { name: `eff$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `MPPTBlock`,
    library: `electrical`,
    summary: `Acausal electrical-domain component MPPTBlock with ports G, out.`,
    tags: [`mpptblock`, `component`, `electrical`, `acausal`],
    ports: [`G`, `out`],
    params: [
      { name: `vmp$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `PMSM`,
    library: `electrical`,
    summary: `A permanent-magnet synchronous motor.`,
    tags: [`pmsm`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `shaft`],
    params: [
      { name: `Rs`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Series resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `lambda_pm`, isString: false, isSelector: false, isMap: false, unit: `Wb`, description: `PM flux linkage [Wb].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `poles`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Number of magnetic pole pairs.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `PVSingleDiode`,
    library: `electrical`,
    summary: `Acausal electrical-domain component PVSingleDiode with ports p, n, G.`,
    tags: [`pvsinglediode`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `G`],
    params: [
      { name: `Isc_ref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Gref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `I0d`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `n_d`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vt`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Rs`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Rsh`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Resistor`,
    library: `electrical`,
    summary: `An Ohmic resistor, V = R·I.`,
    tags: [`resistor`, `component`, `electrical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SolarArray`,
    library: `electrical`,
    summary: `Acausal electrical-domain component SolarArray with ports p, n, G.`,
    tags: [`solararray`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`, `G`],
    params: [
      { name: `Isc_ref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Gref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Voc`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsV`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Supercapacitor`,
    library: `electrical`,
    summary: `Acausal electrical-domain component Supercapacitor with ports p, n.`,
    tags: [`supercapacitor`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R_esr`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ThermalFuse`,
    library: `electrical`,
    summary: `Acausal electrical-domain component ThermalFuse with ports p, n.`,
    tags: [`thermalfuse`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `R0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Iblow`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `kR`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsI`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `VoltageSource`,
    library: `electrical`,
    summary: `An ideal voltage source.`,
    tags: [`voltagesource`, `component`, `electrical`, `acausal`],
    ports: [`p`, `n`],
    params: [
      { name: `E`, isString: false, isSelector: false, isMap: false, unit: `V`, description: `EMF / voltage [V].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Accumulator`,
    library: `fluid`,
    summary: `A fluid accumulator — a compliance volume that stores fluid under pressure and buffers flow transients.`,
    tags: [`accumulator`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Reference/initial pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `AtmosphereSource`,
    library: `fluid`,
    summary: `Acausal fluid-domain component AtmosphereSource with ports out.`,
    tags: [`atmospheresource`, `component`, `fluid`, `acausal`],
    ports: [`out`],
    params: [
      { name: `alt`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Boiler`,
    library: `fluid`,
    summary: `Adds heat to a fluid stream, raising its enthalpy (and generating vapor at saturation).`,
    tags: [`boiler`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `Combustor`,
    library: `fluid`,
    summary: `Acausal fluid-domain component Combustor with ports in, out.`,
    tags: [`combustor`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `mdot_f`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `LHV`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `CombustorSpecies`,
    library: `fluid`,
    summary: `Acausal fluid-domain component CombustorSpecies with ports in, out.`,
    tags: [`combustorspecies`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `mdot_f`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `LHV`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `xC`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `yH`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `Compressor`,
    library: `fluid`,
    summary: `Raises the pressure of a fluid stream, computing the work from an isentropic efficiency.`,
    tags: [`compressor`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Efficiency (0–1).`, required: true, values: [], variants: [`isentropic`, `volumetric`], defaultValue: `` },
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_v`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`volumetric`], defaultValue: `` },
      { name: `disp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`volumetric`], defaultValue: `` },
      { name: `rpm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`volumetric`, `map`], defaultValue: `` },
      { name: `map_eta$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [`map`], defaultValue: `` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: `Model variant — selects the physics body (see Model Variants).`, required: false, values: [`isentropic`, `volumetric`, `map`], variants: [], defaultValue: `isentropic` }
    ],
    variants: [{ name: `isentropic`, requires: [`eta`] }, { name: `volumetric`, requires: [`eta`, `eta_v`, `disp`, `rpm`] }, { name: `map`, requires: [`map_eta$`, `rpm`] }],
  },
  {
    type: `CompressorMap`,
    library: `fluid`,
    summary: `A compressor whose isentropic efficiency comes from a tabulated map (eta vs pressure ratio).`,
    tags: [`compressormap`, `compressor`, `map`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `map_eta$`, isString: true, isSelector: false, isMap: true, unit: ``, description: `Name of a TABLE/FUNCTION giving isentropic efficiency (0–1) vs pressure ratio (out.P/in.P).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `map_mdot$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [`flow`], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `fluid` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`eta`, `flow`], variants: [], defaultValue: `eta` }
    ],
    variants: [{ name: `eta`, requires: [] }, { name: `flow`, requires: [`map_mdot$`] }],
  },
  {
    type: `Condenser`,
    library: `fluid`,
    summary: `Rejects heat from a fluid stream to a coolant/ambient, condensing it.`,
    tags: [`condenser`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `Duct`,
    library: `fluid`,
    summary: `A flow passage that imposes a pressure drop on the stream.`,
    tags: [`duct`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mu`, isString: false, isSelector: false, isMap: false, unit: `Pa-s`, description: `Dynamic viscosity [Pa·s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rough`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Relative wall roughness.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ExpansionValve`,
    library: `fluid`,
    summary: `Throttles a fluid to a lower pressure isenthalpically (Joule–Thomson).`,
    tags: [`expansionvalve`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Discharge coefficient × area Cd·A [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho_in`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Inlet density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Fan`,
    library: `fluid`,
    summary: `Adds a pressure rise to a gas/air stream, computing the fan work.`,
    tags: [`fan`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP0`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Reference pressure drop [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Q0`, isString: false, isSelector: false, isMap: false, unit: `W`, description: `Reference heat [W].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `FanCurve`,
    library: `fluid`,
    summary: `A fan whose pressure rise follows a tabulated pressure–flow performance curve.`,
    tags: [`fancurve`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP0`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Reference pressure drop [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Q0`, isString: false, isSelector: false, isMap: false, unit: `W`, description: `Reference heat [W].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `FanMap`,
    library: `fluid`,
    summary: `A fan whose pressure rise comes from a tabulated performance map (ΔP vs volumetric flow).`,
    tags: [`fanmap`, `fan`, `map`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `map$`, isString: true, isSelector: false, isMap: true, unit: ``, description: `Name of a TABLE/FUNCTION giving pressure rise [Pa] vs volumetric flow [m³/s].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `FlowSensor`,
    library: `fluid`,
    summary: `Measures the mass flow of a stream (a pass-through sensor).`,
    tags: [`flowsensor`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `HeatedDuct`,
    library: `fluid`,
    summary: `Acausal fluid-domain component HeatedDuct with ports in, out, wall.`,
    tags: [`heatedduct`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `HeatExchanger`,
    library: `fluid`,
    summary: `Transfers heat between two fluid streams across a wall.`,
    tags: [`heatexchanger`, `component`, `fluid`, `acausal`],
    ports: [`hot_in`, `hot_out`, `cold_in`, `cold_out`],
    params: [
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Overall conductance UA [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `hot$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Hot-side fluid name (e.g. Water).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cold$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Cold-side fluid name (e.g. EG50).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `arr$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Flow arrangement (passed to hx_effectiveness) — one of \`counterflow\`, \`parallel\`.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Mixer`,
    library: `fluid`,
    summary: `Combines two fluid streams into one, with flow-weighted enthalpy mixing.`,
    tags: [`mixer`, `component`, `fluid`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `Nozzle`,
    library: `fluid`,
    summary: `Accelerates a flow, converting enthalpy into kinetic energy.`,
    tags: [`nozzle`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Stiffness / conductivity.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A_throat`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Throat area [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A_exit`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Exit area [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P_amb`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Ambient pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Reference/initial temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Pipe`,
    library: `fluid`,
    summary: `A flow passage that imposes a frictional pressure drop.`,
    tags: [`pipe`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rough`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Relative wall roughness.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Propeller`,
    library: `fluid`,
    summary: `Acausal fluid-domain component Propeller with ports shaft, veh.`,
    tags: [`propeller`, `component`, `fluid`, `acausal`],
    ports: [`shaft`, `veh`],
    params: [
      { name: `Dp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rhoA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `ct$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cpw$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsn`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Pump`,
    library: `fluid`,
    summary: `Raises the pressure of a liquid stream, computing the work from a pump efficiency.`,
    tags: [`pump`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `PumpMap`,
    library: `fluid`,
    summary: `A pump whose head comes from a tabulated performance map (head vs volumetric flow).`,
    tags: [`pumpmap`, `pump`, `map`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `map$`, isString: true, isSelector: false, isMap: true, unit: ``, description: `Name of a TABLE/FUNCTION giving head [m] vs volumetric flow [m³/s].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Regenerator`,
    library: `fluid`,
    summary: `Acausal fluid-domain component Regenerator with ports hot_in, hot_out, cold_in, cold_out.`,
    tags: [`regenerator`, `component`, `fluid`, `acausal`],
    ports: [`hot_in`, `hot_out`, `cold_in`, `cold_out`],
    params: [
      { name: `hot$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cold$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Sink`,
    library: `fluid`,
    summary: `A fluid boundary that absorbs a stream at a set pressure.`,
    tags: [`sink`, `component`, `fluid`, `acausal`],
    ports: [`in`],
    params: [

    ],
    variants: [],
  },
  {
    type: `Source`,
    library: `fluid`,
    summary: `A fluid boundary that supplies a stream at set conditions.`,
    tags: [`source`, `component`, `fluid`, `acausal`],
    ports: [`out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Mass flow rate [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Splitter`,
    library: `fluid`,
    summary: `Divides a fluid stream into two branches.`,
    tags: [`splitter`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out1`, `out2`],
    params: [

    ],
    variants: [],
  },
  {
    type: `Throttle`,
    library: `fluid`,
    summary: `An isenthalpic pressure-reducing restriction.`,
    tags: [`throttle`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `Turbine`,
    library: `fluid`,
    summary: `Extracts work from an expanding fluid stream, computing it from an isentropic efficiency.`,
    tags: [`turbine`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Turbocharger`,
    library: `fluid`,
    summary: `A turbine-driven compressor pair coupled on a common shaft.`,
    tags: [`turbocharger`, `component`, `fluid`, `acausal`],
    ports: [`t_in`, `t_out`, `c_in`, `c_out`],
    params: [
      { name: `cp`, isString: false, isSelector: false, isMap: false, unit: `J/kg-K`, description: `Specific heat [J/kg·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_t`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Turbine efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_c`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Compressor efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `gam`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Ratio of specific heats.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TwoZoneHX`,
    library: `fluid`,
    summary: `A two-zone heat exchanger resolving distinct thermal regions.`,
    tags: [`twozonehx`, `component`, `fluid`, `acausal`],
    ports: [`hot_in`, `hot_out`, `cold_in`, `cold_out`],
    params: [
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Overall conductance UA [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `hot$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Hot-side fluid name (e.g. Water).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cold$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Cold-side fluid name (e.g. EG50).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `arr$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Flow arrangement (passed to hx_effectiveness) — one of \`counterflow\`, \`parallel\`.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Valve`,
    library: `fluid`,
    summary: `A flow restriction characterized by a flow/pressure-drop coefficient.`,
    tags: [`valve`, `component`, `fluid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `Cv`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Flow coefficient.`, required: true, values: [], variants: [`fixed`, `linear`, `eqpct`], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `u`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`linear`, `eqpct`], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`eqpct`], defaultValue: `` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`fixed`, `linear`, `eqpct`], variants: [], defaultValue: `fixed` }
    ],
    variants: [{ name: `fixed`, requires: [`Cv`] }, { name: `linear`, requires: [`Cv`, `u`] }, { name: `eqpct`, requires: [`Cv`, `u`, `R`] }],
  },
  {
    type: `CellToPackThermal`,
    library: `heat`,
    summary: `Acausal heat-domain component CellToPackThermal with ports cell, plate.`,
    tags: [`celltopackthermal`, `component`, `heat`, `acausal`],
    ports: [`cell`, `plate`],
    params: [
      { name: `Rcc`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Cpl`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Conduction`,
    library: `heat`,
    summary: `A conductive thermal resistance (Fourier), Q̇ = (T1 − T2)/R.`,
    tags: [`conduction`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Stiffness / conductivity.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Area [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ContactResistance`,
    library: `heat`,
    summary: `A thermal contact resistance between two surfaces.`,
    tags: [`contactresistance`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `Rth`, isString: false, isSelector: false, isMap: false, unit: `K/W`, description: `Thermal resistance [K/W].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Convection`,
    library: `heat`,
    summary: `A convective link (Newton’s law of cooling), Q̇ = h·A·ΔT.`,
    tags: [`convection`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `htc`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Heat-transfer coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Area [m²].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `HeatPipe`,
    library: `heat`,
    summary: `Acausal heat-domain component HeatPipe with ports a, b.`,
    tags: [`heatpipe`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `G`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Qmax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `HeatSource`,
    library: `heat`,
    summary: `A prescribed heat input to a thermal node.`,
    tags: [`heatsource`, `component`, `heat`, `acausal`],
    ports: [`port`],
    params: [
      { name: `Q`, isString: false, isSelector: false, isMap: false, unit: `W`, description: `Heat input [W].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `MassGen`,
    library: `heat`,
    summary: `A mass/heat generation source term.`,
    tags: [`massgen`, `component`, `heat`, `acausal`],
    ports: [`port`],
    params: [
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Qgen`, isString: false, isSelector: false, isMap: false, unit: `W`, description: `Generated heat [W].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Reference/initial temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `MultiZoneWall`,
    library: `heat`,
    summary: `Acausal heat-domain component MultiZoneWall with ports a, b.`,
    tags: [`multizonewall`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `h_a`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h_b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C2`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T10`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T20`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `PCMMass`,
    library: `heat`,
    summary: `Acausal heat-domain component PCMMass with ports port.`,
    tags: [`pcmmass`, `component`, `heat`, `acausal`],
    ports: [`port`],
    params: [
      { name: `m`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dTm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `PeltierTEC`,
    library: `heat`,
    summary: `Acausal heat-domain component PeltierTEC with ports p, n, hot, cold.`,
    tags: [`peltiertec`, `component`, `heat`, `acausal`],
    ports: [`p`, `n`, `hot`, `cold`],
    params: [
      { name: `Sab`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Rel`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Kth`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Radiation`,
    library: `heat`,
    summary: `A radiative exchange link (Stefan–Boltzmann), Q̇ = εσA(T1⁴ − T2⁴).`,
    tags: [`radiation`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `emis`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Emissivity (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Area [m²].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `RadiationTwoSurface`,
    library: `heat`,
    summary: `Acausal heat-domain component RadiationTwoSurface with ports a, b.`,
    tags: [`radiationtwosurface`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `e1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `e2`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A2`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `F12`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ThermalMass`,
    library: `heat`,
    summary: `A lumped thermal capacitance, C dT/dt = Q̇.`,
    tags: [`thermalmass`, `component`, `heat`, `acausal`],
    ports: [`port`],
    params: [
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Reference/initial temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ThermalSensor`,
    library: `heat`,
    summary: `A temperature sensor (pass-through).`,
    tags: [`thermalsensor`, `component`, `heat`, `acausal`],
    ports: [`port`],
    params: [

    ],
    variants: [],
  },
  {
    type: `ThermalSource`,
    library: `heat`,
    summary: `A prescribed-temperature boundary.`,
    tags: [`thermalsource`, `component`, `heat`, `acausal`],
    ports: [`port`],
    params: [
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ThermalSwitch`,
    library: `heat`,
    summary: `Acausal heat-domain component ThermalSwitch with ports a, b.`,
    tags: [`thermalswitch`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `G`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Ton`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `band`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `WallRC`,
    library: `heat`,
    summary: `Acausal heat-domain component WallRC with ports a, b.`,
    tags: [`wallrc`, `component`, `heat`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `C1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C2`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T10`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T20`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `CounterbalanceValve`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component CounterbalanceValve with ports in, out, pilot.`,
    tags: [`counterbalancevalve`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`, `pilot`],
    params: [
      { name: `CdA_max`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P_set`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R_p`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_o`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicAccumulator`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicAccumulator with ports port.`,
    tags: [`hydraulicaccumulator`, `component`, `hydraulic`, `acausal`],
    ports: [`port`],
    params: [
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `gamma`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vg0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicCheckValve`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicCheckValve with ports in, out.`,
    tags: [`hydrauliccheckvalve`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicCylinder`,
    library: `hydraulic`,
    summary: `A hydraulic actuator converting flow/pressure to motion/force.`,
    tags: [`hydrauliccylinder`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `rod`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `beta`, isString: false, isSelector: false, isMap: false, unit: `deg`, description: `Chevron angle [deg] / coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V0`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Initial voltage / volume.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Area [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Patm`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Atmospheric pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Reference/initial pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicDoubleActingCylinder`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicDoubleActingCylinder with ports a, b, rod.`,
    tags: [`hydraulicdoubleactingcylinder`, `component`, `hydraulic`, `acausal`],
    ports: [`a`, `b`, `rod`],
    params: [
      { name: `Aa`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Ab`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `beta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Va0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vb0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Pa0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Pb0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicFlowControl`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicFlowControl with ports in, out.`,
    tags: [`hydraulicflowcontrol`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `Qset`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicFlowDivider`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicFlowDivider with ports in, outa, outb.`,
    tags: [`hydraulicflowdivider`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `outa`, `outb`],
    params: [
      { name: `frac`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicMotor`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicMotor with ports in, out, shaft.`,
    tags: [`hydraulicmotor`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`, `shaft`],
    params: [
      { name: `disp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_v`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_m`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicOrifice`,
    library: `hydraulic`,
    summary: `A hydraulic orifice metering flow by ṁ ∝ √Δp.`,
    tags: [`hydraulicorifice`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Discharge coefficient × area Cd·A [m²].`, required: true, values: [], variants: [`turbulent`], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cq_lambda`], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cq_lambda`], defaultValue: `` },
      { name: `nu`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cq_lambda`], defaultValue: `` },
      { name: `Cqmax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cq_lambda`], defaultValue: `` },
      { name: `lamc`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cq_lambda`], defaultValue: `` },
      { name: `epsP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cq_lambda`], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `oil` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`turbulent`, `cq_lambda`], variants: [], defaultValue: `turbulent` }
    ],
    variants: [{ name: `turbulent`, requires: [`CdA`] }, { name: `cq_lambda`, requires: [`A`, `D`, `nu`, `Cqmax`, `lamc`, `epsP`] }],
  },
  {
    type: `HydraulicPilotCheckValve`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicPilotCheckValve with ports in, out, pilot.`,
    tags: [`hydraulicpilotcheckvalve`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`, `pilot`],
    params: [
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicPipe`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicPipe with ports in, out.`,
    tags: [`hydraulicpipe`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `nu`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rough`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicPump`,
    library: `hydraulic`,
    summary: `A hydraulic pump delivering flow against pressure.`,
    tags: [`hydraulicpump`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`, `shaft`],
    params: [
      { name: `disp`, isString: false, isSelector: false, isMap: false, unit: `m^3`, description: `Displacement volume [m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_v`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Volumetric efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_m`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Mechanical efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicResistance`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicResistance with ports in, out.`,
    tags: [`hydraulicresistance`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `K`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicSequenceValve`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicSequenceValve with ports in, out.`,
    tags: [`hydraulicsequencevalve`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `Pset`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicSupply`,
    library: `hydraulic`,
    summary: `A hydraulic pressure supply.`,
    tags: [`hydraulicsupply`, `component`, `hydraulic`, `acausal`],
    ports: [`out`],
    params: [
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicTank`,
    library: `hydraulic`,
    summary: `A hydraulic reservoir at (near) atmospheric pressure.`,
    tags: [`hydraulictank`, `component`, `hydraulic`, `acausal`],
    ports: [`port`],
    params: [
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicThermalVolume`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicThermalVolume with ports in, out, wall.`,
    tags: [`hydraulicthermalvolume`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cp_o`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `beta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `hA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Pvap`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cav`], defaultValue: `` },
      { name: `eps_c`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cav`], defaultValue: `` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`stiff`, `cav`], variants: [], defaultValue: `stiff` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [{ name: `stiff`, requires: [] }, { name: `cav`, requires: [`Pvap`, `eps_c`] }],
  },
  {
    type: `HydraulicValve`,
    library: `hydraulic`,
    summary: `A hydraulic valve metering flow vs. pressure drop.`,
    tags: [`hydraulicvalve`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `CdA_max`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Maximum Cd·A [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `u`, isString: false, isSelector: false, isMap: false, unit: `J/kg`, description: `Specific internal energy [J/kg].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicValveCmd`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicValveCmd with ports in, out, u.`,
    tags: [`hydraulicvalvecmd`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`, `u`],
    params: [
      { name: `CdA_max`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `HydraulicVolume`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component HydraulicVolume with ports in, out.`,
    tags: [`hydraulicvolume`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `beta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `LoadSensingPump`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component LoadSensingPump with ports in, out, ls.`,
    tags: [`loadsensingpump`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`, `ls`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Dv`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `w_p`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP_margin`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `tau`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `d0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `ReliefValve`,
    library: `hydraulic`,
    summary: `A pressure-relief valve that opens above its set pressure.`,
    tags: [`reliefvalve`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `Pcrack`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Cracking (relief) pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `K`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Gain / coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Effectiveness / roughness.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `ServoValveDynamic`,
    library: `hydraulic`,
    summary: `Acausal hydraulic-domain component ServoValveDynamic with ports in, out, u.`,
    tags: [`servovalvedynamic`, `component`, `hydraulic`, `acausal`],
    ports: [`in`, `out`, `u`],
    params: [
      { name: `CdA_max`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `wn`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `zeta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `xs0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `oil` }
    ],
    variants: [],
  },
  {
    type: `CoolingTower`,
    library: `liquid`,
    summary: `Acausal liquid-domain component CoolingTower with ports in, out, wb.`,
    tags: [`coolingtower`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`, `wb`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_t`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mdot_a`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Patm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `GravityDrain`,
    library: `liquid`,
    summary: `Acausal liquid-domain component GravityDrain with ports in, out.`,
    tags: [`gravitydrain`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `Cd`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A_d`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `HydroTurbine`,
    library: `liquid`,
    summary: `Acausal liquid-domain component HydroTurbine with ports in, out, shaft.`,
    tags: [`hydroturbine`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`, `shaft`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsw`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `IceStorageBrine`,
    library: `liquid`,
    summary: `Acausal liquid-domain component IceStorageBrine with ports in, out.`,
    tags: [`icestoragebrine`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `m`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cp_p`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dTm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `LiquidCheckValve`,
    library: `liquid`,
    summary: `Acausal liquid-domain component LiquidCheckValve with ports in, out.`,
    tags: [`liquidcheckvalve`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidColdPlate`,
    library: `liquid`,
    summary: `A liquid cold plate cooling an electronics/heat load.`,
    tags: [`liquidcoldplate`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `Q`, isString: false, isSelector: false, isMap: false, unit: `W`, description: `Heat input [W].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidExpansionTank`,
    library: `liquid`,
    summary: `Acausal liquid-domain component LiquidExpansionTank with ports port.`,
    tags: [`liquidexpansiontank`, `component`, `liquid`, `acausal`],
    ports: [`port`],
    params: [
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidMixer`,
    library: `liquid`,
    summary: `Mixes two single-phase liquid streams.`,
    tags: [`liquidmixer`, `component`, `liquid`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidOrifice`,
    library: `liquid`,
    summary: `A liquid orifice metering flow vs. pressure drop.`,
    tags: [`liquidorifice`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Discharge coefficient × area Cd·A [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: `Model variant — selects the physics body (see Model Variants).`, required: false, values: [`incompressible`, `cavitating`], variants: [], defaultValue: `incompressible` },
      { name: `Pvap`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cavitating`], defaultValue: `` }
    ],
    variants: [{ name: `incompressible`, requires: [] }, { name: `cavitating`, requires: [`Pvap`] }],
  },
  {
    type: `LiquidPipe`,
    library: `liquid`,
    summary: `A single-phase liquid pipe with frictional pressure drop.`,
    tags: [`liquidpipe`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rough`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Relative wall roughness.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidPump`,
    library: `liquid`,
    summary: `A single-phase liquid pump.`,
    tags: [`liquidpump`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Pvap`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`npsh`], defaultValue: `` },
      { name: `NPSHr`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`npsh`], defaultValue: `` },
      { name: `eps_n`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`npsh`], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`basic`, `npsh`], variants: [], defaultValue: `basic` }
    ],
    variants: [{ name: `basic`, requires: [] }, { name: `npsh`, requires: [`Pvap`, `NPSHr`, `eps_n`] }],
  },
  {
    type: `LiquidPumpMap`,
    library: `liquid`,
    summary: `Acausal liquid-domain component LiquidPumpMap with ports in, out.`,
    tags: [`liquidpumpmap`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `map$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidSink`,
    library: `liquid`,
    summary: `A liquid boundary absorbing a stream.`,
    tags: [`liquidsink`, `component`, `liquid`, `acausal`],
    ports: [`in`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidSource`,
    library: `liquid`,
    summary: `A liquid boundary supplying a stream of set state.`,
    tags: [`liquidsource`, `component`, `liquid`, `acausal`],
    ports: [`out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Mass flow rate [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidTank`,
    library: `liquid`,
    summary: `Acausal liquid-domain component LiquidTank with ports in, out, wall.`,
    tags: [`liquidtank`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `m`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidThermostat`,
    library: `liquid`,
    summary: `Acausal liquid-domain component LiquidThermostat with ports in, out.`,
    tags: [`liquidthermostat`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Topen`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tband`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidThreeWayValve`,
    library: `liquid`,
    summary: `Acausal liquid-domain component LiquidThreeWayValve with ports in, outa, outb.`,
    tags: [`liquidthreewayvalve`, `component`, `liquid`, `acausal`],
    ports: [`in`, `outa`, `outb`],
    params: [
      { name: `u`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidVolume`,
    library: `liquid`,
    summary: `A single-phase liquid control volume.`,
    tags: [`liquidvolume`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Reference/initial pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `LiquidWallHX`,
    library: `liquid`,
    summary: `A liquid-to-wall heat exchanger.`,
    tags: [`liquidwallhx`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Overall conductance UA [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `OpenTank`,
    library: `liquid`,
    summary: `Acausal liquid-domain component OpenTank with ports in, out.`,
    tags: [`opentank`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `A_t`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `ThermalStorageTank`,
    library: `liquid`,
    summary: `Acausal liquid-domain component ThermalStorageTank with ports in, out.`,
    tags: [`thermalstoragetank`, `component`, `liquid`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `m_node`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cp_f`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA_loss`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T_amb`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `kmix`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T10`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T20`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T30`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `liquid` }
    ],
    variants: [],
  },
  {
    type: `BeltDrive`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component BeltDrive with ports a, b.`,
    tags: [`beltdrive`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `ratio`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Brake`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component Brake with ports a, b, u.`,
    tags: [`brake`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`, `u`],
    params: [
      { name: `Tmax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Cam`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component Cam with ports shaft, rod.`,
    tags: [`cam`, `component`, `mechanical`, `acausal`],
    ports: [`shaft`, `rod`],
    params: [
      { name: `prof$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `theta0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `CamFollower`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component CamFollower with ports rod.`,
    tags: [`camfollower`, `component`, `mechanical`, `acausal`],
    ports: [`rod`],
    params: [
      { name: `m`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `kspring`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `x0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `v0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Clutch`,
    library: `mechanical`,
    summary: `A friction clutch coupling/decoupling two rotational shafts.`,
    tags: [`clutch`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `Tmax`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Maximum temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eng`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Engagement fraction (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Effectiveness / roughness.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ClutchCmd`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component ClutchCmd with ports a, b, u.`,
    tags: [`clutchcmd`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`, `u`],
    params: [
      { name: `Tmax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `EndStop`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component EndStop with ports port.`,
    tags: [`endstop`, `component`, `mechanical`, `acausal`],
    ports: [`port`],
    params: [
      { name: `gap`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `c`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `x0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ForceSource`,
    library: `mechanical`,
    summary: `A prescribed translational force.`,
    tags: [`forcesource`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `F`, isString: false, isSelector: false, isMap: false, unit: `N`, description: `Force [N].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Freewheel`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component Freewheel with ports a, b.`,
    tags: [`freewheel`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Friction`,
    library: `mechanical`,
    summary: `A friction element opposing motion.`,
    tags: [`friction`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `Fc`, isString: false, isSelector: false, isMap: false, unit: `N`, description: `Coulomb friction force [N].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Fs`, isString: false, isSelector: false, isMap: false, unit: `N`, description: `Static friction force [N].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `vs`, isString: false, isSelector: false, isMap: false, unit: `m/s`, description: `Reference / slip velocity [m/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `bv`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Viscous-friction coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Effectiveness / roughness.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Gear`,
    library: `mechanical`,
    summary: `A gear pair imposing a fixed speed/torque ratio between two shafts.`,
    tags: [`gear`, `component`, `mechanical`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `ratio`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Gear / split ratio.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Inertia`,
    library: `mechanical`,
    summary: `A rotational inertia, τ = J dω/dt.`,
    tags: [`inertia`, `component`, `mechanical`, `acausal`],
    ports: [`port`],
    params: [
      { name: `J`, isString: false, isSelector: false, isMap: false, unit: `kg-m^2`, description: `Inertia [kg·m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `w0`, isString: false, isSelector: false, isMap: false, unit: `rad/s`, description: `Natural frequency [rad/s].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Lever`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component Lever with ports a, b.`,
    tags: [`lever`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `ratio`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `MechGround`,
    library: `mechanical`,
    summary: `The rotational reference (ω = 0).`,
    tags: [`mechground`, `component`, `mechanical`, `acausal`],
    ports: [`port`],
    params: [

    ],
    variants: [],
  },
  {
    type: `Planetary`,
    library: `mechanical`,
    summary: `A planetary gearset relating sun, ring, and carrier speeds.`,
    tags: [`planetary`, `component`, `mechanical`, `acausal`],
    ports: [`sun`, `ring`, `carrier`],
    params: [
      { name: `g`, isString: false, isSelector: false, isMap: false, unit: `m/s^2`, description: `Gravitational acceleration [m/s²].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `RackPinion`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component RackPinion with ports shaft, rod.`,
    tags: [`rackpinion`, `component`, `mechanical`, `acausal`],
    ports: [`shaft`, `rod`],
    params: [
      { name: `r`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `RotationalDamper`,
    library: `mechanical`,
    summary: `A rotational viscous damper, τ = c·ω.`,
    tags: [`rotationaldamper`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `c`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Damping / specific-heat coefficient.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `RotationalSpring`,
    library: `mechanical`,
    summary: `A torsional spring, τ = k·θ.`,
    tags: [`rotationalspring`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Stiffness / conductivity.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `theta0`, isString: false, isSelector: false, isMap: false, unit: `rad`, description: `Initial angle [rad].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ScrewDrive`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component ScrewDrive with ports shaft, rod.`,
    tags: [`screwdrive`, `component`, `mechanical`, `acausal`],
    ports: [`shaft`, `rod`],
    params: [
      { name: `lead`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SpeedSource`,
    library: `mechanical`,
    summary: `A prescribed angular velocity.`,
    tags: [`speedsource`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `w`, isString: false, isSelector: false, isMap: false, unit: `rad/s`, description: `Frequency [rad/s].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TorqueSource`,
    library: `mechanical`,
    summary: `A prescribed torque.`,
    tags: [`torquesource`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Temperature [K].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TorsionalBacklash`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component TorsionalBacklash with ports a, b.`,
    tags: [`torsionalbacklash`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `half`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `theta0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TransDamper`,
    library: `mechanical`,
    summary: `A translational viscous damper, F = c·v.`,
    tags: [`transdamper`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `c`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Damping / specific-heat coefficient.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TransGround`,
    library: `mechanical`,
    summary: `The translational reference (v = 0).`,
    tags: [`transground`, `component`, `mechanical`, `acausal`],
    ports: [`port`],
    params: [

    ],
    variants: [],
  },
  {
    type: `TransMass`,
    library: `mechanical`,
    summary: `A translational mass, F = m dv/dt.`,
    tags: [`transmass`, `component`, `mechanical`, `acausal`],
    ports: [`port`],
    params: [
      { name: `m`, isString: false, isSelector: false, isMap: false, unit: `kg`, description: `Mass [kg].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `v0`, isString: false, isSelector: false, isMap: false, unit: `m/s`, description: `Initial velocity [m/s].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TransSpring`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component TransSpring with ports a, b.`,
    tags: [`transspring`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`],
    params: [
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `x0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `WheelBrakeThermal`,
    library: `mechanical`,
    summary: `Acausal mechanical-domain component WheelBrakeThermal with ports a, b, u.`,
    tags: [`wheelbrakethermal`, `component`, `mechanical`, `acausal`],
    ports: [`a`, `b`, `u`],
    params: [
      { name: `Tmax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `hA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T_amb`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T_fade`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `k_fade`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_f`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `AHU`,
    library: `moistair`,
    summary: `Acausal moistair-domain component AHU with ports ret_in, oa_in, sup_out.`,
    tags: [`ahu`, `component`, `moistair`, `acausal`],
    ports: [`ret_in`, `oa_in`, `sup_out`],
    params: [
      { name: `Kf`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `foul`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tc`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Qh`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dPfan`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_fan`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `AirFilter`,
    library: `moistair`,
    summary: `Acausal moistair-domain component AirFilter with ports in, out.`,
    tags: [`airfilter`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `K`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `foul`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `ApparatusDewPointCoil`,
    library: `moistair`,
    summary: `Acausal moistair-domain component ApparatusDewPointCoil with ports in, out.`,
    tags: [`apparatusdewpointcoil`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `T_adp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `BF`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`wet`, `dry`], variants: [], defaultValue: `wet` }
    ],
    variants: [{ name: `wet`, requires: [] }, { name: `dry`, requires: [] }],
  },
  {
    type: `CabinZone`,
    library: `moistair`,
    summary: `Acausal moistair-domain component CabinZone with ports in, out, wall.`,
    tags: [`cabinzone`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `Vz`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `W0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `n_occ`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `q_sens`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mw_occ`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Q_aux`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `ChilledBeam`,
    library: `moistair`,
    summary: `Acausal moistair-domain component ChilledBeam with ports in, out, wall.`,
    tags: [`chilledbeam`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `CoolingCoil`,
    library: `moistair`,
    summary: `Cools and (below dew point) dehumidifies a humid-air stream.`,
    tags: [`coolingcoil`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `Tout`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Outlet temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `DesiccantWheel`,
    library: `moistair`,
    summary: `Acausal moistair-domain component DesiccantWheel with ports proc_in, proc_out, reg_in, reg_out.`,
    tags: [`desiccantwheel`, `component`, `moistair`, `acausal`],
    ports: [`proc_in`, `proc_out`, `reg_in`, `reg_out`],
    params: [
      { name: `eff_L`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `W_eq`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `f_carry`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `Diffuser`,
    library: `moistair`,
    summary: `Acausal moistair-domain component Diffuser with ports in, out.`,
    tags: [`diffuser`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `A1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A2`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_rec`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `DOAS`,
    library: `moistair`,
    summary: `Acausal moistair-domain component DOAS with ports oa_in, sup_out, exh_in, exh_out.`,
    tags: [`doas`, `component`, `moistair`, `acausal`],
    ports: [`oa_in`, `sup_out`, `exh_in`, `exh_out`],
    params: [
      { name: `eff_h`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eff_w`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T_adp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `BF`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Q_reheat`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Economizer`,
    library: `moistair`,
    summary: `Acausal moistair-domain component Economizer with ports oa_in, ret_in, mix_out.`,
    tags: [`economizer`, `component`, `moistair`, `acausal`],
    ports: [`oa_in`, `ret_in`, `mix_out`],
    params: [
      { name: `mdot_sup`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `f_min`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `lim`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `band`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`drybulb`, `enthalpy`], variants: [], defaultValue: `drybulb` }
    ],
    variants: [{ name: `drybulb`, requires: [] }, { name: `enthalpy`, requires: [] }],
  },
  {
    type: `EnthalpyWheel`,
    library: `moistair`,
    summary: `Acausal moistair-domain component EnthalpyWheel with ports sup_in, sup_out, exh_in, exh_out.`,
    tags: [`enthalpywheel`, `component`, `moistair`, `acausal`],
    ports: [`sup_in`, `sup_out`, `exh_in`, `exh_out`],
    params: [
      { name: `eff_h`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eff_w`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `EvaporativeCooler`,
    library: `moistair`,
    summary: `Acausal moistair-domain component EvaporativeCooler with ports in, out.`,
    tags: [`evaporativecooler`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `eff`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `FaceAndBypassCoil`,
    library: `moistair`,
    summary: `Acausal moistair-domain component FaceAndBypassCoil with ports in, out, wall.`,
    tags: [`faceandbypasscoil`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `u_face`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `FanCoilUnit`,
    library: `moistair`,
    summary: `Acausal moistair-domain component FanCoilUnit with ports in, out, wall.`,
    tags: [`fancoilunit`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `K`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `foul`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `FanPoweredBox`,
    library: `moistair`,
    summary: `Acausal moistair-domain component FanPoweredBox with ports pri_in, ind_in, out.`,
    tags: [`fanpoweredbox`, `component`, `moistair`, `acausal`],
    ports: [`pri_in`, `ind_in`, `out`],
    params: [
      { name: `Q_fan`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Q_reheat`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `HeatingCoil`,
    library: `moistair`,
    summary: `Heats a humid-air stream at constant humidity ratio.`,
    tags: [`heatingcoil`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `Q`, isString: false, isSelector: false, isMap: false, unit: `W`, description: `Heat input [W].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `HeatPipeWrapAround`,
    library: `moistair`,
    summary: `Acausal moistair-domain component HeatPipeWrapAround with ports pre_in, pre_out, re_in, re_out.`,
    tags: [`heatpipewraparound`, `component`, `moistair`, `acausal`],
    ports: [`pre_in`, `pre_out`, `re_in`, `re_out`],
    params: [
      { name: `eff`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `Humidifier`,
    library: `moistair`,
    summary: `Adds moisture to a humid-air stream, raising its humidity ratio.`,
    tags: [`humidifier`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `mdot_w`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Water/coolant mass flow [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h_w`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Wall heat-transfer coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `IndirectDirectEvaporativeCooler`,
    library: `moistair`,
    summary: `Acausal moistair-domain component IndirectDirectEvaporativeCooler with ports pri_in, pri_out, sec_in, sec_out.`,
    tags: [`indirectdirectevaporativecooler`, `component`, `moistair`, `acausal`],
    ports: [`pri_in`, `pri_out`, `sec_in`, `sec_out`],
    params: [
      { name: `wbde`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eff_sec`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eff_dir`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `IndirectEvaporativeCooler`,
    library: `moistair`,
    summary: `Acausal moistair-domain component IndirectEvaporativeCooler with ports pri_in, pri_out, sec_in, sec_out.`,
    tags: [`indirectevaporativecooler`, `component`, `moistair`, `acausal`],
    ports: [`pri_in`, `pri_out`, `sec_in`, `sec_out`],
    params: [
      { name: `wbde`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eff_sec`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `InductionUnit`,
    library: `moistair`,
    summary: `Acausal moistair-domain component InductionUnit with ports pri_in, ind_in, out, wall.`,
    tags: [`inductionunit`, `component`, `moistair`, `acausal`],
    ports: [`pri_in`, `ind_in`, `out`, `wall`],
    params: [
      { name: `ratio`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `Infiltration`,
    library: `moistair`,
    summary: `Acausal moistair-domain component Infiltration with ports in, out.`,
    tags: [`infiltration`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `C_inf`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `n_exp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `LiquidDesiccantContactor`,
    library: `moistair`,
    summary: `Acausal moistair-domain component LiquidDesiccantContactor with ports in, out, wall.`,
    tags: [`liquiddesiccantcontactor`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `eff_L`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `W_eq`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_T`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`cooled`], defaultValue: `` },
      { name: `f_excess`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`adiabatic`], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`cooled`, `adiabatic`], variants: [], defaultValue: `cooled` }
    ],
    variants: [{ name: `cooled`, requires: [`eps_T`] }, { name: `adiabatic`, requires: [`f_excess`] }],
  },
  {
    type: `MembraneHumidifier`,
    library: `moistair`,
    summary: `Acausal moistair-domain component MembraneHumidifier with ports dry_in, dry_out, wet_in, wet_out.`,
    tags: [`membranehumidifier`, `component`, `moistair`, `acausal`],
    ports: [`dry_in`, `dry_out`, `wet_in`, `wet_out`],
    params: [
      { name: `eff_h`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eff_w`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `MixingBox`,
    library: `moistair`,
    summary: `Mixes two humid-air streams with flow-weighted enthalpy and humidity ratio.`,
    tags: [`mixingbox`, `component`, `moistair`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `MoistAirDamper`,
    library: `moistair`,
    summary: `Acausal moistair-domain component MoistAirDamper with ports in, outa, outb.`,
    tags: [`moistairdamper`, `component`, `moistair`, `acausal`],
    ports: [`in`, `outa`, `outb`],
    params: [
      { name: `u`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `MoistAirDuct`,
    library: `moistair`,
    summary: `Acausal moistair-domain component MoistAirDuct with ports in, out.`,
    tags: [`moistairduct`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rough`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mu_a`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `MoistAirFan`,
    library: `moistair`,
    summary: `Acausal moistair-domain component MoistAirFan with ports in, out.`,
    tags: [`moistairfan`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `dP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `MoistAirSink`,
    library: `moistair`,
    summary: `A humid-air boundary absorbing a stream.`,
    tags: [`moistairsink`, `component`, `moistair`, `acausal`],
    ports: [`in`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `MoistAirSource`,
    library: `moistair`,
    summary: `A humid-air boundary supplying a stream of set state.`,
    tags: [`moistairsource`, `component`, `moistair`, `acausal`],
    ports: [`out`],
    params: [
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `W`, isString: false, isSelector: false, isMap: false, unit: `W`, description: `Humidity ratio [kg/kg] / work [W].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Mass flow rate [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `MoistAirWallHX`,
    library: `moistair`,
    summary: `A humid-air-to-wall heat exchanger.`,
    tags: [`moistairwallhx`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Effectiveness / roughness.`, required: true, values: [], variants: [`eps_t`], defaultValue: `` },
      { name: `NTU`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`ntu_wet`], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `moistair` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`eps_t`, `ntu_wet`], variants: [], defaultValue: `eps_t` }
    ],
    variants: [{ name: `eps_t`, requires: [`eps`] }, { name: `ntu_wet`, requires: [`NTU`] }],
  },
  {
    type: `RadiantPanel`,
    library: `moistair`,
    summary: `Acausal moistair-domain component RadiantPanel with ports zone, wall.`,
    tags: [`radiantpanel`, `component`, `moistair`, `acausal`],
    ports: [`zone`, `wall`],
    params: [
      { name: `A`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `n`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_dT`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `W_room`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P_room`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SensibleAirToAirHX`,
    library: `moistair`,
    summary: `Acausal moistair-domain component SensibleAirToAirHX with ports sup_in, sup_out, exh_in, exh_out.`,
    tags: [`sensibleairtoairhx`, `component`, `moistair`, `acausal`],
    ports: [`sup_in`, `sup_out`, `exh_in`, `exh_out`],
    params: [
      { name: `eff`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eatr`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `oacf`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `SteamHumidifier`,
    library: `moistair`,
    summary: `Acausal moistair-domain component SteamHumidifier with ports in, out.`,
    tags: [`steamhumidifier`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `W_set`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h_steam`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `TotalEnergyExchanger`,
    library: `moistair`,
    summary: `Acausal moistair-domain component TotalEnergyExchanger with ports sup_in, sup_out, exh_in, exh_out.`,
    tags: [`totalenergyexchanger`, `component`, `moistair`, `acausal`],
    ports: [`sup_in`, `sup_out`, `exh_in`, `exh_out`],
    params: [
      { name: `eps_s`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_L`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eatr`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `oacf`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `VAVBox`,
    library: `moistair`,
    summary: `Acausal moistair-domain component VAVBox with ports in, out, u, ur.`,
    tags: [`vavbox`, `component`, `moistair`, `acausal`],
    ports: [`in`, `out`, `u`, `ur`],
    params: [
      { name: `mdot_max`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Qr_max`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `moistair` }
    ],
    variants: [],
  },
  {
    type: `AnodeRecirc`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component AnodeRecirc with ports sup_in, ret_in, out.`,
    tags: [`anoderecirc`, `component`, `pneumatic`, `acausal`],
    ports: [`sup_in`, `ret_in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `ER`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `GasMixer`,
    library: `pneumatic`,
    summary: `Mixes pneumatic gas streams, carrying the species composition rider.`,
    tags: [`gasmixer`, `component`, `pneumatic`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `GasMixerN`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component GasMixerN with ports in1, in2, out.`,
    tags: [`gasmixern`, `component`, `pneumatic`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `GasPipe`,
    library: `pneumatic`,
    summary: `A pneumatic pipe with compressible-flow pressure drop.`,
    tags: [`gaspipe`, `component`, `pneumatic`, `acausal`],
    ports: [`in`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `GasSource`,
    library: `pneumatic`,
    summary: `A boundary supplying gas at set conditions.`,
    tags: [`gassource`, `component`, `pneumatic`, `acausal`],
    ports: [`out`],
    params: [
      { name: `y`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Position / fraction.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Mass flow rate [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h0`, isString: false, isSelector: false, isMap: false, unit: `J/kg`, description: `Reference enthalpy [J/kg].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `PneumaticActuator`,
    library: `pneumatic`,
    summary: `A pneumatic cylinder/actuator converting pressure to force.`,
    tags: [`pneumaticactuator`, `component`, `pneumatic`, `acausal`],
    ports: [`in`, `rod`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `area`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Area [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Patm`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Atmospheric pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticAtmosphere`,
    library: `pneumatic`,
    summary: `An atmospheric (ambient-pressure) pneumatic boundary.`,
    tags: [`pneumaticatmosphere`, `component`, `pneumatic`, `acausal`],
    ports: [`port`],
    params: [
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticCheckValve`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component PneumaticCheckValve with ports in, out.`,
    tags: [`pneumaticcheckvalve`, `component`, `pneumatic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticDoubleActingCylinder`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component PneumaticDoubleActingCylinder with ports a, b, rod.`,
    tags: [`pneumaticdoubleactingcylinder`, `component`, `pneumatic`, `acausal`],
    ports: [`a`, `b`, `rod`],
    params: [
      { name: `Aa`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Ab`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Va0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Vb0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Pa0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Pb0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticOrifice`,
    library: `pneumatic`,
    summary: `A pneumatic orifice metering flow by ISO 6358 (sonic conductance).`,
    tags: [`pneumaticorifice`, `component`, `pneumatic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Critical pressure ratio / coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticServoValve`,
    library: `pneumatic`,
    summary: `A pneumatic servo valve with a commanded spool position.`,
    tags: [`pneumaticservovalve`, `component`, `pneumatic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Cmax`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Maximum capacity rate [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Critical pressure ratio / coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `u`, isString: false, isSelector: false, isMap: false, unit: `J/kg`, description: `Specific internal energy [J/kg].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticServoValveCmd`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component PneumaticServoValveCmd with ports in, out, u.`,
    tags: [`pneumaticservovalvecmd`, `component`, `pneumatic`, `acausal`],
    ports: [`in`, `out`, `u`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Cmax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticSupply`,
    library: `pneumatic`,
    summary: `A pneumatic pressure supply.`,
    tags: [`pneumaticsupply`, `component`, `pneumatic`, `acausal`],
    ports: [`out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticThermalVolume`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component PneumaticThermalVolume with ports in, out, wall.`,
    tags: [`pneumaticthermalvolume`, `component`, `pneumatic`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cv`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `m0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticValve32`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component PneumaticValve32 with ports sup_in, work, exh_out, u.`,
    tags: [`pneumaticvalve32`, `component`, `pneumatic`, `acausal`],
    ports: [`sup_in`, `work`, `exh_out`, `u`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticValve52`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component PneumaticValve52 with ports sup_in, wa, wb, ea_out, eb_out, u.`,
    tags: [`pneumaticvalve52`, `component`, `pneumatic`, `acausal`],
    ports: [`sup_in`, `wa`, `wb`, `ea_out`, `eb_out`, `u`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `PneumaticVolume`,
    library: `pneumatic`,
    summary: `A pneumatic control volume (compressible capacitance).`,
    tags: [`pneumaticvolume`, `component`, `pneumatic`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: `m^3`, description: `Volume [m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: `ohm`, description: `Resistance [Ω].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Reference/initial pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `VacuumEjector`,
    library: `pneumatic`,
    summary: `Acausal pneumatic-domain component VacuumEjector with ports sup_in, suc_in, exh_out.`,
    tags: [`vacuumejector`, `component`, `pneumatic`, `acausal`],
    ports: [`sup_in`, `suc_in`, `exh_out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `ER`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `gas` }
    ],
    variants: [],
  },
  {
    type: `AutomaticTransmission`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component AutomaticTransmission with ports in, out, gear, lock.`,
    tags: [`automatictransmission`, `component`, `powertrain`, `acausal`],
    ports: [`in`, `out`, `gear`, `lock`],
    params: [
      { name: `Kmap$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `TRmap$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tlock`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `CatalystLightOff`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component CatalystLightOff with ports in, out.`,
    tags: [`catalystlightoff`, `component`, `powertrain`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T50`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `q_exo`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Differential`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component Differential with ports in, left, right.`,
    tags: [`differential`, `component`, `powertrain`, `acausal`],
    ports: [`in`, `left`, `right`],
    params: [
      { name: `ratio`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `DriveCycleSource`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component DriveCycleSource with ports port.`,
    tags: [`drivecyclesource`, `component`, `powertrain`, `acausal`],
    ports: [`port`],
    params: [
      { name: `map$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Engine`,
    library: `powertrain`,
    summary: `An internal-combustion engine acting as a torque source.`,
    tags: [`engine`, `component`, `powertrain`, `acausal`],
    ports: [`shaft`],
    params: [
      { name: `Tmax`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Maximum temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `throttle`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Throttle (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `bf`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Friction coefficient.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ExhaustPipeThermal`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component ExhaustPipeThermal with ports in, out, amb.`,
    tags: [`exhaustpipethermal`, `component`, `powertrain`, `acausal`],
    ports: [`in`, `out`, `amb`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `hA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C2`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T10`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T20`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `GearboxScheduled`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component GearboxScheduled with ports in, out, u.`,
    tags: [`gearboxscheduled`, `component`, `powertrain`, `acausal`],
    ports: [`in`, `out`, `u`],
    params: [
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `GradeProfile`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component GradeProfile with ports port.`,
    tags: [`gradeprofile`, `component`, `powertrain`, `acausal`],
    ports: [`port`],
    params: [
      { name: `m`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `g`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `map$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `s0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `GradeRoadLoad`,
    library: `powertrain`,
    summary: `A vehicle road load including the road-grade contribution.`,
    tags: [`graderoadload`, `component`, `powertrain`, `acausal`],
    ports: [`shaft`],
    params: [
      { name: `Crr`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Rolling-resistance coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Caero`, isString: false, isSelector: false, isMap: false, unit: `kg/m`, description: `Aerodynamic drag term ½ρCdA [kg/m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `m`, isString: false, isSelector: false, isMap: false, unit: `kg`, description: `Mass [kg].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `g`, isString: false, isSelector: false, isMap: false, unit: `m/s^2`, description: `Gravitational acceleration [m/s²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `grade`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Road grade (rise/run).`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `HybridPowerSplit`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component HybridPowerSplit with ports eng, out, sun, p, n, u1, u2, heat.`,
    tags: [`hybridpowersplit`, `component`, `powertrain`, `acausal`],
    ports: [`eng`, `out`, `sun`, `p`, `n`, `u1`, `u2`, `heat`],
    params: [
      { name: `g`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eff1$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eff2$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `MeanValueEngine`,
    library: `powertrain`,
    summary: `A mean-value engine model (cycle-averaged torque and flows).`,
    tags: [`meanvalueengine`, `component`, `powertrain`, `acausal`],
    ports: [`shaft`],
    params: [
      { name: `throttle`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Throttle (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Tpeak`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Peak temperature [K].`, required: true, values: [], variants: [`parabolic`, `chenflynn`], defaultValue: `` },
      { name: `w_peak`, isString: false, isSelector: false, isMap: false, unit: `rad/s`, description: `Peak frequency [rad/s].`, required: true, values: [], variants: [`parabolic`, `chenflynn`], defaultValue: `` },
      { name: `FMEP_a`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Friction-MEP constant [Pa].`, required: true, values: [], variants: [`parabolic`, `wotmap`, `bsfc`, `chenflynn`], defaultValue: `` },
      { name: `FMEP_b`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Friction-MEP slope coefficient.`, required: true, values: [], variants: [`parabolic`, `wotmap`, `bsfc`, `chenflynn`], defaultValue: `` },
      { name: `FMEP_c`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`chenflynn`], defaultValue: `` },
      { name: `map_wot$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [`wotmap`, `bsfc`], defaultValue: `` },
      { name: `bsfc$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`bsfc`], defaultValue: `` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`parabolic`, `wotmap`, `bsfc`, `chenflynn`], variants: [], defaultValue: `parabolic` }
    ],
    variants: [{ name: `parabolic`, requires: [`Tpeak`, `w_peak`, `FMEP_a`, `FMEP_b`] }, { name: `wotmap`, requires: [`map_wot$`, `FMEP_a`, `FMEP_b`] }, { name: `bsfc`, requires: [`map_wot$`, `bsfc$`, `FMEP_a`, `FMEP_b`] }, { name: `chenflynn`, requires: [`Tpeak`, `w_peak`, `FMEP_a`, `FMEP_b`, `FMEP_c`] }],
  },
  {
    type: `QuarterCar`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component QuarterCar with ports road.`,
    tags: [`quartercar`, `component`, `powertrain`, `acausal`],
    ports: [`road`],
    params: [
      { name: `ms`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mu`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `ks`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cs`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `kt`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `RoadLoad`,
    library: `powertrain`,
    summary: `A vehicle road load (aerodynamic drag + rolling resistance).`,
    tags: [`roadload`, `component`, `powertrain`, `acausal`],
    ports: [`shaft`],
    params: [
      { name: `Crr`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Rolling-resistance coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Caero`, isString: false, isSelector: false, isMap: false, unit: `kg/m`, description: `Aerodynamic drag term ½ρCdA [kg/m].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TireLongitudinal`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component TireLongitudinal with ports wheel, veh.`,
    tags: [`tirelongitudinal`, `component`, `powertrain`, `acausal`],
    ports: [`wheel`, `veh`],
    params: [
      { name: `r`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Fz`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `B`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsv`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TirePacejka`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component TirePacejka with ports wheel, veh.`,
    tags: [`tirepacejka`, `component`, `powertrain`, `acausal`],
    ports: [`wheel`, `veh`],
    params: [
      { name: `r`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Fz`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `B`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `E`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsv`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TorqueConverter`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component TorqueConverter with ports pump, turb.`,
    tags: [`torqueconverter`, `component`, `powertrain`, `acausal`],
    ports: [`pump`, `turb`],
    params: [
      { name: `Kmap$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `TRmap$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `Transmission`,
    library: `powertrain`,
    summary: `A gearbox/transmission imposing a ratio between engine and wheels.`,
    tags: [`transmission`, `component`, `powertrain`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `ratio`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Gear / split ratio.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Efficiency (0–1).`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `VehicleBody`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component VehicleBody with ports port.`,
    tags: [`vehiclebody`, `component`, `powertrain`, `acausal`],
    ports: [`port`],
    params: [
      { name: `m`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Cd`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Af`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rhoA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Crr`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `grade`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `v0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `WindRotor`,
    library: `powertrain`,
    summary: `Acausal powertrain-domain component WindRotor with ports shaft, wind, pitch.`,
    tags: [`windrotor`, `component`, `powertrain`, `acausal`],
    ports: [`shaft`, `wind`, `pitch`],
    params: [
      { name: `rho`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `R`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cp$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsv`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `epsw`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigAbs`,
    library: `signal`,
    summary: `Acausal signal-domain component SigAbs with ports in, out.`,
    tags: [`sigabs`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigBias`,
    library: `signal`,
    summary: `Acausal signal-domain component SigBias with ports in, out.`,
    tags: [`sigbias`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `b`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigConstant`,
    library: `signal`,
    summary: `Acausal signal-domain component SigConstant with ports out.`,
    tags: [`sigconstant`, `component`, `signal`, `acausal`],
    ports: [`out`],
    params: [
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigDeadband`,
    library: `signal`,
    summary: `Acausal signal-domain component SigDeadband with ports in, out.`,
    tags: [`sigdeadband`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `w`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigDerivative`,
    library: `signal`,
    summary: `Acausal signal-domain component SigDerivative with ports in, out.`,
    tags: [`sigderivative`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `tau`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `y0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigDiff`,
    library: `signal`,
    summary: `Acausal signal-domain component SigDiff with ports in1, in2, out.`,
    tags: [`sigdiff`, `component`, `signal`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigDivide`,
    library: `signal`,
    summary: `Acausal signal-domain component SigDivide with ports in1, in2, out.`,
    tags: [`sigdivide`, `component`, `signal`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigFirstOrder`,
    library: `signal`,
    summary: `Acausal signal-domain component SigFirstOrder with ports in, out.`,
    tags: [`sigfirstorder`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `tau`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `y0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigGain`,
    library: `signal`,
    summary: `Acausal signal-domain component SigGain with ports in, out.`,
    tags: [`siggain`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `k`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigIntegrator`,
    library: `signal`,
    summary: `Acausal signal-domain component SigIntegrator with ports in, out.`,
    tags: [`sigintegrator`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `y0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigLeadLag`,
    library: `signal`,
    summary: `Acausal signal-domain component SigLeadLag with ports in, out.`,
    tags: [`sigleadlag`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `T1`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T2`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `y0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigMap`,
    library: `signal`,
    summary: `Acausal signal-domain component SigMap with ports in, out.`,
    tags: [`sigmap`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `map$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigMap2`,
    library: `signal`,
    summary: `Acausal signal-domain component SigMap2 with ports in1, in2, out.`,
    tags: [`sigmap2`, `component`, `signal`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [
      { name: `map$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigMax`,
    library: `signal`,
    summary: `Acausal signal-domain component SigMax with ports in1, in2, out.`,
    tags: [`sigmax`, `component`, `signal`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigMin`,
    library: `signal`,
    summary: `Acausal signal-domain component SigMin with ports in1, in2, out.`,
    tags: [`sigmin`, `component`, `signal`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigPID`,
    library: `signal`,
    summary: `Acausal signal-domain component SigPID with ports sp, pv, out.`,
    tags: [`sigpid`, `component`, `signal`, `acausal`],
    ports: [`sp`, `pv`, `out`],
    params: [
      { name: `Kp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Ki`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Kd`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `tau`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `i0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `d0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: ``, required: false, values: [`basic`, `clamped`], variants: [], defaultValue: `basic` },
      { name: `umin`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`clamped`], defaultValue: `` },
      { name: `umax`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`clamped`], defaultValue: `` },
      { name: `Taw`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`clamped`], defaultValue: `` }
    ],
    variants: [{ name: `basic`, requires: [] }, { name: `clamped`, requires: [`umin`, `umax`, `Taw`] }],
  },
  {
    type: `SigProduct`,
    library: `signal`,
    summary: `Acausal signal-domain component SigProduct with ports in1, in2, out.`,
    tags: [`sigproduct`, `component`, `signal`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigPulse`,
    library: `signal`,
    summary: `Acausal signal-domain component SigPulse with ports out.`,
    tags: [`sigpulse`, `component`, `signal`, `acausal`],
    ports: [`out`],
    params: [
      { name: `t0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `width`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `high`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `low`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigRamp`,
    library: `signal`,
    summary: `Acausal signal-domain component SigRamp with ports out.`,
    tags: [`sigramp`, `component`, `signal`, `acausal`],
    ports: [`out`],
    params: [
      { name: `t0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `slope`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigRateLimiter`,
    library: `signal`,
    summary: `Acausal signal-domain component SigRateLimiter with ports in, out.`,
    tags: [`sigratelimiter`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `rate`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `tau`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `y0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigRelay`,
    library: `signal`,
    summary: `Acausal signal-domain component SigRelay with ports in, out.`,
    tags: [`sigrelay`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `thresh`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `low`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `high`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigSaturation`,
    library: `signal`,
    summary: `Acausal signal-domain component SigSaturation with ports in, out.`,
    tags: [`sigsaturation`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `lo`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `hi`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigSecondOrder`,
    library: `signal`,
    summary: `Acausal signal-domain component SigSecondOrder with ports in, out.`,
    tags: [`sigsecondorder`, `component`, `signal`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `wn`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `zeta`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `y0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `v0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigSine`,
    library: `signal`,
    summary: `Acausal signal-domain component SigSine with ports out.`,
    tags: [`sigsine`, `component`, `signal`, `acausal`],
    ports: [`out`],
    params: [
      { name: `amp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `freq`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `phase`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `bias`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigSpeedProbe`,
    library: `signal`,
    summary: `Acausal signal-domain component SigSpeedProbe with ports shaft, out.`,
    tags: [`sigspeedprobe`, `component`, `signal`, `acausal`],
    ports: [`shaft`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigStep`,
    library: `signal`,
    summary: `Acausal signal-domain component SigStep with ports out.`,
    tags: [`sigstep`, `component`, `signal`, `acausal`],
    ports: [`out`],
    params: [
      { name: `t0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `before`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `after`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigSum`,
    library: `signal`,
    summary: `Acausal signal-domain component SigSum with ports in1, in2, out.`,
    tags: [`sigsum`, `component`, `signal`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigSwitch`,
    library: `signal`,
    summary: `Acausal signal-domain component SigSwitch with ports in1, in2, ctrl, out.`,
    tags: [`sigswitch`, `component`, `signal`, `acausal`],
    ports: [`in1`, `in2`, `ctrl`, `out`],
    params: [
      { name: `thresh`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigTable`,
    library: `signal`,
    summary: `Acausal signal-domain component SigTable with ports out.`,
    tags: [`sigtable`, `component`, `signal`, `acausal`],
    ports: [`out`],
    params: [
      { name: `map$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `SigThermalProbe`,
    library: `signal`,
    summary: `Acausal signal-domain component SigThermalProbe with ports port, out.`,
    tags: [`sigthermalprobe`, `component`, `signal`, `acausal`],
    ports: [`port`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigTime`,
    library: `signal`,
    summary: `Acausal signal-domain component SigTime with ports out.`,
    tags: [`sigtime`, `component`, `signal`, `acausal`],
    ports: [`out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SigVelProbe`,
    library: `signal`,
    summary: `Acausal signal-domain component SigVelProbe with ports port, out.`,
    tags: [`sigvelprobe`, `component`, `signal`, `acausal`],
    ports: [`port`, `out`],
    params: [

    ],
    variants: [],
  },
  {
    type: `SupervisoryECMS`,
    library: `signal`,
    summary: `Acausal signal-domain component SupervisoryECMS with ports soc, dem, eng, mot.`,
    tags: [`supervisoryecms`, `component`, `signal`, `acausal`],
    ports: [`soc`, `dem`, `eng`, `mot`],
    params: [
      { name: `soc_ref`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `ZoneCO2`,
    library: `signal`,
    summary: `Acausal signal-domain component ZoneCO2 with ports vent, occ, out.`,
    tags: [`zoneco2`, `component`, `signal`, `acausal`],
    ports: [`vent`, `occ`, `out`],
    params: [
      { name: `Vz`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `c_amb`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `gen_occ`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `c0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `BlendMixer`,
    library: `twophase`,
    summary: `A gas-blend (mixture) mixing junction carrying the species rider.`,
    tags: [`blendmixer`, `component`, `twophase`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `BlendSensor`,
    library: `twophase`,
    summary: `A sensor reading the state of a gas-blend stream.`,
    tags: [`blendsensor`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `BlendSink`,
    library: `twophase`,
    summary: `A boundary absorbing a gas-blend stream.`,
    tags: [`blendsink`, `component`, `twophase`, `acausal`],
    ports: [`in`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `BlendSource`,
    library: `twophase`,
    summary: `A boundary supplying a gas-blend stream of set composition.`,
    tags: [`blendsource`, `component`, `twophase`, `acausal`],
    ports: [`out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Mass flow rate [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `x`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Vapor quality / fraction (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `z`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Elevation [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `BoilingVessel`,
    library: `twophase`,
    summary: `A rigid vessel boiling a two-phase fluid (rigid two-phase boil-off).`,
    tags: [`boilingvessel`, `component`, `twophase`, `acausal`],
    ports: [`vent`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: `m^3`, description: `Volume [m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `m0`, isString: false, isSelector: false, isMap: false, unit: `kg`, description: `Initial mass [kg].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T0`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Reference/initial temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `CapillaryTube`,
    library: `twophase`,
    summary: `Acausal twophase-domain component CapillaryTube with ports in, out.`,
    tags: [`capillarytube`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `n`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `EjectorMomentum`,
    library: `twophase`,
    summary: `Acausal twophase-domain component EjectorMomentum with ports mot_in, suc_in, out.`,
    tags: [`ejectormomentum`, `component`, `twophase`, `acausal`],
    ports: [`mot_in`, `suc_in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_n`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta_m`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `FewCellCondenser`,
    library: `twophase`,
    summary: `Acausal twophase-domain component FewCellCondenser with ports in, out, w1, w2, w3.`,
    tags: [`fewcellcondenser`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `w1`, `w2`, `w3`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Cc`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Kv`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `FewCellEvaporator`,
    library: `twophase`,
    summary: `Acausal twophase-domain component FewCellEvaporator with ports in, out, w1, w2, w3.`,
    tags: [`fewcellevaporator`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `w1`, `w2`, `w3`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Cc`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Kv`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `FlashTank`,
    library: `twophase`,
    summary: `Acausal twophase-domain component FlashTank with ports in, liq, vap.`,
    tags: [`flashtank`, `component`, `twophase`, `acausal`],
    ports: [`in`, `liq`, `vap`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `GasCooler`,
    library: `twophase`,
    summary: `Acausal twophase-domain component GasCooler with ports in, out, wall.`,
    tags: [`gascooler`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `MovingBoundaryCondenser`,
    library: `twophase`,
    summary: `A moving-boundary condenser tracking the two-phase/subcooled zone lengths.`,
    tags: [`movingboundarycondenser`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U_cond`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Condenser-zone overall coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U_sc`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Subcool-zone overall coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_zone`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Zone-collapse smoothing width.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `MovingBoundaryEvaporator`,
    library: `twophase`,
    summary: `A moving-boundary evaporator tracking the two-phase/superheat zone lengths.`,
    tags: [`movingboundaryevaporator`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U_tp`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Two-phase-zone overall coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `U_sh`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Superheat-zone overall coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps_zone`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Zone-collapse smoothing width.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `OilSeparator`,
    library: `twophase`,
    summary: `Acausal twophase-domain component OilSeparator with ports in, out, bleed.`,
    tags: [`oilseparator`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `bleed`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `f`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `ProportionalReliefValve`,
    library: `twophase`,
    summary: `A pressure-relief valve whose opening rises proportionally above the set pressure.`,
    tags: [`proportionalreliefvalve`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Pcrack`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Cracking (relief) pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `grad`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Road grade (rise/run).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Effectiveness / roughness.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `ReversingValve`,
    library: `twophase`,
    summary: `Acausal twophase-domain component ReversingValve with ports d, s, i, o.`,
    tags: [`reversingvalve`, `component`, `twophase`, `acausal`],
    ports: [`d`, `s`, `i`, `o`],
    params: [
      { name: `mode`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `SteamReliefValve`,
    library: `twophase`,
    summary: `A steam relief valve venting above the set pressure.`,
    tags: [`steamreliefvalve`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `A`, isString: false, isSelector: false, isMap: false, unit: `m^2`, description: `Area [m²].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Pset`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Set pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Cd`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Discharge coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `kgas`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Gas specific-heat ratio.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Rgas`, isString: false, isSelector: false, isMap: false, unit: `J/kg-K`, description: `Specific gas constant [J/kg·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Effectiveness / roughness.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `SuctionAccumulator`,
    library: `twophase`,
    summary: `Acausal twophase-domain component SuctionAccumulator with ports in, out.`,
    tags: [`suctionaccumulator`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `m0`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `ThreeZoneHX`,
    library: `twophase`,
    summary: `A three-zone (subcooled / two-phase / superheat) heat exchanger.`,
    tags: [`threezonehx`, `component`, `twophase`, `acausal`],
    ports: [`hot_in`, `hot_out`, `cold_in`, `cold_out`],
    params: [
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Overall conductance UA [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `hot$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Hot-side fluid name (e.g. Water).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `cold$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Cold-side fluid name (e.g. EG50).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `arr$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Flow arrangement (passed to hx_effectiveness) — one of \`counterflow\`, \`parallel\`.`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TranscriticalBackPressureValve`,
    library: `twophase`,
    summary: `Acausal twophase-domain component TranscriticalBackPressureValve with ports in, out, u.`,
    tags: [`transcriticalbackpressurevalve`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `u`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `CdA_max`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseCap`,
    library: `twophase`,
    summary: `A two-phase capacitive volume (a pressure-compliance node).`,
    tags: [`twophasecap`, `component`, `twophase`, `acausal`],
    ports: [`in`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseChamber`,
    library: `twophase`,
    summary: `A two-phase control volume.`,
    tags: [`twophasechamber`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: `m^3`, description: `Volume [m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Overall conductance UA [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Reference/initial pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h0`, isString: false, isSelector: false, isMap: false, unit: `J/kg`, description: `Reference enthalpy [J/kg].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseCompressor`,
    library: `twophase`,
    summary: `A refrigerant compressor with selectable isentropic/volumetric variants.`,
    tags: [`twophasecompressor`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eta`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Efficiency (0–1).`, required: true, values: [], variants: [`isentropic`, `volumetric`], defaultValue: `` },
      { name: `eta_v`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`volumetric`], defaultValue: `` },
      { name: `disp`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`volumetric`, `map2d`], defaultValue: `` },
      { name: `rpm`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [`volumetric`, `map2d`], defaultValue: `` },
      { name: `map_etas$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [`map2d`], defaultValue: `` },
      { name: `map_etav$`, isString: true, isSelector: false, isMap: true, unit: ``, description: ``, required: true, values: [], variants: [`map2d`], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` },
      { name: `model$`, isString: true, isSelector: true, isMap: false, unit: ``, description: `Model variant — selects the physics body (see Model Variants).`, required: false, values: [`isentropic`, `volumetric`, `map2d`], variants: [], defaultValue: `isentropic` }
    ],
    variants: [{ name: `isentropic`, requires: [`eta`] }, { name: `volumetric`, requires: [`eta`, `eta_v`, `disp`, `rpm`] }, { name: `map2d`, requires: [`map_etas$`, `map_etav$`, `disp`, `rpm`] }],
  },
  {
    type: `TwoPhaseCondenser`,
    library: `twophase`,
    summary: `A two-phase condenser rejecting heat from the refrigerant.`,
    tags: [`twophasecondenser`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `SC_set`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Target subcooling [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Nominal pressure drop [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseCondenserFloat`,
    library: `twophase`,
    summary: `A two-phase condenser whose pressure floats with the charge/ambient balance.`,
    tags: [`twophasecondenserfloat`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Overall conductance UA [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T_amb`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Ambient temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseCondenserUA`,
    library: `twophase`,
    summary: `A two-phase condenser sized by an overall conductance UA.`,
    tags: [`twophasecondenserua`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Overall conductance UA [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `T_amb`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Ambient temperature [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: `m^3`, description: `Volume [m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseEjector`,
    library: `twophase`,
    summary: `Acausal twophase-domain component TwoPhaseEjector with ports m, s, out.`,
    tags: [`twophaseejector`, `component`, `twophase`, `acausal`],
    ports: [`m`, `s`, `out`],
    params: [
      { name: `PLR`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseEnthalpySource`,
    library: `twophase`,
    summary: `A two-phase boundary fixing the stream enthalpy.`,
    tags: [`twophaseenthalpysource`, `component`, `twophase`, `acausal`],
    ports: [`out`],
    params: [
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Mass flow rate [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Heat-transfer coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseEvaporator`,
    library: `twophase`,
    summary: `A two-phase evaporator absorbing heat into the refrigerant.`,
    tags: [`twophaseevaporator`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `SH_set`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Target superheat [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Nominal pressure drop [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseEvaporatorUA`,
    library: `twophase`,
    summary: `A two-phase evaporator sized by an overall conductance UA.`,
    tags: [`twophaseevaporatorua`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `wall`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `UA`, isString: false, isSelector: false, isMap: false, unit: `W/K`, description: `Overall conductance UA [W/K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `dP`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Nominal pressure drop [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `SH`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Superheat [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseExpansionValve`,
    library: `twophase`,
    summary: `A refrigerant expansion valve (isenthalpic throttle).`,
    tags: [`twophaseexpansionvalve`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Cv`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Flow coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseFlowRes`,
    library: `twophase`,
    summary: `A two-phase flow resistance relating pressure drop to mass flow.`,
    tags: [`twophaseflowres`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseInternalHX`,
    library: `twophase`,
    summary: `Acausal twophase-domain component TwoPhaseInternalHX with ports liq_in, liq_out, vap_in, vap_out.`,
    tags: [`twophaseinternalhx`, `component`, `twophase`, `acausal`],
    ports: [`liq_in`, `liq_out`, `vap_in`, `vap_out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `eps`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseInventory`,
    library: `twophase`,
    summary: `Tracks the refrigerant charge inventory across the circuit.`,
    tags: [`twophaseinventory`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: `m^3`, description: `Volume [m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseMixer`,
    library: `twophase`,
    summary: `Mixes two two-phase streams with flow-weighted enthalpy.`,
    tags: [`twophasemixer`, `component`, `twophase`, `acausal`],
    ports: [`in1`, `in2`, `out`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseOilRider`,
    library: `twophase`,
    summary: `Acausal twophase-domain component TwoPhaseOilRider with ports in, out.`,
    tags: [`twophaseoilrider`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `oc_set`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `k_deg`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhasePipe`,
    library: `twophase`,
    summary: `A two-phase pipe with a Lockhart–Martinelli frictional pressure drop.`,
    tags: [`twophasepipe`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `L`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Length [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `D`, isString: false, isSelector: false, isMap: false, unit: `m`, description: `Diameter [m].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rough`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Relative wall roughness.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `x`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Vapor quality / fraction (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho_l`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Liquid density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `rho_g`, isString: false, isSelector: false, isMap: false, unit: `kg/m^3`, description: `Vapor density [kg/m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mu_l`, isString: false, isSelector: false, isMap: false, unit: `Pa-s`, description: `Liquid viscosity [Pa·s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mu_g`, isString: false, isSelector: false, isMap: false, unit: `Pa-s`, description: `Vapor viscosity [Pa·s].`, required: true, values: [], variants: [], defaultValue: `` }
    ],
    variants: [],
  },
  {
    type: `TwoPhasePressureSink`,
    library: `twophase`,
    summary: `A two-phase boundary fixing the pressure (sink).`,
    tags: [`twophasepressuresink`, `component`, `twophase`, `acausal`],
    ports: [`in`],
    params: [
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhasePressureSource`,
    library: `twophase`,
    summary: `A two-phase boundary fixing the pressure (source).`,
    tags: [`twophasepressuresource`, `component`, `twophase`, `acausal`],
    ports: [`out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `x`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Vapor quality / fraction (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseReceiver`,
    library: `twophase`,
    summary: `A liquid receiver buffering refrigerant charge at saturation.`,
    tags: [`twophasereceiver`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: `m^3`, description: `Volume [m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseSensor`,
    library: `twophase`,
    summary: `A sensor reading the two-phase stream state.`,
    tags: [`twophasesensor`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseShortTube`,
    library: `twophase`,
    summary: `Acausal twophase-domain component TwoPhaseShortTube with ports in, out.`,
    tags: [`twophaseshorttube`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `CdA`, isString: false, isSelector: false, isMap: false, unit: ``, description: ``, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: ``, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseSink`,
    library: `twophase`,
    summary: `A boundary absorbing a two-phase stream.`,
    tags: [`twophasesink`, `component`, `twophase`, `acausal`],
    ports: [`in`],
    params: [
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseSource`,
    library: `twophase`,
    summary: `A boundary supplying a two-phase stream.`,
    tags: [`twophasesource`, `component`, `twophase`, `acausal`],
    ports: [`out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Mass flow rate [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `x`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Vapor quality / fraction (0–1).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseSourcePH`,
    library: `twophase`,
    summary: `A two-phase source specified by pressure and enthalpy (P, h).`,
    tags: [`twophasesourceph`, `component`, `twophase`, `acausal`],
    ports: [`out`],
    params: [
      { name: `mdot`, isString: false, isSelector: false, isMap: false, unit: `kg/s`, description: `Mass flow rate [kg/s].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `h`, isString: false, isSelector: false, isMap: false, unit: `W/m^2-K`, description: `Heat-transfer coefficient [W/m²·K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TwoPhaseVolume`,
    library: `twophase`,
    summary: `A finite-volume two-phase control volume with mass and energy states ((p, h) states).`,
    tags: [`twophasevolume`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `V`, isString: false, isSelector: false, isMap: false, unit: `m^3`, description: `Volume [m³].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `C`, isString: false, isSelector: false, isMap: false, unit: `F`, description: `Capacitance [F].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `P0`, isString: false, isSelector: false, isMap: false, unit: `Pa`, description: `Reference/initial pressure [Pa].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  },
  {
    type: `TXVSuperheat`,
    library: `twophase`,
    summary: `A thermostatic expansion valve that meters flow to hold a target superheat.`,
    tags: [`txvsuperheat`, `component`, `twophase`, `acausal`],
    ports: [`in`, `out`, `bulb`],
    params: [
      { name: `fluid$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Fluid name (e.g. Water, R134a, Air).`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `Kv`, isString: false, isSelector: false, isMap: false, unit: ``, description: `Flow coefficient.`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `SH_set`, isString: false, isSelector: false, isMap: false, unit: `K`, description: `Target superheat [K].`, required: true, values: [], variants: [], defaultValue: `` },
      { name: `domain$`, isString: true, isSelector: false, isMap: false, unit: ``, description: `Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`.`, required: false, values: [], variants: [], defaultValue: `twophase` }
    ],
    variants: [],
  }
];
