---
name: InductionUnit
category: Component (moistair)
summary: Acausal moistair-domain component InductionUnit with ports pri_in, ind_in, out, wall.
related: []
examples: []
tags: [inductionunit, component, moistair, acausal]
references: []
generated: true
---

# InductionUnit

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
InductionUnit inst(ratio, eps, domain$)
```

## Ports

`pri_in`, `ind_in`, `out`, `wall`

## Parameters

| Parameter | Type |
| --- | --- |
| `ratio` | Number |
| `eps` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
ind_in.mdot = ratio * pri_in.mdot
out.mdot    = pri_in.mdot + ind_in.mdot
out.P       = pri_in.P
T_i_in      = Temperature(AirH2O, h=ind_in.h, P=ind_in.P, W=ind_in.W)
T_i_out     = T_i_in - eps * (T_i_in - wall.T)
h_i_out     = Enthalpy(AirH2O, T=T_i_out, P=ind_in.P, W=ind_in.W)
out.mdot * out.W = pri_in.mdot * pri_in.W + ind_in.mdot * ind_in.W
out.mdot * out.h = pri_in.mdot * pri_in.h + ind_in.mdot * h_i_out
Q           = ind_in.mdot * (ind_in.h - h_i_out)
wall.Qdot   = -Q
T_dp_ind    = DewPoint(AirH2O, h=ind_in.h, P=ind_in.P, W=ind_in.W)
margin_dp   = wall.T - T_dp_ind
```

