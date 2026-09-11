---
name: XCorr
category: Stats
summary: Full linear cross-correlation of two series, zero lag at the centre.
related: [Convolve, FiltFilt, Welch, Smooth]
examples: []
tags: [xcorr, cross-correlation, autocorrelation, lag, delay, alignment, signal]
---

# XCorr

Finds how far one signal leads or lags another — the transport delay between two thermocouples, the phase between a drive and a response, the period hidden in a noisy trace.

## Syntax

```
CALL XCorr(a, b : c)
```

## Description

Returns the full linear cross-correlation, `m + n − 1` samples long. Output element `c[i]` carries lag `i − n` in 1-based terms: the **centre** element `c[n]` is the zero-lag correlation, a peak to the right of centre means `a` leads `b`, and a peak to the left means it lags.

`XCorr(a, a)` is the autocorrelation, which always peaks at the centre with the signal's energy `Σ aᵢ²`; the position of its next peak is the dominant period.

Both series are used raw. Remove any trend first (**Detrend**) — a common ramp correlates with itself and will dominate whatever you were actually looking for.

## Mathematical Formulation

$$ c_{\ell} = \sum_{j} a_j\, b_{j-\ell}, \qquad \ell = -(n-1),\ \dots,\ m-1 $$

which is the convolution of `a` with `b` reversed.

> **Method:** direct sum, `O(m·n)`. The evaluator's equation budget rejects vectors long before an FFT-based correlation would win.

## Examples

### Example 1 — measuring a one-sample delay

```
a = [0, 0, 1, 2, 1, 0, 0]
b = [0, 1, 2, 1, 0, 0, 0]
CALL XCorr(a, b : c)
```

**Expected:** the peak lands one element right of the centre — `a` leads `b` by one sample.

### Example 2 — autocorrelation energy

```
x = [1, 3, 2, 6, 4, 9, 5, 12]
CALL XCorr(x, x : r)
```

**Expected:** `r[8]` (the centre of 15) is `Σ xᵢ² = 316`, and `r` is symmetric about it.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `a` | Vector | Yes | The reference series, length `m`. |
| `b` | Vector | Yes | The series compared against it, length `n`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `c` | Vector | `m + n − 1` correlation values, lag `−(n−1)` first. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `XCorr requires the output length to be m + n - 1` | A declared output of the wrong size. | Leave it bare and let it be sized, or declare `[1:m+n-1]`. |
