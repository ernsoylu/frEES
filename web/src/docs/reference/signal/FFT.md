---
name: FFT
category: Stats
summary: Discrete Fourier transform of a complex sequence, any length.
related: [IFFT, Convolve, Welch, Window, XCorr]
examples: []
tags: [fft, dft, fourier, spectrum, frequency, transform, signal]
---

# FFT

Transforms a sampled signal into its frequency content.

## Syntax

```
CALL FFT(re, im : outRe, outIm)
```

## Description

Takes the complex sequence carried as two equal-length real vectors and returns its discrete Fourier transform in the same form. For a real signal, pass a vector of zeros as `im`.

**Any length works, including primes.** Short transforms run the direct `O(n²)` sum; longer ones take a radix-2 Cooley–Tukey path when the length is a power of two and Bluestein's chirp-z otherwise, both `O(n log n)`. There is no power-of-two restriction at any size and no zero-padding happening behind your back — the transform you get is the transform of the samples you gave.

Bin `k` (1-based `outRe[k+1]`) corresponds to frequency `k·fs/n`. Bins above `n/2` are the negative frequencies, mirrored for a real input.

Two things to do first, for a record that is not exactly periodic: remove the trend (**Detrend**) and taper the ends (**Window**). Without them, a step between the last sample and the first leaks energy across every bin. For a power spectrum specifically, **Welch** does both and averages, and is usually the better tool.

## Mathematical Formulation

$$ X_k = \sum_{j=0}^{n-1} x_j\, e^{-2\pi \mathrm{i} jk/n} $$

> **Method:** direct sum up to 32 points; radix-2 Cooley–Tukey at power-of-two lengths; Bluestein's chirp-z otherwise. All trigonometry goes through `libm`, so native and browser runs agree bit for bit.

## Examples

### Example 1 — the transform of a unit impulse is flat

```
re = [1, 0, 0, 0]
im = [0, 0, 0, 0]
CALL FFT(re, im : fr, fi)
```

**Expected:** `fr = 1, 1, 1, 1` and `fi = 0, 0, 0, 0`.

### Example 2 — a real ramp

```
re = [1, 2, 3, 4]
im = [0, 0, 0, 0]
CALL FFT(re, im : outRe, outIm)
```

**Expected:** `outRe[1] = 10`, the sum of the samples — bin 0 is always the DC total.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `re` | Vector | Yes | Real parts of the sequence. |
| `im` | Vector | Yes | Imaginary parts, same length as `re`. Pass zeros for a real signal. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `outRe` | Vector | Real parts of the transform, same length. |
| `outIm` | Vector | Imaginary parts of the transform, same length. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `FFT requires all four vectors to have the same length` | Mismatched inputs or outputs. | All four are the same `n`. |
| `FFT real and imaginary parts must have equal length` | The two input vectors differ. | Supply an imaginary vector of zeros the same length as the real one. |
