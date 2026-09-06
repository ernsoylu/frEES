---
name: LiquidDesiccantContactor
category: Component (moistair)
summary: Acausal moistair-domain component LiquidDesiccantContactor with ports in, out, wall.
related: []
examples: []
tags: [liquiddesiccantcontactor, component, moistair, acausal]
references: []
generated: true
---

# LiquidDesiccantContactor

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
LiquidDesiccantContactor inst(eff_L, W_eq, eps_T, f_excess, domain$, model$)
```

## Ports

`in`, `out`, `wall`

## Parameters

| Parameter | Type |
| --- | --- |
| `eff_L` | Number |
| `W_eq` | Number |
| `eps_T` | Number |
| `f_excess` | Number |
| `domain$` | String |
| `model$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
out.mdot = in.mdot
out.P    = in.P
out.W    = in.W - eff_L * (in.W - W_eq)
T_in     = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
mdot_w   = in.mdot * (in.W - out.W)
```

## Model Variants

Selected via the `model$` parameter; each adds its own equations (and `REQUIRE`d parameters):

### `cooled` — requires `eps_T`

```
T_out     = T_in - eps_T * (T_in - wall.T)
out.h     = Enthalpy(AirH2O, T=T_out, P=in.P, W=out.W)
h_f       = 4186 * (wall.T - 273.15)
Q         = in.mdot * (in.h - out.h) - mdot_w * h_f
wall.Qdot = -Q
```

### `adiabatic` — requires `f_excess`

```
h_f       = 4186 * (T_in - 273.15)
h_pure    = in.h - mdot_w * h_f / in.mdot
T_pure    = Temperature(AirH2O, h=h_pure, P=in.P, W=out.W)
T_out     = T_in + (1 + f_excess) * (T_pure - T_in)
out.h     = Enthalpy(AirH2O, T=T_out, P=in.P, W=out.W)
Q         = 0
wall.Qdot = 0
```

