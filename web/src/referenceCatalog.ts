// GENERATED FILE - DO NOT EDIT DIRECTLY.
// Compiled from src/docs/reference/**/*.md by scripts/compile-docs.js (npm run compile-docs).

export interface ReferencePage {
  name: string;
  slug: string;
  category: string;
  summary: string;
  related: string[];
  examples: string[];
  tags: string[];
  references: string[];
  /** Guide topic ids whose text calls this symbol (auto-computed backlinks). */
  guides: string[];
  body: string;
}

export const REFERENCE_PAGES: ReferencePage[] = [
  {
    name: `isa_p`,
    slug: `isa_p`,
    category: `Atmosphere`,
    summary: `ISA 1976 pressure [Pa] at geopotential altitude [m]`,
    related: [],
    examples: [],
    tags: [`isa`, `atmosphere`],
    references: [`U.S. Standard Atmosphere, 1976 (NOAA/NASA/USAF)`],
    guides: [],
    body: `ISA 1976 pressure [Pa] at geopotential altitude [m]


## Syntax

\`\`\`
isa_P(alt)
\`\`\`

## Description

ISA 1976 pressure [Pa] at geopotential altitude [m]

## Mathematical Formulation

$$ P(h) = P_b\\left(\\frac{T_b}{T_b + L_b(h-h_b)}\\right)^{g_0 M/(R L_b)} \\quad (L_b \\ne 0) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`alt\` | Number | Yes | Geopotential altitude [m]. |

## References

1. U.S. Standard Atmosphere, 1976 (NOAA/NASA/USAF).`,
  },
  {
    name: `isa_rho`,
    slug: `isa_rho`,
    category: `Atmosphere`,
    summary: `ISA 1976 density [kg/m^3] at geopotential altitude [m]`,
    related: [],
    examples: [],
    tags: [`isa`, `rho`, `atmosphere`],
    references: [`U.S. Standard Atmosphere, 1976 (NOAA/NASA/USAF)`],
    guides: [],
    body: `ISA 1976 density [kg/m^3] at geopotential altitude [m]


## Syntax

\`\`\`
isa_rho(alt)
\`\`\`

## Description

ISA 1976 density [kg/m^3] at geopotential altitude [m]

## Mathematical Formulation

$$ \\rho(h) = \\frac{P(h)\\,M}{R\\,T(h)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`alt\` | Number | Yes | Geopotential altitude [m]. |

## References

1. U.S. Standard Atmosphere, 1976 (NOAA/NASA/USAF).`,
  },
  {
    name: `isa_t`,
    slug: `isa_t`,
    category: `Atmosphere`,
    summary: `ISA 1976 temperature [K] at geopotential altitude [m]`,
    related: [],
    examples: [],
    tags: [`isa`, `atmosphere`],
    references: [`U.S. Standard Atmosphere, 1976 (NOAA/NASA/USAF)`],
    guides: [],
    body: `ISA 1976 temperature [K] at geopotential altitude [m]


## Syntax

\`\`\`
isa_T(alt)
\`\`\`

## Description

ISA 1976 temperature [K] at geopotential altitude [m]

## Mathematical Formulation

$$ T(h) = T_b + L_b\\,(h - h_b) \\quad\\text{(layer lapse rate } L_b) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`alt\` | Number | Yes | Geopotential altitude [m]. |

## References

1. U.S. Standard Atmosphere, 1976 (NOAA/NASA/USAF).`,
  },
  {
    name: `differentiate`,
    slug: `differentiate`,
    category: `Calculus`,
    summary: `Numerical dy/dx at xv from a TABLE`,
    related: [],
    examples: [],
    tags: [`differentiate`, `calculus`],
    references: [],
    guides: [`lookup-tables`],
    body: `Numerical dy/dx at xv from a TABLE


## Syntax

\`\`\`
Differentiate('t', y, x, xv)
\`\`\`

## Description

Numerical dy/dx at xv from a TABLE

## Mathematical Formulation

$$ \\left.\\frac{dy}{dx}\\right|_{x_v} \\approx \\frac{y_{i+1}-y_{i-1}}{x_{i+1}-x_{i-1}} \\quad\\text{(central difference on the table)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | Number | Yes | Name of a TABLE block (string). |
| \`y\` | Number | Yes | Value / second coordinate. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`xv\` | Number | Yes | Point at which to evaluate. |`,
  },
  {
    name: `gaussintegral`,
    slug: `gaussintegral`,
    category: `Calculus`,
    summary: `Definite integral by Gauss-Legendre quadrature`,
    related: [],
    examples: [],
    tags: [`gaussintegral`, `calculus`],
    references: [],
    guides: [],
    body: `Definite integral by Gauss-Legendre quadrature


## Syntax

\`\`\`
GaussIntegral(expr, var, lower, upper)
\`\`\`

## Description

Definite integral by Gauss-Legendre quadrature

## Mathematical Formulation

$$ \\int_a^b f(x)\\,dx \\approx \\frac{b-a}{2}\\sum_{i=1}^{n} w_i\\,f\\!\\left(\\tfrac{b-a}{2}\\xi_i + \\tfrac{a+b}{2}\\right) \\quad\\text{(Gauss–Legendre)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |
| \`var\` | Number | Yes | Integration variable. |
| \`lower\` | Number | Yes | Lower limit. |
| \`upper\` | Number | Yes | Upper limit. |`,
  },
  {
    name: `integral`,
    slug: `integral`,
    category: `Calculus`,
    summary: `Definite integral; with self-reference, a scalar first-order ODE.`,
    related: [`gaussintegral`, `differentiate`, `IntegralValue`],
    examples: [`tank-draining`, `newton-cooling`],
    tags: [`calculus`, `integral`, `quadrature`, `ode`, `definite integral`],
    references: [],
    guides: [`calculus`],
    body: `Computes a **definite integral** of an expression with respect to a variable over a
range. When the integrand references the result variable itself, frees detects the
self-reference and integrates the corresponding **first-order initial-value ODE**
starting from 0 at the lower limit.

## Syntax

\`\`\`
A = Integral(expr, var, lower, upper)
\`\`\`

## Description

For a plain definite integral, \`expr\` depends only on \`var\`. For the ODE pattern,
\`expr\` contains the result variable — frees then integrates the *change*, so you
rebuild the quantity of interest from the integrated increment. For coupled,
stiff, or multi-state systems use a \`DYNAMIC\` block instead.

## Mathematical Formulation

$$ A = \\int_{\\text{lower}}^{\\text{upper}} \\text{expr}(\\text{var})\\,d(\\text{var}) $$

In the self-referential (ODE) form, with \`y\` the result and \`y(lower) = 0\`,

$$ \\frac{dy}{d\\,\\text{var}} = \\text{expr}(y, \\text{var}), \\qquad y = \\int_{\\text{lower}}^{\\text{var}} \\text{expr}\\,d(\\text{var}) $$

> **Method:** adaptive quadrature for a definite integral; an initial-value ODE
> integration when self-reference is detected.

## Examples

### Example 1 — Draining-tank ODE via the self-reference pattern

[Run: tank-draining]

**Expected:** integrating the volume drop and rebuilding \`V = V0 − drop\` gives the
tank volume after the elapsed time.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Expression | Yes | Integrand (may self-reference the result for the ODE form). |
| \`var\` | Variable | Yes | Integration variable. |
| \`lower\` | Number | Yes | Lower limit. |
| \`upper\` | Number | Yes | Upper limit. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`A\` | Number | The definite integral (or integrated change for the ODE form). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NON_CONVERGENT\` | integrand singular/stiff | Use \`GaussIntegral\`, a finer setup, or a \`DYNAMIC\` block. |`,
  },
  {
    name: `uncertaintyof`,
    slug: `uncertaintyof`,
    category: `Calculus`,
    summary: `Propagated uncertainty of X (resolved in a second solve pass)`,
    related: [],
    examples: [],
    tags: [`uncertaintyof`, `calculus`],
    references: [`JCGM 100:2008 (GUM)`],
    guides: [`uncertainty`, `tut-vccycle`],
    body: `Propagated uncertainty of X (resolved in a second solve pass)


## Syntax

\`\`\`
UncertaintyOf(X)
\`\`\`

## Description

Propagated uncertainty of X (resolved in a second solve pass)

## Mathematical Formulation

$$ u(X) = \\text{user-supplied or RSS-propagated uncertainty of } X $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`X\` | Number | Yes | Lockhart–Martinelli parameter. |

## References

1. JCGM 100:2008 — Evaluation of measurement data: Guide to the expression of uncertainty in measurement (GUM).`,
  },
  {
    name: `apart`,
    slug: `apart`,
    category: `CAS (REPL)`,
    summary: `Symbolic apart (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`apart`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`gs-repl`, `repl`],
    body: `Symbolic computer-algebra operation **apart**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
apart(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `cancel`,
    slug: `cancel`,
    category: `CAS (REPL)`,
    summary: `Symbolic cancel (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`cancel`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **cancel**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
cancel(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `collect`,
    slug: `collect`,
    category: `CAS (REPL)`,
    summary: `Symbolic collect (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`collect`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **collect**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
collect(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `denominator`,
    slug: `denominator`,
    category: `CAS (REPL)`,
    summary: `Symbolic denominator (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`denominator`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **denominator**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
denominator(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `diff`,
    slug: `diff`,
    category: `CAS (REPL)`,
    summary: `Symbolic diff (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`diff`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **diff**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
diff(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `expand`,
    slug: `expand`,
    category: `CAS (REPL)`,
    summary: `Symbolic expand (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`expand`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **expand**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
expand(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `factor`,
    slug: `factor`,
    category: `CAS (REPL)`,
    summary: `Symbolic factor (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`factor`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`gs-repl`, `repl`],
    body: `Symbolic computer-algebra operation **factor**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
factor(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `ilaplace`,
    slug: `ilaplace`,
    category: `CAS (REPL)`,
    summary: `Symbolic ilaplace (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`ilaplace`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [],
    body: `Symbolic computer-algebra operation **ilaplace**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
ilaplace(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `integrate`,
    slug: `integrate`,
    category: `CAS (REPL)`,
    summary: `Symbolic integrate (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`integrate`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **integrate**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
integrate(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `inverselaplace`,
    slug: `inverselaplace`,
    category: `CAS (REPL)`,
    summary: `Symbolic inverselaplace (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`inverselaplace`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **inverselaplace**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
inverselaplace(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `laplace`,
    slug: `laplace`,
    category: `CAS (REPL)`,
    summary: `Symbolic laplace (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`laplace`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`gs-repl`, `repl`],
    body: `Symbolic computer-algebra operation **laplace**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
laplace(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `numerator`,
    slug: `numerator`,
    category: `CAS (REPL)`,
    summary: `Symbolic numerator (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`numerator`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **numerator**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
numerator(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `simplify`,
    slug: `simplify`,
    category: `CAS (REPL)`,
    summary: `Symbolic simplify (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`simplify`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **simplify**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
simplify(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `together`,
    slug: `together`,
    category: `CAS (REPL)`,
    summary: `Symbolic together (REPL-only Symja CAS operation).`,
    related: [],
    examples: [],
    tags: [`together`, `cas`, `symbolic`, `repl`],
    references: [],
    guides: [`repl`],
    body: `Symbolic computer-algebra operation **together**, available in the REPL terminal (Symja backend).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
together(expr)
\`\`\`

## Description

A REPL-only symbolic transform — it operates on an algebraic expression rather than a solved numeric value, so it is not available in the editor document body.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`expr\` | Number | Yes | Expression to evaluate. |`,
  },
  {
    name: `AdiabaticFlameTemp`,
    slug: `adiabaticflametemp`,
    category: `Combustion`,
    summary: `Constant-pressure adiabatic flame temperature of a fuel-air mixture.`,
    related: [`AdiabaticFlameTempEq`, `mix_enthalpy`, `wiebe_rate`],
    examples: [`adiabatic-flame-temp`],
    tags: [`combustion`, `adiabatic flame temperature`, `energy balance`, `equivalence ratio`, `methane`],
    references: [],
    guides: [],
    body: `Returns the **constant-pressure adiabatic flame temperature** \`T_ad\` [K] of a fuel
burning in air at equivalence ratio \`phi\`, with reactants entering at \`T_react\`.
It is the temperature the products reach when all the combustion energy goes into
heating them (no heat loss, no work). This variant uses **frozen** complete
products (no dissociation) — see \`AdiabaticFlameTempEq\` for the dissociating case.

## Syntax

\`\`\`
T_ad = AdiabaticFlameTemp(fuel$, phi, T_react)
\`\`\`

## Description

\`phi = 1\` is stoichiometric; \`phi < 1\` is lean (excess air, which dilutes and
lowers the flame temperature). The result is the upper bound on combustor
temperature for the given mixture.

## Mathematical Formulation

For an adiabatic, constant-pressure combustor with no work, the first law reduces
to equal reactant and product enthalpies — solve for \`T_ad\`:

$$ H_{\\text{react}}(T_{\\text{react}}) = H_{\\text{prod}}(T_{\\text{ad}}) $$

i.e.

$$ \\sum_{\\text{react}} N_i\\big(\\bar h_f^\\circ + \\Delta\\bar h(T_{\\text{react}})\\big)_i = \\sum_{\\text{prod}} N_j\\big(\\bar h_f^\\circ + \\Delta\\bar h(T_{\\text{ad}})\\big)_j $$

where $\\bar h_f^\\circ$ is the enthalpy of formation and $\\Delta\\bar h(T)$ the
sensible enthalpy (here from NASA-7 polynomials).

> **Method:** root-find \`T_ad\` so the product enthalpy matches the reactant
> enthalpy; products are the complete (frozen) combustion species.

## Examples

### Example 1 — Stoichiometric methane–air flame

[Run: adiabatic-flame-temp]

**Expected (approx.):** for \`CH4\`, \`phi = 1\`, reactants at 298.15 K,
\`T_flame ≈ 2300 K\` (frozen products; dissociation would lower this by ~100–150 K).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fuel$\` | String | Yes | Fuel name/formula (e.g. \`'CH4'\`). |
| \`phi\` | Number | Yes | Equivalence ratio (1 = stoichiometric, < 1 = lean). |
| \`T_react\` | Number | Yes | Reactant inlet temperature [K]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`T_ad\` | Number | Adiabatic flame temperature [K]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_FUEL\` | \`fuel$\` has no thermo data | Use a fuel present in the NASA-7 species set. |`,
  },
  {
    name: `adiabaticflametempeq`,
    slug: `adiabaticflametempeq`,
    category: `Combustion`,
    summary: `Adiabatic flame temperature with dissociation [K]`,
    related: [],
    examples: [],
    tags: [`adiabaticflametempeq`, `combustion`],
    references: [],
    guides: [],
    body: `Adiabatic flame temperature with dissociation [K]


## Syntax

\`\`\`
AdiabaticFlameTempEq(fuel$, phi, T_react, P)
\`\`\`

## Description

Adiabatic flame temperature with dissociation [K]

## Mathematical Formulation

$$ H_{\\text{react}}(T_r) = H_{\\text{prod}}(T_{ad}) \\quad\\text{with equilibrium dissociation at } (T_{ad}, P) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fuel$\` | String | Yes | Fuel name/formula (e.g. 'CH4'). |
| \`phi\` | Number | Yes | Equivalence ratio (1 = stoichiometric). |
| \`T_react\` | Number | Yes | Reactant inlet temperature [K]. |
| \`P\` | Number | Yes | Pressure [Pa]. |`,
  },
  {
    name: `eq_molefraction`,
    slug: `eq_molefraction`,
    category: `Combustion`,
    summary: `Equilibrium product mole fraction (dissociation)`,
    related: [],
    examples: [],
    tags: [`eq`, `molefraction`, `combustion`],
    references: [],
    guides: [],
    body: `Equilibrium product mole fraction (dissociation)


## Syntax

\`\`\`
eq_molefraction(fuel$, phi, T, P, species$)
\`\`\`

## Description

Equilibrium product mole fraction (dissociation)

## Mathematical Formulation

$$ \\text{species mole fraction from chemical equilibrium } \\big(\\min G \\text{ at } T, P\\big) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fuel$\` | String | Yes | Fuel name/formula (e.g. 'CH4'). |
| \`phi\` | Number | Yes | Equivalence ratio (1 = stoichiometric). |
| \`T\` | Number | Yes | Temperature [K]. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`species$\` | String | Yes | Product species name (e.g. CO, NO). |`,
  },
  {
    name: `mix_conductivity`,
    slug: `mix_conductivity`,
    category: `Combustion`,
    summary: `Ideal-gas mixture conductivity [W/m-K]`,
    related: [],
    examples: [],
    tags: [`mix`, `conductivity`, `combustion`],
    references: [],
    guides: [],
    body: `Ideal-gas mixture conductivity [W/m-K]


## Syntax

\`\`\`
mix_conductivity(comp$, T)
\`\`\`

## Description

Ideal-gas mixture conductivity [W/m-K]

## Mathematical Formulation

$$ \\lambda = \\sum_i \\frac{y_i \\lambda_i}{\\sum_j y_j \\phi_{ij}} \\quad\\text{(Wassiljewa/Wilke)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`comp$\` | String | Yes | Mixture composition string, e.g. 'N2:0.79,O2:0.21'. |
| \`T\` | Number | Yes | Temperature [K]. |`,
  },
  {
    name: `mix_cp`,
    slug: `mix_cp`,
    category: `Combustion`,
    summary: `Ideal-gas mixture cp [J/kg-K]`,
    related: [],
    examples: [],
    tags: [`mix`, `cp`, `combustion`],
    references: [],
    guides: [],
    body: `Ideal-gas mixture cp [J/kg-K]


## Syntax

\`\`\`
mix_cp(comp$, T)
\`\`\`

## Description

Ideal-gas mixture cp [J/kg-K]

## Mathematical Formulation

$$ c_p = \\sum_i Y_i\\,c_{p,i}(T) \\quad\\text{(mass-weighted, NASA-7)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`comp$\` | String | Yes | Mixture composition string, e.g. 'N2:0.79,O2:0.21'. |
| \`T\` | Number | Yes | Temperature [K]. |`,
  },
  {
    name: `mix_enthalpy`,
    slug: `mix_enthalpy`,
    category: `Combustion`,
    summary: `Ideal-gas mixture enthalpy [J/kg]`,
    related: [],
    examples: [],
    tags: [`mix`, `enthalpy`, `combustion`],
    references: [],
    guides: [],
    body: `Ideal-gas mixture enthalpy [J/kg]


## Syntax

\`\`\`
mix_enthalpy(comp$, T)
\`\`\`

## Description

Ideal-gas mixture enthalpy [J/kg]

## Mathematical Formulation

$$ h = \\sum_i Y_i\\,h_i(T) \\quad\\text{(NASA-7 polynomials)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`comp$\` | String | Yes | Mixture composition string, e.g. 'N2:0.79,O2:0.21'. |
| \`T\` | Number | Yes | Temperature [K]. |`,
  },
  {
    name: `mix_entropy`,
    slug: `mix_entropy`,
    category: `Combustion`,
    summary: `Ideal-gas mixture entropy [J/kg-K]`,
    related: [],
    examples: [],
    tags: [`mix`, `entropy`, `combustion`],
    references: [],
    guides: [],
    body: `Ideal-gas mixture entropy [J/kg-K]


## Syntax

\`\`\`
mix_entropy(comp$, T, P)
\`\`\`

## Description

Ideal-gas mixture entropy [J/kg-K]

## Mathematical Formulation

$$ s = \\sum_i Y_i\\big[s_i(T) - R_i\\ln(y_i P/P_0)\\big] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`comp$\` | String | Yes | Mixture composition string, e.g. 'N2:0.79,O2:0.21'. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`P\` | Number | Yes | Pressure [Pa]. |`,
  },
  {
    name: `mix_mw`,
    slug: `mix_mw`,
    category: `Combustion`,
    summary: `Ideal-gas mixture molar mass [kg/mol], comp 'N2:0.79,O2:0.21`,
    related: [],
    examples: [],
    tags: [`mix`, `mw`, `combustion`],
    references: [],
    guides: [],
    body: `Ideal-gas mixture molar mass [kg/mol], comp 'N2:0.79,O2:0.21'


## Syntax

\`\`\`
mix_mw(comp$)
\`\`\`

## Description

Ideal-gas mixture molar mass [kg/mol], comp 'N2:0.79,O2:0.21'

## Mathematical Formulation

$$ \\overline{M} = \\sum_i y_i M_i $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`comp$\` | String | Yes | Mixture composition string, e.g. 'N2:0.79,O2:0.21'. |`,
  },
  {
    name: `mix_viscosity`,
    slug: `mix_viscosity`,
    category: `Combustion`,
    summary: `Ideal-gas mixture viscosity [Pa-s] (Chapman-Enskog/Wilke)`,
    related: [],
    examples: [],
    tags: [`mix`, `viscosity`, `combustion`],
    references: [],
    guides: [],
    body: `Ideal-gas mixture viscosity [Pa-s] (Chapman-Enskog/Wilke)


## Syntax

\`\`\`
mix_viscosity(comp$, T)
\`\`\`

## Description

Ideal-gas mixture viscosity [Pa-s] (Chapman-Enskog/Wilke)

## Mathematical Formulation

$$ \\mu = \\sum_i \\frac{y_i \\mu_i}{\\sum_j y_j \\phi_{ij}} \\quad\\text{(Wilke)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`comp$\` | String | Yes | Mixture composition string, e.g. 'N2:0.79,O2:0.21'. |
| \`T\` | Number | Yes | Temperature [K]. |`,
  },
  {
    name: `wiebe`,
    slug: `wiebe`,
    category: `Combustion`,
    summary: `Wiebe burned mass fraction`,
    related: [],
    examples: [],
    tags: [`wiebe`, `combustion`],
    references: [],
    guides: [],
    body: `Wiebe burned mass fraction


## Syntax

\`\`\`
wiebe(theta, theta0, dtheta, a, m)
\`\`\`

## Description

Wiebe burned mass fraction

## Mathematical Formulation

$$ x_b(\\theta) = 1 - \\exp\\!\\left[-a\\left(\\frac{\\theta-\\theta_0}{\\Delta\\theta}\\right)^{m+1}\\right] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`theta\` | Number | Yes | Flow-deflection angle [rad]. |
| \`theta0\` | Number | Yes | Start of combustion [deg]. |
| \`dtheta\` | Number | Yes | Combustion duration [deg]. |
| \`a\` | Number | Yes | First operand. |
| \`m\` | Number | Yes | Shape / form parameter. |`,
  },
  {
    name: `wiebe_rate`,
    slug: `wiebe_rate`,
    category: `Combustion`,
    summary: `Wiebe heat-release rate dxb/dθ for an engine combustion model.`,
    related: [`wiebe`, `AdiabaticFlameTemp`],
    examples: [`engine-cycle-wiebe`],
    tags: [`combustion`, `engine`, `wiebe`, `vibe`, `heat release`, `burn rate`, `crank angle`],
    references: [],
    guides: [],
    body: `Returns the **Wiebe (Vibe) burn rate** \`dxb/dθ\` — the rate of change of burned mass
fraction with crank angle — for a single-zone engine heat-release model. Multiply
by the total heat release to get the instantaneous heat-release rate \`dQ/dθ\` that
drives the cylinder-pressure trace.

## Syntax

\`\`\`
rate = wiebe_rate(theta, theta0, dtheta, a, m)
\`\`\`

## Description

The Wiebe function is the standard empirical S-curve for the cumulative mass-fraction
burned over a combustion event; its derivative is the bell-shaped heat-release rate.
\`theta0\` is the start of combustion, \`dtheta\` the burn duration, \`a\` the efficiency
parameter (≈ 5 for ~99% completion), and \`m\` the form factor (≈ 2 for SI engines).

## Mathematical Formulation

Burned mass fraction and its rate:

$$ x_b(\\theta) = 1 - \\exp\\!\\left[-a\\left(\\frac{\\theta-\\theta_0}{\\Delta\\theta}\\right)^{m+1}\\right] $$

$$ \\frac{dx_b}{d\\theta} = \\frac{a(m+1)}{\\Delta\\theta}\\left(\\frac{\\theta-\\theta_0}{\\Delta\\theta}\\right)^{m}\\exp\\!\\left[-a\\left(\\frac{\\theta-\\theta_0}{\\Delta\\theta}\\right)^{m+1}\\right] $$

> **Method:** direct evaluation; the rate is zero before \`theta0\` and decays to
> zero as the burn completes.

## Examples

### Example 1 — SI-engine heat release over crank angle

The single-zone cycle integrates \`dQ/dθ = Q_tot · wiebe_rate(θ, θ_soc, θ_dur, 5, 2)\`
to build the cylinder-pressure trace.

[Run: engine-cycle-wiebe]

**Expected:** a bell-shaped release peaking partway through the burn duration
(\`a = 5\`, \`m = 2\`), zero outside \`[θ_soc, θ_soc + θ_dur]\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`theta\` | Number | Yes | Current crank angle [deg]. |
| \`theta0\` | Number | Yes | Start of combustion [deg]. |
| \`dtheta\` | Number | Yes | Burn duration [deg]. |
| \`a\` | Number | Yes | Efficiency parameter (≈ 5 for ~99% completion). |
| \`m\` | Number | Yes | Form factor (≈ 2 for SI engines). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`rate\` | Number | Burn rate dxb/dθ [1/deg]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| (zero result) | \`theta < theta0\` | The rate is zero before combustion starts — expected. |`,
  },
  {
    name: `angle`,
    slug: `angle`,
    category: `Complex`,
    summary: `Argument of z [rad] (alias anglerad)`,
    related: [],
    examples: [],
    tags: [`angle`, `complex`],
    references: [],
    guides: [`complex`],
    body: `Argument of z [rad] (alias anglerad)


## Syntax

\`\`\`
angle(z)
\`\`\`

## Description

Argument of z [rad] (alias anglerad)

## Mathematical Formulation

$$ \\arg(z) = \\operatorname{atan2}(b, a)\\ \\ [\\text{rad}] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`z\` | Number | Yes | Argument (complex or real). |`,
  },
  {
    name: `angledeg`,
    slug: `angledeg`,
    category: `Complex`,
    summary: `Argument of z [deg]`,
    related: [],
    examples: [],
    tags: [`angledeg`, `complex`],
    references: [],
    guides: [`complex`],
    body: `Argument of z [deg]


## Syntax

\`\`\`
angledeg(z)
\`\`\`

## Description

Argument of z [deg]

## Mathematical Formulation

$$ \\arg(z) = \\operatorname{atan2}(b, a)\\cdot\\tfrac{180}{\\pi}\\ \\ [\\text{deg}] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`z\` | Number | Yes | Argument (complex or real). |`,
  },
  {
    name: `cis`,
    slug: `cis`,
    category: `Complex`,
    summary: `e^(j*theta) = cos(theta) + j*sin(theta)`,
    related: [],
    examples: [],
    tags: [`cis`, `complex`],
    references: [],
    guides: [`complex`],
    body: `e^(j*theta) = cos(theta) + j*sin(theta)


## Syntax

\`\`\`
cis(theta)
\`\`\`

## Description

e^(j*theta) = cos(theta) + j*sin(theta)

## Mathematical Formulation

$$ \\operatorname{cis}(\\theta) = e^{j\\theta} = \\cos\\theta + j\\sin\\theta $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`theta\` | Number | Yes | Flow-deflection angle [rad]. |`,
  },
  {
    name: `conj`,
    slug: `conj`,
    category: `Complex`,
    summary: `Complex conjugate`,
    related: [],
    examples: [],
    tags: [`conj`, `complex`],
    references: [],
    guides: [`complex`],
    body: `Complex conjugate


## Syntax

\`\`\`
conj(z)
\`\`\`

## Description

Complex conjugate

## Mathematical Formulation

$$ \\bar z = \\overline{a + jb} = a - jb $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`z\` | Number | Yes | Argument (complex or real). |`,
  },
  {
    name: `imag`,
    slug: `imag`,
    category: `Complex`,
    summary: `Imaginary part of a complex value`,
    related: [],
    examples: [],
    tags: [`imag`, `complex`],
    references: [],
    guides: [`complex`],
    body: `Imaginary part of a complex value


## Syntax

\`\`\`
imag(z)
\`\`\`

## Description

Imaginary part of a complex value

## Mathematical Formulation

$$ \\Im(z) = \\Im(a + jb) = b $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`z\` | Number | Yes | Argument (complex or real). |`,
  },
  {
    name: `magnitude`,
    slug: `magnitude`,
    category: `Complex`,
    summary: `Modulus |z|`,
    related: [],
    examples: [],
    tags: [`magnitude`, `complex`],
    references: [],
    guides: [`complex`],
    body: `Modulus |z|


## Syntax

\`\`\`
magnitude(z)
\`\`\`

## Description

Modulus |z|

## Mathematical Formulation

$$ |z| = \\sqrt{a^2 + b^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`z\` | Number | Yes | Argument (complex or real). |`,
  },
  {
    name: `real`,
    slug: `real`,
    category: `Complex`,
    summary: `Real part of a complex value`,
    related: [],
    examples: [],
    tags: [`real`, `complex`],
    references: [],
    guides: [`complex`],
    body: `Real part of a complex value


## Syntax

\`\`\`
real(z)
\`\`\`

## Description

Real part of a complex value

## Mathematical Formulation

$$ \\Re(z) = \\Re(a + jb) = a $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`z\` | Number | Yes | Argument (complex or real). |`,
  },
  {
    name: `AirCoil`,
    slug: `aircoil`,
    category: `Component (ac)`,
    summary: `An air-to-refrigerant coil (the air side of an evaporator or condenser).`,
    related: [],
    examples: [],
    tags: [`aircoil`, `component`, `ac`, `acausal`],
    references: [],
    guides: [],
    body: `An air-to-refrigerant coil (the air side of an evaporator or condenser).

## Domain

A reusable **acausal ac-domain** component — its refrigerant/air ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`ref_in\`, \`ref_out\`, \`air_in\`, \`air_out\`

## Usage

\`\`\`
AirCoil inst(ref$, U_tp, U_sh, D, L, eps_zone, eps_air)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`ref$\` | String | Refrigerant name (e.g. R134a, R1234yf). |
| \`U_tp\` | Number | Two-phase-zone overall coefficient [W/m²·K]. |
| \`U_sh\` | Number | Superheat-zone overall coefficient [W/m²·K]. |
| \`D\` | Number | Diameter [m]. |
| \`L\` | Number | Length [m]. |
| \`eps_zone\` | Number | Zone-collapse smoothing width. |
| \`eps_air\` | Number | Air-side effectiveness. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
MovingBoundaryEvaporator EV(fluid$=ref$, U_tp=U_tp, U_sh=U_sh, D=D, L=L, eps_zone=eps_zone)
MoistAirWallHX AC(eps=eps_air)
connect(ref_in, EV.in)
connect(EV.out, ref_out)
connect(air_in, AC.in)
connect(AC.out, air_out)
connect(EV.wall, AC.wall)
\`\`\``,
  },
  {
    name: `Chiller`,
    slug: `chiller`,
    category: `Component (ac)`,
    summary: `A refrigerant-to-coolant chiller transferring heat between the two loops.`,
    related: [],
    examples: [],
    tags: [`chiller`, `component`, `ac`, `acausal`],
    references: [],
    guides: [],
    body: `A refrigerant-to-coolant chiller transferring heat between the two loops.

## Domain

A reusable **acausal ac-domain** component — its refrigerant/air ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`ref_in\`, \`ref_out\`, \`cool_in\`, \`cool_out\`

## Usage

\`\`\`
Chiller inst(ref$, cool$, U_tp, U_sh, D, L, eps_zone, UA_cool)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`ref$\` | String | Refrigerant name (e.g. R134a, R1234yf). |
| \`cool$\` | String | Coolant name (e.g. EG50, Water). |
| \`U_tp\` | Number | Two-phase-zone overall coefficient [W/m²·K]. |
| \`U_sh\` | Number | Superheat-zone overall coefficient [W/m²·K]. |
| \`D\` | Number | Diameter [m]. |
| \`L\` | Number | Length [m]. |
| \`eps_zone\` | Number | Zone-collapse smoothing width. |
| \`UA_cool\` | Number | Coolant-side conductance [W/K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
MovingBoundaryEvaporator EV(fluid$=ref$, U_tp=U_tp, U_sh=U_sh, D=D, L=L, eps_zone=eps_zone)
LiquidWallHX CL(fluid$=cool$, UA=UA_cool)
connect(ref_in, EV.in)
connect(EV.out, ref_out)
connect(cool_in, CL.in)
connect(CL.out, cool_out)
connect(EV.wall, CL.wall)
\`\`\``,
  },
  {
    name: `EXV`,
    slug: `exv`,
    category: `Component (ac)`,
    summary: `An electronic expansion valve with a commanded opening.`,
    related: [],
    examples: [],
    tags: [`exv`, `component`, `ac`, `acausal`],
    references: [],
    guides: [],
    body: `An electronic expansion valve with a commanded opening.

## Domain

A reusable **acausal ac-domain** component — its refrigerant/air ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
EXV inst(fluid$, CdA_max, u, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`CdA_max\` | Number | Maximum Cd·A [m²]. |
| \`u\` | Number | Specific internal energy [J/kg]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
rho_in   = Density(fluid$, P=in.P, h=in.h)
in.mdot * abs(in.mdot) = (u * CdA_max)^2 * 2 * rho_in * (in.P - out.P)
\`\`\``,
  },
  {
    name: `EXVCmd`,
    slug: `exvcmd`,
    category: `Component (ac)`,
    summary: `Acausal ac-domain component EXVCmd with ports in, out, u.`,
    related: [],
    examples: [],
    tags: [`exvcmd`, `component`, `ac`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **ac-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
EXVCmd inst(fluid$, CdA_max, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`CdA_max\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
rho_in   = Density(fluid$, P=in.P, h=in.h)
in.mdot * abs(in.mdot) = (u.sig * CdA_max)^2 * 2 * rho_in * (in.P - out.P)
\`\`\``,
  },
  {
    name: `HeaterCore`,
    slug: `heatercore`,
    category: `Component (ac)`,
    summary: `Acausal ac-domain component HeaterCore with ports cool_in, cool_out, air_in, air_out.`,
    related: [],
    examples: [],
    tags: [`heatercore`, `component`, `ac`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **ac-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HeaterCore inst(cool$, UA_cool, eps_air)
\`\`\`

## Ports

\`cool_in\`, \`cool_out\`, \`air_in\`, \`air_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`cool$\` | String |
| \`UA_cool\` | Number |
| \`eps_air\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
LiquidWallHX   CL(fluid$=cool$, UA=UA_cool)
MoistAirWallHX AR(model$=eps_t, eps=eps_air)
connect(cool_in, CL.in)
connect(CL.out, cool_out)
connect(air_in, AR.in)
connect(AR.out, air_out)
connect(CL.wall, AR.wall)
\`\`\``,
  },
  {
    name: `Radiator`,
    slug: `radiator`,
    category: `Component (ac)`,
    summary: `Acausal ac-domain component Radiator with ports cool_in, cool_out, air_in, air_out.`,
    related: [],
    examples: [],
    tags: [`radiator`, `component`, `ac`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **ac-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Radiator inst(cool$, UA_cool, eps_air)
\`\`\`

## Ports

\`cool_in\`, \`cool_out\`, \`air_in\`, \`air_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`cool$\` | String |
| \`UA_cool\` | Number |
| \`eps_air\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
LiquidWallHX   CL(fluid$=cool$, UA=UA_cool)
MoistAirWallHX AR(model$=eps_t, eps=eps_air)
connect(cool_in, CL.in)
connect(CL.out, cool_out)
connect(air_in, AR.in)
connect(AR.out, air_out)
connect(CL.wall, AR.wall)
\`\`\``,
  },
  {
    name: `TXV`,
    slug: `txv`,
    category: `Component (ac)`,
    summary: `A thermostatic expansion valve that meters refrigerant to hold a target superheat.`,
    related: [],
    examples: [],
    tags: [`txv`, `component`, `ac`, `acausal`],
    references: [],
    guides: [],
    body: `A thermostatic expansion valve that meters refrigerant to hold a target superheat.

## Domain

A reusable **acausal ac-domain** component — its refrigerant/air ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`bulb\`

## Usage

\`\`\`
TXV inst(fluid$, Kv, SH_set, CdA0, tau_valve, tau_bulb, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`Kv\` | Number | Flow coefficient. |
| \`SH_set\` | Number | Target superheat [K]. |
| \`CdA0\` | Number | Reference Cd·A [m²]. |
| \`tau_valve\` | Number | Valve time constant [s]. |
| \`tau_bulb\` | Number | Bulb time constant [s]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot   = in.mdot
out.h      = in.h
bulb.Qdot  = 0
Tsat       = T_sat(fluid$, P=out.P)
SH_sensed  = bulb.T - Tsat
der(SH_b)  = (SH_sensed - SH_b) / tau_bulb
init(SH_b) = SH_set
CdA_t      = CdA0 + Kv * (SH_b - SH_set)
der(CdA)   = (CdA_t - CdA) / tau_valve
init(CdA)  = CdA0
rho_in     = Density(fluid$, P=in.P, h=in.h)
in.mdot * abs(in.mdot) = CdA^2 * 2 * rho_in * (in.P - out.P)
\`\`\``,
  },
  {
    name: `PIThermostat`,
    slug: `pithermostat`,
    category: `Component (control)`,
    summary: `A proportional–integral thermostat controller driving an actuator to a setpoint.`,
    related: [],
    examples: [],
    tags: [`pithermostat`, `component`, `control`, `acausal`],
    references: [],
    guides: [],
    body: `A proportional–integral thermostat controller driving an actuator to a setpoint.

## Domain

A reusable **acausal control-domain** component — its signal ports carry the measured and commanded scalar values. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
PIThermostat inst(Kp, Ki, Tref)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Kp\` | Number | Proportional gain. |
| \`Ki\` | Number | Integral gain. |
| \`Tref\` | Number | Reference (setpoint) temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
err         = Tref - port.T
der(integ)  = err
init(integ) = 0
port.Qdot   = -(Kp * err + Ki * integ)
\`\`\``,
  },
  {
    name: `Battery`,
    slug: `battery`,
    category: `Component (electrical)`,
    summary: `An electrical battery modeled as an EMF in series with an internal resistance.`,
    related: [],
    examples: [],
    tags: [`battery`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `An electrical battery modeled as an EMF in series with an internal resistance.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`

## Usage

\`\`\`
Battery inst(Voc, R0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Voc\` | Number | Open-circuit voltage [V]. |
| \`R0\` | Number | Series (ohmic) resistance [Ω]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.V - n.V = Voc + R0 * p.I
p.I + n.I = 0
W = (p.V - n.V) * (0 - p.I)
\`\`\``,
  },
  {
    name: `Battery2RC`,
    slug: `battery2rc`,
    category: `Component (electrical)`,
    summary: `A battery with two RC branches for second-order transient terminal behavior.`,
    related: [],
    examples: [],
    tags: [`battery2rc`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A battery with two RC branches for second-order transient terminal behavior.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`

## Usage

\`\`\`
Battery2RC inst(Voc, R0, R1, C1, R2, C2, Vrc1_0, Vrc2_0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Voc\` | Number | Open-circuit voltage [V]. |
| \`R0\` | Number | Series (ohmic) resistance [Ω]. |
| \`R1\` | Number | First RC-branch resistance [Ω]. |
| \`C1\` | Number | First RC-branch capacitance [F]. |
| \`R2\` | Number | Second RC-branch resistance [Ω]. |
| \`C2\` | Number | Second RC-branch capacitance [F]. |
| \`Vrc1_0\` | Number | Initial first-RC voltage [V]. |
| \`Vrc2_0\` | Number | Initial second-RC voltage [V]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.V - n.V  = Voc + R0 * p.I - Vrc1 - Vrc2
der(Vrc1)  = -p.I / C1 - Vrc1 / (R1 * C1)
init(Vrc1) = Vrc1_0
der(Vrc2)  = -p.I / C2 - Vrc2 / (R2 * C2)
init(Vrc2) = Vrc2_0
p.I + n.I  = 0
\`\`\``,
  },
  {
    name: `BatteryCellMap`,
    slug: `batterycellmap`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component BatteryCellMap with ports p, n, heat.`,
    related: [],
    examples: [],
    tags: [`batterycellmap`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
BatteryCellMap inst(ocv$, dudt$, R0ref, Tref, Ea, Q0, C_th, SOC0, T0, k_age, model$)
\`\`\`

## Ports

\`p\`, \`n\`, \`heat\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ocv$\` | String |
| \`dudt$\` | String |
| \`R0ref\` | Number |
| \`Tref\` | Number |
| \`Ea\` | Number |
| \`Q0\` | Number |
| \`C_th\` | Number |
| \`SOC0\` | Number |
| \`T0\` | Number |
| \`k_age\` | Number |
| \`model$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
I         = -p.I
R0        = R0ref * exp((Ea / 8.314) * (1 / T - 1 / Tref))
Voc       = ocv$(SOC)
p.V - n.V = Voc - R0 * I
p.I + n.I = 0
der(SOC)  = -I / (3600 * Qcap)
init(SOC) = SOC0
Qgen      = R0 * I^2 - I * T * dudt$(SOC)
heat.T    = T
der(T)    = (Qgen + heat.Qdot) / C_th
init(T)   = T0
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`static\`

\`\`\`
Qcap = Q0
\`\`\`

### \`aging\` — requires \`k_age\`

\`\`\`
der(Ah)  = abs(I) / 3600
init(Ah) = 0
Qcap     = Q0 * (1 - k_age * Ah)
\`\`\``,
  },
  {
    name: `BatteryPack`,
    slug: `batterypack`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component BatteryPack with ports p, n, heat.`,
    related: [],
    examples: [],
    tags: [`batterypack`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
BatteryPack inst(Ns, Np, ocv$, dudt$, R0ref, Tref, Ea, Q0, C_th, SOC0, T0)
\`\`\`

## Ports

\`p\`, \`n\`, \`heat\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Ns\` | Number |
| \`Np\` | Number |
| \`ocv$\` | String |
| \`dudt$\` | String |
| \`R0ref\` | Number |
| \`Tref\` | Number |
| \`Ea\` | Number |
| \`Q0\` | Number |
| \`C_th\` | Number |
| \`SOC0\` | Number |
| \`T0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
I_cell    = -p.I / Np
R0        = R0ref * exp((Ea / 8.314) * (1 / T - 1 / Tref))
p.V - n.V = Ns * (ocv$(SOC) - R0 * I_cell)
p.I + n.I = 0
der(SOC)  = -I_cell / (3600 * Q0)
init(SOC) = SOC0
Qgen      = Ns * Np * (R0 * I_cell^2 - I_cell * T * dudt$(SOC))
heat.T    = T
der(T)    = (Qgen + heat.Qdot) / C_th
init(T)   = T0
\`\`\``,
  },
  {
    name: `BatteryRC`,
    slug: `batteryrc`,
    category: `Component (electrical)`,
    summary: `A battery with one RC branch for first-order transient terminal behavior.`,
    related: [],
    examples: [],
    tags: [`batteryrc`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A battery with one RC branch for first-order transient terminal behavior.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`

## Usage

\`\`\`
BatteryRC inst(Voc, R0, R1, C1, Vrc0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Voc\` | Number | Open-circuit voltage [V]. |
| \`R0\` | Number | Series (ohmic) resistance [Ω]. |
| \`R1\` | Number | First RC-branch resistance [Ω]. |
| \`C1\` | Number | First RC-branch capacitance [F]. |
| \`Vrc0\` | Number | Initial RC-branch voltage [V]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.V - n.V = Voc + R0 * p.I - Vrc
der(Vrc)  = -p.I / C1 - Vrc / (R1 * C1)
init(Vrc) = Vrc0
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `BatteryThermal`,
    slug: `batterythermal`,
    category: `Component (electrical)`,
    summary: `A battery with a coupled thermal model relating losses to temperature.`,
    related: [],
    examples: [],
    tags: [`batterythermal`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A battery with a coupled thermal model relating losses to temperature.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`, \`heat\`

## Usage

\`\`\`
BatteryThermal inst(Voc, R0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Voc\` | Number | Open-circuit voltage [V]. |
| \`R0\` | Number | Series (ohmic) resistance [Ω]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.V - n.V = Voc + R0 * p.I
p.I + n.I = 0
Q         = R0 * p.I^2
heat.Qdot = -Q
W         = (p.V - n.V) * (0 - p.I)
\`\`\``,
  },
  {
    name: `BatteryTransient`,
    slug: `batterytransient`,
    category: `Component (electrical)`,
    summary: `A transient battery model carrying state-of-charge dynamics.`,
    related: [],
    examples: [],
    tags: [`batterytransient`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A transient battery model carrying state-of-charge dynamics.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`, \`heat\`

## Usage

\`\`\`
BatteryTransient inst(Voc, R0, Q0, C_th, SOC0, T0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Voc\` | Number | Open-circuit voltage [V]. |
| \`R0\` | Number | Series (ohmic) resistance [Ω]. |
| \`Q0\` | Number | Reference heat [W]. |
| \`C_th\` | Number | Thermal capacitance [J/K]. |
| \`SOC0\` | Number | Initial state of charge (0–1). |
| \`T0\` | Number | Reference/initial temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.V - n.V = Voc + R0 * p.I
p.I + n.I = 0
Qgen      = R0 * p.I^2
heat.T    = T
der(T)    = (Qgen + heat.Qdot) / C_th
init(T)   = T0
der(SOC)  = p.I / (3600 * Q0)
init(SOC) = SOC0
\`\`\``,
  },
  {
    name: `Capacitor`,
    slug: `capacitor`,
    category: `Component (electrical)`,
    summary: `A capacitor storing charge, with i = C dV/dt.`,
    related: [],
    examples: [],
    tags: [`capacitor`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A capacitor storing charge, with \`i = C dV/dt\`.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`

## Usage

\`\`\`
Capacitor inst(C, V0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`C\` | Number | Capacitance [F]. |
| \`V0\` | Number | Initial voltage / volume. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Vc       = p.V - n.V
der(Vc)  = p.I / C
init(Vc) = V0
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `ChargerCCCV`,
    slug: `chargercccv`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component ChargerCCCV with ports p, n.`,
    related: [],
    examples: [],
    tags: [`chargercccv`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ChargerCCCV inst(Imax, Vmax, epsV)
\`\`\`

## Ports

\`p\`, \`n\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Imax\` | Number |
| \`Vmax\` | Number |
| \`epsV\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
V   = p.V - n.V
p.I = -Imax * 0.5 * (1 + tanh((Vmax - V) / epsV))
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `CurrentSource`,
    slug: `currentsource`,
    category: `Component (electrical)`,
    summary: `An ideal current source.`,
    related: [],
    examples: [],
    tags: [`currentsource`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `An ideal current source.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`

## Usage

\`\`\`
CurrentSource inst(I)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`I\` | Number | Current [A]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.I = -I
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `DCDCConverter`,
    slug: `dcdcconverter`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component DCDCConverter with ports in_p, in_n, out_p, out_n.`,
    related: [],
    examples: [],
    tags: [`dcdcconverter`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
DCDCConverter inst(ratio, eta, epsP)
\`\`\`

## Ports

\`in_p\`, \`in_n\`, \`out_p\`, \`out_n\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ratio\` | Number |
| \`eta\` | Number |
| \`epsP\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out_p.V - out_n.V = ratio * (in_p.V - in_n.V)
in_p.I + in_n.I   = 0
out_p.I + out_n.I = 0
Pout = (out_p.V - out_n.V) * (0 - out_p.I)
s    = 0.5 * (1 + tanh(Pout / epsP))
Pin  = s * Pout / eta + (1 - s) * Pout * eta
Pin  = (in_p.V - in_n.V) * in_p.I
\`\`\``,
  },
  {
    name: `DCMotor`,
    slug: `dcmotor`,
    category: `Component (electrical)`,
    summary: `A DC motor — an electrical-to-mechanical transducer (back-EMF and torque constants).`,
    related: [],
    examples: [],
    tags: [`dcmotor`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A DC motor — an electrical-to-mechanical transducer (back-EMF and torque constants).

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`, \`shaft\`

## Usage

\`\`\`
DCMotor inst(Kt, Ke, R)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Kt\` | Number | Torque constant [N·m/A]. |
| \`Ke\` | Number | Back-EMF constant [V·s/rad]. |
| \`R\` | Number | Resistance [Ω]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.V - n.V  = R * p.I + Ke * shaft.w
p.I + n.I  = 0
shaft.tau  = -Kt * p.I
\`\`\``,
  },
  {
    name: `Diode`,
    slug: `diode`,
    category: `Component (electrical)`,
    summary: `A nonlinear diode with an exponential current–voltage characteristic.`,
    related: [],
    examples: [],
    tags: [`diode`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A nonlinear diode with an exponential current–voltage characteristic.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`

## Usage

\`\`\`
Diode inst(Gon, eps)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Gon\` | Number | On-state conductance [S]. |
| \`eps\` | Number | Effectiveness / roughness. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
vd  = p.V - n.V
p.I = Gon * vd * (0.5 + 0.5 * tanh(vd / eps))
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `Electrolyzer`,
    slug: `electrolyzer`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component Electrolyzer with ports p, n, heat.`,
    related: [],
    examples: [],
    tags: [`electrolyzer`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Electrolyzer inst(ncells, area, i0, Rohm, E0, alpha, Eth, T)
\`\`\`

## Ports

\`p\`, \`n\`, \`heat\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ncells\` | Number |
| \`area\` | Number |
| \`i0\` | Number |
| \`Rohm\` | Number |
| \`E0\` | Number |
| \`alpha\` | Number |
| \`Eth\` | Number |
| \`T\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
I_cell    = p.I
i         = I_cell / area
V_cell    = E0 + (8.314 * T / (alpha * 96485)) * ln(i / i0 + 1) + i * Rohm
p.V - n.V = ncells * V_cell
p.I + n.I = 0
mdot_h2   = ncells * I_cell * 2.016e-3 / (2 * 96485)
Q         = I_cell * ncells * (V_cell - Eth)
heat.Qdot = -Q
\`\`\``,
  },
  {
    name: `ElectrolyzerThermal`,
    slug: `electrolyzerthermal`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component ElectrolyzerThermal with ports p, n, cool_in, cool_out.`,
    related: [],
    examples: [],
    tags: [`electrolyzerthermal`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ElectrolyzerThermal inst(ncells, area, i0, Rohm, E0, alpha, Eth, T, fluid$, UA)
\`\`\`

## Ports

\`p\`, \`n\`, \`cool_in\`, \`cool_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ncells\` | Number |
| \`area\` | Number |
| \`i0\` | Number |
| \`Rohm\` | Number |
| \`E0\` | Number |
| \`alpha\` | Number |
| \`Eth\` | Number |
| \`T\` | Number |
| \`fluid$\` | String |
| \`UA\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
Electrolyzer EL(ncells=ncells, area=area, i0=i0, Rohm=Rohm, E0=E0, alpha=alpha, Eth=Eth, T=T)
LiquidWallHX HX(fluid$=fluid$, UA=UA)
connect(p, EL.p)
connect(n, EL.n)
connect(EL.heat, HX.wall)
connect(cool_in, HX.in)
connect(HX.out, cool_out)
\`\`\``,
  },
  {
    name: `FuelCellStack`,
    slug: `fuelcellstack`,
    category: `Component (electrical)`,
    summary: `A PEM fuel-cell stack producing voltage from its polarization curve.`,
    related: [],
    examples: [],
    tags: [`fuelcellstack`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A PEM fuel-cell stack producing voltage from its polarization curve.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`, \`heat\`

## Usage

\`\`\`
FuelCellStack inst(ncells, area, i0, ilim, Rohm, E0, alpha, Eth, T)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`ncells\` | Number | Number of cells. |
| \`area\` | Number | Area [m²]. |
| \`i0\` | Number | Initial current [A]. |
| \`ilim\` | Number | Current limit [A]. |
| \`Rohm\` | Number | Ohmic resistance [Ω]. |
| \`E0\` | Number | Reference EMF [V]. |
| \`alpha\` | Number | Void fraction / coefficient. |
| \`Eth\` | Number | Activation/threshold energy. |
| \`T\` | Number | Temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
I_cell    = -p.I
i         = I_cell / area
V_cell    = E0 - (8.314 * T / (alpha * 96485)) * ln(i / i0) - i * Rohm - (8.314 * T / (2 * 96485)) * ln(ilim / (ilim - i))
p.V - n.V = ncells * V_cell
p.I + n.I = 0
Q         = I_cell * ncells * (Eth - V_cell)
heat.Qdot = -Q
\`\`\``,
  },
  {
    name: `FuelCellStackCooled`,
    slug: `fuelcellstackcooled`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component FuelCellStackCooled with ports p, n, cool_in, cool_out.`,
    related: [],
    examples: [],
    tags: [`fuelcellstackcooled`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
FuelCellStackCooled inst(ncells, area, i0, ilim, Rohm, E0, alpha, Eth, T, fluid$, UA)
\`\`\`

## Ports

\`p\`, \`n\`, \`cool_in\`, \`cool_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ncells\` | Number |
| \`area\` | Number |
| \`i0\` | Number |
| \`ilim\` | Number |
| \`Rohm\` | Number |
| \`E0\` | Number |
| \`alpha\` | Number |
| \`Eth\` | Number |
| \`T\` | Number |
| \`fluid$\` | String |
| \`UA\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
FuelCellStack FC(ncells=ncells, area=area, i0=i0, ilim=ilim, Rohm=Rohm, E0=E0, alpha=alpha, Eth=Eth, T=T)
LiquidWallHX  HX(fluid$=fluid$, UA=UA)
connect(p, FC.p)
connect(n, FC.n)
connect(FC.heat, HX.wall)
connect(cool_in, HX.in)
connect(HX.out, cool_out)
\`\`\``,
  },
  {
    name: `Ground`,
    slug: `ground`,
    category: `Component (electrical)`,
    summary: `The electrical reference node (V = 0).`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`ground`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `The electrical reference node (\`V = 0\`).

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
Ground inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
port.V = 0
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pressure-cooker]`,
  },
  {
    name: `HarnessResistance`,
    slug: `harnessresistance`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component HarnessResistance with ports a, b, heat.`,
    related: [],
    examples: [],
    tags: [`harnessresistance`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HarnessResistance inst(R20, alphaT)
\`\`\`

## Ports

\`a\`, \`b\`, \`heat\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`R20\` | Number |
| \`alphaT\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
R         = R20 * (1 + alphaT * (heat.T - 293.15))
a.V - b.V = R * a.I
a.I + b.I = 0
Q         = R * a.I^2
heat.Qdot = -Q
\`\`\``,
  },
  {
    name: `HeatingResistor`,
    slug: `heatingresistor`,
    category: `Component (electrical)`,
    summary: `A resistor that dissipates its electrical power as heat (electrical→thermal transducer).`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`heatingresistor`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A resistor that dissipates its electrical power as heat (electrical→thermal transducer).

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`, \`heat\`

## Usage

\`\`\`
HeatingResistor inst(R)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`R\` | Number | Resistance [Ω]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.V - n.V = R * p.I
p.I + n.I = 0
Q         = (p.V - n.V) * p.I
heat.Qdot = -Q
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pressure-cooker]`,
  },
  {
    name: `Inductor`,
    slug: `inductor`,
    category: `Component (electrical)`,
    summary: `An inductor storing magnetic energy, with V = L di/dt.`,
    related: [],
    examples: [],
    tags: [`inductor`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `An inductor storing magnetic energy, with \`V = L di/dt\`.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`

## Usage

\`\`\`
Inductor inst(L, I0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`L\` | Number | Length [m]. |
| \`I0\` | Number | Saturation current [A]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
der(IL)  = (p.V - n.V) / L
init(IL) = I0
p.I = IL
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `InverterLoss`,
    slug: `inverterloss`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component InverterLoss with ports in_p, out_p, heat.`,
    related: [],
    examples: [],
    tags: [`inverterloss`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
InverterLoss inst(V0, r, Esw, fsw, Iref, Vref, Vnom, epsI)
\`\`\`

## Ports

\`in_p\`, \`out_p\`, \`heat\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`V0\` | Number |
| \`r\` | Number |
| \`Esw\` | Number |
| \`fsw\` | Number |
| \`Iref\` | Number |
| \`Vref\` | Number |
| \`Vnom\` | Number |
| \`epsI\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
I         = in_p.I
in_p.I + out_p.I = 0
Vsw       = Esw * fsw * Vnom / (Iref * Vref)
dV        = (V0 + Vsw) * tanh(I / epsI) + r * I
out_p.V   = in_p.V - dV
Q         = dV * I
heat.Qdot = -Q
\`\`\``,
  },
  {
    name: `MotorMap`,
    slug: `motormap`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component MotorMap with ports p, n, shaft, heat, u.`,
    related: [],
    examples: [],
    tags: [`motormap`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
MotorMap inst(eff$, epsP)
\`\`\`

## Ports

\`p\`, \`n\`, \`shaft\`, \`heat\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff$\` | String |
| \`epsP\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
tau_e     = u.sig
shaft.tau = -tau_e
Pm        = tau_e * shaft.w
eff       = eff$(shaft.w, tau_e)
s         = 0.5 * (1 + tanh(Pm / epsP))
Pe        = s * Pm / eff + (1 - s) * Pm * eff
(p.V - n.V) * p.I = Pe
p.I + n.I = 0
Q         = Pe - Pm
heat.Qdot = -Q
\`\`\``,
  },
  {
    name: `MPPTBlock`,
    slug: `mpptblock`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component MPPTBlock with ports G, out.`,
    related: [],
    examples: [],
    tags: [`mpptblock`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
MPPTBlock inst(vmp$)
\`\`\`

## Ports

\`G\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`vmp$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = vmp$(G.sig)
\`\`\``,
  },
  {
    name: `PMSM`,
    slug: `pmsm`,
    category: `Component (electrical)`,
    summary: `A permanent-magnet synchronous motor.`,
    related: [],
    examples: [],
    tags: [`pmsm`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `A permanent-magnet synchronous motor.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`, \`shaft\`

## Usage

\`\`\`
PMSM inst(Rs, lambda_pm, poles)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Rs\` | Number | Series resistance [Ω]. |
| \`lambda_pm\` | Number | PM flux linkage [Wb]. |
| \`poles\` | Number | Number of magnetic pole pairs. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Kt        = 1.5 * poles * lambda_pm
p.V - n.V = Rs * p.I + Kt * shaft.w
p.I + n.I = 0
shaft.tau = -Kt * p.I
\`\`\``,
  },
  {
    name: `PVSingleDiode`,
    slug: `pvsinglediode`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component PVSingleDiode with ports p, n, G.`,
    related: [],
    examples: [],
    tags: [`pvsinglediode`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PVSingleDiode inst(Isc_ref, Gref, I0d, n_d, Vt, Rs, Rsh)
\`\`\`

## Ports

\`p\`, \`n\`, \`G\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Isc_ref\` | Number |
| \`Gref\` | Number |
| \`I0d\` | Number |
| \`n_d\` | Number |
| \`Vt\` | Number |
| \`Rs\` | Number |
| \`Rsh\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
V   = p.V - n.V
I   = -p.I
Iph = Isc_ref * G.sig / Gref
Vd  = V + I * Rs
I   = Iph - I0d * (exp(Vd / (n_d * Vt)) - 1) - Vd / Rsh
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `Resistor`,
    slug: `resistor`,
    category: `Component (electrical)`,
    summary: `An Ohmic resistor, V = R·I.`,
    related: [],
    examples: [],
    tags: [`resistor`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `An Ohmic resistor, \`V = R·I\`.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
Resistor inst(R)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`R\` | Number | Resistance [Ω]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
a.V - b.V = R * a.I
a.I + b.I = 0
\`\`\``,
  },
  {
    name: `SolarArray`,
    slug: `solararray`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component SolarArray with ports p, n, G.`,
    related: [],
    examples: [],
    tags: [`solararray`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SolarArray inst(Isc_ref, Gref, Voc, epsV)
\`\`\`

## Ports

\`p\`, \`n\`, \`G\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Isc_ref\` | Number |
| \`Gref\` | Number |
| \`Voc\` | Number |
| \`epsV\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
Iph = Isc_ref * G.sig / Gref
V   = p.V - n.V
p.I = -Iph * 0.5 * (1 + tanh((Voc - V) / epsV))
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `Supercapacitor`,
    slug: `supercapacitor`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component Supercapacitor with ports p, n.`,
    related: [],
    examples: [],
    tags: [`supercapacitor`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Supercapacitor inst(C, R_esr, V0)
\`\`\`

## Ports

\`p\`, \`n\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`C\` | Number |
| \`R_esr\` | Number |
| \`V0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(Vc)   = p.I / C
init(Vc)  = V0
p.V - n.V = Vc + R_esr * p.I
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `ThermalFuse`,
    slug: `thermalfuse`,
    category: `Component (electrical)`,
    summary: `Acausal electrical-domain component ThermalFuse with ports p, n.`,
    related: [],
    examples: [],
    tags: [`thermalfuse`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **electrical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ThermalFuse inst(R0, Iblow, kR, epsI)
\`\`\`

## Ports

\`p\`, \`n\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`R0\` | Number |
| \`Iblow\` | Number |
| \`kR\` | Number |
| \`epsI\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
m         = 1 + kR * 0.5 * (1 + tanh((abs(p.I) - Iblow) / epsI))
p.V - n.V = R0 * m * p.I
p.I + n.I = 0
\`\`\``,
  },
  {
    name: `VoltageSource`,
    slug: `voltagesource`,
    category: `Component (electrical)`,
    summary: `An ideal voltage source.`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`voltagesource`, `component`, `electrical`, `acausal`],
    references: [],
    guides: [],
    body: `An ideal voltage source.

## Domain

A reusable **acausal electrical-domain** component — its electrical ports carry potential \`V\` and current \`I\`; a node enforces equal \`V\` and \`ΣI = 0\` (Kirchhoff). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`p\`, \`n\`

## Usage

\`\`\`
VoltageSource inst(E)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`E\` | Number | EMF / voltage [V]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
p.V - n.V = E
p.I + n.I = 0
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pressure-cooker]`,
  },
  {
    name: `Accumulator`,
    slug: `accumulator`,
    category: `Component (fluid)`,
    summary: `A fluid accumulator — a compliance volume that stores fluid under pressure and buffers flow transients.`,
    related: [],
    examples: [],
    tags: [`accumulator`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A fluid accumulator — a compliance volume that stores fluid under pressure and buffers flow transients.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Accumulator inst(C, P0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`C\` | Number | Capacitance [F]. |
| \`P0\` | Number | Reference/initial pressure [Pa]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P       = in.P
out.h       = in.h
der(in.P)   = (in.mdot - out.mdot) / C
init(in.P)  = P0
\`\`\``,
  },
  {
    name: `AtmosphereSource`,
    slug: `atmospheresource`,
    category: `Component (fluid)`,
    summary: `Acausal fluid-domain component AtmosphereSource with ports out.`,
    related: [],
    examples: [],
    tags: [`atmospheresource`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **fluid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
AtmosphereSource inst(alt, mdot)
\`\`\`

## Ports

\`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`alt\` | Number |
| \`mdot\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = mdot
out.P    = isa_P(alt)
out.h    = Enthalpy(Air, P=isa_P(alt), T=isa_T(alt))
\`\`\``,
  },
  {
    name: `Boiler`,
    slug: `boiler`,
    category: `Component (fluid)`,
    summary: `Adds heat to a fluid stream, raising its enthalpy (and generating vapor at saturation).`,
    related: [],
    examples: [],
    tags: [`boiler`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Adds heat to a fluid stream, raising its enthalpy (and generating vapor at saturation).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Boiler inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
Q        = in.mdot * (out.h - in.h)
\`\`\``,
  },
  {
    name: `Combustor`,
    slug: `combustor`,
    category: `Component (fluid)`,
    summary: `Acausal fluid-domain component Combustor with ports in, out.`,
    related: [],
    examples: [],
    tags: [`combustor`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **fluid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Combustor inst(mdot_f, LHV, eta_b, dP)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`mdot_f\` | Number |
| \`LHV\` | Number |
| \`eta_b\` | Number |
| \`dP\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot + mdot_f
out.P    = in.P - dP
out.mdot * out.h = in.mdot * in.h + eta_b * mdot_f * LHV
\`\`\``,
  },
  {
    name: `CombustorSpecies`,
    slug: `combustorspecies`,
    category: `Component (fluid)`,
    summary: `Acausal fluid-domain component CombustorSpecies with ports in, out.`,
    related: [],
    examples: [],
    tags: [`combustorspecies`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **fluid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
CombustorSpecies inst(mdot_f, LHV, eta_b, dP, xC, yH, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`mdot_f\` | Number |
| \`LHV\` | Number |
| \`eta_b\` | Number |
| \`dP\` | Number |
| \`xC\` | Number |
| \`yH\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
mfuel    = 12 * xC + yH
mCO2     = mdot_f * 44 * xC / mfuel
mH2O     = mdot_f * 9 * yH / mfuel
mO2      = mdot_f * 32 * (xC + yH / 4) / mfuel
out.mdot = in.mdot + mdot_f
out.P    = in.P - dP
out.mdot * out.h    = in.mdot * in.h + eta_b * mdot_f * LHV
out.mdot * out.yco2 = in.mdot * in.yco2 + mCO2
out.mdot * out.yh2o = in.mdot * in.yh2o + mH2O
out.mdot * out.yo2  = in.mdot * in.yo2  - mO2
out.mdot * out.yn2  = in.mdot * in.yn2
\`\`\``,
  },
  {
    name: `Compressor`,
    slug: `compressor`,
    category: `Component (fluid)`,
    summary: `Raises the pressure of a fluid stream, computing the work from an isentropic efficiency.`,
    related: [`CompressorMap`, `TwoPhaseCompressor`],
    examples: [`ev-thermal-management`],
    tags: [`compressor`, `compressor-family`, `data:eta`, `ports:in-out`, `flow-closed`, `energy-work`, `steady`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Raises the pressure of a fluid stream, computing the work from an isentropic efficiency.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Compressor inst(eta, fluid$, model$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`eta\` | Number | Efficiency (0–1). |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`model$\` | String | Model variant — selects the physics body (see Model Variants). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
s_in     = Entropy(fluid$, P=in.P, h=in.h)
h_s      = Enthalpy(fluid$, P=out.P, s=s_in)
out.mdot = in.mdot
out.h    = in.h + (h_s - in.h) / eta
W        = in.mdot * (out.h - in.h)
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`isentropic\`

_No additional equations (uses the shared body)._

### \`volumetric\` — requires \`eta_v\`, \`disp\`, \`rpm\`

\`\`\`
rho_in  = Density(fluid$, P=in.P, h=in.h)
in.mdot = eta_v * disp * (rpm / 60) * rho_in
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `CompressorMap`,
    slug: `compressormap`,
    category: `Component (fluid)`,
    summary: `A compressor whose isentropic efficiency comes from a tabulated map (eta vs pressure ratio).`,
    related: [`Compressor`, `TwoPhaseCompressor`],
    examples: [],
    tags: [`compressor`, `compressor-family`, `compressormap`, `map`, `data:map-eta`, `ports:in-out`, `flow-closed`, `energy-work`, `steady`, `map-driven`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A compressor whose isentropic efficiency comes from a tabulated map (eta vs pressure ratio).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
CompressorMap inst(fluid$, map_eta$, model$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. R134a, Air). |
| \`map_eta$\` | String | Name of a TABLE/FUNCTION giving isentropic efficiency (0–1) vs pressure ratio (out.P/in.P). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
s_in     = Entropy(fluid$, P=in.P, h=in.h)
h_s      = Enthalpy(fluid$, P=out.P, s=s_in)
PR       = out.P / in.P
eta      = map_eta$(PR)
out.mdot = in.mdot
out.h    = in.h + (h_s - in.h) / eta
W        = in.mdot * (out.h - in.h)
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`eta\`

_No additional equations (uses the shared body; the through-flow is imposed by the surrounding network)._

### \`flow\` — requires \`map_mdot$\`

\`\`\`
in.mdot = map_mdot$(PR)
\`\`\`

The flow rung makes the machine a true flow-determining (R) element — the mass
flow comes from the pressure-ratio characteristic, so a supply → compressor →
volume chain is well-posed on every integrator.`,
  },
  {
    name: `Condenser`,
    slug: `condenser`,
    category: `Component (fluid)`,
    summary: `Rejects heat from a fluid stream to a coolant/ambient, condensing it.`,
    related: [],
    examples: [],
    tags: [`condenser`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Rejects heat from a fluid stream to a coolant/ambient, condensing it.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Condenser inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
Q        = in.mdot * (in.h - out.h)
\`\`\``,
  },
  {
    name: `Duct`,
    slug: `duct`,
    category: `Component (fluid)`,
    summary: `A flow passage that imposes a pressure drop on the stream.`,
    related: [],
    examples: [],
    tags: [`duct`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A flow passage that imposes a pressure drop on the stream.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Duct inst(rho, mu, L, D, rough)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`rho\` | Number | Density [kg/m³]. |
| \`mu\` | Number | Dynamic viscosity [Pa·s]. |
| \`L\` | Number | Length [m]. |
| \`D\` | Number | Diameter [m]. |
| \`rough\` | Number | Relative wall roughness. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
A        = pi# / 4 * D^2
V        = in.mdot / (rho * A)
Re_d     = reynolds(rho, V, D, mu)
f        = friction_factor(Re_d, rough / D)
out.P    = in.P - f * (L / D) * rho * V^2 / 2
\`\`\``,
  },
  {
    name: `ExpansionValve`,
    slug: `expansionvalve`,
    category: `Component (fluid)`,
    summary: `Throttles a fluid to a lower pressure isenthalpically (Joule–Thomson).`,
    related: [],
    examples: [],
    tags: [`expansionvalve`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Throttles a fluid to a lower pressure isenthalpically (Joule–Thomson).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
ExpansionValve inst(CdA, rho_in)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`CdA\` | Number | Discharge coefficient × area Cd·A [m²]. |
| \`rho_in\` | Number | Inlet density [kg/m³]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
in.mdot * abs(in.mdot) = CdA^2 * 2 * rho_in * (in.P - out.P)
\`\`\``,
  },
  {
    name: `Fan`,
    slug: `fan`,
    category: `Component (fluid)`,
    summary: `Adds a pressure rise to a gas/air stream, computing the fan work.`,
    related: [`FanCurve`, `FanMap`],
    examples: [],
    tags: [`fan`, `fan-family`, `constant-rise`, `data:dP0-Q0-eta`, `ports:in-out`, `flow-closed`, `energy-work`, `steady`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Adds a pressure rise to a gas/air stream, computing the fan work.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Fan inst(fluid$, dP0, Q0, eta)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`dP0\` | Number | Reference pressure drop [Pa]. |
| \`Q0\` | Number | Reference heat [W]. |
| \`eta\` | Number | Efficiency (0–1). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
rho      = Density(fluid$, P=in.P, h=in.h)
Q        = in.mdot / rho
dP       = dP0 * (1 - (Q / Q0)^2)
out.mdot = in.mdot
out.P    = in.P + dP
out.h    = in.h + dP / (rho * eta)
\`\`\``,
  },
  {
    name: `FanCurve`,
    slug: `fancurve`,
    category: `Component (fluid)`,
    summary: `A fan whose pressure rise follows a tabulated pressure–flow performance curve.`,
    related: [`Fan`, `FanMap`],
    examples: [],
    tags: [`fan`, `fan-family`, `fancurve`, `curve`, `data:rho-dP0-Q0`, `ports:in-out`, `flow-closed`, `energy-pressure`, `steady`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A fan whose pressure rise follows a tabulated pressure–flow performance curve.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
FanCurve inst(rho, dP0, Q0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`rho\` | Number | Density [kg/m³]. |
| \`dP0\` | Number | Reference pressure drop [Pa]. |
| \`Q0\` | Number | Reference heat [W]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Q        = in.mdot / rho
dP       = dP0 * (1 - (Q / Q0)^2)
out.mdot = in.mdot
out.P    = in.P + dP
\`\`\``,
  },
  {
    name: `FanMap`,
    slug: `fanmap`,
    category: `Component (fluid)`,
    summary: `A fan whose pressure rise comes from a tabulated performance map (ΔP vs volumetric flow).`,
    related: [`Fan`, `FanCurve`],
    examples: [],
    tags: [`fan`, `fan-family`, `fanmap`, `map`, `data:rho-map`, `ports:in-out`, `flow-closed`, `energy-pressure`, `steady`, `map-driven`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A fan whose pressure rise comes from a tabulated performance map (ΔP vs volumetric flow).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
FanMap inst(rho, map$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`rho\` | Number | Density [kg/m³]. |
| \`map$\` | String | Name of a TABLE/FUNCTION giving pressure rise [Pa] vs volumetric flow [m³/s]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Q        = in.mdot / rho
dP       = map$(Q)
out.mdot = in.mdot
out.P    = in.P + dP
\`\`\``,
  },
  {
    name: `FlowSensor`,
    slug: `flowsensor`,
    category: `Component (fluid)`,
    summary: `Measures the mass flow of a stream (a pass-through sensor).`,
    related: [],
    examples: [],
    tags: [`flowsensor`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Measures the mass flow of a stream (a pass-through sensor).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
FlowSensor inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
out.h     = in.h
mdot_meas = in.mdot
\`\`\``,
  },
  {
    name: `HeatedDuct`,
    slug: `heatedduct`,
    category: `Component (fluid)`,
    summary: `Acausal fluid-domain component HeatedDuct with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`heatedduct`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **fluid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HeatedDuct inst(fluid$, UA)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`UA\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
T_in      = Temperature(fluid$, P=in.P, h=in.h)
cp_d      = Cp(fluid$, P=in.P, h=in.h)
epsd      = 1 - exp(-UA / (in.mdot * cp_d))
Q         = epsd * in.mdot * cp_d * (wall.T - T_in)
out.h     = in.h + Q / in.mdot
wall.Qdot = Q
\`\`\``,
  },
  {
    name: `HeatExchanger`,
    slug: `heatexchanger`,
    category: `Component (fluid)`,
    summary: `Transfers heat between two fluid streams across a wall.`,
    related: [],
    examples: [],
    tags: [`heatexchanger`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Transfers heat between two fluid streams across a wall.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`hot_in\`, \`hot_out\`, \`cold_in\`, \`cold_out\`

## Usage

\`\`\`
HeatExchanger inst(UA, hot$, cold$, arr$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`UA\` | Number | Overall conductance UA [W/K]. |
| \`hot$\` | String | Hot-side fluid name (e.g. Water). |
| \`cold$\` | String | Cold-side fluid name (e.g. EG50). |
| \`arr$\` | String | Flow arrangement (passed to hx_effectiveness) — one of \`counterflow\`, \`parallel\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
hot_out.mdot  = hot_in.mdot
hot_out.P     = hot_in.P
cold_out.mdot = cold_in.mdot
cold_out.P    = cold_in.P
Th   = Temperature(hot$,  P=hot_in.P,  h=hot_in.h)
Tc   = Temperature(cold$, P=cold_in.P, h=cold_in.h)
C_h  = hot_in.mdot  * Cp(hot$,  P=hot_in.P,  h=hot_in.h)
C_c  = cold_in.mdot * Cp(cold$, P=cold_in.P, h=cold_in.h)
Cmin = min(C_h, C_c)
Cmax = max(C_h, C_c)
eps  = hx_effectiveness(arr$, UA / Cmin, Cmin / Cmax)
Q    = eps * Cmin * (Th - Tc)
hot_out.h  = hot_in.h  - Q / hot_in.mdot
cold_out.h = cold_in.h + Q / cold_in.mdot
\`\`\``,
  },
  {
    name: `Mixer`,
    slug: `mixer`,
    category: `Component (fluid)`,
    summary: `Combines two fluid streams into one, with flow-weighted enthalpy mixing.`,
    related: [],
    examples: [],
    tags: [`mixer`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Combines two fluid streams into one, with flow-weighted enthalpy mixing.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in1\`, \`in2\`, \`out\`

## Usage

\`\`\`
Mixer inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P    = in1.P
out.mdot = in1.mdot + in2.mdot
out.mdot * out.h = in1.mdot * in1.h + in2.mdot * in2.h
\`\`\``,
  },
  {
    name: `Nozzle`,
    slug: `nozzle`,
    category: `Component (fluid)`,
    summary: `Accelerates a flow, converting enthalpy into kinetic energy.`,
    related: [],
    examples: [`cd-nozzle-shock`],
    tags: [`nozzle`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Accelerates a flow, converting enthalpy into kinetic energy.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Nozzle inst(k, R, A_throat, A_exit, P_amb, T0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`k\` | Number | Stiffness / conductivity. |
| \`R\` | Number | Resistance [Ω]. |
| \`A_throat\` | Number | Throat area [m²]. |
| \`A_exit\` | Number | Exit area [m²]. |
| \`P_amb\` | Number | Ambient pressure [Pa]. |
| \`T0\` | Number | Reference/initial temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
M_exit   = mach_A_Astar(A_exit / A_throat, k, 'supersonic')
out.P    = in.P / P0_P(M_exit, k)
T_exit   = T0 / T0_T(M_exit, k)
V_exit   = M_exit * sqrt(k * R * T_exit)
out.h    = in.h - V_exit^2 / 2
thrust   = in.mdot * V_exit + (out.P - P_amb) * A_exit
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: cd-nozzle-shock]`,
  },
  {
    name: `Pipe`,
    slug: `pipe`,
    category: `Component (fluid)`,
    summary: `A flow passage that imposes a frictional pressure drop.`,
    related: [],
    examples: [],
    tags: [`pipe`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A flow passage that imposes a frictional pressure drop.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Pipe inst(fluid$, L, D, rough)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`L\` | Number | Length [m]. |
| \`D\` | Number | Diameter [m]. |
| \`rough\` | Number | Relative wall roughness. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
rho      = Density(fluid$, P=in.P, h=in.h)
mu       = Viscosity(fluid$, P=in.P, h=in.h)
A        = pi# / 4 * D^2
V        = in.mdot / (rho * A)
Re_d     = reynolds(rho, V, D, mu)
f        = friction_factor(Re_d, rough / D)
out.P    = in.P - f * (L / D) * rho * V^2 / 2
\`\`\``,
  },
  {
    name: `Propeller`,
    slug: `propeller`,
    category: `Component (fluid)`,
    summary: `Acausal fluid-domain component Propeller with ports shaft, veh.`,
    related: [],
    examples: [],
    tags: [`propeller`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **fluid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Propeller inst(Dp, rhoA, ct$, cpw$, epsn)
\`\`\`

## Ports

\`shaft\`, \`veh\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Dp\` | Number |
| \`rhoA\` | Number |
| \`ct$\` | String |
| \`cpw$\` | String |
| \`epsn\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
n         = shaft.w / (2 * pi#)
J         = veh.vel / (n * Dp + epsn)
veh.f     = -(ct$(J) * rhoA * n^2 * Dp^4)
shaft.tau = cpw$(J) * rhoA * n^2 * Dp^5 / (2 * pi#)
\`\`\``,
  },
  {
    name: `Pump`,
    slug: `pump`,
    category: `Component (fluid)`,
    summary: `Raises the pressure of a liquid stream, computing the work from a pump efficiency.`,
    related: [`PumpMap`, `LiquidPump`],
    examples: [`pump-sizing`, `rankine-cycle`],
    tags: [`pump`, `pump-family`, `data:eta`, `ports:in-out`, `flow-closed`, `energy-work`, `steady`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Raises the pressure of a liquid stream, computing the work from a pump efficiency.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Pump inst(eta, fluid$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`eta\` | Number | Efficiency (0–1). |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
v        = Volume(fluid$, P=in.P, h=in.h)
out.mdot = in.mdot
out.h    = in.h + v * (out.P - in.P) / eta
W        = in.mdot * (out.h - in.h)
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pump-sizing]`,
  },
  {
    name: `PumpMap`,
    slug: `pumpmap`,
    category: `Component (fluid)`,
    summary: `A pump whose head comes from a tabulated performance map (head vs volumetric flow).`,
    related: [`Pump`, `LiquidPumpMap`],
    examples: [],
    tags: [`pump`, `pump-family`, `pumpmap`, `map`, `data:rho-map`, `ports:in-out`, `flow-closed`, `energy-pressure`, `steady`, `map-driven`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A pump whose head comes from a tabulated performance map (head vs volumetric flow).

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
PumpMap inst(rho, map$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`rho\` | Number | Density [kg/m³]. |
| \`map$\` | String | Name of a TABLE/FUNCTION giving head [m] vs volumetric flow [m³/s]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Q        = in.mdot / rho
head     = map$(Q)
out.mdot = in.mdot
out.P    = in.P + rho * 9.80665 * head
\`\`\``,
  },
  {
    name: `Regenerator`,
    slug: `regenerator`,
    category: `Component (fluid)`,
    summary: `Acausal fluid-domain component Regenerator with ports hot_in, hot_out, cold_in, cold_out.`,
    related: [],
    examples: [],
    tags: [`regenerator`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **fluid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Regenerator inst(hot$, cold$, eps)
\`\`\`

## Ports

\`hot_in\`, \`hot_out\`, \`cold_in\`, \`cold_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`hot$\` | String |
| \`cold$\` | String |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
hot_out.mdot  = hot_in.mdot
hot_out.P     = hot_in.P
cold_out.mdot = cold_in.mdot
cold_out.P    = cold_in.P
Th   = Temperature(hot$,  P=hot_in.P,  h=hot_in.h)
Tc   = Temperature(cold$, P=cold_in.P, h=cold_in.h)
C_h  = hot_in.mdot  * Cp(hot$,  P=hot_in.P,  h=hot_in.h)
C_c  = cold_in.mdot * Cp(cold$, P=cold_in.P, h=cold_in.h)
Q    = eps * min(C_h, C_c) * (Th - Tc)
hot_out.h  = hot_in.h  - Q / hot_in.mdot
cold_out.h = cold_in.h + Q / cold_in.mdot
\`\`\``,
  },
  {
    name: `Sink`,
    slug: `sink`,
    category: `Component (fluid)`,
    summary: `A fluid boundary that absorbs a stream at a set pressure.`,
    related: [],
    examples: [],
    tags: [`sink`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A fluid boundary that absorbs a stream at a set pressure.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`

## Usage

\`\`\`
Sink inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
mdot = in.mdot
P    = in.P
h    = in.h
\`\`\``,
  },
  {
    name: `Source`,
    slug: `source`,
    category: `Component (fluid)`,
    summary: `A fluid boundary that supplies a stream at set conditions.`,
    related: [],
    examples: [],
    tags: [`source`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A fluid boundary that supplies a stream at set conditions.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
Source inst(fluid$, mdot, P, T)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`mdot\` | Number | Mass flow rate [kg/s]. |
| \`P\` | Number | Pressure [Pa]. |
| \`T\` | Number | Temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = mdot
out.P    = P
out.h    = Enthalpy(fluid$, P=P, T=T)
\`\`\``,
  },
  {
    name: `Splitter`,
    slug: `splitter`,
    category: `Component (fluid)`,
    summary: `Divides a fluid stream into two branches.`,
    related: [],
    examples: [],
    tags: [`splitter`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Divides a fluid stream into two branches.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out1\`, \`out2\`

## Usage

\`\`\`
Splitter inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out1.P   = in.P
out2.P   = in.P
out1.h   = in.h
out2.h   = in.h
in.mdot  = out1.mdot + out2.mdot
\`\`\``,
  },
  {
    name: `Throttle`,
    slug: `throttle`,
    category: `Component (fluid)`,
    summary: `An isenthalpic pressure-reducing restriction.`,
    related: [],
    examples: [],
    tags: [`throttle`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `An isenthalpic pressure-reducing restriction.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Throttle inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
\`\`\``,
  },
  {
    name: `Turbine`,
    slug: `turbine`,
    category: `Component (fluid)`,
    summary: `Extracts work from an expanding fluid stream, computing it from an isentropic efficiency.`,
    related: [],
    examples: [],
    tags: [`turbine`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `Extracts work from an expanding fluid stream, computing it from an isentropic efficiency.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Turbine inst(eta, fluid$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`eta\` | Number | Efficiency (0–1). |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
s_in     = Entropy(fluid$, P=in.P, h=in.h)
h_s      = Enthalpy(fluid$, P=out.P, s=s_in)
out.mdot = in.mdot
out.h    = in.h - eta * (in.h - h_s)
W        = in.mdot * (in.h - out.h)
\`\`\``,
  },
  {
    name: `Turbocharger`,
    slug: `turbocharger`,
    category: `Component (fluid)`,
    summary: `A turbine-driven compressor pair coupled on a common shaft.`,
    related: [],
    examples: [],
    tags: [`turbocharger`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A turbine-driven compressor pair coupled on a common shaft.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`t_in\`, \`t_out\`, \`c_in\`, \`c_out\`

## Usage

\`\`\`
Turbocharger inst(cp, eta_t, eta_c, gam)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`cp\` | Number | Specific heat [J/kg·K]. |
| \`eta_t\` | Number | Turbine efficiency (0–1). |
| \`eta_c\` | Number | Compressor efficiency (0–1). |
| \`gam\` | Number | Ratio of specific heats. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
PRt        = t_in.P / t_out.P
t_out.T    = t_in.T * (1 - eta_t * (1 - PRt^((1 - gam) / gam)))
t_out.mdot = t_in.mdot
Wt         = t_in.mdot * cp * (t_in.T - t_out.T)
PRc        = c_out.P / c_in.P
c_out.T    = c_in.T * (1 + (PRc^((gam - 1) / gam) - 1) / eta_c)
c_out.mdot = c_in.mdot
Wc         = c_in.mdot * cp * (c_out.T - c_in.T)
Wt         = Wc
\`\`\``,
  },
  {
    name: `TwoZoneHX`,
    slug: `twozonehx`,
    category: `Component (fluid)`,
    summary: `A two-zone heat exchanger resolving distinct thermal regions.`,
    related: [],
    examples: [],
    tags: [`twozonehx`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [],
    body: `A two-zone heat exchanger resolving distinct thermal regions.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`hot_in\`, \`hot_out\`, \`cold_in\`, \`cold_out\`

## Usage

\`\`\`
TwoZoneHX inst(UA, hot$, cold$, arr$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`UA\` | Number | Overall conductance UA [W/K]. |
| \`hot$\` | String | Hot-side fluid name (e.g. Water). |
| \`cold$\` | String | Cold-side fluid name (e.g. EG50). |
| \`arr$\` | String | Flow arrangement (passed to hx_effectiveness) — one of \`counterflow\`, \`parallel\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
HeatExchanger C1(UA=UA/2, hot$=hot$, cold$=cold$, arr$=arr$)
HeatExchanger C2(UA=UA/2, hot$=hot$, cold$=cold$, arr$=arr$)
connect(hot_in, C1.hot_in)
connect(C1.hot_out, C2.hot_in)
connect(C2.hot_out, hot_out)
connect(cold_in, C2.cold_in)
connect(C2.cold_out, C1.cold_in)
connect(C1.cold_out, cold_out)
\`\`\``,
  },
  {
    name: `Valve`,
    slug: `valve`,
    category: `Component (fluid)`,
    summary: `A flow restriction characterized by a flow/pressure-drop coefficient.`,
    related: [],
    examples: [],
    tags: [`valve`, `component`, `fluid`, `acausal`],
    references: [],
    guides: [`comp-domains`],
    body: `A flow restriction characterized by a flow/pressure-drop coefficient.

## Domain

A reusable **acausal fluid-domain** component — its thermofluid ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`; a node enforces equal \`P\` and \`Σṁ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Valve inst(Cv, rho)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Cv\` | Number | Flow coefficient. |
| \`rho\` | Number | Density [kg/m³]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
in.mdot * abs(in.mdot) = Cv^2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `CellToPackThermal`,
    slug: `celltopackthermal`,
    category: `Component (heat)`,
    summary: `Acausal heat-domain component CellToPackThermal with ports cell, plate.`,
    related: [],
    examples: [],
    tags: [`celltopackthermal`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **heat-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
CellToPackThermal inst(Rcc, Cpl, T0)
\`\`\`

## Ports

\`cell\`, \`plate\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Rcc\` | Number |
| \`Cpl\` | Number |
| \`T0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
Q         = (cell.T - Tp) / Rcc
cell.Qdot = Q
der(Tp)   = (Q + plate.Qdot) / Cpl
init(Tp)  = T0
plate.T   = Tp
\`\`\``,
  },
  {
    name: `Conduction`,
    slug: `conduction`,
    category: `Component (heat)`,
    summary: `A conductive thermal resistance (Fourier), Q̇ = (T1 − T2)/R.`,
    related: [],
    examples: [`heat-conduction`, `transient-heat-rod`, `heisler-transient`, `material-conduction`],
    tags: [`conduction`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A conductive thermal resistance (Fourier), \`Q̇ = (T1 − T2)/R\`.

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
Conduction inst(k, area, L)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`k\` | Number | Stiffness / conductivity. |
| \`area\` | Number | Area [m²]. |
| \`L\` | Number | Length [m]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Q      = k * area / L * (a.T - b.T)
a.Qdot = Q
b.Qdot = -Q
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: heat-conduction]`,
  },
  {
    name: `ContactResistance`,
    slug: `contactresistance`,
    category: `Component (heat)`,
    summary: `A thermal contact resistance between two surfaces.`,
    related: [],
    examples: [],
    tags: [`contactresistance`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A thermal contact resistance between two surfaces.

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
ContactResistance inst(Rth)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Rth\` | Number | Thermal resistance [K/W]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Q      = (a.T - b.T) / Rth
a.Qdot = Q
b.Qdot = -Q
\`\`\``,
  },
  {
    name: `Convection`,
    slug: `convection`,
    category: `Component (heat)`,
    summary: `A convective link (Newton’s law of cooling), Q̇ = h·A·ΔT.`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`convection`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A convective link (Newton’s law of cooling), \`Q̇ = h·A·ΔT\`.

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
Convection inst(htc, area)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`htc\` | Number | Heat-transfer coefficient [W/m²·K]. |
| \`area\` | Number | Area [m²]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Q      = htc * area * (a.T - b.T)
a.Qdot = Q
b.Qdot = -Q
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pressure-cooker]`,
  },
  {
    name: `HeatPipe`,
    slug: `heatpipe`,
    category: `Component (heat)`,
    summary: `Acausal heat-domain component HeatPipe with ports a, b.`,
    related: [],
    examples: [],
    tags: [`heatpipe`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **heat-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HeatPipe inst(G, Qmax)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`G\` | Number |
| \`Qmax\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
a.Qdot = Qmax * tanh(G * (a.T - b.T) / Qmax)
a.Qdot + b.Qdot = 0
\`\`\``,
  },
  {
    name: `HeatSource`,
    slug: `heatsource`,
    category: `Component (heat)`,
    summary: `A prescribed heat input to a thermal node.`,
    related: [],
    examples: [],
    tags: [`heatsource`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A prescribed heat input to a thermal node.

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
HeatSource inst(Q)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Q\` | Number | Heat input [W]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
port.Qdot = -Q
\`\`\``,
  },
  {
    name: `MassGen`,
    slug: `massgen`,
    category: `Component (heat)`,
    summary: `A mass/heat generation source term.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`massgen`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A mass/heat generation source term.

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
MassGen inst(C, Qgen, T0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`C\` | Number | Capacitance [F]. |
| \`Qgen\` | Number | Generated heat [W]. |
| \`T0\` | Number | Reference/initial temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
der(port.T)  = (Qgen + port.Qdot) / C
init(port.T) = T0
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `MultiZoneWall`,
    slug: `multizonewall`,
    category: `Component (heat)`,
    summary: `Acausal heat-domain component MultiZoneWall with ports a, b.`,
    related: [],
    examples: [],
    tags: [`multizonewall`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **heat-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
MultiZoneWall inst(h_a, h_b, U, A, C1, C2, T10, T20)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`h_a\` | Number |
| \`h_b\` | Number |
| \`U\` | Number |
| \`A\` | Number |
| \`C1\` | Number |
| \`C2\` | Number |
| \`T10\` | Number |
| \`T20\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
qa       = h_a * A * (a.T - T1)
a.Qdot   = qa
q        = U * A * (T1 - T2)
der(T1)  = (qa - q) / C1
init(T1) = T10
qb       = h_b * A * (T2 - b.T)
b.Qdot   = -qb
der(T2)  = (q - qb) / C2
init(T2) = T20
\`\`\``,
  },
  {
    name: `PCMMass`,
    slug: `pcmmass`,
    category: `Component (heat)`,
    summary: `Acausal heat-domain component PCMMass with ports port.`,
    related: [],
    examples: [],
    tags: [`pcmmass`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **heat-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PCMMass inst(m, cp, L, Tm, dTm, T0)
\`\`\`

## Ports

\`port\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`m\` | Number |
| \`cp\` | Number |
| \`L\` | Number |
| \`Tm\` | Number |
| \`dTm\` | Number |
| \`T0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
cpe          = cp + (L / (dTm * 1.7724539)) * exp(-((port.T - Tm) / dTm)^2)
der(port.T)  = port.Qdot / (m * cpe)
init(port.T) = T0
\`\`\``,
  },
  {
    name: `PeltierTEC`,
    slug: `peltiertec`,
    category: `Component (heat)`,
    summary: `Acausal heat-domain component PeltierTEC with ports p, n, hot, cold.`,
    related: [],
    examples: [],
    tags: [`peltiertec`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **heat-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PeltierTEC inst(Sab, Rel, Kth)
\`\`\`

## Ports

\`p\`, \`n\`, \`hot\`, \`cold\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Sab\` | Number |
| \`Rel\` | Number |
| \`Kth\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
I         = p.I
p.V - n.V = Sab * (hot.T - cold.T) + Rel * I
p.I + n.I = 0
Qc        = Sab * cold.T * I - 0.5 * Rel * I^2 - Kth * (hot.T - cold.T)
Qh        = Sab * hot.T * I + 0.5 * Rel * I^2 - Kth * (hot.T - cold.T)
cold.Qdot = Qc
hot.Qdot  = -Qh
\`\`\``,
  },
  {
    name: `Radiation`,
    slug: `radiation`,
    category: `Component (heat)`,
    summary: `A radiative exchange link (Stefan–Boltzmann), Q̇ = εσA(T1⁴ − T2⁴).`,
    related: [],
    examples: [`radiation-view-factors`],
    tags: [`radiation`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A radiative exchange link (Stefan–Boltzmann), \`Q̇ = εσA(T1⁴ − T2⁴)\`.

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
Radiation inst(emis, area)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`emis\` | Number | Emissivity (0–1). |
| \`area\` | Number | Area [m²]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
Q      = emis * 5.670374419e-8 * area * (a.T^4 - b.T^4)
a.Qdot = Q
b.Qdot = -Q
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: radiation-view-factors]`,
  },
  {
    name: `RadiationTwoSurface`,
    slug: `radiationtwosurface`,
    category: `Component (heat)`,
    summary: `Acausal heat-domain component RadiationTwoSurface with ports a, b.`,
    related: [],
    examples: [],
    tags: [`radiationtwosurface`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **heat-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
RadiationTwoSurface inst(e1, e2, A1, A2, F12)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`e1\` | Number |
| \`e2\` | Number |
| \`A1\` | Number |
| \`A2\` | Number |
| \`F12\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
Rrad   = (1 - e1) / (e1 * A1) + 1 / (A1 * F12) + (1 - e2) / (e2 * A2)
a.Qdot = 5.670374419e-8 * (a.T^4 - b.T^4) / Rrad
a.Qdot + b.Qdot = 0
\`\`\``,
  },
  {
    name: `ThermalMass`,
    slug: `thermalmass`,
    category: `Component (heat)`,
    summary: `A lumped thermal capacitance, C dT/dt = Q̇.`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`thermalmass`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A lumped thermal capacitance, \`C dT/dt = Q̇\`.

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
ThermalMass inst(C, T0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`C\` | Number | Capacitance [F]. |
| \`T0\` | Number | Reference/initial temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
der(port.T)  = port.Qdot / C
init(port.T) = T0
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pressure-cooker]`,
  },
  {
    name: `ThermalSensor`,
    slug: `thermalsensor`,
    category: `Component (heat)`,
    summary: `A temperature sensor (pass-through).`,
    related: [],
    examples: [],
    tags: [`thermalsensor`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A temperature sensor (pass-through).

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
ThermalSensor inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
port.Qdot = 0
T_meas    = port.T
\`\`\``,
  },
  {
    name: `ThermalSource`,
    slug: `thermalsource`,
    category: `Component (heat)`,
    summary: `A prescribed-temperature boundary.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`thermalsource`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `A prescribed-temperature boundary.

## Domain

A reusable **acausal heat-domain** component — its thermal ports carry temperature \`T\` and heat-flow rate \`Q̇\`; a node enforces equal \`T\` and \`ΣQ̇ = 0\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
ThermalSource inst(T)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`T\` | Number | Temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
port.T = T
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `ThermalSwitch`,
    slug: `thermalswitch`,
    category: `Component (heat)`,
    summary: `Acausal heat-domain component ThermalSwitch with ports a, b.`,
    related: [],
    examples: [],
    tags: [`thermalswitch`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **heat-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ThermalSwitch inst(G, Ton, band)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`G\` | Number |
| \`Ton\` | Number |
| \`band\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
u      = 0.5 * (1 + tanh((a.T - Ton) / band))
a.Qdot = u * G * (a.T - b.T)
a.Qdot + b.Qdot = 0
\`\`\``,
  },
  {
    name: `WallRC`,
    slug: `wallrc`,
    category: `Component (heat)`,
    summary: `Acausal heat-domain component WallRC with ports a, b.`,
    related: [],
    examples: [],
    tags: [`wallrc`, `component`, `heat`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **heat-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
WallRC inst(C1, C2, R, T10, T20)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`C1\` | Number |
| \`C2\` | Number |
| \`R\` | Number |
| \`T10\` | Number |
| \`T20\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(T1)  = (a.Qdot - (T1 - T2) / R) / C1
init(T1) = T10
der(T2)  = ((T1 - T2) / R + b.Qdot) / C2
init(T2) = T20
a.T = T1
b.T = T2
\`\`\``,
  },
  {
    name: `CounterbalanceValve`,
    slug: `counterbalancevalve`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component CounterbalanceValve with ports in, out, pilot.`,
    related: [],
    examples: [],
    tags: [`counterbalancevalve`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
CounterbalanceValve inst(CdA_max, rho, P_set, R_p, eps_o, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`pilot\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`CdA_max\` | Number |
| \`rho\` | Number |
| \`P_set\` | Number |
| \`R_p\` | Number |
| \`eps_o\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
x_o      = 0.5 * (1 + tanh((in.P + R_p * pilot.P - P_set) / eps_o))
out.mdot = in.mdot
out.h    = in.h
pilot.mdot = 0
in.mdot * abs(in.mdot) = (x_o * CdA_max)^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `HydraulicAccumulator`,
    slug: `hydraulicaccumulator`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicAccumulator with ports port.`,
    related: [],
    examples: [],
    tags: [`hydraulicaccumulator`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicAccumulator inst(P0, V0, gamma, rho, Vg0, domain$)
\`\`\`

## Ports

\`port\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`P0\` | Number |
| \`V0\` | Number |
| \`gamma\` | Number |
| \`rho\` | Number |
| \`Vg0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(Vg)  = -port.mdot / rho
init(Vg) = Vg0
port.P   = P0 * (V0 / Vg)^gamma
\`\`\``,
  },
  {
    name: `HydraulicCheckValve`,
    slug: `hydrauliccheckvalve`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicCheckValve with ports in, out.`,
    related: [],
    examples: [],
    tags: [`hydrauliccheckvalve`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicCheckValve inst(CdA, rho, eps, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`CdA\` | Number |
| \`rho\` | Number |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
dP       = in.P - out.P
g        = 0.5 * (1 + tanh(dP / eps))
in.mdot * abs(in.mdot) = g * CdA^2 * 2 * rho * dP
\`\`\``,
  },
  {
    name: `HydraulicCylinder`,
    slug: `hydrauliccylinder`,
    category: `Component (hydraulic)`,
    summary: `A hydraulic actuator converting flow/pressure to motion/force.`,
    related: [],
    examples: [],
    tags: [`hydrauliccylinder`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `A hydraulic actuator converting flow/pressure to motion/force.

## Domain

A reusable **acausal hydraulic-domain** component — its oil-hydraulic ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`rod\`

## Usage

\`\`\`
HydraulicCylinder inst(rho, beta, V0, area, Patm, P0, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`rho\` | Number | Density [kg/m³]. |
| \`beta\` | Number | Chevron angle [deg] / coefficient. |
| \`V0\` | Number | Initial voltage / volume. |
| \`area\` | Number | Area [m²]. |
| \`Patm\` | Number | Atmospheric pressure [Pa]. |
| \`P0\` | Number | Reference/initial pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
rod.f      = -(in.P - Patm) * area
der(in.P)  = (beta / V0) * (in.mdot / rho - area * rod.vel)
init(in.P) = P0
\`\`\``,
  },
  {
    name: `HydraulicDoubleActingCylinder`,
    slug: `hydraulicdoubleactingcylinder`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicDoubleActingCylinder with ports a, b, rod.`,
    related: [],
    examples: [],
    tags: [`hydraulicdoubleactingcylinder`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicDoubleActingCylinder inst(Aa, Ab, rho, beta, Va0, Vb0, Pa0, Pb0, domain$)
\`\`\`

## Ports

\`a\`, \`b\`, \`rod\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Aa\` | Number |
| \`Ab\` | Number |
| \`rho\` | Number |
| \`beta\` | Number |
| \`Va0\` | Number |
| \`Vb0\` | Number |
| \`Pa0\` | Number |
| \`Pb0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
rod.f     = -(a.P * Aa - b.P * Ab)
der(a.P)  = (beta / Va0) * (a.mdot / rho - Aa * rod.vel)
init(a.P) = Pa0
der(b.P)  = (beta / Vb0) * (b.mdot / rho + Ab * rod.vel)
init(b.P) = Pb0
\`\`\``,
  },
  {
    name: `HydraulicFlowControl`,
    slug: `hydraulicflowcontrol`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicFlowControl with ports in, out.`,
    related: [],
    examples: [],
    tags: [`hydraulicflowcontrol`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicFlowControl inst(Qset, rho, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Qset\` | Number |
| \`rho\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
in.mdot  = rho * Qset
\`\`\``,
  },
  {
    name: `HydraulicFlowDivider`,
    slug: `hydraulicflowdivider`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicFlowDivider with ports in, outa, outb.`,
    related: [],
    examples: [],
    tags: [`hydraulicflowdivider`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicFlowDivider inst(frac, domain$)
\`\`\`

## Ports

\`in\`, \`outa\`, \`outb\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`frac\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
outa.mdot = frac * in.mdot
outb.mdot = (1 - frac) * in.mdot
outa.h    = in.h
outb.h    = in.h
in.P      = frac * outa.P + (1 - frac) * outb.P
\`\`\``,
  },
  {
    name: `HydraulicMotor`,
    slug: `hydraulicmotor`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicMotor with ports in, out, shaft.`,
    related: [],
    examples: [],
    tags: [`hydraulicmotor`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicMotor inst(disp, rho, eta_v, eta_m, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`shaft\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`disp\` | Number |
| \`rho\` | Number |
| \`eta_v\` | Number |
| \`eta_m\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
n_rev     = shaft.w / (2 * pi#)
in.mdot   = rho * disp * n_rev / eta_v
out.mdot  = in.mdot
out.h     = in.h
shaft.tau = -(disp * (in.P - out.P) / (2 * pi#)) * eta_m
\`\`\``,
  },
  {
    name: `HydraulicOrifice`,
    slug: `hydraulicorifice`,
    category: `Component (hydraulic)`,
    summary: `A hydraulic orifice metering flow by ṁ ∝ √Δp.`,
    related: [],
    examples: [],
    tags: [`hydraulicorifice`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `A hydraulic orifice metering flow by \`ṁ ∝ √Δp\`.

## Domain

A reusable **acausal hydraulic-domain** component — its oil-hydraulic ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
HydraulicOrifice inst(CdA, rho, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`CdA\` | Number | Discharge coefficient × area Cd·A [m²]. |
| \`rho\` | Number | Density [kg/m³]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
in.mdot * abs(in.mdot) = CdA^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `HydraulicPilotCheckValve`,
    slug: `hydraulicpilotcheckvalve`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicPilotCheckValve with ports in, out, pilot.`,
    related: [],
    examples: [],
    tags: [`hydraulicpilotcheckvalve`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicPilotCheckValve inst(CdA, rho, rp, eps, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`pilot\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`CdA\` | Number |
| \`rho\` | Number |
| \`rp\` | Number |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
pilot.mdot = 0
out.mdot   = in.mdot
out.h      = in.h
dPe        = (in.P - out.P) + rp * (pilot.P - in.P)
g          = 0.5 * (1 + tanh(dPe / eps))
in.mdot * abs(in.mdot) = g * CdA^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `HydraulicPipe`,
    slug: `hydraulicpipe`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicPipe with ports in, out.`,
    related: [],
    examples: [],
    tags: [`hydraulicpipe`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicPipe inst(rho, nu, L, D, rough, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`rho\` | Number |
| \`nu\` | Number |
| \`L\` | Number |
| \`D\` | Number |
| \`rough\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
A        = pi# / 4 * D^2
V        = in.mdot / (rho * A)
Re       = reynolds(rho, abs(V) + 1e-9, D, rho * nu)
f        = friction_factor(Re, rough / D)
out.P    = in.P - f * (L / D) * rho * V * abs(V) / 2
\`\`\``,
  },
  {
    name: `HydraulicPump`,
    slug: `hydraulicpump`,
    category: `Component (hydraulic)`,
    summary: `A hydraulic pump delivering flow against pressure.`,
    related: [],
    examples: [],
    tags: [`hydraulicpump`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `A hydraulic pump delivering flow against pressure.

## Domain

A reusable **acausal hydraulic-domain** component — its oil-hydraulic ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`shaft\`

## Usage

\`\`\`
HydraulicPump inst(disp, rho, eta_v, eta_m, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`disp\` | Number | Displacement volume [m³]. |
| \`rho\` | Number | Density [kg/m³]. |
| \`eta_v\` | Number | Volumetric efficiency (0–1). |
| \`eta_m\` | Number | Mechanical efficiency (0–1). |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
n_rev     = shaft.w / (2 * pi#)
out.mdot  = rho * disp * n_rev * eta_v
in.mdot   = out.mdot
out.h     = in.h
shaft.tau = -(disp * (out.P - in.P) / (2 * pi#)) / eta_m
\`\`\``,
  },
  {
    name: `HydraulicResistance`,
    slug: `hydraulicresistance`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicResistance with ports in, out.`,
    related: [],
    examples: [],
    tags: [`hydraulicresistance`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicResistance inst(K, rho, D, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`K\` | Number |
| \`rho\` | Number |
| \`D\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
A        = pi# / 4 * D^2
V        = in.mdot / (rho * A)
out.P    = in.P - K * rho * V * abs(V) / 2
\`\`\``,
  },
  {
    name: `HydraulicSequenceValve`,
    slug: `hydraulicsequencevalve`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicSequenceValve with ports in, out.`,
    related: [],
    examples: [],
    tags: [`hydraulicsequencevalve`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicSequenceValve inst(Pset, CdA, rho, eps, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Pset\` | Number |
| \`CdA\` | Number |
| \`rho\` | Number |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
g        = 0.5 * (1 + tanh((in.P - Pset) / eps))
in.mdot * abs(in.mdot) = g * CdA^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `HydraulicSupply`,
    slug: `hydraulicsupply`,
    category: `Component (hydraulic)`,
    summary: `A hydraulic pressure supply.`,
    related: [],
    examples: [],
    tags: [`hydraulicsupply`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `A hydraulic pressure supply.

## Domain

A reusable **acausal hydraulic-domain** component — its oil-hydraulic ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
HydraulicSupply inst(P, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`P\` | Number | Pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P = P
out.h = 0
\`\`\``,
  },
  {
    name: `HydraulicTank`,
    slug: `hydraulictank`,
    category: `Component (hydraulic)`,
    summary: `A hydraulic reservoir at (near) atmospheric pressure.`,
    related: [],
    examples: [],
    tags: [`hydraulictank`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `A hydraulic reservoir at (near) atmospheric pressure.

## Domain

A reusable **acausal hydraulic-domain** component — its oil-hydraulic ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
HydraulicTank inst(P, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`P\` | Number | Pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
port.P = P
\`\`\``,
  },
  {
    name: `HydraulicThermalVolume`,
    slug: `hydraulicthermalvolume`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicThermalVolume with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`hydraulicthermalvolume`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicThermalVolume inst(V, rho, cp_o, beta, hA, P0, T0, Pvap, eps_c, model$, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`V\` | Number |
| \`rho\` | Number |
| \`cp_o\` | Number |
| \`beta\` | Number |
| \`hA\` | Number |
| \`P0\` | Number |
| \`T0\` | Number |
| \`Pvap\` | Number |
| \`eps_c\` | Number |
| \`model$\` | String |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(Pm)  = beta_eff / (rho * V) * (in.mdot - out.mdot)
init(Pm) = P0
T_in     = in.h / cp_o
der(Tm)  = (in.mdot * cp_o * (T_in - Tm) + hA * (wall.T - Tm)) / (rho * V * cp_o)
init(Tm) = T0
in.P     = Pm
out.P    = Pm
out.h    = cp_o * Tm
wall.Qdot = hA * (wall.T - Tm)
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`stiff\`

\`\`\`
beta_eff = beta
\`\`\`

### \`cav\` — requires \`Pvap\`, \`eps_c\`

\`\`\`
beta_eff = beta * 0.5 * (1 + tanh((Pm - Pvap) / eps_c))
\`\`\``,
  },
  {
    name: `HydraulicValve`,
    slug: `hydraulicvalve`,
    category: `Component (hydraulic)`,
    summary: `A hydraulic valve metering flow vs. pressure drop.`,
    related: [],
    examples: [],
    tags: [`hydraulicvalve`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `A hydraulic valve metering flow vs. pressure drop.

## Domain

A reusable **acausal hydraulic-domain** component — its oil-hydraulic ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
HydraulicValve inst(CdA_max, rho, u, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`CdA_max\` | Number | Maximum Cd·A [m²]. |
| \`rho\` | Number | Density [kg/m³]. |
| \`u\` | Number | Specific internal energy [J/kg]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
in.mdot * abs(in.mdot) = (u * CdA_max)^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `HydraulicValveCmd`,
    slug: `hydraulicvalvecmd`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicValveCmd with ports in, out, u.`,
    related: [],
    examples: [],
    tags: [`hydraulicvalvecmd`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicValveCmd inst(CdA_max, rho, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`CdA_max\` | Number |
| \`rho\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
in.mdot * abs(in.mdot) = (u.sig * CdA_max)^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `HydraulicVolume`,
    slug: `hydraulicvolume`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component HydraulicVolume with ports in, out.`,
    related: [],
    examples: [],
    tags: [`hydraulicvolume`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydraulicVolume inst(V, beta, rho, P0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`V\` | Number |
| \`beta\` | Number |
| \`rho\` | Number |
| \`P0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.P      = in.P
out.h      = in.h
der(in.P)  = (beta / (V * rho)) * (in.mdot - out.mdot)
init(in.P) = P0
\`\`\``,
  },
  {
    name: `LoadSensingPump`,
    slug: `loadsensingpump`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component LoadSensingPump with ports in, out, ls.`,
    related: [],
    examples: [],
    tags: [`loadsensingpump`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
LoadSensingPump inst(rho, Dv, w_p, dP_margin, tau, d0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`ls\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`rho\` | Number |
| \`Dv\` | Number |
| \`w_p\` | Number |
| \`dP_margin\` | Number |
| \`tau\` | Number |
| \`d0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(dfrac)  = ((ls.P + dP_margin) - out.P) / (tau * dP_margin)
init(dfrac) = d0
deff     = 1 / (1 + exp(-8 * (dfrac - 0.5)))
out.mdot = deff * rho * Dv * w_p / (2 * pi#)
in.mdot  = out.mdot
out.h    = in.h
ls.mdot  = 0
\`\`\``,
  },
  {
    name: `ReliefValve`,
    slug: `reliefvalve`,
    category: `Component (hydraulic)`,
    summary: `A pressure-relief valve that opens above its set pressure.`,
    related: [],
    examples: [],
    tags: [`reliefvalve`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `A pressure-relief valve that opens above its set pressure.

## Domain

A reusable **acausal hydraulic-domain** component — its oil-hydraulic ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
ReliefValve inst(Pcrack, K, eps, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Pcrack\` | Number | Cracking (relief) pressure [Pa]. |
| \`K\` | Number | Gain / coefficient. |
| \`eps\` | Number | Effectiveness / roughness. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
open     = 0.5 * (1 + tanh((in.P - Pcrack) / eps))
in.mdot  = K * open * (in.P - out.P)
\`\`\``,
  },
  {
    name: `ServoValveDynamic`,
    slug: `servovalvedynamic`,
    category: `Component (hydraulic)`,
    summary: `Acausal hydraulic-domain component ServoValveDynamic with ports in, out, u.`,
    related: [],
    examples: [],
    tags: [`servovalvedynamic`, `component`, `hydraulic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **hydraulic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ServoValveDynamic inst(CdA_max, rho, wn, zeta, xs0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`CdA_max\` | Number |
| \`rho\` | Number |
| \`wn\` | Number |
| \`zeta\` | Number |
| \`xs0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(xs)  = vs
init(xs) = xs0
der(vs)  = wn^2 * (u.sig - xs) - 2 * zeta * wn * vs
init(vs) = 0
out.mdot = in.mdot
out.h    = in.h
in.mdot * abs(in.mdot) = (xs * CdA_max)^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `CoolingTower`,
    slug: `coolingtower`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component CoolingTower with ports in, out, wb.`,
    related: [],
    examples: [],
    tags: [`coolingtower`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
CoolingTower inst(fluid$, eps_t, mdot_a, Patm, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wb\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`eps_t\` | Number |
| \`mdot_a\` | Number |
| \`Patm\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
T_in   = Temperature(fluid$, P=in.P, h=in.h)
h_s_in = Enthalpy(AirH2O, T=T_in, P=Patm, R=1)
h_wb   = Enthalpy(AirH2O, T=wb.sig, P=Patm, R=1)
Q      = eps_t * mdot_a * (h_s_in - h_wb)
out.mdot = in.mdot
out.P    = in.P
out.h    = in.h - Q / in.mdot
\`\`\``,
  },
  {
    name: `GravityDrain`,
    slug: `gravitydrain`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component GravityDrain with ports in, out.`,
    related: [],
    examples: [],
    tags: [`gravitydrain`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
GravityDrain inst(Cd, A_d, rho, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Cd\` | Number |
| \`A_d\` | Number |
| \`rho\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
dP = in.P - out.P
in.mdot * abs(in.mdot) = (Cd * A_d)^2 * 2 * rho * dP
out.mdot = in.mdot
out.h    = in.h
\`\`\``,
  },
  {
    name: `HydroTurbine`,
    slug: `hydroturbine`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component HydroTurbine with ports in, out, shaft.`,
    related: [],
    examples: [],
    tags: [`hydroturbine`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HydroTurbine inst(fluid$, rho, eta$, epsw, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`shaft\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`rho\` | Number |
| \`eta$\` | String |
| \`epsw\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot  = in.mdot
dP        = in.P - out.P
Pf        = (in.mdot / rho) * dP
eta       = eta$(in.mdot)
Pm        = eta * Pf
shaft.tau = -Pm / (shaft.w + epsw)
out.h     = in.h - Pm / in.mdot
\`\`\``,
  },
  {
    name: `IceStorageBrine`,
    slug: `icestoragebrine`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component IceStorageBrine with ports in, out.`,
    related: [],
    examples: [],
    tags: [`icestoragebrine`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
IceStorageBrine inst(fluid$, UA, m, cp_p, L, Tm, dTm, T0)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`UA\` | Number |
| \`m\` | Number |
| \`cp_p\` | Number |
| \`L\` | Number |
| \`Tm\` | Number |
| \`dTm\` | Number |
| \`T0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
LiquidWallHX HX(fluid$=fluid$, UA=UA)
PCMMass      ICE(m=m, cp=cp_p, L=L, Tm=Tm, dTm=dTm, T0=T0)
connect(in, HX.in)
connect(HX.out, out)
connect(HX.wall, ICE.port)
\`\`\``,
  },
  {
    name: `LiquidCheckValve`,
    slug: `liquidcheckvalve`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component LiquidCheckValve with ports in, out.`,
    related: [],
    examples: [],
    tags: [`liquidcheckvalve`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
LiquidCheckValve inst(CdA, rho, eps, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`CdA\` | Number |
| \`rho\` | Number |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
dP       = in.P - out.P
fwd      = 0.5 * (1 + tanh(dP / eps))
in.mdot * abs(in.mdot) = fwd * CdA^2 * 2 * rho * dP
\`\`\``,
  },
  {
    name: `LiquidColdPlate`,
    slug: `liquidcoldplate`,
    category: `Component (liquid)`,
    summary: `A liquid cold plate cooling an electronics/heat load.`,
    related: [],
    examples: [],
    tags: [`liquidcoldplate`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `A liquid cold plate cooling an electronics/heat load.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
LiquidColdPlate inst(Q, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Q\` | Number | Heat input [W]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
Q        = in.mdot * (out.h - in.h)
\`\`\``,
  },
  {
    name: `LiquidExpansionTank`,
    slug: `liquidexpansiontank`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component LiquidExpansionTank with ports port.`,
    related: [],
    examples: [],
    tags: [`liquidexpansiontank`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
LiquidExpansionTank inst(P, domain$)
\`\`\`

## Ports

\`port\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`P\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
port.P = P
\`\`\``,
  },
  {
    name: `LiquidMixer`,
    slug: `liquidmixer`,
    category: `Component (liquid)`,
    summary: `Mixes two single-phase liquid streams.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`liquidmixer`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Mixes two single-phase liquid streams.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in1\`, \`in2\`, \`out\`

## Usage

\`\`\`
LiquidMixer inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P    = in1.P
in2.P    = in1.P
out.mdot = in1.mdot + in2.mdot
out.mdot * out.h = in1.mdot * in1.h + in2.mdot * in2.h
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `LiquidOrifice`,
    slug: `liquidorifice`,
    category: `Component (liquid)`,
    summary: `A liquid orifice metering flow vs. pressure drop.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`liquidorifice`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `A liquid orifice metering flow vs. pressure drop.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
LiquidOrifice inst(CdA, rho, domain$, model$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`CdA\` | Number | Discharge coefficient × area Cd·A [m²]. |
| \`rho\` | Number | Density [kg/m³]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |
| \`model$\` | String | Model variant — selects the physics body (see Model Variants). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`incompressible\`

\`\`\`
in.mdot * abs(in.mdot) = CdA^2 * 2 * rho * (in.P - out.P)
\`\`\`

### \`cavitating\` — requires \`Pvap\`

\`\`\`
dP_eff = in.P - max(out.P, Pvap)
in.mdot * abs(in.mdot) = CdA^2 * 2 * rho * dP_eff
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `LiquidPipe`,
    slug: `liquidpipe`,
    category: `Component (liquid)`,
    summary: `A single-phase liquid pipe with frictional pressure drop.`,
    related: [],
    examples: [],
    tags: [`liquidpipe`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `A single-phase liquid pipe with frictional pressure drop.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
LiquidPipe inst(fluid$, L, D, rough, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`L\` | Number | Length [m]. |
| \`D\` | Number | Diameter [m]. |
| \`rough\` | Number | Relative wall roughness. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
rho      = Density(fluid$, P=in.P, h=in.h)
mu       = Viscosity(fluid$, P=in.P, h=in.h)
A        = pi# / 4 * D^2
V        = in.mdot / (rho * A)
Re_d     = reynolds(rho, V, D, mu)
f        = friction_factor(Re_d, rough / D)
out.P    = in.P - f * (L / D) * rho * V^2 / 2
\`\`\``,
  },
  {
    name: `LiquidPump`,
    slug: `liquidpump`,
    category: `Component (liquid)`,
    summary: `A single-phase liquid pump.`,
    related: [`LiquidPumpMap`, `Pump`],
    examples: [`ev-thermal-management`],
    tags: [`pump`, `liquid-pump-family`, `liquidpump`, `data:eta`, `ports:in-out`, `flow-closed`, `energy-work`, `steady`, `liquid`, `component`, `acausal`],
    references: [],
    guides: [],
    body: `A single-phase liquid pump.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
LiquidPump inst(eta, fluid$, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`eta\` | Number | Efficiency (0–1). |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
v        = Volume(fluid$, P=in.P, h=in.h)
out.mdot = in.mdot
out.h    = in.h + v * (out.P - in.P) / eta
W        = in.mdot * (out.h - in.h)
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `LiquidPumpMap`,
    slug: `liquidpumpmap`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component LiquidPumpMap with ports in, out.`,
    related: [`LiquidPump`, `PumpMap`],
    examples: [],
    tags: [`pump`, `liquid-pump-family`, `liquidpumpmap`, `map`, `data:rho-map`, `ports:in-out`, `flow-closed`, `energy-work`, `steady`, `map-driven`, `liquid`, `component`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
LiquidPumpMap inst(rho, eta, map$, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`rho\` | Number |
| \`eta\` | Number |
| \`map$\` | String |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
Q        = in.mdot / rho
head     = map$(Q)
out.P    = in.P + rho * 9.80665 * head
out.mdot = in.mdot
out.h    = in.h + 9.80665 * head / eta
W        = in.mdot * 9.80665 * head / eta
\`\`\``,
  },
  {
    name: `LiquidSink`,
    slug: `liquidsink`,
    category: `Component (liquid)`,
    summary: `A liquid boundary absorbing a stream.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`liquidsink`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `A liquid boundary absorbing a stream.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`

## Usage

\`\`\`
LiquidSink inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
mdot = in.mdot
P    = in.P
h    = in.h
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `LiquidSource`,
    slug: `liquidsource`,
    category: `Component (liquid)`,
    summary: `A liquid boundary supplying a stream of set state.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`liquidsource`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `A liquid boundary supplying a stream of set state.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
LiquidSource inst(fluid$, mdot, P, T, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`mdot\` | Number | Mass flow rate [kg/s]. |
| \`P\` | Number | Pressure [Pa]. |
| \`T\` | Number | Temperature [K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = mdot
out.P    = P
out.h    = Enthalpy(fluid$, P=P, T=T)
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `LiquidTank`,
    slug: `liquidtank`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component LiquidTank with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`liquidtank`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
LiquidTank inst(fluid$, m, UA, T0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`m\` | Number |
| \`UA\` | Number |
| \`T0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.P     = in.P
out.mdot  = in.mdot
out.h     = Enthalpy(fluid$, P=in.P, T=Tt)
cp_t      = Cp(fluid$, P=in.P, T=Tt)
Q         = UA * (wall.T - Tt)
der(Tt)   = (in.mdot * (in.h - out.h) + Q) / (m * cp_t)
init(Tt)  = T0
wall.Qdot = Q
\`\`\``,
  },
  {
    name: `LiquidThermostat`,
    slug: `liquidthermostat`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component LiquidThermostat with ports in, out.`,
    related: [],
    examples: [],
    tags: [`liquidthermostat`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
LiquidThermostat inst(fluid$, CdA, rho, Topen, Tband, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`CdA\` | Number |
| \`rho\` | Number |
| \`Topen\` | Number |
| \`Tband\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
T_in     = Temperature(fluid$, P=in.P, h=in.h)
u        = 0.5 * (1 + tanh((T_in - Topen) / Tband))
in.mdot * abs(in.mdot) = (u * CdA)^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `LiquidThreeWayValve`,
    slug: `liquidthreewayvalve`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component LiquidThreeWayValve with ports in, outa, outb.`,
    related: [],
    examples: [],
    tags: [`liquidthreewayvalve`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
LiquidThreeWayValve inst(u, domain$)
\`\`\`

## Ports

\`in\`, \`outa\`, \`outb\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`u\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
outa.P    = in.P
outb.P    = in.P
outa.h    = in.h
outb.h    = in.h
outa.mdot = u * in.mdot
outb.mdot = (1 - u) * in.mdot
\`\`\``,
  },
  {
    name: `LiquidVolume`,
    slug: `liquidvolume`,
    category: `Component (liquid)`,
    summary: `A single-phase liquid control volume.`,
    related: [],
    examples: [],
    tags: [`liquidvolume`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `A single-phase liquid control volume.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
LiquidVolume inst(C, P0, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`C\` | Number | Capacitance [F]. |
| \`P0\` | Number | Reference/initial pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.h       = in.h
der(in.P)   = (in.mdot - out.mdot) / C
init(in.P)  = P0
\`\`\``,
  },
  {
    name: `LiquidWallHX`,
    slug: `liquidwallhx`,
    category: `Component (liquid)`,
    summary: `A liquid-to-wall heat exchanger.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`liquidwallhx`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `A liquid-to-wall heat exchanger.

## Domain

A reusable **acausal liquid-domain** component — its single-phase liquid-coolant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`wall\`

## Usage

\`\`\`
LiquidWallHX inst(fluid$, UA, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`UA\` | Number | Overall conductance UA [W/K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
T_in      = Temperature(fluid$, P=in.P, h=in.h)
Q         = UA * (T_in - wall.T)
out.h     = in.h - Q / in.mdot
wall.Qdot = -Q
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `OpenTank`,
    slug: `opentank`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component OpenTank with ports in, out.`,
    related: [],
    examples: [],
    tags: [`opentank`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
OpenTank inst(A_t, P0, rho, L0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`A_t\` | Number |
| \`P0\` | Number |
| \`rho\` | Number |
| \`L0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(lvl)  = (in.mdot - out.mdot) / (rho * A_t)
init(lvl) = L0
in.P  = P0
out.P = P0 + rho * 9.80665 * lvl
out.h = in.h
\`\`\``,
  },
  {
    name: `ThermalStorageTank`,
    slug: `thermalstoragetank`,
    category: `Component (liquid)`,
    summary: `Acausal liquid-domain component ThermalStorageTank with ports in, out.`,
    related: [],
    examples: [],
    tags: [`thermalstoragetank`, `component`, `liquid`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **liquid-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ThermalStorageTank inst(fluid$, m_node, cp_f, UA_loss, T_amb, kmix, T10, T20, T30, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`m_node\` | Number |
| \`cp_f\` | Number |
| \`UA_loss\` | Number |
| \`T_amb\` | Number |
| \`kmix\` | Number |
| \`T10\` | Number |
| \`T20\` | Number |
| \`T30\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.P    = in.P
T_in     = Temperature(fluid$, P=in.P, h=in.h)
der(T1)  = (in.mdot * cp_f * (T_in - T1) + kmix * (T2 - T1) - UA_loss * (T1 - T_amb)) / (m_node * cp_f)
init(T1) = T10
der(T2)  = (in.mdot * cp_f * (T1 - T2) + kmix * (T1 - T2) + kmix * (T3 - T2) - UA_loss * (T2 - T_amb)) / (m_node * cp_f)
init(T2) = T20
der(T3)  = (in.mdot * cp_f * (T2 - T3) + kmix * (T2 - T3) - UA_loss * (T3 - T_amb)) / (m_node * cp_f)
init(T3) = T30
out.h    = Enthalpy(fluid$, P=out.P, T=T3)
\`\`\``,
  },
  {
    name: `BeltDrive`,
    slug: `beltdrive`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component BeltDrive with ports a, b.`,
    related: [],
    examples: [],
    tags: [`beltdrive`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
BeltDrive inst(ratio, eta)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ratio\` | Number |
| \`eta\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
a.w   = ratio * b.w
b.tau = -ratio * eta * a.tau
\`\`\``,
  },
  {
    name: `Brake`,
    slug: `brake`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component Brake with ports a, b, u.`,
    related: [],
    examples: [],
    tags: [`brake`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Brake inst(Tmax, eps)
\`\`\`

## Ports

\`a\`, \`b\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Tmax\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
dw    = a.w - b.w
a.tau = u.sig * Tmax * tanh(dw / eps)
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `Cam`,
    slug: `cam`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component Cam with ports shaft, rod.`,
    related: [],
    examples: [],
    tags: [`cam`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Cam inst(prof$, theta0)
\`\`\`

## Ports

\`shaft\`, \`rod\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`prof$\` | String |
| \`theta0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(theta)  = shaft.w
init(theta) = theta0
slope     = dtable(prof$, theta)
lift      = prof$(theta)
rod.vel   = slope * shaft.w
shaft.tau = slope * rod.f
\`\`\``,
  },
  {
    name: `CamFollower`,
    slug: `camfollower`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component CamFollower with ports rod.`,
    related: [],
    examples: [],
    tags: [`camfollower`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
CamFollower inst(m, kspring, x0, v0)
\`\`\`

## Ports

\`rod\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`m\` | Number |
| \`kspring\` | Number |
| \`x0\` | Number |
| \`v0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
TransMass   M(m=m, v0=v0)
TransSpring S(k=kspring, x0=x0)
TransGround G()
connect(rod, M.port, S.a)
connect(S.b, G.port)
\`\`\``,
  },
  {
    name: `Clutch`,
    slug: `clutch`,
    category: `Component (mechanical)`,
    summary: `A friction clutch coupling/decoupling two rotational shafts.`,
    related: [],
    examples: [],
    tags: [`clutch`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A friction clutch coupling/decoupling two rotational shafts.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
Clutch inst(Tmax, eng, eps)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Tmax\` | Number | Maximum temperature [K]. |
| \`eng\` | Number | Engagement fraction (0–1). |
| \`eps\` | Number | Effectiveness / roughness. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
dw    = a.w - b.w
a.tau = eng * Tmax * tanh(dw / eps)
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `ClutchCmd`,
    slug: `clutchcmd`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component ClutchCmd with ports a, b, u.`,
    related: [],
    examples: [],
    tags: [`clutchcmd`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ClutchCmd inst(Tmax, eps)
\`\`\`

## Ports

\`a\`, \`b\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Tmax\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
dw    = a.w - b.w
a.tau = u.sig * Tmax * tanh(dw / eps)
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `EndStop`,
    slug: `endstop`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component EndStop with ports port.`,
    related: [],
    examples: [],
    tags: [`endstop`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
EndStop inst(gap, k, c, eps, x0)
\`\`\`

## Ports

\`port\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`gap\` | Number |
| \`k\` | Number |
| \`c\` | Number |
| \`eps\` | Number |
| \`x0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(x)  = port.vel
init(x) = x0
pen     = 0.5 * ((x - gap) + sqrt((x - gap)^2 + eps^2))
port.f  = k * pen + c * port.vel * 0.5 * (1 + tanh((x - gap) / eps))
\`\`\``,
  },
  {
    name: `ForceSource`,
    slug: `forcesource`,
    category: `Component (mechanical)`,
    summary: `A prescribed translational force.`,
    related: [],
    examples: [],
    tags: [`forcesource`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A prescribed translational force.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
ForceSource inst(F)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`F\` | Number | Force [N]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
a.f = -F
a.f + b.f = 0
\`\`\``,
  },
  {
    name: `Freewheel`,
    slug: `freewheel`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component Freewheel with ports a, b.`,
    related: [],
    examples: [],
    tags: [`freewheel`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Freewheel inst(k, eps)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`k\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
dw    = a.w - b.w
a.tau = k * 0.5 * (dw + sqrt(dw^2 + eps^2))
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `Friction`,
    slug: `friction`,
    category: `Component (mechanical)`,
    summary: `A friction element opposing motion.`,
    related: [],
    examples: [],
    tags: [`friction`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A friction element opposing motion.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
Friction inst(Fc, Fs, vs, bv, eps)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Fc\` | Number | Coulomb friction force [N]. |
| \`Fs\` | Number | Static friction force [N]. |
| \`vs\` | Number | Reference / slip velocity [m/s]. |
| \`bv\` | Number | Viscous-friction coefficient. |
| \`eps\` | Number | Effectiveness / roughness. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
dw    = a.w - b.w
a.tau = (Fc + (Fs - Fc) * exp(-(dw / vs)^2)) * tanh(dw / eps) + bv * dw
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `Gear`,
    slug: `gear`,
    category: `Component (mechanical)`,
    summary: `A gear pair imposing a fixed speed/torque ratio between two shafts.`,
    related: [],
    examples: [],
    tags: [`gear`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A gear pair imposing a fixed speed/torque ratio between two shafts.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Gear inst(ratio)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`ratio\` | Number | Gear / split ratio. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
in.w    = ratio * out.w
out.tau = -ratio * in.tau
\`\`\``,
  },
  {
    name: `Inertia`,
    slug: `inertia`,
    category: `Component (mechanical)`,
    summary: `A rotational inertia, τ = J dω/dt.`,
    related: [],
    examples: [],
    tags: [`inertia`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A rotational inertia, \`τ = J dω/dt\`.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
Inertia inst(J, w0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`J\` | Number | Inertia [kg·m²]. |
| \`w0\` | Number | Natural frequency [rad/s]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
der(port.w)  = port.tau / J
init(port.w) = w0
\`\`\``,
  },
  {
    name: `Lever`,
    slug: `lever`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component Lever with ports a, b.`,
    related: [],
    examples: [],
    tags: [`lever`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Lever inst(ratio)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ratio\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
a.vel = ratio * b.vel
b.f   = -ratio * a.f
\`\`\``,
  },
  {
    name: `MechGround`,
    slug: `mechground`,
    category: `Component (mechanical)`,
    summary: `The rotational reference (ω = 0).`,
    related: [],
    examples: [],
    tags: [`mechground`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `The rotational reference (\`ω = 0\`).

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
MechGround inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
port.w = 0
\`\`\``,
  },
  {
    name: `Planetary`,
    slug: `planetary`,
    category: `Component (mechanical)`,
    summary: `A planetary gearset relating sun, ring, and carrier speeds.`,
    related: [],
    examples: [],
    tags: [`planetary`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A planetary gearset relating sun, ring, and carrier speeds.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`sun\`, \`ring\`, \`carrier\`

## Usage

\`\`\`
Planetary inst(g)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`g\` | Number | Gravitational acceleration [m/s²]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
sun.w + g * ring.w = (1 + g) * carrier.w
ring.tau           = g * sun.tau
sun.tau + ring.tau + carrier.tau = 0
\`\`\``,
  },
  {
    name: `RackPinion`,
    slug: `rackpinion`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component RackPinion with ports shaft, rod.`,
    related: [],
    examples: [],
    tags: [`rackpinion`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
RackPinion inst(r)
\`\`\`

## Ports

\`shaft\`, \`rod\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`r\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
rod.vel   = r * shaft.w
shaft.tau = -r * rod.f
\`\`\``,
  },
  {
    name: `RotationalDamper`,
    slug: `rotationaldamper`,
    category: `Component (mechanical)`,
    summary: `A rotational viscous damper, τ = c·ω.`,
    related: [],
    examples: [],
    tags: [`rotationaldamper`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A rotational viscous damper, \`τ = c·ω\`.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
RotationalDamper inst(c)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`c\` | Number | Damping / specific-heat coefficient. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
a.tau = c * (a.w - b.w)
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `RotationalSpring`,
    slug: `rotationalspring`,
    category: `Component (mechanical)`,
    summary: `A torsional spring, τ = k·θ.`,
    related: [],
    examples: [],
    tags: [`rotationalspring`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A torsional spring, \`τ = k·θ\`.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
RotationalSpring inst(k, theta0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`k\` | Number | Stiffness / conductivity. |
| \`theta0\` | Number | Initial angle [rad]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
der(theta)  = a.w - b.w
init(theta) = theta0
a.tau       = k * theta
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `ScrewDrive`,
    slug: `screwdrive`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component ScrewDrive with ports shaft, rod.`,
    related: [],
    examples: [],
    tags: [`screwdrive`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ScrewDrive inst(lead)
\`\`\`

## Ports

\`shaft\`, \`rod\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`lead\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
rod.vel   = lead / (2 * pi#) * shaft.w
shaft.tau = -(lead / (2 * pi#)) * rod.f
\`\`\``,
  },
  {
    name: `SpeedSource`,
    slug: `speedsource`,
    category: `Component (mechanical)`,
    summary: `A prescribed angular velocity.`,
    related: [],
    examples: [],
    tags: [`speedsource`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A prescribed angular velocity.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
SpeedSource inst(w)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`w\` | Number | Frequency [rad/s]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
a.w - b.w = w
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `TorqueSource`,
    slug: `torquesource`,
    category: `Component (mechanical)`,
    summary: `A prescribed torque.`,
    related: [],
    examples: [],
    tags: [`torquesource`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A prescribed torque.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
TorqueSource inst(T)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`T\` | Number | Temperature [K]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
a.tau = -T
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `TorsionalBacklash`,
    slug: `torsionalbacklash`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component TorsionalBacklash with ports a, b.`,
    related: [],
    examples: [],
    tags: [`torsionalbacklash`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TorsionalBacklash inst(k, half, eps, theta0)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`k\` | Number |
| \`half\` | Number |
| \`eps\` | Number |
| \`theta0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(th)  = a.w - b.w
init(th) = theta0
up       = th - half
dn       = th + half
a.tau    = k * (0.5 * (up + sqrt(up^2 + eps^2)) + 0.5 * (dn - sqrt(dn^2 + eps^2)))
a.tau + b.tau = 0
\`\`\``,
  },
  {
    name: `TransDamper`,
    slug: `transdamper`,
    category: `Component (mechanical)`,
    summary: `A translational viscous damper, F = c·v.`,
    related: [],
    examples: [],
    tags: [`transdamper`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A translational viscous damper, \`F = c·v\`.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`a\`, \`b\`

## Usage

\`\`\`
TransDamper inst(c)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`c\` | Number | Damping / specific-heat coefficient. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
a.f = c * (a.vel - b.vel)
a.f + b.f = 0
\`\`\``,
  },
  {
    name: `TransGround`,
    slug: `transground`,
    category: `Component (mechanical)`,
    summary: `The translational reference (v = 0).`,
    related: [],
    examples: [],
    tags: [`transground`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `The translational reference (\`v = 0\`).

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
TransGround inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
port.vel = 0
\`\`\``,
  },
  {
    name: `TransMass`,
    slug: `transmass`,
    category: `Component (mechanical)`,
    summary: `A translational mass, F = m dv/dt.`,
    related: [],
    examples: [],
    tags: [`transmass`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `A translational mass, \`F = m dv/dt\`.

## Domain

A reusable **acausal mechanical-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\` (\`Στ = 0\`); translational ports carry velocity \`v\` and force \`F\` (\`ΣF = 0\`). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
TransMass inst(m, v0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`m\` | Number | Mass [kg]. |
| \`v0\` | Number | Initial velocity [m/s]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
der(port.vel)  = port.f / m
init(port.vel) = v0
\`\`\``,
  },
  {
    name: `TransSpring`,
    slug: `transspring`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component TransSpring with ports a, b.`,
    related: [],
    examples: [],
    tags: [`transspring`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TransSpring inst(k, x0)
\`\`\`

## Ports

\`a\`, \`b\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`k\` | Number |
| \`x0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(x)  = a.vel - b.vel
init(x) = x0
a.f     = k * x
a.f + b.f = 0
\`\`\``,
  },
  {
    name: `WheelBrakeThermal`,
    slug: `wheelbrakethermal`,
    category: `Component (mechanical)`,
    summary: `Acausal mechanical-domain component WheelBrakeThermal with ports a, b, u.`,
    related: [],
    examples: [],
    tags: [`wheelbrakethermal`, `component`, `mechanical`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **mechanical-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
WheelBrakeThermal inst(Tmax, eps, C, hA, T_amb, T_fade, k_fade, eps_f, T0)
\`\`\`

## Ports

\`a\`, \`b\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Tmax\` | Number |
| \`eps\` | Number |
| \`C\` | Number |
| \`hA\` | Number |
| \`T_amb\` | Number |
| \`T_fade\` | Number |
| \`k_fade\` | Number |
| \`eps_f\` | Number |
| \`T0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
fade  = 1 - k_fade * 0.5 * (1 + tanh((Tr - T_fade) / eps_f))
dw    = a.w - b.w
tau_b = fade * u.sig * Tmax * tanh(dw / eps)
a.tau = tau_b
a.tau + b.tau = 0
Pf       = tau_b * dw
der(Tr)  = (Pf - hA * (Tr - T_amb)) / C
init(Tr) = T0
\`\`\``,
  },
  {
    name: `AHU`,
    slug: `ahu`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component AHU with ports ret_in, oa_in, sup_out.`,
    related: [],
    examples: [],
    tags: [`ahu`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [`tut-coil`],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
AHU inst(Kf, foul, Tc, Qh, dPfan, eta_fan)
\`\`\`

## Ports

\`ret_in\`, \`oa_in\`, \`sup_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Kf\` | Number |
| \`foul\` | Number |
| \`Tc\` | Number |
| \`Qh\` | Number |
| \`dPfan\` | Number |
| \`eta_fan\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
MixingBox   MB()
AirFilter   FL(K=Kf, foul=foul)
CoolingCoil CC(Tout=Tc)
HeatingCoil HC(Q=Qh)
MoistAirFan FN(dP=dPfan, eta=eta_fan)
connect(ret_in, MB.in1)
connect(oa_in, MB.in2)
connect(MB.out, FL.in)
connect(FL.out, CC.in)
connect(CC.out, HC.in)
connect(HC.out, FN.in)
connect(FN.out, sup_out)
\`\`\``,
  },
  {
    name: `AirFilter`,
    slug: `airfilter`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component AirFilter with ports in, out.`,
    related: [],
    examples: [],
    tags: [`airfilter`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
AirFilter inst(K, foul, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`K\` | Number |
| \`foul\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.W    = in.W
out.h    = in.h
out.P    = in.P - foul * K * in.mdot^2
\`\`\``,
  },
  {
    name: `ApparatusDewPointCoil`,
    slug: `apparatusdewpointcoil`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component ApparatusDewPointCoil with ports in, out.`,
    related: [],
    examples: [],
    tags: [`apparatusdewpointcoil`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ApparatusDewPointCoil inst(T_adp, BF, domain$, model$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`T_adp\` | Number |
| \`BF\` | Number |
| \`domain$\` | String |
| \`model$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot   = in.mdot
out.P      = in.P
T_in       = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
T_out      = Temperature(AirH2O, h=out.h, P=in.P, W=out.W)
T_dp_in    = DewPoint(AirH2O, h=in.h, P=in.P, W=in.W)
margin_adp = T_dp_in - T_adp
mdot_w     = in.mdot * (in.W - out.W)
h_f        = 4186 * (T_out - 273.15)
Q_air      = in.mdot * (in.h - out.h)
Q          = Q_air - mdot_w * h_f
Q_sens     = in.mdot * Cp(AirH2O, T=T_in, P=in.P, W=in.W) * (T_in - T_out)
SHR        = Q_sens / Q_air
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`wet\`

\`\`\`
W_adp = HumRat(AirH2O, T=T_adp, P=in.P, R=1)
h_adp = Enthalpy(AirH2O, T=T_adp, P=in.P, W=W_adp)
out.W = W_adp + BF * (in.W - W_adp)
out.h = h_adp + BF * (in.h - h_adp)
\`\`\`

### \`dry\`

\`\`\`
out.W = in.W
out.h = Enthalpy(AirH2O, T=T_adp + BF * (T_in - T_adp), P=in.P, W=in.W)
\`\`\``,
  },
  {
    name: `CabinZone`,
    slug: `cabinzone`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component CabinZone with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`cabinzone`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
CabinZone inst(Vz, T0, W0, n_occ, q_sens, mw_occ, Q_aux, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Vz\` | Number |
| \`T0\` | Number |
| \`W0\` | Number |
| \`n_occ\` | Number |
| \`q_sens\` | Number |
| \`mw_occ\` | Number |
| \`Q_aux\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
out.W     = Wz
out.h     = Enthalpy(AirH2O, T=Tz, P=in.P, W=Wz)
v_z       = Volume(AirH2O, T=Tz, P=in.P, W=Wz)
cp_z      = Cp(AirH2O, T=Tz, P=in.P, W=Wz)
m_air     = Vz / v_z
der(Wz)   = (in.mdot * (in.W - Wz) + n_occ * mw_occ) / m_air
init(Wz)  = W0
der(Tz)   = (in.mdot * (in.h - out.h) + n_occ * q_sens + Q_aux + wall.Qdot) / (m_air * cp_z)
init(Tz)  = T0
wall.T    = Tz
\`\`\``,
  },
  {
    name: `ChilledBeam`,
    slug: `chilledbeam`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component ChilledBeam with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`chilledbeam`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ChilledBeam inst(eps, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
out.W     = in.W
T_in      = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
T_out     = T_in - eps * (T_in - wall.T)
out.h     = Enthalpy(AirH2O, T=T_out, P=in.P, W=in.W)
Q         = in.mdot * (in.h - out.h)
wall.Qdot = -Q
T_dp_in   = DewPoint(AirH2O, h=in.h, P=in.P, W=in.W)
margin_dp = wall.T - T_dp_in
\`\`\``,
  },
  {
    name: `CoolingCoil`,
    slug: `coolingcoil`,
    category: `Component (moistair)`,
    summary: `Cools and (below dew point) dehumidifies a humid-air stream.`,
    related: [],
    examples: [],
    tags: [`coolingcoil`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Cools and (below dew point) dehumidifies a humid-air stream.

## Domain

A reusable **acausal moistair-domain** component — its humid-air ports carry pressure \`P\`, dry-air mass-flow \`ṁ_da\`, enthalpy \`h\`, and humidity ratio \`W\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
CoolingCoil inst(Tout, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Tout\` | Number | Outlet temperature [K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.W    = HumRat(AirH2O, T=Tout, P=in.P, R=1)
out.h    = Enthalpy(AirH2O, T=Tout, P=in.P, W=out.W)
Q        = in.mdot * (in.h - out.h)
Q_lat    = in.mdot * 2.501e6 * (in.W - out.W)
\`\`\``,
  },
  {
    name: `DesiccantWheel`,
    slug: `desiccantwheel`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component DesiccantWheel with ports proc_in, proc_out, reg_in, reg_out.`,
    related: [],
    examples: [],
    tags: [`desiccantwheel`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
DesiccantWheel inst(eff_L, W_eq, f_carry, domain$)
\`\`\`

## Ports

\`proc_in\`, \`proc_out\`, \`reg_in\`, \`reg_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff_L\` | Number |
| \`W_eq\` | Number |
| \`f_carry\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
proc_out.mdot = proc_in.mdot
reg_out.mdot  = reg_in.mdot
proc_out.P    = proc_in.P
reg_out.P     = reg_in.P
proc_out.W    = proc_in.W - eff_L * (proc_in.W - W_eq)
proc_out.h    = proc_in.h + f_carry * (reg_in.h - proc_in.h)
reg_out.W     = reg_in.W + (proc_in.mdot / reg_in.mdot) * (proc_in.W - proc_out.W)
reg_out.h     = reg_in.h - (proc_in.mdot / reg_in.mdot) * (proc_out.h - proc_in.h)
\`\`\``,
  },
  {
    name: `Diffuser`,
    slug: `diffuser`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component Diffuser with ports in, out.`,
    related: [],
    examples: [],
    tags: [`diffuser`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Diffuser inst(A1, A2, eta_rec, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`A1\` | Number |
| \`A2\` | Number |
| \`eta_rec\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.W    = in.W
out.h    = in.h
rho      = 1 / Volume(AirH2O, h=in.h, P=in.P, W=in.W)
V1       = in.mdot * (1 + in.W) / (rho * A1)
out.P    = in.P + eta_rec * 0.5 * rho * V1^2 * (1 - (A1 / A2)^2)
\`\`\``,
  },
  {
    name: `DOAS`,
    slug: `doas`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component DOAS with ports oa_in, sup_out, exh_in, exh_out.`,
    related: [],
    examples: [],
    tags: [`doas`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
DOAS inst(eff_h, eff_w, T_adp, BF, Q_reheat)
\`\`\`

## Ports

\`oa_in\`, \`sup_out\`, \`exh_in\`, \`exh_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff_h\` | Number |
| \`eff_w\` | Number |
| \`T_adp\` | Number |
| \`BF\` | Number |
| \`Q_reheat\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
EnthalpyWheel         ERV(eff_h=eff_h, eff_w=eff_w)
ApparatusDewPointCoil CC(T_adp=T_adp, BF=BF)
HeatingCoil           RH(Q=Q_reheat)
connect(oa_in, ERV.sup_in)
connect(ERV.sup_out, CC.in)
connect(CC.out, RH.in)
connect(RH.out, sup_out)
connect(exh_in, ERV.exh_in)
connect(ERV.exh_out, exh_out)
\`\`\``,
  },
  {
    name: `Economizer`,
    slug: `economizer`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component Economizer with ports oa_in, ret_in, mix_out.`,
    related: [],
    examples: [],
    tags: [`economizer`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Economizer inst(mdot_sup, f_min, lim, band, domain$, model$)
\`\`\`

## Ports

\`oa_in\`, \`ret_in\`, \`mix_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`mdot_sup\` | Number |
| \`f_min\` | Number |
| \`lim\` | Number |
| \`band\` | Number |
| \`domain$\` | String |
| \`model$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
mix_out.P    = oa_in.P
mix_out.mdot = mdot_sup
oa_in.mdot   = f_oa * mdot_sup
ret_in.mdot  = (1 - f_oa) * mdot_sup
f_oa         = f_min + (1 - f_min) * g_diff * g_lim
mix_out.mdot * mix_out.W = oa_in.mdot * oa_in.W + ret_in.mdot * ret_in.W
mix_out.mdot * mix_out.h = oa_in.mdot * oa_in.h + ret_in.mdot * ret_in.h
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`drybulb\`

\`\`\`
T_oa   = Temperature(AirH2O, h=oa_in.h, P=oa_in.P, W=oa_in.W)
T_ret  = Temperature(AirH2O, h=ret_in.h, P=ret_in.P, W=ret_in.W)
g_diff = 0.5 * (1 + tanh((T_ret - T_oa) / band))
g_lim  = 0.5 * (1 + tanh((lim - T_oa) / band))
\`\`\`

### \`enthalpy\`

\`\`\`
g_diff = 0.5 * (1 + tanh((ret_in.h - oa_in.h) / band))
g_lim  = 0.5 * (1 + tanh((lim - oa_in.h) / band))
\`\`\``,
  },
  {
    name: `EnthalpyWheel`,
    slug: `enthalpywheel`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component EnthalpyWheel with ports sup_in, sup_out, exh_in, exh_out.`,
    related: [],
    examples: [],
    tags: [`enthalpywheel`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
EnthalpyWheel inst(eff_h, eff_w, domain$)
\`\`\`

## Ports

\`sup_in\`, \`sup_out\`, \`exh_in\`, \`exh_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff_h\` | Number |
| \`eff_w\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
sup_out.mdot = sup_in.mdot
exh_out.mdot = exh_in.mdot
sup_out.P    = sup_in.P
exh_out.P    = exh_in.P
sup_out.W    = sup_in.W + eff_w * (exh_in.W - sup_in.W)
sup_out.h    = sup_in.h + eff_h * (exh_in.h - sup_in.h)
exh_out.W    = exh_in.W - (sup_in.mdot / exh_in.mdot) * (sup_out.W - sup_in.W)
exh_out.h    = exh_in.h - (sup_in.mdot / exh_in.mdot) * (sup_out.h - sup_in.h)
\`\`\``,
  },
  {
    name: `EvaporativeCooler`,
    slug: `evaporativecooler`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component EvaporativeCooler with ports in, out.`,
    related: [],
    examples: [],
    tags: [`evaporativecooler`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
EvaporativeCooler inst(eff, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.h    = in.h
W_sat    = HumRat(AirH2O, h=in.h, P=in.P, R=1)
out.W    = in.W + eff * (W_sat - in.W)
\`\`\``,
  },
  {
    name: `FaceAndBypassCoil`,
    slug: `faceandbypasscoil`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component FaceAndBypassCoil with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`faceandbypasscoil`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
FaceAndBypassCoil inst(u_face, eps, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`u_face\` | Number |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
T_in      = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
T_face    = T_in - eps * (T_in - wall.T)
W_sat     = HumRat(AirH2O, T=T_face, P=in.P, R=1)
W_face    = 0.5 * (in.W + W_sat - sqrt((in.W - W_sat)^2 + 1e-12))
h_face    = Enthalpy(AirH2O, T=T_face, P=in.P, W=W_face)
out.W     = u_face * W_face + (1 - u_face) * in.W
out.h     = u_face * h_face + (1 - u_face) * in.h
BF        = 1 - u_face
Q         = in.mdot * (in.h - out.h)
wall.Qdot = -Q
\`\`\``,
  },
  {
    name: `FanCoilUnit`,
    slug: `fancoilunit`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component FanCoilUnit with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`fancoilunit`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
FanCoilUnit inst(K, foul, dP, eta, eps)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`K\` | Number |
| \`foul\` | Number |
| \`dP\` | Number |
| \`eta\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
AirFilter      FL(K=K, foul=foul)
MoistAirFan    FN(dP=dP, eta=eta)
MoistAirWallHX CO(model$=eps_t, eps=eps)
connect(in, FL.in)
connect(FL.out, FN.in)
connect(FN.out, CO.in)
connect(CO.out, out)
connect(wall, CO.wall)
\`\`\``,
  },
  {
    name: `FanPoweredBox`,
    slug: `fanpoweredbox`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component FanPoweredBox with ports pri_in, ind_in, out.`,
    related: [],
    examples: [],
    tags: [`fanpoweredbox`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
FanPoweredBox inst(Q_fan, Q_reheat, domain$)
\`\`\`

## Ports

\`pri_in\`, \`ind_in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Q_fan\` | Number |
| \`Q_reheat\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.P    = pri_in.P
out.mdot = pri_in.mdot + ind_in.mdot
out.mdot * out.W = pri_in.mdot * pri_in.W + ind_in.mdot * ind_in.W
out.mdot * out.h = pri_in.mdot * pri_in.h + ind_in.mdot * ind_in.h + Q_fan + Q_reheat
\`\`\``,
  },
  {
    name: `HeatingCoil`,
    slug: `heatingcoil`,
    category: `Component (moistair)`,
    summary: `Heats a humid-air stream at constant humidity ratio.`,
    related: [],
    examples: [],
    tags: [`heatingcoil`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Heats a humid-air stream at constant humidity ratio.

## Domain

A reusable **acausal moistair-domain** component — its humid-air ports carry pressure \`P\`, dry-air mass-flow \`ṁ_da\`, enthalpy \`h\`, and humidity ratio \`W\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
HeatingCoil inst(Q, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Q\` | Number | Heat input [W]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.W    = in.W
out.h    = in.h + Q / in.mdot
\`\`\``,
  },
  {
    name: `HeatPipeWrapAround`,
    slug: `heatpipewraparound`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component HeatPipeWrapAround with ports pre_in, pre_out, re_in, re_out.`,
    related: [],
    examples: [],
    tags: [`heatpipewraparound`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HeatPipeWrapAround inst(eff, domain$)
\`\`\`

## Ports

\`pre_in\`, \`pre_out\`, \`re_in\`, \`re_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
pre_out.mdot = pre_in.mdot
re_out.mdot  = re_in.mdot
pre_out.P    = pre_in.P
re_out.P     = re_in.P
pre_out.W    = pre_in.W
re_out.W     = re_in.W
T_p_in    = Temperature(AirH2O, h=pre_in.h, P=pre_in.P, W=pre_in.W)
T_r_in    = Temperature(AirH2O, h=re_in.h, P=re_in.P, W=re_in.W)
T_p_out   = T_p_in - eff * (T_p_in - T_r_in)
pre_out.h = Enthalpy(AirH2O, T=T_p_out, P=pre_in.P, W=pre_out.W)
Q         = pre_in.mdot * (pre_in.h - pre_out.h)
re_out.h  = re_in.h + Q / re_in.mdot
T_r_out   = Temperature(AirH2O, h=re_out.h, P=re_in.P, W=re_out.W)
\`\`\``,
  },
  {
    name: `Humidifier`,
    slug: `humidifier`,
    category: `Component (moistair)`,
    summary: `Adds moisture to a humid-air stream, raising its humidity ratio.`,
    related: [],
    examples: [],
    tags: [`humidifier`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Adds moisture to a humid-air stream, raising its humidity ratio.

## Domain

A reusable **acausal moistair-domain** component — its humid-air ports carry pressure \`P\`, dry-air mass-flow \`ṁ_da\`, enthalpy \`h\`, and humidity ratio \`W\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Humidifier inst(mdot_w, h_w, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`mdot_w\` | Number | Water/coolant mass flow [kg/s]. |
| \`h_w\` | Number | Wall heat-transfer coefficient [W/m²·K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.W    = in.W + mdot_w / in.mdot
out.h    = in.h + mdot_w * h_w / in.mdot
\`\`\``,
  },
  {
    name: `IndirectDirectEvaporativeCooler`,
    slug: `indirectdirectevaporativecooler`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component IndirectDirectEvaporativeCooler with ports pri_in, pri_out, sec_in, sec_out.`,
    related: [],
    examples: [],
    tags: [`indirectdirectevaporativecooler`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
IndirectDirectEvaporativeCooler inst(wbde, eff_sec, eff_dir)
\`\`\`

## Ports

\`pri_in\`, \`pri_out\`, \`sec_in\`, \`sec_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`wbde\` | Number |
| \`eff_sec\` | Number |
| \`eff_dir\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
IndirectEvaporativeCooler IEC(wbde=wbde, eff_sec=eff_sec)
EvaporativeCooler         DEC(eff=eff_dir)
connect(pri_in, IEC.pri_in)
connect(IEC.pri_out, DEC.in)
connect(DEC.out, pri_out)
connect(sec_in, IEC.sec_in)
connect(IEC.sec_out, sec_out)
\`\`\``,
  },
  {
    name: `IndirectEvaporativeCooler`,
    slug: `indirectevaporativecooler`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component IndirectEvaporativeCooler with ports pri_in, pri_out, sec_in, sec_out.`,
    related: [],
    examples: [],
    tags: [`indirectevaporativecooler`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
IndirectEvaporativeCooler inst(wbde, eff_sec, domain$)
\`\`\`

## Ports

\`pri_in\`, \`pri_out\`, \`sec_in\`, \`sec_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`wbde\` | Number |
| \`eff_sec\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
pri_out.mdot = pri_in.mdot
sec_out.mdot = sec_in.mdot
pri_out.P    = pri_in.P
sec_out.P    = sec_in.P
pri_out.W    = pri_in.W
T_p_in   = Temperature(AirH2O, h=pri_in.h, P=pri_in.P, W=pri_in.W)
T_wb_sec = WetBulb(AirH2O, h=sec_in.h, P=sec_in.P, W=sec_in.W)
T_p_out  = T_p_in - wbde * (T_p_in - T_wb_sec)
pri_out.h = Enthalpy(AirH2O, T=T_p_out, P=pri_in.P, W=pri_out.W)
Q         = pri_in.mdot * (pri_in.h - pri_out.h)
W_sat_sec = HumRat(AirH2O, h=sec_in.h, P=sec_in.P, R=1)
sec_out.W = sec_in.W + eff_sec * (W_sat_sec - sec_in.W)
sec_out.h = sec_in.h + Q / sec_in.mdot
\`\`\``,
  },
  {
    name: `InductionUnit`,
    slug: `inductionunit`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component InductionUnit with ports pri_in, ind_in, out, wall.`,
    related: [],
    examples: [],
    tags: [`inductionunit`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
InductionUnit inst(ratio, eps, domain$)
\`\`\`

## Ports

\`pri_in\`, \`ind_in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ratio\` | Number |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
ind_in.mdot = ratio * pri_in.mdot
out.mdot    = pri_in.mdot + ind_in.mdot
out.P       = pri_in.P
T_i_in      = Temperature(AirH2O, h=ind_in.h, P=ind_in.P, W=ind_in.W)
T_i_out     = T_i_in - eps * (T_i_in - wall.T)
h_i_out     = Enthalpy(AirH2O, T=T_i_out, P=ind_in.P, W=ind_in.W)
out.mdot * out.W = pri_in.mdot * pri_in.W + ind_in.mdot * ind_in.W
out.mdot * out.h = pri_in.mdot * pri_in.h + ind_in.mdot * h_i_out
Q           = ind_in.mdot * (ind_in.h - h_i_out)
wall.Qdot   = -Q
T_dp_ind    = DewPoint(AirH2O, h=ind_in.h, P=ind_in.P, W=ind_in.W)
margin_dp   = wall.T - T_dp_ind
\`\`\``,
  },
  {
    name: `Infiltration`,
    slug: `infiltration`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component Infiltration with ports in, out.`,
    related: [],
    examples: [],
    tags: [`infiltration`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Infiltration inst(C_inf, n_exp, eps, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`C_inf\` | Number |
| \`n_exp\` | Number |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
dP       = in.P - out.P
in.mdot  = C_inf * dP * (dP^2 + eps^2)^((n_exp - 1) / 2)
out.mdot = in.mdot
out.W    = in.W
out.h    = in.h
\`\`\``,
  },
  {
    name: `LiquidDesiccantContactor`,
    slug: `liquiddesiccantcontactor`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component LiquidDesiccantContactor with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`liquiddesiccantcontactor`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
LiquidDesiccantContactor inst(eff_L, W_eq, eps_T, f_excess, domain$, model$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff_L\` | Number |
| \`W_eq\` | Number |
| \`eps_T\` | Number |
| \`f_excess\` | Number |
| \`domain$\` | String |
| \`model$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.W    = in.W - eff_L * (in.W - W_eq)
T_in     = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
mdot_w   = in.mdot * (in.W - out.W)
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`cooled\` — requires \`eps_T\`

\`\`\`
T_out     = T_in - eps_T * (T_in - wall.T)
out.h     = Enthalpy(AirH2O, T=T_out, P=in.P, W=out.W)
h_f       = 4186 * (wall.T - 273.15)
Q         = in.mdot * (in.h - out.h) - mdot_w * h_f
wall.Qdot = -Q
\`\`\`

### \`adiabatic\` — requires \`f_excess\`

\`\`\`
h_f       = 4186 * (T_in - 273.15)
h_pure    = in.h - mdot_w * h_f / in.mdot
T_pure    = Temperature(AirH2O, h=h_pure, P=in.P, W=out.W)
T_out     = T_in + (1 + f_excess) * (T_pure - T_in)
out.h     = Enthalpy(AirH2O, T=T_out, P=in.P, W=out.W)
Q         = 0
wall.Qdot = 0
\`\`\``,
  },
  {
    name: `MembraneHumidifier`,
    slug: `membranehumidifier`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component MembraneHumidifier with ports dry_in, dry_out, wet_in, wet_out.`,
    related: [],
    examples: [],
    tags: [`membranehumidifier`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
MembraneHumidifier inst(eff_h, eff_w, domain$)
\`\`\`

## Ports

\`dry_in\`, \`dry_out\`, \`wet_in\`, \`wet_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff_h\` | Number |
| \`eff_w\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
dry_out.mdot = dry_in.mdot
wet_out.mdot = wet_in.mdot
dry_out.P    = dry_in.P
wet_out.P    = wet_in.P
dry_out.W    = dry_in.W + eff_w * (wet_in.W - dry_in.W)
dry_out.h    = dry_in.h + eff_h * (wet_in.h - dry_in.h)
wet_out.W    = wet_in.W - (dry_in.mdot / wet_in.mdot) * (dry_out.W - dry_in.W)
wet_out.h    = wet_in.h - (dry_in.mdot / wet_in.mdot) * (dry_out.h - dry_in.h)
\`\`\``,
  },
  {
    name: `MixingBox`,
    slug: `mixingbox`,
    category: `Component (moistair)`,
    summary: `Mixes two humid-air streams with flow-weighted enthalpy and humidity ratio.`,
    related: [],
    examples: [],
    tags: [`mixingbox`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Mixes two humid-air streams with flow-weighted enthalpy and humidity ratio.

## Domain

A reusable **acausal moistair-domain** component — its humid-air ports carry pressure \`P\`, dry-air mass-flow \`ṁ_da\`, enthalpy \`h\`, and humidity ratio \`W\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in1\`, \`in2\`, \`out\`

## Usage

\`\`\`
MixingBox inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P    = in1.P
out.mdot = in1.mdot + in2.mdot
out.mdot * out.W = in1.mdot * in1.W + in2.mdot * in2.W
out.mdot * out.h = in1.mdot * in1.h + in2.mdot * in2.h
\`\`\``,
  },
  {
    name: `MoistAirDamper`,
    slug: `moistairdamper`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component MoistAirDamper with ports in, outa, outb.`,
    related: [],
    examples: [],
    tags: [`moistairdamper`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
MoistAirDamper inst(u, domain$)
\`\`\`

## Ports

\`in\`, \`outa\`, \`outb\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`u\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
outa.P    = in.P
outb.P    = in.P
outa.h    = in.h
outb.h    = in.h
outa.W    = in.W
outb.W    = in.W
outa.mdot = u * in.mdot
outb.mdot = (1 - u) * in.mdot
\`\`\``,
  },
  {
    name: `MoistAirDuct`,
    slug: `moistairduct`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component MoistAirDuct with ports in, out.`,
    related: [],
    examples: [],
    tags: [`moistairduct`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
MoistAirDuct inst(L, D, rough, mu_a, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`L\` | Number |
| \`D\` | Number |
| \`rough\` | Number |
| \`mu_a\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.W    = in.W
out.h    = in.h
rho      = 1 / Volume(AirH2O, h=in.h, P=in.P, W=in.W)
A        = pi# / 4 * D^2
V        = in.mdot * (1 + in.W) / (rho * A)
Re_d     = reynolds(rho, V, D, mu_a)
f        = friction_factor(Re_d, rough / D)
out.P    = in.P - f * (L / D) * rho * V^2 / 2
\`\`\``,
  },
  {
    name: `MoistAirFan`,
    slug: `moistairfan`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component MoistAirFan with ports in, out.`,
    related: [],
    examples: [],
    tags: [`moistairfan`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
MoistAirFan inst(dP, eta, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`dP\` | Number |
| \`eta\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.W    = in.W
out.P    = in.P + dP
T_in     = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
v_in     = Volume(AirH2O, T=T_in, P=in.P, W=in.W)
out.h    = in.h + v_in * dP / eta
W_el     = in.mdot * v_in * dP / eta
\`\`\``,
  },
  {
    name: `MoistAirSink`,
    slug: `moistairsink`,
    category: `Component (moistair)`,
    summary: `A humid-air boundary absorbing a stream.`,
    related: [],
    examples: [],
    tags: [`moistairsink`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `A humid-air boundary absorbing a stream.

## Domain

A reusable **acausal moistair-domain** component — its humid-air ports carry pressure \`P\`, dry-air mass-flow \`ṁ_da\`, enthalpy \`h\`, and humidity ratio \`W\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`

## Usage

\`\`\`
MoistAirSink inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
mdot = in.mdot
P    = in.P
h    = in.h
W    = in.W
\`\`\``,
  },
  {
    name: `MoistAirSource`,
    slug: `moistairsource`,
    category: `Component (moistair)`,
    summary: `A humid-air boundary supplying a stream of set state.`,
    related: [],
    examples: [],
    tags: [`moistairsource`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `A humid-air boundary supplying a stream of set state.

## Domain

A reusable **acausal moistair-domain** component — its humid-air ports carry pressure \`P\`, dry-air mass-flow \`ṁ_da\`, enthalpy \`h\`, and humidity ratio \`W\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
MoistAirSource inst(P, T, W, mdot, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`P\` | Number | Pressure [Pa]. |
| \`T\` | Number | Temperature [K]. |
| \`W\` | Number | Humidity ratio [kg/kg] / work [W]. |
| \`mdot\` | Number | Mass flow rate [kg/s]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P    = P
out.mdot = mdot
out.W    = W
out.h    = Enthalpy(AirH2O, T=T, P=P, W=W)
\`\`\``,
  },
  {
    name: `MoistAirWallHX`,
    slug: `moistairwallhx`,
    category: `Component (moistair)`,
    summary: `A humid-air-to-wall heat exchanger.`,
    related: [],
    examples: [],
    tags: [`moistairwallhx`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `A humid-air-to-wall heat exchanger.

## Domain

A reusable **acausal moistair-domain** component — its humid-air ports carry pressure \`P\`, dry-air mass-flow \`ṁ_da\`, enthalpy \`h\`, and humidity ratio \`W\`. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`wall\`

## Usage

\`\`\`
MoistAirWallHX inst(eps, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`eps\` | Number | Effectiveness / roughness. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
T_in      = Temperature(AirH2O, h=in.h, P=in.P, W=in.W)
T_out     = T_in - eps * (T_in - wall.T)
W_sat     = HumRat(AirH2O, T=T_out, P=in.P, R=1)
out.W     = 0.5 * (in.W + W_sat - sqrt((in.W - W_sat)^2 + 1e-12))
out.h     = Enthalpy(AirH2O, T=T_out, P=in.P, W=out.W)
Q         = in.mdot * (in.h - out.h)
Q_lat     = in.mdot * 2.501e6 * (in.W - out.W)
wall.Qdot = -Q
\`\`\``,
  },
  {
    name: `RadiantPanel`,
    slug: `radiantpanel`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component RadiantPanel with ports zone, wall.`,
    related: [],
    examples: [],
    tags: [`radiantpanel`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
RadiantPanel inst(A, C, n, eps_dT, W_room, P_room)
\`\`\`

## Ports

\`zone\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`A\` | Number |
| \`C\` | Number |
| \`n\` | Number |
| \`eps_dT\` | Number |
| \`W_room\` | Number |
| \`P_room\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
dT        = zone.T - wall.T
q_flux    = C * dT * (dT^2 + eps_dT^2)^((n - 1) / 2)
Q         = A * q_flux
zone.Qdot = Q
wall.Qdot = -Q
T_dp_room = DewPoint(AirH2O, T=zone.T, P=P_room, W=W_room)
margin_dp = wall.T - T_dp_room
\`\`\``,
  },
  {
    name: `SensibleAirToAirHX`,
    slug: `sensibleairtoairhx`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component SensibleAirToAirHX with ports sup_in, sup_out, exh_in, exh_out.`,
    related: [],
    examples: [],
    tags: [`sensibleairtoairhx`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SensibleAirToAirHX inst(eff, eatr, oacf, domain$)
\`\`\`

## Ports

\`sup_in\`, \`sup_out\`, \`exh_in\`, \`exh_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eff\` | Number |
| \`eatr\` | Number |
| \`oacf\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
sup_out.mdot = sup_in.mdot / oacf
exh_out.mdot = exh_in.mdot + (sup_in.mdot - sup_out.mdot)
sup_out.P    = sup_in.P
exh_out.P    = exh_in.P
T_s_in  = Temperature(AirH2O, h=sup_in.h, P=sup_in.P, W=sup_in.W)
T_e_in  = Temperature(AirH2O, h=exh_in.h, P=exh_in.P, W=exh_in.W)
C_s     = sup_in.mdot * Cp(AirH2O, T=T_s_in, P=sup_in.P, W=sup_in.W)
C_e     = exh_in.mdot * Cp(AirH2O, T=T_e_in, P=exh_in.P, W=exh_in.W)
Q       = eff * min(C_s, C_e) * (T_e_in - T_s_in)
T_s_out = T_s_in + Q / C_s
sup_out.W = sup_in.W + eatr * (exh_in.W - sup_in.W)
sup_out.h = Enthalpy(AirH2O, T=T_s_out, P=sup_in.P, W=sup_out.W)
exh_out.mdot * exh_out.W = exh_in.mdot * exh_in.W + sup_in.mdot * sup_in.W - sup_out.mdot * sup_out.W
exh_out.mdot * exh_out.h = exh_in.mdot * exh_in.h + sup_in.mdot * sup_in.h - sup_out.mdot * sup_out.h
T_e_out = Temperature(AirH2O, h=exh_out.h, P=exh_in.P, W=exh_out.W)
\`\`\``,
  },
  {
    name: `SteamHumidifier`,
    slug: `steamhumidifier`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component SteamHumidifier with ports in, out.`,
    related: [],
    examples: [],
    tags: [`steamhumidifier`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SteamHumidifier inst(W_set, h_steam, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`W_set\` | Number |
| \`h_steam\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.W    = W_set
mdot_w   = in.mdot * (W_set - in.W)
out.h    = in.h + mdot_w * h_steam / in.mdot
\`\`\``,
  },
  {
    name: `TotalEnergyExchanger`,
    slug: `totalenergyexchanger`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component TotalEnergyExchanger with ports sup_in, sup_out, exh_in, exh_out.`,
    related: [],
    examples: [],
    tags: [`totalenergyexchanger`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from this port's component library (\`crates/frees-core/src/components/library-data/\`). The ports, parameters, and variants are taken from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TotalEnergyExchanger inst(eps_s, eps_L, eatr, oacf, domain$)
\`\`\`

## Ports

\`sup_in\`, \`sup_out\`, \`exh_in\`, \`exh_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eps_s\` | Number |
| \`eps_L\` | Number |
| \`eatr\` | Number |
| \`oacf\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
sup_out.mdot = sup_in.mdot / oacf
exh_out.mdot = exh_in.mdot + (sup_in.mdot - sup_out.mdot)
sup_out.P    = sup_in.P
exh_out.P    = exh_in.P
T_s_in  = Temperature(AirH2O, h=sup_in.h, P=sup_in.P, W=sup_in.W)
T_e_in  = Temperature(AirH2O, h=exh_in.h, P=exh_in.P, W=exh_in.W)
W_x     = sup_in.W + eps_L * (exh_in.W - sup_in.W)
T_s_out = T_s_in + eps_s * (T_e_in - T_s_in)
sup_out.W = W_x + eatr * (exh_in.W - W_x)
sup_out.h = Enthalpy(AirH2O, T=T_s_out, P=sup_in.P, W=sup_out.W)
exh_out.mdot * exh_out.W = exh_in.mdot * exh_in.W + sup_in.mdot * sup_in.W - sup_out.mdot * sup_out.W
exh_out.mdot * exh_out.h = exh_in.mdot * exh_in.h + sup_in.mdot * sup_in.h - sup_out.mdot * sup_out.h
\`\`\``,
  },
  {
    name: `VAVBox`,
    slug: `vavbox`,
    category: `Component (moistair)`,
    summary: `Acausal moistair-domain component VAVBox with ports in, out, u, ur.`,
    related: [],
    examples: [],
    tags: [`vavbox`, `component`, `moistair`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **moistair-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
VAVBox inst(mdot_max, Qr_max, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`u\`, \`ur\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`mdot_max\` | Number |
| \`Qr_max\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
in.mdot  = u.sig * mdot_max
out.mdot = in.mdot
out.W    = in.W
out.h    = in.h + ur.sig * Qr_max / in.mdot
\`\`\``,
  },
  {
    name: `AnodeRecirc`,
    slug: `anoderecirc`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component AnodeRecirc with ports sup_in, ret_in, out.`,
    related: [],
    examples: [],
    tags: [`anoderecirc`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
AnodeRecirc inst(fluid$, C, b, ER, domain$)
\`\`\`

## Ports

\`sup_in\`, \`ret_in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`C\` | Number |
| \`b\` | Number |
| \`ER\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
T_s         = Temperature(fluid$, P=sup_in.P, h=sup_in.h)
sup_in.mdot = iso6358(C, b, sup_in.P, T_s, out.P)
ret_in.mdot = ER * sup_in.mdot
out.mdot    = sup_in.mdot + ret_in.mdot
out.mdot * out.h = sup_in.mdot * sup_in.h + ret_in.mdot * ret_in.h
\`\`\``,
  },
  {
    name: `GasMixer`,
    slug: `gasmixer`,
    category: `Component (pneumatic)`,
    summary: `Mixes pneumatic gas streams, carrying the species composition rider.`,
    related: [],
    examples: [],
    tags: [`gasmixer`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `Mixes pneumatic gas streams, carrying the species composition rider.

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in1\`, \`in2\`, \`out\`

## Usage

\`\`\`
GasMixer inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P    = in1.P
out.mdot = in1.mdot + in2.mdot
out.mdot * out.h = in1.mdot * in1.h + in2.mdot * in2.h
out.mdot * out.y = in1.mdot * in1.y + in2.mdot * in2.y
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `GasMixerN`,
    slug: `gasmixern`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component GasMixerN with ports in1, in2, out.`,
    related: [],
    examples: [],
    tags: [`gasmixern`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
GasMixerN inst(domain$)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.P    = in1.P
out.mdot = in1.mdot + in2.mdot
out.mdot * out.h    = in1.mdot * in1.h    + in2.mdot * in2.h
out.mdot * out.yo2  = in1.mdot * in1.yo2  + in2.mdot * in2.yo2
out.mdot * out.yco2 = in1.mdot * in1.yco2 + in2.mdot * in2.yco2
out.mdot * out.yh2o = in1.mdot * in1.yh2o + in2.mdot * in2.yh2o
out.mdot * out.yn2  = in1.mdot * in1.yn2  + in2.mdot * in2.yn2
\`\`\``,
  },
  {
    name: `GasPipe`,
    slug: `gaspipe`,
    category: `Component (pneumatic)`,
    summary: `A pneumatic pipe with compressible-flow pressure drop.`,
    related: [],
    examples: [],
    tags: [`gaspipe`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `A pneumatic pipe with compressible-flow pressure drop.

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
GasPipe inst(...)
\`\`\`

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.h    = in.h
out.y    = in.y
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `GasSource`,
    slug: `gassource`,
    category: `Component (pneumatic)`,
    summary: `A boundary supplying gas at set conditions.`,
    related: [],
    examples: [],
    tags: [`gassource`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `A boundary supplying gas at set conditions.

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
GasSource inst(y, mdot, P, h0)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Position / fraction. |
| \`mdot\` | Number | Mass flow rate [kg/s]. |
| \`P\` | Number | Pressure [Pa]. |
| \`h0\` | Number | Reference enthalpy [J/kg]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.y    = y
out.mdot = mdot
out.P    = P
out.h    = h0
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `PneumaticActuator`,
    slug: `pneumaticactuator`,
    category: `Component (pneumatic)`,
    summary: `A pneumatic cylinder/actuator converting pressure to force.`,
    related: [],
    examples: [],
    tags: [`pneumaticactuator`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `A pneumatic cylinder/actuator converting pressure to force.

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`rod\`

## Usage

\`\`\`
PneumaticActuator inst(fluid$, area, Patm, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`area\` | Number | Area [m²]. |
| \`Patm\` | Number | Atmospheric pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
rho     = Density(fluid$, P=in.P, h=in.h)
rod.f   = -(in.P - Patm) * area
in.mdot = rho * area * rod.vel
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `PneumaticAtmosphere`,
    slug: `pneumaticatmosphere`,
    category: `Component (pneumatic)`,
    summary: `An atmospheric (ambient-pressure) pneumatic boundary.`,
    related: [],
    examples: [],
    tags: [`pneumaticatmosphere`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `An atmospheric (ambient-pressure) pneumatic boundary.

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`port\`

## Usage

\`\`\`
PneumaticAtmosphere inst(P, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`P\` | Number | Pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
port.P = P
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `PneumaticCheckValve`,
    slug: `pneumaticcheckvalve`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component PneumaticCheckValve with ports in, out.`,
    related: [],
    examples: [],
    tags: [`pneumaticcheckvalve`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PneumaticCheckValve inst(fluid$, C, b, eps, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`C\` | Number |
| \`b\` | Number |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
g        = 0.5 * (1 + tanh((in.P - out.P) / eps))
T_in     = Temperature(fluid$, P=in.P, h=in.h)
in.mdot  = g * iso6358(C, b, in.P, T_in, out.P)
\`\`\``,
  },
  {
    name: `PneumaticDoubleActingCylinder`,
    slug: `pneumaticdoubleactingcylinder`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component PneumaticDoubleActingCylinder with ports a, b, rod.`,
    related: [],
    examples: [],
    tags: [`pneumaticdoubleactingcylinder`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PneumaticDoubleActingCylinder inst(Aa, Ab, R, T, Va0, Vb0, Pa0, Pb0, domain$)
\`\`\`

## Ports

\`a\`, \`b\`, \`rod\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Aa\` | Number |
| \`Ab\` | Number |
| \`R\` | Number |
| \`T\` | Number |
| \`Va0\` | Number |
| \`Vb0\` | Number |
| \`Pa0\` | Number |
| \`Pb0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
rod.f     = -(a.P * Aa - b.P * Ab)
der(a.P)  = (R * T * a.mdot - a.P * Aa * rod.vel) / Va0
init(a.P) = Pa0
der(b.P)  = (R * T * b.mdot + b.P * Ab * rod.vel) / Vb0
init(b.P) = Pb0
\`\`\``,
  },
  {
    name: `PneumaticOrifice`,
    slug: `pneumaticorifice`,
    category: `Component (pneumatic)`,
    summary: `A pneumatic orifice metering flow by ISO 6358 (sonic conductance).`,
    related: [],
    examples: [],
    tags: [`pneumaticorifice`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `A pneumatic orifice metering flow by ISO 6358 (sonic conductance).

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
PneumaticOrifice inst(fluid$, C, b, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`C\` | Number | Capacitance [F]. |
| \`b\` | Number | Critical pressure ratio / coefficient. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.h    = in.h
T_in     = Temperature(fluid$, P=in.P, h=in.h)
in.mdot  = iso6358(C, b, in.P, T_in, out.P)
out.mdot = in.mdot
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `PneumaticServoValve`,
    slug: `pneumaticservovalve`,
    category: `Component (pneumatic)`,
    summary: `A pneumatic servo valve with a commanded spool position.`,
    related: [],
    examples: [],
    tags: [`pneumaticservovalve`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `A pneumatic servo valve with a commanded spool position.

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
PneumaticServoValve inst(fluid$, Cmax, b, u, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`Cmax\` | Number | Maximum capacity rate [W/K]. |
| \`b\` | Number | Critical pressure ratio / coefficient. |
| \`u\` | Number | Specific internal energy [J/kg]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.h    = in.h
T_in     = Temperature(fluid$, P=in.P, h=in.h)
in.mdot  = iso6358(u * Cmax, b, in.P, T_in, out.P)
out.mdot = in.mdot
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `PneumaticServoValveCmd`,
    slug: `pneumaticservovalvecmd`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component PneumaticServoValveCmd with ports in, out, u.`,
    related: [],
    examples: [],
    tags: [`pneumaticservovalvecmd`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PneumaticServoValveCmd inst(fluid$, Cmax, b, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`Cmax\` | Number |
| \`b\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.h    = in.h
T_in     = Temperature(fluid$, P=in.P, h=in.h)
in.mdot  = iso6358(u.sig * Cmax, b, in.P, T_in, out.P)
out.mdot = in.mdot
\`\`\``,
  },
  {
    name: `PneumaticSupply`,
    slug: `pneumaticsupply`,
    category: `Component (pneumatic)`,
    summary: `A pneumatic pressure supply.`,
    related: [],
    examples: [],
    tags: [`pneumaticsupply`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `A pneumatic pressure supply.

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
PneumaticSupply inst(fluid$, P, T, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`P\` | Number | Pressure [Pa]. |
| \`T\` | Number | Temperature [K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P = P
out.h = Enthalpy(fluid$, P=P, T=T)
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `PneumaticThermalVolume`,
    slug: `pneumaticthermalvolume`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component PneumaticThermalVolume with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`pneumaticthermalvolume`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PneumaticThermalVolume inst(fluid$, V, R, cv, cp, m0, T0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`V\` | Number |
| \`R\` | Number |
| \`cv\` | Number |
| \`cp\` | Number |
| \`m0\` | Number |
| \`T0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(m)  = in.mdot - out.mdot
init(m) = m0
T_in    = Temperature(fluid$, P=in.P, h=in.h)
der(T)  = (in.mdot * cp * (T_in - T) + wall.Qdot) / (m * cv)
init(T) = T0
in.P    = m * R * T / V
out.P   = in.P
out.h   = Enthalpy(fluid$, P=out.P, T=T)
wall.T  = T
\`\`\``,
  },
  {
    name: `PneumaticValve32`,
    slug: `pneumaticvalve32`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component PneumaticValve32 with ports sup_in, work, exh_out, u.`,
    related: [],
    examples: [],
    tags: [`pneumaticvalve32`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PneumaticValve32 inst(fluid$, C, b, domain$)
\`\`\`

## Ports

\`sup_in\`, \`work\`, \`exh_out\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`C\` | Number |
| \`b\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
T_s          = Temperature(fluid$, P=sup_in.P, h=sup_in.h)
T_w          = Temperature(fluid$, P=work.P, h=work.h)
m_in         = iso6358(u.sig * C, b, sup_in.P, T_s, work.P)
m_out        = iso6358((1 - u.sig) * C, b, work.P, T_w, exh_out.P)
sup_in.mdot  = m_in
work.mdot    = m_in - m_out
exh_out.mdot = m_out
work.h       = sup_in.h
exh_out.h    = work.h
\`\`\``,
  },
  {
    name: `PneumaticValve52`,
    slug: `pneumaticvalve52`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component PneumaticValve52 with ports sup_in, wa, wb, ea_out, eb_out, u.`,
    related: [],
    examples: [],
    tags: [`pneumaticvalve52`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
PneumaticValve52 inst(fluid$, C, b, domain$)
\`\`\`

## Ports

\`sup_in\`, \`wa\`, \`wb\`, \`ea_out\`, \`eb_out\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`C\` | Number |
| \`b\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
T_s         = Temperature(fluid$, P=sup_in.P, h=sup_in.h)
T_a         = Temperature(fluid$, P=wa.P, h=wa.h)
T_b         = Temperature(fluid$, P=wb.P, h=wb.h)
m_sa        = iso6358(u.sig * C, b, sup_in.P, T_s, wa.P)
m_be        = iso6358(u.sig * C, b, wb.P, T_b, eb_out.P)
m_sb        = iso6358((1 - u.sig) * C, b, sup_in.P, T_s, wb.P)
m_ae        = iso6358((1 - u.sig) * C, b, wa.P, T_a, ea_out.P)
sup_in.mdot = m_sa + m_sb
wa.mdot     = m_sa - m_ae
wb.mdot     = m_sb - m_be
ea_out.mdot = m_ae
eb_out.mdot = m_be
wa.h        = sup_in.h
wb.h        = sup_in.h
ea_out.h    = wa.h
eb_out.h    = wb.h
\`\`\``,
  },
  {
    name: `PneumaticVolume`,
    slug: `pneumaticvolume`,
    category: `Component (pneumatic)`,
    summary: `A pneumatic control volume (compressible capacitance).`,
    related: [],
    examples: [],
    tags: [`pneumaticvolume`, `component`, `pneumatic`, `acausal`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `A pneumatic control volume (compressible capacitance).

## Domain

A reusable **acausal pneumatic-domain** component — its compressible-gas ports carry pressure \`P\`, mass-flow \`ṁ\`, and enthalpy \`h\` (ISO 6358 flow). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
PneumaticVolume inst(V, T, R, P0, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`V\` | Number | Volume [m³]. |
| \`T\` | Number | Temperature [K]. |
| \`R\` | Number | Resistance [Ω]. |
| \`P0\` | Number | Reference/initial pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P      = in.P
out.h      = in.h
der(in.P)  = (R * T / V) * (in.mdot - out.mdot)
init(in.P) = P0
\`\`\`

## References

1. ISO 6358 — Pneumatic fluid power: flow-rate characteristics.`,
  },
  {
    name: `VacuumEjector`,
    slug: `vacuumejector`,
    category: `Component (pneumatic)`,
    summary: `Acausal pneumatic-domain component VacuumEjector with ports sup_in, suc_in, exh_out.`,
    related: [],
    examples: [],
    tags: [`vacuumejector`, `component`, `pneumatic`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **pneumatic-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
VacuumEjector inst(fluid$, C, b, ER, domain$)
\`\`\`

## Ports

\`sup_in\`, \`suc_in\`, \`exh_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`C\` | Number |
| \`b\` | Number |
| \`ER\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
T_s          = Temperature(fluid$, P=sup_in.P, h=sup_in.h)
sup_in.mdot  = iso6358(C, b, sup_in.P, T_s, exh_out.P)
suc_in.mdot  = ER * sup_in.mdot
exh_out.mdot = sup_in.mdot + suc_in.mdot
exh_out.mdot * exh_out.h = sup_in.mdot * sup_in.h + suc_in.mdot * suc_in.h
\`\`\``,
  },
  {
    name: `AutomaticTransmission`,
    slug: `automatictransmission`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component AutomaticTransmission with ports in, out, gear, lock.`,
    related: [],
    examples: [],
    tags: [`automatictransmission`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
AutomaticTransmission inst(Kmap$, TRmap$, eta, Tlock, eps)
\`\`\`

## Ports

\`in\`, \`out\`, \`gear\`, \`lock\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Kmap$\` | String |
| \`TRmap$\` | String |
| \`eta\` | Number |
| \`Tlock\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
TorqueConverter  TC(Kmap$=Kmap$, TRmap$=TRmap$)
GearboxScheduled GB(eta=eta)
ClutchCmd        LU(Tmax=Tlock, eps=eps)
connect(in, TC.pump, LU.a)
connect(TC.turb, LU.b, GB.in)
connect(GB.out, out)
connect(gear, GB.u)
connect(lock, LU.u)
\`\`\``,
  },
  {
    name: `CatalystLightOff`,
    slug: `catalystlightoff`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component CatalystLightOff with ports in, out.`,
    related: [],
    examples: [],
    tags: [`catalystlightoff`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
CatalystLightOff inst(fluid$, C, UA, T50, k, q_exo, T0)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`C\` | Number |
| \`UA\` | Number |
| \`T50\` | Number |
| \`k\` | Number |
| \`q_exo\` | Number |
| \`T0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
T_g      = Temperature(fluid$, P=in.P, h=in.h)
eta      = 0.5 * (1 + tanh((Tb - T50) / k))
Q        = UA * (T_g - Tb)
Qexo     = eta * in.mdot * q_exo
der(Tb)  = (Q + Qexo) / C
init(Tb) = T0
out.mdot = in.mdot
out.P    = in.P
out.h    = in.h - Q / in.mdot
\`\`\``,
  },
  {
    name: `Differential`,
    slug: `differential`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component Differential with ports in, left, right.`,
    related: [],
    examples: [],
    tags: [`differential`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
Differential inst(ratio)
\`\`\`

## Ports

\`in\`, \`left\`, \`right\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ratio\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
in.w      = ratio * 0.5 * (left.w + right.w)
left.tau  = -0.5 * ratio * in.tau
right.tau = -0.5 * ratio * in.tau
\`\`\``,
  },
  {
    name: `DriveCycleSource`,
    slug: `drivecyclesource`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component DriveCycleSource with ports port.`,
    related: [],
    examples: [],
    tags: [`drivecyclesource`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
DriveCycleSource inst(map$)
\`\`\`

## Ports

\`port\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`map$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
port.vel = map$(time)
\`\`\``,
  },
  {
    name: `Engine`,
    slug: `engine`,
    category: `Component (powertrain)`,
    summary: `An internal-combustion engine acting as a torque source.`,
    related: [],
    examples: [`engine-map-2d`, `engine-cycle-wiebe`],
    tags: [`engine`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `An internal-combustion engine acting as a torque source.

## Domain

A reusable **acausal powertrain-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\`, with vehicle-level speed/force signals. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`shaft\`

## Usage

\`\`\`
Engine inst(Tmax, throttle, bf)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Tmax\` | Number | Maximum temperature [K]. |
| \`throttle\` | Number | Throttle (0–1). |
| \`bf\` | Number | Friction coefficient. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
shaft.tau = -(throttle * Tmax - bf * shaft.w)
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: engine-map-2d]`,
  },
  {
    name: `ExhaustPipeThermal`,
    slug: `exhaustpipethermal`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component ExhaustPipeThermal with ports in, out, amb.`,
    related: [],
    examples: [],
    tags: [`exhaustpipethermal`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ExhaustPipeThermal inst(fluid$, UA, hA, C1, C2, R, T10, T20)
\`\`\`

## Ports

\`in\`, \`out\`, \`amb\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`UA\` | Number |
| \`hA\` | Number |
| \`C1\` | Number |
| \`C2\` | Number |
| \`R\` | Number |
| \`T10\` | Number |
| \`T20\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
HeatedDuct D(fluid$=fluid$, UA=UA)
WallRC     W(C1=C1, C2=C2, R=R, T10=T10, T20=T20)
Convection CV(htc=hA, area=1)
connect(in, D.in)
connect(D.out, out)
connect(D.wall, W.a)
connect(W.b, CV.a)
connect(CV.b, amb)
\`\`\``,
  },
  {
    name: `GearboxScheduled`,
    slug: `gearboxscheduled`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component GearboxScheduled with ports in, out, u.`,
    related: [],
    examples: [],
    tags: [`gearboxscheduled`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
GearboxScheduled inst(eta)
\`\`\`

## Ports

\`in\`, \`out\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`eta\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
in.w    = u.sig * out.w
out.tau = -u.sig * eta * in.tau
\`\`\``,
  },
  {
    name: `GradeProfile`,
    slug: `gradeprofile`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component GradeProfile with ports port.`,
    related: [],
    examples: [],
    tags: [`gradeprofile`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
GradeProfile inst(m, g, map$, s0)
\`\`\`

## Ports

\`port\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`m\` | Number |
| \`g\` | Number |
| \`map$\` | String |
| \`s0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(s)  = port.vel
init(s) = s0
port.f  = m * g * sin(map$(s))
\`\`\``,
  },
  {
    name: `GradeRoadLoad`,
    slug: `graderoadload`,
    category: `Component (powertrain)`,
    summary: `A vehicle road load including the road-grade contribution.`,
    related: [],
    examples: [],
    tags: [`graderoadload`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `A vehicle road load including the road-grade contribution.

## Domain

A reusable **acausal powertrain-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\`, with vehicle-level speed/force signals. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`shaft\`

## Usage

\`\`\`
GradeRoadLoad inst(Crr, Caero, m, g, grade)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Crr\` | Number | Rolling-resistance coefficient. |
| \`Caero\` | Number | Aerodynamic drag term ½ρCdA [kg/m]. |
| \`m\` | Number | Mass [kg]. |
| \`g\` | Number | Gravitational acceleration [m/s²]. |
| \`grade\` | Number | Road grade (rise/run). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
shaft.tau = Crr + Caero * shaft.w^2 + m * g * sin(grade)
\`\`\``,
  },
  {
    name: `HybridPowerSplit`,
    slug: `hybridpowersplit`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component HybridPowerSplit with ports eng, out, sun, p, n, u1, u2, heat.`,
    related: [],
    examples: [],
    tags: [`hybridpowersplit`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
HybridPowerSplit inst(g, eff1$, eff2$, epsP)
\`\`\`

## Ports

\`eng\`, \`out\`, \`sun\`, \`p\`, \`n\`, \`u1\`, \`u2\`, \`heat\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`g\` | Number |
| \`eff1$\` | String |
| \`eff2$\` | String |
| \`epsP\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
Planetary PL(g=g)
MotorMap  MG1(eff$=eff1$, epsP=epsP)
MotorMap  MG2(eff$=eff2$, epsP=epsP)
connect(eng, PL.carrier)
connect(PL.sun, MG1.shaft, sun)
connect(PL.ring, MG2.shaft, out)
connect(MG1.p, MG2.p, p)
connect(MG1.n, MG2.n, n)
connect(MG1.u, u1)
connect(MG2.u, u2)
connect(MG1.heat, MG2.heat, heat)
\`\`\``,
  },
  {
    name: `MeanValueEngine`,
    slug: `meanvalueengine`,
    category: `Component (powertrain)`,
    summary: `A mean-value engine model (cycle-averaged torque and flows).`,
    related: [],
    examples: [],
    tags: [`meanvalueengine`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `A mean-value engine model (cycle-averaged torque and flows).

## Domain

A reusable **acausal powertrain-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\`, with vehicle-level speed/force signals. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`shaft\`

## Usage

\`\`\`
MeanValueEngine inst(throttle, Tpeak, w_peak, FMEP_a, FMEP_b)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`throttle\` | Number | Throttle (0–1). |
| \`Tpeak\` | Number | Peak temperature [K]. |
| \`w_peak\` | Number | Peak frequency [rad/s]. |
| \`FMEP_a\` | Number | Friction-MEP constant [Pa]. |
| \`FMEP_b\` | Number | Friction-MEP slope coefficient. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
T_wot     = Tpeak * (1 - ((shaft.w - w_peak) / w_peak)^2)
T_ind     = throttle * T_wot
T_fric    = FMEP_a + FMEP_b * shaft.w
shaft.tau = -(T_ind - T_fric)
\`\`\``,
  },
  {
    name: `QuarterCar`,
    slug: `quartercar`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component QuarterCar with ports road.`,
    related: [],
    examples: [],
    tags: [`quartercar`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
QuarterCar inst(ms, mu, ks, cs, kt)
\`\`\`

## Ports

\`road\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`ms\` | Number |
| \`mu\` | Number |
| \`ks\` | Number |
| \`cs\` | Number |
| \`kt\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
TransMass   MS(m=ms, v0=0)
TransMass   MU(m=mu, v0=0)
TransSpring KS(k=ks, x0=0)
TransDamper CS(c=cs)
TransSpring KT(k=kt, x0=0)
connect(MS.port, KS.a, CS.a)
connect(KS.b, CS.b, MU.port, KT.a)
connect(KT.b, road)
\`\`\``,
  },
  {
    name: `RoadLoad`,
    slug: `roadload`,
    category: `Component (powertrain)`,
    summary: `A vehicle road load (aerodynamic drag + rolling resistance).`,
    related: [],
    examples: [],
    tags: [`roadload`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `A vehicle road load (aerodynamic drag + rolling resistance).

## Domain

A reusable **acausal powertrain-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\`, with vehicle-level speed/force signals. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`shaft\`

## Usage

\`\`\`
RoadLoad inst(Crr, Caero)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`Crr\` | Number | Rolling-resistance coefficient. |
| \`Caero\` | Number | Aerodynamic drag term ½ρCdA [kg/m]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
shaft.tau = Crr + Caero * shaft.w^2
\`\`\``,
  },
  {
    name: `TireLongitudinal`,
    slug: `tirelongitudinal`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component TireLongitudinal with ports wheel, veh.`,
    related: [],
    examples: [],
    tags: [`tirelongitudinal`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TireLongitudinal inst(r, Fz, B, C, D, epsv)
\`\`\`

## Ports

\`wheel\`, \`veh\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`r\` | Number |
| \`Fz\` | Number |
| \`B\` | Number |
| \`C\` | Number |
| \`D\` | Number |
| \`epsv\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
v_w       = r * wheel.w
slip      = (v_w - veh.vel) / (abs(veh.vel) + epsv)
Fx        = Fz * D * sin(C * arctan(B * slip))
veh.f     = -Fx
wheel.tau = r * Fx
\`\`\``,
  },
  {
    name: `TirePacejka`,
    slug: `tirepacejka`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component TirePacejka with ports wheel, veh.`,
    related: [],
    examples: [],
    tags: [`tirepacejka`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TirePacejka inst(r, Fz, B, C, D, E, epsv)
\`\`\`

## Ports

\`wheel\`, \`veh\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`r\` | Number |
| \`Fz\` | Number |
| \`B\` | Number |
| \`C\` | Number |
| \`D\` | Number |
| \`E\` | Number |
| \`epsv\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
v_w       = r * wheel.w
slip      = (v_w - veh.vel) / (abs(veh.vel) + epsv)
Bs        = B * slip
Fx        = Fz * D * sin(C * arctan(Bs - E * (Bs - arctan(Bs))))
veh.f     = -Fx
wheel.tau = r * Fx
\`\`\``,
  },
  {
    name: `TorqueConverter`,
    slug: `torqueconverter`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component TorqueConverter with ports pump, turb.`,
    related: [],
    examples: [],
    tags: [`torqueconverter`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TorqueConverter inst(Kmap$, TRmap$)
\`\`\`

## Ports

\`pump\`, \`turb\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Kmap$\` | String |
| \`TRmap$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
SR       = turb.w / pump.w
tau_p    = (pump.w / Kmap$(SR))^2
pump.tau = tau_p
turb.tau = -TRmap$(SR) * tau_p
\`\`\``,
  },
  {
    name: `Transmission`,
    slug: `transmission`,
    category: `Component (powertrain)`,
    summary: `A gearbox/transmission imposing a ratio between engine and wheels.`,
    related: [],
    examples: [],
    tags: [`transmission`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `A gearbox/transmission imposing a ratio between engine and wheels.

## Domain

A reusable **acausal powertrain-domain** component — its rotational ports carry angular velocity \`ω\` and torque \`τ\`, with vehicle-level speed/force signals. Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
Transmission inst(ratio, eta)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`ratio\` | Number | Gear / split ratio. |
| \`eta\` | Number | Efficiency (0–1). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
in.w    = ratio * out.w
out.tau = -ratio * eta * in.tau
\`\`\``,
  },
  {
    name: `VehicleBody`,
    slug: `vehiclebody`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component VehicleBody with ports port.`,
    related: [],
    examples: [],
    tags: [`vehiclebody`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
VehicleBody inst(m, Cd, Af, rhoA, Crr, grade, v0)
\`\`\`

## Ports

\`port\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`m\` | Number |
| \`Cd\` | Number |
| \`Af\` | Number |
| \`rhoA\` | Number |
| \`Crr\` | Number |
| \`grade\` | Number |
| \`v0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
F_res = 0.5 * rhoA * Cd * Af * port.vel * abs(port.vel) + m * 9.80665 * (Crr * tanh(port.vel / 0.1) + sin(grade))
der(port.vel)  = (port.f - F_res) / m
init(port.vel) = v0
\`\`\``,
  },
  {
    name: `WindRotor`,
    slug: `windrotor`,
    category: `Component (powertrain)`,
    summary: `Acausal powertrain-domain component WindRotor with ports shaft, wind, pitch.`,
    related: [],
    examples: [],
    tags: [`windrotor`, `component`, `powertrain`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **powertrain-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
WindRotor inst(rho, R, cp$, epsv, epsw)
\`\`\`

## Ports

\`shaft\`, \`wind\`, \`pitch\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`rho\` | Number |
| \`R\` | Number |
| \`cp$\` | String |
| \`epsv\` | Number |
| \`epsw\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
lam       = shaft.w * R / (wind.sig + epsv)
Cpw       = cp$(lam, pitch.sig)
Pw        = 0.5 * rho * pi# * R^2 * wind.sig^3 * Cpw
shaft.tau = -Pw / (shaft.w + epsw)
\`\`\``,
  },
  {
    name: `SigAbs`,
    slug: `sigabs`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigAbs with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigabs`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigAbs inst(param = value, ...)
\`\`\`

## Ports

\`in\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = abs(in.sig)
\`\`\``,
  },
  {
    name: `SigBias`,
    slug: `sigbias`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigBias with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigbias`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigBias inst(b)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`b\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = in.sig + b
\`\`\``,
  },
  {
    name: `SigConstant`,
    slug: `sigconstant`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigConstant with ports out.`,
    related: [],
    examples: [],
    tags: [`sigconstant`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigConstant inst(k)
\`\`\`

## Ports

\`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`k\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = k
\`\`\``,
  },
  {
    name: `SigDeadband`,
    slug: `sigdeadband`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigDeadband with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigdeadband`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigDeadband inst(w, eps)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`w\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
up      = in.sig - w
dn      = in.sig + w
out.sig = 0.5 * (up + sqrt(up^2 + eps^2)) + 0.5 * (dn - sqrt(dn^2 + eps^2))
\`\`\``,
  },
  {
    name: `SigDerivative`,
    slug: `sigderivative`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigDerivative with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigderivative`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigDerivative inst(tau, y0)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`tau\` | Number |
| \`y0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(y)  = (in.sig - y) / tau
init(y) = y0
out.sig = (in.sig - y) / tau
\`\`\``,
  },
  {
    name: `SigDiff`,
    slug: `sigdiff`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigDiff with ports in1, in2, out.`,
    related: [],
    examples: [],
    tags: [`sigdiff`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigDiff inst(param = value, ...)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = in1.sig - in2.sig
\`\`\``,
  },
  {
    name: `SigDivide`,
    slug: `sigdivide`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigDivide with ports in1, in2, out.`,
    related: [],
    examples: [],
    tags: [`sigdivide`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigDivide inst(param = value, ...)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = in1.sig / in2.sig
\`\`\``,
  },
  {
    name: `SigFirstOrder`,
    slug: `sigfirstorder`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigFirstOrder with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigfirstorder`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigFirstOrder inst(tau, y0)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`tau\` | Number |
| \`y0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(y)  = (in.sig - y) / tau
init(y) = y0
out.sig = y
\`\`\``,
  },
  {
    name: `SigGain`,
    slug: `siggain`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigGain with ports in, out.`,
    related: [],
    examples: [],
    tags: [`siggain`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigGain inst(k)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`k\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = k * in.sig
\`\`\``,
  },
  {
    name: `SigIntegrator`,
    slug: `sigintegrator`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigIntegrator with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigintegrator`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigIntegrator inst(y0)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`y0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(y)  = in.sig
init(y) = y0
out.sig = y
\`\`\``,
  },
  {
    name: `SigLeadLag`,
    slug: `sigleadlag`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigLeadLag with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigleadlag`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigLeadLag inst(T1, T2, y0)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`T1\` | Number |
| \`T2\` | Number |
| \`y0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(x)  = (in.sig - x) / T2
init(x) = y0
out.sig = x + (T1 / T2) * (in.sig - x)
\`\`\``,
  },
  {
    name: `SigMap`,
    slug: `sigmap`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigMap with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigmap`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigMap inst(map$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`map$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = map$(in.sig)
\`\`\``,
  },
  {
    name: `SigMap2`,
    slug: `sigmap2`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigMap2 with ports in1, in2, out.`,
    related: [],
    examples: [],
    tags: [`sigmap2`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigMap2 inst(map$)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`map$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = map$(in1.sig, in2.sig)
\`\`\``,
  },
  {
    name: `SigMax`,
    slug: `sigmax`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigMax with ports in1, in2, out.`,
    related: [],
    examples: [],
    tags: [`sigmax`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigMax inst(param = value, ...)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = max(in1.sig, in2.sig)
\`\`\``,
  },
  {
    name: `SigMin`,
    slug: `sigmin`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigMin with ports in1, in2, out.`,
    related: [],
    examples: [],
    tags: [`sigmin`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigMin inst(param = value, ...)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = min(in1.sig, in2.sig)
\`\`\``,
  },
  {
    name: `SigPID`,
    slug: `sigpid`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigPID with ports sp, pv, out.`,
    related: [],
    examples: [],
    tags: [`sigpid`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigPID inst(Kp, Ki, Kd, tau, i0, d0, model$)
\`\`\`

## Ports

\`sp\`, \`pv\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Kp\` | Number |
| \`Ki\` | Number |
| \`Kd\` | Number |
| \`tau\` | Number |
| \`i0\` | Number |
| \`d0\` | Number |
| \`model$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
e        = sp.sig - pv.sig
der(df)  = (e - df) / tau
init(df) = d0
dterm    = (e - df) / tau
init(ie) = i0
u_raw    = Kp * e + Ki * ie + Kd * dterm
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`basic\`

\`\`\`
der(ie) = e
out.sig = u_raw
\`\`\`

### \`clamped\` — requires \`umin\`, \`umax\`, \`Taw\`

\`\`\`
out.sig = min(max(u_raw, umin), umax)
der(ie) = e + (out.sig - u_raw) / Taw
\`\`\``,
  },
  {
    name: `SigProduct`,
    slug: `sigproduct`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigProduct with ports in1, in2, out.`,
    related: [],
    examples: [],
    tags: [`sigproduct`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigProduct inst(param = value, ...)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = in1.sig * in2.sig
\`\`\``,
  },
  {
    name: `SigPulse`,
    slug: `sigpulse`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigPulse with ports out.`,
    related: [],
    examples: [],
    tags: [`sigpulse`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigPulse inst(t0, width, high, low, eps)
\`\`\`

## Ports

\`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`t0\` | Number |
| \`width\` | Number |
| \`high\` | Number |
| \`low\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = low + (high - low) * 0.5 * (tanh((time - t0) / eps) - tanh((time - t0 - width) / eps))
\`\`\``,
  },
  {
    name: `SigRamp`,
    slug: `sigramp`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigRamp with ports out.`,
    related: [],
    examples: [],
    tags: [`sigramp`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigRamp inst(t0, slope, eps)
\`\`\`

## Ports

\`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`t0\` | Number |
| \`slope\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
dt      = time - t0
out.sig = slope * 0.5 * (dt + sqrt(dt^2 + eps^2))
\`\`\``,
  },
  {
    name: `SigRateLimiter`,
    slug: `sigratelimiter`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigRateLimiter with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigratelimiter`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigRateLimiter inst(rate, tau, y0)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`rate\` | Number |
| \`tau\` | Number |
| \`y0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(y)  = rate * tanh((in.sig - y) / (rate * tau))
init(y) = y0
out.sig = y
\`\`\``,
  },
  {
    name: `SigRelay`,
    slug: `sigrelay`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigRelay with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigrelay`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigRelay inst(thresh, low, high, eps)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`thresh\` | Number |
| \`low\` | Number |
| \`high\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = low + (high - low) * 0.5 * (1 + tanh((in.sig - thresh) / eps))
\`\`\``,
  },
  {
    name: `SigSaturation`,
    slug: `sigsaturation`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigSaturation with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigsaturation`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigSaturation inst(lo, hi)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`lo\` | Number |
| \`hi\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = min(max(in.sig, lo), hi)
\`\`\``,
  },
  {
    name: `SigSecondOrder`,
    slug: `sigsecondorder`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigSecondOrder with ports in, out.`,
    related: [],
    examples: [],
    tags: [`sigsecondorder`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigSecondOrder inst(wn, zeta, y0, v0)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`wn\` | Number |
| \`zeta\` | Number |
| \`y0\` | Number |
| \`v0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(y)  = v
init(y) = y0
der(v)  = wn^2 * (in.sig - y) - 2 * zeta * wn * v
init(v) = v0
out.sig = y
\`\`\``,
  },
  {
    name: `SigSine`,
    slug: `sigsine`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigSine with ports out.`,
    related: [],
    examples: [],
    tags: [`sigsine`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigSine inst(amp, freq, phase, bias)
\`\`\`

## Ports

\`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`amp\` | Number |
| \`freq\` | Number |
| \`phase\` | Number |
| \`bias\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = bias + amp * sin(2 * pi# * freq * time + phase)
\`\`\``,
  },
  {
    name: `SigSpeedProbe`,
    slug: `sigspeedprobe`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigSpeedProbe with ports shaft, out.`,
    related: [],
    examples: [],
    tags: [`sigspeedprobe`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigSpeedProbe inst(param = value, ...)
\`\`\`

## Ports

\`shaft\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
shaft.tau = 0
out.sig   = shaft.w
\`\`\``,
  },
  {
    name: `SigStep`,
    slug: `sigstep`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigStep with ports out.`,
    related: [],
    examples: [],
    tags: [`sigstep`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigStep inst(t0, before, after, eps)
\`\`\`

## Ports

\`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`t0\` | Number |
| \`before\` | Number |
| \`after\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = before + (after - before) * 0.5 * (1 + tanh((time - t0) / eps))
\`\`\``,
  },
  {
    name: `SigSum`,
    slug: `sigsum`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigSum with ports in1, in2, out.`,
    related: [],
    examples: [],
    tags: [`sigsum`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigSum inst(param = value, ...)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = in1.sig + in2.sig
\`\`\``,
  },
  {
    name: `SigSwitch`,
    slug: `sigswitch`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigSwitch with ports in1, in2, ctrl, out.`,
    related: [],
    examples: [],
    tags: [`sigswitch`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigSwitch inst(thresh, eps)
\`\`\`

## Ports

\`in1\`, \`in2\`, \`ctrl\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`thresh\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
wgt     = 0.5 * (1 + tanh((ctrl.sig - thresh) / eps))
out.sig = wgt * in1.sig + (1 - wgt) * in2.sig
\`\`\``,
  },
  {
    name: `SigTable`,
    slug: `sigtable`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigTable with ports out.`,
    related: [],
    examples: [],
    tags: [`sigtable`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigTable inst(map$)
\`\`\`

## Ports

\`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`map$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = map$(time)
\`\`\``,
  },
  {
    name: `SigThermalProbe`,
    slug: `sigthermalprobe`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigThermalProbe with ports port, out.`,
    related: [],
    examples: [],
    tags: [`sigthermalprobe`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigThermalProbe inst(param = value, ...)
\`\`\`

## Ports

\`port\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
port.Qdot = 0
out.sig   = port.T
\`\`\``,
  },
  {
    name: `SigTime`,
    slug: `sigtime`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigTime with ports out.`,
    related: [],
    examples: [],
    tags: [`sigtime`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigTime inst(param = value, ...)
\`\`\`

## Ports

\`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.sig = time
\`\`\``,
  },
  {
    name: `SigVelProbe`,
    slug: `sigvelprobe`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SigVelProbe with ports port, out.`,
    related: [],
    examples: [],
    tags: [`sigvelprobe`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SigVelProbe inst(param = value, ...)
\`\`\`

## Ports

\`port\`, \`out\`

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
port.f  = 0
out.sig = port.vel
\`\`\``,
  },
  {
    name: `SupervisoryECMS`,
    slug: `supervisoryecms`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component SupervisoryECMS with ports soc, dem, eng, mot.`,
    related: [],
    examples: [],
    tags: [`supervisoryecms`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SupervisoryECMS inst(soc_ref, eps)
\`\`\`

## Ports

\`soc\`, \`dem\`, \`eng\`, \`mot\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`soc_ref\` | Number |
| \`eps\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
s       = 0.5 * (1 + tanh((soc.sig - soc_ref) / eps))
mot.sig = s * dem.sig
eng.sig = (1 - s) * dem.sig
\`\`\``,
  },
  {
    name: `ZoneCO2`,
    slug: `zoneco2`,
    category: `Component (signal)`,
    summary: `Acausal signal-domain component ZoneCO2 with ports vent, occ, out.`,
    related: [],
    examples: [],
    tags: [`zoneco2`, `component`, `signal`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **signal-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ZoneCO2 inst(Vz, c_amb, gen_occ, c0)
\`\`\`

## Ports

\`vent\`, \`occ\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`Vz\` | Number |
| \`c_amb\` | Number |
| \`gen_occ\` | Number |
| \`c0\` | Number |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
der(c)  = (vent.sig * (c_amb - c) + occ.sig * gen_occ) / Vz
init(c) = c0
out.sig = c
\`\`\``,
  },
  {
    name: `BlendMixer`,
    slug: `blendmixer`,
    category: `Component (twophase)`,
    summary: `A gas-blend (mixture) mixing junction carrying the species rider.`,
    related: [],
    examples: [],
    tags: [`blendmixer`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A gas-blend (mixture) mixing junction carrying the species rider.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in1\`, \`in2\`, \`out\`

## Usage

\`\`\`
BlendMixer inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P    = in1.P
out.mdot = in1.mdot + in2.mdot
out.mdot * out.h = in1.mdot * in1.h + in2.mdot * in2.h
out.mdot * out.z = in1.mdot * in1.z + in2.mdot * in2.z
\`\`\``,
  },
  {
    name: `BlendSensor`,
    slug: `blendsensor`,
    category: `Component (twophase)`,
    summary: `A sensor reading the state of a gas-blend stream.`,
    related: [],
    examples: [],
    tags: [`blendsensor`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A sensor reading the state of a gas-blend stream.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
BlendSensor inst(fluid$, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.h    = in.h
out.z    = in.z
hf       = Enthalpy(fluid$, P=in.P, x=0)
hg       = Enthalpy(fluid$, P=in.P, x=1)
x        = (in.h - hf) / (hg - hf)
bubble   = Temperature(fluid$, P=in.P, x=0)
dew      = Temperature(fluid$, P=in.P, x=1)
glide    = dew - bubble
\`\`\``,
  },
  {
    name: `BlendSink`,
    slug: `blendsink`,
    category: `Component (twophase)`,
    summary: `A boundary absorbing a gas-blend stream.`,
    related: [],
    examples: [],
    tags: [`blendsink`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A boundary absorbing a gas-blend stream.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`

## Usage

\`\`\`
BlendSink inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
mdot  = in.mdot
P     = in.P
h     = in.h
z     = in.z
\`\`\``,
  },
  {
    name: `BlendSource`,
    slug: `blendsource`,
    category: `Component (twophase)`,
    summary: `A boundary supplying a gas-blend stream of set composition.`,
    related: [],
    examples: [],
    tags: [`blendsource`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A boundary supplying a gas-blend stream of set composition.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
BlendSource inst(fluid$, mdot, P, x, z, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`mdot\` | Number | Mass flow rate [kg/s]. |
| \`P\` | Number | Pressure [Pa]. |
| \`x\` | Number | Vapor quality / fraction (0–1). |
| \`z\` | Number | Elevation [m]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = mdot
out.P    = P
out.h    = Enthalpy(fluid$, P=P, x=x)
out.z    = z
bubble   = Temperature(fluid$, P=P, x=0)
dew      = Temperature(fluid$, P=P, x=1)
glide    = dew - bubble
\`\`\``,
  },
  {
    name: `BoilingVessel`,
    slug: `boilingvessel`,
    category: `Component (twophase)`,
    summary: `A rigid vessel boiling a two-phase fluid (rigid two-phase boil-off).`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`boilingvessel`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A rigid vessel boiling a two-phase fluid (rigid two-phase boil-off).

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`vent\`, \`wall\`

## Usage

\`\`\`
BoilingVessel inst(fluid$, V, m0, T0, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`V\` | Number | Volume [m³]. |
| \`m0\` | Number | Initial mass [kg]. |
| \`T0\` | Number | Reference/initial temperature [K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
der(mass)  = -vent.mdot
init(mass) = m0
der(Utot)  = wall.Qdot - vent.mdot * vent.h
init(Utot) = m0 * IntEnergy(fluid$, T=T0, x=0)
rho_cv = mass / V
u_cv   = Utot / mass
vent.P = Pressure(fluid$, d=rho_cv, u=u_cv)      { (rho,u) flash -> pressure }
T_cv   = Temperature(fluid$, d=rho_cv, u=u_cv)
x_cv   = Quality(fluid$, d=rho_cv, u=u_cv)        { vapour mass fraction }
vent.h = Enthalpy(fluid$, P=vent.P, x=1)          { vented stream is sat. vapour }
wall.T = T_cv
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pressure-cooker]`,
  },
  {
    name: `CapillaryTube`,
    slug: `capillarytube`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component CapillaryTube with ports in, out.`,
    related: [],
    examples: [],
    tags: [`capillarytube`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
CapillaryTube inst(fluid$, C, n, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`C\` | Number |
| \`n\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
T_in     = Temperature(fluid$, P=in.P, h=in.h)
Pf       = P_sat(fluid$, T=T_in)
dP_eff   = in.P - max(out.P, Pf)
in.mdot  = C * dP_eff^n
\`\`\``,
  },
  {
    name: `EjectorMomentum`,
    slug: `ejectormomentum`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component EjectorMomentum with ports mot_in, suc_in, out.`,
    related: [],
    examples: [],
    tags: [`ejectormomentum`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
EjectorMomentum inst(fluid$, eta_n, eta_m, domain$)
\`\`\`

## Ports

\`mot_in\`, \`suc_in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`eta_n\` | Number |
| \`eta_m\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
s_m     = Entropy(fluid$, P=mot_in.P, h=mot_in.h)
h_mi    = Enthalpy(fluid$, P=suc_in.P, s=s_m)
v_m     = sqrt(2 * eta_n * (mot_in.h - h_mi))
out.mdot = mot_in.mdot + suc_in.mdot
v_mix   = eta_m * mot_in.mdot * v_m / out.mdot
out.mdot * out.h = mot_in.mdot * mot_in.h + suc_in.mdot * suc_in.h
h_mix   = out.h - v_mix^2 / 2
rho_mix = Density(fluid$, P=suc_in.P, h=h_mix)
out.P   = suc_in.P + rho_mix * v_mix^2 / 2
\`\`\``,
  },
  {
    name: `FewCellCondenser`,
    slug: `fewcellcondenser`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component FewCellCondenser with ports in, out, w1, w2, w3.`,
    related: [],
    examples: [],
    tags: [`fewcellcondenser`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
FewCellCondenser inst(fluid$, V, Cc, UA, Kv, P0, h0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`w1\`, \`w2\`, \`w3\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`V\` | Number |
| \`Cc\` | Number |
| \`UA\` | Number |
| \`Kv\` | Number |
| \`P0\` | Number |
| \`h0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
in.mdot  = Kv * (in.P - P1)
m2       = Kv * (P1 - P2)
m3       = Kv * (P2 - P3)
out.mdot = Kv * (P3 - out.P)
der(P1)  = (in.mdot - m2) / Cc
init(P1) = P0
der(P2)  = (m2 - m3) / Cc
init(P2) = P0
der(P3)  = (m3 - out.mdot) / Cc
init(P3) = P0
T1 = Temperature(fluid$, P=P1, h=h1)
T2 = Temperature(fluid$, P=P2, h=h2)
T3 = Temperature(fluid$, P=P3, h=h3)
Q1 = UA * (w1.T - T1)
Q2 = UA * (w2.T - T2)
Q3 = UA * (w3.T - T3)
w1.Qdot = Q1
w2.Qdot = Q2
w3.Qdot = Q3
rho1 = Density(fluid$, P=P1, h=h1)
rho2 = Density(fluid$, P=P2, h=h2)
rho3 = Density(fluid$, P=P3, h=h3)
der(h1)  = (in.mdot * (in.h - h1) + Q1) / (rho1 * V)
init(h1) = h0
der(h2)  = (m2 * (h1 - h2) + Q2) / (rho2 * V)
init(h2) = h0
der(h3)  = (m3 * (h2 - h3) + Q3) / (rho3 * V)
init(h3) = h0
out.h = h3
\`\`\``,
  },
  {
    name: `FewCellEvaporator`,
    slug: `fewcellevaporator`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component FewCellEvaporator with ports in, out, w1, w2, w3.`,
    related: [],
    examples: [],
    tags: [`fewcellevaporator`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
FewCellEvaporator inst(fluid$, V, Cc, UA, Kv, P0, h0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`w1\`, \`w2\`, \`w3\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`V\` | Number |
| \`Cc\` | Number |
| \`UA\` | Number |
| \`Kv\` | Number |
| \`P0\` | Number |
| \`h0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
in.mdot  = Kv * (in.P - P1)
m2       = Kv * (P1 - P2)
m3       = Kv * (P2 - P3)
out.mdot = Kv * (P3 - out.P)
der(P1)  = (in.mdot - m2) / Cc
init(P1) = P0
der(P2)  = (m2 - m3) / Cc
init(P2) = P0
der(P3)  = (m3 - out.mdot) / Cc
init(P3) = P0
T1 = Temperature(fluid$, P=P1, h=h1)
T2 = Temperature(fluid$, P=P2, h=h2)
T3 = Temperature(fluid$, P=P3, h=h3)
Q1 = UA * (w1.T - T1)
Q2 = UA * (w2.T - T2)
Q3 = UA * (w3.T - T3)
w1.Qdot = Q1
w2.Qdot = Q2
w3.Qdot = Q3
rho1 = Density(fluid$, P=P1, h=h1)
rho2 = Density(fluid$, P=P2, h=h2)
rho3 = Density(fluid$, P=P3, h=h3)
der(h1)  = (in.mdot * (in.h - h1) + Q1) / (rho1 * V)
init(h1) = h0
der(h2)  = (m2 * (h1 - h2) + Q2) / (rho2 * V)
init(h2) = h0
der(h3)  = (m3 * (h2 - h3) + Q3) / (rho3 * V)
init(h3) = h0
out.h = h3
\`\`\``,
  },
  {
    name: `FlashTank`,
    slug: `flashtank`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component FlashTank with ports in, liq, vap.`,
    related: [],
    examples: [],
    tags: [`flashtank`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
FlashTank inst(fluid$, domain$)
\`\`\`

## Ports

\`in\`, \`liq\`, \`vap\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
liq.P = in.P
vap.P = in.P
liq.h = Enthalpy(fluid$, P=in.P, x=0)
vap.h = Enthalpy(fluid$, P=in.P, x=1)
in.mdot = liq.mdot + vap.mdot
in.mdot * in.h = liq.mdot * liq.h + vap.mdot * vap.h
\`\`\``,
  },
  {
    name: `GasCooler`,
    slug: `gascooler`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component GasCooler with ports in, out, wall.`,
    related: [],
    examples: [],
    tags: [`gascooler`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
GasCooler inst(fluid$, UA, dP, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`wall\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`UA\` | Number |
| \`dP\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot  = in.mdot
out.P     = in.P - dP
T_in      = Temperature(fluid$, P=in.P, h=in.h)
cp_g      = Cp(fluid$, P=in.P, h=in.h)
epsg      = 1 - exp(-UA / (in.mdot * cp_g))
Q         = epsg * in.mdot * cp_g * (T_in - wall.T)
out.h     = in.h - Q / in.mdot
wall.Qdot = -Q
\`\`\``,
  },
  {
    name: `MovingBoundaryCondenser`,
    slug: `movingboundarycondenser`,
    category: `Component (twophase)`,
    summary: `A moving-boundary condenser tracking the two-phase/subcooled zone lengths.`,
    related: [],
    examples: [],
    tags: [`movingboundarycondenser`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A moving-boundary condenser tracking the two-phase/subcooled zone lengths.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`wall\`

## Usage

\`\`\`
MovingBoundaryCondenser inst(fluid$, U_cond, U_sc, D, L, eps_zone, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`U_cond\` | Number | Condenser-zone overall coefficient [W/m²·K]. |
| \`U_sc\` | Number | Subcool-zone overall coefficient [W/m²·K]. |
| \`D\` | Number | Diameter [m]. |
| \`L\` | Number | Length [m]. |
| \`eps_zone\` | Number | Zone-collapse smoothing width. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
Tsat      = T_sat(fluid$, P=in.P)
hf        = Enthalpy(fluid$, P=in.P, x=0)
L_need    = in.mdot * (in.h - hf) / (U_cond * pi# * D * (Tsat - wall.T))
L_cond    = 0.5 * (L_need + L - sqrt((L_need - L)^2 + eps_zone^2))
Q_cond    = U_cond * pi# * D * L_cond * (Tsat - wall.T)
L_sc      = L - L_cond
r_sc      = zone_ramp(L_sc, eps_zone)
T_out     = Temperature(fluid$, P=out.P, h=out.h)
Q_sc      = U_sc * pi# * D * L_sc * (0.5 * (Tsat + T_out) - wall.T) * r_sc
out.h     = in.h - (Q_cond + Q_sc) / in.mdot
Q         = Q_cond + Q_sc
wall.Qdot = -Q
SC        = Tsat - T_out
\`\`\``,
  },
  {
    name: `MovingBoundaryEvaporator`,
    slug: `movingboundaryevaporator`,
    category: `Component (twophase)`,
    summary: `A moving-boundary evaporator tracking the two-phase/superheat zone lengths.`,
    related: [],
    examples: [],
    tags: [`movingboundaryevaporator`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A moving-boundary evaporator tracking the two-phase/superheat zone lengths.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`wall\`

## Usage

\`\`\`
MovingBoundaryEvaporator inst(fluid$, U_tp, U_sh, D, L, eps_zone, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`U_tp\` | Number | Two-phase-zone overall coefficient [W/m²·K]. |
| \`U_sh\` | Number | Superheat-zone overall coefficient [W/m²·K]. |
| \`D\` | Number | Diameter [m]. |
| \`L\` | Number | Length [m]. |
| \`eps_zone\` | Number | Zone-collapse smoothing width. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot  = in.mdot
out.P     = in.P
Tsat      = T_sat(fluid$, P=in.P)
hg        = Enthalpy(fluid$, P=in.P, x=1)
L_need    = in.mdot * (hg - in.h) / (U_tp * pi# * D * (wall.T - Tsat))
L_tp      = 0.5 * (L_need + L - sqrt((L_need - L)^2 + eps_zone^2))
Q_tp      = U_tp * pi# * D * L_tp * (wall.T - Tsat)
L_sh      = L - L_tp
r_sh      = zone_ramp(L_sh, eps_zone)
T_out     = Temperature(fluid$, P=out.P, h=out.h)
Q_sh      = U_sh * pi# * D * L_sh * (wall.T - 0.5 * (Tsat + T_out)) * r_sh
out.h     = in.h + (Q_tp + Q_sh) / in.mdot
Q         = Q_tp + Q_sh
wall.Qdot = Q
SH        = T_out - Tsat
\`\`\``,
  },
  {
    name: `OilSeparator`,
    slug: `oilseparator`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component OilSeparator with ports in, out, bleed.`,
    related: [],
    examples: [],
    tags: [`oilseparator`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
OilSeparator inst(fluid$, f, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`bleed\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`f\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
bleed.mdot = f * in.mdot
out.mdot   = (1 - f) * in.mdot
out.P      = in.P
bleed.P    = in.P
bleed.h    = Enthalpy(fluid$, P=in.P, x=0)
out.mdot * out.h = in.mdot * in.h - bleed.mdot * bleed.h
\`\`\``,
  },
  {
    name: `ProportionalReliefValve`,
    slug: `proportionalreliefvalve`,
    category: `Component (twophase)`,
    summary: `A pressure-relief valve whose opening rises proportionally above the set pressure.`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`proportionalreliefvalve`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A pressure-relief valve whose opening rises proportionally above the set pressure.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
ProportionalReliefValve inst(fluid$, Pcrack, grad, eps, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`Pcrack\` | Number | Cracking (relief) pressure [Pa]. |
| \`grad\` | Number | Road grade (rise/run). |
| \`eps\` | Number | Effectiveness / roughness. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
dpv      = in.P - Pcrack
in.mdot  = grad * 0.5 * (dpv + sqrt(dpv * dpv + eps * eps))
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pressure-cooker]`,
  },
  {
    name: `ReversingValve`,
    slug: `reversingvalve`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component ReversingValve with ports d, s, i, o.`,
    related: [],
    examples: [],
    tags: [`reversingvalve`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
ReversingValve inst(mode, domain$)
\`\`\`

## Ports

\`d\`, \`s\`, \`i\`, \`o\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`mode\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
o.P    = (1 - mode) * d.P + mode * s.P
i.P    = (1 - mode) * s.P + mode * d.P
o.mdot = (1 - mode) * d.mdot + mode * s.mdot
i.mdot = (1 - mode) * s.mdot + mode * d.mdot
(1 - mode) * (o.h - d.h) + mode * (s.h - o.h) = 0
(1 - mode) * (s.h - i.h) + mode * (i.h - d.h) = 0
\`\`\``,
  },
  {
    name: `SteamReliefValve`,
    slug: `steamreliefvalve`,
    category: `Component (twophase)`,
    summary: `A steam relief valve venting above the set pressure.`,
    related: [],
    examples: [],
    tags: [`steamreliefvalve`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A steam relief valve venting above the set pressure.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
SteamReliefValve inst(fluid$, A, Pset, Cd, kgas, Rgas, eps, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`A\` | Number | Area [m²]. |
| \`Pset\` | Number | Set pressure [Pa]. |
| \`Cd\` | Number | Discharge coefficient. |
| \`kgas\` | Number | Gas specific-heat ratio. |
| \`Rgas\` | Number | Specific gas constant [J/kg·K]. |
| \`eps\` | Number | Effectiveness / roughness. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
opening  = 0.5 * (1 + tanh((in.P - Pset) / eps))
T0v      = Temperature(fluid$, P=in.P, x=1)
PRc      = (2 / (kgas + 1)) ^ (kgas / (kgas - 1))
mdot_ch  = Cd * A * in.P * sqrt(kgas / (Rgas * T0v)) * (2 / (kgas + 1)) ^ ((kgas + 1) / (2 * (kgas - 1)))
ratio    = (min(max(out.P / in.P, PRc), 1) - PRc) / (1 - PRc)
efact    = 1 - ratio ^ 2
in.mdot  = opening * mdot_ch * efact
\`\`\``,
  },
  {
    name: `SuctionAccumulator`,
    slug: `suctionaccumulator`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component SuctionAccumulator with ports in, out.`,
    related: [],
    examples: [],
    tags: [`suctionaccumulator`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
SuctionAccumulator inst(fluid$, m0, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`m0\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.P   = in.P
hf      = Enthalpy(fluid$, P=in.P, x=0)
hg      = Enthalpy(fluid$, P=in.P, x=1)
out.h   = hg
der(m)  = in.mdot - out.mdot
init(m) = m0
hf * (in.mdot - out.mdot) = in.mdot * in.h - out.mdot * hg
\`\`\``,
  },
  {
    name: `ThreeZoneHX`,
    slug: `threezonehx`,
    category: `Component (twophase)`,
    summary: `A three-zone (subcooled / two-phase / superheat) heat exchanger.`,
    related: [],
    examples: [],
    tags: [`threezonehx`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A three-zone (subcooled / two-phase / superheat) heat exchanger.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`hot_in\`, \`hot_out\`, \`cold_in\`, \`cold_out\`

## Usage

\`\`\`
ThreeZoneHX inst(UA, hot$, cold$, arr$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`UA\` | Number | Overall conductance UA [W/K]. |
| \`hot$\` | String | Hot-side fluid name (e.g. Water). |
| \`cold$\` | String | Cold-side fluid name (e.g. EG50). |
| \`arr$\` | String | Flow arrangement (passed to hx_effectiveness) — one of \`counterflow\`, \`parallel\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
HeatExchanger Z1(UA=UA/3, hot$=hot$, cold$=cold$, arr$=arr$)
HeatExchanger Z2(UA=UA/3, hot$=hot$, cold$=cold$, arr$=arr$)
HeatExchanger Z3(UA=UA/3, hot$=hot$, cold$=cold$, arr$=arr$)
connect(hot_in, Z1.hot_in)
connect(Z1.hot_out, Z2.hot_in)
connect(Z2.hot_out, Z3.hot_in)
connect(Z3.hot_out, hot_out)
connect(cold_in, Z3.cold_in)
connect(Z3.cold_out, Z2.cold_in)
connect(Z2.cold_out, Z1.cold_in)
connect(Z1.cold_out, cold_out)
\`\`\``,
  },
  {
    name: `TranscriticalBackPressureValve`,
    slug: `transcriticalbackpressurevalve`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component TranscriticalBackPressureValve with ports in, out, u.`,
    related: [],
    examples: [],
    tags: [`transcriticalbackpressurevalve`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TranscriticalBackPressureValve inst(fluid$, CdA_max, domain$)
\`\`\`

## Ports

\`in\`, \`out\`, \`u\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`CdA_max\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
rho      = Density(fluid$, P=in.P, h=in.h)
out.mdot = in.mdot
out.h    = in.h
in.mdot * abs(in.mdot) = (u.sig * CdA_max)^2 * 2 * rho * (in.P - out.P)
\`\`\``,
  },
  {
    name: `TwoPhaseCap`,
    slug: `twophasecap`,
    category: `Component (twophase)`,
    summary: `A two-phase capacitive volume (a pressure-compliance node).`,
    related: [],
    examples: [],
    tags: [`twophasecap`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase capacitive volume (a pressure-compliance node).

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`

## Usage

\`\`\`
TwoPhaseCap inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
in.mdot = 0
\`\`\``,
  },
  {
    name: `TwoPhaseChamber`,
    slug: `twophasechamber`,
    category: `Component (twophase)`,
    summary: `A two-phase control volume.`,
    related: [],
    examples: [],
    tags: [`twophasechamber`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase control volume.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`wall\`

## Usage

\`\`\`
TwoPhaseChamber inst(fluid$, V, C, UA, P0, h0, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`V\` | Number | Volume [m³]. |
| \`C\` | Number | Capacitance [F]. |
| \`UA\` | Number | Overall conductance UA [W/K]. |
| \`P0\` | Number | Reference/initial pressure [Pa]. |
| \`h0\` | Number | Reference enthalpy [J/kg]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
der(in.P)  = (in.mdot - out.mdot) / C
init(in.P) = P0
rho        = Density(fluid$, P=in.P, h=hcv)
Q          = UA * (wall.T - Tcv)
der(hcv)   = (in.mdot * (in.h - hcv) + Q) / (rho * V)
init(hcv)  = h0
out.P     = in.P
out.h     = hcv
Tcv       = Temperature(fluid$, P=in.P, h=hcv)
wall.Qdot = Q
m         = rho * V
\`\`\``,
  },
  {
    name: `TwoPhaseCompressor`,
    slug: `twophasecompressor`,
    category: `Component (twophase)`,
    summary: `A refrigerant compressor with selectable isentropic/volumetric variants.`,
    related: [`Compressor`, `CompressorMap`],
    examples: [`ev-thermal-management`],
    tags: [`compressor`, `compressor-family`, `twophasecompressor`, `ports:in-out`, `flow-closed`, `energy-work`, `steady`, `twophase`, `component`, `acausal`],
    references: [],
    guides: [],
    body: `A refrigerant compressor with selectable isentropic/volumetric variants.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseCompressor inst(fluid$, eta, domain$, model$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`eta\` | Number | Efficiency (0–1). |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |
| \`model$\` | String | Model variant — selects the physics body (see Model Variants). |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
s_in     = Entropy(fluid$, P=in.P, h=in.h)
h_s      = Enthalpy(fluid$, P=out.P, s=s_in)
out.mdot = in.mdot
out.h    = in.h + (h_s - in.h) / eta
W        = in.mdot * (out.h - in.h)
\`\`\`

## Model Variants

Selected via the \`model$\` parameter; each adds its own equations (and \`REQUIRE\`d parameters):

### \`isentropic\`

_No additional equations (uses the shared body)._

### \`volumetric\` — requires \`eta_v\`, \`disp\`, \`rpm\`

\`\`\`
rho_in  = Density(fluid$, P=in.P, h=in.h)
in.mdot = eta_v * disp * (rpm / 60) * rho_in
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `TwoPhaseCondenser`,
    slug: `twophasecondenser`,
    category: `Component (twophase)`,
    summary: `A two-phase condenser rejecting heat from the refrigerant.`,
    related: [],
    examples: [],
    tags: [`twophasecondenser`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase condenser rejecting heat from the refrigerant.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseCondenser inst(fluid$, SC_set, dP, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`SC_set\` | Number | Target subcooling [K]. |
| \`dP\` | Number | Nominal pressure drop [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P - dP
Tsat     = T_sat(fluid$, P=out.P)
out.h    = Enthalpy(fluid$, P=out.P, T=Tsat - SC_set)
Q        = in.mdot * (in.h - out.h)
\`\`\``,
  },
  {
    name: `TwoPhaseCondenserFloat`,
    slug: `twophasecondenserfloat`,
    category: `Component (twophase)`,
    summary: `A two-phase condenser whose pressure floats with the charge/ambient balance.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`twophasecondenserfloat`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase condenser whose pressure floats with the charge/ambient balance.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseCondenserFloat inst(fluid$, UA, T_amb, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`UA\` | Number | Overall conductance UA [W/K]. |
| \`T_amb\` | Number | Ambient temperature [K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
Tcond    = T_sat(fluid$, P=in.P)
out.h    = Enthalpy(fluid$, P=in.P, x=0)
Q        = in.mdot * (in.h - out.h)
Q        = UA * (Tcond - T_amb)
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `TwoPhaseCondenserUA`,
    slug: `twophasecondenserua`,
    category: `Component (twophase)`,
    summary: `A two-phase condenser sized by an overall conductance UA.`,
    related: [],
    examples: [],
    tags: [`twophasecondenserua`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase condenser sized by an overall conductance \`UA\`.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseCondenserUA inst(fluid$, UA, T_amb, V, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`UA\` | Number | Overall conductance UA [W/K]. |
| \`T_amb\` | Number | Ambient temperature [K]. |
| \`V\` | Number | Volume [m³]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
Tcond    = T_sat(fluid$, P=in.P)
Q        = UA * (Tcond - T_amb)
Q        = in.mdot * (in.h - out.h)
rho_in   = Density(fluid$, P=in.P, h=in.h)
rho_out  = Density(fluid$, P=out.P, h=out.h)
m        = V * 0.5 * (rho_in + rho_out)
SC       = Tcond - Temperature(fluid$, P=out.P, h=out.h)
\`\`\``,
  },
  {
    name: `TwoPhaseEjector`,
    slug: `twophaseejector`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component TwoPhaseEjector with ports m, s, out.`,
    related: [],
    examples: [],
    tags: [`twophaseejector`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TwoPhaseEjector inst(PLR, domain$)
\`\`\`

## Ports

\`m\`, \`s\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`PLR\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = m.mdot + s.mdot
out.mdot * out.h = m.mdot * m.h + s.mdot * s.h
out.P = PLR * s.P
\`\`\``,
  },
  {
    name: `TwoPhaseEnthalpySource`,
    slug: `twophaseenthalpysource`,
    category: `Component (twophase)`,
    summary: `A two-phase boundary fixing the stream enthalpy.`,
    related: [],
    examples: [],
    tags: [`twophaseenthalpysource`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase boundary fixing the stream enthalpy.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
TwoPhaseEnthalpySource inst(mdot, h, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`mdot\` | Number | Mass flow rate [kg/s]. |
| \`h\` | Number | Heat-transfer coefficient [W/m²·K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = mdot
out.h    = h
\`\`\``,
  },
  {
    name: `TwoPhaseEvaporator`,
    slug: `twophaseevaporator`,
    category: `Component (twophase)`,
    summary: `A two-phase evaporator absorbing heat into the refrigerant.`,
    related: [],
    examples: [],
    tags: [`twophaseevaporator`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase evaporator absorbing heat into the refrigerant.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseEvaporator inst(fluid$, SH_set, dP, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`SH_set\` | Number | Target superheat [K]. |
| \`dP\` | Number | Nominal pressure drop [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P - dP
Tsat     = T_sat(fluid$, P=out.P)
out.h    = Enthalpy(fluid$, P=out.P, T=Tsat + SH_set)
Q        = in.mdot * (out.h - in.h)
\`\`\``,
  },
  {
    name: `TwoPhaseEvaporatorUA`,
    slug: `twophaseevaporatorua`,
    category: `Component (twophase)`,
    summary: `A two-phase evaporator sized by an overall conductance UA.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`twophaseevaporatorua`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase evaporator sized by an overall conductance \`UA\`.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`wall\`

## Usage

\`\`\`
TwoPhaseEvaporatorUA inst(fluid$, UA, dP, SH, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`UA\` | Number | Overall conductance UA [W/K]. |
| \`dP\` | Number | Nominal pressure drop [Pa]. |
| \`SH\` | Number | Superheat [K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P     = in.P - dP
Tevap     = T_sat(fluid$, P=in.P)
Q         = frac * UA * (wall.T - Tevap)
out.h     = Enthalpy(fluid$, P=out.P, T=Tevap + SH)
in.mdot   = Q / (out.h - in.h)
out.mdot  = in.mdot
wall.Qdot = Q
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `TwoPhaseExpansionValve`,
    slug: `twophaseexpansionvalve`,
    category: `Component (twophase)`,
    summary: `A refrigerant expansion valve (isenthalpic throttle).`,
    related: [],
    examples: [],
    tags: [`twophaseexpansionvalve`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A refrigerant expansion valve (isenthalpic throttle).

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseExpansionValve inst(fluid$, Cv, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`Cv\` | Number | Flow coefficient. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
rho_in   = Density(fluid$, P=in.P, h=in.h)
in.mdot * abs(in.mdot) = Cv^2 * 2 * rho_in * (in.P - out.P)
\`\`\``,
  },
  {
    name: `TwoPhaseFlowRes`,
    slug: `twophaseflowres`,
    category: `Component (twophase)`,
    summary: `A two-phase flow resistance relating pressure drop to mass flow.`,
    related: [],
    examples: [],
    tags: [`twophaseflowres`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase flow resistance relating pressure drop to mass flow.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseFlowRes inst(fluid$, L, D, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`L\` | Number | Length [m]. |
| \`D\` | Number | Diameter [m]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
hf       = Enthalpy(fluid$, P=in.P, x=0)
hg       = Enthalpy(fluid$, P=in.P, x=1)
x        = (in.h - hf) / (hg - hf)
rho_l    = Density(fluid$, P=in.P, x=0)
rho_g    = Density(fluid$, P=in.P, x=1)
mu_l     = Viscosity(fluid$, P=in.P, x=0)
mu_g     = Viscosity(fluid$, P=in.P, x=1)
sigma    = SurfaceTension(fluid$, P=in.P)
A        = pi# / 4 * D^2
G        = in.mdot / A
V_lo     = G / rho_l
Re_lo    = reynolds(rho_l, V_lo, D, mu_l)
f_lo     = friction_factor(Re_lo, 0)
dP_lo    = f_lo * (L / D) * rho_l * V_lo^2 / 2
phi2     = friedel_phi2(x, rho_l, rho_g, mu_l, mu_g, G, D, sigma)
out.P    = in.P - phi2 * dP_lo
\`\`\``,
  },
  {
    name: `TwoPhaseInternalHX`,
    slug: `twophaseinternalhx`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component TwoPhaseInternalHX with ports liq_in, liq_out, vap_in, vap_out.`,
    related: [],
    examples: [],
    tags: [`twophaseinternalhx`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TwoPhaseInternalHX inst(fluid$, eps, domain$)
\`\`\`

## Ports

\`liq_in\`, \`liq_out\`, \`vap_in\`, \`vap_out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`eps\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
liq_out.mdot = liq_in.mdot
vap_out.mdot = vap_in.mdot
liq_out.P    = liq_in.P
vap_out.P    = vap_in.P
T_liq        = Temperature(fluid$, P=liq_in.P, h=liq_in.h)
T_vap        = Temperature(fluid$, P=vap_in.P, h=vap_in.h)
cp_v         = Cp(fluid$, P=vap_in.P, h=vap_in.h)
Q            = eps * vap_in.mdot * cp_v * (T_liq - T_vap)
vap_out.h    = vap_in.h + Q / vap_in.mdot
liq_out.h    = liq_in.h - Q / liq_in.mdot
\`\`\``,
  },
  {
    name: `TwoPhaseInventory`,
    slug: `twophaseinventory`,
    category: `Component (twophase)`,
    summary: `Tracks the refrigerant charge inventory across the circuit.`,
    related: [],
    examples: [],
    tags: [`twophaseinventory`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Tracks the refrigerant charge inventory across the circuit.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseInventory inst(fluid$, V, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`V\` | Number | Volume [m³]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.h    = in.h
hf       = Enthalpy(fluid$, P=in.P, x=0)
hg       = Enthalpy(fluid$, P=in.P, x=1)
x        = (in.h - hf) / (hg - hf)
rho_l    = Density(fluid$, P=in.P, x=0)
rho_g    = Density(fluid$, P=in.P, x=1)
alpha    = void_zivi(x, rho_l, rho_g)
rho_mix  = alpha * rho_g + (1 - alpha) * rho_l
m        = V * rho_mix
\`\`\``,
  },
  {
    name: `TwoPhaseMixer`,
    slug: `twophasemixer`,
    category: `Component (twophase)`,
    summary: `Mixes two two-phase streams with flow-weighted enthalpy.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`twophasemixer`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Mixes two two-phase streams with flow-weighted enthalpy.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in1\`, \`in2\`, \`out\`

## Usage

\`\`\`
TwoPhaseMixer inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P    = in1.P
out.mdot = in1.mdot + in2.mdot
out.mdot * out.h = in1.mdot * in1.h + in2.mdot * in2.h
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `TwoPhaseOilRider`,
    slug: `twophaseoilrider`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component TwoPhaseOilRider with ports in, out.`,
    related: [],
    examples: [],
    tags: [`twophaseoilrider`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TwoPhaseOilRider inst(oc_set, k_deg, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`oc_set\` | Number |
| \`k_deg\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.h    = in.h
out.oc   = oc_set
f_deg    = 1 - k_deg * oc_set
\`\`\``,
  },
  {
    name: `TwoPhasePipe`,
    slug: `twophasepipe`,
    category: `Component (twophase)`,
    summary: `A two-phase pipe with a Lockhart–Martinelli frictional pressure drop.`,
    related: [],
    examples: [],
    tags: [`twophasepipe`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase pipe with a Lockhart–Martinelli frictional pressure drop.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhasePipe inst(fluid$, L, D, rough, x, rho_l, rho_g, mu_l, mu_g)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`L\` | Number | Length [m]. |
| \`D\` | Number | Diameter [m]. |
| \`rough\` | Number | Relative wall roughness. |
| \`x\` | Number | Vapor quality / fraction (0–1). |
| \`rho_l\` | Number | Liquid density [kg/m³]. |
| \`rho_g\` | Number | Vapor density [kg/m³]. |
| \`mu_l\` | Number | Liquid viscosity [Pa·s]. |
| \`mu_g\` | Number | Vapor viscosity [Pa·s]. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.h    = in.h
A        = pi# / 4 * D^2
V_l      = in.mdot * (1 - x) / (rho_l * A)
Re_l     = reynolds(rho_l, V_l, D, mu_l)
f_l      = friction_factor(Re_l, rough / D)
dP_l     = f_l * (L / D) * rho_l * V_l^2 / 2
X_tt     = lm_martinelli_tt(x, rho_l, rho_g, mu_l, mu_g)
phi2     = lm_phi2(X_tt, 20)
out.P    = in.P - phi2 * dP_l
\`\`\``,
  },
  {
    name: `TwoPhasePressureSink`,
    slug: `twophasepressuresink`,
    category: `Component (twophase)`,
    summary: `A two-phase boundary fixing the pressure (sink).`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`twophasepressuresink`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase boundary fixing the pressure (sink).

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`

## Usage

\`\`\`
TwoPhasePressureSink inst(P, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`P\` | Number | Pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
in.P = P
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: pressure-cooker]`,
  },
  {
    name: `TwoPhasePressureSource`,
    slug: `twophasepressuresource`,
    category: `Component (twophase)`,
    summary: `A two-phase boundary fixing the pressure (source).`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`twophasepressuresource`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase boundary fixing the pressure (source).

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
TwoPhasePressureSource inst(fluid$, P, x, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`P\` | Number | Pressure [Pa]. |
| \`x\` | Number | Vapor quality / fraction (0–1). |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P = P
out.h = Enthalpy(fluid$, P=P, x=x)
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `TwoPhaseReceiver`,
    slug: `twophasereceiver`,
    category: `Component (twophase)`,
    summary: `A liquid receiver buffering refrigerant charge at saturation.`,
    related: [],
    examples: [],
    tags: [`twophasereceiver`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A liquid receiver buffering refrigerant charge at saturation.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseReceiver inst(fluid$, V, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`V\` | Number | Volume [m³]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.h    = Enthalpy(fluid$, P=in.P, x=0)
rho_l    = Density(fluid$, P=in.P, x=0)
rho_g    = Density(fluid$, P=in.P, x=1)
m        = V * (LL * rho_l + (1 - LL) * rho_g)
\`\`\``,
  },
  {
    name: `TwoPhaseSensor`,
    slug: `twophasesensor`,
    category: `Component (twophase)`,
    summary: `A sensor reading the two-phase stream state.`,
    related: [],
    examples: [],
    tags: [`twophasesensor`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A sensor reading the two-phase stream state.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseSensor inst(fluid$, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = in.mdot
out.P    = in.P
out.h    = in.h
hf       = Enthalpy(fluid$, P=in.P, x=0)
hg       = Enthalpy(fluid$, P=in.P, x=1)
x        = (in.h - hf) / (hg - hf)
T        = Temperature(fluid$, P=in.P, h=in.h)
Tsat     = T_sat(fluid$, P=in.P)
SH       = T - Tsat
rho_l    = Density(fluid$, P=in.P, x=0)
rho_g    = Density(fluid$, P=in.P, x=1)
alpha    = void_zivi(x, rho_l, rho_g)
\`\`\``,
  },
  {
    name: `TwoPhaseShortTube`,
    slug: `twophaseshorttube`,
    category: `Component (twophase)`,
    summary: `Acausal twophase-domain component TwoPhaseShortTube with ports in, out.`,
    related: [],
    examples: [],
    tags: [`twophaseshorttube`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `Reusable acausal **twophase-domain** component. Instantiate it and connect its ports; instantiation expands the constitutive equations below into scalar equations solved by the standard Newton/Tarjan pipeline.

> **Auto-generated** from the component library (\`backend/core/src/main/resources/components/\`). The ports, parameters, and constitutive equations are taken verbatim from the component definition; a worked example and prose discussion are added as the page is curated.

## Usage

\`\`\`
TwoPhaseShortTube inst(fluid$, CdA, domain$)
\`\`\`

## Ports

\`in\`, \`out\`

## Parameters

| Parameter | Type |
| --- | --- |
| \`fluid$\` | String |
| \`CdA\` | Number |
| \`domain$\` | String |

## Constitutive Equations

The acausal equations this component expands into (over its port members and parameters):

\`\`\`
out.mdot = in.mdot
out.h    = in.h
rho_in   = Density(fluid$, P=in.P, h=in.h)
T_in     = Temperature(fluid$, P=in.P, h=in.h)
Pf       = P_sat(fluid$, T=T_in)
dP_eff   = in.P - max(out.P, Pf)
in.mdot * abs(in.mdot) = CdA^2 * 2 * rho_in * dP_eff
\`\`\``,
  },
  {
    name: `TwoPhaseSink`,
    slug: `twophasesink`,
    category: `Component (twophase)`,
    summary: `A boundary absorbing a two-phase stream.`,
    related: [],
    examples: [`ev-thermal-management`],
    tags: [`twophasesink`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A boundary absorbing a two-phase stream.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`

## Usage

\`\`\`
TwoPhaseSink inst(domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
mdot = in.mdot
P    = in.P
h    = in.h
\`\`\`

## Examples

Instantiated in the verified example below:

[Run: ev-thermal-management]`,
  },
  {
    name: `TwoPhaseSource`,
    slug: `twophasesource`,
    category: `Component (twophase)`,
    summary: `A boundary supplying a two-phase stream.`,
    related: [],
    examples: [],
    tags: [`twophasesource`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A boundary supplying a two-phase stream.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
TwoPhaseSource inst(fluid$, mdot, P, x, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`mdot\` | Number | Mass flow rate [kg/s]. |
| \`P\` | Number | Pressure [Pa]. |
| \`x\` | Number | Vapor quality / fraction (0–1). |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = mdot
out.P    = P
out.h    = Enthalpy(fluid$, P=P, x=x)
\`\`\``,
  },
  {
    name: `TwoPhaseSourcePH`,
    slug: `twophasesourceph`,
    category: `Component (twophase)`,
    summary: `A two-phase source specified by pressure and enthalpy (P, h).`,
    related: [],
    examples: [],
    tags: [`twophasesourceph`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A two-phase source specified by pressure and enthalpy \`(P, h)\`.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`out\`

## Usage

\`\`\`
TwoPhaseSourcePH inst(mdot, P, h, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`mdot\` | Number | Mass flow rate [kg/s]. |
| \`P\` | Number | Pressure [Pa]. |
| \`h\` | Number | Heat-transfer coefficient [W/m²·K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot = mdot
out.P    = P
out.h    = h
\`\`\``,
  },
  {
    name: `TwoPhaseVolume`,
    slug: `twophasevolume`,
    category: `Component (twophase)`,
    summary: `A finite-volume two-phase control volume with mass and energy states ((p, h) states).`,
    related: [],
    examples: [],
    tags: [`twophasevolume`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A finite-volume two-phase control volume with mass and energy states (\`(p, h)\` states).

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`

## Usage

\`\`\`
TwoPhaseVolume inst(fluid$, V, C, P0, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`V\` | Number | Volume [m³]. |
| \`C\` | Number | Capacitance [F]. |
| \`P0\` | Number | Reference/initial pressure [Pa]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.P       = in.P
out.h       = in.h
der(in.P)   = (in.mdot - out.mdot) / C
init(in.P)  = P0
hf          = Enthalpy(fluid$, P=in.P, x=0)
hg          = Enthalpy(fluid$, P=in.P, x=1)
x           = (in.h - hf) / (hg - hf)
rho_l       = Density(fluid$, P=in.P, x=0)
rho_g       = Density(fluid$, P=in.P, x=1)
alpha       = void_zivi(x, rho_l, rho_g)
rho_mix     = alpha * rho_g + (1 - alpha) * rho_l
m           = V * rho_mix
\`\`\``,
  },
  {
    name: `TXVSuperheat`,
    slug: `txvsuperheat`,
    category: `Component (twophase)`,
    summary: `A thermostatic expansion valve that meters flow to hold a target superheat.`,
    related: [],
    examples: [],
    tags: [`txvsuperheat`, `component`, `twophase`, `acausal`],
    references: [],
    guides: [],
    body: `A thermostatic expansion valve that meters flow to hold a target superheat.

## Domain

A reusable **acausal twophase-domain** component — its two-phase refrigerant ports carry pressure \`P\`, mass-flow \`ṁ\`, and specific enthalpy \`h\` (quality/void follow from the properties). Instantiate it and connect its ports; the constitutive equations below expand into the global scalar system.

## Ports

\`in\`, \`out\`, \`bulb\`

## Usage

\`\`\`
TXVSuperheat inst(fluid$, Kv, SH_set, domain$)
\`\`\`

## Parameters

| Parameter | Type | Description |
| --- | --- | --- |
| \`fluid$\` | String | Fluid name (e.g. Water, R134a, Air). |
| \`Kv\` | Number | Flow coefficient. |
| \`SH_set\` | Number | Target superheat [K]. |
| \`domain$\` | String | Connector fluid family — one of \`fluid\`, \`gas\`, \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`. |

## Constitutive Equations

Instantiating the component expands these acausal equations (over its port members and parameters) into scalar equations solved by the standard Newton/Tarjan pipeline:

\`\`\`
out.mdot  = in.mdot
out.h     = in.h
bulb.Qdot = 0
T_sat     = Temperature(fluid$, P=out.P, x=1)
SH        = bulb.T - T_sat
in.mdot   = Kv * (SH - SH_set)
\`\`\``,
  },
  {
    name: `a_astar`,
    slug: `a_astar`,
    category: `Compressible Flow`,
    summary: `Isentropic area ratio A/A*`,
    related: [],
    examples: [],
    tags: [`astar`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Isentropic area ratio A/A*


## Syntax

\`\`\`
A_Astar(M, k)
\`\`\`

## Description

Isentropic area ratio A/A*

## Mathematical Formulation

$$ \\frac{A}{A^*} = \\frac{1}{M}\\left[\\frac{2}{k+1}\\left(1 + \\tfrac{k-1}{2}M^2\\right)\\right]^{(k+1)/[2(k-1)]} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `beta_oblique`,
    slug: `beta_oblique`,
    category: `Compressible Flow`,
    summary: `Oblique-shock wave angle ('weak'|'strong') [rad]`,
    related: [],
    examples: [],
    tags: [`beta`, `oblique`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Oblique-shock wave angle ('weak'|'strong') [rad]


## Syntax

\`\`\`
beta_oblique(M1, theta, k, branch$)
\`\`\`

## Description

Oblique-shock wave angle ('weak'|'strong') [rad]

## Mathematical Formulation

$$ \\text{solve the } \\theta\\text{-}\\beta\\text{-}M \\text{ relation for the wave angle } \\beta \\ (\\text{weak/strong root}) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M1\` | Number | Yes | Upstream Mach number (≥ 1). |
| \`theta\` | Number | Yes | Flow-deflection angle [rad]. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |
| \`branch$\` | String | Yes | Selector — One of \`weak\`, \`strong\`. |`,
  },
  {
    name: `fanno_fld`,
    slug: `fanno_fld`,
    category: `Compressible Flow`,
    summary: `Fanno friction parameter 4*f*Lmax/D`,
    related: [],
    examples: [],
    tags: [`fanno`, `fld`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Fanno friction parameter 4*f*Lmax/D


## Syntax

\`\`\`
fanno_fLD(M, k)
\`\`\`

## Description

Fanno friction parameter 4*f*Lmax/D

## Mathematical Formulation

$$ \\frac{4 f L^*}{D} = \\frac{1-M^2}{kM^2} + \\frac{k+1}{2k}\\ln\\frac{(k+1)M^2}{2 + (k-1)M^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `fanno_p_pstar`,
    slug: `fanno_p_pstar`,
    category: `Compressible Flow`,
    summary: `Fanno static-pressure ratio`,
    related: [],
    examples: [],
    tags: [`fanno`, `pstar`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Fanno static-pressure ratio


## Syntax

\`\`\`
fanno_P_Pstar(M, k)
\`\`\`

## Description

Fanno static-pressure ratio

## Mathematical Formulation

$$ \\frac{P}{P^*} = \\frac{1}{M}\\sqrt{\\frac{k+1}{2 + (k-1)M^2}} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `fanno_p0_p0star`,
    slug: `fanno_p0_p0star`,
    category: `Compressible Flow`,
    summary: `Fanno stagnation-pressure ratio`,
    related: [],
    examples: [],
    tags: [`fanno`, `p0`, `p0star`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Fanno stagnation-pressure ratio


## Syntax

\`\`\`
fanno_P0_P0star(M, k)
\`\`\`

## Description

Fanno stagnation-pressure ratio

## Mathematical Formulation

$$ \\frac{P_0}{P_0^*} = \\frac{1}{M}\\left[\\frac{2 + (k-1)M^2}{k+1}\\right]^{(k+1)/[2(k-1)]} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `fanno_t_tstar`,
    slug: `fanno_t_tstar`,
    category: `Compressible Flow`,
    summary: `Fanno static-temperature ratio`,
    related: [],
    examples: [],
    tags: [`fanno`, `tstar`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Fanno static-temperature ratio


## Syntax

\`\`\`
fanno_T_Tstar(M, k)
\`\`\`

## Description

Fanno static-temperature ratio

## Mathematical Formulation

$$ \\frac{T}{T^*} = \\frac{k+1}{2 + (k-1)M^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `M2_shock`,
    slug: `m2_shock`,
    category: `Compressible Flow`,
    summary: `Downstream Mach number across a normal shock M2(M1, k).`,
    related: [`P2_P1_shock`, `P02_P01_shock`, `mach_A_Astar`],
    examples: [`cd-nozzle-shock`],
    tags: [`compressible`, `normal shock`, `mach`, `supersonic`, `subsonic`, `nozzle`],
    references: [],
    guides: [],
    body: `Returns the **Mach number downstream of a normal shock** given the supersonic
upstream Mach \`M1\` and specific-heat ratio \`k\`. A normal shock always decelerates
the flow to subsonic (\`M2 < 1\`).

## Syntax

\`\`\`
M2 = M2_shock(M1, k)
\`\`\`

## Description

Across a normal shock the flow jumps discontinuously from supersonic to subsonic
while conserving mass, momentum, and energy. \`M2\` depends only on \`M1\` and \`k\`.

## Mathematical Formulation

$$ M_2 = \\sqrt{\\frac{(k-1)\\,M_1^2 + 2}{2k\\,M_1^2 - (k-1)}} $$

> **Method:** direct evaluation; valid for \`M1 ≥ 1\` (a shock requires supersonic
> inflow). \`M1 = 1\` gives \`M2 = 1\` (vanishing shock).

## Examples

### Example 1 — Subsonic Mach after a nozzle shock

[Run: cd-nozzle-shock]

**Expected:** at \`M1 ≈ 2.20\`, \`k = 1.4\`, \`M2_shock ≈ 0.55\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M1\` | Number | Yes | Upstream Mach number (≥ 1, dimensionless). |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`M2\` | Number | Downstream (subsonic) Mach number. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`M1 < 1\` | A normal shock requires supersonic inflow; check the upstream state. |`,
  },
  {
    name: `mach_A_Astar`,
    slug: `mach_a_astar`,
    category: `Compressible Flow`,
    summary: `Mach number from the isentropic area ratio A/A* and flow regime.`,
    related: [`T0_T`, `P0_P`, `M2_shock`],
    examples: [`cd-nozzle-shock`],
    tags: [`compressible`, `isentropic`, `area ratio`, `mach`, `nozzle`, `subsonic`, `supersonic`],
    references: [],
    guides: [],
    body: `Returns the **Mach number** at a duct station from its isentropic **area ratio**
\`A/A*\` and a regime selector. Because the area ratio is double-valued, \`regime$\`
picks the \`'subsonic'\` or \`'supersonic'\` root — essential for locating the state
in a converging–diverging nozzle.

## Syntax

\`\`\`
M = mach_A_Astar(A_Astar, k, regime$)
\`\`\`

## Description

For isentropic flow, each area ratio \`A/A* ≥ 1\` corresponds to one subsonic and
one supersonic Mach number (the throat is \`A/A* = 1\`, \`M = 1\`). This function
inverts the area–Mach relation for the requested branch (a bounded root solve).

## Mathematical Formulation

The forward area–Mach relation (inverted here for \`M\`):

$$ \\frac{A}{A^*} = \\frac{1}{M}\\left[\\left(\\frac{2}{k+1}\\right)\\left(1 + \\frac{k-1}{2}M^2\\right)\\right]^{(k+1)/[2(k-1)]} $$

> **Method:** bounded numerical inversion of the area–Mach relation on the branch selected by
> \`regime$\` (\`M ≤ 1\` subsonic, \`M ≥ 1\` supersonic).

## Examples

### Example 1 — Supersonic Mach at a nozzle shock station

A normal shock stands where \`A/A* = 2.0\`; the supersonic upstream Mach is
\`M1 = mach_A_Astar(2.0, 1.4, 'supersonic')\`.

[Run: cd-nozzle-shock]

**Expected:** \`mach_A_Astar(2.0, 1.4, 'supersonic') ≈ 2.20\` (the subsonic root is ≈ 0.31).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A_Astar\` | Number | Yes | Area ratio A/A* (≥ 1, dimensionless). |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |
| \`regime$\` | String | Yes | Branch: \`'subsonic'\` or \`'supersonic'\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`M\` | Number | Mach number on the selected branch. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`A_Astar < 1\` | The area ratio cannot be below 1 (the sonic throat). Check the geometry. |
| \`UNKNOWN_REGIME\` | \`regime$\` not recognized | Use \`'subsonic'\` or \`'supersonic'\`. |`,
  },
  {
    name: `mach_prandtlmeyer`,
    slug: `mach_prandtlmeyer`,
    category: `Compressible Flow`,
    summary: `Mach from Prandtl-Meyer angle [rad]`,
    related: [],
    examples: [],
    tags: [`mach`, `prandtlmeyer`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Mach from Prandtl-Meyer angle [rad]


## Syntax

\`\`\`
mach_PrandtlMeyer(nu, k)
\`\`\`

## Description

Mach from Prandtl-Meyer angle [rad]

## Mathematical Formulation

$$ \\text{solve } \\nu(M) = \\nu_{\\text{target}} \\text{ for } M \\quad (M \\ge 1) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`nu\` | Number | Yes | Prandtl–Meyer angle [rad]. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `machangle`,
    slug: `machangle`,
    category: `Compressible Flow`,
    summary: `Mach angle mu = asin(1/M) [rad]`,
    related: [],
    examples: [],
    tags: [`machangle`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Mach angle mu = asin(1/M) [rad]


## Syntax

\`\`\`
MachAngle(M)
\`\`\`

## Description

Mach angle mu = asin(1/M) [rad]

## Mathematical Formulation

$$ \\mu = \\arcsin\\!\\frac{1}{M} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |`,
  },
  {
    name: `P0_P`,
    slug: `p0_p`,
    category: `Compressible Flow`,
    summary: `Isentropic stagnation-to-static pressure ratio P0/P(M, k).`,
    related: [`T0_T`, `mach_A_Astar`, `stagnationpres`],
    examples: [`cd-nozzle-shock`],
    tags: [`compressible`, `isentropic`, `stagnation`, `pressure`, `mach`, `nozzle`],
    references: [],
    guides: [],
    body: `Returns the **isentropic stagnation-to-static pressure ratio** \`P0/P\` for an ideal
gas at Mach number \`M\` with specific-heat ratio \`k\`. Use it to convert between
reservoir and local pressure in isentropic nozzle and diffuser flow.

## Syntax

\`\`\`
ratio = P0_P(M, k)
\`\`\`

## Description

For isentropic flow the pressure ratio is the temperature ratio raised to
\`k/(k−1)\`. The local static pressure follows from a known stagnation value as
\`P = P0 / P0_P(M, k)\`.

## Mathematical Formulation

$$ \\frac{P_0}{P} = \\left(1 + \\frac{k-1}{2}\\,M^2\\right)^{\\!k/(k-1)} $$

> **Method:** direct evaluation; consistent with \`T0_T\` via the isentropic
> relation \`P0/P = (T0/T)^{k/(k-1)}\`.

## Examples

### Example 1 — Static pressure upstream of a nozzle shock

The supersonic static pressure at the shock station: \`P1 = P0 / P0_P(M1, k)\`.

[Run: cd-nozzle-shock]

**Expected:** at \`M1 ≈ 2.20\`, \`k = 1.4\`, \`P0_P ≈ 10.6\`, so \`P1 ≈ 94 kPa\` from \`P0 = 1 MPa\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number (≥ 0, dimensionless). |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`ratio\` | Number | Stagnation-to-static pressure ratio P0/P (≥ 1). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`M\` negative or \`k ≤ 1\` | Use a non-negative Mach and a physical \`k > 1\`. |`,
  },
  {
    name: `P02_P01_shock`,
    slug: `p02_p01_shock`,
    category: `Compressible Flow`,
    summary: `Stagnation pressure ratio across a normal shock P02/P01(M1, k).`,
    related: [`M2_shock`, `P2_P1_shock`, `P0_P`],
    examples: [`cd-nozzle-shock`],
    tags: [`compressible`, `normal shock`, `stagnation pressure`, `loss`, `irreversibility`, `nozzle`],
    references: [],
    guides: [],
    body: `Returns the **stagnation pressure ratio across a normal shock** \`P02/P01\` from the
upstream Mach \`M1\` and specific-heat ratio \`k\`. A shock is irreversible, so
stagnation pressure always drops (\`P02/P01 < 1\`) — this ratio quantifies the loss.

## Syntax

\`\`\`
ratio = P02_P01_shock(M1, k)
\`\`\`

## Description

Although static pressure rises across a shock, the entropy generated reduces the
stagnation (total) pressure. The recovered stagnation pressure downstream is
\`P02 = P01 · P02_P01_shock(M1, k)\`.

## Mathematical Formulation

$$ \\frac{P_{02}}{P_{01}} = \\left[\\frac{(k+1)M_1^2}{2 + (k-1)M_1^2}\\right]^{k/(k-1)}\\left[\\frac{k+1}{2k\\,M_1^2 - (k-1)}\\right]^{1/(k-1)} $$

> **Method:** direct evaluation; \`≤ 1\` with equality only at \`M1 = 1\`, decreasing
> as the shock strengthens.

## Examples

### Example 1 — Stagnation pressure loss across a nozzle shock

[Run: cd-nozzle-shock]

**Expected:** at \`M1 ≈ 2.20\`, \`k = 1.4\`, \`P02_P01_shock ≈ 0.63\`, so \`P02 ≈ 628 kPa\` from \`P01 = 1 MPa\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M1\` | Number | Yes | Upstream Mach number (≥ 1, dimensionless). |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`ratio\` | Number | Stagnation pressure ratio P02/P01 (≤ 1). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`M1 < 1\` | A normal shock requires supersonic inflow; check the upstream state. |`,
  },
  {
    name: `P2_P1_shock`,
    slug: `p2_p1_shock`,
    category: `Compressible Flow`,
    summary: `Static pressure ratio across a normal shock P2/P1(M1, k).`,
    related: [`M2_shock`, `P02_P01_shock`, `P0_P`],
    examples: [`cd-nozzle-shock`],
    tags: [`compressible`, `normal shock`, `pressure`, `rankine-hugoniot`, `nozzle`],
    references: [],
    guides: [],
    body: `Returns the **static pressure ratio across a normal shock** \`P2/P1\` from the
upstream Mach \`M1\` and specific-heat ratio \`k\`. The flow compresses across the
shock, so \`P2/P1 > 1\`.

## Syntax

\`\`\`
ratio = P2_P1_shock(M1, k)
\`\`\`

## Description

The static pressure rises sharply across a normal shock. The downstream static
pressure follows from the upstream value as \`P2 = P1 · P2_P1_shock(M1, k)\`.

## Mathematical Formulation

$$ \\frac{P_2}{P_1} = \\frac{2k\\,M_1^2 - (k-1)}{k+1} $$

> **Method:** direct evaluation (Rankine–Hugoniot static pressure ratio); valid
> for \`M1 ≥ 1\`.

## Examples

### Example 1 — Static pressure jump across a nozzle shock

[Run: cd-nozzle-shock]

**Expected:** at \`M1 ≈ 2.20\`, \`k = 1.4\`, \`P2_P1_shock ≈ 5.47\`, so \`P2 ≈ 514 kPa\` from \`P1 ≈ 94 kPa\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M1\` | Number | Yes | Upstream Mach number (≥ 1, dimensionless). |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`ratio\` | Number | Static pressure ratio P2/P1 (≥ 1). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`M1 < 1\` | A normal shock requires supersonic inflow; check the upstream state. |`,
  },
  {
    name: `prandtlmeyer`,
    slug: `prandtlmeyer`,
    category: `Compressible Flow`,
    summary: `Prandtl-Meyer angle nu(M) [rad]`,
    related: [],
    examples: [],
    tags: [`prandtlmeyer`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Prandtl-Meyer angle nu(M) [rad]


## Syntax

\`\`\`
PrandtlMeyer(M, k)
\`\`\`

## Description

Prandtl-Meyer angle nu(M) [rad]

## Mathematical Formulation

$$ \\nu(M) = \\sqrt{\\tfrac{k+1}{k-1}}\\,\\arctan\\!\\sqrt{\\tfrac{k-1}{k+1}(M^2-1)} - \\arctan\\!\\sqrt{M^2-1} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `rayleigh_p_pstar`,
    slug: `rayleigh_p_pstar`,
    category: `Compressible Flow`,
    summary: `Rayleigh static-pressure ratio`,
    related: [],
    examples: [],
    tags: [`rayleigh`, `pstar`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Rayleigh static-pressure ratio


## Syntax

\`\`\`
rayleigh_P_Pstar(M, k)
\`\`\`

## Description

Rayleigh static-pressure ratio

## Mathematical Formulation

$$ \\frac{P}{P^*} = \\frac{k+1}{1 + kM^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `rayleigh_p0_p0star`,
    slug: `rayleigh_p0_p0star`,
    category: `Compressible Flow`,
    summary: `Rayleigh stagnation-pressure ratio`,
    related: [],
    examples: [],
    tags: [`rayleigh`, `p0`, `p0star`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Rayleigh stagnation-pressure ratio


## Syntax

\`\`\`
rayleigh_P0_P0star(M, k)
\`\`\`

## Description

Rayleigh stagnation-pressure ratio

## Mathematical Formulation

$$ \\frac{P_0}{P_0^*} = \\frac{k+1}{1+kM^2}\\left[\\frac{2 + (k-1)M^2}{k+1}\\right]^{k/(k-1)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `rayleigh_t_tstar`,
    slug: `rayleigh_t_tstar`,
    category: `Compressible Flow`,
    summary: `Rayleigh static-temperature ratio`,
    related: [],
    examples: [],
    tags: [`rayleigh`, `tstar`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Rayleigh static-temperature ratio


## Syntax

\`\`\`
rayleigh_T_Tstar(M, k)
\`\`\`

## Description

Rayleigh static-temperature ratio

## Mathematical Formulation

$$ \\frac{T}{T^*} = \\left(\\frac{(k+1)M}{1 + kM^2}\\right)^2 $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `rayleigh_t0_t0star`,
    slug: `rayleigh_t0_t0star`,
    category: `Compressible Flow`,
    summary: `Rayleigh stagnation-temperature ratio`,
    related: [],
    examples: [],
    tags: [`rayleigh`, `t0`, `t0star`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Rayleigh stagnation-temperature ratio


## Syntax

\`\`\`
rayleigh_T0_T0star(M, k)
\`\`\`

## Description

Rayleigh stagnation-temperature ratio

## Mathematical Formulation

$$ \\frac{T_0}{T_0^*} = \\frac{(k+1)M^2\\,[2 + (k-1)M^2]}{(1 + kM^2)^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `rho0_rho`,
    slug: `rho0_rho`,
    category: `Compressible Flow`,
    summary: `Isentropic stagnation/static density ratio`,
    related: [],
    examples: [],
    tags: [`rho0`, `rho`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Isentropic stagnation/static density ratio


## Syntax

\`\`\`
rho0_rho(M, k)
\`\`\`

## Description

Isentropic stagnation/static density ratio

## Mathematical Formulation

$$ \\frac{\\rho_0}{\\rho} = \\left(1 + \\tfrac{k-1}{2}M^2\\right)^{1/(k-1)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `rho2_rho1_shock`,
    slug: `rho2_rho1_shock`,
    category: `Compressible Flow`,
    summary: `Normal-shock density ratio`,
    related: [],
    examples: [],
    tags: [`rho2`, `rho1`, `shock`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Normal-shock density ratio


## Syntax

\`\`\`
rho2_rho1_shock(M1, k)
\`\`\`

## Description

Normal-shock density ratio

## Mathematical Formulation

$$ \\frac{\\rho_2}{\\rho_1} = \\frac{(k+1)M_1^2}{2 + (k-1)M_1^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M1\` | Number | Yes | Upstream Mach number (≥ 1). |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `StagnationPres`,
    slug: `stagnationpres`,
    category: `Compressible Flow`,
    summary: `Stagnation pressure P0 = P·(T0/T)^(k/(k-1)).`,
    related: [`StagnationTemp`, `P0_P`],
    examples: [`thermo-compliance`],
    tags: [`compressible`, `stagnation pressure`, `total pressure`, `isentropic`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **stagnation (total) pressure** \`P0\` of a flowing gas brought
isentropically to rest, from the static pressure \`P\`, static temperature \`T\`,
stagnation temperature \`T0\`, and specific-heat ratio \`k\`.

## Syntax

\`\`\`
P0 = StagnationPres(P, T, T0, k)
\`\`\`

## Description

Built on the isentropic relation between pressure and temperature ratios, it pairs
with \`StagnationTemp\` to give the full stagnation state.

## Mathematical Formulation

$$ P_0 = P\\left(\\frac{T_0}{T}\\right)^{\\!k/(k-1)} $$

> **Method:** direct evaluation of the isentropic stagnation relation.

## Examples

### Example 1 — Total pressure of a flow

[Run: thermo-compliance]

**Expected:** \`P0 > P\`, the ratio set by \`(T0/T)^{k/(k−1)}\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`P\` | Number | Yes | Static pressure [Pa]. |
| \`T\` | Number | Yes | Static temperature [K]. |
| \`T0\` | Number | Yes | Stagnation temperature [K]. |
| \`k\` | Number | Yes | Ratio of specific heats. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`P0\` | Number | Stagnation pressure [Pa]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`k ≤ 1\` or \`T ≤ 0\` | Use a physical \`k > 1\` and positive temperatures. |`,
  },
  {
    name: `StagnationTemp`,
    slug: `stagnationtemp`,
    category: `Compressible Flow`,
    summary: `Stagnation temperature T0 = T + V²/(2·cp).`,
    related: [`StagnationPres`, `T0_T`],
    examples: [`thermo-compliance`],
    tags: [`compressible`, `stagnation temperature`, `total temperature`, `energy`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **stagnation (total) temperature** \`T0\` of a flowing gas — the
temperature it would reach if brought adiabatically to rest — from the static
temperature \`T\`, velocity \`V\`, and specific heat \`cp\`.

## Syntax

\`\`\`
T0 = StagnationTemp(T, V, cp)
\`\`\`

## Description

The stagnation temperature adds the kinetic-energy contribution of the flow to the
static temperature. It is conserved along an adiabatic flow even as static
conditions change.

## Mathematical Formulation

$$ T_0 = T + \\frac{V^2}{2\\,c_p} $$

> **Method:** direct evaluation of the energy balance.

## Examples

### Example 1 — Total temperature of a flow

[Run: thermo-compliance]

**Expected:** \`T0 > T\`, the excess set by the kinetic term \`V²/(2cp)\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`T\` | Number | Yes | Static temperature [K]. |
| \`V\` | Number | Yes | Flow velocity [m/s]. |
| \`cp\` | Number | Yes | Specific heat at constant pressure [J/kg·K]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`T0\` | Number | Stagnation temperature [K]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`cp ≤ 0\` | Provide a positive specific heat. |`,
  },
  {
    name: `T0_T`,
    slug: `t0_t`,
    category: `Compressible Flow`,
    summary: `Isentropic stagnation-to-static temperature ratio T0/T(M, k).`,
    related: [`P0_P`, `mach_A_Astar`, `stagnationtemp`],
    examples: [`cd-nozzle-shock`],
    tags: [`compressible`, `isentropic`, `stagnation`, `temperature`, `mach`, `nozzle`],
    references: [],
    guides: [],
    body: `Returns the **isentropic stagnation-to-static temperature ratio** \`T0/T\` for an
ideal gas at Mach number \`M\` with specific-heat ratio \`k\`. Use it to convert
between reservoir (stagnation) and local (static) temperature in nozzle, diffuser,
and duct flow.

## Syntax

\`\`\`
ratio = T0_T(M, k)
\`\`\`

## Description

Bringing a compressible stream isentropically to rest raises its temperature by
the kinetic-energy term; the ratio depends only on \`M\` and \`k\`. The static
temperature follows from a known stagnation value as \`T = T0 / T0_T(M, k)\`.

## Mathematical Formulation

$$ \\frac{T_0}{T} = 1 + \\frac{k-1}{2}\\,M^2 $$

> **Method:** direct evaluation; dimensionless \`M\` and \`k\`.

## Examples

### Example 1 — Static temperature upstream of a nozzle shock

In a C-D nozzle, the supersonic static temperature at the shock station follows
from the reservoir temperature: \`T1 = T0 / T0_T(M1, k)\`.

[Run: cd-nozzle-shock]

**Expected:** at \`M1 ≈ 2.20\`, \`k = 1.4\`, \`T0_T ≈ 1.965\`, so \`T1 ≈ 254 K\` from \`T0 = 500 K\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M\` | Number | Yes | Mach number (≥ 0, dimensionless). |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`ratio\` | Number | Stagnation-to-static temperature ratio T0/T (≥ 1). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`M\` negative or \`k ≤ 1\` | Use a non-negative Mach and a physical \`k > 1\`. |`,
  },
  {
    name: `t2_t1_shock`,
    slug: `t2_t1_shock`,
    category: `Compressible Flow`,
    summary: `Normal-shock static temperature ratio`,
    related: [],
    examples: [],
    tags: [`t2`, `t1`, `shock`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Normal-shock static temperature ratio


## Syntax

\`\`\`
T2_T1_shock(M1, k)
\`\`\`

## Description

Normal-shock static temperature ratio

## Mathematical Formulation

$$ \\frac{T_2}{T_1} = \\frac{\\big[1 + \\tfrac{k-1}{2}M_1^2\\big]\\big[\\tfrac{2k}{k-1}M_1^2 - 1\\big]}{M_1^2\\,(k+1)^2/[2(k-1)]} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M1\` | Number | Yes | Upstream Mach number (≥ 1). |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `theta_oblique`,
    slug: `theta_oblique`,
    category: `Compressible Flow`,
    summary: `Oblique-shock deflection from wave angle [rad]`,
    related: [],
    examples: [],
    tags: [`theta`, `oblique`, `compressible`, `flow`],
    references: [],
    guides: [],
    body: `Oblique-shock deflection from wave angle [rad]


## Syntax

\`\`\`
theta_oblique(M1, beta, k)
\`\`\`

## Description

Oblique-shock deflection from wave angle [rad]

## Mathematical Formulation

$$ \\tan\\theta = 2\\cot\\beta\\,\\frac{M_1^2\\sin^2\\beta - 1}{M_1^2(k + \\cos 2\\beta) + 2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`M1\` | Number | Yes | Upstream Mach number (≥ 1). |
| \`beta\` | Number | Yes | Oblique-shock wave angle [rad]. |
| \`k\` | Number | Yes | Ratio of specific heats (e.g. 1.4 for air). |`,
  },
  {
    name: `acker`,
    slug: `acker`,
    category: `Control Systems`,
    summary: `Single-input pole placement via Ackermann's formula.`,
    related: [`place`, `ctrb`, `lqr`],
    examples: [],
    tags: [`control`, `pole placement`, `ackermann`, `state feedback`],
    references: [],
    guides: [],
    body: `Returns the **state-feedback gain** \`K\` for a single-input system using
**Ackermann's formula**, placing the closed-loop poles of \`(A, B)\` at the desired
locations \`pr ± j·pi\`. It is the explicit closed-form counterpart of
\`place\`.

## Syntax

\`\`\`
CALL acker(A, B, pr, pi : K)
K = acker(A, B, pr, pi)
\`\`\`

## Description

Ackermann's formula gives \`K\` directly from the desired characteristic polynomial
and the controllability matrix; it is exact for single-input systems but
numerically sensitive for high order.

## Mathematical Formulation

With desired characteristic polynomial \`Φ(s) = Π(s − p_i)\` and controllability
matrix \`C = [B AB … Aⁿ⁻¹B]\`:

$$ K = \\begin{bmatrix} 0 & \\cdots & 0 & 1 \\end{bmatrix}\\,\\mathcal{C}^{-1}\\,\\Phi(A) $$

> **Method:** Ackermann's formula evaluated from \`Φ(A)\` and \`ctrb(A, B)\`.

## Examples

\`\`\`
{ K = acker(A, B, [-2,-2], [1,-1]) for a single-input controllable plant }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix. |
| \`B\` | Vector | Yes | Single-input matrix. |
| \`pr\` | Vector | Yes | Real parts of the desired poles. |
| \`pi\` | Vector | Yes | Imaginary parts of the desired poles. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`K\` | Vector | State-feedback gain (\`u = −Kx\`). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NOT_CONTROLLABLE\` | \`ctrb(A, B)\` singular | Ackermann needs a controllable single-input pair. |`,
  },
  {
    name: `balreal`,
    slug: `balreal`,
    category: `Control Systems`,
    summary: `Internally-balanced state-space realization for model reduction.`,
    related: [`gram`, `ctrb`, `obsv`, `ss2tf`],
    examples: [`estimator-gramian-balreal`],
    tags: [`control`, `balanced realization`, `model reduction`, `gramian`, `hankel`],
    references: [],
    guides: [],
    body: `Returns an **internally-balanced realization** \`(Ab, Bb, Cb)\` of a state-space
system — a coordinate transform in which the controllability and observability
Gramians are equal and diagonal (the Hankel singular values). States with small
Hankel values can then be truncated for model reduction.

## Syntax

\`\`\`
CALL balreal(A, B, C : Ab, Bb, Cb)
[Ab, Bb, Cb] = balreal(A, B, C)
\`\`\`

## Description

Balancing ranks the state directions by their joint input-output energy, so a
reduced model that drops the least-significant states keeps the dominant dynamics.

## Mathematical Formulation

Find \`T\` such that the transformed Gramians satisfy:

$$ \\tilde W_c = \\tilde W_o = \\Sigma = \\mathrm{diag}(\\sigma_1 \\ge \\sigma_2 \\ge \\dots), \\qquad (A_b, B_b, C_b) = (T^{-1}AT,\\ T^{-1}B,\\ CT) $$

where the \`σ_i\` are the Hankel singular values.

> **Method:** compute the Gramians, form the balancing transform from their joint
> eigenstructure, and apply it.

## Examples

### Example 1 — Balanced realization of a plant

[Run: estimator-gramian-balreal]

**Expected:** a realization whose equal, diagonal Gramians expose the Hankel
singular values for truncation.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix (stable). |
| \`B\` | Matrix | Yes | Input matrix. |
| \`C\` | Matrix | Yes | Output matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Ab\` | Matrix | Balanced state matrix. |
| \`Bb\` | Matrix | Balanced input matrix. |
| \`Cb\` | Matrix | Balanced output matrix. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NOT_MINIMAL\` | system not controllable/observable | Balancing requires a minimal (or stable) realization. |`,
  },
  {
    name: `bode`,
    slug: `bode`,
    category: `Control Systems`,
    summary: `Bode frequency response — magnitude (dB) and phase (deg) versus frequency.`,
    related: [`nyquist`, `margin`],
    examples: [`control-analysis-report`],
    tags: [`control`, `bode`, `frequency response`, `magnitude`, `phase`, `frequency`],
    references: [],
    guides: [`comp-linearize`, `gs-repl`, `repl`, `symbolic-cas`, `tut-msd`, `tut-rlc`],
    body: `Returns the **Bode frequency response** of \`G(s) = num/den\` over a frequency
vector \`omega\`: magnitude in decibels and phase in degrees. Use it to read
bandwidth, resonance, roll-off, and the stability margins.

## Syntax

\`\`\`
CALL bode(num, den, omega : mag, phase)
[mag, phase] = bode(num, den, omega)
\`\`\`

## Description

Evaluating the transfer function on the imaginary axis \`s = jω\` gives the
steady-state response to a sinusoid at each frequency. \`mag\` and \`phase\` are
vectors aligned with \`omega\` (typically log-spaced).

## Mathematical Formulation

$$ \\text{mag}(\\omega) = 20\\log_{10}\\big|G(j\\omega)\\big| \\quad[\\text{dB}], \\qquad \\text{phase}(\\omega) = \\angle G(j\\omega) \\quad[\\text{deg}] $$

> **Method:** evaluate \`G(jω)\` at each \`omega\`; magnitude in dB, phase unwrapped in
> degrees.

## Examples

### Example 1 — Bode response of a second-order plant

50 log-spaced frequencies over \`G(s) = (s + 2)/(s² + 4s + 25)\`:

[Run: control-analysis-report]

**Expected:** a resonant rise near \`ω_n = 5 rad/s\` (ζ ≈ 0.4) and a high-frequency
roll-off of −20 dB/decade (one more pole than zero).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |
| \`omega\` | Vector | Yes | Frequencies [rad/s] (usually log-spaced). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`mag\` | Vector | Magnitude [dB] at each frequency. |
| \`phase\` | Vector | Phase [deg] at each frequency. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`EMPTY_FREQUENCY\` | \`omega\` is empty | Provide a frequency vector, e.g. \`omega = 0.1:50:100 | Log\`. |`,
  },
  {
    name: `c2d`,
    slug: `c2d`,
    category: `Control Systems`,
    summary: `Continuous-to-discrete transfer-function conversion (ZOH / Tustin).`,
    related: [`d2c`, `tf`, `pole`],
    examples: [`digital-control-c2d`],
    tags: [`control`, `discretization`, `c2d`, `zoh`, `tustin`, `digital`, `sampling`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Converts a continuous transfer function \`G(s) = num/den\` to its **discrete-time**
equivalent \`G(z) = numz/denz\` at sample time \`Ts\`, using the requested method
(\`'zoh'\` zero-order hold or \`'tustin'\` bilinear). Use it to design or implement a
controller on a digital (sampled) platform.

## Syntax

\`\`\`
CALL c2d(num, den, Ts, 'zoh' : numz, denz)
[numz, denz] = c2d(num, den, Ts, 'tustin')
\`\`\`

## Description

\`'zoh'\` assumes the input is held constant over each sample (the usual model for a
DAC); \`'tustin'\` (bilinear) maps the \`s\`-plane to the \`z\`-plane by a frequency-
warping substitution that preserves stability.

## Mathematical Formulation

Zero-order hold:

$$ G(z) = (1 - z^{-1})\\,\\mathcal{Z}\\!\\left\\{\\frac{G(s)}{s}\\right\\} $$

Tustin (bilinear):

$$ G(z) = G(s)\\Big|_{\\,s = \\frac{2}{T_s}\\frac{z-1}{z+1}} $$

> **Method:** ZOH step-invariant transform, or the bilinear substitution.

## Examples

### Example 1 — Discretize a controller

[Run: digital-control-c2d]

**Expected:** a \`numz/denz\` pair whose sampled response approximates the continuous
\`G(s)\` at the chosen \`Ts\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Continuous numerator (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Continuous denominator. |
| \`Ts\` | Number | Yes | Sample time [s]. |
| \`method$\` | String | Yes | \`'zoh'\` or \`'tustin'\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`numz\` | Vector | Discrete numerator (descending powers of \`z\`). |
| \`denz\` | Vector | Discrete denominator. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`BAD_SAMPLE_TIME\` | \`Ts ≤ 0\` | Use a positive sample time. |
| \`UNKNOWN_METHOD\` | method not recognized | Use \`'zoh'\` or \`'tustin'\`. |`,
  },
  {
    name: `ctrb`,
    slug: `ctrb`,
    category: `Control Systems`,
    summary: `Controllability matrix of a state-space pair (A, B).`,
    related: [`obsv`, `place`, `acker`, `gram`],
    examples: [],
    tags: [`control`, `controllability`, `state space`, `rank`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **controllability matrix** \`Co\` of the pair \`(A, B)\`. The system is
controllable — every state reachable by the input, hence arbitrary pole placement
is possible — iff \`Co\` has full rank.

## Syntax

\`\`\`
CALL ctrb(A, B : Co)
Co = ctrb(A, B)
\`\`\`

## Mathematical Formulation

For an \`n\`-state system:

$$ \\mathcal{C} = \\begin{bmatrix} B & AB & A^2B & \\cdots & A^{n-1}B \\end{bmatrix} $$

The pair is controllable iff \`rank(C) = n\`.

> **Method:** assemble the Krylov block \`[B, AB, …, Aⁿ⁻¹B]\`.

## Examples

\`\`\`
{ Co = ctrb(A, B); controllable iff rank(Co) = n }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix (n×n). |
| \`B\` | Matrix | Yes | Input matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Co\` | Matrix | Controllability matrix. |`,
  },
  {
    name: `d2c`,
    slug: `d2c`,
    category: `Control Systems`,
    summary: `Discrete-to-continuous transfer-function conversion (Tustin / ZOH).`,
    related: [`c2d`, `tf`, `pole`],
    examples: [],
    tags: [`control`, `discretization`, `d2c`, `tustin`, `zoh`, `continuous`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Converts a discrete transfer function \`G(z) = numz/denz\` to a continuous-time
equivalent \`G(s) = num/den\` at sample time \`Ts\` — the inverse of \`c2d\`,
using the requested method (\`'tustin'\` bilinear or \`'zoh'\`).

## Syntax

\`\`\`
CALL d2c(numz, denz, Ts, 'tustin' : num, den)
[num, den] = d2c(numz, denz, Ts, 'zoh')
\`\`\`

## Description

Use it to recover a continuous model from an identified or implemented discrete
controller for analysis in the s-domain.

## Mathematical Formulation

Tustin (bilinear), the inverse of the \`c2d\` substitution:

$$ G(s) = G(z)\\Big|_{\\,z = \\frac{1 + (T_s/2)s}{1 - (T_s/2)s}} $$

> **Method:** inverse bilinear substitution (or inverse ZOH).

## Examples

\`\`\`
{ [num, den] = d2c(numz, denz, Ts, 'tustin') }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`numz\` | Vector | Yes | Discrete numerator (descending powers of \`z\`). |
| \`denz\` | Vector | Yes | Discrete denominator. |
| \`Ts\` | Number | Yes | Sample time [s]. |
| \`method$\` | String | Yes | \`'tustin'\` or \`'zoh'\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`num\` | Vector | Continuous numerator (descending powers of \`s\`). |
| \`den\` | Vector | Continuous denominator. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`BAD_SAMPLE_TIME\` | \`Ts ≤ 0\` | Use a positive sample time. |`,
  },
  {
    name: `dare`,
    slug: `dare`,
    category: `Control Systems`,
    summary: `Solve the discrete algebraic Riccati equation.`,
    related: [`dlqr`, `lqr`, `dlyap`],
    examples: [],
    tags: [`control`, `riccati`, `discrete`, `optimal`, `dare`],
    references: [],
    guides: [],
    body: `Solves the **discrete algebraic Riccati equation** (DARE) for the stabilizing
solution \`X\`. It is the core of discrete optimal control — \`dlqr\` builds its
gain from this \`X\`.

## Syntax

\`\`\`
CALL dare(A, B, Q, R : X)
X = dare(A, B, Q, R)
\`\`\`

## Mathematical Formulation

$$ X = A^\\top X A - A^\\top X B\\,(R + B^\\top X B)^{-1} B^\\top X A + Q $$

with \`Q ⪰ 0\` (state weight) and \`R ≻ 0\` (input weight); the stabilizing \`X ⪰ 0\`
is the one for which the closed loop is Schur-stable.

> **Method:** Schur / structured-eigenvector solve of the symplectic pencil.

## Examples

\`\`\`
{ X = dare(A, B, Q, R); the LQR-optimal cost-to-go matrix }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | Discrete state matrix. |
| \`B\` | Matrix | Yes | Input matrix. |
| \`Q\` | Matrix | Yes | State weight (⪰ 0). |
| \`R\` | Matrix | Yes | Input weight (≻ 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`X\` | Matrix | Stabilizing symmetric solution. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NOT_STABILIZABLE\` | \`(A, B)\` not stabilizable | A stabilizing solution requires a stabilizable pair. |`,
  },
  {
    name: `dlqr`,
    slug: `dlqr`,
    category: `Control Systems`,
    summary: `Discrete-time LQR optimal state-feedback gain (via the DARE).`,
    related: [`lqr`, `dare`, `place`],
    examples: [],
    tags: [`control`, `lqr`, `discrete`, `optimal`, `state feedback`, `riccati`],
    references: [],
    guides: [],
    body: `Returns the **discrete-time LQR gain** \`K\` — the discrete counterpart of
\`lqr\`. The control \`u_k = −K x_k\` minimizes a summed quadratic cost on the
states and effort.

## Syntax

\`\`\`
CALL dlqr(A, B, Q, R : K)
K = dlqr(A, B, Q, R)
\`\`\`

## Mathematical Formulation

Minimizing \`J = Σ (xₖᵀQxₖ + uₖᵀRuₖ)\` gives

$$ K = (R + B^\\top X B)^{-1} B^\\top X A $$

where \`X\` is the stabilizing solution of the discrete algebraic Riccati equation
(\`dare\`).

> **Method:** solve the DARE for \`X\`, then form \`K\`.

## Examples

\`\`\`
{ K = dlqr(A, B, Q, R); discrete optimal regulator gain }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | Discrete state matrix. |
| \`B\` | Matrix | Yes | Input matrix. |
| \`Q\` | Matrix | Yes | State weight (⪰ 0). |
| \`R\` | Matrix | Yes | Input weight (≻ 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`K\` | Matrix | Discrete state-feedback gain (\`u = −Kx\`). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NOT_STABILIZABLE\` | \`(A, B)\` not stabilizable | The pair must be stabilizable. |`,
  },
  {
    name: `dlyap`,
    slug: `dlyap`,
    category: `Control Systems`,
    summary: `Solve the discrete Lyapunov (Stein) equation A·X·Aᵀ − X + Q = 0.`,
    related: [`lyap`, `dare`, `dlqr`],
    examples: [],
    tags: [`control`, `lyapunov`, `stein`, `discrete`, `stability`],
    references: [],
    guides: [],
    body: `Solves the **discrete Lyapunov (Stein) equation** for \`X\` — the discrete-time
counterpart of \`lyap\`. A positive-definite \`X\` for \`Q > 0\` certifies that
the discrete system \`A\` is Schur-stable (all eigenvalues inside the unit circle).

## Syntax

\`\`\`
CALL dlyap(A, Q : X)
X = dlyap(A, Q)
\`\`\`

## Mathematical Formulation

$$ A X A^\\top - X + Q = 0 $$

For a Schur-stable \`A\` (\`|λ_i(A)| < 1\`) and \`Q = Qᵀ ⪰ 0\`, the unique solution is

$$ X = \\sum_{k=0}^{\\infty} A^k Q\\,(A^\\top)^k $$

> **Method:** Bartels–Stewart-type (Schur-based) solve of the Stein equation.

## Examples

\`\`\`
{ X = dlyap(A, Q); X > 0 certifies A is Schur-stable when Q > 0 }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | Discrete state matrix. |
| \`Q\` | Matrix | Yes | Symmetric right-hand side. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`X\` | Matrix | Symmetric solution. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NO_UNIQUE_SOLUTION\` | \`λ_i·λ_j = 1\` for some eigenvalue pair | The Stein operator is singular; check \`A\`'s spectrum. |`,
  },
  {
    name: `errorconst`,
    slug: `errorconst`,
    category: `Control Systems`,
    summary: `Static error constants Kp, Kv, Ka of an open-loop system.`,
    related: [`margin`, `feedback`, `step`],
    examples: [],
    tags: [`control`, `error constant`, `steady state error`, `position`, `velocity`, `acceleration`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **static error constants** — position \`Kp\`, velocity \`Kv\`, and
acceleration \`Ka\` — of an open-loop system \`num/den\`. They set the steady-state
tracking error of the unity-feedback closed loop to step, ramp, and parabolic
inputs.

## Syntax

\`\`\`
CALL errorconst(num, den : Kp, Kv, Ka)
[Kp, Kv, Ka] = errorconst(num, den)
\`\`\`

## Mathematical Formulation

For open-loop \`G(s)\`:

$$ K_p = \\lim_{s\\to 0} G(s), \\quad K_v = \\lim_{s\\to 0} s\\,G(s), \\quad K_a = \\lim_{s\\to 0} s^2 G(s) $$

with steady-state errors \`e_step = 1/(1+Kp)\`, \`e_ramp = 1/Kv\`, \`e_parabola = 1/Ka\`.

> **Method:** evaluate the low-frequency limits from the system type (number of
> integrators).

## Examples

\`\`\`
{ [Kp, Kv, Ka] = errorconst(num, den); a type-1 system has finite Kv, infinite Kp }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Open-loop numerator (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Open-loop denominator. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Kp\` | Number | Position error constant. |
| \`Kv\` | Number | Velocity error constant. |
| \`Ka\` | Number | Acceleration error constant. |`,
  },
  {
    name: `feedback`,
    slug: `feedback`,
    category: `Control Systems`,
    summary: `Close a feedback loop, T = G1/(1 + G1·G2).`,
    related: [`series`, `margin`, `pole`],
    examples: [`cruise-control`],
    tags: [`control`, `feedback`, `closed loop`, `block diagram`, `transfer function`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **closed-loop transfer function** of a feedback interconnection —
\`T(s) = G1(s) / (1 + G1(s)·G2(s))\` — as a single \`num/den\` pair. With \`G2 = 1\`
(unity feedback) it gives the standard reference-to-output closed loop.

## Syntax

\`\`\`
CALL feedback(num1, den1, num2, den2 : num, den)
[num, den] = feedback(num1, den1, num2, den2)
\`\`\`

## Description

\`G1\` is the forward path and \`G2\` the feedback path (default negative feedback).
The closed-loop poles — the roots of \`1 + G1·G2\` — set the stability and transient
behavior of the loop.

## Mathematical Formulation

For negative feedback,

$$ T(s) = \\frac{G_1(s)}{1 + G_1(s)\\,G_2(s)} = \\frac{\\text{num}_1\\,\\text{den}_2}{\\text{den}_1\\,\\text{den}_2 + \\text{num}_1\\,\\text{num}_2} $$

> **Method:** form the closed-loop numerator and denominator by polynomial
> multiplication and addition.

## Examples

### Example 1 — Closed-loop cruise control

Close the open-loop \`L(s)\` with unity feedback (\`H(s) = 1\`) to get the
reference-to-velocity closed loop \`T(s)\`:

[Run: cruise-control]

**Expected:** \`T(s) = L/(1 + L)\` — a stable closed loop tracking the set speed,
with the PI integrator giving zero steady-state error.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num1\`, \`den1\` | Vector | Yes | Forward-path transfer function \`G1\`. |
| \`num2\`, \`den2\` | Vector | Yes | Feedback-path transfer function \`G2\` (use \`[1],[1]\` for unity feedback). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`num\` | Vector | Closed-loop numerator. |
| \`den\` | Vector | Closed-loop denominator (roots = closed-loop poles). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`IMPROPER_LOOP\` | Forward path improper | Ensure \`G1\` is proper so the closed loop is realizable. |`,
  },
  {
    name: `gram`,
    slug: `gram`,
    category: `Control Systems`,
    summary: `Controllability or observability Gramian of a state-space system.`,
    related: [`ctrb`, `obsv`, `balreal`],
    examples: [`estimator-gramian-balreal`],
    tags: [`control`, `gramian`, `controllability`, `observability`, `balanced`],
    references: [],
    guides: [],
    body: `Returns the **controllability** (\`'c'\`) or **observability** (\`'o'\`) Gramian of a
stable state-space system. The Gramians quantify how strongly each state direction
is excited by the input / observed at the output, and underpin balanced model
reduction (\`balreal\`).

## Syntax

\`\`\`
CALL gram(A, M, 'c' : W)
W = gram(A, M, 'o')
\`\`\`

## Description

For \`type$ = 'c'\`, \`M = B\` and \`W\` is the controllability Gramian; for \`'o'\`,
\`M = C\` and \`W\` is the observability Gramian. Both are symmetric positive-definite
for a stable, controllable/observable system.

## Mathematical Formulation

The Gramians solve the Lyapunov equations:

$$ A W_c + W_c A^\\top + B B^\\top = 0, \\qquad A^\\top W_o + W_o A + C^\\top C = 0 $$

equivalently $W_c = \\int_0^\\infty e^{A t} B B^\\top e^{A^\\top t}\\,dt$.

> **Method:** solve the appropriate Lyapunov equation for \`W\`.

## Examples

### Example 1 — Gramian of a plant

[Run: estimator-gramian-balreal]

**Expected:** a symmetric positive-definite Gramian whose eigenstructure ranks the
state directions by controllability/observability.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix (must be stable). |
| \`M\` | Matrix | Yes | \`B\` for controllability, \`C\` for observability. |
| \`type$\` | String | Yes | \`'c'\` (controllability) or \`'o'\` (observability). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`W\` | Matrix | The requested Gramian (symmetric). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NOT_STABLE\` | \`A\` has non-negative eigenvalues | The Gramian integral converges only for a stable \`A\`. |`,
  },
  {
    name: `impulse`,
    slug: `impulse`,
    category: `Control Systems`,
    summary: `Impulse response of a transfer function over a time vector.`,
    related: [`step`, `lsim`, `pole`],
    examples: [`step-impulse-response`],
    tags: [`control`, `impulse response`, `transient`, `time domain`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **impulse response** \`y(t)\` of \`G(s) = num/den\` sampled at the times in
\`t\` — the system output to a unit impulse input. It is the inverse Laplace
transform of \`G(s)\` itself and the kernel of the convolution that gives any
response.

## Syntax

\`\`\`
CALL impulse(num, den, t : y)
y = impulse(num, den, t)
\`\`\`

## Description

The impulse response characterizes the system's natural modes directly; it is also
the derivative of the step response.

## Mathematical Formulation

$$ y(t) = \\mathcal{L}^{-1}\\{G(s)\\}, \\qquad g(t) = \\frac{d}{dt}\\,y_{\\text{step}}(t) $$

> **Method:** numerical evaluation of the impulse response at each \`t\`.

## Examples

### Example 1 — Impulse response of a plant

[Run: step-impulse-response]

**Expected:** a decaying (and, for complex poles, oscillating) response returning to
zero, reflecting the open-loop poles.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |
| \`t\` | Vector | Yes | Time samples [s]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Vector | Impulse response at each time. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`IMPROPER_TF\` | \`num\` order exceeds \`den\` order | Provide a proper transfer function. |`,
  },
  {
    name: `lqe`,
    slug: `lqe`,
    category: `Control Systems`,
    summary: `Linear-quadratic (Kalman) estimator gain.`,
    related: [`lqr`, `gram`, `balreal`, `obsv`],
    examples: [`estimator-gramian-balreal`],
    tags: [`control`, `kalman`, `estimator`, `observer`, `lqe`, `riccati`],
    references: [],
    guides: [],
    body: `Returns the **optimal estimator (Kalman) gain** \`L\` for a linear system with process
noise (intensity via \`G\`, \`Q\`) and measurement noise (\`R\`). It is the dual of
\`lqr\`: the observer \`x̂̇ = Ax̂ + Bu + L(y − Cx̂)\` reconstructs the state from
noisy measurements with minimum error variance.

## Syntax

\`\`\`
CALL lqe(A, G, C, Q, R : L)
L = lqe(A, G, C, Q, R)
\`\`\`

## Description

\`Q\` is the process-noise covariance entering through \`G\`; \`R\` is the measurement-
noise covariance. The gain balances trust in the model against trust in the sensor.

## Mathematical Formulation

\`L = PCᵀR⁻¹\`, where \`P\` (the error covariance) solves the filter algebraic Riccati
equation (dual of the LQR ARE):

$$ A P + P A^\\top - P C^\\top R^{-1} C P + G Q G^\\top = 0 $$

> **Method:** solve the filter ARE for \`P\`, then \`L = PCᵀR⁻¹\`.

## Examples

### Example 1 — Estimator gain for a plant

[Run: estimator-gramian-balreal]

**Expected:** an observer gain \`L\` that places the estimator poles \`(A − LC)\` for
the chosen noise weights.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix. |
| \`G\` | Matrix | Yes | Process-noise input matrix. |
| \`C\` | Matrix | Yes | Output matrix. |
| \`Q\` | Matrix | Yes | Process-noise covariance (≥ 0). |
| \`R\` | Matrix | Yes | Measurement-noise covariance (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`L\` | Matrix | Optimal estimator gain. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NOT_DETECTABLE\` | \`(A, C)\` not detectable | The pair must be detectable for a solution to exist. |`,
  },
  {
    name: `lqr`,
    slug: `lqr`,
    category: `Control Systems`,
    summary: `Linear-quadratic regulator optimal state-feedback gain.`,
    related: [`lqe`, `place`, `dare`, `pidtune`],
    examples: [`controller-design-lqr-pid`],
    tags: [`control`, `lqr`, `optimal`, `state feedback`, `riccati`, `regulator`],
    references: [],
    guides: [`comp-linearize`, `symbolic-cas`],
    body: `Returns the **optimal state-feedback gain** \`K\` for a continuous linear-quadratic
regulator: the control \`u = −Kx\` that minimizes a weighted quadratic cost on the
states and the effort. Use it for systematic multi-state feedback design.

## Syntax

\`\`\`
CALL lqr(A, B, Q, R : K)
K = lqr(A, B, Q, R)
\`\`\`

## Description

\`Q\` (state weighting, ≥ 0) penalizes deviations; \`R\` (input weighting, > 0)
penalizes effort. Larger \`Q/R\` gives a faster, more aggressive regulator.

## Mathematical Formulation

Minimizing

$$ J = \\int_0^\\infty \\big(\\mathbf{x}^\\top Q\\,\\mathbf{x} + \\mathbf{u}^\\top R\\,\\mathbf{u}\\big)\\,dt $$

gives \`K = R⁻¹BᵀP\`, where \`P\` solves the continuous algebraic Riccati equation:

$$ A^\\top P + P A - P B R^{-1} B^\\top P + Q = 0 $$

> **Method:** solve the continuous ARE for \`P\`, then \`K = R⁻¹BᵀP\`.

## Examples

### Example 1 — LQR gain for a plant

[Run: controller-design-lqr-pid]

**Expected:** a gain \`K\` placing the closed-loop poles \`(A − BK)\` for the chosen
\`Q\`, \`R\` trade-off.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix. |
| \`B\` | Matrix | Yes | Input matrix. |
| \`Q\` | Matrix | Yes | State weighting (symmetric, ≥ 0). |
| \`R\` | Matrix | Yes | Input weighting (symmetric, > 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`K\` | Matrix | Optimal state-feedback gain (\`u = −Kx\`). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NOT_STABILIZABLE\` | \`(A, B)\` not stabilizable | The pair must be stabilizable for a solution to exist. |
| \`R_NOT_PD\` | \`R\` not positive-definite | Use a positive-definite input weighting. |`,
  },
  {
    name: `lsim`,
    slug: `lsim`,
    category: `Control Systems`,
    summary: `Response of a transfer function to an arbitrary input u(t).`,
    related: [`step`, `impulse`],
    examples: [`step-impulse-response`],
    tags: [`control`, `simulation`, `arbitrary input`, `convolution`, `time domain`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **time response** \`y(t)\` of \`G(s) = num/den\` to an **arbitrary input**
\`u(t)\` sampled on the time vector \`t\`. Use it to simulate a system under a custom
forcing (ramps, pulses, measured signals) rather than the canned step/impulse.

## Syntax

\`\`\`
CALL lsim(num, den, u, t : y)
y = lsim(num, den, u, t)
\`\`\`

## Description

\`u\` and \`t\` are aligned vectors describing the input over time; \`y\` is the
corresponding output. The response is the convolution of the input with the
system's impulse response.

## Mathematical Formulation

$$ y(t) = \\int_0^t g(t-\\tau)\\,u(\\tau)\\,d\\tau, \\qquad g(t) = \\mathcal{L}^{-1}\\{G(s)\\} $$

> **Method:** numerical convolution / state-space integration of the input over \`t\`.

## Examples

### Example 1 — Response to a custom input

[Run: step-impulse-response]

**Expected:** the output tracking the supplied input, shaped by the plant dynamics.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |
| \`u\` | Vector | Yes | Input samples aligned with \`t\`. |
| \`t\` | Vector | Yes | Time samples [s]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Vector | Output response at each time. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`LENGTH_MISMATCH\` | \`u\` and \`t\` differ in length | Provide input and time vectors of equal length. |`,
  },
  {
    name: `lyap`,
    slug: `lyap`,
    category: `Control Systems`,
    summary: `Solve the continuous Lyapunov equation A·X + X·Aᵀ + Q = 0.`,
    related: [`dlyap`, `dare`, `gram`],
    examples: [],
    tags: [`control`, `lyapunov`, `stability`, `gramian`, `riccati`],
    references: [],
    guides: [],
    body: `Solves the **continuous Lyapunov equation** for \`X\`. It underpins stability
analysis (a positive-definite \`X\` for \`Q > 0\` certifies stability of \`A\`) and the
controllability/observability Gramians.

## Syntax

\`\`\`
CALL lyap(A, Q : X)
X = lyap(A, Q)
\`\`\`

## Mathematical Formulation

$$ A X + X A^\\top + Q = 0 $$

For a Hurwitz \`A\` and \`Q = Qᵀ ⪰ 0\`, the unique solution is

$$ X = \\int_0^\\infty e^{A t} Q\\, e^{A^\\top t}\\,dt $$

> **Method:** Bartels–Stewart (Schur-based) solve of the linear Lyapunov system.

## Examples

\`\`\`
{ X = lyap(A, Q); X > 0 certifies A is stable when Q > 0 }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix (stable for a bounded solution). |
| \`Q\` | Matrix | Yes | Symmetric right-hand side. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`X\` | Matrix | Symmetric solution. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NO_UNIQUE_SOLUTION\` | \`A\` shares eigenvalues with \`−Aᵀ\` | The Lyapunov operator is singular; check \`A\`'s spectrum. |`,
  },
  {
    name: `margin`,
    slug: `margin`,
    category: `Control Systems`,
    summary: `Gain and phase margins and their crossover frequencies.`,
    related: [`bode`, `nyquist`, `pole`],
    examples: [`control-analysis-report`],
    tags: [`control`, `gain margin`, `phase margin`, `stability`, `crossover`, `frequency response`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **gain margin** \`gm\`, **phase margin** \`pm\`, and the gain- and
phase-crossover frequencies (\`w_cg\`, \`w_cp\`) of an open-loop transfer function —
the classical frequency-domain measures of relative stability for the closed loop.

## Syntax

\`\`\`
CALL margin(num, den : gm, pm, w_cg, w_cp)
[gm, pm, w_cg, w_cp] = margin(num, den)
\`\`\`

## Description

Applied to the open-loop \`L(s) = num/den\`, the margins quantify how much
additional gain or phase lag the loop tolerates before the closed loop goes
unstable. Positive \`gm\` (in dB) and positive \`pm\` (in degrees) indicate a stable
closed loop.

## Mathematical Formulation

At the **phase-crossover** frequency $\\omega_{cg}$ where $\\angle L(j\\omega_{cg}) = -180°$:

$$ GM = \\frac{1}{|L(j\\omega_{cg})|} \\quad\\text{(often in dB: } 20\\log_{10} GM\\text{)} $$

At the **gain-crossover** frequency $\\omega_{cp}$ where $|L(j\\omega_{cp})| = 1$:

$$ PM = 180° + \\angle L(j\\omega_{cp}) $$

> **Method:** locate the crossover frequencies on the open-loop frequency response,
> then evaluate the margins there.

## Examples

### Example 1 — Margins of a second-order plant

[Run: control-analysis-report]

**Expected:** with no \`−180°\` crossing, the gain margin is infinite and the phase
margin is large — consistent with the stable left-half-plane poles.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Open-loop numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Open-loop denominator coefficients (descending powers of \`s\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`gm\` | Number | Gain margin (factor; report as \`20·log10(gm)\` dB). |
| \`pm\` | Number | Phase margin [deg]. |
| \`w_cg\` | Number | Gain-margin (phase-crossover) frequency [rad/s]. |
| \`w_cp\` | Number | Phase-margin (gain-crossover) frequency [rad/s]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NO_CROSSOVER\` | The response never crosses \`−180°\` or \`0 dB\` | Margin is infinite/undefined for this loop — interpret accordingly. |`,
  },
  {
    name: `mason`,
    slug: `mason`,
    category: `Control Systems`,
    summary: `Overall gain of a signal-flow graph by Mason's gain formula.`,
    related: [`series`, `parallel`, `feedback`],
    examples: [],
    tags: [`control`, `mason`, `signal flow graph`, `gain formula`, `block diagram`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **overall transfer gain** \`T\` between a source and sink node of a
signal-flow graph, using **Mason's gain formula**. It reduces an arbitrary
interconnection — including multiple loops and forward paths — to a single
input-output gain.

## Syntax

\`\`\`
CALL mason(G, source, sink : T)
T = mason(G, source, sink)
\`\`\`

## Mathematical Formulation

Mason's rule:

$$ T = \\frac{\\sum_k P_k \\Delta_k}{\\Delta}, \\qquad \\Delta = 1 - \\sum L_i + \\sum L_iL_j - \\dots $$

where \`P_k\` are the forward-path gains, \`Δ\` is the graph determinant built from the
loop gains \`L_i\`, and \`Δ_k\` is \`Δ\` with the paths touching \`P_k\` removed.

> **Method:** enumerate forward paths and loops on the graph \`G\`, then apply the
> formula.

## Examples

\`\`\`
{ T = mason(G, source, sink) for a signal-flow graph G }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`G\` | Matrix | Yes | Signal-flow graph (branch-gain adjacency). |
| \`source\` | Number | Yes | Source node index. |
| \`sink\` | Number | Yes | Sink node index. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`T\` | Number | Overall source-to-sink gain. |`,
  },
  {
    name: `nichols`,
    slug: `nichols`,
    category: `Control Systems`,
    summary: `Nichols frequency response — open-loop gain (dB) versus phase (deg).`,
    related: [`bode`, `nyquist`, `margin`],
    examples: [`nichols-chart`],
    tags: [`control`, `nichols`, `frequency response`, `gain`, `phase`, `stability`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **Nichols frequency response** of \`G(s) = num/den\` over \`omega\`:
open-loop magnitude (dB) and phase (deg). Plotted as magnitude-versus-phase on the
Nichols chart, it reads off closed-loop gain and stability margins in one view.

## Syntax

\`\`\`
CALL nichols(num, den, omega : mag, phase)
[mag, phase] = nichols(num, den, omega)
\`\`\`

## Description

The Nichols chart overlays loci of constant closed-loop magnitude and phase on the
open-loop gain-phase plane, so the closed-loop peak (and hence damping) is read
directly from where the open-loop curve grazes them.

## Mathematical Formulation

$$ \\text{mag}(\\omega) = 20\\log_{10}|G(j\\omega)|\\ [\\text{dB}], \\qquad \\text{phase}(\\omega) = \\angle G(j\\omega)\\ [\\text{deg}] $$

plotted as \`mag\` vs \`phase\`.

> **Method:** evaluate \`G(jω)\` at each \`omega\`; return magnitude in dB and phase in
> degrees for the gain-phase plane.

## Examples

### Example 1 — Nichols response of a plant

[Run: nichols-chart]

**Expected:** a gain-phase locus whose proximity to the \`0 dB, −180°\` point reflects
the gain and phase margins.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |
| \`omega\` | Vector | Yes | Frequencies [rad/s]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`mag\` | Vector | Magnitude [dB]. |
| \`phase\` | Vector | Phase [deg]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`EMPTY_FREQUENCY\` | \`omega\` empty | Provide a frequency vector. |`,
  },
  {
    name: `nyquist`,
    slug: `nyquist`,
    category: `Control Systems`,
    summary: `Nyquist frequency response — real and imaginary parts of G(jω).`,
    related: [`bode`, `margin`],
    examples: [`control-analysis-report`],
    tags: [`control`, `nyquist`, `frequency response`, `stability`, `polar plot`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **Nyquist (polar) frequency response** of \`G(s) = num/den\` over a
frequency vector \`omega\`: the real (\`re\`) and imaginary (\`im\`) parts of \`G(jω)\`.
Plotting \`im\` against \`re\` and applying the Nyquist criterion (encirclements of the
\`−1 + j0\` point) tests closed-loop stability.

## Syntax

\`\`\`
CALL nyquist(num, den, omega : re, im)
[re, im] = nyquist(num, den, omega)
\`\`\`

## Description

The Nyquist locus traces \`G(jω)\` in the complex plane as \`ω\` sweeps. Its proximity
to the critical point \`−1 + j0\` is the geometric basis of the gain and phase
margins.

## Mathematical Formulation

$$ G(j\\omega) = \\mathrm{re}(\\omega) + j\\,\\mathrm{im}(\\omega), \\qquad \\mathrm{re} = \\Re\\{G(j\\omega)\\},\\ \\ \\mathrm{im} = \\Im\\{G(j\\omega)\\} $$

The Nyquist stability criterion relates closed-loop right-half-plane poles \`Z\` to
encirclements \`N\` of \`−1\` and open-loop RHP poles \`P\` by \`Z = N + P\`.

> **Method:** evaluate \`G(jω)\` at each \`omega\`; return Cartesian parts.

## Examples

### Example 1 — Nyquist locus of a second-order plant

[Run: control-analysis-report]

**Expected:** a locus that stays clear of \`−1 + j0\` (no encirclements), consistent
with the stable closed loop.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |
| \`omega\` | Vector | Yes | Frequencies [rad/s]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`re\` | Vector | Real part of \`G(jω)\` at each frequency. |
| \`im\` | Vector | Imaginary part of \`G(jω)\` at each frequency. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`EMPTY_FREQUENCY\` | \`omega\` is empty | Provide a frequency vector spanning the dynamics of interest. |`,
  },
  {
    name: `obsv`,
    slug: `obsv`,
    category: `Control Systems`,
    summary: `Observability matrix of a state-space pair (A, C).`,
    related: [`ctrb`, `lqe`, `gram`],
    examples: [],
    tags: [`control`, `observability`, `state space`, `observer`, `rank`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **observability matrix** \`Ob\` of the pair \`(A, C)\`. The system is
observable — its full state can be reconstructed from the output, so an observer of
arbitrary speed exists — iff \`Ob\` has full rank.

## Syntax

\`\`\`
CALL obsv(A, C : Ob)
Ob = obsv(A, C)
\`\`\`

## Mathematical Formulation

For an \`n\`-state system, the dual of \`ctrb\`:

$$ \\mathcal{O} = \\begin{bmatrix} C \\\\ CA \\\\ CA^2 \\\\ \\vdots \\\\ CA^{n-1} \\end{bmatrix} $$

The pair is observable iff \`rank(O) = n\`.

> **Method:** stack \`[C; CA; …; CAⁿ⁻¹]\`.

## Examples

\`\`\`
{ Ob = obsv(A, C); observable iff rank(Ob) = n }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix (n×n). |
| \`C\` | Matrix | Yes | Output matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Ob\` | Matrix | Observability matrix. |`,
  },
  {
    name: `pade`,
    slug: `pade`,
    category: `Control Systems`,
    summary: `Padé rational approximation of a pure time delay.`,
    related: [`tf`, `series`, `feedback`],
    examples: [],
    tags: [`control`, `pade`, `time delay`, `dead time`, `approximation`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns a **Padé rational approximation** \`num/den\` of a pure time delay
\`e^{−Td·s}\` of the given order. It replaces the transcendental delay with a rational
transfer function so the loop can be analyzed and designed with standard tools.

## Syntax

\`\`\`
CALL pade(Td, order : num, den)
[num, den] = pade(Td, order)
\`\`\`

## Mathematical Formulation

The order-\`n\` Padé approximant of the delay:

$$ e^{-T_d s} \\approx \\frac{N_n(-T_d s)}{N_n(T_d s)}, \\qquad \\text{e.g. (n=1): } \\frac{1 - T_d s/2}{1 + T_d s/2} $$

with the right-half-plane zeros that give a delay its characteristic phase lag.

> **Method:** form the order-\`n\` Padé numerator/denominator polynomials in \`Td·s\`.

## Examples

\`\`\`
{ [num, den] = pade(0.2, 2) approximates a 0.2 s delay to 2nd order }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Td\` | Number | Yes | Time delay [s]. |
| \`order\` | Number | Yes | Approximation order (≥ 1). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`num\` | Vector | Numerator of the approximant. |
| \`den\` | Vector | Denominator of the approximant. |`,
  },
  {
    name: `parallel`,
    slug: `parallel`,
    category: `Control Systems`,
    summary: `Parallel connection of two transfer functions, G = G1 + G2.`,
    related: [`series`, `feedback`],
    examples: [],
    tags: [`control`, `parallel`, `block diagram`, `transfer function`, `sum`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **parallel connection** of two transfer functions —
\`G(s) = G1(s) + G2(s)\` — as a single \`num/den\` pair. It models two blocks fed the
same input whose outputs are summed.

## Syntax

\`\`\`
CALL parallel(num1, den1, num2, den2 : num, den)
[num, den] = parallel(num1, den1, num2, den2)
\`\`\`

## Mathematical Formulation

$$ G(s) = G_1(s) + G_2(s) = \\frac{\\text{num}_1\\,\\text{den}_2 + \\text{num}_2\\,\\text{den}_1}{\\text{den}_1\\,\\text{den}_2} $$

> **Method:** common-denominator polynomial addition.

## Examples

\`\`\`
{ [num, den] = parallel(num1, den1, num2, den2) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num1\`, \`den1\` | Vector | Yes | First transfer function \`G1\`. |
| \`num2\`, \`den2\` | Vector | Yes | Second transfer function \`G2\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`num\` | Vector | Numerator of \`G1 + G2\`. |
| \`den\` | Vector | Denominator of \`G1 + G2\`. |`,
  },
  {
    name: `pidtune`,
    slug: `pidtune`,
    category: `Control Systems`,
    summary: `Automatic PID gain tuning by loop-shaping to a target crossover.`,
    related: [`margin`, `feedback`, `lqr`],
    examples: [`controller-design-lqr-pid`],
    tags: [`control`, `pid`, `tuning`, `loop shaping`, `crossover`, `kp ki kd`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns tuned **PID gains** \`Kp\`, \`Ki\`, \`Kd\` for a plant \`G(s) = num/den\`, designed
to achieve a target gain-crossover frequency \`wc\` with adequate phase margin. Use it
for a quick, systematic classical controller without manual loop shaping.

## Syntax

\`\`\`
CALL pidtune(num, den, 'PID', wc : Kp, Ki, Kd)
[Kp, Ki, Kd] = pidtune(num, den, 'PID', wc)
\`\`\`

## Description

The controller \`C(s) = Kp + Ki/s + Kd·s\` is shaped so the open loop \`C·G\` crosses
0 dB near \`wc\` with a phase margin that yields a well-damped closed loop. The type
string selects \`'P'\`, \`'PI'\`, \`'PD'\`, or \`'PID'\`.

## Mathematical Formulation

$$ C(s) = K_p + \\frac{K_i}{s} + K_d\\,s $$

The gains are chosen so that at the target crossover \`ωc\`:

$$ |C(j\\omega_c)G(j\\omega_c)| = 1, \\qquad \\angle C(j\\omega_c)G(j\\omega_c) = -180° + \\text{PM} $$

> **Method:** solve the magnitude/phase loop-shaping conditions at \`wc\` for the
> controller gains.

## Examples

### Example 1 — Tune a PID controller

[Run: controller-design-lqr-pid]

**Expected:** \`Kp\`, \`Ki\`, \`Kd\` giving a stable closed loop with the targeted
crossover and margin.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Plant numerator (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Plant denominator. |
| \`type$\` | String | Yes | \`'P'\`, \`'PI'\`, \`'PD'\`, or \`'PID'\`. |
| \`wc\` | Number | Yes | Target gain-crossover frequency [rad/s]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Kp\` | Number | Proportional gain. |
| \`Ki\` | Number | Integral gain. |
| \`Kd\` | Number | Derivative gain. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`INFEASIBLE_WC\` | target crossover unreachable | Choose a \`wc\` consistent with the plant bandwidth. |`,
  },
  {
    name: `place`,
    slug: `place`,
    category: `Control Systems`,
    summary: `State-feedback pole placement to a set of desired closed-loop poles.`,
    related: [`acker`, `lqr`, `ctrb`],
    examples: [],
    tags: [`control`, `pole placement`, `state feedback`, `ackermann`, `design`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **state-feedback gain** \`K\` that moves the closed-loop poles of
\`(A, B)\` to the desired locations given by their real/imaginary parts (\`pr\`, \`pi\`).
The control law \`u = −Kx\` makes \`eig(A − BK)\` equal the requested poles.

## Syntax

\`\`\`
CALL place(A, B, pr, pi : K)
K = place(A, B, pr, pi)
\`\`\`

## Description

The pair \`(A, B)\` must be controllable for arbitrary pole placement. Provide the
desired poles as conjugate pairs in \`pr ± j·pi\`.

## Mathematical Formulation

Find \`K\` such that

$$ \\det\\!\\big(sI - (A - BK)\\big) = \\prod_i (s - p_i) $$

with the desired characteristic polynomial set by \`{p_i}\`.

> **Method:** solve for \`K\` from the desired characteristic polynomial (Ackermann /
> robust placement); see \`acker\` for the single-input Ackermann form.

## Examples

\`\`\`
{ place the regulator poles of a controllable (A,B) at -2 +/- j }
{ K = place(A, B, [-2,-2], [1,-1]) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix. |
| \`B\` | Matrix | Yes | Input matrix. |
| \`pr\` | Vector | Yes | Real parts of the desired poles. |
| \`pi\` | Vector | Yes | Imaginary parts of the desired poles. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`K\` | Matrix | State-feedback gain (\`u = −Kx\`). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NOT_CONTROLLABLE\` | \`(A, B)\` not controllable | Arbitrary placement needs a controllable pair (check \`ctrb\`). |`,
  },
  {
    name: `pole`,
    slug: `pole`,
    category: `Control Systems`,
    summary: `Poles of a transfer function (roots of the denominator).`,
    related: [`zero`, `margin`, `residue`],
    examples: [`control-analysis-report`],
    tags: [`control`, `poles`, `stability`, `transfer function`, `eigenvalues`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **poles** of a transfer function \`G(s) = num(s)/den(s)\` — the roots of
its denominator — split into real (\`pr\`) and imaginary (\`pi\`) parts. The poles set
the natural modes and stability: a continuous-time system is stable iff every pole
has a negative real part.

## Syntax

\`\`\`
CALL pole(num, den : pr, pi)
[pr, pi] = pole(num, den)
\`\`\`

## Description

\`num\` and \`den\` are coefficient vectors in descending powers of \`s\`. The poles
govern the transient response (decay rates and oscillation frequencies); their
location in the s-plane is the primary stability indicator.

## Mathematical Formulation

The poles are the roots of the characteristic (denominator) polynomial,

$$ \\text{den}(s) = 0 \\quad\\Longrightarrow\\quad s = p_k = \\sigma_k \\pm j\\omega_k $$

A complex pair $\\sigma \\pm j\\omega$ corresponds to natural frequency
$\\omega_n = \\sqrt{\\sigma^2 + \\omega^2}$ and damping ratio $\\zeta = -\\sigma/\\omega_n$.

> **Method:** numerical polynomial root-finding on \`den\`.

## Examples

### Example 1 — Poles of an underdamped second-order plant

For \`G(s) = (s + 2)/(s² + 4s + 25)\`:

[Run: control-analysis-report]

**Expected:** poles \`s = −2 ± 4.583j\` (\`pr = −2\`, \`pi = ±4.583\`); both in the
left half-plane, so the plant is stable (\`ω_n = 5 rad/s\`, \`ζ = 0.4\`).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`pr\` | Vector | Real parts of the poles. |
| \`pi\` | Vector | Imaginary parts of the poles. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`EMPTY_DENOMINATOR\` | \`den\` has no nonzero leading coefficient | Provide a valid denominator polynomial. |`,
  },
  {
    name: `residue`,
    slug: `residue`,
    category: `Control Systems`,
    summary: `Partial-fraction residues and poles of a transfer function.`,
    related: [`pole`, `tf`, `zero`],
    examples: [`inverse-laplace-residue`],
    tags: [`control`, `partial fraction`, `residue`, `poles`, `inverse laplace`],
    references: [],
    guides: [`repl`, `symbolic-cas`],
    body: `Returns the **partial-fraction expansion** of \`G(s) = num/den\`: the residues
(\`rr\`/\`ri\`, real/imaginary parts), the poles (\`pr\`/\`pi\`), and the direct term \`k\`.
It is the basis for analytic inverse-Laplace transforms — each pole/residue pair
maps to a time-domain mode.

## Syntax

\`\`\`
CALL residue(num, den : rr, ri, pr, pi, k)
[rr, ri, pr, pi, k] = residue(num, den)
\`\`\`

## Description

The expansion decomposes a rational function into a sum of simple terms over its
poles, so the time response is read off as a sum of exponentials/sinusoids.

## Mathematical Formulation

$$ G(s) = \\frac{\\text{num}(s)}{\\text{den}(s)} = \\sum_{i} \\frac{r_i}{s - p_i} + k(s) $$

with the residue at a simple pole \`p_i\` given by:

$$ r_i = \\big[(s - p_i)\\,G(s)\\big]_{s = p_i} $$

> **Method:** factor \`den\` for the poles, then evaluate the residues (and any
> polynomial direct term \`k\` when \`num\` and \`den\` are equal order).

## Examples

### Example 1 — Residues for an inverse Laplace transform

[Run: inverse-laplace-residue]

**Expected:** residue/pole pairs that reconstruct the time response as a sum of
modal terms.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`rr\`, \`ri\` | Vector | Real / imaginary parts of the residues. |
| \`pr\`, \`pi\` | Vector | Real / imaginary parts of the poles. |
| \`k\` | Vector | Direct (polynomial) term, empty if \`num\` order < \`den\` order. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`REPEATED_POLE\` | high-multiplicity poles | Repeated poles need the extended residue form; check the result. |`,
  },
  {
    name: `rlocus`,
    slug: `rlocus`,
    category: `Control Systems`,
    summary: `Root-locus trajectories of the closed-loop poles as gain K varies.`,
    related: [`pole`, `margin`, `place`],
    examples: [`root-locus-analysis`],
    tags: [`control`, `root locus`, `poles`, `gain`, `design`, `stability`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **root-locus** of \`G(s) = num/den\` — the paths the closed-loop poles
trace in the s-plane as the loop gain \`K\` sweeps from 0 to ∞. Use it to choose a
gain that places the dominant poles for a target damping or settling time.

## Syntax

\`\`\`
CALL rlocus(num, den : K, cpr, cpi)
[K, cpr, cpi] = rlocus(num, den)
\`\`\`

## Description

For unity feedback \`1 + K·G(s) = 0\`, the roots move from the open-loop poles
(\`K = 0\`) toward the open-loop zeros and asymptotes (\`K → ∞\`). \`cpr\`/\`cpi\` are the
real/imaginary parts of the closed-loop poles at each gain \`K\`.

## Mathematical Formulation

The locus is the set of \`s\` satisfying the characteristic equation

$$ 1 + K\\,G(s) = 0 \\quad\\Longleftrightarrow\\quad \\angle G(s) = \\pm 180°(2\\ell+1) $$

with the gain at any locus point \`K = 1/|G(s)|\`.

> **Method:** sweep \`K\`, solving the characteristic polynomial roots at each value.

## Examples

### Example 1 — Root locus of a plant

[Run: root-locus-analysis]

**Expected:** branches leaving the open-loop poles and ending on the zeros /
asymptotes; crossings of the imaginary axis mark the stability-limiting gain.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Open-loop numerator (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Open-loop denominator. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`K\` | Vector | Gain values along the locus. |
| \`cpr\` | Vector/Matrix | Real parts of the closed-loop poles. |
| \`cpi\` | Vector/Matrix | Imaginary parts of the closed-loop poles. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`EMPTY_DENOMINATOR\` | invalid \`den\` | Provide a valid open-loop denominator. |`,
  },
  {
    name: `routh`,
    slug: `routh`,
    category: `Control Systems`,
    summary: `Routh-Hurwitz stability test — count of right-half-plane roots.`,
    related: [`pole`, `margin`, `rlocus`],
    examples: [`routh-stability`],
    tags: [`control`, `routh`, `hurwitz`, `stability`, `characteristic polynomial`],
    references: [],
    guides: [`repl`, `symbolic-cas`],
    body: `Applies the **Routh-Hurwitz criterion** to a characteristic polynomial \`den(s)\` and
returns the number of right-half-plane roots \`nRHP\` and a stability flag \`stable\`.
It decides stability without computing the roots — useful for symbolic gain ranges.

## Syntax

\`\`\`
CALL routh(den : nRHP, stable)
[nRHP, stable] = routh(den)
\`\`\`

## Description

The Routh array is built from the polynomial coefficients; the number of sign
changes in its first column equals the number of poles in the right half-plane. A
system is stable iff there are none.

## Mathematical Formulation

For \`den(s) = a_n s^n + … + a_0\`, the Routh array's first-column sign changes count
the RHP roots; stability requires:

$$ \\text{all first-column entries} > 0 \\quad\\Longleftrightarrow\\quad n_{RHP} = 0 $$

> **Method:** construct the Routh array (handling zero-pivot and zero-row special
> cases) and count first-column sign changes.

## Examples

### Example 1 — Stability of a characteristic polynomial

[Run: routh-stability]

**Expected:** \`nRHP\` right-half-plane roots and \`stable = 1\` only when \`nRHP = 0\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`den\` | Vector | Yes | Characteristic-polynomial coefficients (descending powers of \`s\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`nRHP\` | Number | Count of right-half-plane roots. |
| \`stable\` | Number | 1 if stable (\`nRHP = 0\`), else 0. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`EMPTY_DENOMINATOR\` | invalid \`den\` | Provide a valid characteristic polynomial. |`,
  },
  {
    name: `series`,
    slug: `series`,
    category: `Control Systems`,
    summary: `Cascade (series) connection of two transfer functions, G = G1·G2.`,
    related: [`feedback`, `ss2tf`],
    examples: [`cruise-control`],
    tags: [`control`, `series`, `cascade`, `block diagram`, `transfer function`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **series (cascade) connection** of two transfer functions —
\`G(s) = G1(s)·G2(s)\` — as a single \`num/den\` pair. Use it to build an open-loop
\`L(s) = C(s)·G(s)\` from a controller and a plant.

## Syntax

\`\`\`
CALL series(num1, den1, num2, den2 : num, den)
[num, den] = series(num1, den1, num2, den2)
\`\`\`

## Description

Two blocks in cascade multiply: the combined numerator and denominator are the
polynomial products (convolutions) of the individual ones.

## Mathematical Formulation

$$ G(s) = G_1(s)\\,G_2(s) = \\frac{\\text{num}_1 \\ast \\text{num}_2}{\\text{den}_1 \\ast \\text{den}_2} $$

where $\\ast$ is polynomial multiplication (coefficient convolution).

> **Method:** convolve the numerator and denominator coefficient vectors.

## Examples

### Example 1 — Open-loop cruise-control system

Cascade the PI controller \`C(s) = (Kp·s + Ki)/s\` with the car plant \`G(s)\` to form
the open-loop \`L(s)\`:

[Run: cruise-control]

**Expected:** \`L(s) = C(s)·G(s)\` with the controller zero and integrator combined
with the first-order plant pole.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num1\`, \`den1\` | Vector | Yes | First transfer function \`G1\`. |
| \`num2\`, \`den2\` | Vector | Yes | Second transfer function \`G2\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`num\` | Vector | Numerator of \`G1·G2\`. |
| \`den\` | Vector | Denominator of \`G1·G2\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`EMPTY_POLYNOMIAL\` | A \`num\`/\`den\` vector is empty | Provide valid coefficient vectors for both blocks. |`,
  },
  {
    name: `ss`,
    slug: `ss`,
    category: `Control Systems`,
    summary: `Create a state-space model from (A, B, C, D).`,
    related: [`ss2tf`, `tf2ss`, `ss2ss`],
    examples: [],
    tags: [`control`, `state space`, `model`, `ss`],
    references: [],
    guides: [],
    body: `Builds a **state-space model** from the matrices \`(A, B, C, D)\` — the time-domain
representation \`ẋ = Ax + Bu\`, \`y = Cx + Du\` on which modern (state-feedback,
observer, LQR/LQE) design operates.

## Syntax

\`\`\`
sys = ss(A, B, C, D)
\`\`\`

## Mathematical Formulation

$$ \\dot{\\mathbf{x}} = A\\mathbf{x} + B\\mathbf{u}, \\qquad \\mathbf{y} = C\\mathbf{x} + D\\mathbf{u} $$

with transfer function \`G(s) = C(sI − A)⁻¹B + D\` (see \`ss2tf\`).

> **Method:** stores the \`(A, B, C, D)\` quadruple as a state-space model.

## Examples

\`\`\`
{ sys = ss(A, B, C, D) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | State matrix. |
| \`B\` | Matrix | Yes | Input matrix. |
| \`C\` | Matrix | Yes | Output matrix. |
| \`D\` | Number/Matrix | Yes | Feedthrough. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`sys\` | State-space | The model \`(A, B, C, D)\`. |`,
  },
  {
    name: `ss2ss`,
    slug: `ss2ss`,
    category: `Control Systems`,
    summary: `Similarity transform of a state-space model, x = P·z.`,
    related: [`ss`, `balreal`, `ss2tf`],
    examples: [],
    tags: [`control`, `similarity transform`, `state space`, `coordinate change`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Applies a **similarity (coordinate) transform** \`x = P·z\` to a state-space model,
returning the equivalent realization \`(An, Bn, Cn, Dn)\`. The input-output behavior
is unchanged; only the state coordinates differ — used to reach canonical or
balanced forms.

## Syntax

\`\`\`
CALL ss2ss(A, B, C, D, P : An, Bn, Cn, Dn)
[An, Bn, Cn, Dn] = ss2ss(A, B, C, D, P)
\`\`\`

## Mathematical Formulation

With \`x = P·z\`:

$$ A_n = P^{-1}AP, \\quad B_n = P^{-1}B, \\quad C_n = CP, \\quad D_n = D $$

The transfer function \`C(sI−A)⁻¹B + D\` is invariant under the transform.

> **Method:** apply the change of basis \`P\` to the quadruple.

## Examples

\`\`\`
{ [An,Bn,Cn,Dn] = ss2ss(A,B,C,D,P) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\`, \`B\`, \`C\`, \`D\` | Matrix | Yes | Original realization. |
| \`P\` | Matrix | Yes | Invertible transform (\`x = P·z\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`An\`, \`Bn\`, \`Cn\`, \`Dn\` | Matrix | Transformed realization. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`SINGULAR_TRANSFORM\` | \`P\` not invertible | Use a nonsingular transform matrix. |`,
  },
  {
    name: `ss2tf`,
    slug: `ss2tf`,
    category: `Control Systems`,
    summary: `Convert a state-space model (A, B, C, D) to a transfer function.`,
    related: [`tf2ss`, `series`, `feedback`],
    examples: [`cruise-control`],
    tags: [`control`, `state space`, `transfer function`, `ss2tf`, `model conversion`],
    references: [],
    guides: [`comp-linearize`, `symbolic-cas`],
    body: `Converts a single-input single-output **state-space model** \`(A, B, C, D)\` into a
**transfer function** \`G(s) = num(s)/den(s)\`. Use it to move from a physically-built
state model to the frequency-domain form needed for classical loop design.

## Syntax

\`\`\`
CALL ss2tf(A, B, C, D : num, den)
[num, den] = ss2tf(A, B, C, D)
\`\`\`

## Description

\`A\` is the system matrix, \`B\` the input matrix, \`C\` the output matrix, and \`D\` the
feedthrough. The result is the rational transfer function relating the single input
to the single output.

## Mathematical Formulation

$$ G(s) = C\\,(sI - A)^{-1} B + D = \\frac{\\text{num}(s)}{\\text{den}(s)} $$

The denominator is the characteristic polynomial \`den(s) = det(sI − A)\`.

> **Method:** form \`det(sI − A)\` for \`den\` and the adjugate product for \`num\`.

## Examples

### Example 1 — Car velocity model

A 1000 kg car with viscous drag, output = velocity, converted to a transfer
function for PI cruise-control design.

[Run: cruise-control]

**Expected:** \`G(s) = (1/m) / (s + c_drag/m) = 0.001/(s + 0.05)\` — a first-order
velocity-from-force plant.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | System (state) matrix. |
| \`B\` | Vector | Yes | Input matrix/column. |
| \`C\` | Vector | Yes | Output matrix/row. |
| \`D\` | Number | Yes | Direct feedthrough term. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`num\` | Vector | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Denominator coefficients (descending powers of \`s\`). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DIMENSION_MISMATCH\` | \`A\`, \`B\`, \`C\`, \`D\` shapes inconsistent | \`A\` is n×n, \`B\` is n×1, \`C\` is 1×n, \`D\` is scalar (SISO). |`,
  },
  {
    name: `step`,
    slug: `step`,
    category: `Control Systems`,
    summary: `Unit step response of a transfer function over a time vector.`,
    related: [`impulse`, `lsim`, `pole`],
    examples: [`control-analysis-report`],
    tags: [`control`, `step response`, `transient`, `overshoot`, `time domain`],
    references: [],
    guides: [`math-funcs`, `symbolic-cas`, `tut-msd`],
    body: `Returns the **unit step response** \`y(t)\` of \`G(s) = num/den\` sampled at the times
in \`t\`. Use it to read transient metrics — rise time, peak overshoot, settling
time — directly from the time-domain response.

## Syntax

\`\`\`
CALL step(num, den, t : y)
y = step(num, den, t)
\`\`\`

## Description

The step response is the inverse Laplace transform of \`G(s)/s\` (a unit step input
\`U(s) = 1/s\`). For an underdamped second-order system it exhibits the familiar
overshoot-and-ring governed by the damping ratio and natural frequency.

## Mathematical Formulation

$$ Y(s) = G(s)\\cdot\\frac{1}{s}, \\qquad y(t) = \\mathcal{L}^{-1}\\{Y(s)\\} $$

For a standard second-order system $G = \\omega_n^2/(s^2 + 2\\zeta\\omega_n s + \\omega_n^2)$,
the peak overshoot is $M_p = \\exp\\!\\big(-\\pi\\zeta/\\sqrt{1-\\zeta^2}\\big)$.

> **Method:** numerical evaluation of the step response at each \`t\`.

## Examples

### Example 1 — Step response of an underdamped plant

\`G(s) = (s + 2)/(s² + 4s + 25)\`, integrated over 4 s:

[Run: control-analysis-report]

**Expected:** an overshooting, ringing response (ζ ≈ 0.4) that settles toward its
steady-state value within a few seconds.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |
| \`t\` | Vector | Yes | Time samples [s]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Vector | Step response \`y(t)\` at each time. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`IMPROPER_TF\` | \`num\` higher order than \`den\` | The transfer function must be proper (order of \`num\` ≤ order of \`den\`). |`,
  },
  {
    name: `stepinfo`,
    slug: `stepinfo`,
    category: `Control Systems`,
    summary: `Step-response performance metrics (rise time, peak time, settling time, overshoot).`,
    related: [`step`, `pole`, `margin`],
    examples: [],
    tags: [`control`, `step response`, `rise time`, `settling time`, `overshoot`, `transient`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **transient performance metrics** of a step response sampled as
\`(t, y)\`: rise time \`Tr\`, peak time \`Tp\`, settling time \`Ts\`, and percent overshoot
\`OS\`. Use it to quantify a closed-loop design against time-domain specifications.

## Syntax

\`\`\`
CALL stepinfo(t, y : Tr, Tp, Ts, OS)
[Tr, Tp, Ts, OS] = stepinfo(t, y)
\`\`\`

## Mathematical Formulation

From the response \`y(t)\` with steady-state value \`y_∞\` and peak \`y_p\`:

$$ OS = \\frac{y_p - y_\\infty}{y_\\infty}\\times 100\\%, \\qquad T_p = \\arg\\max_t y(t) $$

\`Tr\` is the 10–90% rise time and \`Ts\` the time after which \`|y − y_∞|\` stays within
a 2% band.

> **Method:** scan the response for the crossing, peak, and settling instants.

## Examples

\`\`\`
{ [Tr, Tp, Ts, OS] = stepinfo(t, y) from a step() response }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`t\` | Vector | Yes | Time samples [s]. |
| \`y\` | Vector | Yes | Step response aligned with \`t\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Tr\` | Number | Rise time (10–90%) [s]. |
| \`Tp\` | Number | Peak time [s]. |
| \`Ts\` | Number | Settling time (2% band) [s]. |
| \`OS\` | Number | Percent overshoot. |`,
  },
  {
    name: `tf`,
    slug: `tf`,
    category: `Control Systems`,
    summary: `Create a transfer function from numerator and denominator coefficients.`,
    related: [`tf2ss`, `ss2tf`, `pole`, `zero`],
    examples: [`partial-fractions`],
    tags: [`control`, `transfer function`, `tf`, `model`, `laplace`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Builds a **transfer function** \`G(s) = num(s)/den(s)\` from coefficient vectors in
descending powers of \`s\`. It is the basic linear-systems model the analysis
(\`pole\`, \`bode\`, \`step\`) and design (\`feedback\`, \`lqr\`, \`pidtune\`) routines act on.

## Syntax

\`\`\`
G = tf(num, den)
\`\`\`

## Description

\`num\` and \`den\` list the polynomial coefficients from the highest power of \`s\` down
to the constant term. The result is the Laplace-domain ratio of output to input for
a SISO linear system.

## Mathematical Formulation

$$ G(s) = \\frac{\\text{num}(s)}{\\text{den}(s)} = \\frac{b_m s^m + \\dots + b_0}{a_n s^n + \\dots + a_0} $$

with \`m ≤ n\` for a proper system.

> **Method:** stores the coefficient pair as a transfer-function model.

## Examples

### Example 1 — Transfer function for a partial-fraction expansion

[Run: partial-fractions]

**Expected:** the rational \`G(s)\` whose residues and poles the expansion resolves.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`G\` | Transfer function | The model \`num/den\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`IMPROPER_TF\` | \`num\` order exceeds \`den\` order | Provide a proper transfer function (\`m ≤ n\`). |`,
  },
  {
    name: `tf2ss`,
    slug: `tf2ss`,
    category: `Control Systems`,
    summary: `Convert a transfer function to a state-space realization (A, B, C, D).`,
    related: [`ss2tf`, `tf`, `pole`],
    examples: [`multi-output-destructuring`],
    tags: [`control`, `transfer function`, `state space`, `tf2ss`, `realization`, `controllable canonical`],
    references: [],
    guides: [`functions`, `symbolic-cas`],
    body: `Converts a transfer function \`G(s) = num/den\` into a **state-space realization**
\`(A, B, C, D)\` — the inverse of \`ss2tf\`. Use it to move from a frequency-
domain model to the state form needed for modern (state-feedback, observer,
LQR/LQE) design.

## Syntax

\`\`\`
CALL tf2ss(num, den : A, B, C, D)
[A, B, C, D] = tf2ss(num, den)
\`\`\`

## Description

The realization returned is the controllable canonical form, one valid choice among
the infinitely many state-space models sharing the same input-output behavior.

## Mathematical Formulation

For \`G(s) = num/den\`, the controllable canonical realization places the denominator
coefficients in the companion \`A\` and the numerator in \`C\`:

$$ \\dot{\\mathbf{x}} = A\\mathbf{x} + B u, \\qquad y = C\\mathbf{x} + D u, \\qquad C(sI-A)^{-1}B + D = G(s) $$

> **Method:** build the controllable canonical companion form from the coefficients.

## Examples

### Example 1 — Realize a transfer function

[Run: multi-output-destructuring]

**Expected:** an \`(A, B, C, D)\` set whose transfer function recovers the original
\`num/den\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`A\` | Matrix | State matrix (companion form). |
| \`B\` | Vector | Input matrix. |
| \`C\` | Vector | Output matrix. |
| \`D\` | Number | Direct feedthrough. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`IMPROPER_TF\` | \`num\` order exceeds \`den\` order | Provide a proper transfer function. |`,
  },
  {
    name: `tf2zp`,
    slug: `tf2zp`,
    category: `Control Systems`,
    summary: `Transfer function to zero-pole-gain form.`,
    related: [`zp2tf`, `pole`, `zero`, `tf`],
    examples: [],
    tags: [`control`, `zero pole gain`, `zpk`, `transfer function`, `factorization`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Converts a transfer function \`G(s) = num/den\` to **zero-pole-gain** form: the zeros
(\`zr\`/\`zi\`), poles (\`pr\`/\`pi\`), and scalar gain \`k\`. It is the factored view of the
rational system, the inverse of \`zp2tf\`.

## Syntax

\`\`\`
CALL tf2zp(num, den : zr, zi, pr, pi, k)
[zr, zi, pr, pi, k] = tf2zp(num, den)
\`\`\`

## Mathematical Formulation

$$ G(s) = k\\,\\frac{\\prod_i (s - z_i)}{\\prod_j (s - p_j)} $$

where the zeros are the roots of \`num\`, the poles the roots of \`den\`, and \`k\` the
leading-coefficient ratio.

> **Method:** factor \`num\` and \`den\` (root-finding) and extract the gain.

## Examples

\`\`\`
{ [zr,zi,pr,pi,k] = tf2zp(num, den) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`zr\`, \`zi\` | Vector | Real / imaginary parts of the zeros. |
| \`pr\`, \`pi\` | Vector | Real / imaginary parts of the poles. |
| \`k\` | Number | Scalar gain. |`,
  },
  {
    name: `zero`,
    slug: `zero`,
    category: `Control Systems`,
    summary: `Zeros of a transfer function (roots of the numerator).`,
    related: [`pole`, `margin`],
    examples: [`control-analysis-report`],
    tags: [`control`, `zeros`, `transfer function`, `root locus`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Returns the **zeros** of a transfer function \`G(s) = num(s)/den(s)\` — the roots of
its numerator — split into real (\`zr\`) and imaginary (\`zi\`) parts. Zeros shape the
transient response and the root-locus departure, and a right-half-plane zero
signals non-minimum-phase behavior.

## Syntax

\`\`\`
CALL zero(num, den : zr, zi)
[zr, zi] = zero(num, den)
\`\`\`

## Description

Zeros are the values of \`s\` that make \`G(s) = 0\`. They do not affect stability
(that is the poles) but strongly influence overshoot, undershoot, and how a root
locus bends.

## Mathematical Formulation

$$ \\text{num}(s) = 0 \\quad\\Longrightarrow\\quad s = z_k $$

> **Method:** numerical polynomial root-finding on \`num\`.

## Examples

### Example 1 — Zero of a second-order plant

For \`G(s) = (s + 2)/(s² + 4s + 25)\`:

[Run: control-analysis-report]

**Expected:** a single real zero at \`s = −2\` (\`zr = −2\`, \`zi = 0\`).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`num\` | Vector | Yes | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Yes | Denominator coefficients (descending powers of \`s\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`zr\` | Vector | Real parts of the zeros. |
| \`zi\` | Vector | Imaginary parts of the zeros. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`EMPTY_NUMERATOR\` | \`num\` is constant or empty | A constant numerator has no finite zeros. |`,
  },
  {
    name: `zp2tf`,
    slug: `zp2tf`,
    category: `Control Systems`,
    summary: `Zero-pole-gain to transfer-function form.`,
    related: [`tf2zp`, `tf`, `pole`, `zero`],
    examples: [],
    tags: [`control`, `zero pole gain`, `zpk`, `transfer function`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Converts a **zero-pole-gain** description — zeros (\`zr\`/\`zi\`), poles (\`pr\`/\`pi\`),
and gain \`k\` — into a transfer function \`num/den\`. It is the inverse of
\`tf2zp\`, used to build a model from a factored (root) specification.

## Syntax

\`\`\`
CALL zp2tf(zr, zi, pr, pi, k : num, den)
[num, den] = zp2tf(zr, zi, pr, pi, k)
\`\`\`

## Mathematical Formulation

$$ G(s) = k\\,\\frac{\\prod_i (s - z_i)}{\\prod_j (s - p_j)} = \\frac{\\text{num}(s)}{\\text{den}(s)} $$

> **Method:** expand the zero and pole factors into polynomials and scale by \`k\`.

## Examples

\`\`\`
{ [num, den] = zp2tf(zr, zi, pr, pi, k) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`zr\`, \`zi\` | Vector | Yes | Real / imaginary parts of the zeros. |
| \`pr\`, \`pi\` | Vector | Yes | Real / imaginary parts of the poles. |
| \`k\` | Number | Yes | Scalar gain. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`num\` | Vector | Numerator coefficients (descending powers of \`s\`). |
| \`den\` | Vector | Denominator coefficients. |`,
  },
  {
    name: `Designing a Controller (PID & LQR)`,
    slug: `designing a controller (pid & lqr)`,
    category: `Cookbook`,
    summary: `Tune a PID and an LQR state-feedback controller for a plant and check the closed loop.`,
    related: [`pidtune`, `lqr`, `place`, `feedback`, `pole`, `margin`],
    examples: [`controller-design-lqr-pid`],
    tags: [`cookbook`, `control`, `pid`, `lqr`, `controller`, `state feedback`, `design`],
    references: [],
    guides: [],
    body: `**Goal:** take a plant model and design two controllers — a classical **PID** by
loop shaping and a modern **LQR** state-feedback — then confirm the closed loop is
stable and well-damped.

## What you'll build

Starting from the plant \`G(s) = num/den\` (or its state space \`(A, B)\`):

- **PID:** pick a target crossover and let \`pidtune\` return \`Kp, Ki, Kd\`.
- **LQR:** choose state/effort weights \`Q, R\` and let \`lqr\` return the optimal
  gain \`K\` (the control \`u = −Kx\`).

## Approach

The PID controller \`C(s) = Kp + Ki/s + Kd·s\` is shaped to cross 0 dB near \`ωc\` with
adequate phase margin. The LQR minimizes

$$ J = \\int_0^\\infty (\\mathbf{x}^\\top Q\\,\\mathbf{x} + \\mathbf{u}^\\top R\\,\\mathbf{u})\\,dt $$

giving \`K = R⁻¹BᵀP\` with \`P\` solving the algebraic Riccati equation.
Verify each design with \`pole\`/\`margin\` on the closed loop, formed
with \`feedback\`.

## Worked example

[Run: controller-design-lqr-pid]

**What it tells you:** the PID gains and the LQR gain, plus where each places the
closed-loop poles. Increasing \`Q/R\` (or \`ωc\`) gives a faster, more aggressive
response; both should land the dominant poles in the left half-plane.`,
  },
  {
    name: `EV Thermal-Management System`,
    slug: `ev thermal-management system`,
    category: `Cookbook`,
    summary: `Couple a coolant loop, a refrigerant loop, and a cabin into one acausal system model.`,
    related: [`ua_hx`, `htc_1phase`, `htc_evap`, `htc_cond`, `dp_2phase`, `hx_eta_surf`, `LiquidPump`, `TwoPhaseCompressor`, `Chiller`],
    examples: [`ev-thermal-management`],
    tags: [`cookbook`, `ev`, `thermal management`, `coolant`, `refrigerant`, `chiller`, `multi-domain`, `system`],
    references: [],
    guides: [],
    body: `**Goal:** model a complete electric-vehicle thermal-management system — a coolant
loop and a refrigerant loop coupled through a chiller — as one acausal model whose
heat exchangers are **sized from correlations and geometry**, not hand-set \`UA\`.

## What you'll build

[Diagram: EvThermal]

Three interacting sub-systems solved together:

- **Coolant loop (EG50):** a pump feeds a branch split (battery + motor), rejoined and
  pushed through a radiator that rejects heat to ambient.
- **Refrigerant loop (R1234yf):** a compressor drives a condenser → expansion → evaporator
  circuit whose head pressure floats with ambient and load.
- **Cross-domain bridge:** the chiller's refrigerant evaporator wall is tied to the
  battery-branch coolant — heat crosses domains in one solve.

## Approach

Every exchanger's conductance is **built from first principles** rather than guessed:
each side's film coefficient comes from a correlation (\`htc_1phase\` for
single-phase coolant, \`htc_evap\`/\`htc_cond\` for the boiling/
condensing refrigerant, \`htc_extair\` for the air side), combined with the
wall through \`ua_hx\`; pressure drops follow from \`dp_2phase\` and
the compact-core geometry helpers. The fin sides use
\`hx_eta_surf\`·\`fin_efficiency\`.

The coupled network expands to scalar equations and solves through the standard
Newton/Tarjan pipeline — the heat balances close themselves (e.g. chiller \`Q_ref ≡ Q_cool\`).

## Worked example

[Run: ev-thermal-management]

**What it tells you:** the operating point of the whole system — coolant and
refrigerant flows, the floating condenser/evaporator pressures, the battery-chiller
duty, and every \`UA\` that the geometry implies. Change the ambient temperature or a
fan speed and the floating pressures and duties re-solve consistently.`,
  },
  {
    name: `Frequency-Domain Stability Analysis`,
    slug: `frequency-domain stability analysis`,
    category: `Cookbook`,
    summary: `Analyze a plant end to end — poles/zeros, gain & phase margins, Bode, Nyquist, step.`,
    related: [`pole`, `zero`, `margin`, `bode`, `nyquist`, `step`],
    examples: [`control-analysis-report`],
    tags: [`cookbook`, `control`, `bode`, `nyquist`, `margin`, `stability`, `frequency response`],
    references: [],
    guides: [],
    body: `**Goal:** take a transfer function and characterize it completely — pole/zero
locations, gain and phase margins, the Bode and Nyquist responses, and the
time-domain step response — in one document.

## What you'll build

From \`G(s) = num/den\`:

1. \`pole\`/\`zero\` — the s-plane map and stability check.
2. \`margin\` — gain margin, phase margin, and crossover frequencies.
3. \`bode\`/\`nyquist\` — the frequency response, plotted inline.
4. \`step\` — the unit step response and its transient metrics.

## Approach

Stability is read three ways that must agree: all poles in the left
half-plane; positive gain/phase margins; and a Nyquist locus that does not encircle
\`−1 + j0\`. The frequency response evaluates \`G(jω)\` along the imaginary axis:

$$ \\text{mag} = 20\\log_{10}|G(j\\omega)|,\\qquad \\text{phase} = \\angle G(j\\omega) $$

Each figure is declared by a named \`PLOT … END\` block and appears in the Plots window after the solve.

## Worked example

[Run: control-analysis-report]

**What it tells you:** for the underdamped plant \`G(s) = (s+2)/(s²+4s+25)\` — poles at
\`−2 ± 4.58j\` (stable, \`ω_n = 5\`, \`ζ ≈ 0.4\`), a resonant Bode peak near 5 rad/s, a
Nyquist locus clear of \`−1\`, and a step response that overshoots and rings before
settling.`,
  },
  {
    name: `Rankine Steam Power Cycle`,
    slug: `rankine steam power cycle`,
    category: `Cookbook`,
    summary: `Model an ideal steam Rankine cycle and compute its thermal efficiency.`,
    related: [`Enthalpy`, `Entropy`, `Quality`, `Pump`, `Boiler`, `Turbine`, `Condenser`],
    examples: [`rankine-cycle`],
    tags: [`cookbook`, `rankine`, `steam`, `power cycle`, `efficiency`, `thermodynamics`],
    references: [],
    guides: [],
    body: `**Goal:** model the ideal steam power cycle and read off its **thermal efficiency**
from real-water properties.

## What you'll build

[Diagram: RankineCycle]

Water is carried around four state points:

1. **Pump** — isentropic compression of saturated liquid to the boiler pressure.
2. **Boiler** — heat addition \`q_in\` to superheated vapor.
3. **Turbine** — isentropic expansion to the condenser pressure, producing \`w_turb\`.
4. **Condenser** — heat rejection \`q_out\` back to saturated liquid.

## Approach

Fix the boiler and condenser pressures, then evaluate each enthalpy from the
fluid state. The ideal-cycle balances are:

$$ w_{turb} = h_3 - h_4,\\quad w_{pump} = h_2 - h_1,\\quad q_{in} = h_3 - h_2,\\quad \\eta_{th} = \\frac{w_{turb} - w_{pump}}{q_{in}} $$

The isentropic turbine sets \`s_4 = s_3\` (use \`Entropy\` at state 3, then
\`Enthalpy\` at the condenser pressure with that entropy). Check the
turbine-exit \`Quality\` — too low risks blade erosion (reheat fixes it).

## Worked example

[Run: rankine-cycle]

**What it tells you:** the thermal efficiency and the turbine-exit quality. Raising
the boiler pressure/temperature or lowering the condenser pressure increases \`η_th\`;
a reheat stage keeps the exit quality acceptable.

## Build it from components

A connected plant chains \`Pump\` → \`Boiler\` → \`Turbine\`
→ \`Condenser\` on a single \`fluid$\` stream.`,
  },
  {
    name: `Rating a Heat Exchanger (ε-NTU)`,
    slug: `rating a heat exchanger (ε-ntu)`,
    category: `Cookbook`,
    summary: `Find a heat exchanger's duty and outlet temperatures from its UA by the effectiveness-NTU method.`,
    related: [`hx_effectiveness`, `hx_NTU`, `LMTD`, `ua_hx`],
    examples: [`hx-effectiveness-ntu`],
    tags: [`cookbook`, `heat exchanger`, `effectiveness`, `ntu`, `rating`, `duty`, `heat transfer`],
    references: [],
    guides: [],
    body: `**Goal:** given an exchanger's conductance \`UA\` and the two inlet streams, find the
**heat duty and both outlet temperatures** — without iterating on the unknown
outlets (which the LMTD method would require).

## What you'll build

A rating calculation in four steps:

1. Form each stream's heat-capacity rate \`C = ṁ·cp\`; take \`Cmin\`, \`Cmax\`.
2. Compute \`NTU = UA/Cmin\` and the capacity ratio \`Cr = Cmin/Cmax\`.
3. Get the effectiveness ε from the arrangement.
4. Back out the duty and outlets.

## Approach

The effectiveness–NTU relations give ε directly per flow arrangement
(\`hx_effectiveness\`), so the duty follows from the inlets alone:

$$ \\varepsilon = f(NTU, C_r),\\quad Q = \\varepsilon\\,C_{min}(T_{h,in} - T_{c,in}),\\quad T_{h,out} = T_{h,in} - \\frac{Q}{C_h},\\ T_{c,out} = T_{c,in} + \\frac{Q}{C_c} $$

Use \`hx_NTU\` for the inverse (sizing to a target ε) and \`LMTD\` to
cross-check the mean driving temperature.

## Worked example

[Run: hx-effectiveness-ntu]

**What it tells you:** the duty \`Q ≈ 312 kW\`, the outlet temperatures
(\`Th_out ≈ 323 K\`, \`Tc_out ≈ 340 K\`) and the effectiveness \`ε ≈ 0.71\` for the
counterflow case — read straight from \`UA\` with no iteration.`,
  },
  {
    name: `Real-Gas Properties with a Cubic EOS`,
    slug: `real-gas properties with a cubic eos`,
    category: `Cookbook`,
    summary: `Get Z, density, enthalpy, and saturation pressure of a real gas from a cubic equation of state.`,
    related: [`eos_z`, `eos_density`, `eos_enthalpy`, `eos_psat`, `eos_entropy`],
    examples: [`cubic-eos-properties`],
    tags: [`cookbook`, `eos`, `peng-robinson`, `srk`, `real gas`, `properties`, `thermodynamics`],
    references: [],
    guides: [],
    body: `**Goal:** evaluate real-gas properties — compressibility factor, density, enthalpy,
and saturation pressure — from a cubic equation of state, with **no real-fluid backend
needed** (only critical constants and the acentric factor are needed).

## What you'll build

For a chosen fluid and model (\`'SRK'\` or \`'PR'\`):

- \`eos_z\` — the compressibility factor \`Z\`, the root of the cubic.
- \`eos_density\`/\`eos_volume\` — density and specific volume.
- \`eos_enthalpy\`/\`eos_entropy\` — with the EOS departure term.
- \`eos_psat\` — vapor pressure from the equal-fugacity condition.

## Approach

Peng–Robinson casts the equation of state as a cubic in \`Z\` (with \`A = aαP/(RT)²\`,
\`B = bP/RT\`):

$$ Z^3 - (1-B)Z^2 + (A - 2B - 3B^2)Z - (AB - B^2 - B^3) = 0 $$

The largest real root is the vapor branch, the smallest the liquid. Density follows
from \`ρ = P/(ZRT)\`; enthalpy adds the analytic departure to the ideal-gas value; the
saturation pressure is the \`P\` at which the liquid and vapor fugacities match.

## Worked example

[Run: cubic-eos-properties]

**What it tells you:** for CO₂ near its critical region (320 K, 6 MPa), \`Z ≈ 0.7\`
(strong real-gas deviation) and a density far above the ideal-gas estimate;
\`eos_psat('co2','PR',300) ≈ 6.7 MPa\` matches the known vapor pressure.`,
  },
  {
    name: `SI Engine Cycle (Wiebe Heat Release)`,
    slug: `si engine cycle (wiebe heat release)`,
    category: `Cookbook`,
    summary: `Build a single-zone spark-ignition engine cycle and integrate its cylinder-pressure trace.`,
    related: [`wiebe_rate`, `AdiabaticFlameTemp`],
    examples: [`engine-cycle-wiebe`],
    tags: [`cookbook`, `engine`, `wiebe`, `heat release`, `indicator diagram`, `powertrain`, `dynamic`],
    references: [],
    guides: [],
    body: `**Goal:** model a single-zone spark-ignition engine over the compression–combustion–
expansion strokes and integrate the **cylinder-pressure trace** for the indicator
(p–V) diagram.

## What you'll build

A crank-angle first-law model integrated by a \`DYNAMIC\` block:

- Cylinder volume from slider-crank kinematics (compression ratio, displacement).
- Burned-mass-fraction rate from a Wiebe function (\`wiebe_rate\`).
- \`dp/dθ\` from the first law, integrated over crank angle.

## Approach

The Wiebe burn rate spreads the total heat release \`Q_tot\` over the burn duration:

$$ \\frac{dQ}{d\\theta} = Q_{tot}\\,\\frac{dx_b}{d\\theta},\\qquad x_b = 1 - \\exp\\!\\left[-a\\left(\\tfrac{\\theta-\\theta_0}{\\Delta\\theta}\\right)^{m+1}\\right] $$

The single-zone energy balance then gives \`dp/dθ\` as a function of the changing volume
and the instantaneous heat release; the \`DYNAMIC\` integrator marches it over the crank
angle. Plot \`p\` vs \`V\` for the indicator diagram.

## Worked example

[Run: engine-cycle-wiebe]

**What it tells you:** the cylinder-pressure history and the closed p–V loop whose
area is the indicated work. The burn looks bell-shaped (peaking partway through the
duration); advancing or retarding \`θ_soc\` shifts the peak pressure and the work.`,
  },
  {
    name: `Supersonic Nozzle with a Normal Shock`,
    slug: `supersonic nozzle with a normal shock`,
    category: `Cookbook`,
    summary: `Trace a converging-diverging nozzle flow through a normal shock and find the stagnation-pressure loss.`,
    related: [`mach_A_Astar`, `T0_T`, `P0_P`, `M2_shock`, `P2_P1_shock`, `P02_P01_shock`],
    examples: [`cd-nozzle-shock`],
    tags: [`cookbook`, `compressible`, `nozzle`, `shock`, `supersonic`, `aerospace`, `gas dynamics`],
    references: [],
    guides: [],
    body: `**Goal:** trace ideal-gas flow through a converging–diverging nozzle that contains a
**normal shock** in its diverging section, and quantify the **stagnation-pressure
loss** across the shock.

## What you'll build

From the reservoir conditions and the area ratio at the shock:

1. \`mach_A_Astar\` — the supersonic Mach just upstream of the shock.
2. \`T0_T\`/\`P0_P\` — the static temperature and pressure there.
3. \`M2_shock\`/\`P2_P1_shock\` — the subsonic Mach and pressure jump.
4. \`P02_P01_shock\` — the stagnation-pressure recovery (the loss).

## Approach

The area–Mach relation is double-valued, so the regime selector picks the supersonic
root upstream of the shock. The shock then jumps the flow to subsonic
with a static-pressure rise but a stagnation-pressure drop:

$$ \\frac{P_2}{P_1} = \\frac{2kM_1^2 - (k-1)}{k+1},\\qquad \\frac{P_{02}}{P_{01}} < 1 $$

the latter quantifying the irreversibility of the shock.

## Worked example

[Run: cd-nozzle-shock]

**What it tells you:** at \`A/A* = 2.0\` (air), the upstream Mach is \`M1 ≈ 2.20\`, the
static pressure jumps roughly five-fold (\`P2/P1 ≈ 5.5\`), and the stagnation pressure
recovers to only \`≈ 63 %\` (\`P02 ≈ 628 kPa\` from a 1 MPa reservoir) — the price of the
shock.`,
  },
  {
    name: `Vapor-Compression Refrigeration Cycle`,
    slug: `vapor-compression refrigeration cycle`,
    category: `Cookbook`,
    summary: `Build an R134a vapor-compression cycle and compute its COP from real-refrigerant properties.`,
    related: [`Enthalpy`, `Quality`, `Compressor`, `Condenser`, `ExpansionValve`, `TwoPhaseEvaporator`],
    examples: [`refrigeration-vcr`],
    tags: [`cookbook`, `refrigeration`, `vcr`, `cop`, `refrigerant`, `cycle`, `thermodynamics`],
    references: [],
    guides: [],
    body: `**Goal:** model the standard four-process refrigeration cycle and read off its
**coefficient of performance (COP)** using real-refrigerant properties.

## What you'll build

[Diagram: RefrigerationCycle]

The cycle walks one refrigerant (R134a here) around four state points:

1. **Evaporator** — saturated/superheated vapor leaves at the low pressure, absorbing \`q_L\`.
2. **Compressor** — isentropic (or efficiency-corrected) compression to the high pressure.
3. **Condenser** — heat rejection \`q_H\`, leaving saturated/subcooled liquid.
4. **Expansion valve** — isenthalpic throttle back to the low pressure.

## Approach

Anchor the two pressures by saturation temperatures, then evaluate each state's
enthalpy from a real-fluid property call. The cycle energy balances are:

$$ q_L = h_1 - h_4,\\quad w_c = h_2 - h_1,\\quad q_H = h_2 - h_3,\\quad \\text{COP} = \\frac{q_L}{w_c} $$

with the isenthalpic valve giving \`h_4 = h_3\`. Use \`T_sat\`/\`P_sat\`
to set the pressures and \`Enthalpy\` (with quality or superheat) for each point.

## Worked example

[Run: refrigeration-vcr]

**What it tells you:** the COP — cooling delivered per unit compressor work — and
how it falls as the condensing/evaporating temperature spread widens. Swapping the
fluid name (e.g. to R1234yf) re-evaluates every property in place.

## Build it from components

For a *connected* circuit rather than a state-by-state script, assemble the
two-phase component library: \`TwoPhaseCompressor\` →
\`TwoPhaseCondenser\` → \`TwoPhaseExpansionValve\`
→ \`TwoPhaseEvaporator\`, closed into a loop (see the
*EV Thermal-Management System* guide for a full coupled example).`,
  },
  {
    name: `friction_factor`,
    slug: `friction_factor`,
    category: `Flow Networks`,
    summary: `Darcy friction factor (Colebrook-Moody, laminar+turbulent)`,
    related: [],
    examples: [],
    tags: [`friction`, `factor`, `flow`, `networks`],
    references: [],
    guides: [],
    body: `Darcy friction factor (Colebrook-Moody, laminar+turbulent)


## Syntax

\`\`\`
friction_factor(Re, rel_rough)
\`\`\`

## Description

Darcy friction factor (Colebrook-Moody, laminar+turbulent)

## Mathematical Formulation

$$ \\frac{1}{\\sqrt{f}} = -2\\log_{10}\\!\\left(\\frac{\\varepsilon/D}{3.7} + \\frac{2.51}{Re\\sqrt{f}}\\right) \\quad\\text{(Colebrook; } f = 64/Re \\text{ laminar)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re\` | Number | Yes | Reynolds number. |
| \`rel_rough\` | Number | Yes | Relative wall roughness ε/D. |`,
  },
  {
    name: `minor_loss`,
    slug: `minor_loss`,
    category: `Flow Networks`,
    summary: `Minor (fitting) pressure loss K*0.5*rho*V^2 [Pa]`,
    related: [],
    examples: [],
    tags: [`minor`, `loss`, `flow`, `networks`],
    references: [],
    guides: [],
    body: `Minor (fitting) pressure loss K*0.5*rho*V^2 [Pa]


## Syntax

\`\`\`
minor_loss(K, rho, V)
\`\`\`

## Description

Minor (fitting) pressure loss K*0.5*rho*V^2 [Pa]

## Mathematical Formulation

$$ \\Delta P = K\\,\\tfrac12\\rho V^2 $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`K\` | Number | Yes | Loss coefficient / gain. |
| \`rho\` | Number | Yes | Density [kg/m³]. |
| \`V\` | Number | Yes | Velocity [m/s]. |`,
  },
  {
    name: `reynolds`,
    slug: `reynolds`,
    category: `Flow Networks`,
    summary: `Reynolds number rho*V*D/mu`,
    related: [],
    examples: [],
    tags: [`reynolds`, `flow`, `networks`],
    references: [],
    guides: [],
    body: `Reynolds number rho*V*D/mu


## Syntax

\`\`\`
reynolds(rho, V, D, mu)
\`\`\`

## Description

Reynolds number rho*V*D/mu

## Mathematical Formulation

$$ Re = \\frac{\\rho V D}{\\mu} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`rho\` | Number | Yes | Density [kg/m³]. |
| \`V\` | Number | Yes | Velocity [m/s]. |
| \`D\` | Number | Yes | Diameter [m]. |
| \`mu\` | Number | Yes | Dynamic viscosity [Pa·s]. |`,
  },
  {
    name: `compressibility`,
    slug: `compressibility`,
    category: `Fluid Properties`,
    summary: `Fluid property: compressibility from the real-fluid property backend.`,
    related: [],
    examples: [`thermo-compliance`],
    tags: [`compressibility`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [],
    body: `Returns the **compressibility** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
compressibility(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `compressibilityfactor`,
    slug: `compressibilityfactor`,
    category: `Fluid Properties`,
    summary: `Fluid property: compressibilityfactor from the real-fluid property backend.`,
    related: [],
    examples: [`thermo-compliance`],
    tags: [`compressibilityfactor`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **compressibilityfactor** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
compressibilityfactor(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `conductivity`,
    slug: `conductivity`,
    category: `Fluid Properties`,
    summary: `Fluid property: conductivity from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`conductivity`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **conductivity** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
conductivity(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `cp`,
    slug: `cp`,
    category: `Fluid Properties`,
    summary: `Fluid property: cp from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`cp`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **cp** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
cp(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `cv`,
    slug: `cv`,
    category: `Fluid Properties`,
    summary: `Fluid property: cv from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`cv`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **cv** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
cv(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `density`,
    slug: `density`,
    category: `Fluid Properties`,
    summary: `Fluid property: density from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`density`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **density** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
density(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `dewpoint`,
    slug: `dewpoint`,
    category: `Fluid Properties`,
    summary: `Humid-air property: dewpoint from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`dewpoint`, `property`, `humid-air`, `coolprop`],
    references: [],
    guides: [`humidair`, `tut-coil`],
    body: `Returns the **dewpoint** of a humid-air (AirH2O) from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
dewpoint(AirH2O, T=, P=, R=)
\`\`\`

## Description

A humid-air property; supply the dry-bulb T, total pressure P, and one humidity coordinate (R, W, B, or D). Property names are case-insensitive.`,
  },
  {
    name: `enthalpy`,
    slug: `enthalpy`,
    category: `Fluid Properties`,
    summary: `Fluid property: enthalpy from the real-fluid property backend.`,
    related: [],
    examples: [`rankine-cycle`, `state-tables-multifluid`, `rankine-cycle`, `refrigeration-vcr`],
    tags: [`enthalpy`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`debugging`, `errors`, `thermo`, `humidair`, `gs-repl`, `repl`],
    body: `Returns the **enthalpy** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
enthalpy(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `entropy`,
    slug: `entropy`,
    category: `Fluid Properties`,
    summary: `Fluid property: entropy from the real-fluid property backend.`,
    related: [],
    examples: [`rankine-cycle`, `rankine-cycle`, `refrigeration-vcr`],
    tags: [`entropy`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`, `tut-vccycle`],
    body: `Returns the **entropy** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
entropy(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `gibbs`,
    slug: `gibbs`,
    category: `Fluid Properties`,
    summary: `Fluid property: gibbs from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`gibbs`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **gibbs** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
gibbs(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `humrat`,
    slug: `humrat`,
    category: `Fluid Properties`,
    summary: `Humid-air property: humrat from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`humrat`, `property`, `humid-air`, `coolprop`],
    references: [],
    guides: [`humidair`, `tut-coil`],
    body: `Returns the **humrat** of a humid-air (AirH2O) from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
humrat(AirH2O, T=, P=, R=)
\`\`\`

## Description

A humid-air property; supply the dry-bulb T, total pressure P, and one humidity coordinate (R, W, B, or D). Property names are case-insensitive.`,
  },
  {
    name: `intenergy`,
    slug: `intenergy`,
    category: `Fluid Properties`,
    summary: `Fluid property: intenergy from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`intenergy`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **intenergy** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
intenergy(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `prandtl`,
    slug: `prandtl`,
    category: `Fluid Properties`,
    summary: `Fluid property: prandtl from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`prandtl`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [],
    body: `Returns the **prandtl** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
prandtl(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `pressure`,
    slug: `pressure`,
    category: `Fluid Properties`,
    summary: `Fluid property: pressure from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`pressure`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **pressure** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
pressure(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `quality`,
    slug: `quality`,
    category: `Fluid Properties`,
    summary: `Fluid property: quality from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`quality`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [],
    body: `Returns the **quality** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
quality(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `relhum`,
    slug: `relhum`,
    category: `Fluid Properties`,
    summary: `Humid-air property: relhum from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`relhum`, `property`, `humid-air`, `coolprop`],
    references: [],
    guides: [`humidair`],
    body: `Returns the **relhum** of a humid-air (AirH2O) from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
relhum(AirH2O, T=, P=, R=)
\`\`\`

## Description

A humid-air property; supply the dry-bulb T, total pressure P, and one humidity coordinate (R, W, B, or D). Property names are case-insensitive.`,
  },
  {
    name: `soundspeed`,
    slug: `soundspeed`,
    category: `Fluid Properties`,
    summary: `Fluid property: soundspeed from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`soundspeed`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **soundspeed** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
soundspeed(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `specheat`,
    slug: `specheat`,
    category: `Fluid Properties`,
    summary: `Fluid property: specheat from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`specheat`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [],
    body: `Returns the **specheat** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
specheat(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `temperature`,
    slug: `temperature`,
    category: `Fluid Properties`,
    summary: `Fluid property: temperature from the real-fluid property backend.`,
    related: [],
    examples: [`pressure-cooker`],
    tags: [`temperature`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`debugging`, `errors`, `comp-authoring`, `thermo`],
    body: `Returns the **temperature** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
temperature(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `viscosity`,
    slug: `viscosity`,
    category: `Fluid Properties`,
    summary: `Fluid property: viscosity from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`viscosity`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **viscosity** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
viscosity(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `volexpcoef`,
    slug: `volexpcoef`,
    category: `Fluid Properties`,
    summary: `Fluid property: volexpcoef from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`volexpcoef`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [],
    body: `Returns the **volexpcoef** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
volexpcoef(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `volume`,
    slug: `volume`,
    category: `Fluid Properties`,
    summary: `Fluid property: volume from the real-fluid property backend.`,
    related: [],
    examples: [`rankine-cycle`, `thermo-compliance`, `rankine-cycle`, `engine-cycle-wiebe`],
    tags: [`volume`, `property`, `fluid`, `coolprop`],
    references: [],
    guides: [`thermo`],
    body: `Returns the **volume** of a real fluid from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
volume(Fluid, P=, T=)
\`\`\`

## Description

Supply the fluid name and any two independent state properties (T, P, h, s, x, …). Property names are case-insensitive.`,
  },
  {
    name: `wetbulb`,
    slug: `wetbulb`,
    category: `Fluid Properties`,
    summary: `Humid-air property: wetbulb from the real-fluid property backend.`,
    related: [],
    examples: [],
    tags: [`wetbulb`, `property`, `humid-air`, `coolprop`],
    references: [],
    guides: [`humidair`],
    body: `Returns the **wetbulb** of a humid-air (AirH2O) from any valid pair of independent state properties (rustprop, a pure-Rust port of CoolProp 8.0.0).

> Real-fluid/material/symbolic operation — see the inputs and references below.

## Syntax

\`\`\`
wetbulb(AirH2O, T=, P=, R=)
\`\`\`

## Description

A humid-air property; supply the dry-bulb T, total pressure P, and one humidity coordinate (R, W, B, or D). Property names are case-insensitive.`,
  },
  {
    name: `dp_1phase`,
    slug: `dp_1phase`,
    category: `Heat Transfer`,
    summary: `dP [Pa], Darcy. SIDE: single-phase liquid/gas line (coolant, water, air channel, pipe). HX: radiator/CAC fluid channels`,
    related: [],
    examples: [],
    tags: [`dp`, `1phase`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `dP [Pa], Darcy. SIDE: single-phase liquid/gas line (coolant, water, air channel, pipe). HX: radiator/CAC fluid channels


## Syntax

\`\`\`
dp_1phase(fluid$, P, T, mdot, Dh, Aflow, L)
\`\`\`

## Description

dP [Pa], Darcy. SIDE: single-phase liquid/gas line (coolant, water, air channel, pipe). HX: radiator/CAC fluid channels

## Mathematical Formulation

$$ \\Delta P = f\\,\\frac{L}{D_h}\\,\\frac{G^2}{2\\rho}, \\qquad G = \\dot m / A_{\\text{flow}} \\quad\\text{(Darcy)} $$

## Applicability

- **Where it applies:** A single-phase liquid/gas line (coolant, water, air channel, pipe).
- **Valid when:** Single-phase Darcy flow; turbulent or laminar.
- **How it's used:** Friction \`ΔP\` for radiator/CAC fluid channels and connecting lines.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name (e.g. Water, R134a, Air). |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`Dh\` | Number | Yes | Hydraulic diameter [m]. |
| \`Aflow\` | Number | Yes | Free-flow (minimum) cross-sectional area [m²]. |
| \`L\` | Number | Yes | Length [m]. |`,
  },
  {
    name: `dp_2phase`,
    slug: `dp_2phase`,
    category: `Heat Transfer`,
    summary: `Two-phase frictional pressure drop — Lockhart-Martinelli / Chisholm multiplier.`,
    related: [`htc_evap`, `htc_cond`, `dp_1phase`],
    examples: [`ev-thermal-management`],
    tags: [`two-phase`, `pressure drop`, `lockhart-martinelli`, `chisholm`, `friction`, `refrigerant`],
    references: [],
    guides: [],
    body: `Returns the **two-phase frictional pressure drop** \`dP\` [Pa] of an evaporating or
condensing refrigerant over a passage length \`L\`, using the **Lockhart-Martinelli /
Chisholm** two-phase multiplier on the liquid-alone pressure gradient.

## Syntax

\`\`\`
dP = dp_2phase(fluid$, P, x, mdot, Dh, Aflow, L)
\`\`\`

## Description

Two-phase flow drops more pressure than the liquid alone because the vapor
accelerates and roughens the flow. The Chisholm multiplier scales the liquid-only
Darcy drop by a factor that depends on the Martinelli parameter \`X\`.

## Mathematical Formulation

With the liquid-only pressure gradient $(dP/dz)_l$ and the Martinelli parameter \`X\`,

$$ \\phi_l^2 = 1 + \\frac{C}{X} + \\frac{1}{X^2}, \\qquad \\Delta P = \\phi_l^2\\left(\\frac{dP}{dz}\\right)_l L $$

where the Chisholm constant \`C\` ranges from 5 (laminar–laminar) to 20
(turbulent–turbulent).

> **Method:** liquid-only Darcy gradient × Chisholm multiplier \`φ_l²(X, C)\`,
> integrated over \`L\` at the local quality \`x\`.

## Examples

### Example 1 — Evaporator refrigerant-side pressure drop

[Run: ev-thermal-management]

**Expected:** a pressure drop several times the liquid-only value, rising with
quality as the vapor fraction grows.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Refrigerant name. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`Dh\` | Number | Yes | Hydraulic diameter [m]. |
| \`Aflow\` | Number | Yes | Free-flow area [m²]. |
| \`L\` | Number | Yes | Passage length [m]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`dP\` | Number | Two-phase frictional pressure drop [Pa]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x\` outside [0, 1] or \`L ≤ 0\` | Quality in [0, 1], positive length. |`,
  },
  {
    name: `dp_2phase_avg`,
    slug: `dp_2phase_avg`,
    category: `Heat Transfer`,
    summary: `dP [Pa], quality-integrated (n cells). SIDE: two-phase refrigerant along an evaporator/condenser pass`,
    related: [],
    examples: [],
    tags: [`dp`, `2phase`, `avg`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `dP [Pa], quality-integrated (n cells). SIDE: two-phase refrigerant along an evaporator/condenser pass


## Syntax

\`\`\`
dp_2phase_avg(fluid$, P, x_in, x_out, mdot, Dh, Aflow, L, n)
\`\`\`

## Description

dP [Pa], quality-integrated (n cells). SIDE: two-phase refrigerant along an evaporator/condenser pass

## Mathematical Formulation

$$ \\Delta P = \\frac{1}{n}\\sum_{i=1}^{n} \\phi_l^2(x_i)\\,\\left(\\frac{dP}{dz}\\right)_{l,i} \\Delta z \\quad\\text{(quality-integrated)} $$

## Applicability

- **Where it applies:** Two-phase refrigerant along an evaporator/condenser pass.
- **Valid when:** Integrates the two-phase multiplier over \`n\` quality cells from \`x_in\` to \`x_out\`.
- **How it's used:** A quality-averaged frictional \`ΔP\` for a whole pass.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name (e.g. Water, R134a, Air). |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`x_in\` | Number | Yes | Inlet vapor quality (0–1). |
| \`x_out\` | Number | Yes | Outlet vapor quality (0–1). |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`Dh\` | Number | Yes | Hydraulic diameter [m]. |
| \`Aflow\` | Number | Yes | Free-flow (minimum) cross-sectional area [m²]. |
| \`L\` | Number | Yes | Length [m]. |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `dp_compact_core`,
    slug: `dp_compact_core`,
    category: `Heat Transfer`,
    summary: `dP [Pa], compact-core (entrance/accel/core-friction/exit). SIDE: air/gas through a compact finned core. HX: fin-and-tube/plate-fin radiator, condenser, CAC air side`,
    related: [],
    examples: [],
    tags: [`dp`, `compact`, `core`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `dP [Pa], compact-core (entrance/accel/core-friction/exit). SIDE: air/gas through a compact finned core. HX: fin-and-tube/plate-fin radiator, condenser, CAC air side


## Syntax

\`\`\`
dp_compact_core(G, rho_in, rho_out, rho_mean, sigma, f, AoverAc, Kc, Ke)
\`\`\`

## Description

dP [Pa], compact-core (entrance/accel/core-friction/exit). SIDE: air/gas through a compact finned core. HX: fin-and-tube/plate-fin radiator, condenser, CAC air side

## Mathematical Formulation

$$ \\frac{\\Delta P}{P_1} = \\frac{G^2}{2\\rho_1 P_1}\\left[(1+\\sigma^2)\\!\\left(\\tfrac{\\rho_1}{\\rho_2}-1\\right) + f\\tfrac{A}{A_c}\\tfrac{\\rho_1}{\\rho_m}\\right] \\quad\\text{(compact core)} $$

## Applicability

- **Where it applies:** Air/gas through a compact finned core.
- **Valid when:** Includes the entrance, acceleration, core-friction and exit terms.
- **How it's used:** Air-side \`ΔP\` for a fin-and-tube / plate-fin radiator, condenser, or CAC.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`G\` | Number | Yes | Mass flux G = ṁ/Aflow [kg/m²·s]. |
| \`rho_in\` | Number | Yes | Inlet density [kg/m³]. |
| \`rho_out\` | Number | Yes | Outlet density [kg/m³]. |
| \`rho_mean\` | Number | Yes | Mean density [kg/m³]. |
| \`sigma\` | Number | Yes | Surface tension [N/m]. |
| \`f\` | Number | Yes | Fanning/Darcy friction factor. |
| \`AoverAc\` | Number | Yes | Area ratio A/Ac. |
| \`Kc\` | Number | Yes | Contraction (entrance) loss coefficient. |
| \`Ke\` | Number | Yes | Exit (expansion) loss coefficient. |`,
  },
  {
    name: `dp_gravity`,
    slug: `dp_gravity`,
    category: `Heat Transfer`,
    summary: `dP [Pa], static head. SIDE: two-phase refrigerant in a vertical riser/downcomer. HX: evaporator/condenser vertical passes`,
    related: [],
    examples: [],
    tags: [`dp`, `gravity`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `dP [Pa], static head. SIDE: two-phase refrigerant in a vertical riser/downcomer. HX: evaporator/condenser vertical passes


## Syntax

\`\`\`
dp_gravity(rho_l, rho_g, alpha, L, theta_deg)
\`\`\`

## Description

dP [Pa], static head. SIDE: two-phase refrigerant in a vertical riser/downcomer. HX: evaporator/condenser vertical passes

## Mathematical Formulation

$$ \\Delta P_{\\text{grav}} = \\big[\\alpha\\rho_g + (1-\\alpha)\\rho_l\\big]\\,g\\,L\\sin\\theta $$

## Applicability

- **Where it applies:** Two-phase refrigerant in a vertical riser/downcomer.
- **Valid when:** Static-head term; sign follows the flow direction \`θ\`.
- **How it's used:** Add to the frictional and acceleration terms for the total vertical-pass \`ΔP\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`rho_l\` | Number | Yes | Saturated-liquid density [kg/m³]. |
| \`rho_g\` | Number | Yes | Saturated-vapor density [kg/m³]. |
| \`alpha\` | Number | Yes | Void fraction (0–1). |
| \`L\` | Number | Yes | Length [m]. |
| \`theta_deg\` | Number | Yes | Angle [deg]. |`,
  },
  {
    name: `dp_mueller_steinhagen`,
    slug: `dp_mueller_steinhagen`,
    category: `Heat Transfer`,
    summary: `dP [Pa], Mueller-Steinhagen-Heck. SIDE: two-phase refrigerant (alt to dp_2phase). HX: evaporator/condenser refrigerant line`,
    related: [],
    examples: [],
    tags: [`dp`, `mueller`, `steinhagen`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `dP [Pa], Mueller-Steinhagen-Heck. SIDE: two-phase refrigerant (alt to dp_2phase). HX: evaporator/condenser refrigerant line


## Syntax

\`\`\`
dp_mueller_steinhagen(fluid$, P, x, mdot, Dh, Aflow, L)
\`\`\`

## Description

dP [Pa], Mueller-Steinhagen-Heck. SIDE: two-phase refrigerant (alt to dp_2phase). HX: evaporator/condenser refrigerant line

## Mathematical Formulation

$$ \\frac{dP}{dz} = G_{ms}(1-x)^{1/3} + B\\,x^3, \\quad G_{ms} = A + 2(B-A)x \\quad\\text{(Müller-Steinhagen–Heck)} $$

## Applicability

- **Where it applies:** Two-phase refrigerant in an evaporator/condenser line.
- **Valid when:** Two-phase frictional drop; an alternative to the Chisholm/Friedel route (\`dp_2phase\`).
- **How it's used:** Interpolates between the all-liquid and all-vapor drops over the quality range.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name (e.g. Water, R134a, Air). |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`Dh\` | Number | Yes | Hydraulic diameter [m]. |
| \`Aflow\` | Number | Yes | Free-flow (minimum) cross-sectional area [m²]. |
| \`L\` | Number | Yes | Length [m]. |`,
  },
  {
    name: `f_fin`,
    slug: `f_fin`,
    category: `Heat Transfer`,
    summary: `Fanning friction for a compact fin surface. SIDE: air/gas finned side dP (pair with j_fin). HX: same as j_fin`,
    related: [],
    examples: [],
    tags: [`fin`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Fanning friction for a compact fin surface. SIDE: air/gas finned side dP (pair with j_fin). HX: same as j_fin


## Syntax

\`\`\`
f_fin(surface$, Re)
\`\`\`

## Description

Fanning friction for a compact fin surface. SIDE: air/gas finned side dP (pair with j_fin). HX: same as j_fin

## Mathematical Formulation

$$ f = C_f\\,Re^{m_f} \\quad\\text{(Fanning friction for the fin surface)} $$

## Applicability

- **Where it applies:** The air/gas finned side of a compact surface.
- **Valid when:** Same fin surfaces as \`j_fin\`.
- **How it's used:** Air-side friction (Fanning) for the core \`ΔP\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`surface$\` | String | Yes | Selector — One of \`plain\`, \`wavy\`, \`louvered\`, \`offset\`. |
| \`Re\` | Number | Yes | Reynolds number. |`,
  },
  {
    name: `fin_efficiency`,
    slug: `fin_efficiency`,
    category: `Heat Transfer`,
    summary: `Efficiency of a straight fin with an insulated tip.`,
    related: [`hx_effectiveness`, `hx_eta_surf`],
    examples: [`ev-thermal-management`],
    tags: [`fin`, `efficiency`, `extended surface`, `tanh`, `conduction`],
    references: [],
    guides: [],
    body: `Returns the **efficiency of a straight fin** with an insulated tip from its
dimensionless parameter \`mL\` — the ratio of the heat the fin actually dissipates
to what it would dissipate if its entire surface were at the base temperature.
Use it when sizing extended surfaces (fin-and-tube and plate-fin cores), typically
combined with \`hx_eta_surf\` into an overall surface efficiency.

## Syntax

\`\`\`
eta = fin_efficiency(mL)
\`\`\`

## Description

A real fin's temperature falls along its length as it sheds heat, so it is less
effective than an isothermal fin. The single group \`mL\` captures the competition
between convection off the surface and conduction along the fin. The result drops
from 1 (short/high-conductivity fin) toward 0 (long/poorly-conducting fin).

## Mathematical Formulation

For a straight fin of length $L$ with an insulated (adiabatic) tip,

$$ \\eta_f = \\frac{\\tanh(mL)}{mL} $$

where the fin parameter follows from the 1-D fin energy balance,

$$ m = \\sqrt{\\frac{h\\,P}{k\\,A_c}} $$

for convection coefficient $h$, fin perimeter $P$, thermal conductivity $k$, and
cross-sectional area $A_c$.

> **Method:** direct evaluation; as $mL \\to 0$, $\\eta_f \\to 1$.

## Examples

### Example 1 — Air-side fin efficiency in an EV condenser/radiator

The thermal-management sizing computes a fin parameter \`mL\` for each air-side
core and feeds \`fin_efficiency(mL)\` into the overall surface efficiency
\`hx_eta_surf(...)\` before forming \`UA = h·A·η\`.

[Run: ev-thermal-management]

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`mL\` | Number | Yes | Fin parameter–length product \`m·L\` (dimensionless, ≥ 0), with \`m = sqrt(h·P/(k·Ac))\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`eta\` | Number | Fin efficiency η ∈ (0, 1]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`mL\` negative | \`m\` and \`L\` are positive; check \`h\`, \`P\`, \`k\`, \`Ac\` in \`m = sqrt(h·P/(k·Ac))\`. |`,
  },
  {
    name: `heisler_q`,
    slug: `heisler_q`,
    category: `Heat Transfer`,
    summary: `Fraction of total heat transferred Q/Q0 (Heisler one-term) for a wall, cylinder, or sphere.`,
    related: [`heisler_temp`],
    examples: [`heisler-transient`],
    tags: [`transient conduction`, `heisler`, `heat fraction`, `biot`, `fourier`, `energy`],
    references: [],
    guides: [`chemistry`],
    body: `Returns the **fraction of the maximum possible heat** \`Q/Q0\` that a plane wall,
infinite cylinder, or sphere has exchanged with its surroundings up to Fourier
number \`Fo\`, using the one-term (Heisler) approximation. \`Q0 = ρ·c·V·(Ti − T∞)\` is
the energy available relative to the ambient.

## Syntax

\`\`\`
ratio = heisler_q(geom$, Bi, Fo)
\`\`\`

## Description

While \`heisler_temp\` gives a point temperature, \`heisler_q\` gives
the integrated energy removed (or added) so far — useful for transient duty and
storage calculations.

## Mathematical Formulation

With the midplane ratio $\\theta_0^* = C_1\\exp(-\\lambda_1^2 Fo)$ and
$Q_0 = \\rho c V(T_i - T_\\infty)$:

$$ \\text{wall: } \\frac{Q}{Q_0} = 1 - \\frac{\\theta_0^*}{\\lambda_1}\\sin\\lambda_1 $$
$$ \\text{cylinder: } \\frac{Q}{Q_0} = 1 - \\frac{2\\theta_0^*}{\\lambda_1}J_1(\\lambda_1), \\qquad \\text{sphere: } \\frac{Q}{Q_0} = 1 - \\frac{3\\theta_0^*}{\\lambda_1^3}\\big(\\sin\\lambda_1 - \\lambda_1\\cos\\lambda_1\\big) $$

> **Method:** first-term truncation using the same \`λ1(Bi)\`, \`C1(Bi)\` as
> \`heisler_temp\`.

## Examples

### Example 1 — Heat removed from a cooling plate

[Run: heisler-transient]

**Expected (approx.):** for the plane wall at \`Bi = 3.33\`, \`Fo = 0.225\`,
\`heisler_q ≈ 0.33\` (about a third of the removable heat has left).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`geom$\` | String | Yes | Geometry: \`'wall'\`, \`'cylinder'\`, or \`'sphere'\`. |
| \`Bi\` | Number | Yes | Biot number \`h·s/k\`. |
| \`Fo\` | Number | Yes | Fourier number \`α·t/s²\` (one-term valid for \`Fo > 0.2\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`ratio\` | Number | Heat fraction Q/Q0 ∈ [0, 1]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_GEOMETRY\` | \`geom$\` not recognized | Use \`'wall'\`, \`'cylinder'\`, or \`'sphere'\`. |
| (inaccurate result) | \`Fo < 0.2\` | The one-term approximation is invalid very early in the transient. |`,
  },
  {
    name: `heisler_temp`,
    slug: `heisler_temp`,
    category: `Heat Transfer`,
    summary: `One-term (Heisler) transient temperature ratio for a wall, cylinder, or sphere.`,
    related: [`heisler_q`],
    examples: [`heisler-transient`],
    tags: [`transient conduction`, `heisler`, `biot`, `fourier`, `one-term`, `unsteady`],
    references: [],
    guides: [`chemistry`],
    body: `Returns the **dimensionless temperature** \`θ* = (T − T∞)/(Ti − T∞)\` at a point in a
plane wall, infinite cylinder, or sphere undergoing 1-D transient conduction with
surface convection — the one-term (Heisler) approximation, valid for Fourier number
\`Fo > 0.2\`. Use it when the Biot number is large enough that lumped capacitance
fails and internal gradients matter.

## Syntax

\`\`\`
theta = heisler_temp(geom$, Bi, Fo, xstar)
\`\`\`

## Description

\`geom$\` selects the geometry (\`'wall'\`, \`'cylinder'\`, \`'sphere'\`); \`Bi = h·s/k\` and
\`Fo = α·t/s²\` use the characteristic length \`s\` (half-thickness \`L\` for a wall,
radius \`r0\` for a cylinder/sphere). \`xstar\` is the dimensionless position (\`0\` =
centre/midplane, \`1\` = surface). Recover the temperature with
\`T = T∞ + θ*·(Ti − T∞)\`.

## Mathematical Formulation

Midplane/centre temperature (\`Fo > 0.2\`):

$$ \\theta_0^* = C_1\\,\\exp\\!\\left(-\\lambda_1^2\\,Fo\\right) $$

Position correction \`θ*/θ_0*\`:

$$ \\text{wall: }\\cos\\!\\left(\\lambda_1 x^*\\right),\\quad \\text{cylinder: }J_0\\!\\left(\\lambda_1 x^*\\right),\\quad \\text{sphere: }\\frac{\\sin(\\lambda_1 x^*)}{\\lambda_1 x^*} $$

where $\\lambda_1(Bi)$ and $C_1(Bi)$ are the first-eigenvalue coefficients for the
geometry, with $Bi = hs/k$ and $Fo = \\alpha t/s^2$.

> **Method:** first-term series truncation; the eigenvalue \`λ1\` and coefficient
> \`C1\` are evaluated for \`Bi\` and the selected geometry.

## Examples

### Example 1 — Centre and surface temperature of a cooling plate

A plane wall (\`Bi = 3.33\`, \`Fo = 0.225\`) cooling from 200 °C into a 25 °C stream.

[Run: heisler-transient]

**Expected (approx.):** \`θ_c ≈ 0.87\` → \`T_centre ≈ 177 °C\`; \`θ_s ≈ 0.30\` →
\`T_surface ≈ 77 °C\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`geom$\` | String | Yes | Geometry: \`'wall'\`, \`'cylinder'\`, or \`'sphere'\`. |
| \`Bi\` | Number | Yes | Biot number \`h·s/k\`. |
| \`Fo\` | Number | Yes | Fourier number \`α·t/s²\` (one-term valid for \`Fo > 0.2\`). |
| \`xstar\` | Number | Yes | Dimensionless position: \`0\` centre, \`1\` surface. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`theta\` | Number | Dimensionless temperature θ* = (T − T∞)/(Ti − T∞). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_GEOMETRY\` | \`geom$\` not recognized | Use \`'wall'\`, \`'cylinder'\`, or \`'sphere'\`. |
| (inaccurate result) | \`Fo < 0.2\` | The one-term approximation is invalid early in the transient; the centre has barely responded. |`,
  },
  {
    name: `htc_1phase`,
    slug: `htc_1phase`,
    category: `Heat Transfer`,
    summary: `Single-phase in-tube heat-transfer coefficient (Gnielinski / laminar).`,
    related: [`htc_evap`, `htc_cond`, `ua_hx`],
    examples: [`ev-thermal-management`],
    tags: [`heat transfer`, `convection`, `gnielinski`, `nusselt`, `single phase`, `tube`, `film coefficient`],
    references: [],
    guides: [],
    body: `Returns the **single-phase convective heat-transfer coefficient** \`h\` [W/m²·K] for
a fluid flowing in a tube/channel, from the fluid state and the flow geometry.
Turbulent flow uses the **Gnielinski** correlation; laminar flow falls back to the
constant-Nusselt limit. Use it for coolant/water/oil, refrigerant liquid lines, or
internal air — i.e. the single-phase side of a heat exchanger.

## Syntax

\`\`\`
h = htc_1phase(fluid$, P, T, mdot, Dh, Aflow)
\`\`\`

## Description

The function evaluates the fluid properties at \`(P, T)\`, forms the Reynolds and
Prandtl numbers from the mass flow and hydraulic diameter, applies the
turbulent/laminar Nusselt correlation, and returns \`h = Nu·k/Dh\`.

## Mathematical Formulation

With $Re = \\dot m\\,D_h/(A_{\\text{flow}}\\,\\mu)$ and Darcy factor $f$, the Gnielinski
turbulent Nusselt number is

$$ Nu = \\frac{(f/8)(Re-1000)\\,Pr}{1 + 12.7\\sqrt{f/8}\\,\\big(Pr^{2/3}-1\\big)} $$

valid for $3000 \\lesssim Re \\lesssim 5\\times10^6$; laminar flow uses the
fully-developed constant-Nu limit. Then

$$ h = \\frac{Nu\\,k}{D_h} $$

> **Method:** properties at \`(P, T)\` → \`Re\`, \`Pr\` → Gnielinski (turbulent) or
> constant-\`Nu\` (laminar) → \`h = Nu·k/Dh\`.

## Examples

### Example 1 — Coolant-side film in a chiller

[Run: ev-thermal-management]

**Expected:** a single-phase liquid film coefficient in the ~hundreds–few-thousand
W/m²·K range, well below the boiling/condensing refrigerant side.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`Dh\` | Number | Yes | Hydraulic diameter [m]. |
| \`Aflow\` | Number | Yes | Free-flow cross-sectional area [m²]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`h\` | Number | Convective heat-transfer coefficient [W/m²·K]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_FLUID\` | \`fluid$\` not resolvable | Use a supported fluid name. |
| \`DOMAIN_ERROR\` | \`Dh\` or \`Aflow\` ≤ 0 | Provide positive geometry. |`,
  },
  {
    name: `htc_cond`,
    slug: `htc_cond`,
    category: `Heat Transfer`,
    summary: `In-tube condensation heat-transfer coefficient — Shah correlation.`,
    related: [`htc_evap`, `htc_1phase`, `dp_2phase`],
    examples: [`ev-thermal-management`],
    tags: [`heat transfer`, `condensation`, `two-phase`, `shah`, `refrigerant`, `film coefficient`],
    references: [],
    guides: [],
    body: `Returns the **in-tube condensation heat-transfer coefficient** \`h\` [W/m²·K] for a
condensing two-phase refrigerant at quality \`x\`, using the **Shah**
correlation. Use it for the refrigerant side of a condenser or gas cooler.

## Syntax

\`\`\`
h = htc_cond(fluid$, P, x, mdot, Dh, Aflow)
\`\`\`

## Description

Condensation augments the liquid-only coefficient through the thinning liquid film
and vapor shear; Shah's correlation expresses this as a reduced-pressure and
quality-dependent enhancement.

## Mathematical Formulation

With the liquid-only coefficient $h_l$ (Dittus–Boelter), reduced pressure $p_r$,
and $Z = \\big(\\tfrac{1}{x}-1\\big)^{0.8}p_r^{0.4}$,

$$ h_{TP} = h_l\\left(1 + \\frac{3.8}{Z^{0.95}}\\right) $$

> **Method:** liquid-only \`h_l\` → Shah enhancement from \`Z(x, p_r)\` → \`h_TP\` at the
> local quality.

## Examples

### Example 1 — Condenser refrigerant-side film

[Run: ev-thermal-management]

**Expected:** a condensing film coefficient well above the single-phase liquid
value, decreasing as quality falls toward the subcooled liquid outlet.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Refrigerant name. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`Dh\` | Number | Yes | Hydraulic diameter [m]. |
| \`Aflow\` | Number | Yes | Free-flow area [m²]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`h\` | Number | Two-phase condensation coefficient [W/m²·K]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x\` outside [0, 1] | Quality must be a mass fraction in [0, 1]. |`,
  },
  {
    name: `htc_evap`,
    slug: `htc_evap`,
    category: `Heat Transfer`,
    summary: `Flow-boiling (evaporation) heat-transfer coefficient — Shah correlation.`,
    related: [`htc_cond`, `htc_1phase`, `dp_2phase`],
    examples: [`ev-thermal-management`],
    tags: [`heat transfer`, `boiling`, `evaporation`, `two-phase`, `shah`, `refrigerant`, `film coefficient`],
    references: [],
    guides: [],
    body: `Returns the **flow-boiling heat-transfer coefficient** \`h\` [W/m²·K] for an
evaporating two-phase refrigerant at quality \`x\`, using the **Shah** correlation.
Use it for the refrigerant side of an evaporator or battery chiller.

## Syntax

\`\`\`
h = htc_evap(fluid$, P, x, mdot, Dh, Aflow)
\`\`\`

## Description

Flow boiling enhances heat transfer above the liquid-only value through convective
and nucleate-boiling mechanisms; the Shah correlation captures this as an
enhancement factor on the liquid-only coefficient.

## Mathematical Formulation

With the liquid-only coefficient $h_l$ (Dittus–Boelter on the liquid fraction) and
the convection number $Co = \\big(\\tfrac{1-x}{x}\\big)^{0.8}(\\rho_g/\\rho_l)^{0.5}$,
the Shah convective-boiling enhancement is

$$ h_{TP} = h_l\\,F_{cb}, \\qquad F_{cb} = \\frac{1.8}{Co^{0.8}} $$

with the nucleate-boiling branch taken where it dominates.

> **Method:** liquid-only \`h_l\` → Shah enhancement from \`Co\` (and boiling number) →
> \`h_TP\`, evaluated at the local quality \`x\`.

## Examples

### Example 1 — Evaporator refrigerant-side film

[Run: ev-thermal-management]

**Expected:** a boiling film coefficient several times the single-phase liquid
value — the high-conductance side of the chiller.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Refrigerant name. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`Dh\` | Number | Yes | Hydraulic diameter [m]. |
| \`Aflow\` | Number | Yes | Free-flow area [m²]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`h\` | Number | Two-phase boiling coefficient [W/m²·K]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x\` outside [0, 1] | Quality must be a mass fraction in [0, 1]. |`,
  },
  {
    name: `htc_extair`,
    slug: `htc_extair`,
    category: `Heat Transfer`,
    summary: `External air-side heat-transfer coefficient — Zukauskas tube-bank cross-flow.`,
    related: [`htc_1phase`, `ua_hx`, `hx_eta_surf`],
    examples: [`ev-thermal-management`],
    tags: [`heat transfer`, `air side`, `zukauskas`, `tube bank`, `cross-flow`, `nusselt`, `film coefficient`],
    references: [],
    guides: [],
    body: `Returns the **external air/gas-side heat-transfer coefficient** \`h\` [W/m²·K] for
cross-flow over a finned-tube bank, using the **Žukauskas** correlation. Use it for
the air side of a radiator, condenser, or cabin evaporator.

## Syntax

\`\`\`
h = htc_extair(fluid$, P, T, mdot, D, Aflow)
\`\`\`

## Description

The air-side film is usually the controlling resistance of an automotive heat
exchanger. The Žukauskas correlation gives the bank-averaged Nusselt number from
the maximum-velocity Reynolds number and Prandtl number, with a wall-property
correction.

## Mathematical Formulation

With $Re_{d,\\max} = \\rho V_{\\max} D/\\mu$ at the minimum free-flow area,

$$ Nu_d = C\\,Re_{d,\\max}^{\\,n}\\,Pr^{0.36}\\left(\\frac{Pr}{Pr_w}\\right)^{1/4} $$

where \`C\` and \`n\` depend on the bank arrangement (in-line/staggered) and Reynolds
band; then $h = Nu_d\\,k/D$.

> **Method:** properties at \`(P, T)\` → \`Re_{d,max}\`, \`Pr\` → Žukauskas \`Nu\` →
> \`h = Nu·k/D\`.

## Examples

### Example 1 — Radiator/condenser air-side film

[Run: ev-thermal-management]

**Expected:** an air-side film coefficient of order tens–low-hundreds W/m²·K — the
weak side that sets the overall \`UA\`, which is why it is finned.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Gas name (typically air). |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`D\` | Number | Yes | Tube outer diameter [m]. |
| \`Aflow\` | Number | Yes | Minimum free-flow area [m²]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`h\` | Number | Air-side coefficient [W/m²·K]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`D\` or \`Aflow\` ≤ 0 | Provide positive geometry. |`,
  },
  {
    name: `hx_aconv`,
    slug: `hx_aconv`,
    category: `Heat Transfer`,
    summary: `Convective surface area of a compact heat-exchanger core from its geometry.`,
    related: [`hx_dh`, `hx_eta_surf`, `ua_hx`],
    examples: [`ev-thermal-management`],
    tags: [`heat exchanger`, `geometry`, `convective area`, `hydraulic diameter`, `compact core`],
    references: [],
    guides: [],
    body: `Returns the **convective (wetted) surface area** \`A\` [m²] of one side of a compact
heat-exchanger core from its free-flow area, flow length, and hydraulic diameter —
the area that enters \`UA = h·A\`.

## Syntax

\`\`\`
A = hx_aconv(Aflow, L, Dh)
\`\`\`

## Description

A compact core is characterized by its hydraulic diameter
\`Dh = 4·Aflow·L/A_total\`; inverting that definition gives the surface area for a
known free-flow area and flow length.

## Mathematical Formulation

$$ A = \\frac{4\\,A_{\\text{flow}}\\,L}{D_h} $$

the inverse of the hydraulic-diameter definition
$D_h = 4 A_{\\text{flow}} L / A$.

> **Method:** direct evaluation from the compact-core geometry.

## Examples

### Example 1 — Core convective area for a UA estimate

[Run: ev-thermal-management]

**Expected:** the wetted area used with the side's film coefficient to build \`UA\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Aflow\` | Number | Yes | Free-flow (minimum) area [m²]. |
| \`L\` | Number | Yes | Flow length [m]. |
| \`Dh\` | Number | Yes | Hydraulic diameter [m]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`A\` | Number | Convective surface area [m²]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`Dh ≤ 0\` | Hydraulic diameter must be positive. |`,
  },
  {
    name: `hx_area_direct`,
    slug: `hx_area_direct`,
    category: `Heat Transfer`,
    summary: `Primary (bare tube-wall) surface area of a fin-and-tube heat exchanger.`,
    related: [`hx_area_indirect`, `hx_eta_surf`, `ua_hx`],
    examples: [`ev-thermal-management`],
    tags: [`heat exchanger`, `geometry`, `primary area`, `tube wall`, `fin-and-tube`],
    references: [],
    guides: [],
    body: `Returns the **primary surface area** [m²] — the exposed bare tube-wall area — of a
fin-and-tube heat-exchanger air side, from the core width, tube count, tube height,
core depth, and fin thickness. With \`hx_area_indirect\` (the fin
area) it gives the area split used by \`hx_eta_surf\`.

## Syntax

\`\`\`
A_primary = hx_area_direct(W, tubeCount, Htube, depth, t)
\`\`\`

## Description

The primary area is the tube outer wall exposed to the air stream — i.e. the tube
surface minus the footprint occupied by the fins. It is the fully-effective part of
the extended surface (efficiency 1).

## Mathematical Formulation

The exposed tube-wall area over all tubes, net of the fin-occupied fraction:

$$ A_{\\text{primary}} = f\\big(W,\\,\\text{tubeCount},\\,H_{\\text{tube}},\\,\\text{depth},\\,t\\big) $$

(a fin-and-tube geometric construction).

> **Method:** direct geometric evaluation of the bare tube-wall area.

## Examples

### Example 1 — Primary area of an air-side core

[Run: ev-thermal-management]

**Expected:** the bare tube-wall area, the smaller (fully-effective) part of the
air-side \`A_total = A_primary + A_fin\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`W\` | Number | Yes | Core width [m]. |
| \`tubeCount\` | Number | Yes | Number of tubes. |
| \`Htube\` | Number | Yes | Tube height/spacing [m]. |
| \`depth\` | Number | Yes | Core depth [m]. |
| \`t\` | Number | Yes | Fin thickness [m]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`A_primary\` | Number | Primary (tube-wall) area [m²]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | A geometry input ≤ 0 | All dimensions must be positive. |`,
  },
  {
    name: `hx_area_indirect`,
    slug: `hx_area_indirect`,
    category: `Heat Transfer`,
    summary: `Secondary (fin) surface area of a fin-and-tube heat exchanger.`,
    related: [`hx_area_direct`, `hx_fin_len`, `hx_eta_surf`],
    examples: [`ev-thermal-management`],
    tags: [`heat exchanger`, `geometry`, `secondary area`, `fin area`, `fin-and-tube`],
    references: [],
    guides: [],
    body: `Returns the **secondary surface area** [m²] — the total fin area — of a fin-and-tube
heat-exchanger air side, from the core width, tube count, and developed fin length.
With \`hx_area_direct\` it gives the primary/secondary area split
that \`hx_eta_surf\` weights by efficiency.

## Syntax

\`\`\`
A_fin = hx_area_indirect(W, tubeCount, finLen)
\`\`\`

## Description

The secondary (fin) area is the extended surface that operates below the base
temperature at efficiency \`eta_fin < 1\`. It typically dominates the total air-side
area, which is why the overall surface efficiency matters.

## Mathematical Formulation

The total fin area is the developed fin length (from
\`hx_fin_len\`) summed over the fins across the core width and tube
count (both fin faces):

$$ A_{\\text{fin}} = f\\big(W,\\,\\text{tubeCount},\\,\\text{finLen}\\big) $$

(a fin-and-tube geometric construction).

> **Method:** direct geometric evaluation of the total fin area.

## Examples

### Example 1 — Fin area of an air-side core

[Run: ev-thermal-management]

**Expected:** the secondary area, usually the larger part of the air-side
\`A_total = A_primary + A_fin\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`W\` | Number | Yes | Core width [m]. |
| \`tubeCount\` | Number | Yes | Number of tubes. |
| \`finLen\` | Number | Yes | Developed fin length [m] (from \`hx_fin_len\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`A_fin\` | Number | Secondary (fin) area [m²]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | A geometry input ≤ 0 | All dimensions must be positive. |`,
  },
  {
    name: `hx_dh`,
    slug: `hx_dh`,
    category: `Heat Transfer`,
    summary: `GEOMETRY: hydraulic diameter D_h=4*Aflow*L/Atotal [m] of a compact HX core (any side)`,
    related: [],
    examples: [],
    tags: [`hx`, `dh`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `GEOMETRY: hydraulic diameter D_h=4*Aflow*L/Atotal [m] of a compact HX core (any side)


## Syntax

\`\`\`
hx_dh(Aflow, Atotal, L)
\`\`\`

## Description

GEOMETRY: hydraulic diameter D_h=4*Aflow*L/Atotal [m] of a compact HX core (any side)

## Mathematical Formulation

$$ D_h = \\frac{4\\,A_{\\text{flow}}\\,L}{A_{\\text{total}}} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Aflow\` | Number | Yes | Free-flow (minimum) cross-sectional area [m²]. |
| \`Atotal\` | Number | Yes | Total convective surface area [m²]. |
| \`L\` | Number | Yes | Length [m]. |`,
  },
  {
    name: `hx_effectiveness`,
    slug: `hx_effectiveness`,
    category: `Heat Transfer`,
    summary: `Heat-exchanger effectiveness ε(NTU, Cr) for a given flow arrangement.`,
    related: [`hx_NTU`, `LMTD`, `fin_efficiency`],
    examples: [`hx-effectiveness-ntu`],
    tags: [`heat exchanger`, `effectiveness`, `ntu`, `epsilon`, `counterflow`, `parallelflow`, `crossflow`, `shell and tube`, `evaporator`, `condenser`],
    references: [],
    guides: [],
    body: `Returns the **effectiveness** \`ε\` of a heat exchanger — the fraction of the
thermodynamically maximum heat duty actually transferred — from its \`NTU\`, capacity
ratio \`Cr\`, and flow arrangement. Use it to **rate** an exchanger (find the duty and
outlet temperatures) when \`UA\` is known but the outlet states are not, avoiding the
iteration the LMTD method would require.

## Syntax

\`\`\`
eps = hx_effectiveness(type$, NTU, Cr)
\`\`\`

## Description

The effectiveness–NTU method characterizes an exchanger by three dimensionless groups
and a closed-form \`ε(NTU, Cr)\` relation per flow arrangement, so the duty follows
directly from the inlet states without solving for the (unknown) outlet temperatures
first. \`type$\` selects the arrangement — \`'counterflow'\`, \`'parallelflow'\`, the three
**crossflow** variants (both fluids unmixed, or one of the two mixed), and a 1-shell-pass
**shell-and-tube** — and any \`type$\` collapses to the same \`ε = 1 - e^{-NTU}\` boiling/
condensing limit when \`Cr = 0\` (one stream isothermal). Pair it with \`hx_NTU\`
(the inverse, for sizing) and \`LMTD\`.

## Mathematical Formulation

With heat-capacity rates $C = \\dot m\\,c_p$, define $C_{min}=\\min(C_h,C_c)$,
$C_{max}=\\max(C_h,C_c)$, and

$$ \\varepsilon = \\frac{Q}{Q_{max}}, \\qquad NTU = \\frac{UA}{C_{min}}, \\qquad C_r = \\frac{C_{min}}{C_{max}} $$

where $Q_{max}=C_{min}(T_{h,in}-T_{c,in})$ is the duty of an infinite-area counterflow
exchanger.

**Counterflow**, with the $C_r = 1$ limit:

$$ \\varepsilon = \\frac{1-\\exp\\!\\big[-NTU\\,(1-C_r)\\big]}{1-C_r\\,\\exp\\!\\big[-NTU\\,(1-C_r)\\big]}, \\qquad \\varepsilon = \\frac{NTU}{1+NTU}\\ \\text{ when } C_r = 1 $$

(the $C_r = 1$ form is the removable limit of the general relation.)

**Parallel flow**:

$$ \\varepsilon = \\frac{1-\\exp\\!\\big[-NTU\\,(1+C_r)\\big]}{1+C_r} $$

**Crossflow, both fluids unmixed** — standard approximate correlation; the exact
analytic solution has no closed form (only an infinite-series solution exists). The
approximation is widely tabulated in the heat-exchanger literature:

$$ \\varepsilon = 1 - \\exp\\!\\left[\\frac{NTU^{0.22}}{C_r}\\Big(e^{-C_r\\,NTU^{0.78}}-1\\Big)\\right] $$

**Crossflow, $C_{max}$ mixed / $C_{min}$ unmixed**:

$$ \\varepsilon = \\frac{1}{C_r}\\Big(1-\\exp\\!\\big[-C_r\\,(1-e^{-NTU})\\big]\\Big) $$

**Crossflow, $C_{min}$ mixed / $C_{max}$ unmixed**:

$$ \\varepsilon = 1 - \\exp\\!\\left[-\\frac{1}{C_r}\\big(1-e^{-C_r\\,NTU}\\big)\\right] $$

**Shell-and-tube**, one shell pass and 2, 4, … tube passes (parallel-counterflow,
shell fluid mixed; TEMA E 1-2), with $\\Lambda = \\sqrt{1+C_r^{2}}$:

$$ \\varepsilon = 2\\left[\\,1 + C_r + \\Lambda\\,\\frac{1+e^{-NTU\\,\\Lambda}}{1-e^{-NTU\\,\\Lambda}}\\,\\right]^{-1} $$

**Condenser / evaporator limit** $C_r\\to 0$ (one stream isothermal; arrangement-independent):

$$ \\varepsilon = 1 - e^{-NTU} $$

> **Method:** direct evaluation of the closed-form relation for the selected
> arrangement; the $C_r\\to 0$ form is used as the limit for condensers/evaporators,
> and the $C_r = 1$ counterflow form is taken as the removable limit.

## Examples

### Example 1 — Counterflow water-to-water exchanger (rating)

A counterflow exchanger with \`UA = 12 kW/K\` heats 1.5 kg/s of water against 2.0 kg/s of
hot water — solve for the duty and both outlet temperatures.

[Run: hx-effectiveness-ntu]

**Expected:** \`Cr = 0.75\`, \`NTU ≈ 1.91\`, \`ε ≈ 0.711\`, \`Q ≈ 312 kW\`,
\`Th_out ≈ 323 K\`, \`Tc_out ≈ 340 K\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`type$\` | String | Yes | Flow arrangement (case/space/punctuation-insensitive). See the table below for accepted values. A condenser/evaporator is the \`Cr = 0\` case of any arrangement. |
| \`NTU\` | Number | Yes | Number of transfer units, \`UA/C_min\` (dimensionless, ≥ 0). |
| \`Cr\` | Number | Yes | Capacity ratio \`C_min/C_max\` ∈ [0, 1]. Use \`0\` for a condenser/evaporator. |

**Accepted \`type$\` values** (matching ignores case, spaces and punctuation):

| Arrangement | Canonical | Aliases |
| --- | --- | --- |
| Counterflow | \`'counterflow'\` | \`counter\`, \`countercurrent\` |
| Parallel flow | \`'parallelflow'\` | \`parallel\`, \`cocurrent\`, \`coflow\` |
| Crossflow, both unmixed | \`'crossflow_both_unmixed'\` | \`crossflow\`, \`bothunmixed\`, \`crossflowunmixed\` |
| Crossflow, C_max mixed (C_min unmixed) | \`'crossflow_cmax_mixed'\` | \`cmaxmixed\`, \`crossflowcminunmixed\` |
| Crossflow, C_min mixed (C_max unmixed) | \`'crossflow_cmin_mixed'\` | \`cminmixed\`, \`crossflowcmaxunmixed\` |
| Shell-and-tube (1 shell pass; 2, 4, … tube passes) | \`'shell&tube'\` | \`shell\`, \`shelltube\`, \`shellandtube1\` |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`eps\` | Number | Effectiveness ε ∈ [0, 1]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_HX_TYPE\` | \`type$\` is not a recognized arrangement | Use one of \`'counterflow'\`, \`'parallelflow'\`, \`'crossflow_both_unmixed'\`, \`'crossflow_cmax_mixed'\`, \`'crossflow_cmin_mixed'\`, \`'shell&tube'\`; pass \`Cr = 0\` for a condenser/evaporator. |
| \`DOMAIN_ERROR\` | \`Cr\` outside [0, 1] or \`NTU < 0\` | Ensure \`C_min = min(C_h, C_c)\` so \`Cr ≤ 1\`; check \`UA\` and flow rates. |`,
  },
  {
    name: `hx_eta_surf`,
    slug: `hx_eta_surf`,
    category: `Heat Transfer`,
    summary: `Overall surface (fin) efficiency of an extended-surface heat-exchanger side.`,
    related: [`fin_efficiency`, `ua_hx`, `htc_extair`],
    examples: [`ev-thermal-management`],
    tags: [`heat exchanger`, `fin`, `overall surface efficiency`, `extended surface`, `compact`],
    references: [],
    guides: [],
    body: `Returns the **overall surface efficiency** \`η_o\` of a finned heat-exchanger side —
the area-weighted blend of the fully-effective primary (bare) area and the
less-effective fin area. Multiply the film coefficient by \`η_o\` (or use
\`η_o·h·A\`) when forming \`UA\` for an extended surface.

## Syntax

\`\`\`
eta_o = hx_eta_surf(Afin, Atotal, eta_fin)
\`\`\`

## Description

On a finned surface, the primary tube wall sits at the base temperature
(efficiency 1) while the fins droop toward the fluid temperature (efficiency
\`eta_fin\` < 1, from \`fin_efficiency\`). The overall efficiency
weights the two by their area shares.

## Mathematical Formulation

$$ \\eta_o = 1 - \\frac{A_{\\text{fin}}}{A_{\\text{total}}}\\big(1 - \\eta_{\\text{fin}}\\big) $$

> **Method:** direct evaluation; \`η_o = 1\` for an unfinned surface
> (\`A_fin = 0\`), and \`η_o → η_fin\` as the fins dominate the area.

## Examples

### Example 1 — Air-side overall efficiency of a finned core

The EV thermal-management sizing forms \`η_o\` from the fin area share and
\`fin_efficiency(mL)\` before computing \`UA = η_o·h·A\`.

[Run: ev-thermal-management]

**Expected:** \`η_o\` between the bare value (1) and the fin efficiency — typically
0.7–0.95 for an automotive finned core.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Afin\` | Number | Yes | Fin (secondary) surface area [m²]. |
| \`Atotal\` | Number | Yes | Total surface area (primary + fin) [m²]. |
| \`eta_fin\` | Number | Yes | Single-fin efficiency (from \`fin_efficiency\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`eta_o\` | Number | Overall surface efficiency η_o ∈ (0, 1]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`Afin > Atotal\` | The fin area cannot exceed the total area. |`,
  },
  {
    name: `hx_fin_len`,
    slug: `hx_fin_len`,
    category: `Heat Transfer`,
    summary: `Developed fin length of a fin-and-tube heat-exchanger air side.`,
    related: [`hx_area_indirect`, `hx_eta_surf`, `fin_efficiency`],
    examples: [`ev-thermal-management`],
    tags: [`heat exchanger`, `geometry`, `fin length`, `fin-and-tube`, `air side`],
    references: [],
    guides: [],
    body: `Returns the **developed fin length** [m] of the finned air side of a fin-and-tube
heat exchanger from the core depth, fin thickness, fin density, and tube height —
a geometry helper feeding the fin area and efficiency.

## Syntax

\`\`\`
finLen = hx_fin_len(depth, t, finDensity, Htube)
\`\`\`

## Description

The developed (unrolled) length of the fin material between adjacent tubes, derived
from the core depth, the fin spacing implied by \`finDensity\`, and the tube-to-tube
gap \`Htube\`. It is the conduction length \`L\` used in the fin parameter
\`mL = sqrt(2h/(k·t))·finLen\`.

## Mathematical Formulation

The developed fin length combines the core depth and the inter-tube span at the
given fin pitch (\`1/finDensity\`) and thickness \`t\`:

$$ \\text{finLen} = f\\big(\\text{depth},\\,t,\\,\\text{finDensity},\\,H_{\\text{tube}}\\big) $$

(a fin-and-tube geometric construction).

> **Method:** direct geometric evaluation of the developed fin path between tubes.

## Examples

### Example 1 — Fin length for the air-side fin parameter

[Run: ev-thermal-management]

**Expected:** the developed length that, with the fin thickness and conductivity,
sets \`mL\` and hence \`fin_efficiency(mL)\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`depth\` | Number | Yes | Core depth in the airflow direction [m]. |
| \`t\` | Number | Yes | Fin thickness [m]. |
| \`finDensity\` | Number | Yes | Fins per unit length [1/m]. |
| \`Htube\` | Number | Yes | Tube-to-tube vertical gap [m]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`finLen\` | Number | Developed fin length [m]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | A geometry input ≤ 0 | All dimensions must be positive. |`,
  },
  {
    name: `hx_NTU`,
    slug: `hx_ntu`,
    category: `Heat Transfer`,
    summary: `Number of transfer units NTU(ε, Cr) — the inverse of hx_effectiveness.`,
    related: [`hx_effectiveness`, `LMTD`],
    examples: [`hx-effectiveness-ntu`],
    tags: [`heat exchanger`, `ntu`, `effectiveness`, `sizing`, `counterflow`],
    references: [],
    guides: [],
    body: `Returns the **number of transfer units** \`NTU\` required to achieve a target
effectiveness \`ε\` at capacity ratio \`Cr\` for a given flow arrangement — the
inverse of \`hx_effectiveness\`. Use it to **size** an exchanger:
from a required duty you back out \`ε\`, then \`NTU\`, then \`UA = NTU·C_min\`.

## Syntax

\`\`\`
NTU = hx_NTU(type$, eps, Cr)
\`\`\`

## Description

Rating asks "what duty for this \`UA\`?" (effectiveness from NTU); sizing asks the
reverse — "what \`UA\` for this duty?". \`hx_NTU\` closes the sizing loop by inverting
the \`ε\`(NTU, Cr) relation analytically per arrangement.

## Mathematical Formulation

With $NTU = UA/C_{min}$ and $C_r = C_{min}/C_{max}$:

**Counterflow** ($C_r < 1$):

$$ NTU = \\frac{1}{C_r - 1}\\,\\ln\\!\\left(\\frac{\\varepsilon - 1}{\\varepsilon\\,C_r - 1}\\right) $$

**Counterflow, balanced** ($C_r = 1$):

$$ NTU = \\frac{\\varepsilon}{1 - \\varepsilon} $$

**Condenser / evaporator limit** ($C_r \\to 0$; arrangement-independent):

$$ NTU = -\\ln(1 - \\varepsilon) $$

> **Method:** direct evaluation of the inverse relation for the selected
> arrangement; the $C_r \\to 0$ form is used for condensers/evaporators.

## Examples

### Example 1 — NTU of a counterflow exchanger

The effectiveness–NTU rating example forms \`NTU = UA/C_min\` directly; \`hx_NTU\`
performs the reverse map \`ε → NTU\` for the same arrangement when sizing to a
target effectiveness.

[Run: hx-effectiveness-ntu]

**Expected:** for \`ε ≈ 0.711\`, \`Cr = 0.75\`, \`hx_NTU('counterflow', 0.711, 0.75) ≈ 1.91\`
(recovering the example's \`NTU = UA/C_min\`).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`type$\` | String | Yes | Flow arrangement: \`'counterflow'\`, \`'parallel'\`. A condenser/evaporator is the \`Cr = 0\` case. |
| \`eps\` | Number | Yes | Target effectiveness ε ∈ [0, 1). Must satisfy \`ε < 1/(1+Cr)\` for parallel flow. |
| \`Cr\` | Number | Yes | Capacity ratio \`C_min/C_max\` ∈ [0, 1]. Use \`0\` for a condenser/evaporator. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`NTU\` | Number | Number of transfer units (≥ 0); \`UA = NTU·C_min\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`INFEASIBLE_EFFECTIVENESS\` | \`ε\` at or above the arrangement's ceiling (e.g. \`ε ≥ 1/(1+Cr)\` for parallel flow) | No finite \`NTU\` reaches it — lower the target or switch to counterflow. |
| \`UNKNOWN_HX_TYPE\` | \`type$\` not recognized | Use \`'counterflow'\` or \`'parallel'\`; pass \`Cr = 0\` for a condenser/evaporator. |`,
  },
  {
    name: `hx_sigma`,
    slug: `hx_sigma`,
    category: `Heat Transfer`,
    summary: `GEOMETRY: free-flow (contraction) ratio sigma=Aflow/Afrontal. SIDE: compact HX air/gas face`,
    related: [],
    examples: [],
    tags: [`hx`, `sigma`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `GEOMETRY: free-flow (contraction) ratio sigma=Aflow/Afrontal. SIDE: compact HX air/gas face


## Syntax

\`\`\`
hx_sigma(Aflow, Afrontal)
\`\`\`

## Description

GEOMETRY: free-flow (contraction) ratio sigma=Aflow/Afrontal. SIDE: compact HX air/gas face

## Mathematical Formulation

$$ \\sigma = \\frac{A_{\\text{flow}}}{A_{\\text{frontal}}} \\quad\\text{(free-flow / contraction ratio)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Aflow\` | Number | Yes | Free-flow (minimum) cross-sectional area [m²]. |
| \`Afrontal\` | Number | Yes | Frontal (face) area [m²]. |`,
  },
  {
    name: `j_fin`,
    slug: `j_fin`,
    category: `Heat Transfer`,
    summary: `Colburn j for a compact fin surface (plain|wavy|louvered|offset). SIDE: air/gas finned side. HX: plate-fin/louvered/offset-strip radiator, condenser, CAC`,
    related: [],
    examples: [],
    tags: [`fin`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Colburn j for a compact fin surface (plain|wavy|louvered|offset). SIDE: air/gas finned side. HX: plate-fin/louvered/offset-strip radiator, condenser, CAC


## Syntax

\`\`\`
j_fin(surface$, Re)
\`\`\`

## Description

Colburn j for a compact fin surface (plain|wavy|louvered|offset). SIDE: air/gas finned side. HX: plate-fin/louvered/offset-strip radiator, condenser, CAC

## Mathematical Formulation

$$ j = St\\,Pr^{2/3} = C\\,Re^{m} \\quad\\text{(Colburn } j \\text{ for the fin surface)} $$

## Applicability

- **Where it applies:** The air/gas finned side of a compact surface.
- **Valid when:** Plain / wavy / louvered / offset-strip fin surfaces (\`surface$\`).
- **How it's used:** Air-side \`h\` via the Colburn \`j\`-factor; pair with \`f_fin\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`surface$\` | String | Yes | Selector — One of \`plain\`, \`wavy\`, \`louvered\`, \`offset\`. |
| \`Re\` | Number | Yes | Reynolds number. |`,
  },
  {
    name: `LMTD`,
    slug: `lmtd`,
    category: `Heat Transfer`,
    summary: `Log-mean temperature difference of a heat exchanger.`,
    related: [`hx_effectiveness`, `hx_NTU`],
    examples: [`hx-effectiveness-ntu`],
    tags: [`heat exchanger`, `lmtd`, `log-mean`, `temperature difference`, `duty`],
    references: [],
    guides: [],
    body: `Returns the **log-mean temperature difference** between two streams from their
terminal temperature differences \`dT1\` and \`dT2\`. It is the correct mean driving
temperature for the heat-exchanger rate equation \`Q = U·A·F·ΔT_lm\` — use it when
the inlet and outlet temperatures are known and you need the duty or the required
\`UA\`.

## Syntax

\`\`\`
dTlm = LMTD(dT1, dT2)
\`\`\`

## Description

For an exchanger, the local temperature difference varies along the flow path, so
the duty is driven not by an arithmetic mean but by the *log-mean* of the two
terminal differences. \`dT1\` and \`dT2\` are the hot-minus-cold temperature
differences at the two ends. The result feeds the rate equation with an overall
conductance \`UA\` and a configuration correction factor \`F\` (1 for pure
counter/parallel flow).

## Mathematical Formulation

$$ \\Delta T_{lm} = \\frac{\\Delta T_1 - \\Delta T_2}{\\ln(\\Delta T_1 / \\Delta T_2)} $$

and the heat-exchanger duty, with overall conductance $UA$ and configuration
correction factor $F$,

$$ Q = U A\\, F\\, \\Delta T_{lm} $$

> **Method:** direct evaluation. As $\\Delta T_1 \\to \\Delta T_2$ the ratio is the
> arithmetic mean (the removable singularity of the log form).

## Examples

### Example 1 — Counterflow exchanger end-difference

After rating a counterflow water-to-water exchanger by effectiveness–NTU, the
log-mean of its two end temperature differences gives the mean driving ΔT.

[Run: hx-effectiveness-ntu]

**Expected:** with \`dT1 = Th_in − Tc_out ≈ 20.3 K\` and \`dT2 = Th_out − Tc_in ≈ 32.7 K\`,
\`dTlm ≈ 26.0 K\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`dT1\` | Number | Yes | Temperature difference at one end (hot − cold), same sign as \`dT2\`. |
| \`dT2\` | Number | Yes | Temperature difference at the other end. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`dTlm\` | Number | Log-mean temperature difference [K]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`dT1\` and \`dT2\` have opposite signs, or one is zero | Use consistent hot−cold differences; a sign change implies a temperature cross — check the stream arrangement. |`,
  },
  {
    name: `mass_flux`,
    slug: `mass_flux`,
    category: `Heat Transfer`,
    summary: `GEOMETRY/flow: mass flux G=mdot/Aflow [kg/m^2/s] (any side)`,
    related: [],
    examples: [],
    tags: [`mass`, `flux`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `GEOMETRY/flow: mass flux G=mdot/Aflow [kg/m^2/s] (any side)


## Syntax

\`\`\`
mass_flux(mdot, Aflow)
\`\`\`

## Description

GEOMETRY/flow: mass flux G=mdot/Aflow [kg/m^2/s] (any side)

## Mathematical Formulation

$$ G = \\frac{\\dot m}{A_{\\text{flow}}} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`mdot\` | Number | Yes | Mass flow rate [kg/s]. |
| \`Aflow\` | Number | Yes | Free-flow (minimum) cross-sectional area [m²]. |`,
  },
  {
    name: `nu_blend`,
    slug: `nu_blend`,
    category: `Heat Transfer`,
    summary: `Cubic free+forced blend (Nu1^3+Nu2^3)^(1/3). USE: combine natural + forced Nu on any side`,
    related: [],
    examples: [],
    tags: [`nu`, `blend`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Cubic free+forced blend (Nu1^3+Nu2^3)^(1/3). USE: combine natural + forced Nu on any side


## Syntax

\`\`\`
nu_blend(Nu1, Nu2)
\`\`\`

## Description

Cubic free+forced blend (Nu1^3+Nu2^3)^(1/3). USE: combine natural + forced Nu on any side

## Mathematical Formulation

$$ Nu = \\big(Nu_1^3 + Nu_2^3\\big)^{1/3} \\quad\\text{(free+forced cubic blend)} $$

## Applicability

- **Where it applies:** Any surface with combined natural + forced convection.
- **Valid when:** Mixed convection where neither mechanism dominates.
- **How it's used:** Combines two Nusselt numbers as \`(Nu₁³ + Nu₂³)^{1/3}\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Nu1\` | Number | Yes | First Nusselt number to blend. |
| \`Nu2\` | Number | Yes | Second Nusselt number to blend. |`,
  },
  {
    name: `nu_churchill_chu`,
    slug: `nu_churchill_chu`,
    category: `Heat Transfer`,
    summary: `Nu, free convection from Rayleigh. SIDE: natural convection (still air / quiescent fluid). HX: passive/low-flow surfaces`,
    related: [],
    examples: [],
    tags: [`nu`, `churchill`, `chu`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Nu, free convection from Rayleigh. SIDE: natural convection (still air / quiescent fluid). HX: passive/low-flow surfaces


## Syntax

\`\`\`
nu_churchill_chu(Ra, Pr)
\`\`\`

## Description

Nu, free convection from Rayleigh. SIDE: natural convection (still air / quiescent fluid). HX: passive/low-flow surfaces

## Mathematical Formulation

$$ Nu = \\left\\{0.60 + \\frac{0.387\\,Ra^{1/6}}{[1 + (0.559/Pr)^{9/16}]^{8/27}}\\right\\}^2 \\quad\\text{(Churchill–Chu)} $$

## Applicability

- **Where it applies:** Natural convection from a surface to still air / quiescent fluid.
- **Valid when:** Free (buoyancy-driven) convection, characterized by the Rayleigh number.
- **How it's used:** Passive/low-flow surfaces; blend with a forced-convection \`Nu\` via \`nu_blend\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Ra\` | Number | Yes | Rayleigh number. |
| \`Pr\` | Number | Yes | Prandtl number. |`,
  },
  {
    name: `nu_colburn`,
    slug: `nu_colburn`,
    category: `Heat Transfer`,
    summary: `Nu=j*Re*Pr^(1/3). SIDE: air/gas through a compact finned surface. HX: plate-fin/louvered-fin air side`,
    related: [],
    examples: [],
    tags: [`nu`, `colburn`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Nu=j*Re*Pr^(1/3). SIDE: air/gas through a compact finned surface. HX: plate-fin/louvered-fin air side


## Syntax

\`\`\`
nu_colburn(j, Re, Pr)
\`\`\`

## Description

Nu=j*Re*Pr^(1/3). SIDE: air/gas through a compact finned surface. HX: plate-fin/louvered-fin air side

## Mathematical Formulation

$$ Nu = j\\,Re\\,Pr^{1/3} \\quad\\text{(Colburn } j\\text{-factor)} $$

## Applicability

- **Where it applies:** Air/gas through a compact finned surface.
- **Valid when:** Compact plate-fin / louvered-fin cores, characterized by a Colburn \`j\`-factor.
- **How it's used:** Air-side \`h = j·Re·Pr^{1/3}·k/D_h\`; pair the friction with \`f_fin\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`j\` | Number | Yes | Colburn j-factor. |
| \`Re\` | Number | Yes | Reynolds number. |
| \`Pr\` | Number | Yes | Prandtl number. |`,
  },
  {
    name: `nu_gungor_winterton`,
    slug: `nu_gungor_winterton`,
    category: `Heat Transfer`,
    summary: `Nu, Gungor-Winterton flow boiling from liquid-only Nu. SIDE: boiling two-phase refrigerant. HX: evaporator refrigerant side`,
    related: [],
    examples: [],
    tags: [`nu`, `gungor`, `winterton`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Nu, Gungor-Winterton flow boiling from liquid-only Nu. SIDE: boiling two-phase refrigerant. HX: evaporator refrigerant side


## Syntax

\`\`\`
nu_gungor_winterton(Nu_l, Xtt, Bo)
\`\`\`

## Description

Nu, Gungor-Winterton flow boiling from liquid-only Nu. SIDE: boiling two-phase refrigerant. HX: evaporator refrigerant side

## Mathematical Formulation

$$ Nu = Nu_l\\big[1 + 3000\\,Bo^{0.86} + 1.12(x/(1-x))^{0.75}(\\rho_l/\\rho_g)^{0.41}\\big] \\quad\\text{(Gungor–Winterton)} $$

## Applicability

- **Where it applies:** Boiling two-phase refrigerant in evaporator tubes.
- **Valid when:** Saturated flow boiling, enhancing the liquid-only Nusselt with the boiling number and Martinelli parameter.
- **How it's used:** Evaporator refrigerant-side \`h\`; an alternative to the Chen (\`chen_f\`/\`chen_s\`) and Shah boiling models.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Nu_l\` | Number | Yes | Liquid-only Nusselt number. |
| \`Xtt\` | Number | Yes | Turbulent–turbulent Martinelli parameter. |
| \`Bo\` | Number | Yes | Boiling number. |`,
  },
  {
    name: `nu_hilpert`,
    slug: `nu_hilpert`,
    category: `Heat Transfer`,
    summary: `Nu, single-cylinder cross-flow. SIDE: air/gas over a single tube. HX: bare-tube / sparse-bank air side`,
    related: [],
    examples: [],
    tags: [`nu`, `hilpert`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Nu, single-cylinder cross-flow. SIDE: air/gas over a single tube. HX: bare-tube / sparse-bank air side


## Syntax

\`\`\`
nu_hilpert(Re, Pr)
\`\`\`

## Description

Nu, single-cylinder cross-flow. SIDE: air/gas over a single tube. HX: bare-tube / sparse-bank air side

## Mathematical Formulation

$$ Nu = C\\,Re^{m}\\,Pr^{1/3} \\quad\\text{(single cylinder, Hilpert)} $$

## Applicability

- **Where it applies:** Air/gas over a single cylinder or a sparse bank.
- **Valid when:** Cross-flow over an isolated tube; band-dependent \`C, m\`.
- **How it's used:** Air-side \`h\` for bare-tube / low-density-bank exchangers.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re\` | Number | Yes | Reynolds number. |
| \`Pr\` | Number | Yes | Prandtl number. |`,
  },
  {
    name: `nu_plate`,
    slug: `nu_plate`,
    category: `Heat Transfer`,
    summary: `Nu, chevron-angle dependent. SIDE: either single-phase stream in a brazed/gasketed PLATE HX. HX: plate heat exchanger (BPHE)`,
    related: [],
    examples: [],
    tags: [`nu`, `plate`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Nu, chevron-angle dependent. SIDE: either single-phase stream in a brazed/gasketed PLATE HX. HX: plate heat exchanger (BPHE)


## Syntax

\`\`\`
nu_plate(Re, Pr, beta_deg)
\`\`\`

## Description

Nu, chevron-angle dependent. SIDE: either single-phase stream in a brazed/gasketed PLATE HX. HX: plate heat exchanger (BPHE)

## Mathematical Formulation

$$ Nu = C(\\beta)\\,Re^{m}\\,Pr^{1/3} \\quad\\text{(chevron plate, angle } \\beta) $$

## Applicability

- **Where it applies:** A single-phase stream in a brazed/gasketed plate heat exchanger (BPHE).
- **Valid when:** Chevron-plate channels; depends on the chevron angle \`β\`.
- **How it's used:** Plate-side \`h\` for either stream of a plate HX.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re\` | Number | Yes | Reynolds number. |
| \`Pr\` | Number | Yes | Prandtl number. |
| \`beta_deg\` | Number | Yes | Chevron / wave angle [deg]. |`,
  },
  {
    name: `nu_traviss`,
    slug: `nu_traviss`,
    category: `Heat Transfer`,
    summary: `Nu, Traviss in-tube condensation. SIDE: condensing two-phase refrigerant. HX: tube/microchannel condenser refrigerant side`,
    related: [],
    examples: [],
    tags: [`nu`, `traviss`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Nu, Traviss in-tube condensation. SIDE: condensing two-phase refrigerant. HX: tube/microchannel condenser refrigerant side


## Syntax

\`\`\`
nu_traviss(Re_l, Pr_l, Xtt)
\`\`\`

## Description

Nu, Traviss in-tube condensation. SIDE: condensing two-phase refrigerant. HX: tube/microchannel condenser refrigerant side

## Mathematical Formulation

$$ Nu = \\frac{Pr_l\\,Re_l^{0.9}\\,F(X_{tt})}{F_2} \\quad\\text{(Traviss condensation)} $$

## Applicability

- **Where it applies:** Condensing two-phase refrigerant in tube/microchannel condensers.
- **Valid when:** In-tube condensation, annular-flow dominated.
- **How it's used:** Condenser refrigerant-side \`h\`; alternative to \`nu_shah\`/\`nu_cavallini_zecchin\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re_l\` | Number | Yes | Liquid-only Reynolds number. |
| \`Pr_l\` | Number | Yes | Liquid Prandtl number. |
| \`Xtt\` | Number | Yes | Turbulent–turbulent Martinelli parameter. |`,
  },
  {
    name: `nu_tubebank`,
    slug: `nu_tubebank`,
    category: `Heat Transfer`,
    summary: `Nu, Zukauskas tube bank (arr$=inline|staggered, Re-band C,m). SIDE: air/gas over a tube bank. HX: fin-and-tube radiator/condenser air side`,
    related: [],
    examples: [],
    tags: [`nu`, `tubebank`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Nu, Zukauskas tube bank (arr$=inline|staggered, Re-band C,m). SIDE: air/gas over a tube bank. HX: fin-and-tube radiator/condenser air side


## Syntax

\`\`\`
nu_tubebank(arr$, Re, Pr)
\`\`\`

## Description

Nu, Zukauskas tube bank (arr$=inline|staggered, Re-band C,m). SIDE: air/gas over a tube bank. HX: fin-and-tube radiator/condenser air side

## Mathematical Formulation

$$ Nu = C\\,Re_{\\max}^{m}\\,Pr^{0.36}\\,(Pr/Pr_w)^{1/4} \\quad (C, m \\text{ by arrangement/Re band}) $$

## Applicability

- **Where it applies:** Air/gas over an in-line or staggered tube bank.
- **Valid when:** Cross-flow; \`arr$\` selects the arrangement and the Reynolds-band coefficients.
- **How it's used:** Air-side \`h\` for a fin-and-tube core.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`arr$\` | String | Yes | Selector — One of \`inline\`, \`staggered\`. |
| \`Re\` | Number | Yes | Reynolds number. |
| \`Pr\` | Number | Yes | Prandtl number. |`,
  },
  {
    name: `nu_zukauskas`,
    slug: `nu_zukauskas`,
    category: `Heat Transfer`,
    summary: `Nu, tube-bank cross-flow. SIDE: air/gas over a tube bank. HX: fin-and-tube radiator/condenser air side`,
    related: [],
    examples: [],
    tags: [`nu`, `zukauskas`, `heat`, `transfer`],
    references: [],
    guides: [],
    body: `Nu, tube-bank cross-flow. SIDE: air/gas over a tube bank. HX: fin-and-tube radiator/condenser air side


## Syntax

\`\`\`
nu_zukauskas(Re, Pr)
\`\`\`

## Description

Nu, tube-bank cross-flow. SIDE: air/gas over a tube bank. HX: fin-and-tube radiator/condenser air side

## Mathematical Formulation

$$ Nu = C\\,Re_{\\max}^{m}\\,Pr^{0.36}\\,(Pr/Pr_w)^{1/4} \\quad\\text{(tube bank)} $$

## Applicability

- **Where it applies:** Air/gas in cross-flow over a tube bank (the air side of a fin-and-tube radiator/condenser).
- **Valid when:** External cross-flow; the constants \`C, m\` depend on the in-line/staggered arrangement and the Reynolds band.
- **How it's used:** Gives the air-side film coefficient \`h = Nu·k/D\`; combine with the refrigerant/coolant side and wall via \`ua_hx\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re\` | Number | Yes | Reynolds number. |
| \`Pr\` | Number | Yes | Prandtl number. |`,
  },
  {
    name: `ua_hx`,
    slug: `ua_hx`,
    category: `Heat Transfer`,
    summary: `Overall heat-exchanger conductance UA from two side films and the wall.`,
    related: [`htc_1phase`, `htc_evap`, `htc_cond`, `hx_effectiveness`],
    examples: [`ev-thermal-management`],
    tags: [`heat exchanger`, `ua`, `overall conductance`, `thermal resistance`, `series`],
    references: [],
    guides: [],
    body: `Returns the **overall thermal conductance** \`UA\` [W/K] of a two-stream heat
exchanger by combining the two convective side films and the wall as series
thermal resistances. Feed the result to \`hx_effectiveness\` /
\`hx_NTU\` to rate or size the exchanger.

## Syntax

\`\`\`
UA = ua_hx(h1, A1, h2, A2, Rwall)
\`\`\`

## Description

Each stream presents a film resistance \`1/(h·A)\`; the wall adds a conductive
resistance \`Rwall\`. In series these sum to the inverse conductance.

## Mathematical Formulation

$$ \\frac{1}{UA} = \\frac{1}{h_1 A_1} + R_{\\text{wall}} + \\frac{1}{h_2 A_2} $$

For finned (extended) surfaces each film term carries its overall surface
efficiency, \`1/(η·h·A)\` — supply the efficiency-weighted area or
an \`h·η\` product.

> **Method:** direct series-resistance sum; the smaller \`h·A\` dominates \`UA\`.

## Examples

### Example 1 — Chiller UA from refrigerant and coolant films

The EV thermal-management sizing forms each side's \`h·A\` (from the \`htc_*\`
correlations and the geometry) and combines them with the wall resistance.

[Run: ev-thermal-management]

**Expected:** \`UA\` is governed by the weaker side — typically the air/gas film,
whose \`h·A\` is far smaller than a boiling/condensing refrigerant side.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`h1\` | Number | Yes | Side-1 film coefficient [W/m²·K]. |
| \`A1\` | Number | Yes | Side-1 (effective) area [m²]. |
| \`h2\` | Number | Yes | Side-2 film coefficient [W/m²·K]. |
| \`A2\` | Number | Yes | Side-2 (effective) area [m²]. |
| \`Rwall\` | Number | Yes | Wall conductive resistance [K/W]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`UA\` | Number | Overall conductance [W/K]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | A film coefficient or area ≤ 0 | All resistances must be finite and positive. |`,
  },
  {
    name: `viewfactor_disks`,
    slug: `viewfactor_disks`,
    category: `Heat Transfer`,
    summary: `Diffuse radiation view factor between two coaxial parallel disks.`,
    related: [`viewfactor_plates`, `viewfactor_perp`],
    examples: [`radiation-view-factors`],
    tags: [`radiation`, `view factor`, `configuration factor`, `disks`, `coaxial`, `enclosure`],
    references: [],
    guides: [`chemistry`],
    body: `Returns the **diffuse radiation view factor** \`F_{1→2}\` from a disk of radius \`r1\`
to a coaxial parallel disk of radius \`r2\` separated by distance \`L\` — the fraction
of diffuse radiation leaving disk 1 that strikes disk 2. Use it in radiation
enclosure analysis instead of reading a chart.

## Syntax

\`\`\`
F = viewfactor_disks(r1, r2, L)
\`\`\`

## Description

For two directly-opposed coaxial disks, the view factor is a closed-form function
of the two radii and the separation. It feeds net radiation exchange via
\`Q_{1→2} = A_1 F_{1→2} σ (T_1^4 − T_2^4)\` (gray-diffuse, with the reciprocity and
summation rules closing the enclosure).

## Mathematical Formulation

With $R_1 = r_1/L$, $R_2 = r_2/L$, and $X = 1 + \\dfrac{1 + R_2^2}{R_1^2}$,

$$ F_{1\\to 2} = \\frac{1}{2}\\left\\{\\,X - \\left[X^2 - 4\\left(\\frac{R_2}{R_1}\\right)^2\\right]^{1/2}\\right\\} $$

> **Method:** direct evaluation of the standard closed-form view-factor expression.

## Examples

### Example 1 — Coaxial disks

[Run: radiation-view-factors]

**Expected:** \`viewfactor_disks(0.5, 1, 0.4) ≈ 0.83\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`r1\` | Number | Yes | Radius of the emitting disk [m]. |
| \`r2\` | Number | Yes | Radius of the opposing coaxial disk [m]. |
| \`L\` | Number | Yes | Axial separation between the disks [m] (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`F\` | Number | View factor F_{1→2} ∈ [0, 1]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`L ≤ 0\` or a radius ≤ 0 | All three lengths must be positive. |`,
  },
  {
    name: `viewfactor_perp`,
    slug: `viewfactor_perp`,
    category: `Heat Transfer`,
    summary: `Diffuse radiation view factor between perpendicular rectangles with a common edge.`,
    related: [`viewfactor_plates`, `viewfactor_disks`],
    examples: [`radiation-view-factors`],
    tags: [`radiation`, `view factor`, `configuration factor`, `perpendicular`, `rectangles`, `corner`],
    references: [],
    guides: [`chemistry`],
    body: `Returns the **diffuse radiation view factor** \`F_{1→2}\` between two **perpendicular
rectangles that share a common edge** (an interior corner). Use it for radiation
exchange between adjoining walls without a chart lookup.

## Syntax

\`\`\`
F = viewfactor_perp(Y, Z, X)
\`\`\`

## Description

Two rectangles meeting at right angles along a common edge exchange radiation with
a view factor that depends on the two aspect ratios formed against the shared
dimension. \`Y\` and \`Z\` are the far extents of the two surfaces; \`X\` is the common
edge length.

## Mathematical Formulation

With $H = Z/X$ and $W = Y/X$,

$$ F_{1\\to 2} = \\frac{1}{\\pi W}\\Bigg( W\\tan^{-1}\\frac{1}{W} + H\\tan^{-1}\\frac{1}{H} - \\sqrt{H^2+W^2}\\,\\tan^{-1}\\frac{1}{\\sqrt{H^2+W^2}} + \\frac{1}{4}\\ln\\Big[\\,\\cdots\\,\\Big] \\Bigg) $$

(the logarithmic term is the full standard closed-form expression in $H$ and $W$.)

> **Method:** direct evaluation of the standard closed-form view-factor expression.

## Examples

### Example 1 — Perpendicular rectangles sharing an edge

[Run: radiation-view-factors]

**Expected:** \`viewfactor_perp(1, 1, 1) ≈ 0.20\` (two equal perpendicular squares).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Y\` | Number | Yes | Extent of surface 1 normal to the common edge [m]. |
| \`Z\` | Number | Yes | Extent of surface 2 normal to the common edge [m]. |
| \`X\` | Number | Yes | Length of the shared (common) edge [m] (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`F\` | Number | View factor F_{1→2} ∈ [0, 1]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`X ≤ 0\` or an extent ≤ 0 | All three lengths must be positive. |`,
  },
  {
    name: `viewfactor_plates`,
    slug: `viewfactor_plates`,
    category: `Heat Transfer`,
    summary: `Diffuse radiation view factor between two aligned parallel rectangles.`,
    related: [`viewfactor_disks`, `viewfactor_perp`],
    examples: [`radiation-view-factors`],
    tags: [`radiation`, `view factor`, `configuration factor`, `parallel plates`, `rectangles`],
    references: [],
    guides: [`chemistry`],
    body: `Returns the **diffuse radiation view factor** \`F_{1→2}\` between two **aligned,
directly-opposed parallel rectangles** of side lengths \`X\` and \`Y\` separated by
distance \`D\`. Use it for parallel-surface radiation exchange without a chart lookup.

## Syntax

\`\`\`
F = viewfactor_plates(X, Y, D)
\`\`\`

## Description

Two identical, aligned, parallel rectangles see each other with a view factor that
depends only on the side-to-gap aspect ratios \`x = X/D\` and \`y = Y/D\`.

## Mathematical Formulation

With $x = X/D$ and $y = Y/D$,

$$ F_{1\\to 2} = \\frac{2}{\\pi x y}\\left\\{ \\ln\\!\\left[\\frac{(1+x^2)(1+y^2)}{1+x^2+y^2}\\right]^{1/2} + x\\sqrt{1+y^2}\\,\\tan^{-1}\\!\\frac{x}{\\sqrt{1+y^2}} + y\\sqrt{1+x^2}\\,\\tan^{-1}\\!\\frac{y}{\\sqrt{1+x^2}} - x\\tan^{-1}x - y\\tan^{-1}y \\right\\} $$

> **Method:** direct evaluation of the standard closed-form view-factor expression.

## Examples

### Example 1 — Aligned parallel rectangles

[Run: radiation-view-factors]

**Expected:** \`viewfactor_plates(2, 2, 1) ≈ 0.41\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`X\` | Number | Yes | First side length of the rectangles [m]. |
| \`Y\` | Number | Yes | Second side length of the rectangles [m]. |
| \`D\` | Number | Yes | Separation between the parallel planes [m] (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`F\` | Number | View factor F_{1→2} ∈ [0, 1]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`D ≤ 0\` or a side ≤ 0 | All three lengths must be positive. |`,
  },
  {
    name: `dtable`,
    slug: `dtable`,
    category: `Interpolation`,
    summary: `Analytic slope of a table's linear interpolant — the exact derivative of t(x).`,
    related: [`dtable1`, `interpolate`, `differentiate`],
    examples: [],
    tags: [`table`, `derivative`, `slope`, `interpolation`, `cam`, `feedforward`, `map`],
    references: [],
    guides: [],
    body: `Returns the **exact derivative of the interpolant** a bare table call evaluates: for
\`t(x)\` (piecewise-linear), \`dtable('t', x)\` is the slope of the segment containing
\`x\`. Unlike \`Differentiate\` (a general column-vs-column numerical derivative), the
first y-curve against the x column is implied — the 1-D map-call convention.

## Syntax

\`\`\`
d = dtable('t', x)
\`\`\`

Inside a component, a \`map$\`-style string parameter works directly — this is the
feedforward/cam idiom the function exists for:

\`\`\`
COMPONENT CamFollower(shaft, rod)
  PARAM prof$
  lift    = prof$(theta)
  rod.vel = dtable(prof$, theta) * shaft.w   { chain rule: dl/dθ · dθ/dt }
  ...
END
\`\`\`

## Description

Because the slope is read from the interpolant itself (not finite-differenced),
it is exact everywhere for the linear interpolant — including between knots — and
piecewise-constant across a segment. At a knot the right-segment slope is
returned; outside the tabulated range the edge segment's slope extends. For a
smooth derivative use \`dtable1\` (cubic spline).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | String | Yes | Name of a \`TABLE\` block (or a bound \`map$\` parameter). |
| \`x\` | Number | Yes | Evaluation point, in the table's x units. |

## Examples

### Example 1 — exact slope of a linear table

\`\`\`
TABLE lin(x)
  0   0
  1   3
  2   6
END
s = dtable('lin', 0.5)    { = 3, exactly }
\`\`\`

## See also

\`dtable1\`, \`Interpolate\`, \`Differentiate\``,
  },
  {
    name: `dtable1`,
    slug: `dtable1`,
    category: `Interpolation`,
    summary: `Cubic-spline derivative of a table at x — the smooth d/dx of Interpolate1.`,
    related: [`dtable`, `interpolate1`, `differentiate`],
    examples: [],
    tags: [`table`, `derivative`, `spline`, `cubic`, `interpolation`, `smooth`],
    references: [],
    guides: [],
    body: `Returns the derivative of the **natural cubic spline** through the table's first
curve at \`x\` — the smooth counterpart of \`dtable\`, matching the interpolant
\`Interpolate1\` evaluates. Use it when the consumer differentiates again (the
linear interpolant's slope is discontinuous at knots) or when the tabulated data
represents a smooth underlying function.

## Syntax

\`\`\`
d = dtable1('t', x)
\`\`\`

## Description

The spline is built over the sorted x column against the first y curve; \`x\`
clamps to the tabulated range. Tables with fewer than three rows fall back to
the linear-segment slope (same as \`dtable\`).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | String | Yes | Name of a \`TABLE\` block (or a bound \`map$\` parameter). |
| \`x\` | Number | Yes | Evaluation point, in the table's x units. |

## Examples

### Example 1 — spline slope through y = x²

\`\`\`
TABLE quad(x)
  0   0
  1   1
  2   4
END
s = dtable1('quad', 1)    { = 2 — the natural spline through x² has the exact slope at the middle knot }
\`\`\`

## See also

\`dtable\`, \`Interpolate1\`, \`Differentiate\``,
  },
  {
    name: `interpolate`,
    slug: `interpolate`,
    category: `Interpolation`,
    summary: `Linear interpolation of table t at x (same as t(x))`,
    related: [],
    examples: [],
    tags: [`interpolate`, `interpolation`],
    references: [],
    guides: [`lookup-tables`],
    body: `Linear interpolation of table t at x (same as t(x))


## Syntax

\`\`\`
Interpolate('t', x)
\`\`\`

## Description

Linear interpolation of table t at x (same as t(x))

## Mathematical Formulation

$$ y = y_i + (y_{i+1}-y_i)\\frac{x - x_i}{x_{i+1} - x_i} \\quad\\text{(linear)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | Number | Yes | Name of a TABLE block (string). |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `interpolate1`,
    slug: `interpolate1`,
    category: `Interpolation`,
    summary: `Cubic-spline interpolation of table t at x`,
    related: [],
    examples: [],
    tags: [`interpolate1`, `interpolation`],
    references: [],
    guides: [`lookup-tables`],
    body: `Cubic-spline interpolation of table t at x


## Syntax

\`\`\`
Interpolate1('t', x)
\`\`\`

## Description

Cubic-spline interpolation of table t at x

## Mathematical Formulation

$$ \\text{piecewise cubic spline through the table knots (} C^2 \\text{ continuous)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | Number | Yes | Name of a TABLE block (string). |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `interpolate2d`,
    slug: `interpolate2d`,
    category: `Interpolation`,
    summary: `Bilinear interpolation of a 2-D table at (x, y).`,
    related: [`interpolate`, `lookup`, `interpolate1`],
    examples: [`engine-map-2d`],
    tags: [`interpolation`, `bilinear`, `2d`, `table`, `lookup`, `map`],
    references: [],
    guides: [`lookup-tables`],
    body: `Performs **bilinear interpolation** of a named 2-D \`TABLE\` \`t\` at the point
\`(x, y)\`. Use it for engine maps, efficiency surfaces, and any quantity tabulated
against two independent variables.

## Syntax

\`\`\`
z = Interpolate2D('t', x, y)
\`\`\`

## Description

The table provides \`z\` on a grid of \`x\` (columns) and \`y\` (rows); the function
blends the four surrounding grid values weighted by the fractional position of
\`(x, y)\` within its cell.

## Mathematical Formulation

For \`(x, y)\` in the cell bounded by \`x_i ≤ x ≤ x_{i+1}\`, \`y_j ≤ y ≤ y_{j+1}\`, with
\`t_x = (x − x_i)/(x_{i+1} − x_i)\` and \`t_y = (y − y_j)/(y_{j+1} − y_j)\`:

$$ z = (1-t_x)(1-t_y)z_{i,j} + t_x(1-t_y)z_{i+1,j} + (1-t_x)t_y\\,z_{i,j+1} + t_x t_y\\,z_{i+1,j+1} $$

> **Method:** locate the bounding grid cell, then bilinear blend the four corners.

## Examples

### Example 1 — Engine efficiency from a 2-D map

[Run: engine-map-2d]

**Expected:** the interpolated map value at the requested speed/load point.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | String | Yes | Name of a 2-D \`TABLE\` block. |
| \`x\` | Number | Yes | First (column) coordinate. |
| \`y\` | Number | Yes | Second (row) coordinate. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`z\` | Number | Bilinearly interpolated table value. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`OUT_OF_RANGE\` | \`(x, y)\` outside the table grid | Keep the query within the tabulated bounds. |
| \`UNKNOWN_TABLE\` | \`'t'\` is not a defined table | Define the 2-D \`TABLE\` block first. |`,
  },
  {
    name: `lookup`,
    slug: `lookup`,
    category: `Interpolation`,
    summary: `Cell value by 1-based row/col indices`,
    related: [],
    examples: [],
    tags: [`lookup`, `interpolation`],
    references: [],
    guides: [`lookup-tables`],
    body: `Cell value by 1-based row/col indices


## Syntax

\`\`\`
Lookup('t', row, col)
\`\`\`

## Description

Cell value by 1-based row/col indices

## Mathematical Formulation

$$ \\operatorname{Lookup}(t, r, c) = t_{r,c} \\quad\\text{(1-based cell)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | Number | Yes | Name of a TABLE block (string). |
| \`row\` | Number | Yes | Row index (1-based). |
| \`col\` | Number | Yes | Name of a result-table column. |`,
  },
  {
    name: `lookuprow`,
    slug: `lookuprow`,
    category: `Interpolation`,
    summary: `Row index where column col crosses val`,
    related: [],
    examples: [],
    tags: [`lookuprow`, `interpolation`],
    references: [],
    guides: [`lookup-tables`],
    body: `Row index where column col crosses val


## Syntax

\`\`\`
LookupRow('t', col, val)
\`\`\`

## Description

Row index where column col crosses val

## Mathematical Formulation

$$ \\text{row } r \\text{ where column } c \\text{ crosses } val \\text{ (interpolated)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | Number | Yes | Name of a TABLE block (string). |
| \`col\` | Number | Yes | Name of a result-table column. |
| \`val\` | Number | Yes | Target value to cross. |`,
  },
  {
    name: `nlookuprows`,
    slug: `nlookuprows`,
    category: `Interpolation`,
    summary: `Number of data rows in table t`,
    related: [],
    examples: [],
    tags: [`nlookuprows`, `interpolation`],
    references: [],
    guides: [`lookup-tables`],
    body: `Number of data rows in table t


## Syntax

\`\`\`
NLookupRows('t')
\`\`\`

## Description

Number of data rows in table t

## Mathematical Formulation

$$ \\operatorname{NLookupRows}(t) = \\#\\text{rows}(t) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'t'\` | Number | Yes | Name of a TABLE block (string). |`,
  },
  {
    name: `if`,
    slug: `if`,
    category: `Logic`,
    summary: `Inline three-way conditional based on comparing a and b.`,
    related: [`min`, `max`, `sign`, `step`],
    examples: [`sounding-rocket-trajectory`],
    tags: [`logic`, `conditional`, `branch`, `if`, `comparison`],
    references: [],
    guides: [`math-funcs`],
    body: `Returns one of three values depending on how \`a\` compares to \`b\`: \`lt\` if \`a < b\`,
\`eq\` if \`a = b\`, \`gt\` if \`a > b\`. It is the inline branch for the declarative top
level (use \`IF…THEN\` inside \`FUNCTION\`/\`PROCEDURE\` bodies).

## Syntax

\`\`\`
y = If(a, b, lt, eq, gt)
\`\`\`

## Description

A smooth-free conditional select. Because frees is an equation solver, \`If\` lets a
value switch on a comparison without introducing imperative control flow into the
document body.

## Mathematical Formulation

$$ y = \\begin{cases} lt & a < b \\\\ eq & a = b \\\\ gt & a > b \\end{cases} $$

## Examples

### Example 1 — Phase switch in a rocket trajectory

[Run: sounding-rocket-trajectory]

**Expected:** the conditional selects the active branch (e.g. powered vs coast)
based on the comparison.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | Left comparison operand. |
| \`b\` | Number | Yes | Right comparison operand. |
| \`lt\` | Number | Yes | Value returned when \`a < b\`. |
| \`eq\` | Number | Yes | Value returned when \`a = b\`. |
| \`gt\` | Number | Yes | Value returned when \`a > b\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | The selected branch value. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNIT_MISMATCH\` | \`a\` and \`b\` have incompatible units | Compare quantities with compatible dimensions. |`,
  },
  {
    name: `abs`,
    slug: `abs`,
    category: `Math`,
    summary: `Absolute value`,
    related: [],
    examples: [],
    tags: [`abs`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Absolute value


## Syntax

\`\`\`
abs(x)
\`\`\`

## Description

Absolute value

## Mathematical Formulation

$$ |x| = \\begin{cases} x & x \\ge 0 \\\\ -x & x < 0 \\end{cases} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `acos`,
    slug: `acos`,
    category: `Math`,
    summary: `Inverse cosine`,
    related: [],
    examples: [],
    tags: [`acos`, `math`],
    references: [],
    guides: [],
    body: `Inverse cosine


## Syntax

\`\`\`
acos(x)
\`\`\`

## Description

Inverse cosine

## Mathematical Formulation

$$ y = \\arccos(x), \\qquad \\cos(y) = x,\\ \\ y \\in [0, \\pi] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `arccos`,
    slug: `arccos`,
    category: `Math`,
    summary: `Inverse cosine [rad] (alias of acos)`,
    related: [],
    examples: [],
    tags: [`arccos`, `math`],
    references: [],
    guides: [],
    body: `Inverse cosine [rad] (alias of acos)


## Syntax

\`\`\`
arccos(x)
\`\`\`

## Description

Inverse cosine [rad] (alias of acos)

## Mathematical Formulation

$$ y = \\arccos(x), \\qquad \\cos(y) = x,\\ \\ y \\in [0, \\pi] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `arccosh`,
    slug: `arccosh`,
    category: `Math`,
    summary: `Inverse hyperbolic cosine (x>=1)`,
    related: [],
    examples: [],
    tags: [`arccosh`, `math`],
    references: [],
    guides: [],
    body: `Inverse hyperbolic cosine (x>=1)


## Syntax

\`\`\`
arccosh(x)
\`\`\`

## Description

Inverse hyperbolic cosine (x>=1)

## Mathematical Formulation

$$ \\operatorname{arccosh}(x) = \\ln\\!\\big(x + \\sqrt{x^2-1}\\big), \\quad x \\ge 1 $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `arcsin`,
    slug: `arcsin`,
    category: `Math`,
    summary: `Inverse sine [rad] (alias of asin)`,
    related: [],
    examples: [],
    tags: [`arcsin`, `math`],
    references: [],
    guides: [],
    body: `Inverse sine [rad] (alias of asin)


## Syntax

\`\`\`
arcsin(x)
\`\`\`

## Description

Inverse sine [rad] (alias of asin)

## Mathematical Formulation

$$ y = \\arcsin(x), \\qquad \\sin(y) = x,\\ \\ y \\in [-\\tfrac{\\pi}{2}, \\tfrac{\\pi}{2}] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `arcsinh`,
    slug: `arcsinh`,
    category: `Math`,
    summary: `Inverse hyperbolic sine`,
    related: [],
    examples: [],
    tags: [`arcsinh`, `math`],
    references: [],
    guides: [],
    body: `Inverse hyperbolic sine


## Syntax

\`\`\`
arcsinh(x)
\`\`\`

## Description

Inverse hyperbolic sine

## Mathematical Formulation

$$ \\operatorname{arcsinh}(x) = \\ln\\!\\big(x + \\sqrt{x^2+1}\\big) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `arctan`,
    slug: `arctan`,
    category: `Math`,
    summary: `Inverse tangent [rad] (alias of atan)`,
    related: [],
    examples: [],
    tags: [`arctan`, `math`],
    references: [],
    guides: [`tut-rlc`],
    body: `Inverse tangent [rad] (alias of atan)


## Syntax

\`\`\`
arctan(x)
\`\`\`

## Description

Inverse tangent [rad] (alias of atan)

## Mathematical Formulation

$$ y = \\arctan(x), \\qquad \\tan(y) = x $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `arctanh`,
    slug: `arctanh`,
    category: `Math`,
    summary: `Inverse hyperbolic tangent (|x|<1)`,
    related: [],
    examples: [],
    tags: [`arctanh`, `math`],
    references: [],
    guides: [],
    body: `Inverse hyperbolic tangent (|x|<1)


## Syntax

\`\`\`
arctanh(x)
\`\`\`

## Description

Inverse hyperbolic tangent (|x|<1)

## Mathematical Formulation

$$ \\operatorname{arctanh}(x) = \\tfrac12\\ln\\!\\frac{1+x}{1-x}, \\quad |x| < 1 $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `arrayelmt`,
    slug: `arrayelmt`,
    category: `Math`,
    summary: `Select the i-th element of an array range`,
    related: [],
    examples: [],
    tags: [`arrayelmt`, `math`],
    references: [],
    guides: [`arrays`],
    body: `Select the i-th element of an array range


## Syntax

\`\`\`
ArrayElmt(arr[1:n], i)
\`\`\`

## Description

Select the i-th element of an array range

## Mathematical Formulation

$$ \\operatorname{ArrayElmt}(\\{a_1,\\dots,a_n\\}, i) = a_i $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`arr[1:n]\` | Array | Yes | Array range to index into, e.g. \`data[1:n]\`. |
| \`i\` | Number | Yes | 1-based index of the element to return. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`n]\` | Number/Array | Computed \`n]\`. |
| \`i\` | Number/Array | Computed \`i\`. |`,
  },
  {
    name: `asin`,
    slug: `asin`,
    category: `Math`,
    summary: `Inverse sine`,
    related: [],
    examples: [],
    tags: [`asin`, `math`],
    references: [],
    guides: [],
    body: `Inverse sine


## Syntax

\`\`\`
asin(x)
\`\`\`

## Description

Inverse sine

## Mathematical Formulation

$$ y = \\arcsin(x), \\qquad \\sin(y) = x,\\ \\ y \\in [-\\tfrac{\\pi}{2}, \\tfrac{\\pi}{2}] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `atan`,
    slug: `atan`,
    category: `Math`,
    summary: `Inverse tangent`,
    related: [],
    examples: [],
    tags: [`atan`, `math`],
    references: [],
    guides: [],
    body: `Inverse tangent


## Syntax

\`\`\`
atan(x)
\`\`\`

## Description

Inverse tangent

## Mathematical Formulation

$$ y = \\arctan(x), \\qquad \\tan(y) = x $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `atan2`,
    slug: `atan2`,
    category: `Math`,
    summary: `Four-quadrant inverse tangent`,
    related: [],
    examples: [],
    tags: [`atan2`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Four-quadrant inverse tangent


## Syntax

\`\`\`
atan2(y, x)
\`\`\`

## Description

Four-quadrant inverse tangent

## Mathematical Formulation

$$ \\operatorname{atan2}(y,x) = \\arg(x + jy) \\in (-\\pi, \\pi] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`y\` | Number | Yes | Value / second coordinate. |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `axpy`,
    slug: `axpy`,
    category: `Math`,
    summary: `BLAS: αx + y`,
    related: [],
    examples: [],
    tags: [`axpy`, `math`],
    references: [],
    guides: [`matrices-blas`],
    body: `BLAS: αx + y


## Syntax

\`\`\`
axpy(α, x, y)
\`\`\`

## Description

BLAS: αx + y

## Mathematical Formulation

$$ y \\leftarrow \\alpha x + y \\quad\\text{(BLAS level 1)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`α\` | Number | Yes | Scalar coefficient α. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`y\` | Number | Yes | Value / second coordinate. |`,
  },
  {
    name: `baseconvert`,
    slug: `baseconvert`,
    category: `Math`,
    summary: `Convert a based-number string literal to a value`,
    related: [],
    examples: [],
    tags: [`baseconvert`, `math`],
    references: [],
    guides: [],
    body: `Convert a based-number string literal to a value


## Syntax

\`\`\`
baseconvert(s$)
\`\`\`

## Description

Convert a based-number string literal to a value

## Mathematical Formulation

$$ \\operatorname{baseconvert}(s) = \\text{numeric value of the based literal } s \\ (\\text{e.g. } \\mathtt{0xFF} \\to 255) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`s$\` | String | Yes | String literal. |`,
  },
  {
    name: `bitand`,
    slug: `bitand`,
    category: `Math`,
    summary: `Bitwise AND`,
    related: [],
    examples: [],
    tags: [`bitand`, `math`],
    references: [],
    guides: [],
    body: `Bitwise AND


## Syntax

\`\`\`
bitand(a, b)
\`\`\`

## Description

Bitwise AND

## Mathematical Formulation

$$ (a \\,\\&\\, b)\\ \\text{— bitwise AND of the integer operands} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `bitnot`,
    slug: `bitnot`,
    category: `Math`,
    summary: `Bitwise NOT`,
    related: [],
    examples: [],
    tags: [`bitnot`, `math`],
    references: [],
    guides: [],
    body: `Bitwise NOT


## Syntax

\`\`\`
bitnot(a)
\`\`\`

## Description

Bitwise NOT

## Mathematical Formulation

$$ (\\sim a) = -(a+1)\\ \\text{(two’s complement)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |`,
  },
  {
    name: `bitor`,
    slug: `bitor`,
    category: `Math`,
    summary: `Bitwise OR`,
    related: [],
    examples: [],
    tags: [`bitor`, `math`],
    references: [],
    guides: [],
    body: `Bitwise OR


## Syntax

\`\`\`
bitor(a, b)
\`\`\`

## Description

Bitwise OR

## Mathematical Formulation

$$ (a \\mathbin{|} b)\\ \\text{— bitwise OR of the integer operands} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `bitshiftl`,
    slug: `bitshiftl`,
    category: `Math`,
    summary: `Left bit shift a<<n`,
    related: [],
    examples: [],
    tags: [`bitshiftl`, `math`],
    references: [],
    guides: [],
    body: `Left bit shift a<<n


## Syntax

\`\`\`
bitshiftl(a, n)
\`\`\`

## Description

Left bit shift a<<n

## Mathematical Formulation

$$ a \\ll n = a\\cdot 2^{n} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `bitshiftr`,
    slug: `bitshiftr`,
    category: `Math`,
    summary: `Right bit shift a>>n`,
    related: [],
    examples: [],
    tags: [`bitshiftr`, `math`],
    references: [],
    guides: [],
    body: `Right bit shift a>>n


## Syntax

\`\`\`
bitshiftr(a, n)
\`\`\`

## Description

Right bit shift a>>n

## Mathematical Formulation

$$ a \\gg n = \\lfloor a / 2^{n} \\rfloor $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `bitxor`,
    slug: `bitxor`,
    category: `Math`,
    summary: `Bitwise XOR`,
    related: [],
    examples: [],
    tags: [`bitxor`, `math`],
    references: [],
    guides: [],
    body: `Bitwise XOR


## Syntax

\`\`\`
bitxor(a, b)
\`\`\`

## Description

Bitwise XOR

## Mathematical Formulation

$$ (a \\oplus b)\\ \\text{— bitwise XOR of the integer operands} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `cbrt`,
    slug: `cbrt`,
    category: `Math`,
    summary: `Cube root of a number (with unit propagation).`,
    related: [`sqrt`, `exp`, `abs`],
    examples: [],
    tags: [`math`, `cube root`, `elementary`],
    references: [],
    guides: [`math-funcs`],
    body: `Returns the **cube root** \`∛x\`. Unlike \`sqrt\`, it is defined for negative
arguments (\`cbrt(-8) = -2\`). In frees it also propagates units — the result
carries one-third of the argument's dimension (e.g. \`cbrt(m^3) → m\`).

## Syntax

\`\`\`
y = cbrt(x)
\`\`\`

## Description

A standard elementary function, equivalent to \`x^(1/3)\` but valid for negative
\`x\` as well (the real cube root). Reach for it for characteristic lengths from a
volume, or any relation that inverts a cube.

## Mathematical Formulation

$$ y = \\sqrt[3]{x} = x^{1/3} $$

> **Method:** direct evaluation via the platform \`cbrt\` (real-valued for all \`x\`);
> the solver differentiates it as $\\frac{dy}{dx} = \\tfrac{1}{3}x^{-2/3}$ for Jacobians.

## Examples

### Example 1 — Characteristic length of a cubic volume

\`\`\`
{ Edge length of a cube from its volume }
Vol = 0.027 [m^3]
L = cbrt(Vol)        { 0.3 m }
\`\`\`

**Expected:** \`L = 0.3 [m]\` — note the unit reduces from \`m^3\` to \`m\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Any real value (any unit); negatives are allowed. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | The cube root, with one-third of the argument's dimension. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DIMENSION_ERROR\` | Argument dimension is not a perfect cube of a representable unit | Ensure the argument's units divide evenly by 3 (e.g. \`m^3\`, \`m^6\`). |`,
  },
  {
    name: `ceil`,
    slug: `ceil`,
    category: `Math`,
    summary: `Ceiling`,
    related: [],
    examples: [],
    tags: [`ceil`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Ceiling


## Syntax

\`\`\`
ceil(x)
\`\`\`

## Description

Ceiling

## Mathematical Formulation

$$ \\lceil x \\rceil = \\min\\{n \\in \\mathbb{Z} : n \\ge x\\} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `cos`,
    slug: `cos`,
    category: `Math`,
    summary: `Cosine of an angle (radians).`,
    related: [`sin`, `tan`, `arccos`, `atan2`],
    examples: [`projectile-motion`, `projectile-trajectory`],
    tags: [`math`, `trigonometry`, `cosine`, `radians`, `elementary`],
    references: [],
    guides: [],
    body: `Returns the **cosine** of \`x\`. The argument is in **radians** unless it carries a
\`[deg]\` unit annotation (which frees converts automatically).

## Syntax

\`\`\`
y = cos(x)
\`\`\`

## Description

A standard trigonometric function. Use \`x [deg]\` or \`Convert\` to work in degrees.

## Mathematical Formulation

$$ y = \\cos(x), \\qquad x \\text{ in radians} $$

## Examples

### Example 1 — Launch-angle component of velocity

[Run: projectile-motion]

**Expected:** \`cos\` of the launch angle gives the horizontal velocity component.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Angle in radians (or \`[deg]\`-annotated). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Cosine of \`x\` (dimensionless, in [−1, 1]). |`,
  },
  {
    name: `cosh`,
    slug: `cosh`,
    category: `Math`,
    summary: `Hyperbolic cosine`,
    related: [],
    examples: [],
    tags: [`cosh`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Hyperbolic cosine


## Syntax

\`\`\`
cosh(x)
\`\`\`

## Description

Hyperbolic cosine

## Mathematical Formulation

$$ \\cosh(x) = \\frac{e^{x} + e^{-x}}{2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `Cross`,
    slug: `cross`,
    category: `Math`,
    summary: `Cross product of 3-vectors`,
    related: [],
    examples: [],
    tags: [`cross`, `math`],
    references: [],
    guides: [`matrices-sys`],
    body: `Cross product of 3-vectors


## Syntax

\`\`\`
Cross(a, b)
\`\`\`

## Description

Cross product of 3-vectors

## Mathematical Formulation

$$ a \\times b = (a_2 b_3 - a_3 b_2,\\ a_3 b_1 - a_1 b_3,\\ a_1 b_2 - a_2 b_1) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `Determinant`,
    slug: `determinant`,
    category: `Math`,
    summary: `Determinant`,
    related: [],
    examples: [],
    tags: [`determinant`, `math`],
    references: [],
    guides: [`matrices-sys`],
    body: `Determinant


## Syntax

\`\`\`
Determinant(A)
\`\`\`

## Description

Determinant

## Mathematical Formulation

$$ \\det(A) = \\sum_{\\sigma} \\operatorname{sgn}(\\sigma)\\prod_i A_{i,\\sigma(i)} = \\pm\\prod_i U_{ii} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Matrix. |`,
  },
  {
    name: `Dot`,
    slug: `dot`,
    category: `Math`,
    summary: `Vector dot product`,
    related: [],
    examples: [],
    tags: [`dot`, `math`],
    references: [],
    guides: [`repl`, `matrices-sys`],
    body: `Vector dot product


## Syntax

\`\`\`
Dot(a, b)
\`\`\`

## Description

Vector dot product

## Mathematical Formulation

$$ a \\cdot b = \\sum_i a_i b_i $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `exp`,
    slug: `exp`,
    category: `Math`,
    summary: `Exponential function e raised to the power x.`,
    related: [`ln`, `log10`, `sqrt`],
    examples: [`damped-oscillator`, `sounding-rocket-trajectory`],
    tags: [`math`, `exponential`, `elementary`],
    references: [],
    guides: [`math-funcs`, `symbolic-cas`],
    body: `Returns the **exponential** \`eˣ\`. The argument must be dimensionless (the function
is only meaningful for a pure number).

## Syntax

\`\`\`
y = exp(x)
\`\`\`

## Description

A standard elementary function, the inverse of \`ln\`. It appears throughout
transient and decay models.

## Mathematical Formulation

$$ y = e^{x}, \\qquad e \\approx 2.71828 $$

## Examples

### Example 1 — Decay envelope of a damped oscillator

[Run: damped-oscillator]

**Expected:** \`exp\` of the negative-rate·time term gives the decaying amplitude.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Dimensionless exponent. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | \`eˣ\` (dimensionless). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNIT_MISMATCH\` | \`x\` carries a unit | The exponent must be dimensionless. |`,
  },
  {
    name: `eye`,
    slug: `eye`,
    category: `Math`,
    summary: `n×n identity`,
    related: [],
    examples: [],
    tags: [`eye`, `math`],
    references: [],
    guides: [`matrices-decl`],
    body: `n×n identity


## Syntax

\`\`\`
eye(n) / identity(n)
\`\`\`

## Description

n×n identity

## Mathematical Formulation

$$ I_{ij} = \\delta_{ij} \\quad (n\\times n) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `factorial`,
    slug: `factorial`,
    category: `Math`,
    summary: `Factorial n!`,
    related: [],
    examples: [],
    tags: [`factorial`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Factorial n!


## Syntax

\`\`\`
factorial(n)
\`\`\`

## Description

Factorial n!

## Mathematical Formulation

$$ n! = \\prod_{k=1}^{n} k = n\\,(n-1)! $$

$$ \\quad n! = \\Gamma(n+1) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `floor`,
    slug: `floor`,
    category: `Math`,
    summary: `Floor`,
    related: [],
    examples: [],
    tags: [`floor`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Floor


## Syntax

\`\`\`
floor(x)
\`\`\`

## Description

Floor

## Mathematical Formulation

$$ \\lfloor x \\rfloor = \\max\\{n \\in \\mathbb{Z} : n \\le x\\} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `gcd`,
    slug: `gcd`,
    category: `Math`,
    summary: `Greatest common divisor`,
    related: [],
    examples: [],
    tags: [`gcd`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Greatest common divisor


## Syntax

\`\`\`
gcd(a, b)
\`\`\`

## Description

Greatest common divisor

## Mathematical Formulation

$$ \\gcd(a,b) = \\gcd(b,\\ a \\bmod b), \\qquad \\gcd(a,0)=a \\quad\\text{(Euclid)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `gemm`,
    slug: `gemm`,
    category: `Math`,
    summary: `BLAS L3: αAB + βC`,
    related: [],
    examples: [],
    tags: [`gemm`, `math`],
    references: [],
    guides: [`matrices-blas`],
    body: `BLAS L3: αAB + βC


## Syntax

\`\`\`
gemm(α, A, B, β, C)
\`\`\`

## Description

BLAS L3: αAB + βC

## Mathematical Formulation

$$ C \\leftarrow \\alpha A B + \\beta C \\quad\\text{(BLAS level 3)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`α\` | Number | Yes | Scalar coefficient α. |
| \`A\` | Number | Yes | Matrix. |
| \`B\` | Number | Yes | Matrix operand. |
| \`β\` | Number | Yes | Scalar coefficient β. |
| \`C\` | Number | Yes | Empirical constant. |`,
  },
  {
    name: `gemv`,
    slug: `gemv`,
    category: `Math`,
    summary: `BLAS L2: αAx + βy`,
    related: [],
    examples: [],
    tags: [`gemv`, `math`],
    references: [],
    guides: [`matrices-blas`],
    body: `BLAS L2: αAx + βy


## Syntax

\`\`\`
gemv(α, A, x, β, y)
\`\`\`

## Description

BLAS L2: αAx + βy

## Mathematical Formulation

$$ y \\leftarrow \\alpha A x + \\beta y \\quad\\text{(BLAS level 2)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`α\` | Number | Yes | Scalar coefficient α. |
| \`A\` | Number | Yes | Matrix. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`β\` | Number | Yes | Scalar coefficient β. |
| \`y\` | Number | Yes | Value / second coordinate. |`,
  },
  {
    name: `Inverse`,
    slug: `inverse`,
    category: `Math`,
    summary: `Matrix inverse A⁻¹`,
    related: [],
    examples: [],
    tags: [`inverse`, `math`],
    references: [],
    guides: [`repl`, `matrices-sys`],
    body: `Matrix inverse A⁻¹


## Syntax

\`\`\`
Inverse(A)
\`\`\`

## Description

Matrix inverse A⁻¹

## Mathematical Formulation

$$ A\\,A^{-1} = A^{-1}A = I $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Matrix. |`,
  },
  {
    name: `lcm`,
    slug: `lcm`,
    category: `Math`,
    summary: `Least common multiple`,
    related: [],
    examples: [],
    tags: [`lcm`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Least common multiple


## Syntax

\`\`\`
lcm(a, b)
\`\`\`

## Description

Least common multiple

## Mathematical Formulation

$$ \\operatorname{lcm}(a,b) = \\frac{|a\\,b|}{\\gcd(a,b)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `linspace`,
    slug: `linspace`,
    category: `Math`,
    summary: `n linearly spaced values`,
    related: [],
    examples: [],
    tags: [`linspace`, `math`],
    references: [],
    guides: [`errors`, `matrices-decl`, `tut-msd`, `tut-rlc`],
    body: `n linearly spaced values


## Syntax

\`\`\`
linspace(a, b, n)
\`\`\`

## Description

n linearly spaced values

## Mathematical Formulation

$$ x_k = a + (b-a)\\,\\frac{k-1}{n-1}, \\quad k = 1,\\dots,n $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `ln`,
    slug: `ln`,
    category: `Math`,
    summary: `Natural logarithm (base e).`,
    related: [`exp`, `log10`, `log2`],
    examples: [`karman-rocket`],
    tags: [`math`, `logarithm`, `natural log`, `elementary`],
    references: [],
    guides: [`math-funcs`],
    body: `Returns the **natural logarithm** \`ln(x)\` (base e), the inverse of \`exp\`.
The argument must be a positive dimensionless number.

## Syntax

\`\`\`
y = ln(x)
\`\`\`

## Description

A standard elementary function. For base-10 or base-2 use \`log10\` /
\`log2\`.

## Mathematical Formulation

$$ y = \\ln(x) = \\log_e(x), \\qquad x > 0 $$

## Examples

### Example 1 — Rocket equation mass ratio

[Run: karman-rocket]

**Expected:** \`ln\` of the mass ratio gives the ideal velocity increment
(Tsiolkovsky).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Positive dimensionless value. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Natural logarithm of \`x\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | The logarithm requires a positive argument. |`,
  },
  {
    name: `log10`,
    slug: `log10`,
    category: `Math`,
    summary: `Base-10 logarithm`,
    related: [],
    examples: [],
    tags: [`log10`, `math`],
    references: [],
    guides: [`math-funcs`, `variables`],
    body: `Base-10 logarithm


## Syntax

\`\`\`
log10(x)
\`\`\`

## Description

Base-10 logarithm

## Mathematical Formulation

$$ y = \\log_{10}(x) = \\frac{\\ln x}{\\ln 10}, \\quad x > 0 $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `log2`,
    slug: `log2`,
    category: `Math`,
    summary: `Base-2 logarithm.`,
    related: [`ln`, `log10`, `exp`],
    examples: [],
    tags: [`math`, `logarithm`, `base-2`, `binary`, `elementary`],
    references: [],
    guides: [`math-funcs`],
    body: `Returns the **base-2 logarithm** \`log₂(x)\`. The argument must be a positive
dimensionless number. Common for information-theoretic quantities (bits) and
anything counted in powers of two.

## Syntax

\`\`\`
y = log2(x)
\`\`\`

## Description

A standard elementary function. For the natural log use \`ln\`; for base-10 use
\`log10\`. All three share the change-of-base identity $\\log_2 x = \\ln x / \\ln 2$.

## Mathematical Formulation

$$ y = \\log_2(x) = \\frac{\\ln x}{\\ln 2}, \\qquad x > 0 $$

> **Method:** evaluated as \`ln(x) / ln(2)\`; differentiated as
> $\\frac{dy}{dx} = \\frac{1}{x\\,\\ln 2}$ for Jacobians.

## Examples

### Example 1 — Bits needed to encode N states

\`\`\`
{ Number of bits to address 1024 states }
N = 1024
bits = log2(N)       { 10 }
\`\`\`

**Expected:** \`bits = 10\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Positive dimensionless value. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Base-2 logarithm of \`x\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | The logarithm requires a positive argument. |`,
  },
  {
    name: `mod`,
    slug: `mod`,
    category: `Math`,
    summary: `Modulo operation`,
    related: [],
    examples: [],
    tags: [`mod`, `math`],
    references: [],
    guides: [`math-funcs`, `functions`],
    body: `Modulo operation


## Syntax

\`\`\`
mod(x, y)
\`\`\`

## Description

Modulo operation

## Mathematical Formulation

$$ \\operatorname{mod}(a,b) = a - b\\,\\lfloor a/b \\rfloor $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`y\` | Number | Yes | Value / second coordinate. |`,
  },
  {
    name: `ones`,
    slug: `ones`,
    category: `Math`,
    summary: `m×n ones matrix`,
    related: [],
    examples: [],
    tags: [`ones`, `math`],
    references: [],
    guides: [`matrices-decl`],
    body: `m×n ones matrix


## Syntax

\`\`\`
ones(m, n)
\`\`\`

## Description

m×n ones matrix

## Mathematical Formulation

$$ J_{ij} = 1 \\quad (m\\times n) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`m\` | Number | Yes | Shape / form parameter. |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `product`,
    slug: `product`,
    category: `Math`,
    summary: `Product series Pi(term) over i = lo..hi`,
    related: [],
    examples: [],
    tags: [`product`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Product series Pi(term) over i = lo..hi


## Syntax

\`\`\`
product(i, lo, hi, term)
\`\`\`

## Description

Product series Pi(term) over i = lo..hi

## Mathematical Formulation

$$ \\prod_{i=\\text{lo}}^{\\text{hi}} \\text{term}(i) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`i\` | Number | Yes | Index. |
| \`lo\` | Number | Yes | Lower bound. |
| \`hi\` | Number | Yes | Upper bound. |
| \`term\` | Number | Yes | Series-term expression. |`,
  },
  {
    name: `round`,
    slug: `round`,
    category: `Math`,
    summary: `Round to nearest integer`,
    related: [],
    examples: [],
    tags: [`round`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Round to nearest integer


## Syntax

\`\`\`
round(x)
\`\`\`

## Description

Round to nearest integer

## Mathematical Formulation

$$ \\operatorname{round}(x, d) = \\frac{\\lfloor x\\cdot 10^{d} + 0.5\\rfloor}{10^{d}} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `sign`,
    slug: `sign`,
    category: `Math`,
    summary: `Sign function (-1, 0, or 1)`,
    related: [],
    examples: [],
    tags: [`sign`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Sign function (-1, 0, or 1)


## Syntax

\`\`\`
sign(x)
\`\`\`

## Description

Sign function (-1, 0, or 1)

## Mathematical Formulation

$$ \\operatorname{sign}(x) = \\begin{cases} -1 & x<0 \\\\ 0 & x=0 \\\\ 1 & x>0 \\end{cases} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `sin`,
    slug: `sin`,
    category: `Math`,
    summary: `Sine of an angle (radians).`,
    related: [`cos`, `tan`, `arcsin`, `atan2`],
    examples: [`projectile-motion`, `projectile-trajectory`],
    tags: [`math`, `trigonometry`, `sine`, `radians`, `elementary`],
    references: [],
    guides: [`optimization`, `math-funcs`],
    body: `Returns the **sine** of \`x\`. The argument is in **radians** unless it carries a
\`[deg]\` unit annotation (which frees converts automatically).

## Syntax

\`\`\`
y = sin(x)
\`\`\`

## Description

A standard trigonometric function. Use \`x [deg]\` or \`Convert\` to work in degrees;
bare numeric arguments are radians.

## Mathematical Formulation

$$ y = \\sin(x), \\qquad x \\text{ in radians} $$

## Examples

### Example 1 — Launch-angle component of velocity

[Run: projectile-motion]

**Expected:** \`sin\` of the launch angle gives the vertical velocity component.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Angle in radians (or \`[deg]\`-annotated). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Sine of \`x\` (dimensionless, in [−1, 1]). |`,
  },
  {
    name: `sinh`,
    slug: `sinh`,
    category: `Math`,
    summary: `Hyperbolic sine`,
    related: [],
    examples: [],
    tags: [`sinh`, `math`],
    references: [],
    guides: [`math-funcs`],
    body: `Hyperbolic sine


## Syntax

\`\`\`
sinh(x)
\`\`\`

## Description

Hyperbolic sine

## Mathematical Formulation

$$ \\sinh(x) = \\frac{e^{x} - e^{-x}}{2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `SolveLinear`,
    slug: `solvelinear`,
    category: `Math`,
    summary: `Solve A·x = b (same as A \\\\ b)`,
    related: [],
    examples: [],
    tags: [`solvelinear`, `math`],
    references: [],
    guides: [`matrices-ops`, `matrices-sys`],
    body: `Solve A·x = b (same as A \\\\ b)


## Syntax

\`\`\`
SolveLinear(A, b)
\`\`\`

## Description

Solve A·x = b (same as A \\\\ b)

## Mathematical Formulation

$$ A\\,x = b \\;\\Rightarrow\\; x = A^{-1}b \\quad\\text{(via } PA = LU\\text{, forward/back substitution)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Matrix. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `sqrt`,
    slug: `sqrt`,
    category: `Math`,
    summary: `Square root of a number (with unit propagation).`,
    related: [`cbrt`, `exp`, `abs`],
    examples: [`tank-draining`, `ev-thermal-management`],
    tags: [`math`, `square root`, `elementary`],
    references: [],
    guides: [`calculus`, `comp-authoring`, `repl`, `math-funcs`, `variables`, `tut-msd`],
    body: `Returns the **square root** \`√x\`. In frees it also propagates units — the result
carries the square root of the argument's unit (e.g. \`sqrt(m^2) → m\`).

## Syntax

\`\`\`
y = sqrt(x)
\`\`\`

## Description

A standard elementary function. The argument must be non-negative for a real
result; a negative argument yields a complex value only in complex-aware contexts.

## Mathematical Formulation

$$ y = \\sqrt{x} = x^{1/2}, \\qquad x \\ge 0 $$

## Examples

### Example 1 — Discharge velocity in a draining tank

[Run: tank-draining]

**Expected:** \`sqrt\` of the head term gives the Torricelli outflow velocity.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Non-negative value (any unit). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | The square root, with the square-root unit. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x < 0\` in a real context | Ensure the argument is non-negative. |`,
  },
  {
    name: `tan`,
    slug: `tan`,
    category: `Math`,
    summary: `Tangent of x`,
    related: [],
    examples: [],
    tags: [`tan`, `math`],
    references: [],
    guides: [],
    body: `Tangent of x


## Syntax

\`\`\`
tan(x)
\`\`\`

## Description

Tangent of x

## Mathematical Formulation

$$ \\tan(x) = \\frac{\\sin x}{\\cos x}, \\qquad x \\text{ in radians} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `tanh`,
    slug: `tanh`,
    category: `Math`,
    summary: `Hyperbolic tangent`,
    related: [],
    examples: [],
    tags: [`tanh`, `math`],
    references: [],
    guides: [`verification`],
    body: `Hyperbolic tangent


## Syntax

\`\`\`
tanh(x)
\`\`\`

## Description

Hyperbolic tangent

## Mathematical Formulation

$$ \\tanh(x) = \\frac{\\sinh x}{\\cosh x} = \\frac{e^{x}-e^{-x}}{e^{x}+e^{-x}} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `trunc`,
    slug: `trunc`,
    category: `Math`,
    summary: `Discard the fractional part (round toward zero)`,
    related: [],
    examples: [],
    tags: [`trunc`, `math`],
    references: [],
    guides: [`math-funcs`, `functions`],
    body: `Discard the fractional part (round toward zero)


## Syntax

\`\`\`
trunc(x)
\`\`\`

## Description

Discard the fractional part (round toward zero)

## Mathematical Formulation

$$ \\operatorname{trunc}(x) = \\operatorname{sign}(x)\\,\\lfloor |x| \\rfloor $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `zeros`,
    slug: `zeros`,
    category: `Math`,
    summary: `m×n zero matrix`,
    related: [],
    examples: [`cruise-control`],
    tags: [`zeros`, `math`],
    references: [],
    guides: [`matrices-decl`],
    body: `m×n zero matrix


## Syntax

\`\`\`
zeros(m, n)
\`\`\`

## Description

m×n zero matrix

## Mathematical Formulation

$$ Z_{ij} = 0 \\quad (m\\times n) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`m\` | Number | Yes | Shape / form parameter. |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `cholesky`,
    slug: `cholesky`,
    category: `Matrix`,
    summary: `Cholesky decomposition`,
    related: [],
    examples: [],
    tags: [`cholesky`, `matrix`],
    references: [],
    guides: [],
    body: `Cholesky decomposition


## Syntax

\`\`\`
cholesky(A : L)
\`\`\`

## Description

Cholesky decomposition

## Mathematical Formulation

$$ A = L\\,L^\\top \\quad\\text{(} A \\text{ symmetric positive-definite)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`L\` | Number/Array | Length [m]. |`,
  },
  {
    name: `cond`,
    slug: `cond`,
    category: `Matrix`,
    summary: `Condition number of a matrix (sensitivity to perturbations).`,
    related: [`norm`, `rank`, `inv`, `svd`],
    examples: [`ev-thermal-management`],
    tags: [`matrix`, `condition number`, `conditioning`, `svd`, `linear algebra`],
    references: [],
    guides: [],
    body: `Returns the **condition number** of a matrix \`A\` — the ratio of its largest to
smallest singular value. It measures how much relative error in the data can be
amplified when solving \`Ax = b\`; a large value flags near-singularity and
ill-conditioning.

## Syntax

\`\`\`
c = cond(A)
\`\`\`

## Description

A condition number near 1 indicates a well-conditioned problem; very large values
mean small input changes can produce large output changes, so solutions should be
treated with caution.

## Mathematical Formulation

$$ \\kappa(A) = \\|A\\|\\,\\|A^{-1}\\| = \\frac{\\sigma_{\\max}(A)}{\\sigma_{\\min}(A)} $$

where the \`σ\` are the singular values of \`A\`.

> **Method:** singular value decomposition; \`κ = σ_max/σ_min\`.

## Examples

### Example 1 — Conditioning check in a coupled solve

[Run: ev-thermal-management]

**Expected:** a finite condition number; a very large value would warn that the
linear system is ill-conditioned.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | The matrix to assess. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`c\` | Number | Condition number κ(A) ≥ 1. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`SINGULAR_MATRIX\` | \`σ_min = 0\` | The matrix is singular — condition number is infinite. |`,
  },
  {
    name: `det`,
    slug: `det`,
    category: `Matrix`,
    summary: `Matrix determinant`,
    related: [],
    examples: [],
    tags: [`det`, `matrix`],
    references: [],
    guides: [],
    body: `Matrix determinant


## Syntax

\`\`\`
det(A)
\`\`\`

## Description

Matrix determinant

## Mathematical Formulation

$$ \\det(A) = \\pm\\prod_i U_{ii} \\quad\\text{(from } PA = LU\\text{)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `diag`,
    slug: `diag`,
    category: `Matrix`,
    summary: `Diagonal matrix built from a vector.`,
    related: [`eye`, `zeros`, `Transpose`],
    examples: [`estimator-gramian-balreal`],
    tags: [`matrix`, `diagonal`, `construction`, `linear algebra`],
    references: [],
    guides: [`matrices-decl`],
    body: `Builds a square **diagonal matrix** whose diagonal is the supplied vector and whose
off-diagonal entries are zero. Common for assembling weighting matrices (e.g. the
\`Q\`/\`R\` of an LQR/LQE design).

## Syntax

\`\`\`
M = diag(v)
\`\`\`

## Description

For a length-\`n\` vector \`v\`, returns the \`n×n\` matrix \`M\` with \`M[i,i] = v[i]\`.

## Mathematical Formulation

$$ M_{ij} = \\begin{cases} v_i & i = j \\\\ 0 & i \\neq j \\end{cases} $$

## Examples

### Example 1 — Weighting matrix for an estimator design

[Run: estimator-gramian-balreal]

**Expected:** a diagonal matrix used as a noise/weighting matrix in the design.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`v\` | Vector | Yes | The diagonal entries. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`M\` | Matrix | \`n×n\` diagonal matrix. |`,
  },
  {
    name: `eig`,
    slug: `eig`,
    category: `Matrix`,
    summary: `Eigenvalues of A`,
    related: [],
    examples: [],
    tags: [`eig`, `matrix`],
    references: [],
    guides: [],
    body: `Eigenvalues of A


## Syntax

\`\`\`
eig(A)
\`\`\`

## Description

Eigenvalues of A

## Mathematical Formulation

$$ A v = \\lambda v, \\qquad \\det(A - \\lambda I) = 0 $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `Eigen`,
    slug: `eigen`,
    category: `Matrix`,
    summary: `Eigenvalues and eigenvectors of a square matrix.`,
    related: [`Eigenvalues`, `balreal`],
    examples: [],
    tags: [`matrix`, `eigenvalues`, `eigenvectors`, `spectral`, `linear algebra`],
    references: [],
    guides: [`matrices-sys`],
    body: `Returns the **eigenvalues** \`lambda\` and **eigenvectors** \`V\` of a square matrix
\`A\` — the full eigendecomposition \`A V = V Λ\`. The eigenvectors give the modal
directions; the eigenvalues their rates/frequencies.

## Syntax

\`\`\`
CALL Eigen(A : lambda, V)
[lambda, V] = Eigen(A)
\`\`\`

## Mathematical Formulation

$$ A\\,v_i = \\lambda_i\\,v_i, \\qquad A = V\\,\\Lambda\\,V^{-1} $$

where \`Λ = diag(λ_i)\` and the columns of \`V\` are the eigenvectors.

> **Method:** QR algorithm with eigenvector back-substitution.

Eigen supports **real spectra only** (symmetric matrices always qualify) and
stops with an error on complex eigenvalues; for a complex spectrum use
\`CALL Eigenvalues(A : re, im)\`, which returns real/imaginary part vectors.

## Examples

\`\`\`
{ [lambda, V] = Eigen(A) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | Square matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`lambda\` | Vector | Eigenvalues. |
| \`V\` | Matrix | Eigenvectors (columns). |`,
  },
  {
    name: `Eigenvalues`,
    slug: `eigenvalues`,
    category: `Matrix`,
    summary: `Eigenvalues of a square matrix.`,
    related: [`Eigen`, `Determinant`, `cond`],
    examples: [],
    tags: [`matrix`, `eigenvalues`, `spectrum`, `linear algebra`],
    references: [],
    guides: [`repl`, `matrices-sys`],
    body: `Returns the **eigenvalues** \`lambda\` of a square matrix \`A\` — the scalars \`λ\` for
which \`A v = λ v\` has a nonzero solution. They set system stability (continuous:
left half-plane; discrete: inside the unit circle) and modal frequencies.

## Syntax

\`\`\`
CALL Eigenvalues(A : lambda)
CALL Eigenvalues(A : re, im)
lambda = Eigenvalues(A)
\`\`\`

## Mathematical Formulation

The eigenvalues are the roots of the characteristic polynomial:

$$ \\det(A - \\lambda I) = 0 $$

> **Method:** QR algorithm on the (balanced) matrix.

The single-output form supports **real spectra only** (symmetric matrices always
qualify) and stops with an error on complex eigenvalues. The two-output form
carries a complex spectrum as real/imaginary part vectors; eigenvalues are
sorted ascending by real part, then imaginary part.

## Examples

\`\`\`
{ lambda = Eigenvalues(A) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | Square matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`lambda\` | Vector | Eigenvalues, ascending (single-output form; real spectra only). |
| \`re\`, \`im\` | Vector | Real and imaginary parts of the spectrum (two-output form). |`,
  },
  {
    name: `eigvec`,
    slug: `eigvec`,
    category: `Matrix`,
    summary: `Eigenvectors of A`,
    related: [],
    examples: [],
    tags: [`eigvec`, `matrix`],
    references: [],
    guides: [],
    body: `Eigenvectors of A


## Syntax

\`\`\`
eigvec(A)
\`\`\`

## Description

Eigenvectors of A

## Mathematical Formulation

$$ A v_i = \\lambda_i v_i \\quad\\text{(columns are the eigenvectors)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `EulerRotate`,
    slug: `eulerrotate`,
    category: `Matrix`,
    summary: `3×3 rotation matrix from Euler angles (φ, θ, ψ).`,
    related: [`Eigen`, `Transpose`],
    examples: [],
    tags: [`matrix`, `rotation`, `euler angles`, `kinematics`, `attitude`],
    references: [],
    guides: [`matrices-sys`],
    body: `Returns the **3×3 rotation matrix** \`R\` corresponding to a sequence of Euler-angle
rotations \`(φ, θ, ψ)\`. Use it for rigid-body attitude, coordinate-frame transforms,
and vehicle/spacecraft kinematics.

## Syntax

\`\`\`
CALL EulerRotate(phi, theta, psi : R)
R = EulerRotate(phi, theta, psi)
\`\`\`

## Description

The angles are applied as elementary rotations about successive axes; the product
is an orthonormal rotation (\`Rᵀ = R⁻¹\`, \`det R = 1\`).

## Mathematical Formulation

The rotation is the product of three elementary rotations:

$$ R(\\phi, \\theta, \\psi) = R_z(\\psi)\\,R_x(\\theta)\\,R_z(\\phi), \\qquad R^\\top R = I,\\ \\det R = 1 $$

(the standard \`z–x–z\` convention).

> **Method:** multiply the three elementary axis rotations.

## Examples

\`\`\`
{ R = EulerRotate(phi, theta, psi) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`phi\` | Number | Yes | First rotation angle [rad]. |
| \`theta\` | Number | Yes | Second rotation angle [rad]. |
| \`psi\` | Number | Yes | Third rotation angle [rad]. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`R\` | Matrix | 3×3 orthonormal rotation matrix. |`,
  },
  {
    name: `inv`,
    slug: `inv`,
    category: `Matrix`,
    summary: `Matrix inverse`,
    related: [],
    examples: [],
    tags: [`inv`, `matrix`],
    references: [],
    guides: [],
    body: `Matrix inverse


## Syntax

\`\`\`
inv(A)
\`\`\`

## Description

Matrix inverse

## Mathematical Formulation

$$ A\\,A^{-1} = A^{-1}A = I $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `LUDecompose`,
    slug: `ludecompose`,
    category: `Matrix`,
    summary: `LU decomposition of a matrix (A = L·U).`,
    related: [`SolveLinear`, `Inverse`, `Determinant`],
    examples: [],
    tags: [`matrix`, `lu decomposition`, `factorization`, `linear solve`],
    references: [],
    guides: [`matrices-sys`],
    body: `Returns the **LU decomposition** of a square matrix \`A\` — a lower-triangular \`L\`
and upper-triangular \`U\` whose product is \`A\` (with partial pivoting). It is the
workhorse factorization behind linear solves and determinants.

## Syntax

\`\`\`
CALL LUDecompose(A : L, U)
[L, U] = LUDecompose(A)
\`\`\`

## Mathematical Formulation

With a permutation \`P\` for partial pivoting:

$$ P A = L U $$

where \`L\` is unit-lower-triangular and \`U\` upper-triangular. Then \`det(A) = ±∏ U_{ii}\`.

> **Method:** Gaussian elimination with partial pivoting.

## Examples

\`\`\`
{ [L, U] = LUDecompose(A) }
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Matrix | Yes | Square matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`L\` | Matrix | Lower-triangular factor. |
| \`U\` | Matrix | Upper-triangular factor. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`SINGULAR_MATRIX\` | a zero pivot remains | The matrix is singular; LU is not unique. |`,
  },
  {
    name: `matexp`,
    slug: `matexp`,
    category: `Matrix`,
    summary: `Matrix exponential`,
    related: [],
    examples: [],
    tags: [`matexp`, `matrix`],
    references: [],
    guides: [],
    body: `Matrix exponential


## Syntax

\`\`\`
matexp(A)
\`\`\`

## Description

Matrix exponential

## Mathematical Formulation

$$ e^{A} = \\sum_{k=0}^{\\infty} \\frac{A^{k}}{k!} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `norm`,
    slug: `norm`,
    category: `Matrix`,
    summary: `Matrix norm`,
    related: [],
    examples: [],
    tags: [`norm`, `matrix`],
    references: [],
    guides: [`matrices-sys`],
    body: `Matrix norm


## Syntax

\`\`\`
norm(A)
\`\`\`

## Description

Matrix norm

## Mathematical Formulation

$$ \\lVert v \\rVert_2 = \\sqrt{\\textstyle\\sum_i v_i^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `qr`,
    slug: `qr`,
    category: `Matrix`,
    summary: `QR decomposition`,
    related: [],
    examples: [],
    tags: [`qr`, `matrix`],
    references: [],
    guides: [],
    body: `QR decomposition


## Syntax

\`\`\`
qr(A : Q, R)
\`\`\`

## Description

QR decomposition

## Mathematical Formulation

$$ A = Q\\,R, \\qquad Q^\\top Q = I,\\ R\\ \\text{upper triangular} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Q\` | Number/Array | Computed \`Q\`. |
| \`R\` | Number/Array | Computed \`R\`. |`,
  },
  {
    name: `rank`,
    slug: `rank`,
    category: `Matrix`,
    summary: `Matrix rank`,
    related: [],
    examples: [],
    tags: [`rank`, `matrix`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Matrix rank


## Syntax

\`\`\`
rank(A)
\`\`\`

## Description

Matrix rank

## Mathematical Formulation

$$ \\operatorname{rank}(A) = \\#\\{\\sigma_i > \\text{tol}\\} \\quad\\text{(numerical, via SVD)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `svd`,
    slug: `svd`,
    category: `Matrix`,
    summary: `Singular value decomposition`,
    related: [],
    examples: [],
    tags: [`svd`, `matrix`],
    references: [],
    guides: [`symbolic-cas`],
    body: `Singular value decomposition


## Syntax

\`\`\`
svd(A : U, S, V)
\`\`\`

## Description

Singular value decomposition

## Mathematical Formulation

$$ A = U\\,\\Sigma\\,V^\\top, \\qquad \\Sigma = \\operatorname{diag}(\\sigma_1 \\ge \\dots \\ge \\sigma_r > 0) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`U\` | Number/Array | Computed \`U\`. |
| \`S\` | Number/Array | Nucleate-suppression factor. |
| \`V\` | Number/Array | Velocity [m/s]. |`,
  },
  {
    name: `trace`,
    slug: `trace`,
    category: `Matrix`,
    summary: `Matrix trace`,
    related: [],
    examples: [],
    tags: [`trace`, `matrix`],
    references: [],
    guides: [],
    body: `Matrix trace


## Syntax

\`\`\`
trace(A)
\`\`\`

## Description

Matrix trace

## Mathematical Formulation

$$ \\operatorname{tr}(A) = \\sum_i A_{ii} = \\sum_i \\lambda_i $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `transpose`,
    slug: `transpose`,
    category: `Matrix`,
    summary: `Matrix transpose`,
    related: [],
    examples: [],
    tags: [`transpose`, `matrix`],
    references: [],
    guides: [`repl`],
    body: `Matrix transpose


## Syntax

\`\`\`
transpose(A)
\`\`\`

## Description

Matrix transpose

## Mathematical Formulation

$$ (A^\\top)_{ij} = A_{ji} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`A\` | Number | Yes | Square input matrix. |`,
  },
  {
    name: `FinalValue`,
    slug: `finalvalue`,
    category: `ODE Results`,
    summary: `Last value of a column in the integrated ODE (DYNAMIC) table.`,
    related: [`MaxValue`, `MinValue`, `ODEValue`, `TimeAt`],
    examples: [`newton-cooling-transient`],
    tags: [`ode`, `dynamic`, `accessor`, `final value`, `transient`, `trajectory`],
    references: [],
    guides: [`dynamic-ode`, `comp-transient`, `tut-msd`],
    body: `Returns the **last value** of a named column of the integrated \`DYNAMIC\` (ODE)
result table — the value at the end of the integration window. Use it to feed a
transient result back into the analytic solve (e.g. close a sizing loop on a final
temperature).

## Syntax

\`\`\`
v = FinalValue('col')
\`\`\`

## Description

After a \`DYNAMIC\` block integrates, every state and auxiliary becomes a column of
the result table. \`FinalValue\` reads the last row of the requested column, so a
transient endpoint can drive an algebraic equation.

## Mathematical Formulation

For a column sampled at times $t_0 < t_1 < \\dots < t_N$,

$$ \\text{FinalValue}('col') = \\text{col}(t_N) $$

> **Method:** read the last sample of the column (no interpolation).

## Examples

### Example 1 — Final temperature of a cooling transient

[Run: newton-cooling-transient]

**Expected:** the temperature at the end of the integration window, used as a
scalar in the analytic part of the document.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | String | Yes | Name of a state or auxiliary column in the \`DYNAMIC\` table. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`v\` | Number | The column value at the final integration time. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_COLUMN\` | \`'col'\` is not a column of the ODE table | Use a state name or a declared auxiliary from the \`DYNAMIC\` block. |
| \`NO_DYNAMIC_RESULT\` | No \`DYNAMIC\` block has been integrated | Define and solve a \`DYNAMIC\` block first. |`,
  },
  {
    name: `MaxValue`,
    slug: `maxvalue`,
    category: `ODE Results`,
    summary: `Peak value of a column in the integrated ODE (DYNAMIC) table.`,
    related: [`MinValue`, `FinalValue`, `TimeAt`, `ODEValue`],
    examples: [`newton-cooling-transient`],
    tags: [`ode`, `dynamic`, `accessor`, `maximum`, `peak`, `transient`, `trajectory`],
    references: [],
    guides: [`dynamic-ode`, `comp-transient`, `tut-msd`],
    body: `Returns the **peak value** of a named column of the integrated \`DYNAMIC\` (ODE)
result table over the whole integration window. Use it to size against a transient
maximum — e.g. peak overshoot, peak temperature, or peak altitude.

## Syntax

\`\`\`
v = MaxValue('col')
\`\`\`

## Description

\`MaxValue\` scans the requested column across all integration samples and returns
its largest value, so a transient peak can drive an algebraic equation (a common
sizing pattern: \`MaxValue('h') = h_target\`).

## Mathematical Formulation

$$ \\text{MaxValue}('col') = \\max_{0 \\le i \\le N} \\text{col}(t_i) $$

> **Method:** maximum over the column's samples.

## Examples

### Example 1 — Peak of a transient

[Run: newton-cooling-transient]

**Expected:** the largest value the column reaches during the integration.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | String | Yes | Name of a state or auxiliary column in the \`DYNAMIC\` table. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`v\` | Number | The maximum of the column over the run. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_COLUMN\` | \`'col'\` is not a column of the ODE table | Use a state name or a declared auxiliary. |
| \`NO_DYNAMIC_RESULT\` | No \`DYNAMIC\` block integrated | Define and solve a \`DYNAMIC\` block first. |`,
  },
  {
    name: `minvalue`,
    slug: `minvalue`,
    category: `ODE Results`,
    summary: `Minimum value of an ODE column`,
    related: [],
    examples: [],
    tags: [`minvalue`, `ode`, `results`],
    references: [],
    guides: [`dynamic-ode`],
    body: `Minimum value of an ODE column


## Syntax

\`\`\`
MinValue('col')
\`\`\`

## Description

Minimum value of an ODE column

## Mathematical Formulation

$$ \\min_{0 \\le i \\le N} \\text{col}(t_i) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `odeavg`,
    slug: `odeavg`,
    category: `ODE Results`,
    summary: `Time-mean of an ODE column`,
    related: [],
    examples: [],
    tags: [`odeavg`, `ode`, `results`],
    references: [],
    guides: [],
    body: `Time-mean of an ODE column


## Syntax

\`\`\`
ODEAvg('col')
\`\`\`

## Description

Time-mean of an ODE column

## Mathematical Formulation

$$ \\frac{1}{N+1}\\sum_{i=0}^{N} \\text{col}(t_i) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `odemax`,
    slug: `odemax`,
    category: `ODE Results`,
    summary: `Maximum of an ODE column`,
    related: [],
    examples: [],
    tags: [`odemax`, `ode`, `results`],
    references: [],
    guides: [],
    body: `Maximum of an ODE column


## Syntax

\`\`\`
ODEMax('col')
\`\`\`

## Description

Maximum of an ODE column

## Mathematical Formulation

$$ \\max_i \\text{col}(t_i) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `odemin`,
    slug: `odemin`,
    category: `ODE Results`,
    summary: `Minimum of an ODE column`,
    related: [],
    examples: [],
    tags: [`odemin`, `ode`, `results`],
    references: [],
    guides: [],
    body: `Minimum of an ODE column


## Syntax

\`\`\`
ODEMin('col')
\`\`\`

## Description

Minimum of an ODE column

## Mathematical Formulation

$$ \\min_i \\text{col}(t_i) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `odestddev`,
    slug: `odestddev`,
    category: `ODE Results`,
    summary: `Standard deviation of an ODE column`,
    related: [],
    examples: [],
    tags: [`odestddev`, `ode`, `results`],
    references: [],
    guides: [],
    body: `Standard deviation of an ODE column


## Syntax

\`\`\`
ODEStdDev('col')
\`\`\`

## Description

Standard deviation of an ODE column

## Mathematical Formulation

$$ s = \\sqrt{\\tfrac{1}{N}\\sum_i (\\text{col}(t_i) - \\overline{\\text{col}})^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `odesum`,
    slug: `odesum`,
    category: `ODE Results`,
    summary: `Sum of an ODE column`,
    related: [],
    examples: [],
    tags: [`odesum`, `ode`, `results`],
    references: [],
    guides: [],
    body: `Sum of an ODE column


## Syntax

\`\`\`
ODESum('col')
\`\`\`

## Description

Sum of an ODE column

## Mathematical Formulation

$$ \\sum_{i=0}^{N} \\text{col}(t_i) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `ODEValue`,
    slug: `odevalue`,
    category: `ODE Results`,
    summary: `Value of an ODE (DYNAMIC) column interpolated at a given time.`,
    related: [`FinalValue`, `MaxValue`, `TimeAt`],
    examples: [`damped-oscillator-ode`],
    tags: [`ode`, `dynamic`, `accessor`, `interpolation`, `time`, `trajectory`],
    references: [],
    guides: [`dynamic-ode`],
    body: `Returns the value of a named \`DYNAMIC\` (ODE) column **interpolated at an arbitrary
time** \`t\` within the integration window. Use it to sample a transient at a
specific instant that need not coincide with an integration step.

## Syntax

\`\`\`
v = ODEValue('col', t)
\`\`\`

## Description

Because the adaptive integrator places samples unevenly, \`ODEValue\` linearly
interpolates the column between the bracketing samples to return the value at the
requested time.

## Mathematical Formulation

For \`t\` bracketed by samples $t_i \\le t \\le t_{i+1}$,

$$ \\text{ODEValue}('col', t) = \\text{col}(t_i) + \\big(\\text{col}(t_{i+1}) - \\text{col}(t_i)\\big)\\frac{t - t_i}{t_{i+1} - t_i} $$

> **Method:** linear interpolation between the two bracketing integration samples.

## Examples

### Example 1 — Sample a transient at a chosen instant

[Run: damped-oscillator-ode]

**Expected:** the column value at the requested time, interpolated from the ODE
trajectory.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | String | Yes | Name of a state or auxiliary column. |
| \`t\` | Number | Yes | Time at which to sample (within the integration window). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`v\` | Number | The interpolated column value at time \`t\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`TIME_OUT_OF_RANGE\` | \`t\` outside the integration window | Sample within \`[t0, tf]\` of the \`DYNAMIC\` block. |
| \`UNKNOWN_COLUMN\` | \`'col'\` not a column | Use a state name or declared auxiliary. |`,
  },
  {
    name: `TimeAt`,
    slug: `timeat`,
    category: `ODE Results`,
    summary: `Time at which an ODE (DYNAMIC) column first crosses a target value.`,
    related: [`ODEValue`, `FinalValue`, `MaxValue`],
    examples: [`newton-cooling-transient`],
    tags: [`ode`, `dynamic`, `accessor`, `crossing`, `time`, `event`, `trajectory`],
    references: [],
    guides: [`dynamic-ode`, `comp-transient`, `tut-msd`],
    body: `Returns the **time** at which a named \`DYNAMIC\` (ODE) column first crosses a target
value \`val\`. Use it to read out event times — when a temperature reaches a
threshold, a tank empties, or a response settles.

## Syntax

\`\`\`
t = TimeAt('col', val)
\`\`\`

## Description

\`TimeAt\` scans the column for the first interval that brackets \`val\`, then linearly
interpolates the crossing time — the inverse of \`ODEValue\`.

## Mathematical Formulation

For the first interval with $\\text{col}(t_i) \\le val \\le \\text{col}(t_{i+1})$ (or
the reverse),

$$ t = t_i + (t_{i+1} - t_i)\\,\\frac{val - \\text{col}(t_i)}{\\text{col}(t_{i+1}) - \\text{col}(t_i)} $$

> **Method:** locate the first bracketing interval, then linear inverse interpolation.

## Examples

### Example 1 — Time to reach a target temperature

[Run: newton-cooling-transient]

**Expected:** the instant the cooling curve first crosses the target value.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | String | Yes | Name of a state or auxiliary column. |
| \`val\` | Number | Yes | Target value to cross. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`t\` | Number | Time of the first crossing. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`NO_CROSSING\` | The column never reaches \`val\` | Check the target is within the column's range over the run. |
| \`UNKNOWN_COLUMN\` | \`'col'\` not a column | Use a state name or declared auxiliary. |`,
  },
  {
    name: `iso6358`,
    slug: `iso6358`,
    category: `Pneumatics`,
    summary: `ISO 6358 pneumatic mass flow [kg/s] (sonic conductance C, critical ratio b)`,
    related: [],
    examples: [],
    tags: [`iso6358`, `pneumatics`],
    references: [`ISO 6358 — Pneumatic fluid power: flow-rate characteristics`],
    guides: [],
    body: `ISO 6358 pneumatic mass flow [kg/s] (sonic conductance C, critical ratio b)


## Syntax

\`\`\`
iso6358(C, b, Pup, Tup, Pdown)
\`\`\`

## Description

ISO 6358 pneumatic mass flow [kg/s] (sonic conductance C, critical ratio b)

## Mathematical Formulation

$$ \\dot m = C\\,\\rho_0\\,P_{up}\\sqrt{\\tfrac{T_0}{T_{up}}}\\cdot\\begin{cases} 1 & P_{down}/P_{up} \\le b \\\\ \\sqrt{1 - \\big(\\tfrac{P_{down}/P_{up} - b}{1-b}\\big)^2} & \\text{else} \\end{cases} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`C\` | Number | Yes | Empirical constant. |
| \`b\` | Number | Yes | Second operand. |
| \`Pup\` | Number | Yes | Upstream pressure [Pa]. |
| \`Tup\` | Number | Yes | Upstream temperature [K]. |
| \`Pdown\` | Number | Yes | Downstream pressure [Pa]. |

## References

1. ISO 6358 — Pneumatic fluid power: determination of flow-rate characteristics.`,
  },
  {
    name: `eos_density`,
    slug: `eos_density`,
    category: `Properties (EOS)`,
    summary: `Mass density from a cubic equation of state (SRK or PR).`,
    related: [`eos_z`, `eos_volume`, `eos_enthalpy`],
    examples: [`cubic-eos-properties`],
    tags: [`eos`, `cubic`, `peng-robinson`, `srk`, `density`, `real gas`],
    references: [],
    guides: [],
    body: `Returns the **mass density** \`ρ\` [kg/m³] of a real fluid from a cubic equation of
state (\`'SRK'\` or \`'PR'\`) at temperature \`T\` and pressure \`P\`. The real-gas density that needs no
real-fluid property data, the reciprocal of \`eos_volume\`.

## Syntax

\`\`\`
rho = eos_density(fluid$, model$, T, P, phase$)
\`\`\`

## Description

The density follows from the EOS compressibility factor; near the critical region
it deviates strongly from the ideal-gas value \`P/(RT)\`.

## Mathematical Formulation

$$ \\rho = \\frac{1}{v} = \\frac{P}{Z\\,R\\,T} $$

with \`Z\` the EOS root for the requested phase and \`R\` the specific gas constant.

> **Method:** \`eos_z\` root → \`ρ = P/(ZRT)\`.

## Examples

### Example 1 — CO₂ density near the critical point

[Run: cubic-eos-properties]

**Expected (approx.):** at 6 MPa, 320 K (PR), \`ρ ≈ 140 kg/m³\` (\`Z ≈ 0.7\`) — far
above the ideal-gas estimate because \`Z < 1\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name. |
| \`model$\` | String | Yes | \`'SRK'\` or \`'PR'\`. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`phase$\` | String | Yes | Root selector: \`'vapor'\` or \`'liquid'\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`rho\` | Number | Mass density [kg/m³]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_FLUID\` | \`fluid$\` not in the table | Use a supported fluid name. |`,
  },
  {
    name: `eos_enthalpy`,
    slug: `eos_enthalpy`,
    category: `Properties (EOS)`,
    summary: `Specific enthalpy from a cubic equation of state (ideal-gas + departure).`,
    related: [`eos_z`, `eos_entropy`, `eos_density`],
    examples: [`cubic-eos-properties`],
    tags: [`eos`, `cubic`, `peng-robinson`, `srk`, `enthalpy`, `departure`, `residual`],
    references: [],
    guides: [],
    body: `Returns the **specific enthalpy** \`h\` [J/kg] of a real fluid from a cubic equation
of state (\`'SRK'\` or \`'PR'\`) at temperature \`T\` and pressure \`P\`. It combines the
ideal-gas enthalpy with the EOS **departure (residual) enthalpy** that captures
real-gas effects.

## Syntax

\`\`\`
h = eos_enthalpy(fluid$, model$, T, P, phase$)
\`\`\`

## Description

Enthalpy is built as the ideal-gas contribution plus a departure term derived from
the equation of state — the analytic real-gas correction to the ideal value.

## Mathematical Formulation

$$ h(T,P) = h^{\\text{ig}}(T) + \\big(h - h^{\\text{ig}}\\big)_{T,P} $$

where the departure is the residual from the EOS:

$$ h - h^{\\text{ig}} = RT\\,(Z-1) + \\frac{T\\,\\dfrac{da}{dT} - a}{2\\sqrt{2}\\,b}\\,\\ln\\!\\left[\\frac{Z + (1+\\sqrt2)B}{Z + (1-\\sqrt2)B}\\right] $$

(Peng–Robinson form; the SRK departure uses the corresponding \`ln[(Z+B)/Z]\` term).

> **Method:** ideal-gas enthalpy + the closed-form EOS departure evaluated at the
> \`eos_z\` root.

## Examples

### Example 1 — CO₂ real-gas enthalpy

[Run: cubic-eos-properties]

**Expected:** the value lies **below** the ideal-gas enthalpy at the same \`T\`
(negative departure near the critical region), reflecting attractive real-gas
forces.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name. |
| \`model$\` | String | Yes | \`'SRK'\` or \`'PR'\`. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`phase$\` | String | Yes | Root selector: \`'vapor'\` or \`'liquid'\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`h\` | Number | Specific enthalpy [J/kg] (relative to the EOS reference). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_FLUID\` | \`fluid$\` not in the table | Use a supported fluid name. |`,
  },
  {
    name: `eos_entropy`,
    slug: `eos_entropy`,
    category: `Properties (EOS)`,
    summary: `Specific entropy [J/kg-K] (SRK/PR)`,
    related: [],
    examples: [],
    tags: [`eos`, `entropy`, `properties`],
    references: [],
    guides: [],
    body: `Specific entropy [J/kg-K] (SRK/PR)


## Syntax

\`\`\`
eos_entropy(fluid$, model$, T, P, phase$)
\`\`\`

## Description

Specific entropy [J/kg-K] (SRK/PR)

## Mathematical Formulation

$$ s(T,P) = s^{\\text{ig}}(T,P) + (s - s^{\\text{ig}})_{T,P} \\quad\\text{(ideal-gas + EOS departure)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name (e.g. Water, R134a, Air). |
| \`model$\` | String | Yes | Selector — One of \`SRK\`, \`PR\`. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`phase$\` | String | Yes | Selector — One of \`vapor\`, \`liquid\`. |`,
  },
  {
    name: `eos_pressure`,
    slug: `eos_pressure`,
    category: `Properties (EOS)`,
    summary: `Pressure [Pa] from (T, specific volume)`,
    related: [],
    examples: [],
    tags: [`eos`, `pressure`, `properties`],
    references: [],
    guides: [],
    body: `Pressure [Pa] from (T, specific volume)


## Syntax

\`\`\`
eos_pressure(fluid$, model$, T, v)
\`\`\`

## Description

Pressure [Pa] from (T, specific volume)

## Mathematical Formulation

$$ P = \\frac{RT}{v-b} - \\frac{a\\,\\alpha(T)}{v(v+b) + b(v-b)} \\quad\\text{(PR; from } T, v) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name (e.g. Water, R134a, Air). |
| \`model$\` | String | Yes | Selector — One of \`SRK\`, \`PR\`. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`v\` | Number | Yes | Specific volume [m³/kg]. |`,
  },
  {
    name: `eos_psat`,
    slug: `eos_psat`,
    category: `Properties (EOS)`,
    summary: `Saturation pressure from a cubic EOS via the equal-fugacity condition.`,
    related: [`eos_z`, `eos_enthalpy`],
    examples: [`cubic-eos-properties`],
    tags: [`eos`, `cubic`, `peng-robinson`, `srk`, `saturation pressure`, `fugacity`, `vapor pressure`],
    references: [],
    guides: [],
    body: `Returns the **saturation (vapor) pressure** \`Psat\` [Pa] of a fluid at temperature
\`T\` from a cubic equation of state (\`'SRK'\` or \`'PR'\`), found by enforcing
equal liquid and vapor fugacities (the phase-equilibrium condition).

## Syntax

\`\`\`
Psat = eos_psat(fluid$, model$, T)
\`\`\`

## Description

At a given \`T\`, the saturation pressure is the unique pressure at which the EOS
admits coexisting liquid and vapor roots with equal fugacity — the cubic-EOS
analogue of the Maxwell equal-area construction.

## Mathematical Formulation

\`Psat(T)\` is the pressure satisfying the isofugacity condition

$$ f^{\\,L}(T, P_{sat}) = f^{\\,V}(T, P_{sat}) \\quad\\Longleftrightarrow\\quad \\varphi^{L} = \\varphi^{V} $$

where the fugacity coefficient from the cubic EOS is

$$ \\ln\\varphi = (Z-1) - \\ln(Z-B) - \\frac{A}{2\\sqrt2\\,B}\\ln\\!\\left[\\frac{Z+(1+\\sqrt2)B}{Z+(1-\\sqrt2)B}\\right] $$

(Peng–Robinson). The liquid and vapor \`Z\` roots are evaluated at trial \`P\` and the
pressure is iterated until $\\varphi^L = \\varphi^V$.

> **Method:** root-find on \`P\` so that the liquid/vapor fugacity coefficients match.

## Examples

### Example 1 — CO₂ vapor pressure at 300 K

[Run: cubic-eos-properties]

**Expected:** \`eos_psat('co2', 'PR', 300) ≈ 6.7 MPa\` (CO₂ vapor pressure at 300 K;
the critical point is 304 K / 7.38 MPa).

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name. |
| \`model$\` | String | Yes | \`'SRK'\` or \`'PR'\`. |
| \`T\` | Number | Yes | Temperature [K] (below the critical temperature). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Psat\` | Number | Saturation pressure [Pa]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`SUPERCRITICAL\` | \`T ≥ Tc\` | No saturation pressure exists above the critical temperature. |
| \`UNKNOWN_FLUID\` | \`fluid$\` not in the table | Use a supported fluid name. |`,
  },
  {
    name: `eos_volume`,
    slug: `eos_volume`,
    category: `Properties (EOS)`,
    summary: `Specific volume from a cubic equation of state (SRK or PR).`,
    related: [`eos_z`, `eos_density`, `eos_pressure`],
    examples: [`cubic-eos-properties`],
    tags: [`eos`, `cubic`, `peng-robinson`, `srk`, `specific volume`, `real gas`],
    references: [],
    guides: [],
    body: `Returns the **specific volume** \`v\` [m³/kg] of a real fluid from a cubic equation
of state (\`'SRK'\` or \`'PR'\`) at temperature \`T\` and pressure \`P\`. It is the
compressibility factor expressed as a volume — the reciprocal of
\`eos_density\`.

## Syntax

\`\`\`
v = eos_volume(fluid$, model$, T, P, phase$)
\`\`\`

## Description

Once the cubic is solved for the compressibility factor \`Z\` (see \`eos_z\`),
the specific volume follows directly from its definition.

## Mathematical Formulation

$$ v = \\frac{Z\\,R\\,T}{P} $$

where \`R\` is the specific gas constant of the fluid and \`Z\` is the EOS root for the
requested phase.

> **Method:** \`eos_z\` root → \`v = ZRT/P\`.

## Examples

### Example 1 — CO₂ specific volume

[Run: cubic-eos-properties]

**Expected (approx.):** at 6 MPa, 320 K (PR), \`v ≈ 7×10⁻³ m³/kg\` (\`Z ≈ 0.7\`),
the reciprocal of the density.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name. |
| \`model$\` | String | Yes | \`'SRK'\` or \`'PR'\`. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`phase$\` | String | Yes | Root selector: \`'vapor'\` or \`'liquid'\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`v\` | Number | Specific volume [m³/kg]. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_FLUID\` | \`fluid$\` not in the table | Use a supported fluid name. |`,
  },
  {
    name: `eos_z`,
    slug: `eos_z`,
    category: `Properties (EOS)`,
    summary: `Compressibility factor Z from a cubic equation of state (SRK or PR).`,
    related: [`eos_volume`, `eos_density`, `eos_enthalpy`, `eos_psat`],
    examples: [`cubic-eos-properties`],
    tags: [`eos`, `cubic`, `peng-robinson`, `srk`, `compressibility`, `real gas`, `z-factor`],
    references: [],
    guides: [],
    body: `Returns the **compressibility factor** \`Z = Pv/(RT)\` of a real fluid from a cubic
equation of state — Soave–Redlich–Kwong (\`'SRK'\`) or Peng–Robinson (\`'PR'\`) — at
temperature \`T\` and pressure \`P\`. \`Z\` measures the departure from ideal-gas
behavior (\`Z = 1\`) and is the root used to build all other EOS properties. A backend that needs no
real-fluid property data — only critical constants and the acentric factor.

## Syntax

\`\`\`
Z = eos_z(fluid$, model$, T, P, phase$)
\`\`\`

## Description

\`model$\` selects \`'SRK'\` or \`'PR'\`; \`phase$\` (\`'vapor'\`/\`'liquid'\`) picks which
real root of the cubic to return when the EOS is multivalued (two-phase region).

## Mathematical Formulation

The Peng–Robinson equation of state,

$$ P = \\frac{RT}{v-b} - \\frac{a\\,\\alpha(T)}{v(v+b) + b(v-b)} $$

with $a = 0.45724\\,R^2T_c^2/P_c$, $b = 0.07780\\,RT_c/P_c$, and
$\\alpha = [1 + m(1-\\sqrt{T/T_r})]^2$, $m = 0.37464 + 1.54226\\omega - 0.26992\\omega^2$,
rearranges into the cubic in \`Z\` (with $A = a\\alpha P/(RT)^2$, $B = bP/RT$):

$$ Z^3 - (1-B)Z^2 + (A - 2B - 3B^2)Z - (AB - B^2 - B^3) = 0 $$

(SRK uses $a = 0.42748\\,R^2T_c^2/P_c$, $b = 0.08664\\,RT_c/P_c$,
$m = 0.480 + 1.574\\omega - 0.176\\omega^2$ and the denominator $v(v+b)$.)

> **Method:** solve the cubic in \`Z\`; for a multivalued root, return the largest
> real root for \`'vapor'\` and the smallest for \`'liquid'\`.

## Examples

### Example 1 — CO₂ near the critical region

CO₂ at 6 MPa, 320 K with Peng–Robinson:

[Run: cubic-eos-properties]

**Expected (approx.):** \`Z ≈ 0.7\` — a strong real-gas deviation, since 320 K is
just above the CO₂ critical temperature (304 K) at near-critical pressure.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`fluid$\` | String | Yes | Fluid name (critical constants + acentric factor looked up). |
| \`model$\` | String | Yes | \`'SRK'\` or \`'PR'\`. |
| \`T\` | Number | Yes | Temperature [K]. |
| \`P\` | Number | Yes | Pressure [Pa]. |
| \`phase$\` | String | Yes | Root selector: \`'vapor'\` or \`'liquid'\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`Z\` | Number | Compressibility factor (dimensionless). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_FLUID\` | \`fluid$\` not in the critical-constant table | Use a supported fluid name. |
| \`UNKNOWN_MODEL\` | \`model$\` not \`'SRK'\`/\`'PR'\` | Pass \`'SRK'\` or \`'PR'\`. |`,
  },
  {
    name: `c_`,
    slug: `c_`,
    category: `Solid Materials`,
    summary: `Solid-material property accessor c_(Material[, T]).`,
    related: [],
    examples: [],
    tags: [`c`, `material`, `solid`, `property`],
    references: [],
    guides: [`solid-materials`],
    body: `Returns a solid-material property via \`c_(Material[, T])\` from the built-in material database.

> Real-fluid/material/symbolic operation — see the inputs below.

## Syntax

\`\`\`
c_(Material[, T])
\`\`\`

## Description

Looks up a thermophysical/mechanical property of a named solid (e.g. Aluminum, Copper, Steel). Some properties accept an optional temperature.`,
  },
  {
    name: `E_`,
    slug: `e_`,
    category: `Solid Materials`,
    summary: `Solid-material property accessor E_(Material[, T]).`,
    related: [],
    examples: [`multi-objective-beam`],
    tags: [`e`, `material`, `solid`, `property`],
    references: [],
    guides: [`solid-materials`],
    body: `Returns a solid-material property via \`E_(Material[, T])\` from the built-in material database.

> Real-fluid/material/symbolic operation — see the inputs below.

## Syntax

\`\`\`
E_(Material[, T])
\`\`\`

## Description

Looks up a thermophysical/mechanical property of a named solid (e.g. Aluminum, Copper, Steel). Some properties accept an optional temperature.`,
  },
  {
    name: `k_`,
    slug: `k_`,
    category: `Solid Materials`,
    summary: `Solid-material property accessor k_(Material[, T]).`,
    related: [],
    examples: [`material-conduction`],
    tags: [`k`, `material`, `solid`, `property`],
    references: [],
    guides: [`solid-materials`],
    body: `Returns a solid-material property via \`k_(Material[, T])\` from the built-in material database.

> Real-fluid/material/symbolic operation — see the inputs below.

## Syntax

\`\`\`
k_(Material[, T])
\`\`\`

## Description

Looks up a thermophysical/mechanical property of a named solid (e.g. Aluminum, Copper, Steel). Some properties accept an optional temperature.`,
  },
  {
    name: `nu_`,
    slug: `nu_`,
    category: `Solid Materials`,
    summary: `Solid-material property accessor nu_(Material[, T]).`,
    related: [],
    examples: [],
    tags: [`nu`, `material`, `solid`, `property`],
    references: [],
    guides: [`solid-materials`],
    body: `Returns a solid-material property via \`nu_(Material[, T])\` from the built-in material database.

> Real-fluid/material/symbolic operation — see the inputs below.

## Syntax

\`\`\`
nu_(Material[, T])
\`\`\`

## Description

Looks up a thermophysical/mechanical property of a named solid (e.g. Aluminum, Copper, Steel). Some properties accept an optional temperature.`,
  },
  {
    name: `rho_`,
    slug: `rho_`,
    category: `Solid Materials`,
    summary: `Solid-material property accessor rho_(Material[, T]).`,
    related: [],
    examples: [`multi-objective-beam`],
    tags: [`rho`, `material`, `solid`, `property`],
    references: [],
    guides: [`solid-materials`],
    body: `Returns a solid-material property via \`rho_(Material[, T])\` from the built-in material database.

> Real-fluid/material/symbolic operation — see the inputs below.

## Syntax

\`\`\`
rho_(Material[, T])
\`\`\`

## Description

Looks up a thermophysical/mechanical property of a named solid (e.g. Aluminum, Copper, Steel). Some properties accept an optional temperature.`,
  },
  {
    name: `besseli`,
    slug: `besseli`,
    category: `Special Functions`,
    summary: `Modified Bessel function of the first kind, order n — I_n(x).`,
    related: [`besselk`, `besselj`, `besseli0`, `besseli1`],
    examples: [],
    tags: [`special function`, `modified bessel`, `first kind`, `fin`, `conduction`],
    references: [`NIST Digital Library of Mathematical Functions, §10.25`],
    guides: [`special-funcs`],
    body: `Returns the **modified Bessel function of the first kind** \`I_n(x)\` of integer
order \`n\` — the finite-at-origin solution of the modified Bessel equation. It grows
exponentially and appears in fin conduction and cylindrical diffusion.

## Syntax

\`\`\`
y = besseli(n, x)
\`\`\`

## Description

Unlike \`J_n\`, \`I_n\` does not oscillate — it increases monotonically for \`x > 0\`.
For fixed orders use \`besseli0\` / \`besseli1\`.

## Mathematical Formulation

\`I_n\` solves the modified Bessel equation:

$$ x^2 y'' + x y' - (x^2 + n^2)y = 0, \\qquad I_n(x) = \\sum_{k=0}^{\\infty}\\frac{1}{k!\\,(n+k)!}\\left(\\frac{x}{2}\\right)^{2k+n} $$

with \`I_n(x) = i^{-n}J_n(ix)\`.

> **Method:** series for small \`x\`, asymptotic \`I_n(x) ~ e^x/\\sqrt{2\\pi x}\` for large \`x\`.

## Examples

\`\`\`
{ besseli(0, 0) = 1 }
y = besseli(0, 0)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Integer order. |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | I_n(x). |

## References

1. NIST *Digital Library of Mathematical Functions*, §10.25.`,
  },
  {
    name: `besseli0`,
    slug: `besseli0`,
    category: `Special Functions`,
    summary: `Modified Bessel function of the first kind, order 0 — I_0(x).`,
    related: [`besseli`, `besseli1`, `besselk0`],
    examples: [],
    tags: [`special function`, `modified bessel`, `i0`, `first kind`],
    references: [],
    guides: [],
    body: `Returns \`I_0(x)\`, the **order-0 modified Bessel function of the first kind** — the
fixed-order specialization of \`besseli\`. \`I_0(0) = 1\`; it grows like
\`e^x/√(2πx)\`.

## Syntax

\`\`\`
y = besseli0(x)
\`\`\`

## Mathematical Formulation

$$ I_0(x) = \\sum_{k=0}^{\\infty}\\frac{1}{(k!)^2}\\left(\\frac{x}{2}\\right)^{2k} $$

## Examples

\`\`\`
{ besseli0(0) = 1 }
y = besseli0(0)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | I_0(x). |`,
  },
  {
    name: `besseli1`,
    slug: `besseli1`,
    category: `Special Functions`,
    summary: `Modified Bessel function of the first kind, order 1 — I_1(x).`,
    related: [`besseli`, `besseli0`, `besselk1`],
    examples: [],
    tags: [`special function`, `modified bessel`, `i1`, `first kind`],
    references: [],
    guides: [],
    body: `Returns \`I_1(x)\`, the **order-1 modified Bessel function of the first kind** — the
fixed-order specialization of \`besseli\`. \`I_1(0) = 0\`, with
\`I_0'(x) = I_1(x)\`.

## Syntax

\`\`\`
y = besseli1(x)
\`\`\`

## Mathematical Formulation

$$ I_1(x) = \\sum_{k=0}^{\\infty}\\frac{1}{k!\\,(k+1)!}\\left(\\frac{x}{2}\\right)^{2k+1}, \\qquad I_0'(x) = I_1(x) $$

## Examples

\`\`\`
{ besseli1(0) = 0 }
y = besseli1(0)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | I_1(x). |`,
  },
  {
    name: `besselj`,
    slug: `besselj`,
    category: `Special Functions`,
    summary: `Bessel function of the first kind, order n — J_n(x).`,
    related: [`bessely`, `besseli`, `besselk`, `besselj0`, `besselj1`],
    examples: [],
    tags: [`special function`, `bessel`, `first kind`, `cylinder`, `wave`],
    references: [`NIST Digital Library of Mathematical Functions, §10.2`],
    guides: [`special-funcs`],
    body: `Returns the **Bessel function of the first kind** \`J_n(x)\` of integer order \`n\`. It
is the finite-at-origin solution of Bessel's equation — the radial mode shape in
cylindrical wave, vibration, and diffusion problems.

## Syntax

\`\`\`
y = besselj(n, x)
\`\`\`

## Description

\`J_n\` oscillates with a slowly decaying amplitude. For the common fixed orders use
\`besselj0\` / \`besselj1\`.

## Mathematical Formulation

\`J_n(x)\` solves Bessel's equation and has the series

$$ x^2 y'' + x y' + (x^2 - n^2)y = 0, \\qquad J_n(x) = \\sum_{k=0}^{\\infty} \\frac{(-1)^k}{k!\\,(n+k)!}\\left(\\frac{x}{2}\\right)^{2k+n} $$

with the recurrence \`J_{n-1}(x) + J_{n+1}(x) = (2n/x)J_n(x)\`.

> **Method:** series for small \`x\`, asymptotic/recurrence for large \`x\`.

## Examples

\`\`\`
{ besselj(0, 0) = 1 }
y = besselj(0, 0)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Integer order. |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | J_n(x). |

## References

1. NIST *Digital Library of Mathematical Functions*, §10.2.`,
  },
  {
    name: `besselj0`,
    slug: `besselj0`,
    category: `Special Functions`,
    summary: `Bessel function of the first kind, order 0 — J_0(x).`,
    related: [`besselj`, `besselj1`, `bessely0`],
    examples: [],
    tags: [`special function`, `bessel`, `j0`, `first kind`],
    references: [],
    guides: [],
    body: `Returns \`J_0(x)\`, the **order-0 Bessel function of the first kind** — the
fixed-order specialization of \`besselj\`. \`J_0(0) = 1\`, then it
oscillates with decaying amplitude.

## Syntax

\`\`\`
y = besselj0(x)
\`\`\`

## Mathematical Formulation

$$ J_0(x) = \\sum_{k=0}^{\\infty}\\frac{(-1)^k}{(k!)^2}\\left(\\frac{x}{2}\\right)^{2k} $$

## Examples

\`\`\`
{ besselj0(0) = 1 }
y = besselj0(0)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | J_0(x). |`,
  },
  {
    name: `besselj1`,
    slug: `besselj1`,
    category: `Special Functions`,
    summary: `Bessel function of the first kind, order 1 — J_1(x).`,
    related: [`besselj`, `besselj0`, `bessely1`],
    examples: [],
    tags: [`special function`, `bessel`, `j1`, `first kind`],
    references: [],
    guides: [],
    body: `Returns \`J_1(x)\`, the **order-1 Bessel function of the first kind** — the
fixed-order specialization of \`besselj\`. \`J_1(0) = 0\`; it is the
derivative companion \`J_0'(x) = −J_1(x)\`.

## Syntax

\`\`\`
y = besselj1(x)
\`\`\`

## Mathematical Formulation

$$ J_1(x) = \\sum_{k=0}^{\\infty}\\frac{(-1)^k}{k!\\,(k+1)!}\\left(\\frac{x}{2}\\right)^{2k+1}, \\qquad J_0'(x) = -J_1(x) $$

## Examples

\`\`\`
{ besselj1(0) = 0 }
y = besselj1(0)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | J_1(x). |`,
  },
  {
    name: `besselk`,
    slug: `besselk`,
    category: `Special Functions`,
    summary: `Modified Bessel function of the second kind, order n — K_n(x).`,
    related: [`besseli`, `bessely`, `besselk0`, `besselk1`],
    examples: [],
    tags: [`special function`, `modified bessel`, `second kind`, `macdonald`, `decay`],
    references: [`NIST Digital Library of Mathematical Functions, §10.25`],
    guides: [`special-funcs`],
    body: `Returns the **modified Bessel function of the second kind** \`K_n(x)\` (Macdonald
function) of integer order \`n\` — the exponentially decaying solution of the modified
Bessel equation, used for outward (decaying) cylindrical fields.

## Syntax

\`\`\`
y = besselk(n, x)
\`\`\`

## Description

\`K_n(x) → ∞\` as \`x → 0⁺\` and decays like \`e^{−x}\` for large \`x\`. The natural
partner to \`besseli\`. For fixed orders use \`besselk0\` /
\`besselk1\`.

## Mathematical Formulation

\`K_n\` is the decaying solution of the modified Bessel equation:

$$ x^2 y'' + x y' - (x^2 + n^2)y = 0, \\qquad K_n(x) = \\frac{\\pi}{2}\\frac{I_{-n}(x) - I_n(x)}{\\sin(n\\pi)} $$

(a limit for integer \`n\`), with asymptotic \`K_n(x) ~ \\sqrt{\\pi/2x}\\,e^{-x}\`.

> **Method:** standard library evaluation via the \`I\`/\`K\` relations.

## Examples

\`\`\`
{ K_0(x) -> inf as x -> 0; decays for large x }
y = besselk(0, 1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Integer order. |
| \`x\` | Number | Yes | Argument (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | K_n(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | \`K_n\` is singular at and below 0; use a positive argument. |

## References

1. NIST *Digital Library of Mathematical Functions*, §10.25.`,
  },
  {
    name: `besselk0`,
    slug: `besselk0`,
    category: `Special Functions`,
    summary: `Modified Bessel function of the second kind, order 0 — K_0(x).`,
    related: [`besselk`, `besselk1`, `besseli0`],
    examples: [],
    tags: [`special function`, `modified bessel`, `k0`, `second kind`, `macdonald`],
    references: [],
    guides: [],
    body: `Returns \`K_0(x)\`, the **order-0 modified Bessel function of the second kind**
(Macdonald) — the fixed-order specialization of \`besselk\`. Singular as
\`x → 0⁺\`, decaying like \`e^{−x}\`.

## Syntax

\`\`\`
y = besselk0(x)
\`\`\`

## Mathematical Formulation

\`K_0\` is the decaying order-0 solution of the modified Bessel equation, with
\`K_0(x) ~ √(π/2x)·e^{−x}\` for large \`x\`.

## Examples

\`\`\`
{ decays for large x }
y = besselk0(1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | K_0(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | Singular at and below 0; use a positive argument. |`,
  },
  {
    name: `besselk1`,
    slug: `besselk1`,
    category: `Special Functions`,
    summary: `Modified Bessel function of the second kind, order 1 — K_1(x).`,
    related: [`besselk`, `besselk0`, `besseli1`],
    examples: [],
    tags: [`special function`, `modified bessel`, `k1`, `second kind`, `macdonald`],
    references: [],
    guides: [],
    body: `Returns \`K_1(x)\`, the **order-1 modified Bessel function of the second kind**
(Macdonald) — the fixed-order specialization of \`besselk\`. Singular as
\`x → 0⁺\`, decaying like \`e^{−x}\`, with \`K_0'(x) = −K_1(x)\`.

## Syntax

\`\`\`
y = besselk1(x)
\`\`\`

## Mathematical Formulation

\`K_1\` is the decaying order-1 solution of the modified Bessel equation, with
\`K_0'(x) = −K_1(x)\` and \`K_1(x) ~ √(π/2x)·e^{−x}\` for large \`x\`.

## Examples

\`\`\`
{ decays for large x }
y = besselk1(1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | K_1(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | Singular at and below 0; use a positive argument. |`,
  },
  {
    name: `bessely`,
    slug: `bessely`,
    category: `Special Functions`,
    summary: `Bessel function of the second kind, order n — Y_n(x).`,
    related: [`besselj`, `besseli`, `besselk`, `bessely0`, `bessely1`],
    examples: [],
    tags: [`special function`, `bessel`, `second kind`, `neumann`, `cylinder`],
    references: [`NIST Digital Library of Mathematical Functions, §10.2`],
    guides: [`special-funcs`],
    body: `Returns the **Bessel function of the second kind** \`Y_n(x)\` (Neumann function) of
integer order \`n\` — the second, singular-at-origin solution of Bessel's equation,
needed for problems with a hollow (non-zero inner radius) domain.

## Syntax

\`\`\`
y = bessely(n, x)
\`\`\`

## Description

\`Y_n(x) → −∞\` as \`x → 0⁺\`, so it appears only where the origin is excluded. For
fixed orders use \`bessely0\` / \`bessely1\`.

## Mathematical Formulation

\`Y_n\` is the second independent solution of Bessel's equation:

$$ x^2 y'' + x y' + (x^2 - n^2)y = 0, \\qquad Y_n(x) = \\frac{J_n(x)\\cos(n\\pi) - J_{-n}(x)}{\\sin(n\\pi)} $$

(taken as a limit for integer \`n\`), with the same recurrence as \`J_n\`.

> **Method:** standard library evaluation via the \`J\`/\`Y\` relations and recurrence.

## Examples

\`\`\`
{ Y_0(x) -> -inf as x -> 0; finite for x > 0 }
y = bessely(0, 1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Integer order. |
| \`x\` | Number | Yes | Argument (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Y_n(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | \`Y_n\` is singular at and below 0; use a positive argument. |

## References

1. NIST *Digital Library of Mathematical Functions*, §10.2.`,
  },
  {
    name: `bessely0`,
    slug: `bessely0`,
    category: `Special Functions`,
    summary: `Bessel function of the second kind, order 0 — Y_0(x).`,
    related: [`bessely`, `bessely1`, `besselj0`],
    examples: [],
    tags: [`special function`, `bessel`, `y0`, `second kind`, `neumann`],
    references: [],
    guides: [],
    body: `Returns \`Y_0(x)\`, the **order-0 Bessel function of the second kind** (Neumann) — the
fixed-order specialization of \`bessely\`. \`Y_0(x) → −∞\` as \`x → 0⁺\`.

## Syntax

\`\`\`
y = bessely0(x)
\`\`\`

## Mathematical Formulation

$$ Y_0(x) = \\frac{2}{\\pi}\\left[\\ln\\!\\frac{x}{2} + \\gamma\\right]J_0(x) + \\dots $$

the second independent order-0 solution of Bessel's equation.

## Examples

\`\`\`
{ finite for x > 0 }
y = bessely0(1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Y_0(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | Singular at and below 0; use a positive argument. |`,
  },
  {
    name: `bessely1`,
    slug: `bessely1`,
    category: `Special Functions`,
    summary: `Bessel function of the second kind, order 1 — Y_1(x).`,
    related: [`bessely`, `bessely0`, `besselj1`],
    examples: [],
    tags: [`special function`, `bessel`, `y1`, `second kind`, `neumann`],
    references: [],
    guides: [],
    body: `Returns \`Y_1(x)\`, the **order-1 Bessel function of the second kind** (Neumann) — the
fixed-order specialization of \`bessely\`. Singular as \`x → 0⁺\`.

## Syntax

\`\`\`
y = bessely1(x)
\`\`\`

## Mathematical Formulation

\`Y_1\` is the second independent order-1 solution of Bessel's equation,
with \`Y_0'(x) = −Y_1(x)\`.

## Examples

\`\`\`
{ finite for x > 0 }
y = bessely1(1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument (> 0). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Y_1(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | Singular at and below 0; use a positive argument. |`,
  },
  {
    name: `beta`,
    slug: `beta`,
    category: `Special Functions`,
    summary: `Beta function B(a, b) = Γ(a)Γ(b)/Γ(a+b).`,
    related: [`gamma`, `loggamma`],
    examples: [],
    tags: [`special function`, `beta`, `gamma`, `integral`],
    references: [`NIST Digital Library of Mathematical Functions, §5.12`],
    guides: [`special-funcs`],
    body: `Returns the **Beta function** \`B(a, b)\` — a normalizing constant built from
\`gamma\` functions, central to the Beta distribution and to many definite
integrals.

## Syntax

\`\`\`
y = beta(a, b)
\`\`\`

## Description

Symmetric in its arguments (\`B(a, b) = B(b, a)\`); defined for positive \`a\`, \`b\`.

## Mathematical Formulation

$$ B(a, b) = \\int_0^1 t^{a-1}(1-t)^{b-1}\\,dt = \\frac{\\Gamma(a)\\,\\Gamma(b)}{\\Gamma(a+b)} $$

> **Method:** evaluated via \`exp(loggamma(a) + loggamma(b) − loggamma(a+b))\` for
> numerical safety.

## Examples

\`\`\`
{ B(2,3) = 1!*2!/4! = 1/12 ~ 0.0833 }
y = beta(2, 3)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First positive parameter. |
| \`b\` | Number | Yes | Second positive parameter. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | B(a, b). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`a ≤ 0\` or \`b ≤ 0\` | Use positive parameters. |

## References

1. NIST *Digital Library of Mathematical Functions*, §5.12.`,
  },
  {
    name: `chebyshevt`,
    slug: `chebyshevt`,
    category: `Special Functions`,
    summary: `Chebyshev polynomial of the first kind, T_n(x).`,
    related: [`chebyshevu`, `legendrep`],
    examples: [],
    tags: [`special function`, `chebyshev`, `orthogonal polynomial`, `approximation`],
    references: [`NIST Digital Library of Mathematical Functions, §18.3`],
    guides: [],
    body: `Returns the **Chebyshev polynomial of the first kind** \`T_n(x)\` of degree \`n\` — the
minimax-optimal polynomials on \`[−1, 1]\`, central to function approximation and
spectral methods.

## Syntax

\`\`\`
y = chebyshevt(n, x)
\`\`\`

## Description

On \`[−1, 1]\`, \`T_n(x) = cos(n·arccos x)\`, so \`|T_n| ≤ 1\`. \`T_0 = 1\`, \`T_1 = x\`.

## Mathematical Formulation

$$ T_n(\\cos\\theta) = \\cos(n\\theta), \\qquad T_{n+1}(x) = 2x\\,T_n(x) - T_{n-1}(x) $$

> **Method:** three-term recurrence from \`T_0 = 1\`, \`T_1 = x\`.

## Examples

\`\`\`
{ T_2(x) = 2x^2 - 1; chebyshevt(2, 1) = 1 }
y = chebyshevt(2, 1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Polynomial degree (≥ 0). |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | T_n(x). |

## References

1. NIST *Digital Library of Mathematical Functions*, §18.3.`,
  },
  {
    name: `chebyshevu`,
    slug: `chebyshevu`,
    category: `Special Functions`,
    summary: `Chebyshev polynomial of the second kind, U_n(x).`,
    related: [`chebyshevt`, `legendrep`],
    examples: [],
    tags: [`special function`, `chebyshev`, `orthogonal polynomial`, `second kind`],
    references: [`NIST Digital Library of Mathematical Functions, §18.3`],
    guides: [],
    body: `Returns the **Chebyshev polynomial of the second kind** \`U_n(x)\` of degree \`n\` —
orthogonal on \`[−1, 1]\` with weight \`√(1 − x²)\`, and the derivative partner of
\`chebyshevt\`.

## Syntax

\`\`\`
y = chebyshevu(n, x)
\`\`\`

## Description

On \`[−1, 1]\`, \`U_n(cos θ) = sin((n+1)θ)/sin θ\`. \`U_0 = 1\`, \`U_1 = 2x\`, with
\`T_n'(x) = n·U_{n−1}(x)\`.

## Mathematical Formulation

$$ U_n(\\cos\\theta) = \\frac{\\sin((n+1)\\theta)}{\\sin\\theta}, \\qquad U_{n+1}(x) = 2x\\,U_n(x) - U_{n-1}(x) $$

> **Method:** three-term recurrence from \`U_0 = 1\`, \`U_1 = 2x\`.

## Examples

\`\`\`
{ U_1(x) = 2x; chebyshevu(1, 1) = 2 }
y = chebyshevu(1, 1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Polynomial degree (≥ 0). |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | U_n(x). |

## References

1. NIST *Digital Library of Mathematical Functions*, §18.3.`,
  },
  {
    name: `digamma`,
    slug: `digamma`,
    category: `Special Functions`,
    summary: `Digamma function ψ(x) = d/dx ln Γ(x).`,
    related: [`gamma`, `loggamma`, `beta`],
    examples: [],
    tags: [`special function`, `digamma`, `psi`, `gamma derivative`],
    references: [`NIST Digital Library of Mathematical Functions, §5.15`],
    guides: [`special-funcs`],
    body: `Returns the **digamma function** \`ψ(x)\` — the logarithmic derivative of the
\`gamma\` function. It arises in series summation, maximum-likelihood
estimation, and the derivatives of many special functions.

## Syntax

\`\`\`
y = digamma(x)
\`\`\`

## Description

Defined for all real \`x\` except the non-positive integers (poles). Satisfies a
recurrence that mirrors the Gamma recurrence.

## Mathematical Formulation

$$ \\psi(x) = \\frac{d}{dx}\\ln\\Gamma(x) = \\frac{\\Gamma'(x)}{\\Gamma(x)} $$

with the recurrence and the harmonic-number link

$$ \\psi(x+1) = \\psi(x) + \\frac{1}{x}, \\qquad \\psi(n) = -\\gamma + \\sum_{k=1}^{n-1}\\frac{1}{k} $$

where \`γ\` is the Euler–Mascheroni constant.

> **Method:** recurrence to shift the argument upward, then an asymptotic series.

## Examples

\`\`\`
{ psi(1) = -gamma (Euler-Mascheroni) ~ -0.5772 }
y = digamma(1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument (not a non-positive integer). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | ψ(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`POLE\` | \`x\` is 0 or a negative integer | ψ has poles there; use a non-integer argument. |

## References

1. NIST *Digital Library of Mathematical Functions*, §5.15.`,
  },
  {
    name: `erf`,
    slug: `erf`,
    category: `Special Functions`,
    summary: `Error function erf(x).`,
    related: [`erfc`, `erfinv`, `normalcdf`],
    examples: [],
    tags: [`special function`, `error function`, `erf`, `gaussian`, `probability`],
    references: [`NIST Digital Library of Mathematical Functions, §7.2`],
    guides: [`special-funcs`],
    body: `Returns the **error function** \`erf(x)\` — the scaled integral of the Gaussian. It
underlies the normal distribution, diffusion, and transient-conduction solutions.

## Syntax

\`\`\`
y = erf(x)
\`\`\`

## Description

An odd function (\`erf(−x) = −erf(x)\`) ranging from −1 to 1, with \`erf(0) = 0\` and
\`erf(∞) = 1\`.

## Mathematical Formulation

$$ \\operatorname{erf}(x) = \\frac{2}{\\sqrt{\\pi}}\\int_0^x e^{-t^2}\\,dt $$

related to the normal CDF by \`Φ(x) = ½[1 + erf(x/√2)]\`.

> **Method:** rational/continued-fraction approximation to machine precision.

## Examples

\`\`\`
{ erf(1) ~ 0.8427 }
y = erf(1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Real argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | erf(x) ∈ (−1, 1). |

## References

1. NIST *Digital Library of Mathematical Functions*, §7.2.`,
  },
  {
    name: `erfc`,
    slug: `erfc`,
    category: `Special Functions`,
    summary: `Complementary error function erfc(x) = 1 − erf(x).`,
    related: [`erf`, `erfinv`],
    examples: [],
    tags: [`special function`, `complementary error function`, `erfc`, `gaussian`, `tail`],
    references: [`NIST Digital Library of Mathematical Functions, §7.2`],
    guides: [`special-funcs`],
    body: `Returns the **complementary error function** \`erfc(x) = 1 − erf(x)\`. It is the
Gaussian tail probability and is computed directly (not as \`1 − erf\`) to preserve
precision for large \`x\`.

## Syntax

\`\`\`
y = erfc(x)
\`\`\`

## Description

Ranges from 2 (at \`−∞\`) to 0 (at \`+∞\`), with \`erfc(0) = 1\`. For large positive \`x\`
it is exponentially small, so the dedicated routine avoids catastrophic
cancellation.

## Mathematical Formulation

$$ \\operatorname{erfc}(x) = 1 - \\operatorname{erf}(x) = \\frac{2}{\\sqrt{\\pi}}\\int_x^\\infty e^{-t^2}\\,dt $$

> **Method:** direct rational/continued-fraction approximation of the tail.

## Examples

\`\`\`
{ erfc(1) ~ 0.1573 }
y = erfc(1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Real argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | erfc(x) ∈ (0, 2). |

## References

1. NIST *Digital Library of Mathematical Functions*, §7.2.`,
  },
  {
    name: `erfinv`,
    slug: `erfinv`,
    category: `Special Functions`,
    summary: `Inverse error function erf⁻¹(x).`,
    related: [`erf`, `erfc`, `normalinvcdf`],
    examples: [],
    tags: [`special function`, `inverse error function`, `erfinv`, `quantile`, `gaussian`],
    references: [`NIST Digital Library of Mathematical Functions, §7.17`],
    guides: [`special-funcs`],
    body: `Returns the **inverse error function** \`erf⁻¹(x)\` — the value \`w\` such that
\`erf(w) = x\`. It maps a probability-like value back to a Gaussian deviate and
underlies normal-quantile (inverse-CDF) calculations.

## Syntax

\`\`\`
w = erfinv(x)
\`\`\`

## Description

Defined on \`−1 < x < 1\`; it diverges as \`x → ±1\`. An odd function.

## Mathematical Formulation

$$ w = \\operatorname{erf}^{-1}(x) \\quad\\Longleftrightarrow\\quad \\operatorname{erf}(w) = x, \\qquad -1 < x < 1 $$

linked to the normal quantile by \`Φ⁻¹(p) = √2·erfinv(2p − 1)\`.

> **Method:** rational approximation refined by Newton iteration on \`erf\`.

## Examples

\`\`\`
{ erfinv(0.8427) ~ 1.0 }
w = erfinv(0.8427)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Value in (−1, 1). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`w\` | Number | erf⁻¹(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`|x| ≥ 1\` | The argument must lie strictly in (−1, 1). |

## References

1. NIST *Digital Library of Mathematical Functions*, §7.17.`,
  },
  {
    name: `gamma`,
    slug: `gamma`,
    category: `Special Functions`,
    summary: `Gamma function Γ(x), the continuous extension of the factorial.`,
    related: [`loggamma`, `digamma`, `beta`, `factorial`],
    examples: [],
    tags: [`special function`, `gamma`, `factorial`, `euler`],
    references: [`NIST Digital Library of Mathematical Functions, §5.2 (dlmf.nist.gov)`],
    guides: [`special-funcs`],
    body: `Returns the **Gamma function** \`Γ(x)\` — the continuous extension of the factorial,
with \`Γ(n+1) = n!\` for non-negative integers. It appears throughout probability,
combinatorics, and special-function identities.

## Syntax

\`\`\`
y = gamma(x)
\`\`\`

## Description

Defined for all real \`x\` except the non-positive integers (where it has poles).
For large arguments use \`loggamma\` to avoid overflow.

## Mathematical Formulation

$$ \\Gamma(x) = \\int_0^\\infty t^{x-1} e^{-t}\\,dt, \\qquad x > 0 $$

with the recurrence and factorial link

$$ \\Gamma(x+1) = x\\,\\Gamma(x), \\qquad \\Gamma(n+1) = n! $$

> **Method:** Lanczos / Stirling approximation evaluated to machine precision.

## Examples

\`\`\`
{ Gamma(5) = 4! = 24 }
y = gamma(5)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Argument (not a non-positive integer). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | Γ(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`POLE\` | \`x\` is 0 or a negative integer | Γ has poles there; use a non-integer or positive argument. |
| \`OVERFLOW\` | \`x\` large | Use \`loggamma(x)\` and work in the log domain. |

## References

1. NIST *Digital Library of Mathematical Functions*, §5.2.`,
  },
  {
    name: `hermiteh`,
    slug: `hermiteh`,
    category: `Special Functions`,
    summary: `Hermite polynomial H_n(x) (physicists' convention).`,
    related: [`laguerrel`, `legendrep`, `chebyshevt`],
    examples: [],
    tags: [`special function`, `hermite`, `orthogonal polynomial`, `gaussian`],
    references: [`NIST Digital Library of Mathematical Functions, §18.3`],
    guides: [],
    body: `Returns the **Hermite polynomial** \`H_n(x)\` (physicists' convention) of degree \`n\`
— orthogonal on \`(−∞, ∞)\` with weight \`e^{−x²}\`, central to Gauss–Hermite
quadrature and the quantum harmonic oscillator.

## Syntax

\`\`\`
y = hermiteh(n, x)
\`\`\`

## Description

\`H_0 = 1\`, \`H_1 = 2x\`, with the standard three-term recurrence.

## Mathematical Formulation

$$ H_{n+1}(x) = 2x\\,H_n(x) - 2n\\,H_{n-1}(x), \\qquad H_0 = 1,\\ H_1 = 2x $$

with orthogonality $\\int_{-\\infty}^{\\infty} H_m H_n\\,e^{-x^2}\\,dx = 2^n n!\\sqrt{\\pi}\\,\\delta_{mn}$.

> **Method:** three-term recurrence from \`H_0\`, \`H_1\`.

## Examples

\`\`\`
{ H_2(x) = 4x^2 - 2; hermiteh(2, 1) = 2 }
y = hermiteh(2, 1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Polynomial degree (≥ 0). |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | H_n(x). |

## References

1. NIST *Digital Library of Mathematical Functions*, §18.3.`,
  },
  {
    name: `laguerrel`,
    slug: `laguerrel`,
    category: `Special Functions`,
    summary: `Laguerre polynomial L_n(x).`,
    related: [`hermiteh`, `legendrep`, `chebyshevt`],
    examples: [],
    tags: [`special function`, `laguerre`, `orthogonal polynomial`, `quadrature`],
    references: [`NIST Digital Library of Mathematical Functions, §18.3`],
    guides: [],
    body: `Returns the **Laguerre polynomial** \`L_n(x)\` of degree \`n\` — orthogonal on
\`[0, ∞)\` with weight \`e^{−x}\`, central to Gauss–Laguerre quadrature and the radial
hydrogen wavefunctions.

## Syntax

\`\`\`
y = laguerrel(n, x)
\`\`\`

## Description

\`L_0 = 1\`, \`L_1 = 1 − x\`, with the standard three-term recurrence.

## Mathematical Formulation

$$ (n+1)L_{n+1}(x) = (2n+1-x)L_n(x) - n\\,L_{n-1}(x), \\qquad L_0 = 1,\\ L_1 = 1 - x $$

with orthogonality $\\int_0^\\infty L_m L_n\\,e^{-x}\\,dx = \\delta_{mn}$.

> **Method:** three-term recurrence from \`L_0\`, \`L_1\`.

## Examples

\`\`\`
{ L_1(x) = 1 - x; laguerrel(1, 0) = 1 }
y = laguerrel(1, 0)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Polynomial degree (≥ 0). |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | L_n(x). |

## References

1. NIST *Digital Library of Mathematical Functions*, §18.3.`,
  },
  {
    name: `legendrep`,
    slug: `legendrep`,
    category: `Special Functions`,
    summary: `Legendre polynomial P_n(x).`,
    related: [`chebyshevt`, `hermiteh`, `laguerrel`],
    examples: [],
    tags: [`special function`, `legendre`, `orthogonal polynomial`, `quadrature`],
    references: [`NIST Digital Library of Mathematical Functions, §18.3`],
    guides: [],
    body: `Returns the **Legendre polynomial** \`P_n(x)\` of degree \`n\` — the orthogonal
polynomials on \`[−1, 1]\` with unit weight, central to Gauss–Legendre quadrature and
spherical-harmonic expansions.

## Syntax

\`\`\`
y = legendrep(n, x)
\`\`\`

## Description

\`P_0 = 1\`, \`P_1 = x\`, and higher degrees follow Bonnet's recurrence. Orthogonal on
\`[−1, 1]\`.

## Mathematical Formulation

Bonnet recurrence (DLMF §18.9):

$$ (n+1)P_{n+1}(x) = (2n+1)\\,x\\,P_n(x) - n\\,P_{n-1}(x), \\qquad P_0 = 1,\\ P_1 = x $$

with orthogonality $\\int_{-1}^{1} P_m P_n\\,dx = \\tfrac{2}{2n+1}\\delta_{mn}$.

> **Method:** three-term recurrence from \`P_0\`, \`P_1\`.

## Examples

\`\`\`
{ P_2(x) = (3x^2 - 1)/2; legendrep(2, 1) = 1 }
y = legendrep(2, 1)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`n\` | Number | Yes | Polynomial degree (≥ 0). |
| \`x\` | Number | Yes | Argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | P_n(x). |

## References

1. NIST *Digital Library of Mathematical Functions*, §18.3.`,
  },
  {
    name: `loggamma`,
    slug: `loggamma`,
    category: `Special Functions`,
    summary: `Natural logarithm of the Gamma function, ln Γ(x) (overflow-safe).`,
    related: [`gamma`, `digamma`, `beta`],
    examples: [],
    tags: [`special function`, `gamma`, `log gamma`, `overflow`],
    references: [`NIST Digital Library of Mathematical Functions, §5.2`],
    guides: [`special-funcs`],
    body: `Returns **ln Γ(x)**, the natural logarithm of the \`gamma\` function. Use it
when \`Γ(x)\` itself would overflow (large \`x\`), or in likelihoods and combinatorial
ratios where the log domain is numerically safer.

## Syntax

\`\`\`
y = loggamma(x)
\`\`\`

## Description

For \`x > 0\`, \`ln Γ(x)\` grows only logarithmically faster than linearly, so it stays
finite far past where \`Γ(x)\` overflows double precision.

## Mathematical Formulation

$$ \\ln\\Gamma(x) = \\ln\\!\\int_0^\\infty t^{x-1}e^{-t}\\,dt, \\qquad \\ln\\Gamma(x+1) = \\ln x + \\ln\\Gamma(x) $$

with the Stirling asymptotic

$$ \\ln\\Gamma(x) \\sim \\left(x-\\tfrac12\\right)\\ln x - x + \\tfrac12\\ln(2\\pi) + \\frac{1}{12x} - \\dots $$

> **Method:** Lanczos approximation of \`ln Γ\` directly (no intermediate overflow).

## Examples

\`\`\`
{ loggamma(101) = ln(100!) }
y = loggamma(101)
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Positive argument. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | ln Γ(x). |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`DOMAIN_ERROR\` | \`x ≤ 0\` | Use a positive argument. |

## References

1. NIST *Digital Library of Mathematical Functions*, §5.2.`,
  },
  {
    name: `average`,
    slug: `average`,
    category: `Stats`,
    summary: `Arithmetic mean (alias avg)`,
    related: [],
    examples: [],
    tags: [`average`, `stats`],
    references: [],
    guides: [`math-funcs`],
    body: `Arithmetic mean (alias avg)


## Syntax

\`\`\`
average(x1, x2, ...)
\`\`\`

## Description

Arithmetic mean (alias avg)

## Mathematical Formulation

$$ \\bar x = \\frac{1}{n}\\sum_{i=1}^{n} x_i $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x1\` | Number | Yes | First value. |
| \`x2\` | Number | Yes | Second value. |
| \`...\` | Number | Yes | Additional values (variadic). |`,
  },
  {
    name: `chi_square`,
    slug: `chi_square`,
    category: `Stats`,
    summary: `Chi-square CDF with df degrees of freedom`,
    related: [],
    examples: [],
    tags: [`chi`, `square`, `stats`],
    references: [],
    guides: [`special-funcs`],
    body: `Chi-square CDF with df degrees of freedom


## Syntax

\`\`\`
chi_square(x, df)
\`\`\`

## Description

Chi-square CDF with df degrees of freedom

## Mathematical Formulation

$$ F(x; k) = \\frac{\\gamma(k/2,\\ x/2)}{\\Gamma(k/2)} \\quad\\text{(chi-square CDF, } k \\text{ d.o.f.)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`df\` | Number | Yes | Degrees of freedom. |`,
  },
  {
    name: `Convolve`,
    slug: `convolve`,
    category: `Stats`,
    summary: `Linear convolution of two sequences, m + n − 1 long.`,
    related: [`XCorr`, `Filter`, `FFT`, `Smooth`],
    examples: [],
    tags: [`convolve`, `convolution`, `impulse response`, `fir`, `kernel`, `signal`],
    references: [],
    guides: [],
    body: `Applies an impulse response to a signal, or combines two kernels into one.

## Syntax

\`\`\`
CALL Convolve(a, b : c)
\`\`\`

## Description

Returns the full linear convolution of \`a\` and \`b\`, \`m + n − 1\` samples long. Convolving a signal with a system's impulse response gives that system's output; convolving two kernels gives the single kernel equivalent to applying them in sequence.

\`Convolve\` is not the same as **Filter**: it returns the *full* result, including the \`n − 1\` samples of run-out past the end of the signal, and it has no recursive (denominator) part. Use **Filter** when you want an output the same length as the input, or a filter with feedback.

Correlation is the same sum with one sequence reversed — see **XCorr**.

## Mathematical Formulation

$$ c_k = \\sum_{i} a_i\\, b_{k-i}, \\qquad k = 0,\\ \\dots,\\ m+n-2 $$

> **Method:** direct \`O(m·n)\` sum. The evaluator's equation budget rejects vectors long before an FFT-based convolution would win.

## Examples

### Example 1 — a small kernel

\`\`\`
a = [1, 2, 3]
b = [4, 5]
CALL Convolve(a, b : c)
\`\`\`

**Expected:** \`c = 4, 13, 22, 15\`.

### Example 2 — an impulse is the identity

\`\`\`
a = [1]
b = [7, -2, 0.5]
CALL Convolve(a, b : c)
\`\`\`

**Expected:** \`c = 7, -2, 0.5\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Vector | Yes | First sequence, length \`m\`, non-empty. |
| \`b\` | Vector | Yes | Second sequence, length \`n\`, non-empty. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`c\` | Vector | The convolution, \`m + n − 1\` long. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`Convolve requires the output length to be m + n - 1\` | A declared output of the wrong size. | Leave it bare and let it be sized. |
| \`Convolve requires two non-empty sequences\` | An empty input. | Both sequences need at least one element. |`,
  },
  {
    name: `Detrend`,
    slug: `detrend`,
    category: `Stats`,
    summary: `Removes a least-squares straight line (or just the mean) from a measured series.`,
    related: [`Smooth`, `Window`, `Welch`, `FFT`],
    examples: [],
    tags: [`detrend`, `trend`, `drift`, `baseline`, `signal`, `sensor`],
    references: [],
    guides: [],
    body: `Removes the slow drift from a measured series so the fluctuation you actually care about is what is left. Thermocouple drift, a settling load cell, a slowly warming ambient — all of them add a ramp that dominates a spectrum or a correlation unless it is taken out first.

## Syntax

\`\`\`
CALL Detrend(y : yd)
CALL Detrend(y, 'linear' : yd)
CALL Detrend(y, 'constant' : yd)
\`\`\`

## Description

\`'linear'\` (the default) fits a straight line by ordinary least squares over the sample index \`0, 1, …, n−1\` and subtracts it, so the result has neither a mean nor a slope. \`'constant'\` subtracts only the mean and leaves the slope intact.

The fit is over the **index**, not over a time column: samples are assumed evenly spaced. On an uneven raster, resample first.

Detrending is a linear operation, which is the property to lean on when reasoning about it: \`Detrend(a + b)\` is \`Detrend(a) + Detrend(b)\`, so adding a ramp to a signal cannot change what detrending returns for the signal itself.

## Mathematical Formulation

With $\\bar t = (n-1)/2$ and $\\bar y$ the sample mean,

$$ m = \\frac{\\sum_{j} (j - \\bar t)(y_j - \\bar y)}{\\sum_{j} (j - \\bar t)^2}, \\qquad yd_j = y_j - \\left[\\bar y + m\\,(j - \\bar t)\\right] $$

> **Method:** the closed-form ordinary-least-squares line on a known uniform design — no factorization, and no conditioning to worry about.

## Examples

### Example 1 — a drifting temperature record

\`\`\`
T = [20.1, 20.4, 20.6, 21.0, 21.2, 21.5, 21.7, 22.0]
CALL Detrend(T : ripple)
peaks = peakcount(0, 0, ripple)
\`\`\`

The ~0.27 K per sample warming trend is removed; what is left is the measurement ripple around it.

### Example 2 — re-centring without flattening

\`\`\`
T = [20.1, 20.4, 20.6, 21.0]
CALL Detrend(T, 'constant' : centred)
\`\`\`

**Expected:** the mean of \`centred\` is 0 and the sample-to-sample slope is unchanged.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`y\` | Vector | Yes | The measured series, at least one sample. |
| \`'mode'\` | String | No | \`'linear'\` (default) or \`'constant'\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`yd\` | Vector | The detrended series, same length as \`y\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`Detrend mode must be 'linear' or 'constant'\` | An unrecognised mode string. | Use one of the two spellings; \`'const'\` and \`'mean'\` are also accepted for the constant form. |
| \`Detrend requires the output vector to match the input length\` | A declared output of a different size. | Leave the output bare and let it be sized, or declare it \`[1:n]\`. |`,
  },
  {
    name: `FFT`,
    slug: `fft`,
    category: `Stats`,
    summary: `Discrete Fourier transform of a complex sequence, any length.`,
    related: [`IFFT`, `Convolve`, `Welch`, `Window`, `XCorr`],
    examples: [],
    tags: [`fft`, `dft`, `fourier`, `spectrum`, `frequency`, `transform`, `signal`],
    references: [],
    guides: [],
    body: `Transforms a sampled signal into its frequency content.

## Syntax

\`\`\`
CALL FFT(re, im : outRe, outIm)
\`\`\`

## Description

Takes the complex sequence carried as two equal-length real vectors and returns its discrete Fourier transform in the same form. For a real signal, pass a vector of zeros as \`im\`.

**Any length works, including primes.** Short transforms run the direct \`O(n²)\` sum; longer ones take a radix-2 Cooley–Tukey path when the length is a power of two and Bluestein's chirp-z otherwise, both \`O(n log n)\`. There is no power-of-two restriction at any size and no zero-padding happening behind your back — the transform you get is the transform of the samples you gave.

Bin \`k\` (1-based \`outRe[k+1]\`) corresponds to frequency \`k·fs/n\`. Bins above \`n/2\` are the negative frequencies, mirrored for a real input.

Two things to do first, for a record that is not exactly periodic: remove the trend (**Detrend**) and taper the ends (**Window**). Without them, a step between the last sample and the first leaks energy across every bin. For a power spectrum specifically, **Welch** does both and averages, and is usually the better tool.

## Mathematical Formulation

$$ X_k = \\sum_{j=0}^{n-1} x_j\\, e^{-2\\pi \\mathrm{i} jk/n} $$

> **Method:** direct sum up to 32 points; radix-2 Cooley–Tukey at power-of-two lengths; Bluestein's chirp-z otherwise. All trigonometry goes through \`libm\`, so native and browser runs agree bit for bit.

## Examples

### Example 1 — the transform of a unit impulse is flat

\`\`\`
re = [1, 0, 0, 0]
im = [0, 0, 0, 0]
CALL FFT(re, im : fr, fi)
\`\`\`

**Expected:** \`fr = 1, 1, 1, 1\` and \`fi = 0, 0, 0, 0\`.

### Example 2 — a real ramp

\`\`\`
re = [1, 2, 3, 4]
im = [0, 0, 0, 0]
CALL FFT(re, im : outRe, outIm)
\`\`\`

**Expected:** \`outRe[1] = 10\`, the sum of the samples — bin 0 is always the DC total.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`re\` | Vector | Yes | Real parts of the sequence. |
| \`im\` | Vector | Yes | Imaginary parts, same length as \`re\`. Pass zeros for a real signal. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`outRe\` | Vector | Real parts of the transform, same length. |
| \`outIm\` | Vector | Imaginary parts of the transform, same length. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`FFT requires all four vectors to have the same length\` | Mismatched inputs or outputs. | All four are the same \`n\`. |
| \`FFT real and imaginary parts must have equal length\` | The two input vectors differ. | Supply an imaginary vector of zeros the same length as the real one. |`,
  },
  {
    name: `Filter`,
    slug: `filter`,
    category: `Stats`,
    summary: `Causal IIR/FIR filtering by the difference equation, starting from rest.`,
    related: [`FiltFilt`, `Smooth`, `Window`, `Convolve`],
    examples: [],
    tags: [`filter`, `iir`, `fir`, `lowpass`, `difference equation`, `signal`],
    references: [],
    guides: [],
    body: `Runs a signal through a digital filter defined by its numerator and denominator coefficients — the same \`(b, a)\` pair MATLAB's \`filter\` and SciPy's \`lfilter\` take.

## Syntax

\`\`\`
CALL Filter(b, a, x : y)
\`\`\`

## Description

Implements

\`\`\`
a[1]·y[j] = b[1]·x[j] + b[2]·x[j-1] + … − a[2]·y[j-1] − a[3]·y[j-2] − …
\`\`\`

as a transposed direct-form II, so only \`max(len(a), len(b)) − 1\` state values are carried regardless of order. Pass \`a = [1]\` for a pure FIR filter.

Initial conditions are **zero**: the first samples carry the filter's start-up transient, and the output is delayed relative to the input by the filter's group delay. Both are properties of causal filtering, not defects — when neither is acceptable, use **FiltFilt**, which cancels the delay and suppresses the transient.

## Mathematical Formulation

$$ y_j = \\frac{1}{a_1}\\left( \\sum_{i=0}^{n_b-1} b_{i+1}\\,x_{j-i} \\;-\\; \\sum_{i=1}^{n_a-1} a_{i+1}\\,y_{j-i} \\right) $$

> **Method:** transposed direct-form II, coefficients normalized by \`a[1]\` once up front.

## Examples

### Example 1 — a two-tap moving average as an FIR filter

\`\`\`
x = [1, 3, 2, 6, 4, 9, 5, 12]
b = [0.5, 0.5]
a = [1]
CALL Filter(b, a, x : y)
\`\`\`

**Expected:** \`y[1] = 0.5\` (the start-up transient — there is no \`x[0]\`), then \`y[2] = 2\`, \`y[3] = 2.5\`.

### Example 2 — a one-pole low-pass

\`\`\`
x = [1, 0, 0, 0, 0, 0]
b = [1]
a = [1, -0.5]
CALL Filter(b, a, x : y)
\`\`\`

**Expected:** the impulse response \`1, 0.5, 0.25, 0.125, …\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`b\` | Vector | Yes | Numerator (feed-forward) coefficients, \`b[1]\` first. |
| \`a\` | Vector | Yes | Denominator (feedback) coefficients; \`a[1]\` must be non-zero. |
| \`x\` | Vector | Yes | The signal. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Vector | The filtered signal, same length as \`x\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`Filter denominator a[1] must be non-zero\` | A leading zero in \`a\`. | Normalize the transfer function so \`a[1] ≠ 0\`. |
| \`Filter coefficients must all be finite\` | A \`NaN\` or infinity among the coefficients. | Check the design step that produced them. |`,
  },
  {
    name: `FiltFilt`,
    slug: `filtfilt`,
    category: `Stats`,
    summary: `Zero-phase filtering — forward then backward, so the phase shifts cancel.`,
    related: [`Filter`, `Smooth`, `XCorr`, `Welch`],
    examples: [],
    tags: [`filtfilt`, `zero phase`, `forward backward`, `filter`, `signal`, `sensor`],
    references: [],
    guides: [],
    body: `Filters a recorded signal without shifting it in time. Use it whenever the timing of a feature matters — the location of a peak, the moment a threshold is crossed, the alignment of two channels.

## Syntax

\`\`\`
CALL FiltFilt(b, a, x : y)
\`\`\`

## Description

Runs **Filter** forward over the record, reverses the result, filters again, and reverses back. Each pass shifts the phase by the same amount in opposite directions, so they cancel exactly: a symmetric feature comes back symmetric, in place.

Two consequences worth stating plainly:

* **The magnitude response is squared.** A design that is −3 dB at some frequency is −6 dB there after \`FiltFilt\`. Design for half the attenuation you want.
* **It is not causal.** Every output sample depends on the whole record, so this is a post-processing tool, never something to put inside a control loop.

Before filtering, the signal is odd-extended by \`3·max(len(a), len(b))\` samples (clipped to \`n − 1\`) at both ends and the extension discarded afterwards. That is what stops the ends from ringing.

## Mathematical Formulation

With $H$ the causal filter of **Filter** and $R$ the reversal operator,

$$ y = R\\,H\\,R\\,H\\,x \\quad\\Longrightarrow\\quad Y(\\omega) = |H(\\omega)|^2 X(\\omega) $$

— real and non-negative, hence zero phase.

> **Method:** two \`lfilter\` passes over an odd-extended record. The SciPy steady-state \`lfilter_zi\` warm start is not applied; the padding carries the edge behaviour.

## Examples

### Example 1 — a constant survives untouched, ends included

\`\`\`
x = [4, 4, 4, 4, 4, 4, 4, 4]
b = [0.2, 0.2, 0.2, 0.2, 0.2]
a = [1]
CALL FiltFilt(b, a, x : y)
\`\`\`

**Expected:** every \`y[j] = 4\`. A zero-padded implementation would sag at both ends instead.

### Example 2 — a symmetric pulse stays symmetric

\`\`\`
x = [0, 0, 0, 1, 0, 0, 0]
b = [0.25, 0.5, 0.25]
a = [1]
CALL FiltFilt(b, a, x : y)
\`\`\`

**Expected:** \`y\` is symmetric about its centre — the property a single causal pass does not have.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`b\` | Vector | Yes | Numerator coefficients. |
| \`a\` | Vector | Yes | Denominator coefficients; \`a[1]\` non-zero. |
| \`x\` | Vector | Yes | The signal, at least two samples. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Vector | The zero-phase filtered signal, same length as \`x\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`FiltFilt requires at least two samples\` | A one-sample record. | There is nothing to reflect about; use **Filter** or a longer record. |`,
  },
  {
    name: `IFFT`,
    slug: `ifft`,
    category: `Stats`,
    summary: `Inverse discrete Fourier transform, including the 1/n normalization.`,
    related: [`FFT`, `Convolve`, `Welch`, `Window`],
    examples: [],
    tags: [`ifft`, `inverse fft`, `fourier`, `transform`, `signal`],
    references: [],
    guides: [],
    body: `Takes a spectrum back to the time domain.

## Syntax

\`\`\`
CALL IFFT(re, im : outRe, outIm)
\`\`\`

## Description

The inverse of **FFT**, including the \`1/n\` normalization — so \`IFFT(FFT(x))\` returns \`x\`, not \`n·x\`. Same length rules and the same fast paths: any length, primes included.

Filtering by editing a spectrum and inverting it is a legitimate use, but note that zeroing bins is a brick-wall filter and rings badly in time. **Filter** or **FiltFilt** with a designed kernel is usually what you want instead.

## Mathematical Formulation

$$ x_j = \\frac{1}{n}\\sum_{k=0}^{n-1} X_k\\, e^{+2\\pi \\mathrm{i} jk/n} $$

> **Method:** as **FFT**, with the conjugate exponent and a final division by \`n\`.

## Examples

### Example 1 — a flat spectrum inverts to an impulse

\`\`\`
re = [1, 1]
im = [0, 0]
CALL IFFT(re, im : gr, gi)
\`\`\`

**Expected:** \`gr = 1, 0\` and \`gi = 0, 0\`.

### Example 2 — round trip

\`\`\`
x  = [3, -1, 0.5, 2.25, -7]
im = [0, 0, 0, 0, 0]
CALL FFT(x, im : fr, fi)
CALL IFFT(fr, fi : br, bi)
\`\`\`

**Expected:** \`br\` reproduces \`x\` and \`bi\` is zero to rounding.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`re\` | Vector | Yes | Real parts of the spectrum. |
| \`im\` | Vector | Yes | Imaginary parts, same length. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`outRe\` | Vector | Real parts of the recovered sequence. |
| \`outIm\` | Vector | Imaginary parts of the recovered sequence. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`IFFT requires all four vectors to have the same length\` | Mismatched inputs or outputs. | All four are the same \`n\`. |`,
  },
  {
    name: `intercept`,
    slug: `intercept`,
    category: `Stats`,
    summary: `Least-squares linear-fit intercept`,
    related: [],
    examples: [],
    tags: [`intercept`, `stats`],
    references: [],
    guides: [],
    body: `Least-squares linear-fit intercept


## Syntax

\`\`\`
intercept(xvals, yvals)
\`\`\`

## Description

Least-squares linear-fit intercept

## Mathematical Formulation

$$ b = \\bar y - m\\,\\bar x $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`xvals\` | Number | Yes | Independent-variable data (vector). |
| \`yvals\` | Number | Yes | Dependent-variable data (vector). |`,
  },
  {
    name: `max`,
    slug: `max`,
    category: `Stats`,
    summary: `Largest of the arguments.`,
    related: [`min`, `average`, `percentile`],
    examples: [`hx-effectiveness-ntu`],
    tags: [`stats`, `maximum`, `comparison`, `elementary`],
    references: [],
    guides: [`math-funcs`],
    body: `Returns the **largest** of its arguments — e.g. \`C_max = max(C_h, C_c)\` in
heat-exchanger analysis.

## Syntax

\`\`\`
y = max(a, b, ...)
\`\`\`

## Description

Accepts two or more numeric arguments and returns the greatest. Units must be
compatible across the arguments.

## Mathematical Formulation

$$ y = \\max(a_1, a_2, \\dots, a_n) $$

## Examples

### Example 1 — Maximum capacity rate of a heat exchanger

[Run: hx-effectiveness-ntu]

**Expected:** \`C_max = max(C_h, C_c)\` selects the larger heat-capacity rate.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a, b, …\` | Number | Yes | Two or more values with compatible units. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | The largest argument. |`,
  },
  {
    name: `mean`,
    slug: `mean`,
    category: `Stats`,
    summary: `Mean of vector x`,
    related: [],
    examples: [],
    tags: [`mean`, `stats`],
    references: [],
    guides: [],
    body: `Mean of vector x


## Syntax

\`\`\`
mean(x)
\`\`\`

## Description

Mean of vector x

## Mathematical Formulation

$$ \\bar x = \\frac{1}{n}\\sum_{i=1}^{n} x_i $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `median`,
    slug: `median`,
    category: `Stats`,
    summary: `Median of vector x`,
    related: [],
    examples: [],
    tags: [`median`, `stats`],
    references: [],
    guides: [],
    body: `Median of vector x


## Syntax

\`\`\`
median(x)
\`\`\`

## Description

Median of vector x

## Mathematical Formulation

$$ \\text{the middle order statistic (mean of the two middle values if } n \\text{ even)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `min`,
    slug: `min`,
    category: `Stats`,
    summary: `Smallest of the arguments.`,
    related: [`max`, `average`, `percentile`],
    examples: [`hx-effectiveness-ntu`, `ev-thermal-management`],
    tags: [`stats`, `minimum`, `comparison`, `elementary`],
    references: [],
    guides: [`comp-transient`, `comp-troubleshooting`, `math-funcs`],
    body: `Returns the **smallest** of its arguments. Commonly used to pick the limiting of
two quantities — e.g. \`C_min = min(C_h, C_c)\` in heat-exchanger analysis.

## Syntax

\`\`\`
y = min(a, b, ...)
\`\`\`

## Description

Accepts two or more numeric arguments and returns the least. Units must be
compatible across the arguments.

## Mathematical Formulation

$$ y = \\min(a_1, a_2, \\dots, a_n) $$

## Examples

### Example 1 — Minimum capacity rate of a heat exchanger

[Run: hx-effectiveness-ntu]

**Expected:** \`C_min = min(C_h, C_c)\` selects the smaller heat-capacity rate.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a, b, …\` | Number | Yes | Two or more values with compatible units. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Number | The smallest argument. |`,
  },
  {
    name: `normalcdf`,
    slug: `normalcdf`,
    category: `Stats`,
    summary: `Normal cumulative distribution at x`,
    related: [],
    examples: [],
    tags: [`normalcdf`, `stats`],
    references: [],
    guides: [`special-funcs`],
    body: `Normal cumulative distribution at x


## Syntax

\`\`\`
normalcdf(x, mu, sigma)
\`\`\`

## Description

Normal cumulative distribution at x

## Mathematical Formulation

$$ \\Phi(x;\\mu,\\sigma) = \\tfrac12\\left[1 + \\operatorname{erf}\\!\\frac{x-\\mu}{\\sigma\\sqrt2}\\right] $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`mu\` | Number | Yes | Dynamic viscosity [Pa·s]. |
| \`sigma\` | Number | Yes | Surface tension [N/m]. |`,
  },
  {
    name: `normalinvcdf`,
    slug: `normalinvcdf`,
    category: `Stats`,
    summary: `Inverse normal CDF (quantile) at p`,
    related: [],
    examples: [],
    tags: [`normalinvcdf`, `stats`],
    references: [],
    guides: [],
    body: `Inverse normal CDF (quantile) at p


## Syntax

\`\`\`
normalinvcdf(p, mu, sigma)
\`\`\`

## Description

Inverse normal CDF (quantile) at p

## Mathematical Formulation

$$ x = \\Phi^{-1}(p;\\mu,\\sigma) = \\mu + \\sigma\\sqrt2\\,\\operatorname{erf}^{-1}(2p-1) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`p\` | Number | Yes | Probability (0–1) / percentile rank. |
| \`mu\` | Number | Yes | Dynamic viscosity [Pa·s]. |
| \`sigma\` | Number | Yes | Surface tension [N/m]. |`,
  },
  {
    name: `normalpdf`,
    slug: `normalpdf`,
    category: `Stats`,
    summary: `Normal probability density at x`,
    related: [],
    examples: [],
    tags: [`normalpdf`, `stats`],
    references: [],
    guides: [],
    body: `Normal probability density at x


## Syntax

\`\`\`
normalpdf(x, mu, sigma)
\`\`\`

## Description

Normal probability density at x

## Mathematical Formulation

$$ \\phi(x;\\mu,\\sigma) = \\frac{1}{\\sigma\\sqrt{2\\pi}}\\,e^{-(x-\\mu)^2/(2\\sigma^2)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`mu\` | Number | Yes | Dynamic viscosity [Pa·s]. |
| \`sigma\` | Number | Yes | Surface tension [N/m]. |`,
  },
  {
    name: `peakcount`,
    slug: `peakcount`,
    category: `Stats`,
    summary: `Number of strict local maxima in a series, with height and separation thresholds.`,
    related: [`peakindex`, `Smooth`, `XCorr`, `Welch`],
    examples: [],
    tags: [`peak`, `peaks`, `local maximum`, `detection`, `count`, `signal`],
    references: [],
    guides: [],
    body: `Counts the peaks in a measured series — cycles in a pressure trace, teeth on an encoder, bursts in a vibration record.

## Syntax

\`\`\`
n = peakcount(minheight, mindistance, x1, x2, ...)
n = peakcount(minheight, mindistance, [ ... ])
\`\`\`

## Description

A peak is a sample **strictly greater than both of its neighbours**. The first and last samples are never peaks, having only one neighbour each, and a plateau of equal values is not a peak either.

The two scalars come first, matching the convention \`ci_mean_lo(0.95, […])\` set:

* \`minheight\` rejects peaks at or below it. Set it below the series minimum to disable.
* \`mindistance\` enforces a minimum index separation. \`0\` or \`1\` imposes none; anything larger keeps the **tallest** peak of each contested group and discards the rest, so a broad noisy shoulder does not out-vote the crest beside it.

Smooth first if the record is noisy: every noise excursion of two samples is a peak by this definition, and **Smooth** is usually the difference between counting cycles and counting samples.

## Mathematical Formulation

$$ P = \\{\\, j : 0 < j < n-1,\\ x_j > x_{j-1},\\ x_j > x_{j+1},\\ x_j > h \\,\\} $$

then greedily thinned by descending $x_j$ under the separation constraint.

> **Method:** one pass for the candidates, then a tallest-first sweep. Ties break to the lower index, so the answer is deterministic.

## Examples

### Example 1 — counting cycles

\`\`\`
x = [1, 3, 2, 6, 4, 9, 5, 12]
n = peakcount(0, 0, x)
\`\`\`

**Expected:** \`n = 3\` — the 3, the 6 and the 9. The final 12 is an endpoint, not a peak.

### Example 2 — ignoring small excursions

\`\`\`
x = [1, 3, 2, 6, 4, 9, 5, 12]
tall = peakcount(5, 0, x)
\`\`\`

**Expected:** \`tall = 2\` — only the 6 and the 9 clear a height floor of 5.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`minheight\` | Number | Yes | Peaks at or below this are rejected. |
| \`mindistance\` | Number | Yes | Minimum index separation; \`0\` or \`1\` for none. |
| \`x1, x2, …\` | Number | Yes | The series, as loose values or a list literal. |`,
  },
  {
    name: `peakindex`,
    slug: `peakindex`,
    category: `Stats`,
    summary: `1-based position of the k-th peak in a series, or 0 when there is none.`,
    related: [`peakcount`, `Smooth`, `XCorr`],
    examples: [],
    tags: [`peak`, `peaks`, `local maximum`, `detection`, `index`, `position`, `signal`],
    references: [],
    guides: [],
    body: `Locates a peak found by **peakcount**, so its value, its time or the interval to the next one can be read out of the series.

## Syntax

\`\`\`
p = peakindex(k, minheight, mindistance, x1, x2, ...)
p = peakindex(k, minheight, mindistance, [ ... ])
\`\`\`

## Description

Returns the 1-based position, within the series, of the \`k\`-th peak in ascending index order. Peaks are defined exactly as in **peakcount** — strictly greater than both neighbours, subject to the same \`minheight\` floor and \`mindistance\` separation — so the same two thresholds must be passed to both to get a consistent answer.

\`k\` is 1-based too: \`peakindex(1, …)\` is the first peak. When there is no \`k\`-th peak the answer is **0**, not an error and not the last peak, so a loop can walk peaks until it gets a zero.

## Examples

### Example 1 — walking the peaks of a trace

\`\`\`
x  = [1, 3, 2, 6, 4, 9, 5, 12]
n  = peakcount(0, 0, x)
p1 = peakindex(1, 0, 0, x)
p2 = peakindex(2, 0, 0, x)
p3 = peakindex(3, 0, 0, x)
p4 = peakindex(4, 0, 0, x)
\`\`\`

**Expected:** \`n = 3\`, \`p1 = 2\`, \`p2 = 4\`, \`p3 = 6\`, and \`p4 = 0\` — there is no fourth peak.

### Example 2 — the interval between the first two peaks

\`\`\`
{ x sampled at 100 Hz }
dt = (peakindex(2, 0, 3, x) - peakindex(1, 0, 3, x)) / 100
\`\`\`

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`k\` | Integer | Yes | Which peak, 1-based. Must be a positive whole number. |
| \`minheight\` | Number | Yes | Peaks at or below this are rejected. |
| \`mindistance\` | Number | Yes | Minimum index separation; \`0\` or \`1\` for none. |
| \`x1, x2, …\` | Number | Yes | The series, as loose values or a list literal. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`the ordinal must be a positive whole number\` | \`k\` was 0, negative or fractional. | Peaks are numbered from 1. |`,
  },
  {
    name: `percentile`,
    slug: `percentile`,
    category: `Stats`,
    summary: `p-th percentile, p in [0,100]`,
    related: [],
    examples: [],
    tags: [`percentile`, `stats`],
    references: [],
    guides: [],
    body: `p-th percentile, p in [0,100]


## Syntax

\`\`\`
percentile(p, x1, x2, ...)
\`\`\`

## Description

p-th percentile, p in [0,100]

## Mathematical Formulation

$$ P_p = \\text{value below which } p\\% \\text{ of the data fall (linear interpolation)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`p\` | Number | Yes | Probability (0–1) / percentile rank. |
| \`x1\` | Number | Yes | First value. |
| \`x2\` | Number | Yes | Second value. |
| \`...\` | Number | Yes | Additional values (variadic). |`,
  },
  {
    name: `probability`,
    slug: `probability`,
    category: `Stats`,
    summary: `Probability that a normal variate falls in the interval [x1, x2].`,
    related: [`normalcdf`, `normalpdf`, `chi_square`],
    examples: [],
    tags: [`probability`, `stats`, `normal`, `gaussian`, `interval`, `range`],
    references: [],
    guides: [`special-funcs`],
    body: `Returns the probability that a normally distributed variable with mean \`mu\` and
standard deviation \`sigma\` falls between \`x1\` and \`x2\`. For a one-sided
cumulative probability \`Pr(X ≤ x)\` use \`normalcdf(x, mu, sigma)\` instead.

## Syntax

\`\`\`
p = probability(x1, x2, mu, sigma)
\`\`\`

## Description

Evaluates the area of the normal density between the two bounds. Use it for
"what fraction lies within these limits" questions — tolerances, pass/fail
bands, ±kσ coverage. Pass a very large/small bound to get a one-sided tail.

## Mathematical Formulation

$$ \\Pr(x_1 \\le X \\le x_2) = \\tfrac{1}{2}\\left[\\operatorname{erf}\\!\\left(\\frac{x_2-\\mu}{\\sigma\\sqrt{2}}\\right) - \\operatorname{erf}\\!\\left(\\frac{x_1-\\mu}{\\sigma\\sqrt{2}}\\right)\\right] $$

> **Method:** direct evaluation via the error function (Apache Commons Math \`Erf.erf\`).

## Examples

### Example 1 — Coverage within ±1σ

\`\`\`
{ Fraction of a N(80, 5) population within one standard deviation }
p = probability(75, 85, 80, 5)   { 0.6827 }
\`\`\`

**Expected:** \`p ≈ 0.6827\` — the classic 68% inside ±1σ.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x1\` | Number | Yes | Lower bound of the interval. |
| \`x2\` | Number | Yes | Upper bound of the interval (\`x2 ≥ x1\`). |
| \`mu\` | Number | Yes | Mean of the normal distribution. |
| \`sigma\` | Number | Yes | Standard deviation (\`> 0\`). |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`p\` | Number | Probability mass in \`[x1, x2]\`, in \`[0, 1]\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`Probability standard deviation must be > 0\` | \`sigma ≤ 0\` | Pass a positive standard deviation. |`,
  },
  {
    name: `r2`,
    slug: `r2`,
    category: `Stats`,
    summary: `Linear-fit coefficient of determination R^2`,
    related: [],
    examples: [],
    tags: [`r2`, `stats`],
    references: [],
    guides: [],
    body: `Linear-fit coefficient of determination R^2


## Syntax

\`\`\`
r2(xvals, yvals)
\`\`\`

## Description

Linear-fit coefficient of determination R^2

## Mathematical Formulation

$$ R^2 = 1 - \\frac{\\sum (y_i - \\hat y_i)^2}{\\sum (y_i - \\bar y)^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`xvals\` | Number | Yes | Independent-variable data (vector). |
| \`yvals\` | Number | Yes | Dependent-variable data (vector). |`,
  },
  {
    name: `randg`,
    slug: `randg`,
    category: `Stats`,
    summary: `Gaussian (normal) random number`,
    related: [],
    examples: [],
    tags: [`randg`, `stats`],
    references: [],
    guides: [`special-funcs`],
    body: `Gaussian (normal) random number


## Syntax

\`\`\`
randg(mu, sigma)
\`\`\`

## Description

Gaussian (normal) random number

## Mathematical Formulation

$$ X \\sim \\mathcal{N}(\\mu, \\sigma^2) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`mu\` | Number | Yes | Dynamic viscosity [Pa·s]. |
| \`sigma\` | Number | Yes | Surface tension [N/m]. |`,
  },
  {
    name: `random`,
    slug: `random`,
    category: `Stats`,
    summary: `Uniform random number in [a, b]`,
    related: [],
    examples: [],
    tags: [`random`, `stats`],
    references: [],
    guides: [`special-funcs`],
    body: `Uniform random number in [a, b]


## Syntax

\`\`\`
random(a, b)
\`\`\`

## Description

Uniform random number in [a, b]

## Mathematical Formulation

$$ X \\sim \\mathcal{U}(a, b), \\qquad X = a + (b-a)\\,U,\\ \\ U\\in[0,1) $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Number | Yes | First operand. |
| \`b\` | Number | Yes | Second operand. |`,
  },
  {
    name: `rms`,
    slug: `rms`,
    category: `Stats`,
    summary: `Root mean square`,
    related: [],
    examples: [],
    tags: [`rms`, `stats`],
    references: [],
    guides: [],
    body: `Root mean square


## Syntax

\`\`\`
rms(x1, x2, ...)
\`\`\`

## Description

Root mean square

## Mathematical Formulation

$$ x_{\\text{rms}} = \\sqrt{\\frac{1}{n}\\sum_{i=1}^{n} x_i^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x1\` | Number | Yes | First value. |
| \`x2\` | Number | Yes | Second value. |
| \`...\` | Number | Yes | Additional values (variadic). |`,
  },
  {
    name: `slope`,
    slug: `slope`,
    category: `Stats`,
    summary: `Least-squares linear-fit slope`,
    related: [],
    examples: [],
    tags: [`slope`, `stats`],
    references: [],
    guides: [],
    body: `Least-squares linear-fit slope


## Syntax

\`\`\`
slope(xvals, yvals)
\`\`\`

## Description

Least-squares linear-fit slope

## Mathematical Formulation

$$ m = \\frac{\\sum (x_i-\\bar x)(y_i-\\bar y)}{\\sum (x_i-\\bar x)^2} \\quad\\text{(least squares)} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`xvals\` | Number | Yes | Independent-variable data (vector). |
| \`yvals\` | Number | Yes | Dependent-variable data (vector). |`,
  },
  {
    name: `Smooth`,
    slug: `smooth`,
    category: `Stats`,
    summary: `Centred moving average over an odd window, with honest ends.`,
    related: [`Detrend`, `Filter`, `FiltFilt`, `Window`],
    examples: [],
    tags: [`smooth`, `moving average`, `filter`, `noise`, `signal`, `sensor`],
    references: [],
    guides: [],
    body: `A centred moving average — the first thing to reach for when a sensor trace is noisy and you want to see the shape of it.

## Syntax

\`\`\`
CALL Smooth(x, k : y)
\`\`\`

## Description

Each output sample is the mean of the \`k\` samples centred on it. \`k\` must be odd, so "centred" means what it says, and no larger than the series.

At the ends, where the full window does not exist, the average is taken over **the samples that do** rather than over zeros. That is the difference between a constant series coming back constant and one that sags at both ends — a zero-padded moving average always does the latter, and the sag is easy to mistake for a real transient.

A moving average is a low-pass filter with a \`sinc\` response, so it is not a good choice when you care about the frequency content; use **Filter** or **FiltFilt** with a designed kernel for that. As a way to see the trend in a plot, it is exactly right.

## Mathematical Formulation

$$ y_j = \\frac{1}{|W_j|}\\sum_{i \\in W_j} x_i, \\qquad W_j = \\{\\,i : |i - j| \\le \\lfloor k/2 \\rfloor,\\; 0 \\le i < n \\,\\} $$

> **Method:** direct windowed mean; the window shrinks at the ends rather than being padded.

## Examples

### Example 1 — smoothing a noisy pressure trace

\`\`\`
p = [1, 3, 2, 6, 4, 9, 5, 12]
CALL Smooth(p, 3 : ps)
\`\`\`

**Expected:** \`ps[1] = 2\` (the mean of the two samples that exist), \`ps[3] = 11/3\`, \`ps[8] = 8.5\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Vector | Yes | The series to smooth. |
| \`k\` | Integer | Yes | Window length; odd, at least 1, at most \`n\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Vector | The smoothed series, same length as \`x\`. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`Smooth requires an odd window length\` | An even \`k\`. | Use \`k − 1\` or \`k + 1\`; an even window has no centre sample. |
| \`Smooth window is longer than the series\` | \`k > n\`. | Shorten the window, or lengthen the record. |`,
  },
  {
    name: `std`,
    slug: `std`,
    category: `Stats`,
    summary: `Standard deviation of vector x`,
    related: [],
    examples: [],
    tags: [`std`, `stats`],
    references: [],
    guides: [],
    body: `Standard deviation of vector x


## Syntax

\`\`\`
std(x)
\`\`\`

## Description

Standard deviation of vector x

## Mathematical Formulation

$$ s = \\sqrt{\\frac{1}{n-1}\\sum_{i=1}^{n}(x_i - \\bar x)^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `sum`,
    slug: `sum`,
    category: `Stats`,
    summary: `Sum of vector elements`,
    related: [],
    examples: [],
    tags: [`sum`, `stats`],
    references: [],
    guides: [`math-funcs`],
    body: `Sum of vector elements


## Syntax

\`\`\`
sum(x)
\`\`\`

## Description

Sum of vector elements

## Mathematical Formulation

$$ \\sum_{i=1}^{n} x_i $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `var`,
    slug: `var`,
    category: `Stats`,
    summary: `Variance of vector x`,
    related: [],
    examples: [],
    tags: [`var`, `stats`],
    references: [],
    guides: [],
    body: `Variance of vector x


## Syntax

\`\`\`
var(x)
\`\`\`

## Description

Variance of vector x

## Mathematical Formulation

$$ s^2 = \\frac{1}{n-1}\\sum_{i=1}^{n}(x_i - \\bar x)^2 $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |`,
  },
  {
    name: `Welch`,
    slug: `welch`,
    category: `Stats`,
    summary: `Averaged-periodogram power spectral density, one-sided and in x²/Hz.`,
    related: [`FFT`, `Window`, `Detrend`, `XCorr`],
    examples: [],
    tags: [`welch`, `psd`, `spectrum`, `power spectral density`, `periodogram`, `vibration`, `noise`],
    references: [],
    guides: [],
    body: `Estimates where a signal's power sits in frequency. A single \`FFT\` of a noisy record gives a spectrum whose variance does not fall as the record gets longer; Welch's method trades resolution for that variance by splitting the record into overlapping segments and averaging their spectra.

## Syntax

\`\`\`
CALL Welch(x, fs, nperseg : f, pxx)
\`\`\`

## Description

Splits \`x\` into 50 %-overlapped segments of \`nperseg\` samples, removes each segment's mean, tapers it with a **periodic** Hann window, and averages the squared spectra. A trailing partial segment is dropped rather than zero-padded — padding would quietly bias the average toward the record's tail.

Both outputs are \`nperseg/2 + 1\` long: \`f[k] = (k−1)·fs/nperseg\` in Hz, and \`pxx\` is a one-sided **power spectral density** in the signal's units squared per hertz. "One-sided" means the negative frequencies have been folded onto their positive twins, so every bin except DC and Nyquist is doubled. The consequence to lean on when checking a result: \`Σ pxx·Δf\` recovers the signal's mean square.

\`nperseg\` sets the trade: a larger segment resolves closely-spaced tones, a smaller one averages more segments and gives a smoother estimate.

## Mathematical Formulation

For segment $i$ with window $w$ of length $L$,

$$ P^{(i)}_k = \\frac{2}{f_s \\sum_j w_j^2}\\left| \\sum_{j=0}^{L-1} (x^{(i)}_j - \\bar x^{(i)})\\, w_j\\, e^{-2\\pi \\mathrm{i} jk/L} \\right|^2 $$

(the factor 2 omitted at $k = 0$ and, for even $L$, at $k = L/2$), and $P_k$ is the mean over segments.

> **Method:** mixed-radix / Bluestein FFT per segment — see **FFT** — with a periodic Hann taper and constant detrending.

## Examples

### Example 1 — locating a tone and checking its power

\`\`\`
{ 4096 samples of a 1 Vrms 50 Hz tone at fs = 1 kHz in x[1..4096] }
CALL Welch(x, 1000, 256 : f, pxx)
\`\`\`

**Expected:** the largest \`pxx\` sits in the bin nearest 50 Hz, and \`sum(pxx)·(f[2] − f[1])\` is 1 — the mean square of a 1 Vrms tone.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Vector | Yes | The uniformly sampled signal. |
| \`fs\` | Number | Yes | Sample rate in Hz; may be any expression. Must be positive. |
| \`nperseg\` | Integer | Yes | Segment length, at least 2 and at most the record length. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`f\` | Vector | Bin centre frequencies in Hz, \`nperseg/2 + 1\` long. |
| \`pxx\` | Vector | One-sided power spectral density in \`x²/Hz\`, same length. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`Welch segment length exceeds the series length\` | \`nperseg > n\`. | Shorten the segment or lengthen the record. |
| \`Welch requires a positive, finite sample rate\` | \`fs ≤ 0\` or non-finite. | Supply the real acquisition rate. |
| \`Welch requires both output vectors to be nperseg/2 + 1 long\` | Declared outputs of another size. | Leave them bare and let them be sized. |

> **Uniform sampling is assumed.** Nothing here inspects a time column, and an uneven raster produces a spectrum that looks fine and means nothing. Resample first.`,
  },
  {
    name: `Window`,
    slug: `window`,
    category: `Stats`,
    summary: `Multiplies a series by a symmetric taper (Hann, Hamming, Blackman, Bartlett or rectangular).`,
    related: [`FFT`, `Welch`, `Detrend`, `Smooth`],
    examples: [],
    tags: [`window`, `taper`, `hann`, `hamming`, `blackman`, `bartlett`, `leakage`, `spectrum`],
    references: [],
    guides: [],
    body: `Tapers a finite record to zero at both ends before a transform, so the discontinuity between the last sample and the first does not smear energy across the whole spectrum.

## Syntax

\`\`\`
CALL Window(x, 'hann' : y)
\`\`\`

## Description

An \`FFT\` treats its input as one period of a periodic signal. A record that does not happen to contain a whole number of cycles therefore has a step in it, and that step leaks energy into every bin — enough to bury a small tone sitting next to a large one. Multiplying by a taper that reaches zero at both ends removes the step.

The windows here are **symmetric** (denominator \`n − 1\`), which is the right taper for a single finite record and matches MATLAB's \`hann(n)\` and SciPy's \`get_window(..., fftbins=False)\`. **Welch** uses the *periodic* form internally, because there the segments tile a longer record; the two differ by one sample and the difference is not cosmetic.

Every window trades resolution for leakage: Hann is the usual default, Blackman suppresses distant leakage further at the cost of a wider main lobe, Hamming sits between them, Bartlett is the simple triangle, and \`'rect'\` is the identity — the taper you already have when you do nothing.

## Mathematical Formulation

With $t = j/(n-1)$,

$$ w^{\\text{hann}}_j = 0.5 - 0.5\\cos 2\\pi t, \\quad w^{\\text{hamming}}_j = 0.54 - 0.46\\cos 2\\pi t $$
$$ w^{\\text{blackman}}_j = 0.42 - 0.5\\cos 2\\pi t + 0.08\\cos 4\\pi t, \\quad w^{\\text{bartlett}}_j = 1 - |2t - 1| $$

and $y_j = w_j x_j$.

> **Method:** the closed-form weights, evaluated through \`libm\` so native and browser runs agree bit for bit.

## Examples

### Example 1 — tapering before a transform

\`\`\`
x  = [1, 2, 3, 4, 5, 6, 7, 8]
im = [0, 0, 0, 0, 0, 0, 0, 0]
CALL Window(x, 'hann' : xw)
CALL FFT(xw, im : re, imf)
\`\`\`

**Expected:** \`xw[1] = 0\` and \`xw[8] = 0\` — a symmetric Hann closes on zero at both ends.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Vector | Yes | The series to taper. |
| \`'kind'\` | String | Yes | \`'rect'\`, \`'hann'\`, \`'hamming'\`, \`'blackman'\` or \`'bartlett'\`. \`'hanning'\`, \`'boxcar'\`, \`'rectangular'\`, \`'none'\` and \`'triangular'\` are accepted spellings. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`y\` | Vector | \`x\` multiplied by the window, same length. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`Unknown window\` | A name outside the list. | Use one of the five; the name is matched case-insensitively. |
| \`Window option 2 must be a quoted name\` | The kind was passed unquoted. | Write \`'hann'\`, with quotes. |`,
  },
  {
    name: `XCorr`,
    slug: `xcorr`,
    category: `Stats`,
    summary: `Full linear cross-correlation of two series, zero lag at the centre.`,
    related: [`Convolve`, `FiltFilt`, `Welch`, `Smooth`],
    examples: [],
    tags: [`xcorr`, `cross-correlation`, `autocorrelation`, `lag`, `delay`, `alignment`, `signal`],
    references: [],
    guides: [],
    body: `Finds how far one signal leads or lags another — the transport delay between two thermocouples, the phase between a drive and a response, the period hidden in a noisy trace.

## Syntax

\`\`\`
CALL XCorr(a, b : c)
\`\`\`

## Description

Returns the full linear cross-correlation, \`m + n − 1\` samples long. Output element \`c[i]\` carries lag \`i − n\` in 1-based terms: the **centre** element \`c[n]\` is the zero-lag correlation, a peak to the right of centre means \`a\` leads \`b\`, and a peak to the left means it lags.

\`XCorr(a, a)\` is the autocorrelation, which always peaks at the centre with the signal's energy \`Σ aᵢ²\`; the position of its next peak is the dominant period.

Both series are used raw. Remove any trend first (**Detrend**) — a common ramp correlates with itself and will dominate whatever you were actually looking for.

## Mathematical Formulation

$$ c_{\\ell} = \\sum_{j} a_j\\, b_{j-\\ell}, \\qquad \\ell = -(n-1),\\ \\dots,\\ m-1 $$

which is the convolution of \`a\` with \`b\` reversed.

> **Method:** direct sum, \`O(m·n)\`. The evaluator's equation budget rejects vectors long before an FFT-based correlation would win.

## Examples

### Example 1 — measuring a one-sample delay

\`\`\`
a = [0, 0, 1, 2, 1, 0, 0]
b = [0, 1, 2, 1, 0, 0, 0]
CALL XCorr(a, b : c)
\`\`\`

**Expected:** the peak lands one element right of the centre — \`a\` leads \`b\` by one sample.

### Example 2 — autocorrelation energy

\`\`\`
x = [1, 3, 2, 6, 4, 9, 5, 12]
CALL XCorr(x, x : r)
\`\`\`

**Expected:** \`r[8]\` (the centre of 15) is \`Σ xᵢ² = 316\`, and \`r\` is symmetric about it.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`a\` | Vector | Yes | The reference series, length \`m\`. |
| \`b\` | Vector | Yes | The series compared against it, length \`n\`. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`c\` | Vector | \`m + n − 1\` correlation values, lag \`−(n−1)\` first. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`XCorr requires the output length to be m + n - 1\` | A declared output of the wrong size. | Leave it bare and let it be sized, or declare \`[1:m+n-1]\`. |`,
  },
  {
    name: `stringlen`,
    slug: `stringlen`,
    category: `Strings`,
    summary: `Length of a string literal`,
    related: [],
    examples: [],
    tags: [`stringlen`, `strings`],
    references: [],
    guides: [`strings`],
    body: `Length of a string literal


## Syntax

\`\`\`
StringLen(s$)
\`\`\`

## Description

Length of a string literal

## Mathematical Formulation

$$ \\operatorname{StringLen}(s) = |s| $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`s$\` | String | Yes | String literal. |`,
  },
  {
    name: `stringpos`,
    slug: `stringpos`,
    category: `Strings`,
    summary: `1-based position of substring sub$ in s$ (0 if absent)`,
    related: [],
    examples: [],
    tags: [`stringpos`, `strings`],
    references: [],
    guides: [`strings`],
    body: `1-based position of substring sub$ in s$ (0 if absent)


## Syntax

\`\`\`
StringPos(s$, sub$)
\`\`\`

## Description

1-based position of substring sub$ in s$ (0 if absent)

## Mathematical Formulation

$$ \\operatorname{StringPos}(s, t) = \\text{1-based index of } t \\text{ in } s,\\ 0 \\text{ if absent} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`s$\` | String | Yes | String literal. |
| \`sub$\` | String | Yes | Substring to search for. |`,
  },
  {
    name: `stringval`,
    slug: `stringval`,
    category: `Strings`,
    summary: `Parse a numeric string literal to a value`,
    related: [],
    examples: [],
    tags: [`stringval`, `strings`],
    references: [],
    guides: [`strings`],
    body: `Parse a numeric string literal to a value


## Syntax

\`\`\`
StringVal(s$)
\`\`\`

## Description

Parse a numeric string literal to a value

## Mathematical Formulation

$$ \\operatorname{StringVal}(s) = \\text{numeric value parsed from } s $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`s$\` | String | Yes | String literal. |`,
  },
  {
    name: `IntegralValue`,
    slug: `integralvalue`,
    category: `Tables`,
    summary: `Trapezoidal integral of one column versus another (ODE or table data).`,
    related: [`TableAvg`, `ODEValue`, `integral`],
    examples: [`driving-cycle-energy`],
    tags: [`accessor`, `integral`, `trapezoidal`, `table`, `ode`, `area`],
    references: [],
    guides: [`optimization`, `table-accessors`],
    body: `Returns the **trapezoidal integral** of one column with respect to another — the
area under \`y\` plotted against \`x\` — over the sampled data of a \`DYNAMIC\` or table
result. Use it to accumulate a transient quantity, e.g. energy from power over a
drive cycle.

## Syntax

\`\`\`
A = IntegralValue('y', 'x')
\`\`\`

## Description

\`IntegralValue\` integrates the \`y\` column against the \`x\` column using the
composite trapezoidal rule over their shared samples.

## Mathematical Formulation

$$ A = \\int y\\,dx \\approx \\sum_{i=0}^{N-1} \\frac{y_i + y_{i+1}}{2}\\,(x_{i+1} - x_i) \\qquad \\text{(trapezoidal rule)} $$

> **Method:** composite trapezoidal quadrature over the column samples.

## Examples

### Example 1 — Energy from power over a drive cycle

[Run: driving-cycle-energy]

**Expected:** the integral of power versus time — the cycle energy [J].

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'y'\` | String | Yes | Integrand column. |
| \`'x'\` | String | Yes | Integration-variable column. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`A\` | Number | The trapezoidal integral ∫ y dx. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_COLUMN\` | \`'y'\` or \`'x'\` not a column | Use valid column names from the result table. |`,
  },
  {
    name: `nparametricruns`,
    slug: `nparametricruns`,
    category: `Tables`,
    summary: `Total number of configured parametric runs`,
    related: [],
    examples: [],
    tags: [`nparametricruns`, `tables`],
    references: [],
    guides: [`table-accessors`],
    body: `Total number of configured parametric runs


## Syntax

\`\`\`
NParametricRuns()
\`\`\`

## Description

Total number of configured parametric runs


## Mathematical Formulation

$$ \\text{total number of configured parametric runs} $$`,
  },
  {
    name: `TableAvg`,
    slug: `tableavg`,
    category: `Tables`,
    summary: `Average of a column across the parametric table runs.`,
    related: [`TableSum`, `TableMin`, `TableMax`, `TableStdDev`],
    examples: [`driving-cycle-energy`],
    tags: [`accessor`, `parametric table`, `average`, `mean`, `column`],
    references: [],
    guides: [`optimization`, `table-accessors`],
    body: `Returns the **arithmetic mean** of a named column across all rows of the
parametric table. Use it to summarize a swept study — e.g. the average consumption
over the points of a drive cycle.

## Syntax

\`\`\`
m = TableAvg('col')
\`\`\`

## Description

After a parametric (swept) solve, each variable becomes a column with one value per
run. \`TableAvg\` averages the requested column over all runs.

## Mathematical Formulation

For a column with values $c_1, \\dots, c_n$ over \`n\` runs,

$$ \\text{TableAvg}('col') = \\frac{1}{n}\\sum_{i=1}^{n} c_i $$

> **Method:** arithmetic mean over the parametric-table rows.

## Examples

### Example 1 — Average over a drive-cycle sweep

[Run: driving-cycle-energy]

**Expected:** the mean of the requested column across the table's runs.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | String | Yes | Name of a parametric-table column. |

## Output Arguments

| Argument | Type | Description |
| --- | --- | --- |
| \`m\` | Number | Mean of the column across all runs. |

## Common Errors

| Error | Cause | Fix |
| --- | --- | --- |
| \`UNKNOWN_COLUMN\` | \`'col'\` not a table column | Use a variable present in the parametric table. |
| \`NO_TABLE\` | No parametric table has been solved | Run the parametric table first (Solve Table). |`,
  },
  {
    name: `tablemax`,
    slug: `tablemax`,
    category: `Tables`,
    summary: `Maximum of a parametric-table column`,
    related: [],
    examples: [],
    tags: [`tablemax`, `tables`],
    references: [],
    guides: [`table-accessors`],
    body: `Maximum of a parametric-table column


## Syntax

\`\`\`
TableMax('col')
\`\`\`

## Description

Maximum of a parametric-table column

## Mathematical Formulation

$$ \\max_r c_r $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `tablemin`,
    slug: `tablemin`,
    category: `Tables`,
    summary: `Minimum of a parametric-table column`,
    related: [],
    examples: [],
    tags: [`tablemin`, `tables`],
    references: [],
    guides: [`table-accessors`],
    body: `Minimum of a parametric-table column


## Syntax

\`\`\`
TableMin('col')
\`\`\`

## Description

Minimum of a parametric-table column

## Mathematical Formulation

$$ \\min_r c_r $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `tablerun#`,
    slug: `tablerun#`,
    category: `Tables`,
    summary: `Current parametric run index (1-based)`,
    related: [],
    examples: [],
    tags: [`tablerun#`, `tables`],
    references: [],
    guides: [`table-accessors`],
    body: `Current parametric run index (1-based)


## Syntax

\`\`\`
TableRun#()
\`\`\`

## Description

Current parametric run index (1-based)


## Mathematical Formulation

$$ \\text{current parametric run index (1-based)} $$`,
  },
  {
    name: `tablestddev`,
    slug: `tablestddev`,
    category: `Tables`,
    summary: `Standard deviation of a parametric-table column`,
    related: [],
    examples: [],
    tags: [`tablestddev`, `tables`],
    references: [],
    guides: [`table-accessors`],
    body: `Standard deviation of a parametric-table column


## Syntax

\`\`\`
TableStdDev('col')
\`\`\`

## Description

Standard deviation of a parametric-table column

## Mathematical Formulation

$$ s = \\sqrt{\\tfrac{1}{n-1}\\sum_r (c_r - \\bar c)^2} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `tablesum`,
    slug: `tablesum`,
    category: `Tables`,
    summary: `Sum of a parametric-table column`,
    related: [],
    examples: [],
    tags: [`tablesum`, `tables`],
    references: [],
    guides: [`table-accessors`],
    body: `Sum of a parametric-table column


## Syntax

\`\`\`
TableSum('col')
\`\`\`

## Description

Sum of a parametric-table column

## Mathematical Formulation

$$ \\sum_{r} c_r $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`'col'\` | Number | Yes | Name of a result-table column (string). |`,
  },
  {
    name: `tablevalue`,
    slug: `tablevalue`,
    category: `Tables`,
    summary: `Cell value in the parametric table`,
    related: [],
    examples: [],
    tags: [`tablevalue`, `tables`],
    references: [],
    guides: [`table-accessors`],
    body: `Cell value in the parametric table


## Syntax

\`\`\`
TableValue(run, col)
\`\`\`

## Description

Cell value in the parametric table

## Mathematical Formulation

$$ \\operatorname{TableValue}(r, c) = \\text{cell } (r, c) \\text{ of the parametric table} $$

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`run\` | Number | Yes | Parametric run index. |
| \`col\` | Number | Yes | Name of a result-table column. |`,
  },
  {
    name: `chen_f`,
    slug: `chen_f`,
    category: `Two-Phase Flow`,
    summary: `Chen flow-boiling convective enhancement factor F`,
    related: [],
    examples: [],
    tags: [`chen`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Chen flow-boiling convective enhancement factor F


## Syntax

\`\`\`
chen_f(X_tt)
\`\`\`

## Description

Returns the **convective enhancement factor \`F\`** of the Chen flow-boiling model — the factor by which two-phase convection exceeds the liquid-only value, as a function of the Martinelli parameter.

## Mathematical Formulation

$$ F = \\big[1 + X_{tt}^{-1}\\big]^{0.736} \\text{-type convective enhancement (Chen)} $$

## Applicability

- **Where it applies:** Saturated flow boiling of a refrigerant in evaporator tubes.
- **Valid when:** Saturated (not subcooled) flow boiling; \`F ≥ 1\`, rising as the vapor fraction grows.
- **How it's used:** Used with \`chen_s\` in the Chen superposition \`h = F·h_conv + S·h_nb\`, where \`h_conv\` is the liquid-only convective coefficient.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`X_tt\` | Number | Yes | Turbulent–turbulent Martinelli parameter. |`,
  },
  {
    name: `chen_s`,
    slug: `chen_s`,
    category: `Two-Phase Flow`,
    summary: `Chen flow-boiling nucleate-suppression factor S`,
    related: [],
    examples: [],
    tags: [`chen`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Chen flow-boiling nucleate-suppression factor S


## Syntax

\`\`\`
chen_s(Re_l, F)
\`\`\`

## Description

Returns the **nucleate-boiling suppression factor \`S\`** of the Chen flow-boiling model — the factor that throttles the pool-boiling nucleate term as the bulk velocity rises.

## Mathematical Formulation

$$ S = \\frac{1}{1 + 2.53\\times10^{-6}\\,Re_l^{1.17}} \\quad\\text{(nucleate suppression, Chen)} $$

## Applicability

- **Where it applies:** Saturated flow boiling of a refrigerant in evaporator tubes.
- **Valid when:** Saturated flow boiling; \`S ≤ 1\`, falling as the two-phase Reynolds number increases.
- **How it's used:** Used with \`chen_f\` in the Chen superposition \`h = F·h_conv + S·h_nb\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re_l\` | Number | Yes | Liquid-only Reynolds number. |
| \`F\` | Number | Yes | Convective enhancement factor. |`,
  },
  {
    name: `friedel_phi2`,
    slug: `friedel_phi2`,
    category: `Two-Phase Flow`,
    summary: `Friedel two-phase frictional multiplier on the liquid-only drop`,
    related: [],
    examples: [],
    tags: [`friedel`, `phi2`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Friedel two-phase frictional multiplier on the liquid-only drop


## Syntax

\`\`\`
friedel_phi2(x, rho_l, rho_g, mu_l, mu_g, G, D, sigma)
\`\`\`

## Description

Returns the **Friedel two-phase frictional multiplier** on the liquid-only pressure drop — an alternative to Chisholm that uses the Froude and Weber numbers for broader validity.

## Mathematical Formulation

$$ \\phi_{lo}^2 = E + \\frac{3.24\\,F H}{Fr^{0.045}We^{0.035}} \\quad\\text{(Friedel)} $$

## Applicability

- **Where it applies:** Two-phase frictional pressure drop in refrigerant passages.
- **Valid when:** Recommended for \`μ_l/μ_g < 1000\`; covers a wider mass-flux range than the simple Chisholm form.
- **How it's used:** Multiply the liquid-only gradient by the multiplier to get the two-phase \`ΔP\`; an alternative to \`lm_phi2\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`rho_l\` | Number | Yes | Saturated-liquid density [kg/m³]. |
| \`rho_g\` | Number | Yes | Saturated-vapor density [kg/m³]. |
| \`mu_l\` | Number | Yes | Liquid dynamic viscosity [Pa·s]. |
| \`mu_g\` | Number | Yes | Vapor dynamic viscosity [Pa·s]. |
| \`G\` | Number | Yes | Mass flux G = ṁ/Aflow [kg/m²·s]. |
| \`D\` | Number | Yes | Diameter [m]. |
| \`sigma\` | Number | Yes | Surface tension [N/m]. |`,
  },
  {
    name: `lm_martinelli_tt`,
    slug: `lm_martinelli_tt`,
    category: `Two-Phase Flow`,
    summary: `Turbulent-turbulent Martinelli parameter X_tt`,
    related: [],
    examples: [],
    tags: [`lm`, `martinelli`, `tt`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Turbulent-turbulent Martinelli parameter X_tt


## Syntax

\`\`\`
lm_martinelli_tt(x, rho_l, rho_g, mu_l, mu_g)
\`\`\`

## Description

Returns the **turbulent–turbulent Lockhart–Martinelli parameter \`X_tt\`** — the ratio of the liquid-alone to vapor-alone pressure gradients that two-phase correlations key on.

## Mathematical Formulation

$$ X_{tt} = \\left(\\frac{1-x}{x}\\right)^{0.9}\\left(\\frac{\\rho_g}{\\rho_l}\\right)^{0.5}\\left(\\frac{\\mu_l}{\\mu_g}\\right)^{0.1} $$

## Applicability

- **Where it applies:** The independent variable for two-phase heat-transfer and pressure-drop correlations.
- **Valid when:** Both phases turbulent (the usual refrigerant case).
- **How it's used:** Feeds \`lm_phi2\`, the Chen factors, and many two-phase Nusselt correlations.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`rho_l\` | Number | Yes | Saturated-liquid density [kg/m³]. |
| \`rho_g\` | Number | Yes | Saturated-vapor density [kg/m³]. |
| \`mu_l\` | Number | Yes | Liquid dynamic viscosity [Pa·s]. |
| \`mu_g\` | Number | Yes | Vapor dynamic viscosity [Pa·s]. |`,
  },
  {
    name: `lm_phi2`,
    slug: `lm_phi2`,
    category: `Two-Phase Flow`,
    summary: `Chisholm two-phase multiplier 1+C/X+1/X^2 on the liquid-alone drop`,
    related: [],
    examples: [],
    tags: [`lm`, `phi2`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Chisholm two-phase multiplier 1+C/X+1/X^2 on the liquid-alone drop


## Syntax

\`\`\`
lm_phi2(X, C)
\`\`\`

## Description

Returns the **Chisholm two-phase frictional multiplier** \`φ_l² = 1 + C/X + 1/X²\` on the liquid-alone pressure gradient — i.e. how much more pressure two-phase flow drops than the liquid flowing alone.

## Mathematical Formulation

$$ \\phi_l^2 = 1 + \\frac{C}{X} + \\frac{1}{X^2} \\quad\\text{(Chisholm)} $$

## Applicability

- **Where it applies:** Two-phase frictional pressure drop in refrigerant evaporator/condenser passages.
- **Valid when:** Separated two-phase flow; the Chisholm constant \`C\` ranges 5 (laminar–laminar) to 20 (turbulent–turbulent).
- **How it's used:** Multiply the liquid-only Darcy gradient by \`φ_l²\` (with \`lm_martinelli_tt\` supplying \`X\`) to get the two-phase frictional \`ΔP\`. Friedel (\`friedel_phi2\`) is an alternative.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`X\` | Number | Yes | Lockhart–Martinelli parameter. |
| \`C\` | Number | Yes | Empirical constant. |`,
  },
  {
    name: `momentum_flux`,
    slug: `momentum_flux`,
    category: `Two-Phase Flow`,
    summary: `Separated-flow momentum flux [Pa] (accel. dP = out-in)`,
    related: [],
    examples: [],
    tags: [`momentum`, `flux`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Separated-flow momentum flux [Pa] (accel. dP = out-in)


## Syntax

\`\`\`
momentum_flux(x, rho_l, rho_g, alpha, G)
\`\`\`

## Description

Returns the **separated-flow momentum flux** — the acceleration pressure change (outlet − inlet) caused by the change in vapor quality, not by friction.

## Mathematical Formulation

$$ \\left(\\frac{d P}{d z}\\right)_{\\text{acc}} = G^2\\frac{d}{dz}\\left[\\frac{x^2}{\\rho_g\\alpha} + \\frac{(1-x)^2}{\\rho_l(1-\\alpha)}\\right] $$

## Applicability

- **Where it applies:** The acceleration \`ΔP\` term along an evaporator/condenser pass.
- **Valid when:** Wherever quality changes appreciably (vapor generation in an evaporator accelerates the flow).
- **How it's used:** Add it to the frictional (\`lm_phi2\`) and gravitational (\`dp_gravity\`) terms for the total pass \`ΔP\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`rho_l\` | Number | Yes | Saturated-liquid density [kg/m³]. |
| \`rho_g\` | Number | Yes | Saturated-vapor density [kg/m³]. |
| \`alpha\` | Number | Yes | Void fraction (0–1). |
| \`G\` | Number | Yes | Mass flux G = ṁ/Aflow [kg/m²·s]. |`,
  },
  {
    name: `nu_cavallini_zecchin`,
    slug: `nu_cavallini_zecchin`,
    category: `Two-Phase Flow`,
    summary: `Cavallini-Zecchin condensation Nusselt number`,
    related: [],
    examples: [],
    tags: [`nu`, `cavallini`, `zecchin`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Cavallini-Zecchin condensation Nusselt number


## Syntax

\`\`\`
nu_cavallini_zecchin(Re_l, Pr_l, x, rho_l, rho_g)
\`\`\`

## Description

Returns the **in-tube condensation Nusselt number** by the Cavallini–Zecchin correlation; the condensing-side film coefficient follows as \`h = Nu·k_l/D_h\`. It is one of the standard shear-dominated condensation correlations.

## Mathematical Formulation

$$ Nu = 0.05\\,Re_{eq}^{0.8}\\,Pr_l^{0.33} \\quad\\text{(Cavallini–Zecchin condensation)} $$

## Applicability

- **Where it applies:** The condensing two-phase refrigerant **inside the tubes of a condenser or gas-cooler**.
- **Valid when:** Annular, vapor-shear-controlled in-tube condensation with a turbulent liquid film; evaluate at the local vapor quality \`x\` (integrate across the pass for a mean value).
- **How it's used:** Convert to a film coefficient \`h = Nu·k_l/D_h\`, then combine it with the air/coolant side and the wall via \`ua_hx\`. Alternatives: \`nu_shah\` (broader range) and \`nu_traviss\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re_l\` | Number | Yes | Liquid-only Reynolds number. |
| \`Pr_l\` | Number | Yes | Liquid Prandtl number. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`rho_l\` | Number | Yes | Saturated-liquid density [kg/m³]. |
| \`rho_g\` | Number | Yes | Saturated-vapor density [kg/m³]. |`,
  },
  {
    name: `nu_dittus_boelter`,
    slug: `nu_dittus_boelter`,
    category: `Two-Phase Flow`,
    summary: `Dittus-Boelter single-phase Nusselt 0.023 Re^0.8 Pr^n`,
    related: [],
    examples: [],
    tags: [`nu`, `dittus`, `boelter`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Dittus-Boelter single-phase Nusselt 0.023 Re^0.8 Pr^n


## Syntax

\`\`\`
nu_dittus_boelter(Re, Pr, n)
\`\`\`

## Description

Returns the **single-phase turbulent Nusselt number** \`Nu = 0.023 Re^0.8 Pr^n\`. It is both a stand-alone single-phase film coefficient and the **liquid-only baseline** that flow-boiling and condensation correlations enhance.

## Mathematical Formulation

$$ Nu = 0.023\\,Re^{0.8}\\,Pr^{n} \\quad (n = 0.4 \\text{ heating},\\ 0.3 \\text{ cooling}) $$

## Applicability

- **Where it applies:** Fully-developed turbulent single-phase flow in a tube — or the liquid-only reference inside a two-phase correlation.
- **Valid when:** Smooth tube, \`Re ≳ 10⁴\`, \`0.7 ≲ Pr ≲ 120\`; \`n = 0.4\` when heating the fluid, \`0.3\` when cooling.
- **How it's used:** Feeds the convective term of the Chen flow-boiling model and the liquid-only term of \`nu_shah\`/\`nu_cavallini_zecchin\`. Use \`nu_gnielinski\` for better transitional accuracy.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re\` | Number | Yes | Reynolds number. |
| \`Pr\` | Number | Yes | Prandtl number. |
| \`n\` | Number | Yes | Order / number of terms. |`,
  },
  {
    name: `nu_gnielinski`,
    slug: `nu_gnielinski`,
    category: `Two-Phase Flow`,
    summary: `Gnielinski single-phase Nusselt number`,
    related: [],
    examples: [],
    tags: [`nu`, `gnielinski`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Gnielinski single-phase Nusselt number


## Syntax

\`\`\`
nu_gnielinski(Re, Pr)
\`\`\`

## Description

Returns the **single-phase Nusselt number** by the Gnielinski correlation — more accurate than Dittus–Boelter, especially in the transitional-turbulent band.

## Mathematical Formulation

$$ Nu = \\frac{(f/8)(Re-1000)Pr}{1 + 12.7\\sqrt{f/8}\\,(Pr^{2/3}-1)} $$

## Applicability

- **Where it applies:** Single-phase liquid or gas flow in a tube/channel (the preferred single-phase baseline).
- **Valid when:** Smooth tube, \`3000 ≲ Re ≲ 5×10⁶\`, \`0.5 ≲ Pr ≲ 2000\`; uses the Darcy friction factor.
- **How it's used:** The single-phase film coefficient (\`h = Nu·k/D_h\`) for coolant/oil/air lines, and the liquid-only baseline for two-phase correlations.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re\` | Number | Yes | Reynolds number. |
| \`Pr\` | Number | Yes | Prandtl number. |`,
  },
  {
    name: `nu_shah`,
    slug: `nu_shah`,
    category: `Two-Phase Flow`,
    summary: `Shah condensation Nusselt number`,
    related: [],
    examples: [],
    tags: [`nu`, `shah`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Shah condensation Nusselt number


## Syntax

\`\`\`
nu_shah(Re_l, Pr_l, x, p_red)
\`\`\`

## Description

Returns the **in-tube condensation Nusselt number** by the Shah correlation — an enhancement on the liquid-only Nusselt number that captures the thinning film and vapor shear.

## Mathematical Formulation

$$ Nu_{TP} = Nu_l\\left(1 + \\frac{3.8}{Z^{0.95}}\\right), \\quad Z = (1/x - 1)^{0.8}p_r^{0.4} \\quad\\text{(Shah)} $$

## Applicability

- **Where it applies:** The condensing two-phase refrigerant side of a condenser / gas-cooler.
- **Valid when:** In-tube condensation across a wide quality and reduced-pressure range; depends on the reduced pressure \`p_red\`.
- **How it's used:** Gives the condensing film coefficient (\`h = Nu·k_l/D_h\`) for the refrigerant side. A robust general-purpose alternative to \`nu_cavallini_zecchin\` / \`nu_traviss\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`Re_l\` | Number | Yes | Liquid-only Reynolds number. |
| \`Pr_l\` | Number | Yes | Liquid Prandtl number. |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`p_red\` | Number | Yes | Reduced pressure P/Pcrit. |`,
  },
  {
    name: `void_homogeneous`,
    slug: `void_homogeneous`,
    category: `Two-Phase Flow`,
    summary: `Homogeneous (no-slip) void fraction`,
    related: [],
    examples: [],
    tags: [`void`, `homogeneous`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Homogeneous (no-slip) void fraction


## Syntax

\`\`\`
void_homogeneous(x, rho_l, rho_g)
\`\`\`

## Description

Returns the **homogeneous (no-slip) void fraction** — the vapor volume fraction assuming both phases move at the same velocity.

## Mathematical Formulation

$$ \\alpha = \\frac{1}{1 + \\frac{1-x}{x}\\frac{\\rho_g}{\\rho_l}} \\quad\\text{(no slip)} $$

## Applicability

- **Where it applies:** The vapor fraction \`α\` used in two-phase density, charge, and gravitational-head terms.
- **Valid when:** High-mass-flux / bubbly flow where slip is negligible; the simplest model (it overpredicts \`α\` at low mass flux).
- **How it's used:** Feeds the two-phase mixture density and \`dp_gravity\`. For better accuracy use \`void_zivi\` or \`void_rouhani\`.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`rho_l\` | Number | Yes | Saturated-liquid density [kg/m³]. |
| \`rho_g\` | Number | Yes | Saturated-vapor density [kg/m³]. |`,
  },
  {
    name: `void_rouhani`,
    slug: `void_rouhani`,
    category: `Two-Phase Flow`,
    summary: `Rouhani-Axelsson drift-flux void fraction (default)`,
    related: [],
    examples: [],
    tags: [`void`, `rouhani`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Rouhani-Axelsson drift-flux void fraction (default)


## Syntax

\`\`\`
void_rouhani(x, rho_l, rho_g, G, sigma)
\`\`\`

## Description

Returns the **Rouhani–Axelsson drift-flux void fraction** (the default) — it accounts for both phase slip and the radial distribution of vapor.

## Mathematical Formulation

$$ \\alpha = \\frac{x}{\\rho_g}\\left[(1 + 0.12(1-x))\\left(\\frac{x}{\\rho_g} + \\frac{1-x}{\\rho_l}\\right) + \\frac{1.18(1-x)[g\\sigma(\\rho_l-\\rho_g)]^{0.25}}{G\\rho_l^{0.5}}\\right]^{-1} $$

## Applicability

- **Where it applies:** The general-purpose vapor fraction \`α\` for refrigerant evaporators and condensers.
- **Valid when:** Recommended across flow regimes and mass fluxes; the default void model.
- **How it's used:** Feeds the refrigerant charge inventory, mixture density, and the gravitational pressure term.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`rho_l\` | Number | Yes | Saturated-liquid density [kg/m³]. |
| \`rho_g\` | Number | Yes | Saturated-vapor density [kg/m³]. |
| \`G\` | Number | Yes | Mass flux G = ṁ/Aflow [kg/m²·s]. |
| \`sigma\` | Number | Yes | Surface tension [N/m]. |`,
  },
  {
    name: `void_zivi`,
    slug: `void_zivi`,
    category: `Two-Phase Flow`,
    summary: `Zivi void fraction (slip S=(rho_l/rho_g)^(1/3))`,
    related: [],
    examples: [],
    tags: [`void`, `zivi`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Zivi void fraction (slip S=(rho_l/rho_g)^(1/3))


## Syntax

\`\`\`
void_zivi(x, rho_l, rho_g)
\`\`\`

## Description

Returns the **Zivi void fraction**, using a slip ratio \`S = (ρ_l/ρ_g)^{1/3}\` from minimum-entropy-production — more realistic than the no-slip model.

## Mathematical Formulation

$$ \\alpha = \\frac{1}{1 + \\frac{1-x}{x}\\left(\\frac{\\rho_g}{\\rho_l}\\right)^{2/3}} \\quad\\text{(slip } S = (\\rho_l/\\rho_g)^{1/3}) $$

## Applicability

- **Where it applies:** The vapor fraction \`α\` for separated two-phase flow.
- **Valid when:** Moderate mass flux with appreciable slip; better than homogeneous, simpler than drift-flux.
- **How it's used:** Feeds mixture density / charge / static-head calculations.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`x\` | Number | Yes | Vapor quality (0–1). |
| \`rho_l\` | Number | Yes | Saturated-liquid density [kg/m³]. |
| \`rho_g\` | Number | Yes | Saturated-vapor density [kg/m³]. |`,
  },
  {
    name: `zone_ramp`,
    slug: `zone_ramp`,
    category: `Two-Phase Flow`,
    summary: `Smooth zone-collapse ramp tanh(L/eps) (moving-boundary models)`,
    related: [],
    examples: [],
    tags: [`zone`, `ramp`, `two`, `phase`, `flow`],
    references: [],
    guides: [],
    body: `Smooth zone-collapse ramp tanh(L/eps) (moving-boundary models)


## Syntax

\`\`\`
zone_ramp(L, eps)
\`\`\`

## Description

Returns a **smooth \`tanh(L/ε)\` ramp** that fades a moving-boundary zone in/out as its length \`L\` approaches zero. It is a numerical \`C¹\` smoothing, not a physical correlation.

## Mathematical Formulation

$$ r(L) = \\tanh\\!\\left(\\frac{L}{\\varepsilon}\\right) \\quad\\text{(smooth zone-collapse ramp)} $$

## Applicability

- **Where it applies:** Moving-boundary heat-exchanger models (subcooled / two-phase / superheat zones).
- **Valid when:** Whenever a zone length can shrink to zero during a solve/transient.
- **How it's used:** Blends a zone's contribution smoothly so the corrector does not chatter at a regime switch.

## Input Arguments

| Argument | Type | Required | Description |
| --- | --- | --- | --- |
| \`L\` | Number | Yes | Length [m]. |
| \`eps\` | Number | Yes | Effectiveness ε (0–1). |`,
  }
];
