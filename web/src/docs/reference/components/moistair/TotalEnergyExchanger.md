---
name: TotalEnergyExchanger
category: Component (moistair)
summary: Acausal moistair-domain component TotalEnergyExchanger with ports sup_in, sup_out, exh_in, exh_out.
related: []
examples: []
tags: [totalenergyexchanger, component, moistair, acausal]
references: []
generated: true
---

# TotalEnergyExchanger

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
TotalEnergyExchanger inst(eps_s, eps_L, eatr, oacf, domain$)
```

## Ports

`sup_in`, `sup_out`, `exh_in`, `exh_out`

## Parameters

| Parameter | Type |
| --- | --- |
| `eps_s` | Number |
| `eps_L` | Number |
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
W_x     = sup_in.W + eps_L * (exh_in.W - sup_in.W)
T_s_out = T_s_in + eps_s * (T_e_in - T_s_in)
sup_out.W = W_x + eatr * (exh_in.W - W_x)
sup_out.h = Enthalpy(AirH2O, T=T_s_out, P=sup_in.P, W=sup_out.W)
exh_out.mdot * exh_out.W = exh_in.mdot * exh_in.W + sup_in.mdot * sup_in.W - sup_out.mdot * sup_out.W
exh_out.mdot * exh_out.h = exh_in.mdot * exh_in.h + sup_in.mdot * sup_in.h - sup_out.mdot * sup_out.h
```

