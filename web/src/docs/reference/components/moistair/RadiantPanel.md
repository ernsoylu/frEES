---
name: RadiantPanel
category: Component (moistair)
summary: Acausal moistair-domain component RadiantPanel with ports zone, wall.
related: []
examples: []
tags: [radiantpanel, component, moistair, acausal]
references: []
generated: true
---

# RadiantPanel

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
RadiantPanel inst(A, C, n, eps_dT, W_room, P_room)
```

## Ports

`zone`, `wall`

## Parameters

| Parameter | Type |
| --- | --- |
| `A` | Number |
| `C` | Number |
| `n` | Number |
| `eps_dT` | Number |
| `W_room` | Number |
| `P_room` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
dT        = zone.T - wall.T
q_flux    = C * dT * (dT^2 + eps_dT^2)^((n - 1) / 2)
Q         = A * q_flux
zone.Qdot = Q
wall.Qdot = -Q
T_dp_room = DewPoint(AirH2O, T=zone.T, P=P_room, W=W_room)
margin_dp = wall.T - T_dp_room
```

