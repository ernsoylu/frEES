---
name: FanCoilUnit
category: Component (moistair)
summary: Acausal moistair-domain component FanCoilUnit with ports in, out, wall.
related: []
examples: []
tags: [fancoilunit, component, moistair, acausal]
references: []
generated: true
---

# FanCoilUnit

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
FanCoilUnit inst(K, foul, dP, eta, eps)
```

## Ports

`in`, `out`, `wall`

## Parameters

| Parameter | Type |
| --- | --- |
| `K` | Number |
| `foul` | Number |
| `dP` | Number |
| `eta` | Number |
| `eps` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
AirFilter      FL(K=K, foul=foul)
MoistAirFan    FN(dP=dP, eta=eta)
MoistAirWallHX CO(model$=eps_t, eps=eps)
connect(in, FL.in)
connect(FL.out, FN.in)
connect(FN.out, CO.in)
connect(CO.out, out)
connect(wall, CO.wall)
```

