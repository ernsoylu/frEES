---
name: IndirectDirectEvaporativeCooler
category: Component (moistair)
summary: Acausal moistair-domain component IndirectDirectEvaporativeCooler with ports pri_in, pri_out, sec_in, sec_out.
related: []
examples: []
tags: [indirectdirectevaporativecooler, component, moistair, acausal]
references: []
generated: true
---

# IndirectDirectEvaporativeCooler

Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (`crates/frees-core/src/components/library-data/`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

```
IndirectDirectEvaporativeCooler inst(wbde, eff_sec, eff_dir)
```

## Ports

`pri_in`, `pri_out`, `sec_in`, `sec_out`

## Parameters

| Parameter | Type |
| --- | --- |
| `wbde` | Number |
| `eff_sec` | Number |
| `eff_dir` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

```
IndirectEvaporativeCooler IEC(wbde=wbde, eff_sec=eff_sec)
EvaporativeCooler         DEC(eff=eff_dir)
connect(pri_in, IEC.pri_in)
connect(IEC.pri_out, DEC.in)
connect(DEC.out, pri_out)
connect(sec_in, IEC.sec_in)
connect(IEC.sec_out, sec_out)
```

