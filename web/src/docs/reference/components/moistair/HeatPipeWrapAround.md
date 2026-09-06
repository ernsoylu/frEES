---
name: HeatPipeWrapAround
category: Component (moistair)
summary: Acausal moistair-domain component HeatPipeWrapAround with ports pre_in, pre_out, re_in, re_out.
related: []
examples: []
tags: [heatpipewraparound, component, moistair, acausal]
references: []
generated: true
---

# HeatPipeWrapAround

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
HeatPipeWrapAround inst(eff, domain$)
```

## Ports

`pre_in`, `pre_out`, `re_in`, `re_out`

## Parameters

| Parameter | Type |
| --- | --- |
| `eff` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
pre_out.mdot = pre_in.mdot
re_out.mdot  = re_in.mdot
pre_out.P    = pre_in.P
re_out.P     = re_in.P
pre_out.W    = pre_in.W
re_out.W     = re_in.W
T_p_in    = Temperature(AirH2O, h=pre_in.h, P=pre_in.P, W=pre_in.W)
T_r_in    = Temperature(AirH2O, h=re_in.h, P=re_in.P, W=re_in.W)
T_p_out   = T_p_in - eff * (T_p_in - T_r_in)
pre_out.h = Enthalpy(AirH2O, T=T_p_out, P=pre_in.P, W=pre_out.W)
Q         = pre_in.mdot * (pre_in.h - pre_out.h)
re_out.h  = re_in.h + Q / re_in.mdot
T_r_out   = Temperature(AirH2O, h=re_out.h, P=re_in.P, W=re_out.W)
```

