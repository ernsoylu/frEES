---
name: FanPoweredBox
category: Component (moistair)
summary: Acausal moistair-domain component FanPoweredBox with ports pri_in, ind_in, out.
related: []
examples: []
tags: [fanpoweredbox, component, moistair, acausal]
references: []
generated: true
---

# FanPoweredBox

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
FanPoweredBox inst(Q_fan, Q_reheat, domain$)
```

## Ports

`pri_in`, `ind_in`, `out`

## Parameters

| Parameter | Type |
| --- | --- |
| `Q_fan` | Number |
| `Q_reheat` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
out.P    = pri_in.P
out.mdot = pri_in.mdot + ind_in.mdot
out.mdot * out.W = pri_in.mdot * pri_in.W + ind_in.mdot * ind_in.W
out.mdot * out.h = pri_in.mdot * pri_in.h + ind_in.mdot * ind_in.h + Q_fan + Q_reheat
```

