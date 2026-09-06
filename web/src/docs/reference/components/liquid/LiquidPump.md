---
name: LiquidPump
category: Component (liquid)
summary: A single-phase liquid pump.
related: [LiquidPumpMap, Pump]
examples: [ev-thermal-management]
tags: [pump, liquid-pump-family, liquidpump, data:eta, ports:in-out, flow-closed, energy-work, steady, liquid, component, acausal]
---

# LiquidPump

A single-phase liquid pump.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
LiquidPump inst(eta, fluid$, domain$)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `eta` | Number | Efficiency (0–1). |
| `fluid$` | String | Fluid name (e.g. Water, R134a, Air). |
| `domain$` | String | Connector fluid family — one of `fluid`, `gas`, `oil`, `moistair`, `liquid`, `twophase`. |

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

[Run: ev-thermal-management]
