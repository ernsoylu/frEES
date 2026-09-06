---
name: SteamHumidifier
category: Component (moistair)
summary: Acausal moistair-domain component SteamHumidifier with ports in, out.
related: []
examples: []
tags: [steamhumidifier, component, moistair, acausal]
references: []
generated: true
---

# SteamHumidifier

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
SteamHumidifier inst(W_set, h_steam, domain$)
```

## Ports

`in`, `out`

## Parameters

| Parameter | Type |
| --- | --- |
| `W_set` | Number |
| `h_steam` | Number |
| `domain$` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
out.mdot = in.mdot
out.P    = in.P
out.W    = W_set
mdot_w   = in.mdot * (W_set - in.W)
out.h    = in.h + mdot_w * h_steam / in.mdot
```

