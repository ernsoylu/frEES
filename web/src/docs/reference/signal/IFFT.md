---
name: IFFT
category: Stats
summary: Inverse discrete Fourier transform, including the 1/n normalization.
related: [FFT, Convolve, Welch, Window]
examples: []
tags: [ifft, inverse fft, fourier, transform, signal]
---

# IFFT

Takes a spectrum back to the time domain.

## Syntax

```
CALL IFFT(re, im : outRe, outIm)
```

## Description

The inverse of **FFT**, including the `1/n` normalization — so `IFFT(FFT(x))` returns `x`, not `n·x`. Same length rules and the same fast paths: any length, primes included.

Filtering by editing a spectrum and inverting it is a legitimate use, but note that zeroing bins is a brick-wall filter and rings badly in time. **Filter** or **FiltFilt** with a designed kernel is usually what you want instead.

## Mathematical Formulation

$$ x_j = \frac{1}{n}\sum_{k=0}^{n-1} X_k\, e^{+2\pi \mathrm{i} jk/n} $$

> **Method:** as **FFT**, with the conjugate exponent and a final division by `n`.

## Examples

### Example 1 — a flat spectrum inverts to an impulse

```
re = [1, 1]
im = [0, 0]
CALL IFFT(re, im : gr, gi)
```

**Expected:** `gr = 1, 0` and `gi = 0, 0`.

### Example 2 — round trip

```
x  = [3, -1, 0.5, 2.25, -7]
im = [0, 0, 0, 0, 0]
CALL FFT(x, im : fr, fi)
CALL IFFT(fr, fi : br, bi)
```

**Expected:** `br` reproduces `x` and `bi` is zero to rounding.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `re` | Vector | Yes | Real parts of the spectrum. |
| `im` | Vector | Yes | Imaginary parts, same length. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `outRe` | Vector | Real parts of the recovered sequence. |
| `outIm` | Vector | Imaginary parts of the recovered sequence. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `IFFT requires all four vectors to have the same length` | Mismatched inputs or outputs. | All four are the same `n`. |
