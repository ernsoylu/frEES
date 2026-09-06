---
name: Economizer
category: Component (moistair)
summary: Acausal moistair-domain component Economizer with ports oa_in, ret_in, mix_out.
related: []
examples: []
tags: [economizer, component, moistair, acausal]
references: []
generated: true
---

# Economizer

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
Economizer inst(mdot_sup, f_min, lim, band, domain$, model$)
```

## Ports

`oa_in`, `ret_in`, `mix_out`

## Parameters

| Parameter | Type |
| --- | --- |
| `mdot_sup` | Number |
| `f_min` | Number |
| `lim` | Number |
| `band` | Number |
| `domain$` | String |
| `model$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
mix_out.P    = oa_in.P
mix_out.mdot = mdot_sup
oa_in.mdot   = f_oa * mdot_sup
ret_in.mdot  = (1 - f_oa) * mdot_sup
f_oa         = f_min + (1 - f_min) * g_diff * g_lim
mix_out.mdot * mix_out.W = oa_in.mdot * oa_in.W + ret_in.mdot * ret_in.W
mix_out.mdot * mix_out.h = oa_in.mdot * oa_in.h + ret_in.mdot * ret_in.h
```

## Model Variants

Selected via the `model$` parameter; each adds its own equations (and `REQUIRE`d parameters):

### `drybulb`

```
T_oa   = Temperature(AirH2O, h=oa_in.h, P=oa_in.P, W=oa_in.W)
T_ret  = Temperature(AirH2O, h=ret_in.h, P=ret_in.P, W=ret_in.W)
g_diff = 0.5 * (1 + tanh((T_ret - T_oa) / band))
g_lim  = 0.5 * (1 + tanh((lim - T_oa) / band))
```

### `enthalpy`

```
g_diff = 0.5 * (1 + tanh((ret_in.h - oa_in.h) / band))
g_lim  = 0.5 * (1 + tanh((lim - oa_in.h) / band))
```

