---
name: Smooth
category: Stats
summary: Centred moving average over an odd window, with honest ends.
related: [Detrend, Filter, FiltFilt, Window]
examples: []
tags: [smooth, moving average, filter, noise, signal, sensor]
---

# Smooth

A centred moving average — the first thing to reach for when a sensor trace is noisy and you want to see the shape of it.

## Syntax

```
CALL Smooth(x, k : y)
```

## Description

Each output sample is the mean of the `k` samples centred on it. `k` must be odd, so "centred" means what it says, and no larger than the series.

At the ends, where the full window does not exist, the average is taken over **the samples that do** rather than over zeros. That is the difference between a constant series coming back constant and one that sags at both ends — a zero-padded moving average always does the latter, and the sag is easy to mistake for a real transient.

A moving average is a low-pass filter with a `sinc` response, so it is not a good choice when you care about the frequency content; use **Filter** or **FiltFilt** with a designed kernel for that. As a way to see the trend in a plot, it is exactly right.

## Mathematical Formulation

$$ y_j = \frac{1}{|W_j|}\sum_{i \in W_j} x_i, \qquad W_j = \{\,i : |i - j| \le \lfloor k/2 \rfloor,\; 0 \le i < n \,\} $$

> **Method:** direct windowed mean; the window shrinks at the ends rather than being padded.

## Examples

### Example 1 — smoothing a noisy pressure trace

```
p = [1, 3, 2, 6, 4, 9, 5, 12]
CALL Smooth(p, 3 : ps)
```

**Expected:** `ps[1] = 2` (the mean of the two samples that exist), `ps[3] = 11/3`, `ps[8] = 8.5`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `x` | Vector | Yes | The series to smooth. |
| `k` | Integer | Yes | Window length; odd, at least 1, at most `n`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `y` | Vector | The smoothed series, same length as `x`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `Smooth requires an odd window length` | An even `k`. | Use `k − 1` or `k + 1`; an even window has no centre sample. |
| `Smooth window is longer than the series` | `k > n`. | Shorten the window, or lengthen the record. |
