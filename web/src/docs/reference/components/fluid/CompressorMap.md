---
name: CompressorMap
category: Component (fluid)
summary: A compressor whose isentropic efficiency comes from a tabulated map (eta vs pressure ratio).
related: [Compressor, TwoPhaseCompressor]
examples: []
tags: [compressor, compressor-family, compressormap, map, data:map-eta, ports:in-out, flow-closed, energy-work, steady, map-driven, component, fluid, acausal]
---

# CompressorMap

A compressor whose isentropic efficiency comes from a tabulated map (eta vs pressure ratio).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
CompressorMap inst(fluid$, map_eta$, model$)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `fluid$` | String | Fluid name (e.g. R134a, Air). |
| `map_eta$` | String | Name of a TABLE/FUNCTION giving isentropic efficiency (0–1) vs pressure ratio (out.P/in.P). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

```
s_in     = Entropy(fluid$, P=in.P, h=in.h)
h_s      = Enthalpy(fluid$, P=out.P, s=s_in)
PR       = out.P / in.P
eta      = map_eta$(PR)
out.mdot = in.mdot
out.h    = in.h + (h_s - in.h) / eta
W        = in.mdot * (out.h - in.h)
```

## Model Variants

Selected via the `model$` parameter; each adds its own equations (and `REQUIRE`d parameters):

### `eta`

_No additional equations (uses the shared body; the through-flow is imposed by the surrounding network)._

### `flow` — requires `map_mdot$`

```
in.mdot = map_mdot$(PR)
```

The flow rung makes the machine a true flow-determining (R) element — the mass
flow comes from the pressure-ratio characteristic, so a supply → compressor →
volume chain is well-posed on every integrator.
