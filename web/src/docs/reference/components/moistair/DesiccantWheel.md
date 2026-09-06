---
name: DesiccantWheel
category: Component (moistair)
summary: Acausal moistair-domain component DesiccantWheel with ports proc_in, proc_out, reg_in, reg_out.
related: []
examples: []
tags: [desiccantwheel, component, moistair, acausal]
references: []
generated: true
---

# DesiccantWheel

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
DesiccantWheel inst(eff_L, W_eq, f_carry, domain$)
```

## Ports

`proc_in`, `proc_out`, `reg_in`, `reg_out`

## Parameters

| Parameter | Type |
| --- | --- |
| `eff_L` | Number |
| `W_eq` | Number |
| `f_carry` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
proc_out.mdot = proc_in.mdot
reg_out.mdot  = reg_in.mdot
proc_out.P    = proc_in.P
reg_out.P     = reg_in.P
proc_out.W    = proc_in.W - eff_L * (proc_in.W - W_eq)
proc_out.h    = proc_in.h + f_carry * (reg_in.h - proc_in.h)
reg_out.W     = reg_in.W + (proc_in.mdot / reg_in.mdot) * (proc_in.W - proc_out.W)
reg_out.h     = reg_in.h - (proc_in.mdot / reg_in.mdot) * (proc_out.h - proc_in.h)
```

