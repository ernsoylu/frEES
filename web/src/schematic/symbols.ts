// What a component LOOKS like on the canvas.
//
// A schematic where every block is the same rounded rectangle is a graph
// drawing, not a circuit: the reader has to read 11px type names to find the
// pump. Engineering drawings solve this with shape — a pump is a circle, a
// valve is a bow-tie, a heat exchanger is a crossed box, a boundary condition
// is a terminal. This module assigns each component type a SHAPE, so the
// topology is legible at a glance and before any zoom.
//
// Classification is by type-name pattern rather than an enumerated table: the
// library ships 312 components and grows, and the names are systematic
// (`LiquidPump`, `HydraulicPump`, `MoistAirFan`), so patterns keep new
// components drawn correctly without a catalog edit. Anything unmatched falls
// back to a plain block, which is exactly the old behaviour.

export type Shape =
  | 'source' // boundary condition feeding the network
  | 'sink' // boundary condition absorbing it
  | 'pump' // rotating machine adding pressure/head
  | 'compressor' // rotating machine adding pressure to a gas
  | 'turbine' // rotating machine extracting work
  | 'valve' // restriction / metering device
  | 'exchanger' // heat exchanger — two media, one wall
  | 'store' // capacitance: thermal mass, tank, accumulator, battery
  | 'junction' // mixer / splitter / manifold
  | 'sensor' // measurement, physical → signal
  | 'control' // causal signal-domain block
  | 'machine' // electro-mechanical transducer (motor, engine, generator)
  | 'ground' // a fixed potential: ambient, mechanical ground, atmosphere
  | 'block' // anything else

/** What a component is FOR — drives the shape and the grouping band. */
export interface Symbolism {
  shape: Shape
  /** True when the component is a boundary of the model rather than part of
   *  its interior — sources, sinks and grounds. These are the terminals a
   *  circuit reading needs labelled. */
  terminal: boolean
}

/** A rule matching any of a list of substrings, case-insensitively. Built from
 *  a word list rather than written as one long alternation so the categories
 *  stay readable and extending one is a list edit, not a regex edit. */
function anyOf(...words: string[]): RegExp {
  return new RegExp(words.join('|'), 'i')
}

/** Components with two media across a wall. */
const EXCHANGER_WORDS = [
  'hx', 'heatexchanger', 'exchanger', 'evaporator', 'condenser', 'radiator', 'coil',
  'chiller', 'intercooler', 'recuperator', 'regenerator', 'coldplate', 'economizer',
  'coolingtower', 'heatpipe', 'conduction', 'convection', 'radiation', 'wallrc',
  'contactresistance', 'peltier', 'multizonewall',
]

/** Capacitance: anything that stores mass, heat, charge or momentum. */
const STORE_WORDS = [
  'mass', 'tank', 'accumulator', 'volume', 'capacitor', 'inertia', 'battery',
  'storage', 'reservoir', 'pcm', 'inductor', 'spring', 'zone', 'cabin',
]

/** Electro-mechanical and chemical transducers. */
const MACHINE_WORDS = [
  'motor', 'engine', 'generator', 'alternator', 'fuelcell', 'stack', 'electrolyzer',
  'converter', 'charger', 'inverter', 'resistor', 'diode', 'clutch', 'brake', 'gear',
  'transmission', 'differential', 'torqueconverter', 'beltdrive', 'planetary',
  'actuator', 'cylinder',
]

/** Ordered rules — first match wins, so put the specific before the generic. */
const RULES: { re: RegExp; shape: Shape }[] = [
  // Boundaries first: `PneumaticSupply` is a source, not a pneumatic device.
  { re: /(^|[a-z])(source|supply|feed)$/i, shape: 'source' },
  { re: /^(drivecycle|grade)(source|profile)$/i, shape: 'source' },
  { re: /(^|[a-z])(sink|drain|outlet)$/i, shape: 'sink' },
  { re: /(ground|atmosphere|ambient|environment)/i, shape: 'ground' },

  // Measurement before the physics it measures (`ThermalSensor`, `SigSpeedProbe`).
  { re: /(sensor|probe|gauge|meter)$/i, shape: 'sensor' },

  // Causal control blocks — the whole Sig* library plus named controllers.
  { re: /^sig/i, shape: 'control' },
  { re: /(pid|thermostat|controller|supervisory|ecms)/i, shape: 'control' },

  // Rotating machines.
  { re: /compressor/i, shape: 'compressor' },
  { re: /(turbine|expander|rotor)/i, shape: 'turbine' },
  { re: /(pump|fan|blower)/i, shape: 'pump' },

  // Restrictions and metering.
  { re: /(valve|orifice|throttle|txv|damper|restrict|nozzle|ejector)/i, shape: 'valve' },

  // Heat exchange — anything with two media across a wall.
  { re: anyOf(...EXCHANGER_WORDS), shape: 'exchanger' },

  // Capacitance / storage.
  { re: anyOf(...STORE_WORDS), shape: 'store' },

  // Junctions.
  { re: /(mixer|splitter|junction|manifold|tee|collect|divider|mixingbox|recirc)/i, shape: 'junction' },

  // Electro-mechanical / chemical transducers.
  { re: anyOf(...MACHINE_WORDS), shape: 'machine' },
]

const CACHE = new Map<string, Symbolism>()

/** The symbol a component type is drawn with. */
export function symbolOf(type: string | undefined): Symbolism {
  const key = (type ?? '').toLowerCase()
  const hit = CACHE.get(key)
  if (hit) {
    return hit
  }
  const shape = RULES.find((r) => r.re.test(key))?.shape ?? 'block'
  const sym: Symbolism = { shape, terminal: shape === 'source' || shape === 'sink' || shape === 'ground' }
  CACHE.set(key, sym)
  return sym
}

/** Short human name for a shape — the tooltip's "what is this" line. */
export const SHAPE_LABELS: Record<Shape, string> = {
  source: 'source (boundary)',
  sink: 'sink (boundary)',
  pump: 'pump / fan',
  compressor: 'compressor',
  turbine: 'turbine / expander',
  valve: 'valve / restriction',
  exchanger: 'heat exchanger',
  store: 'storage / mass',
  junction: 'junction',
  sensor: 'sensor',
  control: 'control block',
  machine: 'machine / transducer',
  ground: 'ground (fixed potential)',
  block: 'component',
}

/**
 * The glyph drawn INSIDE a node box, as SVG path data in a 0..1 unit square
 * (the renderer scales it). Kept as pure geometry so it is testable and so the
 * exported SVG carries no external symbol font.
 */
export function glyphPath(shape: Shape): string {
  switch (shape) {
    case 'pump':
      // Circle with an impeller triangle — ISO rotodynamic pump.
      return 'M0.5 0.05 A0.45 0.45 0 1 1 0.4999 0.05 Z M0.32 0.25 L0.78 0.5 L0.32 0.75 Z'
    case 'compressor':
      // Converging trapezoid — flow compressed left to right.
      return 'M0.08 0.06 L0.92 0.3 L0.92 0.7 L0.08 0.94 Z'
    case 'turbine':
      // Diverging trapezoid — the compressor mirrored, work extracted.
      return 'M0.08 0.3 L0.92 0.06 L0.92 0.94 L0.08 0.7 Z'
    case 'valve':
      // Bow-tie — the universal restriction symbol.
      return 'M0.08 0.1 L0.08 0.9 L0.92 0.1 L0.92 0.9 Z'
    case 'exchanger':
      // Box with the two crossed diagonals of a heat-exchanger symbol.
      return 'M0.06 0.06 H0.94 V0.94 H0.06 Z M0.06 0.06 L0.94 0.94 M0.94 0.06 L0.06 0.94'
    case 'store':
      // Capacitor plates — the storage element of every bond-graph domain.
      return 'M0.5 0.05 V0.35 M0.12 0.35 H0.88 M0.12 0.62 H0.88 M0.5 0.62 V0.95'
    case 'junction':
      // Diamond — a node where streams split or merge.
      return 'M0.5 0.04 L0.96 0.5 L0.5 0.96 L0.04 0.5 Z'
    case 'sensor':
      // Dial with a needle.
      return 'M0.5 0.05 A0.45 0.45 0 1 1 0.4999 0.05 Z M0.5 0.5 L0.78 0.24'
    case 'control':
      // Summing/gain block with a step trace.
      return 'M0.06 0.12 H0.94 V0.88 H0.06 Z M0.2 0.7 H0.45 V0.32 H0.8'
    case 'machine':
      // Circle with a shaft — a rotating electrical machine.
      return 'M0.5 0.08 A0.42 0.42 0 1 1 0.4999 0.08 Z M0.0 0.5 H0.08 M0.92 0.5 H1.0'
    case 'source':
      // Terminal chevron pointing INTO the network.
      return 'M0.1 0.12 L0.9 0.5 L0.1 0.88 Z'
    case 'sink':
      // Terminal chevron pointing OUT of it.
      return 'M0.9 0.12 L0.1 0.5 L0.9 0.88 Z'
    case 'ground':
      // Ladder ground — fixed potential.
      return 'M0.5 0.05 V0.45 M0.14 0.45 H0.86 M0.26 0.66 H0.74 M0.4 0.87 H0.6'
    default:
      return ''
  }
}

/** Shapes drawn as a filled glyph vs. an outline. Filled reads as "terminal",
 *  which is what a boundary condition should look like. */
export function glyphFilled(shape: Shape): boolean {
  return shape === 'source' || shape === 'sink' || shape === 'valve' || shape === 'junction'
}
