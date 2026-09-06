---
name: FanCurve
category: Component (fluid)
summary: A fan whose pressure rise follows a tabulated pressure–flow performance curve.
related: [Fan, FanMap]
examples: []
tags: [fan, fan-family, fancurve, curve, data:rho-dP0-Q0, ports:in-out, flow-closed, energy-pressure, steady, component, fluid, acausal]
---

# FanCurve

A fan whose pressure rise follows a tabulated pressure–flow performance curve.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h`; a node enforces equal `P` and `Σṁ = 0`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
FanCurve inst(rho, dP0, Q0)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `rho` | Number | Density [kg/m³]. |
| `dP0` | Number | Reference pressure drop [Pa]. |
| `Q0` | Number | Reference heat [W]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

```
Q        = in.mdot / rho
dP       = dP0 * (1 - (Q / Q0)^2)
out.mdot = in.mdot
out.P    = in.P + dP
```
