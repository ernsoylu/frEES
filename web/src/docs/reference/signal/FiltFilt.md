---
name: FiltFilt
category: Stats
summary: Zero-phase filtering — forward then backward, so the phase shifts cancel.
related: [Filter, Smooth, XCorr, Welch]
examples: []
tags: [filtfilt, zero phase, forward backward, filter, signal, sensor]
---

# FiltFilt

Filters a recorded signal without shifting it in time. Use it whenever the timing of a feature matters — the location of a peak, the moment a threshold is crossed, the alignment of two channels.

## Syntax

```
CALL FiltFilt(b, a, x : y)
```

## Description

Runs **Filter** forward over the record, reverses the result, filters again, and reverses back. Each pass shifts the phase by the same amount in opposite directions, so they cancel exactly: a symmetric feature comes back symmetric, in place.

Two consequences worth stating plainly:

* **The magnitude response is squared.** A design that is −3 dB at some frequency is −6 dB there after `FiltFilt`. Design for half the attenuation you want.
* **It is not causal.** Every output sample depends on the whole record, so this is a post-processing tool, never something to put inside a control loop.

Before filtering, the signal is odd-extended by `3·max(len(a), len(b))` samples (clipped to `n − 1`) at both ends and the extension discarded afterwards. That is what stops the ends from ringing.

## Mathematical Formulation

With $H$ the causal filter of **Filter** and $R$ the reversal operator,

$$ y = R\,H\,R\,H\,x \quad\Longrightarrow\quad Y(\omega) = |H(\omega)|^2 X(\omega) $$

— real and non-negative, hence zero phase.

> **Method:** two `lfilter` passes over an odd-extended record. The SciPy steady-state `lfilter_zi` warm start is not applied; the padding carries the edge behaviour.

## Examples

### Example 1 — a constant survives untouched, ends included

```
x = [4, 4, 4, 4, 4, 4, 4, 4]
b = [0.2, 0.2, 0.2, 0.2, 0.2]
a = [1]
CALL FiltFilt(b, a, x : y)
```

**Expected:** every `y[j] = 4`. A zero-padded implementation would sag at both ends instead.

### Example 2 — a symmetric pulse stays symmetric

```
x = [0, 0, 0, 1, 0, 0, 0]
b = [0.25, 0.5, 0.25]
a = [1]
CALL FiltFilt(b, a, x : y)
```

**Expected:** `y` is symmetric about its centre — the property a single causal pass does not have.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `b` | Vector | Yes | Numerator coefficients. |
| `a` | Vector | Yes | Denominator coefficients; `a[1]` non-zero. |
| `x` | Vector | Yes | The signal, at least two samples. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `y` | Vector | The zero-phase filtered signal, same length as `x`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `FiltFilt requires at least two samples` | A one-sample record. | There is nothing to reflect about; use **Filter** or a longer record. |
