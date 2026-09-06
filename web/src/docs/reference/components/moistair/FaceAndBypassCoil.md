---
name: FaceAndBypassCoil
category: Component (moistair)
summary: Acausal moistair-domain component FaceAndBypassCoil with ports in, out, wall.
related: []
examples: []
tags: [faceandbypasscoil, component, moistair, acausal]
references: []
generated: true
---

# FaceAndBypassCoil

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
FaceAndBypassCoil inst(u_face, eps, domain$)
```

## Ports

`in`, `out`, `wall`

## Parameters

| Parameter | Type |
| --- | --- |
| `u_face` | Number |
| `eps` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
out.mdot  = in.mdot
out.P     = in.P
T_in      = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
T_face    = T_in - eps * (T_in - wall.T)
W_sat     = HumRat(AirH2O, T=T_face, P=in.P, R=1)
W_face    = 0.5 * (in.W + W_sat - sqrt((in.W - W_sat)^2 + 1e-12))
h_face    = Enthalpy(AirH2O, T=T_face, P=in.P, W=W_face)
out.W     = u_face * W_face + (1 - u_face) * in.W
out.h     = u_face * h_face + (1 - u_face) * in.h
BF        = 1 - u_face
Q         = in.mdot * (in.h - out.h)
wall.Qdot = -Q
```

