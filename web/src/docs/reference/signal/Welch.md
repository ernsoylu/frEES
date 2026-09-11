---
name: Welch
category: Stats
summary: Averaged-periodogram power spectral density, one-sided and in x²/Hz.
related: [FFT, Window, Detrend, XCorr]
examples: []
tags: [welch, psd, spectrum, power spectral density, periodogram, vibration, noise]
---

# Welch

Estimates where a signal's power sits in frequency. A single `FFT` of a noisy record gives a spectrum whose variance does not fall as the record gets longer; Welch's method trades resolution for that variance by splitting the record into overlapping segments and averaging their spectra.

## Syntax

```
CALL Welch(x, fs, nperseg : f, pxx)
```

## Description

Splits `x` into 50 %-overlapped segments of `nperseg` samples, removes each segment's mean, tapers it with a **periodic** Hann window, and averages the squared spectra. A trailing partial segment is dropped rather than zero-padded — padding would quietly bias the average toward the record's tail.

Both outputs are `nperseg/2 + 1` long: `f[k] = (k−1)·fs/nperseg` in Hz, and `pxx` is a one-sided **power spectral density** in the signal's units squared per hertz. "One-sided" means the negative frequencies have been folded onto their positive twins, so every bin except DC and Nyquist is doubled. The consequence to lean on when checking a result: `Σ pxx·Δf` recovers the signal's mean square.

`nperseg` sets the trade: a larger segment resolves closely-spaced tones, a smaller one averages more segments and gives a smoother estimate.

## Mathematical Formulation

For segment $i$ with window $w$ of length $L$,

$$ P^{(i)}_k = \frac{2}{f_s \sum_j w_j^2}\left| \sum_{j=0}^{L-1} (x^{(i)}_j - \bar x^{(i)})\, w_j\, e^{-2\pi \mathrm{i} jk/L} \right|^2 $$

(the factor 2 omitted at $k = 0$ and, for even $L$, at $k = L/2$), and $P_k$ is the mean over segments.

> **Method:** mixed-radix / Bluestein FFT per segment — see **FFT** — with a periodic Hann taper and constant detrending.

## Examples

### Example 1 — locating a tone and checking its power

```
{ 4096 samples of a 1 Vrms 50 Hz tone at fs = 1 kHz in x[1..4096] }
CALL Welch(x, 1000, 256 : f, pxx)
```

**Expected:** the largest `pxx` sits in the bin nearest 50 Hz, and `sum(pxx)·(f[2] − f[1])` is 1 — the mean square of a 1 Vrms tone.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `x` | Vector | Yes | The uniformly sampled signal. |
| `fs` | Number | Yes | Sample rate in Hz; may be any expression. Must be positive. |
| `nperseg` | Integer | Yes | Segment length, at least 2 and at most the record length. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `f` | Vector | Bin centre frequencies in Hz, `nperseg/2 + 1` long. |
| `pxx` | Vector | One-sided power spectral density in `x²/Hz`, same length. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `Welch segment length exceeds the series length` | `nperseg > n`. | Shorten the segment or lengthen the record. |
| `Welch requires a positive, finite sample rate` | `fs ≤ 0` or non-finite. | Supply the real acquisition rate. |
| `Welch requires both output vectors to be nperseg/2 + 1 long` | Declared outputs of another size. | Leave them bare and let them be sized. |

> **Uniform sampling is assumed.** Nothing here inspects a time column, and an uneven raster produces a spectrum that looks fine and means nothing. Resample first.
