[Topic: syntax]
# Equation Syntax & Rules

frees parses standard mathematical notation with a few rules worth knowing up front.

## Core rules
- **Equality (`=`)** — a single `=` is mathematical equality, never assignment. `P * V = m * R * T` is valid; the solver rearranges it to find whichever variable is unknown. There is no `==` or `:=` at the top level (those are for `FUNCTION`/`PROCEDURE` bodies).
- **Case insensitivity** — `Temp`, `TEMP`, and `temp` are one variable. Watch for accidental clashes: a state `T` and a time `t` are the same name (rename one).
- **No implicit multiplication** — write `2 * x`, not `2x`. Likewise `a(b+c)` is a function call, not `a*(b+c)`.
- **Operators** — `+`, `-`, `*`, `/`, `^` (exponentiation), and `%` (modulo). `^` is right-binding: `2^3^2 = 2^9`.
- **Comments** — `{ … }` or `"…"` are inline comments; `//` at the start of a line makes the whole line narrative (markdown). Use comments to label states and document assumptions.
- **Strings** — prefer **single quotes** for values the solver reads (`fluid$='Water'`, `INCOMP::MEG[0.50]`). A double-quoted `"…"` span is a comment, the same as `{ … }`, so `"Water"` is not a fluid name.

## Built-in constants
Physical constants are available with a trailing `#` (by long-standing convention) and substituted at parse time:

| Name | Meaning |
| --- | --- |
| `pi#` | $\pi$ |
| `g#` | Standard gravity, $9.80665$ m/s² |
| `R#` | Universal gas constant, $8.31446$ J/mol·K |
| `N#a` | Avogadro's number |
| `k#` | Boltzmann constant |
| `h#` | Planck constant |
| `c#` | Speed of light |
| `sigma#` | Stefan–Boltzmann constant |
| `epsilon0#` | Vacuum permittivity |

```
{ Free-fall distance in 3 s }
d = 0.5 * g# * t^2
```

[Related: gs-declarative, variables, units]

[Topic: math-funcs]
# Mathematical Functions

frees provides a full set of scalar math functions. Built-in functions are supported by the solver's Jacobian engine via symbolic automatic differentiation for smooth expressions, with numerical finite-difference fallbacks for non-smooth or complex operations.

## Trigonometric (angles in radians)
`sin`, `cos`, `tan` and their inverses `arcsin`, `arccos`, `arctan` take and return **radians**. Work in degrees with a unit annotation or `Convert`:
```
theta = 30 [deg]          { stored as radians internally }
y = sin(theta)            { 0.5 }
deg = theta * Convert('rad', 'deg')
```
`atan2(y, x)` returns the quadrant-correct angle of the point `(x, y)`:
```
phi = atan2(1, -1)        { 2.356 rad = 135 deg }
```

## Logarithms, exponentials, powers
| Function | Description |
| --- | --- |
| `exp(x)`, `ln(x)`, `log10(x)`, `log2(x)` | exponential and logs (natural / base-10 / base-2) |
| `sqrt(x)`, `cbrt(x)` | square / cube root |
| `abs(x)` | absolute value |
| `min(a,b,…)`, `max(a,b,…)` | element selection |
| `mod(a, b)`, `gcd(a, b)`, `lcm(a, b)` | modulo, greatest common divisor, least common multiple |
| `factorial(n)` | $n!$ (integer) |

## Hyperbolic
`sinh`, `cosh`, `tanh` and `arcsinh`, `arccosh` (x ≥ 1), `arctanh` (|x| < 1). Note `sinh(x) + cosh(x) = exp(x)`.

## Rounding & integer
| Function | Description |
| --- | --- |
| `round(x, decimals)` | round to `decimals` places |
| `floor(x)` / `ceil(x)` / `trunc(x)` | round down / up / toward zero |
| `sign(x)` | -1, 0, or 1 |
| `step(x)` | unit step (1 if x ≥ 0, else 0) |

```
val1 = round(3.14159, 3)   { 3.142 }
val2 = floor(2.7)          { 2 }
val3 = step(0.5)           { 1 }
```

## Conditional selection & series
| Function | Description |
| --- | --- |
| `If(a, b, lt, eq, gt)` | returns `lt` if `a<b`, `eq` if `a=b`, `gt` if `a>b` |
| `Sum(i, start, end, term)` | $\sum_{i=start}^{end} term$ |
| `Product(i, start, end, term)` | $\prod_{i=start}^{end} term$ |
| `average(a, b, …)` | arithmetic mean |

`If` is the inline branch for the declarative top level (use `IF…THEN…ELSE` inside `FUNCTION`/`PROCEDURE` bodies):
```
{ Pick k = 1.8 above 300 K, else 1.2 }
temp = 350 [K]
k = If(temp, 300, 1.2, 1.5, 1.8)   { k = 1.8 }

{ Sum of squares 1+4+9+16 = 30 }
s = Sum(i, 1, 4, i^2)
```

> **Looking for one function?** This page teaches the families; every built-in has its own page with full syntax, arguments, and examples. Browse them all in **Reference → A–Z Function Index**.

[Related: special-funcs, ref-index, complex]

[Topic: special-funcs]
# Special & Statistical Functions

Transcendental and statistical distribution functions for less common but important calculations.

## Statistical distributions
- **`Probability(x1, x2, mean, stddev)`** — probability that a normal variate lies in the interval `[x1, x2]`.
- **`NormalCDF(x, mean, stddev)`** — cumulative normal probability `Pr(X ≤ x)`.
- **`Chi_Square(x, df)`** — cumulative chi-square CDF at `x` with `df` degrees of freedom.
- **`Random(a, b[, seed])`** — uniform random number in `[a, b]`.
- **`RandG(mean, stddev[, seed])`** — Gaussian random number.

```
prob = Probability(75, 85, 80, 5)   { 0.6827 — within ±1σ of N(80, 5) }
```

## Special mathematical functions
- **`Gamma(x)`** — $\Gamma(x)$, with $\Gamma(n+1)=n!$.
- **`LogGamma(x)`** — $\ln \Gamma(x)$ (avoids overflow for large x).
- **`Digamma(x)`** — $\psi(x) = \frac{d}{dx}\ln\Gamma(x)$.
- **`Beta(a, b)`** — $B(a,b) = \frac{\Gamma(a)\Gamma(b)}{\Gamma(a+b)}$.
- **`Erf(x)` / `Erfc(x)` / `ErfInv(x)`** — error function, complementary, and inverse.
- **`BesselJ(n, x)` / `BesselY(n, x)`** — Bessel functions of the first and second kind, order $n$.
- **`BesselI(n, x)` / `BesselK(n, x)`** — modified Bessel functions of the first and second kind.

> Each function above has a dedicated reference page with its mathematical definition and worked examples — see **Reference → A–Z Function Index**.

[Related: math-funcs, ref-index, uncertainty]

[Topic: variables]
# Variables, Guesses & Bounds

A system is solvable only when the number of equations equals the number of unknowns — the **degrees of freedom (DoF)** are zero. Press **F4** (Check) to see the DoF and confirm the system is well-posed before solving.

[Diagram: DoF]

## Bounds versus units
`T = 25 [C]` is a **unit annotation**: the compiler converts the literal to SI (298.15 K) before solving. `GUESS T [250, 400]` is a **solver bound** on that same SI value — it does not convert, and it is not a unit. Mixing the two (`GUESS T [0, 100]` intending Celsius) boxes the Newton iterate in kelvin.

In-text `GUESS` wins over Variable Information on conflict; the window is a view of the same scalars, including public member paths such as `HX.in.P`.

## The Variable Information panel
Open it with `Ctrl + I`. For every variable you can set:

- **Guess** — the starting point for the Newton-Raphson solver. Required for nonlinear equations; a poor guess is the most common cause of non-convergence.
- **Lower / Upper bounds** — physical limits that keep the solver out of invalid domains (e.g. `T ≥ 0`, `0 ≤ x ≤ 1` for a quality or fraction, `P > 0`).
- **Fixed** — locks the variable to its guess, removing it from the unknowns. Handy for "what if I hold this constant" studies.

## `time` versus temperature names
Names are case-insensitive, so a state `T` and a time `t` are **one variable**. In a `DYNAMIC` block, name the independent axis `time` (the default) and keep temperatures as `Temp` / `T_wall` / `T_inf`. See *Transient / ODE Systems* for the collision that follows from calling both `t`.

## Why guesses matter
The Colebrook friction equation is transcendental — it has no closed form, so frees iterates from a guess. Without a guess it may diverge or land on the wrong branch:

```
Re = 1e5
eps = 0.00015
D = 0.25 [m]
{ ff is unknown — set a guess of ~0.02 in Variable Info }
1/sqrt(ff) = -2*log10(eps/(3.7*D) + 2.51/(Re*sqrt(ff)))
```
A guess near `0.02` converges in a few iterations; a guess of `0.5` may stall. As a rule of thumb, guess dimensionless ratios near `0.5`, temperatures/pressures near the expected magnitude, and flow rates near the order of magnitude you expect.

> **Tip:** For implicit property lookups (e.g. `h = Enthalpy(Water, P=P, s=s)`), a guess on the unknown output variable also helps the solver pick the right two-phase region.

[Related: gs-units-check, uncertainty, api]

[Topic: uncertainty]
# Uncertainty Propagation

frees does automatic **first-order** uncertainty propagation: declare the tolerance of each measured input, and it propagates the uncertainty to every dependent result using the root-sum-of-squares (RSS) rule. You don't write the partial derivatives — the solver computes them from the numerical Jacobian.

## How to use it
1. **Declare input uncertainties** with `UncertaintyOf(var) = value` on your independent (measured) variables.
2. **Query output uncertainties** with `UncertaintyOf(var)` on any dependent variable — frees returns its propagated absolute uncertainty.

The combination rule is:
$$u_y = \sqrt{\sum \left(\frac{\partial y}{\partial x_i} u_{x_i}\right)^2}$$

## Worked example
```
{ Nominal values }
P = 100000 [Pa]
T = 300 [K]
R = 287 [J/kg-K]
P = rho * R * T          { rho is the computed result }

{ Measured-input uncertainties }
UncertaintyOf(P) = 500 [Pa]
UncertaintyOf(T) = 2.0 [K]

{ Propagated uncertainty in density }
unc_rho = UncertaintyOf(rho)
```
Only independent inputs should carry a declared uncertainty; assigning one to a computed output that you also query is redundant. Uncertainties are shown alongside each value in the Solution panel.

[Related: variables, units, api]

[Topic: units]
# Units & Dimensional Consistency

frees checks every equation for dimensional consistency and runs all calculations in SI base units internally. You annotate **inputs** for convenience; **results** come back in SI and you convert or label them as needed.

## Annotating inputs
Bracket a numeric literal to tag it with a unit; the compiler converts it to SI at parse time:
```
P = 140 [kPa]      { stored as 140000 Pa }
m = 120 [lb]       { stored as 54.43 kg }
T = 25 [C]         { stored as 298.15 K }
```
The full list of recognized units and built-in constants lives in the reference table below (`[Component: UnitsReference]`).

## Results are SI
A computed result has the SI unit of its expression — `P * Vol = m * R * T` gives `m` in kg, `q = k*A*dT/L` gives watts. To display in engineering units, either convert explicitly, set display units in the Variable Information window, or use the `Convert` function:
```
P_kPa  = P / 1000                { kPa, by division }
P_kPa2 = P * Convert(Pa, kPa)    { using Convert }
```

## The Convert() function
`Convert(From, To)` returns the pure scaling factor between two units of the **same dimension** (no offset):
```
area_in2 = area_ft2 * Convert(ft^2, in^2)     { ×144 }
```

## Temperature conversions
Temperature scales have offsets as well as scaling, so use `ConvertTemp(From, To, value)` instead of `Convert`:
```
T_f = ConvertTemp(C, F, 100)   { 212 F }
T_k = ConvertTemp(F, K, 32)    { 273.15 K }
```

## Worked example
```
{ Pressure input in psi, converted to kPa }
P = 100 [psi]                 { stored as 689475.7 Pa internally }
P_kPa = P * Convert(Pa, kPa)  { converts SI Pa to kPa: 689.5 kPa }
```

> **Common pitfall:** `Convert` works for differences and ratios (kPa, ft², mph); it does **not** handle temperature offsets. Mixing them — e.g. `Convert(C, K)` — gives a wrong result. Always use `ConvertTemp` for absolute temperatures.

## Absolute temperatures versus differences
`T = 25 [C]` is an **absolute** temperature (stored as 298.15 K). A *difference* such as a 10 °C rise is 10 K — annotate it `[deltaC]` / `[deltaK]`, or write the kelvin difference directly. `Convert(C, K)` is the wrong tool for either: it has no offset, so it cannot turn 25 °C into 298.15 K, and it is unnecessary for a difference that is already 10 K.

[Component: UnitsReference]

[Related: syntax, variables, ref-units]

[Topic: arrays]
# Arrays & For Loops

An array element is written with a 1-based index in square brackets: `T[1]`, `P[5]`. Declare the array's size with a slice suffix when you first use it (`T[1:5]`), so the compiler can allocate it; afterwards the bare name works.

## The FOR loop = equation expansion
A `FOR ... END` block isn't a runtime loop — the compiler **expands** it into one equation per index at compile time. This is the idiomatic way to generate a family of equations (one per state point, node, or time step):
```
{ One enthalpy equation per state }
P[1:3] = [8000, 2000, 10]      { kPa }
T[1:3] = [480, 200, 45]        { C }
FOR i = 1 TO 3
  h[i] = Enthalpy(Water, P=P[i], T=T[i])
END
```
The loop variable (`i`) is local to the block and must not clash with a model variable (names are case-insensitive).

## Array helper functions
- **`ArrayElmt(array[1:N], index)`** — element at a **dynamically computed** index. Use it when the index is itself a variable (a plain `T[idx]` only works with a literal index):
```
idx = 3
val = ArrayElmt(T[1:10], idx)   { the value of T[3] }
```

> **Tip:** for matrices and vectors declared with literals, see *Declaring Matrices & Vectors*. For column-by-column access to solved tables, see *Table Accessors & Aggregates*.

[Related: matrices-decl, functions, table-accessors]

[Topic: complex]
# Complex Numbers

frees supports complex numbers natively. A complex variable is stored as two real scalars: append `_r` for the real part and `_i` for the imaginary part. So `Z_r = 10` and `Z_i = 5` together represent $Z = 10 + 5j$.

Arithmetic operators (`+`, `-`, `*`, `/`, `^`) work on the paired `_r`/`_i` variables automatically — you write `Z = A * B` and frees keeps both components in step.

## Helper functions
| Function | Returns |
| --- | --- |
| `Real(z)` / `Imag(z)` | real / imaginary part |
| `Conj(z)` | complex conjugate |
| `Magnitude(z)` | modulus $\lvert z \rvert$ |
| `Angle(z)` / `AngleDeg(z)` | argument in radians / degrees |
| `Cis(theta)` | $e^{j\theta} = \cos\theta + j\sin\theta$ |

```
{ Build a phasor from magnitude and angle }
Z_r = Magnitude_r      { reuse the real parts }
Z_i = 0
A = Cis(phi)           { unit phasor at angle phi }
```

[Related: math-funcs, symbolic-cas, matrices-sys]

[Topic: strings]
# String Variables & Functions

A string variable ends with `$` (e.g. `fluid$`), and string literals use **single** quotes (`'R134a'`, `'wall'`). Strings are resolved at compile time — most often as a fluid name in a property call, or as a geometry label for the Heisler functions.

```
fluid$ = 'R134a'
h = Enthalpy(fluid$, P=P1, x=1)     { fluid name from a string variable }
```

## String functions
These take string-literal (or string-variable) arguments and return a number:
- **`StringLen(s)`** — number of characters.
- **`StringPos(s, sub)`** — 1-based index of the first occurrence of `sub` in `s` (0 if not found).
- **`StringVal(s)`** — convert a numeric string to a scalar (`StringVal('3.14')` → `3.14`).

String-returning functions (also suffixed `$`) include `LowerCase$`, `UpperCase$`, `Trim$`, `Concat$`, `Copy$`, `Chr$`, `Date$`, `Time$`, `TimeStamp$`, `UnitSystem$`, and `UnitsOf$`.

[Related: syntax, thermo, ref-index]
