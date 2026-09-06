---
name: ChilledBeam
category: Component (moistair)
summary: Acausal moistair-domain component ChilledBeam with ports in, out, wall.
related: []
examples: []
tags: [chilledbeam, component, moistair, acausal]
references: []
generated: true
---

# ChilledBeam

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
ChilledBeam inst(eps, domain$)
```

## Ports

`in`, `out`, `wall`

## Parameters

| Parameter | Type |
| --- | --- |
| `eps` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
out.mdot  = in.mdot
out.P     = in.P
out.W     = in.W
T_in      = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
T_out     = T_in - eps * (T_in - wall.T)
out.h     = Enthalpy(AirH2O, T=T_out, P=in.P, W=in.W)
Q         = in.mdot * (in.h - out.h)
wall.Qdot = -Q
T_dp_in   = DewPoint(AirH2O, h=in.h, P=in.P, W=in.W)
margin_dp = wall.T - T_dp_in
```

