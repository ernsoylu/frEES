---
name: PumpMap
category: Component (fluid)
summary: A pump whose head comes from a tabulated performance map (head vs volumetric flow).
related: [Pump, LiquidPumpMap]
examples: []
tags: [pump, pump-family, pumpmap, map, data:rho-map, ports:in-out, flow-closed, energy-pressure, steady, map-driven, component, fluid, acausal]
---

# PumpMap

A pump whose head comes from a tabulated performance map (head vs volumetric flow).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
PumpMap inst(rho, map$)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `rho` | Number | Density [kg/m³]. |
| `map$` | String | Name of a TABLE/FUNCTION giving head [m] vs volumetric flow [m³/s]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

```
Q        = in.mdot / rho
head     = map$(Q)
out.mdot = in.mdot
out.P    = in.P + rho * 9.80665 * head
```
