---
name: DOAS
category: Component (moistair)
summary: Acausal moistair-domain component DOAS with ports oa_in, sup_out, exh_in, exh_out.
related: []
examples: []
tags: [doas, component, moistair, acausal]
references: []
generated: true
---

# DOAS

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
DOAS inst(eff_h, eff_w, T_adp, BF, Q_reheat)
```

## Ports

`oa_in`, `sup_out`, `exh_in`, `exh_out`

## Parameters

| Parameter | Type |
| --- | --- |
| `eff_h` | Number |
| `eff_w` | Number |
| `T_adp` | Number |
| `BF` | Number |
| `Q_reheat` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
EnthalpyWheel         ERV(eff_h=eff_h, eff_w=eff_w)
ApparatusDewPointCoil CC(T_adp=T_adp, BF=BF)
HeatingCoil           RH(Q=Q_reheat)
connect(oa_in, ERV.sup_in)
connect(ERV.sup_out, CC.in)
connect(CC.out, RH.in)
connect(RH.out, sup_out)
connect(exh_in, ERV.exh_in)
connect(ERV.exh_out, exh_out)
```

