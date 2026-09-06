---
name: IndirectEvaporativeCooler
category: Component (moistair)
summary: Acausal moistair-domain component IndirectEvaporativeCooler with ports pri_in, pri_out, sec_in, sec_out.
related: []
examples: []
tags: [indirectevaporativecooler, component, moistair, acausal]
references: []
generated: true
---

# IndirectEvaporativeCooler

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
IndirectEvaporativeCooler inst(wbde, eff_sec, domain$)
```

## Ports

`pri_in`, `pri_out`, `sec_in`, `sec_out`

## Parameters

| Parameter | Type |
| --- | --- |
| `wbde` | Number |
| `eff_sec` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
pri_out.mdot = pri_in.mdot
sec_out.mdot = sec_in.mdot
pri_out.P    = pri_in.P
sec_out.P    = sec_in.P
pri_out.W    = pri_in.W
T_p_in   = Temperature(AirH2O, h=pri_in.h, P=pri_in.P, W=pri_in.W)
T_wb_sec = WetBulb(AirH2O, h=sec_in.h, P=sec_in.P, W=sec_in.W)
T_p_out  = T_p_in - wbde * (T_p_in - T_wb_sec)
pri_out.h = Enthalpy(AirH2O, T=T_p_out, P=pri_in.P, W=pri_out.W)
Q         = pri_in.mdot * (pri_in.h - pri_out.h)
W_sat_sec = HumRat(AirH2O, h=sec_in.h, P=sec_in.P, R=1)
sec_out.W = sec_in.W + eff_sec * (W_sat_sec - sec_in.W)
sec_out.h = sec_in.h + Q / sec_in.mdot
```

