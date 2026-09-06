[Topic: comp-first-network]
# Your First Component Network

frees has a library of **312 components** — reusable, parameterized blocks of physics (pumps, pipes, heat exchangers, resistors, gears, cooling coils …) with typed **ports**. You instantiate them, wire the ports together, and frees expands the network into ordinary scalar equations solved by the same Newton/Tarjan pipeline as everything else. There is no separate "simulation mode": components and plain equations mix freely in one document.

## Water through a pipe

```run
{ Supply -> pipe -> return: what pressure is lost to friction? }
Source  SUP(fluid$=Water, mdot=2 [kg/s], P=300000 [Pa], T=298 [K])
Pipe    LINE(fluid$=Water, L=50 [m], D=0.05 [m], rough=0.0001)
Sink    RET()

connect(SUP.out, LINE.in)
connect(LINE.out, RET.in)

dP = SUP.out.P - RET.in.P     { probe: frictional pressure drop, Pa }
```

Solve (F2) and read `dP` in the Solution panel. Three things happened:

1. **Instantiation** — `Pipe LINE(...)` stamped a copy of the `Pipe` template, filling in its parameters. Every parameter is named (`L=50`), and unit annotations work exactly as in plain equations.
2. **Connection** — each `connect` statement tied two ports into a node: pressures equalize, mass is conserved, enthalpy is carried through.
3. **Probing** — dotted **port members** (`SUP.out.P`, `LINE.in.mdot`) are ordinary solver variables. You can read them, plot them, or pin them with a boundary condition like `RET.in.P = 100000 [Pa]`.

## No causality, by design

Notice what you did *not* write: no "input" or "output" designation, no calculation order. The network is **acausal** — the `Pipe` doesn't know whether it is computing a pressure drop from a flow or a flow from a pressure drop. Fix any consistent set of boundary values and the solver finds the rest, exactly like swapping the unknown in an ordinary frees equation. That is the same declarative idea you met in *Your First Solve*, lifted to whole systems.

## Named outputs

Many components compute results you'll want directly — a compressor's power, an exchanger's duty. These are exposed as **named outputs** on the instance:

```
Compressor CMP(fluid$=R134a, eta=0.72, model$=isentropic)
...
W_comp = CMP.W        { compressor power, W }
```

Every component's ports, parameters, equations, and outputs are documented on its page in the **Reference** — see the A–Z index, or browse by library on *The Component Library* page.

[Related: comp-connections, comp-library, gs-declarative]

[Topic: comp-connections]
# Connections & Junctions

There are two ways to wire a network. Both expand to exactly the same equations — pick whichever reads better.

## Style 1 — connect statements

A `connect` statement ties the listed ports into one **node**. It takes any number of endpoints, so branching is native:

```
connect(PUMP.out, RAD.in, BYPASS.in)   { flow splits after the pump }
```

At a node, frees emits the **junction rules** for the ports' domain (see *Domains & Fluid Families*): the *across* variables equalize (e.g. one pressure), and the *through* variables sum to zero (e.g. Σṁ = 0 — what flows in flows out). For fluid streams the specific enthalpy `h` rides along convectively: equal at a split or pass-through. Merging streams at different states needs an explicit mixer component (`Mixer`, `LiquidMixer`, `MixingBox`, …), which flow-weights the enthalpy properly.

Loops close the same way — connecting the last component back to the first is legal and is how closed circuits (refrigeration loops, coolant circuits) are built.

## Style 2 — shared stream names

For simple series chains there is a terser form: bind ports **positionally** to named streams. Two instances that name the same stream are connected.

```
Source SUP(s1, fluid$=Water, mdot=2, P=300000, T=298)
Pipe   LINE(s1, s2, fluid$=Water, L=50, D=0.05, rough=0.0001)
Sink   RET(s2)
```

Leading positional arguments bind the ports in the component's declared order (`Pipe` declares `in, out`, so `s1` is its inlet and `s2` its outlet); the trailing `name=value` arguments set parameters. Stream members are addressed directly: `s2.P`, `s2.h`, `s2.mdot`.

A stream name may join at most **two** ports — a third is a hard error, because a silent three-way tie is almost always a mistake. Use a `connect` statement when you need branching.

## Boundary conditions

A network needs enough pinned values to close its degrees of freedom, just like any equation system. Pin port members with plain equations:

```
RET.in.P = 100000 [Pa]      { fix the return pressure }
```

Source/sink components (`Source`, `PressureSource`, `MoistAirSource`, `VoltageSource`, `ThermalSource`, …) are pre-packaged boundary conditions; a bare equation on a port member does the same job when no component fits.

[Related: comp-first-network, comp-domains, comp-schematic, comp-troubleshooting]

[Topic: comp-schematic]
# Reading the Schematic

The **Schematic** window draws the network your document describes — open it from the left rail, or from the command palette (Ctrl+K → "Schematic"). It is generated from the text on every **Check**, so it is never out of date and there is nothing to lay out by hand. Everything below is derived; the drawing is a view of the model, not a second copy of it.

It is the fastest way to answer "did I wire that the way I meant to?", because a mis-wired network usually *looks* wrong long before it solves wrong.

## Circuits are drawn apart

Each **working fluid gets its own colour and its own framed band.** This matters more than it sounds: the bond-graph domain calls a coolant loop and a refrigerant loop the same thing — both are `fluid` — so a drawing coloured by domain paints two independent circuits identically and they read as one tangle. frees separates them by *connector type* and *fluid*, so an EG50 coolant loop and an R1234yf refrigerant loop land in bands labelled `EG50 · LIQUID` and `R1234YF · TWO-PHASE`.

| Line | Meaning |
|---|---|
| blue | liquid (coolant, water/glycol) |
| violet | two-phase (refrigerant) |
| teal | generic thermofluid / steam |
| orange | pneumatic (`gas`) |
| amber | hydraulic (`oil`) |
| pale cyan | humid air (`moistair`) |
| red | heat |
| yellow | electrical |
| lime | mechanical (rotational and translational) |
| dashed cyan | signal — a causal control value, not a physical flow |

Two different fluids on the same connector type (two coolant loops, say) take different shades of the same hue, so they stay distinguishable while still reading as "both coolant". The legend above the canvas names every line in the drawing.

A **coupling band** — heat, signal, mechanical — is placed next to the circuit it links to most. In the common shape of two loops bridged by a heat exchanger, that puts the thermal band *between* them, so the couplings are short instead of crossing an unrelated circuit.

## Flow sets the left-to-right order

A component network is acausal — you may write the equations in any order — but the *port names* are not. A `connect` from an `out` port to an `in` port says the first feeds the second, so each circuit is laid out source → … → sink. A closed loop is drawn as a chain with its closing edge running back, which is how it would be drawn by hand.

## Every block says what it is

Blocks carry the symbol of their function, so the topology is legible before you read a single label:

| Symbol | Component kind |
|---|---|
| circle with impeller | pump, fan, blower |
| converging trapezoid | compressor |
| diverging trapezoid | turbine, expander |
| bow-tie | valve, orifice, throttle |
| crossed box | heat exchanger, coil, radiator |
| capacitor plates | storage — thermal mass, tank, accumulator, battery |
| diamond | junction — mixer, splitter, manifold |
| dashed border + chevron | **boundary condition** — a source or a sink |
| ladder | ground (fixed potential) |

Boundaries are drawn dashed on purpose: they are where your model *stops*, and seeing them makes an accidentally open circuit obvious.

## The numbers

After a solve, each block prints its most telling result — a heat exchanger's duty, a machine's power, otherwise the flow through it. **Hover** a block for the rest:

- **Ports** — the state at each wired port (`P`, `T`, `ṁ`, `h`, and the domain's own members: `Q̇` for heat, `V`/`I` for electrical, and so on), in SI with units.
- **Results** — the block's named outputs, the same `CHLR.Q` / `CMP.W` you can reference in equations.
- **Parameters** — what the block was built with, **and where the value came from**. A document that sizes a heat exchanger from correlations and geometry injects the answer as a parameter, so the card shows `ua  576.79 W/K  (UA_chl_r)` — the number *and* the variable, so you can trace it back to the correlation that produced it.

Click a block to pin the card and jump to the line that declares it in the editor.

## Moving things around

| Action | Result |
|---|---|
| drag a block | move it; wires re-route live |
| drag the background | pan |
| Ctrl/⌘ + scroll | zoom |
| **Fit to window** | frame the whole network |
| **Reset layout** | discard your arrangement, return to the automatic one |
| **Export SVG** | the whole drawing as a vector file, ready to paste into a report |

The automatic layout keeps the network framed until you first zoom, pan or drag — after that it leaves the view alone.

**Your arrangement is saved with the project.** Positions are stored as offsets from the automatic layout rather than as fixed coordinates, so when you add a component upstream everything shifts with it instead of being stranded. Positions for components you delete are dropped; positions for components that survive an edit are kept.

## Wiring by clicking

Click a **port dot** on one block, then a port dot on another, and frees appends the matching `connect` statement to your document. Port dots are coloured by the line they carry, so an inlet already on the coolant circuit shows blue — a quick check that you are about to join what you think you are.

## What it will not do

- It draws the network the **Check** understood. If the document has errors, the canvas says so rather than showing a stale or half-drawn network.
- Wires are routed orthogonally but do not steer around blocks, so a dense network will have lines crossing boxes.
- Grouping is automatic, by circuit. There is no manual "group these into a subsystem" — for real hierarchy, use a hierarchical `COMPONENT` (see *Writing Your Own Component*).

[Related: comp-connections, comp-domains, comp-cycle-plots, comp-troubleshooting]

[Topic: comp-domains]
# Domains & Fluid Families

Every port belongs to a **domain** — the pair of *across* / *through* variables it carries and the junction rule a node enforces:

| Domain | Across (equal at a node) | Through (sums to zero) | Carried along |
| --- | --- | --- | --- |
| **Fluid** | pressure `P` | mass flow `mdot` | specific enthalpy `h` (convective) |
| **Heat** | temperature `T` | heat flow `Qdot` | — |
| **Electrical** | voltage `V` | current `I` | — |
| **Mechanical (rotational)** | speed `omega` | torque `tau` | — |
| **Mechanical (translational)** | velocity `v` | force `F` | — |
| **Signal** | value `sig` | *(none — causal)* | — |

The domain is inferred from the members a port carries — you never register it. A port carrying `(P, mdot, h)` is fluid; `(T, Qdot)` is heat, and so on.

## The signal domain: causal command wires

The acausal domains conserve something at a node; the **signal** domain deliberately doesn't. A port referenced as `port.sig` carries a bare value with *no* flow member — a `connect` node simply equates it everywhere, so **one writer broadcasts to any number of readers**, exactly like a control-diagram wire. That is what command inputs, setpoints, and measurements are:

```
SigSine  CMD(amp=0.5, freq=0.2, phase=0, bias=0.5)   { 0..1 command wave }
EXVCmd   VALVE(fluid$=R134a, CdA_max=2e-6)
connect(CMD.out, VALVE.u)                             { the wire }
```

The library ships ~30 signal blocks: **sources** (`SigConstant`, `SigStep`, `SigRamp`, `SigSine`, `SigPulse`, `SigTable` drive cycles), **math** (`SigSum`, `SigGain`, `SigProduct`, …), **nonlinearities** (`SigSaturation`, `SigDeadband`, `SigRelay`, `SigRateLimiter`), **dynamics** (`SigIntegrator`, `SigFirstOrder`, `SigSecondOrder`, `SigLeadLag`), the `SigPID` controller, and **probes** (`SigThermalProbe`, `SigSpeedProbe`, `SigVelProbe`) that read a physical port *into* a signal — the sanctioned way to close a loop around a plant. Signal-to-physical wiring is rejected by the same strict single-domain guard as every other mismatch: commandable actuators expose a dedicated signal port instead (the `u` port on `EXVCmd` above).

## One node, one domain — enforced

A `connect` node must be a single domain. Wiring a heat port to an electrical port is a **hard parse error**, not a warning — frees refuses to build a network that would silently solve the wrong physics.

Crossing domains is what **transducer components** are for: they carry one port *per* domain and the coupling physics inside. `HeatingResistor` has electrical terminals and a heat port (its I²R loss); `LiquidWallHX` has fluid ports and a `wall` heat port; a motor couples electrical to rotational. The pressure-cooker example in the Examples library chains electrical → thermal → two-phase fluid through exactly such components, in one solve.

## Fluid families

Several fluid-like domains share the same `(P, mdot, h)` bond but must never be cross-wired — a pneumatic line makes no sense feeding an oil line. A reserved string parameter, `domain$`, tags each fluid-family connector:

| `domain$` | Family | Typical components |
| --- | --- | --- |
| `fluid` *(default)* | General thermofluid | `Source`, `Pipe`, `Compressor`, `HeatExchanger` |
| `liquid` | Incompressible liquid loops | `LiquidPump`, `LiquidWallHX`, `LiquidMixer` |
| `twophase` | Evaporating / condensing refrigerant | `TwoPhaseCompressor`, `TwoPhaseEvaporatorUA` |
| `gas` | Pneumatics (ISO 6358) | `PneumaticOrifice`, `PneumaticVolume` |
| `oil` | Oil hydraulics | `HydraulicPump`, `ReliefValve` |
| `moistair` | Humid air (HVAC) | `MoistAirSource`, `CoolingCoil`, `MixingBox` |

Connecting mismatched families is, again, a hard error. The built-ins carry the right tag already; your own components opt in with `PARAM domain$ = gas` (see *Writing Your Own Component*).

## Humid air: the W rider

The `moistair` family conserves **two** masses. Its basis is `(P, mdot_da, h, W)`: flow is on a *dry-air* basis (Σṁ_da = 0), and the humidity ratio `W` rides along as a second conserved species — equal across a pass-through connection, flow-weighted only in an explicit `MixingBox`. That rider is what makes a cooling coil able to condense water out of the stream while dry air is conserved. The gas-mixture components use the same pattern for species fractions (`.y`).

[Related: comp-connections, comp-library, humidair]

[Topic: comp-library]
# The Component Library

The standard library ships 312 components across thirteen domain libraries. This page is a map, not a catalog — every component's authoritative page (ports, parameters, variants, governing equations) lives in the **Reference**; find it by name in the A–Z index, or browse it from the Component Wizard. The Component Wizard and editor completion are generated from this port's parsed library, so a standalone checkout stays in agreement with the engine.

| Library | What's in it |
| --- | --- |
| **signal** | Causal control wires: sources (`SigConstant`/`SigStep`/`SigRamp`/`SigSine`/`SigPulse`, `SigTable` drive cycles), block-diagram math, saturation/deadband/relay/rate limits, transfer-function dynamics (`SigFirstOrder`, `SigSecondOrder`, `SigLeadLag`), `SigPID`, map lookups, and physical→signal probes |
| **fluid** | General thermofluid plus gas/aero breadth: `Source`/`Sink`, `Pipe`, `Valve`, `Nozzle`, `Pump`, `Fan`, `Compressor`, `Turbine`, `HeatExchanger`, `Mixer`/`Splitter`, map-driven turbomachines, ducts, regenerator, combustor, ISA atmosphere, propeller |
| **liquid** | Incompressible coolant / TMS loops: `LiquidSource`, `LiquidPump` (+ pump map), `LiquidOrifice`, `LiquidWallHX`, `LiquidMixer`, three-way valve, tank, thermostat, expansion tank |
| **twophase** | Evaporating/condensing refrigerant circuits: boundaries, `TwoPhaseCompressor`, moving-boundary heat exchangers, `TwoPhasePipe` (Lockhart–Martinelli), `TXVSuperheat`, `ThreeZoneHX`, charge/receiver volumes, `BoilingVessel`, VCC devices |
| **ac** | Application composites built on the two-phase set: `Chiller`, `AirCoil`, `Radiator`, `HeaterCore`, `TXV`, `EXV`/`EXVCmd` |
| **moistair** | Humid-air HVAC: `MoistAirSource`/`MoistAirSink`, `CoolingCoil` (wet coils), `HeatingCoil`, `Humidifier`, `MixingBox`, `MoistAirWallHX`, cabin zone |
| **pneumatic** | ISO 6358 compressible gas power: orifices, volumes, valves, cylinders, sources |
| **hydraulic** | Oil-hydraulic power: pumps, orifices, valves, cylinders, accumulators, `ReliefValve` |
| **heat** | Lumped heat transfer: `ThermalSource`, `ThermalMass`, `Conduction`/`Convection`/`Radiation`, `ContactResistance`, `MassGen` (self-heating mass), transient walls, PCM, Peltier, heat pipe |
| **electrical** | Circuits & electrification: `VoltageSource`, `Ground`, resistors (`HeatingResistor` couples to heat), `Capacitor`/`Inductor`, battery cells and packs with SOC, motor/inverter/DC-DC, PV, electrolyzer, `FuelCellStack` (PEMFC) |
| **mechanical** | Rotational & translational 1-D mechanics: `Inertia`, `TransMass`, springs, dampers, `Gear`, `Planetary`, `Clutch`, `Friction`, backlash, hard stops, kinematic pairs |
| **powertrain** | Vehicle-level: engines (`MeanValueEngine`), `Transmission`, torque converter, tire, vehicle body, `GradeRoadLoad`, drive cycles |
| **control** | Network-level sensors and controllers (e.g. `PIThermostat`, `ThermalSensor`, `FlowSensor`) — see the **signal** library for full block-diagram control |

Three conventions hold across the whole library:

- **No hidden defaults.** Every physical parameter must be given explicitly at instantiation — a missing one is an error, never a silent assumption.
- **Naming tells you the family.** `Liquid*`, `TwoPhase*`, `Pneumatic*`, `Hydraulic*`, `MoistAir*` prefixes mark the fluid family (and its `domain$` tag).
- **Fidelity is selectable, not duplicated.** Where one machine has several physics levels (a compressor with isentropic-η, volumetric, or map-based models), it is *one* component with a `model$` selector — see *Fidelity Variants*.

[Related: comp-variants, comp-first-network, ref-index]

[Topic: comp-variants]
# Fidelity Variants (model$)

Real projects move through fidelity levels: a first-cut cycle needs only an isentropic efficiency; the sized design wants the volumetric model; the calibrated digital twin wants the manufacturer's map. In frees that is **one component, many models** — a `model$` parameter selects which physics body is expanded:

```
{ concept study }
Compressor CMP(fluid$=R134a, eta=0.72, model$=isentropic)

{ sized design: same component, higher fidelity }
Compressor CMP(fluid$=R134a, eta=0.72, model$=volumetric,
               eta_v=0.92, disp=6.5e-5, rpm=2900)
```

Because the component and its ports don't change, **the network around it doesn't change either** — you upgrade fidelity by editing one line, not rewiring the model.

## Per-variant required parameters

Each variant declares the parameters it needs (`REQUIRE`), validated only when that variant is selected. Choosing `model$=volumetric` without `disp` is an immediate, named error. Supplying `rpm` on an isentropic compressor is accepted for compatibility — Check reports a nonblocking advisory that the parameter is inactive, and the wizard omits inactive drafts from newly generated code. The reference page of every multi-model component lists its variants and their requirements under **Model Variants**, and the Component Wizard shows and requires exactly the parameters the selected variant needs. The default `model$` is the engine's declared default, not whichever variant happens to be documented first.

Variants of your own components use the `VARIANT ... REQUIRE ... END` construct — see *Writing Your Own Component*.

[Related: comp-authoring, comp-library, comp-wizard]

[Topic: comp-authoring]
# Writing Your Own Component

When the library lacks a device — or you want your own correlation inside one — define a component in the document with `COMPONENT ... END`. The header parentheses declare the **ports** (in the order positional binding will use); `PARAM` lines declare parameters; everything else is acausal equations over port members, locals, and outputs.

```run
COMPONENT Heater(in, out)
  PARAM fluid$, Q
  out.mdot = in.mdot
  out.P    = in.P
  out.h    = in.h + Q / in.mdot
  T_out    = Temperature(fluid$, P=out.P, h=out.h)   { named output }
END

Source SUP(fluid$=Water, mdot=0.5 [kg/s], P=200000 [Pa], T=290 [K])
Heater H1(fluid$=Water, Q=50000 [W])
Sink   RET()
connect(SUP.out, H1.in)
connect(H1.out, RET.in)

T_supply = H1.T_out          { read the named output }
```

The rules:

- **Ports** carry whatever members your equations reference. Use `(P, mdot, h)` members and the port is a fluid port; use `(T, Qdot)` and it is a heat port — domain inference is automatic (see *Domains & Fluid Families*). A port referenced only as `port.sig` becomes a causal **signal** port: one writer, any readers — use one for every command input rather than pinning a component's internals from outside.
- **Parameters** — a trailing `$` marks a string parameter (`fluid$` is special: it names the stream's fluid for property calls and per-port fluid inference). `PARAM x = value` gives *your* component a default; the standard library deliberately never uses them.
- **Locals and outputs** — any bare name in the body is instance-private (auto-namespaced, like `MODULE` locals). Reading it from outside as `inst.name` makes it a named output.
- **Fluid family** — a component for a non-default family opts in with `PARAM domain$ = gas` (or `oil`, `moistair`, `liquid`, `twophase`), so the connector guard protects your lines too.
- **Composition** — a component body may instantiate other components and `connect` them: build a subsystem once, stamp it many times.
- **Time** — a body may reference the reserved global `time` (never namespaced) to build time-driven behavior; the `DYNAMIC` integrators pin it, and a steady document sets `time = 0` explicitly.
- **Keep closures C¹-smooth.** Newton differentiates everything, so a hard `if`/corner in a constitutive law stalls convergence. Use smooth surrogates — a `tanh` gate, the hinge `0.5*(x + sqrt(x^2 + eps^2))`, odd-symmetric flow laws — and expose the smoothing width as an `eps` parameter.

> **Contributing to the built-in library?** The end-to-end process (physics in the `.frees` domain files, golden-value fixture tests, generated reference pages) is documented in the repository at `docs/component_authoring.md`.

## Variants

Split fidelity levels with `VARIANT` blocks. Equations outside any variant are shared; each variant adds its own, and `REQUIRE` names the parameters it validates:

```
COMPONENT MyFan(in, out)
  PARAM fluid$, model$ = simple
  out.mdot = in.mdot                  { shared by every variant }
  VARIANT simple
    out.P = in.P + 250
  END
  VARIANT curve REQUIRE dP0, kQ
    out.P = in.P + dP0 - kQ * in.mdot^2
  END
END
```

`MyFan F1(fluid$=Air, model$=curve, dP0=300, kQ=1.5e4)` selects and validates the `curve` body.

[Related: comp-variants, comp-domains, functions]

[Topic: comp-transient]
# Steady ↔ Transient from One Network

A component network describes physics, not a moment in time — the **same wiring** yields a steady operating point or a transient, depending on whether you wrap it in a `DYNAMIC` block.

## Storage components carry the states

Time enters through **storage** components: `ThermalMass` (heat capacity), `MassGen` (self-heating mass), `Inertia` / `TransMass` (mechanical), `Capacitor` / `Inductor` (electrical), `Accumulator` (hydraulic), battery SOC, `BoilingVessel` (two-phase mass + energy). Each contributes a state derivative and an initial condition (its `T0`, `omega0`, `SOC0`, … parameter). Solved without a `DYNAMIC` block, the network settles to its steady operating point; add one and the states integrate in time:

```
{ A 4 kW battery module on a cold plate, warming its thermal mass }
MassGen     BATT(C=60000, Qgen=4000, T0=305 [K])
LiquidWallHX PLATE(fluid$=EG50, UA=800)
connect(PLATE.wall, BATT.port)
{ ... coolant loop around PLATE ... }

DYNAMIC warmup (method = ida, time = 0 .. 600, points = 601)
END
```

An **empty** `DYNAMIC` body is enough — the storage components inject their `der(...)` equations and initial conditions automatically. The block produces an ODE Table (Tables tab) with every state and stream member as columns you can plot, and the trajectory accessors (`FinalValue('...')`, `MaxValue('...')`, `TimeAt('...', v)`) read results back into the analytic solve.

## Scheduling inputs over time

The idiomatic way to drive a transient is a **signal source**: `SigStep`, `SigRamp`, `SigSine`, `SigPulse`, or a `SigTable` drive cycle, wired to an actuator's command port (see *Domains & Fluid Families*). Component bodies may reference the reserved global `time`, which the integrators pin to the running clock — that is what makes those sources tick (in a steady document, pin it yourself with `time = 0`).

Raw equations inside the `DYNAMIC` body may also reference `time` directly — still handy for one-off ramps:

```
DYNAMIC pulldown (method = ida, time = 0 .. 600, points = 1201)
  CHLR.frac = 0.05 + 0.95 * min(time/5, 1)   { capacity ramp over 5 s }
END
```

Starting a ramp from a small floor (here 5%) rather than zero keeps the first step well-conditioned — see *Troubleshooting Networks*.

## Choosing the integrator

Component transients are usually **DAEs** — differential states coupled to a large algebraic network. Use `method = ida` (SUNDIALS implicit DAE integrator) for those; it is the default choice for anything with fluid loops. A pure-ODE network (thermal RC chains, mechanical trains) also runs on the stiff `ode23s` / `ode15s` methods described in *Transient / ODE Systems (DYNAMIC)*.

[Related: dynamic-ode, comp-linearize, comp-troubleshooting]

[Topic: comp-linearize]
# From Plant to Controller (LINEARIZE)

A transient network is a **plant**; the control suite wants it as state-space matrices. The `LINEARIZE` block numerically linearizes a named `DYNAMIC` block about its operating point and hands you `(A, B, C, D)`:

```
LINEARIZE plant (block = warmup, a = A, b = B, c = C, d = D)
  INPUT  Q_load
  OUTPUT BATT.port.T
END
```

- **States** are the `DYNAMIC` block's `der()` variables (the storage components' states).
- **INPUT** names the exogenous inputs to perturb; **OUTPUT** the observed quantities — both accept dotted member accessors like `BATT.port.T`.
- The matrix names in the header default to `A`, `B`, `C`, `D`.

The result is an ordinary set of matrices, so the whole control toolbox applies directly:

```
CALL ss2tf(A, B, C, D : num, den)          { transfer function of the plant }
CALL bode(num, den, omega : mag, phase)    { frequency response }
CALL lqr(A, B, Q_w, R_w : K)               { optimal state feedback }
```

Close the loop back in the time domain with controller components (`PIThermostat` and friends) inside the same `DYNAMIC` network — design in the frequency domain, verify in the transient, all in one document.

[Related: comp-transient, symbolic-cas, plot-code]

[Topic: comp-cycle-plots]
# Cycle Plots & Diagnostics

## Source-mapped diagnostics

Expansion never leaks into your error messages. Diagnostics and residual reports name **components, ports, and streams** (`CMP.out.P`, stream `s2`) — never the internal flattened variables — so a convergence failure points at a device you recognize, and the *Debugging a Solve* workflow (F9 block-solve, residual reading, guess seeding) applies unchanged.

## Cycle overlays on property charts

Stream members are first-class citizens of the plotting system. A `PLOT` block of kind `property` recognizes component stream states, so a refrigeration loop drawn through `s1 … s4` overlays as a cycle path on a P-h or T-s chart:

```
PLOT 'Cycle'
  kind = property
  fluid = R134a
  diagram = 'P-h'
  overlaystates = true
  connectstates = true
END
```

See *Plots in Code (PLOT)* for the full attribute set, and *Fluid State Tables* for the STATE TABLE route to the same overlay.

[Related: plot-code, state-tables]

[Topic: comp-wizard]
# The Component Wizard

The **Component Wizard** builds an instantiation line for you — useful while you are still learning a component's parameter surface, and for the map-driven components whose setup is more than one line.

Open it from the editor toolbar, pick a component, and the wizard presents:

- **Every parameter with its meaning and unit**, validated as you type — string parameters (`fluid$`) offer the known fluid lists.
- **Variant gating** — selecting a `model$` variant shows (and requires) exactly the parameters that variant `REQUIRE`s, so you cannot assemble an invalid combination (see *Fidelity Variants*).
- **UA from correlations** — for heat-exchanger components, a helper computes the conductance from geometry and film-coefficient correlations instead of a guessed number, and writes the supporting equations for you.
- **Map ingestion** — for map-based machines (`CompressorMap`, `FanMap`, `PumpMap`), paste or import tabulated curve data and the wizard emits the backing `TABLE` block wired to the component's map parameter.

The output is plain frees text inserted at the cursor — the wizard is a typing aid, not a separate model format; everything it writes you could have written by hand.

[Related: comp-variants, comp-library, tables-code]

[Topic: comp-troubleshooting]
# Troubleshooting Networks

Everything in *Debugging a Solve* applies to component networks. This page adds the failure modes specific to them.

## Errors at parse time (by design)

frees rejects a malformed network **loudly, before solving** — a hard error beats a silently wrong answer:

- **Port count mismatch** — a shared-name instantiation must bind *all* ports or *none* (none = wire with `connect`). `Component 'LINE' binds 1 port(s) but COMPONENT Pipe declares 2` means a stream is missing.
- **Mixed domains at a node** — connecting, say, a heat port to a fluid port. Cross domains through a transducer component, never a wire (*Domains & Fluid Families*).
- **Mismatched fluid families** — a `gas` line wired to an `oil` or `moistair` line. Check the components' `domain$` tags.
- **Three ports on one shared stream** — the shared-name form is strictly point-to-point; use a `connect` node for branches.
- **Missing parameters** — library components have no defaults; every parameter (and every `REQUIRE` of the selected `model$` variant) must be supplied.

## Convergence: cold-start patterns

A coupled cycle (a refrigeration loop, a pump network) can be structurally perfect and still diverge from a cold start. Three patterns fix most of it:

1. **Seed the pressure level explicitly.** Give every closed loop one component that *pins* pressure — a `PressureSource`-style feed or a pinned port member (`PUMPOUT.in.P = 200000 [Pa]`). A loop with only relative pressure drops has a floating level the solver must guess.
2. **Don't re-equate mixer pressures.** A mixer's node already equalizes the joining pressures; adding your own `MIX.in1.P = MIX.in2.P` duplicates an equation and makes the Jacobian singular.
3. **Floor the capacity, then ramp.** Starting a compressor or valve at exactly zero flow puts property calls at degenerate states. Hold a small floor (`frac = 0.05`) for the steady solve, or ramp from it in a transient (`frac = 0.05 + 0.95 * min(time/5, 1)`).

## Working method

Build the network **one leg at a time**: source → component → sink, solve, extend. Select a subsystem and press **F9** to solve only it. Diagnostics are source-mapped (component and stream names), so the failing block names the device to look at. Set guesses on stream members (they appear under their display names, e.g. `s2.P`) in Variable Info exactly as for scalar variables. And inside the vapor dome, remember the two-phase rule: identify a state by quality `x` with `T` *or* `P`, never both.

[Related: debugging, comp-connections, comp-transient]
