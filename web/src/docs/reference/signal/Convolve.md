---
name: Convolve
category: Stats
summary: Linear convolution of two sequences, m + n − 1 long.
related: [XCorr, Filter, FFT, Smooth]
examples: []
tags: [convolve, convolution, impulse response, fir, kernel, signal]
---

# Convolve

Applies an impulse response to a signal, or combines two kernels into one.

## Syntax

```
CALL Convolve(a, b : c)
```

## Description

Returns the full linear convolution of `a` and `b`, `m + n − 1` samples long. Convolving a signal with a system's impulse response gives that system's output; convolving two kernels gives the single kernel equivalent to applying them in sequence.

`Convolve` is not the same as **Filter**: it returns the *full* result, including the `n − 1` samples of run-out past the end of the signal, and it has no recursive (denominator) part. Use **Filter** when you want an output the same length as the input, or a filter with feedback.

Correlation is the same sum with one sequence reversed — see **XCorr**.

## Mathematical Formulation

$$ c_k = \sum_{i} a_i\, b_{k-i}, \qquad k = 0,\ \dots,\ m+n-2 $$

> **Method:** direct `O(m·n)` sum. The evaluator's equation budget rejects vectors long before an FFT-based convolution would win.

## Examples

### Example 1 — a small kernel

```
a = [1, 2, 3]
b = [4, 5]
CALL Convolve(a, b : c)
```

**Expected:** `c = 4, 13, 22, 15`.

### Example 2 — an impulse is the identity

```
a = [1]
b = [7, -2, 0.5]
CALL Convolve(a, b : c)
```

**Expected:** `c = 7, -2, 0.5`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| `a` | Vector | Yes | First sequence, length `m`, non-empty. |
| `b` | Vector | Yes | Second sequence, length `n`, non-empty. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| `c` | Vector | The convolution, `m + n − 1` long. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| `Convolve requires the output length to be m + n - 1` | A declared output of the wrong size. | Leave it bare and let it be sized. |
| `Convolve requires two non-empty sequences` | An empty input. | Both sequences need at least one element. |
