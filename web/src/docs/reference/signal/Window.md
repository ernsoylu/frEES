---
name: Window
category: Stats
summary: Multiplies a series by a symmetric taper (Hann, Hamming, Blackman, Bartlett or rectangular).
related: [FFT, Welch, Detrend, Smooth]
examples: []
tags: [window, taper, hann, hamming, blackman, bartlett, leakage, spectrum]
---

# Window

Tapers a finite record to zero at both ends before a transform, so the discontinuity between the last sample and the first does not smear energy across the whole spectrum.

## Syntax

```
CALL Window(x, 'hann' : y)
```

## Description

An `FFT` treats its input as one period of a periodic signal. A record that does not happen to contain a whole number of cycles therefore has a step in it, and that step leaks energy into every bin — enough to bury a small tone sitting next to a large one. Multiplying by a taper that reaches zero at both ends removes the step.

The windows here are **symmetric** (denominator `n − 1`), which is the right taper for a single finite record and matches MATLAB's `hann(n)` and SciPy's `get_window(..., fftbins=False)`. **Welch** uses the *periodic* form internally, because there the segments tile a longer record; the two differ by one sample and the difference is not cosmetic.

Every window trades resolution for leakage: Hann is the usual default, Blackman suppresses distant leakage further at the cost of a wider main lobe, Hamming sits between them, Bartlett is the simple triangle, and `'rect'` is the identity — the taper you already have when you do nothing.

## Mathematical Formulation

With $t = j/(n-1)$,

$$ w^{\text{hann}}_j = 0.5 - 0.5\cos 2\pi t, \quad w^{\text{hamming}}_j = 0.54 - 0.46\cos 2\pi t $$
$$ w^{\text{blackman}}_j = 0.42 - 0.5\cos 2\pi t + 0.08\cos 4\pi t, \quad w^{\text{bartlett}}_j = 1 - |2t - 1| $$

and $y_j = w_j x_j$.

> **Method:** the closed-form weights, evaluated through `libm` so native and browser runs agree bit for bit.

## Examples

### Example 1 — tapering before a transform

```
x  = [1, 2, 3, 4, 5, 6, 7, 8]
im = [0, 0, 0, 0, 0, 0, 0, 0]
CALL Window(x, 'hann' : xw)
CALL FFT(xw, im : re, imf)
```

**Expected:** `xw[1] = 0` and `xw[8] = 0` — a symmetric Hann closes on zero at both ends.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `x` | Vector | Yes | The series to taper. |
| `'kind'` | String | Yes | `'rect'`, `'hann'`, `'hamming'`, `'blackman'` or `'bartlett'`. `'hanning'`, `'boxcar'`, `'rectangular'`, `'none'` and `'triangular'` are accepted spellings. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `y` | Vector | `x` multiplied by the window, same length. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `Unknown window` | A name outside the list. | Use one of the five; the name is matched case-insensitively. |
| `Window option 2 must be a quoted name` | The kind was passed unquoted. | Write `'hann'`, with quotes. |
