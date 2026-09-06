---
name: SensibleAirToAirHX
category: Component (moistair)
summary: Acausal moistair-domain component SensibleAirToAirHX with ports sup_in, sup_out, exh_in, exh_out.
related: []
examples: []
tags: [sensibleairtoairhx, component, moistair, acausal]
references: []
generated: true
---

# SensibleAirToAirHX

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
SensibleAirToAirHX inst(eff, eatr, oacf, domain$)
```

## Ports

`sup_in`, `sup_out`, `exh_in`, `exh_out`

## Parameters

| Parameter | Type |
| --- | --- |
| `eff` | Number |
| `eatr` | Number |
| `oacf` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
sup_out.mdot = sup_in.mdot / oacf
exh_out.mdot = exh_in.mdot + (sup_in.mdot - sup_out.mdot)
sup_out.P    = sup_in.P
exh_out.P    = exh_in.P
T_s_in  = Temperature(AirH2O, h=sup_in.h, P=sup_in.P, W=sup_in.W)
T_e_in  = Temperature(AirH2O, h=exh_in.h, P=exh_in.P, W=exh_in.W)
C_s     = sup_in.mdot * Cp(AirH2O, T=T_s_in, P=sup_in.P, W=sup_in.W)
C_e     = exh_in.mdot * Cp(AirH2O, T=T_e_in, P=exh_in.P, W=exh_in.W)
Q       = eff * min(C_s, C_e) * (T_e_in - T_s_in)
T_s_out = T_s_in + Q / C_s
sup_out.W = sup_in.W + eatr * (exh_in.W - sup_in.W)
sup_out.h = Enthalpy(AirH2O, T=T_s_out, P=sup_in.P, W=sup_out.W)
exh_out.mdot * exh_out.W = exh_in.mdot * exh_in.W + sup_in.mdot * sup_in.W - sup_out.mdot * sup_out.W
exh_out.mdot * exh_out.h = exh_in.mdot * exh_in.h + sup_in.mdot * sup_in.h - sup_out.mdot * sup_out.h
T_e_out = Temperature(AirH2O, h=exh_out.h, P=exh_in.P, W=exh_out.W)
```

