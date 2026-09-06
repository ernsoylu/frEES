---
name: LiquidPumpMap
category: Component (liquid)
summary: Acausal liquid-domain component LiquidPumpMap with ports in, out.
related: [LiquidPump, PumpMap]
examples: []
tags: [pump, liquid-pump-family, liquidpumpmap, map, data:rho-map, ports:in-out, flow-closed, energy-work, steady, map-driven, liquid, component, acausal]
references: []
generated: true
---

# LiquidPumpMap

Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (`backend/core/src/main/resources/components/`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
LiquidPumpMap inst(rho, eta, map$, domain$)
```

## Ports

`in`, `out`

## Parameters

| Parameter | Type |
| --- | --- |
| `rho` | Number |
| `eta` | Number |
| `map$` | String |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
Q        = in.mdot / rho
head     = map$(Q)
out.P    = in.P + rho * 9.80665 * head
out.mdot = in.mdot
out.h    = in.h + 9.80665 * head / eta
W        = in.mdot * 9.80665 * head / eta
```
