---
name: Compressor
category: Component (fluid)
summary: Raises the pressure of a fluid stream, computing the work from an isentropic efficiency.
related: [CompressorMap, TwoPhaseCompressor]
examples: [ev-thermal-management]
tags: [compressor, compressor-family, data:eta, ports:in-out, flow-closed, energy-work, steady, component, fluid, acausal]
---

# Compressor

Raises the pressure of a fluid stream, computing the work from an isentropic efficiency.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h`; a node enforces equal `P` and `Σṁ = 0`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
Compressor inst(eta, fluid$, model$)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `eta` | Number | Efficiency (0–1). |
| `fluid$` | String | Fluid name (e.g. Water, R134a, Air). |
| `model$` | String | Model variant — selects the physics body (see Model Variants). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

```
s_in     = Entropy(fluid$, P=in.P, h=in.h)
h_s      = Enthalpy(fluid$, P=out.P, s=s_in)
out.mdot = in.mdot
out.h    = in.h + (h_s - in.h) / eta
W        = in.mdot * (out.h - in.h)
```

## Model Variants

Selected via the `model$` parameter; each adds its own equations (and `REQUIRE`d parameters):

### `isentropic`

_No additional equations (uses the shared body)._

### `volumetric` — requires `eta_v`, `disp`, `rpm`

```
rho_in  = Density(fluid$, P=in.P, h=in.h)
in.mdot = eta_v * disp * (rpm / 60) * rho_in
```

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]
