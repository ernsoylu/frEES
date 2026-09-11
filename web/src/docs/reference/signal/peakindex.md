---
name: peakindex
category: Stats
summary: 1-based position of the k-th peak in a series, or 0 when there is none.
related: [peakcount, Smooth, XCorr]
examples: []
tags: [peak, peaks, local maximum, detection, index, position, signal]
---

# peakindex

Locates a peak found by **peakcount**, so its value, its time or the interval to the next one can be read out of the series.

## Syntax

```
p = peakindex(k, minheight, mindistance, x1, x2, ...)
p = peakindex(k, minheight, mindistance, [ ... ])
```

## Description

Returns the 1-based position, within the series, of the `k`-th peak in ascending index order. Peaks are defined exactly as in **peakcount** — strictly greater than both neighbours, subject to the same `minheight` floor and `mindistance` separation — so the same two thresholds must be passed to both to get a consistent answer.

`k` is 1-based too: `peakindex(1, …)` is the first peak. When there is no `k`-th peak the answer is **0**, not an error and not the last peak, so a loop can walk peaks until it gets a zero.

## Examples

### Example 1 — walking the peaks of a trace

```
x  = [1, 3, 2, 6, 4, 9, 5, 12]
n  = peakcount(0, 0, x)
p1 = peakindex(1, 0, 0, x)
p2 = peakindex(2, 0, 0, x)
p3 = peakindex(3, 0, 0, x)
p4 = peakindex(4, 0, 0, x)
```

**Expected:** `n = 3`, `p1 = 2`, `p2 = 4`, `p3 = 6`, and `p4 = 0` — there is no fourth peak.

### Example 2 — the interval between the first two peaks

```
{ x sampled at 100 Hz }
dt = (peakindex(2, 0, 3, x) - peakindex(1, 0, 3, x)) / 100
```

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `k` | Integer | Yes | Which peak, 1-based. Must be a positive whole number. |
| `minheight` | Number | Yes | Peaks at or below this are rejected. |
| `mindistance` | Number | Yes | Minimum index separation; `0` or `1` for none. |
| `x1, x2, …` | Number | Yes | The series, as loose values or a list literal. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `the ordinal must be a positive whole number` | `k` was 0, negative or fractional. | Peaks are numbered from 1. |
