---
name: Fan
category: Component (fluid)
summary: Adds a pressure rise to a gas/air stream, computing the fan work.
related: [FanCurve, FanMap]
examples: []
tags: [fan, fan-family, constant-rise, data:dP0-Q0-eta, ports:in-out, flow-closed, energy-work, steady, component, fluid, acausal]
---

# Fan

Adds a pressure rise to a gas/air stream, computing the fan work.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h`; a node enforces equal `P` and `Σṁ = 0`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
Fan inst(fluid$, dP0, Q0, eta)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `fluid$` | String | Fluid name (e.g. Water, R134a, Air). |
| `dP0` | Number | Reference pressure drop [Pa]. |
| `Q0` | Number | Reference heat [W]. |
| `eta` | Number | Efficiency (0–1). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

```
rho      = Density(fluid$, P=in.P, h=in.h)
Q        = in.mdot / rho
dP       = dP0 * (1 - (Q / Q0)^2)
out.mdot = in.mdot
out.P    = in.P + dP
out.h    = in.h + dP / (rho * eta)
```
