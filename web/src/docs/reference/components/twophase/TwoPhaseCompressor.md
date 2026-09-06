---
name: TwoPhaseCompressor
category: Component (twophase)
summary: A refrigerant compressor with selectable isentropic/volumetric variants.
related: [Compressor, CompressorMap]
examples: [ev-thermal-management]
tags: [compressor, compressor-family, twophasecompressor, ports:in-out, flow-closed, energy-work, steady, twophase, component, acausal]
---

# TwoPhaseCompressor

A refrigerant compressor with selectable isentropic/volumetric variants.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure `P`, mass-flow `ṁ`, and specific enthalpy `h` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

`in`, `out`

## Usage

```
TwoPhaseCompressor inst(fluid$, eta, domain$, model$)
```

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| `fluid$` | String | Fluid name (e.g. Water, R134a, Air). |
| `eta` | Number | Efficiency (0–1). |
| `domain$` | String | Connector fluid family — one of `fluid`, `gas`, `oil`, `moistair`, `liquid`, `twophase`. |
| `model$` | String | Model variant — selects the physics body (see Model Variants). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

```
s_in     = Entropy(fluid$, P=in.P, h=in.h)
h_s      = Enthalpy(fluid$, P=out.P, s=s_in)
out.mdot = in.mdot
out.h    = in.h + (h_s - in.h) / eta
W        = in.mdot * (out.h - in.h)
```

## Model Variants

Selected via the `model$` parameter; each adds its own equations (and `REQUIRE`d parameters):

### `isentropic`

_No additional equations (uses the shared body)._

### `volumetric` — requires `eta_v`, `disp`, `rpm`

```
rho_in  = Density(fluid$, P=in.P, h=in.h)
in.mdot = eta_v * disp * (rpm / 60) * rho_in
```

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]
