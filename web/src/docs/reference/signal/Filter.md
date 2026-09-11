---
name: Filter
category: Stats
summary: Causal IIR/FIR filtering by the difference equation, starting from rest.
related: [FiltFilt, Smooth, Window, Convolve]
examples: []
tags: [filter, iir, fir, lowpass, difference equation, signal]
---

# Filter

Runs a signal through a digital filter defined by its numerator and denominator coefficients — the same `(b, a)` pair MATLAB's `filter` and SciPy's `lfilter` take.

## Syntax

```
CALL Filter(b, a, x : y)
```

## Description

Implements

```
a[1]·y[j] = b[1]·x[j] + b[2]·x[j-1] + … − a[2]·y[j-1] − a[3]·y[j-2] − …
```

as a transposed direct-form II, so only `max(len(a), len(b)) − 1` state values are carried regardless of order. Pass `a = [1]` for a pure FIR filter.

Initial conditions are **zero**: the first samples carry the filter's start-up transient, and the output is delayed relative to the input by the filter's group delay. Both are properties of causal filtering, not defects — when neither is acceptable, use **FiltFilt**, which cancels the delay and suppresses the transient.

## Mathematical Formulation

$$ y_j = \frac{1}{a_1}\left( \sum_{i=0}^{n_b-1} b_{i+1}\,x_{j-i} \;-\; \sum_{i=1}^{n_a-1} a_{i+1}\,y_{j-i} \right) $$

> **Method:** transposed direct-form II, coefficients normalized by `a[1]` once up front.

## Examples

### Example 1 — a two-tap moving average as an FIR filter

```
x = [1, 3, 2, 6, 4, 9, 5, 12]
b = [0.5, 0.5]
a = [1]
CALL Filter(b, a, x : y)
```

**Expected:** `y[1] = 0.5` (the start-up transient — there is no `x[0]`), then `y[2] = 2`, `y[3] = 2.5`.

### Example 2 — a one-pole low-pass

```
x = [1, 0, 0, 0, 0, 0]
b = [1]
a = [1, -0.5]
CALL Filter(b, a, x : y)
```

**Expected:** the impulse response `1, 0.5, 0.25, 0.125, …`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `b` | Vector | Yes | Numerator (feed-forward) coefficients, `b[1]` first. |
| `a` | Vector | Yes | Denominator (feedback) coefficients; `a[1]` must be non-zero. |
| `x` | Vector | Yes | The signal. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `y` | Vector | The filtered signal, same length as `x`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `Filter denominator a[1] must be non-zero` | A leading zero in `a`. | Normalize the transfer function so `a[1] ≠ 0`. |
| `Filter coefficients must all be finite` | A `NaN` or infinity among the coefficients. | Check the design step that produced them. |
