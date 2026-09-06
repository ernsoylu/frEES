---
name: FanMap
category: Component (fluid)
summary: A fan whose pressure rise comes from a tabulated performance map (ΔP vs volumetric flow).
related: [Fan, FanCurve]
examples: []
tags: [fan, fan-family, fanmap, map, data:rho-map, ports:in-out, flow-closed, energy-pressure, steady, map-driven, component, fluid, acausal]
---

# FanMap

A fan whose pressure rise comes from a tabulated performance map (ΔP vs volumetric flow).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
FanMap inst(rho, map$)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `rho` | Number | Density [kg/m³]. |
| `map$` | String | Name of a TABLE/FUNCTION giving pressure rise [Pa] vs volumetric flow [m³/s]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

```
Q        = in.mdot / rho
dP       = map$(Q)
out.mdot = in.mdot
out.P    = in.P + dP
```
