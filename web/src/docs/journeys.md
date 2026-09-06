[Topic: journey-scalar]
# Journey 1 — A scalar equation with units

**Goal.** Solve one textbook equation, read the SI result, and recover from a missing unit.

**Do this.** Type the block, press **F4** then **F2**. Expected: `m ≈ 0.293 kg`.

```run
{ Mass of air in a rigid tank }
P = 500 [kPa]
Vol = 0.05 [m^3]
T = 25 [C]
R = 0.287 [kJ/kg-K]
P * Vol = m * R * T
```

`T = 25 [C]` is an absolute temperature (298.15 K). A 10 °C *rise* is 10 K — see *Units & Dimensional Consistency*.

**Deliberate error.** Change `P = 500 [kPa]` to `P = 500`. Check still reports the system solvable; the unknown-unit / unconverted-value warning is the recovery, not a failed solve. Put `[kPa]` back and Solve — `m` returns to ~0.293 kg.

[Related: gs-first-solve, units, variables]

[Topic: journey-chain]
# Journey 2 — A component chain

**Goal.** Build source → pipe → sink with named parameters and `connect`, then recover a missing boundary.

**Do this.** Insert from the Component Wizard (or type). Expected: `dP` is a positive frictional drop, on the order of kilopascals for this pipe.

```run
Source SUP(fluid$='Water', mdot=2 [kg/s], P=300000 [Pa], T=298 [K])
Pipe   LINE(fluid$='Water', L=50 [m], D=0.05 [m], rough=0.0001)
Sink   RET()
connect(SUP.out, LINE.in)
connect(LINE.out, RET.in)
dP = SUP.out.P - RET.in.P
```

**Deliberate error.** Delete `mdot=2 [kg/s]` from the Source (or comment out the SUP.out–LINE.in `connect`). Check reports the free quantity; the Schematic highlights the open port. Restore the boundary — do not invent a pressure. See *Your First Component Network* and *Connections & Junctions*.

Positional streams (`Source SUP(s1, …)`) are an advanced shorthand for two-port chains; stay on `connect` until this journey is fluent.

[Related: comp-first-network, comp-connections, journey-loop]

[Topic: journey-upgrade]
# Journey 3 — Upgrade: map or transient

**Goal.** Replace a constant-rise fan with a map, or add a `DYNAMIC` block, without losing the wiring.

**Map path.** Start from a `Fan` (needs `dP0`, `Q0`, `eta`). In the wizard, open **Related models** and pick `FanMap` when you have a ΔP vs Q table. The ports stay `in, out`; only the required data change. Expected: `connect` lines do not need rewriting. A missing `map$` is a Check error naming the parameter — recover by inserting the TABLE (or the map builder), not by switching variants silently.

**Transient path.** Keep the algebraic network and add a first-order lag on a probe, or follow *Tutorial: Mass–Spring–Damper* for a full `DYNAMIC` document. Name the independent axis `time`, not `t`, so it does not collide with a temperature `T`.

[Related: tut-msd, journey-chain, variables]

[Topic: journey-loop]
# Mixers and closed loops

After the chain: two streams at different states that join need a **mixer** (`Mixer`, `LiquidMixer`, `MixingBox`). A three-port `connect` is a legal *branch* (equal pressure, mass conserved) but it does not mix enthalpy.

A **closed loop** is `connect` back through the load, not through a second inlet on the source:

```
connect(PUMP.out, HX.in)
connect(HX.out, PUMP.in)
```

Parameter tables live on each component's Reference page — this journey does not copy them.

[Related: journey-chain, comp-connections, comp-library]
