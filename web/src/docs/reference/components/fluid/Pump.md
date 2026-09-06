---
name: Pump
category: Component (fluid)
summary: Raises the pressure of a liquid stream, computing the work from a pump efficiency.
related: [PumpMap, LiquidPump]
examples: [pump-sizing, rankine-cycle]
tags: [pump, pump-family, data:eta, ports:in-out, flow-closed, energy-work, steady, component, fluid, acausal]
---

# Pump

Raises the pressure of a liquid stream, computing the work from a pump efficiency.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h`; a node enforces equal `P` and `Σṁ = 0`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
Pump inst(eta, fluid$)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `eta` | Number | Efficiency (0–1). |
| `fluid$` | String | Fluid name (e.g. Water, R134a, Air). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

```
v        = Volume(fluid$, P=in.P, h=in.h)
out.mdot = in.mdot
out.h    = in.h + v * (out.P - in.P) / eta
W        = in.mdot * (out.h - in.h)
```

## Examples

Instantiated in the verified example below:

[Run: pump-sizing]
