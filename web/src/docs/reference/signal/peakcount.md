---
name: peakcount
category: Stats
summary: Number of strict local maxima in a series, with height and separation thresholds.
related: [peakindex, Smooth, XCorr, Welch]
examples: []
tags: [peak, peaks, local maximum, detection, count, signal]
---

# peakcount

Counts the peaks in a measured series — cycles in a pressure trace, teeth on an encoder, bursts in a vibration record.

## Syntax

```
n = peakcount(minheight, mindistance, x1, x2, ...)
n = peakcount(minheight, mindistance, [ ... ])
```

## Description

A peak is a sample **strictly greater than both of its neighbours**. The first and last samples are never peaks, having only one neighbour each, and a plateau of equal values is not a peak either.

The two scalars come first, matching the convention `ci_mean_lo(0.95, […])` set:

* `minheight` rejects peaks at or below it. Set it below the series minimum to disable.
* `mindistance` enforces a minimum index separation. `0` or `1` imposes none; anything larger keeps the **tallest** peak of each contested group and discards the rest, so a broad noisy shoulder does not out-vote the crest beside it.

Smooth first if the record is noisy: every noise excursion of two samples is a peak by this definition, and **Smooth** is usually the difference between counting cycles and counting samples.

## Mathematical Formulation

$$ P = \{\, j : 0 < j < n-1,\ x_j > x_{j-1},\ x_j > x_{j+1},\ x_j > h \,\} $$

then greedily thinned by descending $x_j$ under the separation constraint.

> **Method:** one pass for the candidates, then a tallest-first sweep. Ties break to the lower index, so the answer is deterministic.

## Examples

### Example 1 — counting cycles

```
x = [1, 3, 2, 6, 4, 9, 5, 12]
n = peakcount(0, 0, x)
```

**Expected:** `n = 3` — the 3, the 6 and the 9. The final 12 is an endpoint, not a peak.

### Example 2 — ignoring small excursions

```
x = [1, 3, 2, 6, 4, 9, 5, 12]
tall = peakcount(5, 0, x)
```

**Expected:** `tall = 2` — only the 6 and the 9 clear a height floor of 5.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `minheight` | Number | Yes | Peaks at or below this are rejected. |
| `mindistance` | Number | Yes | Minimum index separation; `0` or `1` for none. |
| `x1, x2, …` | Number | Yes | The series, as loose values or a list literal. |
