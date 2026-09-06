---
name: ApparatusDewPointCoil
category: Component (moistair)
summary: Acausal moistair-domain component ApparatusDewPointCoil with ports in, out.
related: []
examples: []
tags: [apparatusdewpointcoil, component, moistair, acausal]
references: []
generated: true
---

# ApparatusDewPointCoil

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
ApparatusDewPointCoil inst(T_adp, BF, domain$, model$)
```

## Ports

`in`, `out`

## Parameters

| Parameter | Type |
| --- | --- |
| `T_adp` | Number |
| `BF` | Number |
| `domain$` | String |
| `model$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
out.mdot   = in.mdot
out.P      = in.P
T_in       = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
T_out      = Temperature(AirH2O, h=out.h, P=in.P, W=out.W)
T_dp_in    = DewPoint(AirH2O, h=in.h, P=in.P, W=in.W)
margin_adp = T_dp_in - T_adp
mdot_w     = in.mdot * (in.W - out.W)
h_f        = 4186 * (T_out - 273.15)
Q_air      = in.mdot * (in.h - out.h)
Q          = Q_air - mdot_w * h_f
Q_sens     = in.mdot * Cp(AirH2O, T=T_in, P=in.P, W=in.W) * (T_in - T_out)
SHR        = Q_sens / Q_air
```

## Model Variants

Selected via the `model$` parameter; each adds its own equations (and `REQUIRE`d parameters):

### `wet`

```
W_adp = HumRat(AirH2O, T=T_adp, P=in.P, R=1)
h_adp = Enthalpy(AirH2O, T=T_adp, P=in.P, W=W_adp)
out.W = W_adp + BF * (in.W - W_adp)
out.h = h_adp + BF * (in.h - h_adp)
```

### `dry`

```
out.W = in.W
out.h = Enthalpy(AirH2O, T=T_adp + BF * (T_in - T_adp), P=in.P, W=in.W)
```

