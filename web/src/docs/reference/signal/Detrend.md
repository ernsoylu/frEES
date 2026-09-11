---
name: Detrend
category: Stats
summary: Removes a least-squares straight line (or just the mean) from a measured series.
related: [Smooth, Window, Welch, FFT]
examples: []
tags: [detrend, trend, drift, baseline, signal, sensor]
---

# Detrend

Removes the slow drift from a measured series so the fluctuation you actually care about is what is left. Thermocouple drift, a settling load cell, a slowly warming ambient — all of them add a ramp that dominates a spectrum or a correlation unless it is taken out first.

## Syntax

```
CALL Detrend(y : yd)
CALL Detrend(y, 'linear' : yd)
CALL Detrend(y, 'constant' : yd)
```

## Description

`'linear'` (the default) fits a straight line by ordinary least squares over the sample index `0, 1, …, n−1` and subtracts it, so the result has neither a mean nor a slope. `'constant'` subtracts only the mean and leaves the slope intact.

The fit is over the **index**, not over a time column: samples are assumed evenly spaced. On an uneven raster, resample first.

Detrending is a linear operation, which is the property to lean on when reasoning about it: `Detrend(a + b)` is `Detrend(a) + Detrend(b)`, so adding a ramp to a signal cannot change what detrending returns for the signal itself.

## Mathematical Formulation

With $\bar t = (n-1)/2$ and $\bar y$ the sample mean,

$$ m = \frac{\sum_{j} (j - \bar t)(y_j - \bar y)}{\sum_{j} (j - \bar t)^2}, \qquad yd_j = y_j - \left[\bar y + m\,(j - \bar t)\right] $$

> **Method:** the closed-form ordinary-least-squares line on a known uniform design — no factorization, and no conditioning to worry about.

## Examples

### Example 1 — a drifting temperature record

```
T = [20.1, 20.4, 20.6, 21.0, 21.2, 21.5, 21.7, 22.0]
CALL Detrend(T : ripple)
peaks = peakcount(0, 0, ripple)
```

The ~0.27 K per sample warming trend is removed; what is left is the measurement ripple around it.

### Example 2 — re-centring without flattening

```
T = [20.1, 20.4, 20.6, 21.0]
CALL Detrend(T, 'constant' : centred)
```

**Expected:** the mean of `centred` is 0 and the sample-to-sample slope is unchanged.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `y` | Vector | Yes | The measured series, at least one sample. |
| `'mode'` | String | No | `'linear'` (default) or `'constant'`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `yd` | Vector | The detrended series, same length as `y`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `Detrend mode must be 'linear' or 'constant'` | An unrecognised mode string. | Use one of the two spellings; `'const'` and `'mean'` are also accepted for the constant form. |
| `Detrend requires the output vector to match the input length` | A declared output of a different size. | Leave the output bare and let it be sized, or declare it `[1:n]`. |
