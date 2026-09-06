[Topic: started]
Welcome to **frees** — a declarative equation-solving environment for engineering problems: thermodynamics, fluid mechanics, heat transfer, control systems, structural analysis, and multi-domain simulation.

You write equations the way they appear in a textbook; frees figures out what is unknown and in what order to solve it. This **Get Started** path is the fastest way in — work through the seven steps below in order, then use *Where to go next* to branch into the area you need.

[Diagram: SolverPipeline]

**New here? Start with step 1 below and press *Next* at the bottom of each page.** The rest of the portal is organized as: **Language Fundamentals** (the grammar), **Matrices**, **Programming & Tables**, **Fluids & Materials** (property data), **Solving & Optimization** (how the solver works, debugging, uncertainty), **Dynamic Systems & Control** (ODEs, transfer functions, Bode), **System Modeling with Components** (the acausal component library), **Tools & Workflow** (the REPL, shortcuts, reports), **Examples & Tutorials**, **Architecture & Deployment**, and a per-symbol **Reference**.

[Topic: gs-first-solve]
# 1. Your First Solve

The quickest way to understand frees is to solve something. Type this into the editor and press **F2** (Solve):

```run
{ Mass of air in a rigid tank }
P = 500 [kPa]
Vol = 0.05 [m^3]
T = 25 [C]
R = 0.287 [kJ/kg-K]
P * Vol = m * R * T      { frees solves this for m }
```

You never told frees to "compute `m`". It read the five equations, saw that `m` was the only unknown, and rearranged the ideal-gas relation to find it. The result appears in the **Solution** panel, in SI units, with any propagated uncertainty.

## Any variable can be the unknown
Swap one line — change `T = 25 [C]` to `m = 0.3 [kg]` — and the *same* equation now solves for temperature instead. You describe the physics; frees decides the calculation order. That is the whole idea, and the next page explains why it matters.

## The four-step loop
Every model follows the same rhythm:

1. **Describe** the system — algebraic, matrix, or differential equations, in any order.
2. **Check (F4)** — validates syntax and the degrees of freedom (see step 3).
3. **Solve (F2)** — runs the Newton–Raphson solver; results land in the Solution panel.
4. **Sweep** — optionally build a **Parametric Table** (`Ctrl + T`) to vary an input and plot the response.

[Related: gs-declarative, shortcuts, variables]

[Topic: gs-declarative]
# 2. Thinking Declaratively

In a traditional language you write *assignments*: `x = y + 2` means "compute `x` from `y`". frees is **declarative** — an `=` is a mathematical **equality**, a constraint that must hold once solved. The solver looks at the whole system at once and finds the values that satisfy every equation simultaneously.

```
P * V = m * R * T      { solve for m, or for V, or for T — all valid }
```

Because equations are constraints, **order does not matter** and **any variable on either side can be the unknown**. A consequence: you can transcribe a textbook problem line for line without first rearranging it to isolate the answer.

## Rules that follow from this
- A single `=` is equality, never assignment. There is no `==`.
- Names are **case-insensitive** — `Temp`, `TEMP`, and `temp` are one variable.
- Implicit multiplication is **not** allowed — write `2 * x`, not `2x`.
- Everything is computed in **SI base units** internally; you annotate inputs with `[unit]`.

The full grammar — operators, comments, constants — is in *Equation Syntax & Rules*. The next page covers the two things that most often decide whether a solve succeeds: units and the Check.

[Related: gs-units-check, syntax, variables]

[Topic: gs-units-check]
# 3. Units & Checking the Model

### Annotate inputs; read SI results
frees runs every calculation in SI base units. You annotate **inputs** for convenience and the compiler converts them at parse time:

```
P = 500 [kPa]      { stored as 500000 Pa }
T = 25 [C]         { stored as 298.15 K }
m = 120 [lb]       { stored as 54.43 kg }
```

Results come back in SI; convert or label them for display (see *Units & Consistency*). Mixing inconsistent units is reported as a warning, never a silent error — and warnings never block a solve.

### Check before you solve (F4)
A system is solvable only when the number of equations equals the number of unknowns — the **degrees of freedom (DoF)** are zero. Press **F4** (Check) to verify this *before* solving: it reports the DoF and any unit mismatches instantly, so you fix structural problems before the solver ever runs. Make F4-before-F2 a habit.

### Guesses make nonlinear solves converge
For nonlinear or transcendental equations, the Newton solver iterates from a **guess**. Open **Variable Info** (`Ctrl + I`) to set a starting guess and physical bounds (e.g. `T ≥ 0`, `0 ≤ x ≤ 1`). A good guess is usually the difference between convergence and divergence.

> **Tip:** If a solve fails to converge, the cause is almost always a missing guess or a wrong unit annotation — not a bug. Check the Solution panel's diagnostics and the Variable Info guesses first.

[Related: gs-plots, units, variables]

[Topic: gs-plots]
# 4. See It: Tables & Plots

A single answer is rarely the goal — engineers want the *response*: how the answer moves when an input does. In frees that is a **parametric sweep**, and it takes four lines more than your first solve:

```run vary=P=100000:25000:900000
P = 500 [kPa]
Vol = 0.05 [m^3]
T = 25 [C]
R = 0.287 [kJ/kg-K]
P * Vol = m * R * T

PARAMETRIC tank_sweep(T, m)
  T = 275 : 5 : 375 | Linear
END
```

*(Drag the `P` slider above the code — in pascals — and the whole system re-solves live, the same override mechanism the REPL uses.)*

The `PARAMETRIC` block **drives** `T` across the range (overriding any fixed value each run) and records `m` as a computed output. Open the **Tables** tab and click **Solve Table** — one solve per row fills the grid.

## From table to plot
Select the columns in the table and click **Plot curve** — the figure opens in the **Plots** panel. For figures you want built every solve, declare them in code with a `PLOT` block instead (see *Plots in Code*). Property plots (T-s, P-h, psychrometric) with your state points overlaid come later in the *Fluids & Materials* group.

That is the everyday loop: model → sweep → curve. The next two steps add the interactive console and the component library.

[Related: gs-repl, optimization, plot-code]

[Topic: gs-repl]
# 5. Ask Questions: the REPL

After a solve, the **REPL terminal** (a dockable console window) holds the whole solved session as a live **workspace**. Instead of editing the document to ask a side question, ask it directly:

```
>> m                                  { query a solved value -> with units }
>> m * 3600                           { unit-aware calculator; result stored in ans }
>> Enthalpy('Water', T=400, P=1e5)    { any property or math function }
>> vars                               { list the workspace }
```

Three things make it more than a calculator:

- **Implicit solve** — type an equation with one unknown and the REPL solves it on the spot.
- **The CALL library** — eigenvalues, Bode data, partial fractions: `CALL bode(num, den, omega : mag, phase)` works interactively, with output sizes inferred for you.
- **Symbolic CAS** — `Factor(x^2 - 1)`, `Apart(...)`, `Laplace(...)` return transformed expressions (REPL-only).

The full command set is on the *REPL Terminal & Workspace* page. One step left: components.

[Related: gs-components, repl, shortcuts]

[Topic: gs-components]
# 6. Wire Components

For system problems — loops, circuits, networks — frees has a library of ~295 **components**: parameterized, connectable blocks of physics. You wire them; frees turns the network into equations and solves it like everything else:

```run
{ What pressure does 50 m of pipe cost? }
Source  SUP(fluid$=Water, mdot=2 [kg/s], P=300000 [Pa], T=298 [K])
Pipe    LINE(fluid$=Water, L=50 [m], D=0.05 [m], rough=0.0001)
Sink    RET()

connect(SUP.out, LINE.in)
connect(LINE.out, RET.in)

dP = SUP.out.P - RET.in.P
```

Solve, and read `dP` — the `Pipe` computed density, Reynolds number, and friction factor internally. Port members like `LINE.out.P` are ordinary variables you can probe or pin.

This scales a long way: pumps, heat exchangers, refrigerant circuits, electrical and mechanical elements, humid-air HVAC — including transients, from the same wiring. The **System Modeling with Components** group teaches it properly, starting with *Your First Component Network*.

[Related: gs-next, comp-first-network, comp-library]

[Topic: gs-next]
# 7. Where to Go Next

You now know the whole loop: describe equations, Check (F4), Solve (F2), sweep and plot, ask follow-ups in the REPL, and wire component networks. Where you go next depends on what you're modeling — the map below is clickable.

[Diagram: LearningMap]

## Pick your direction
- **Master the language** — operators, arrays, complex numbers, and strings: *Language Fundamentals*.
- **Work with matrices** — declare, operate, and solve linear systems: *Matrices & Linear Algebra*.
- **Reuse logic & data** — custom functions, submodels, and tables: *Programming & Tables*.
- **Use property data** — real fluids, ideal gases, humid air, and solid materials: *Fluids & Materials*.
- **Understand and steer the solver** — convergence, debugging, uncertainty propagation, and optimization: *Solving & Optimization*.
- **Go dynamic** — ODE transients, linearization, transfer functions, and Bode plots: *Dynamic Systems & Control*.
- **Model whole systems** — the acausal component library, from a pipe run to a full refrigeration loop: *System Modeling with Components*.
- **Work faster** — the REPL console, keyboard shortcuts, and automated reports: *Tools & Workflow*.
- **Run it yourself** — the async architecture, the REST API, Docker, and Railway: *Architecture & Deployment*.

## Learn by example
Three short journeys — a scalar with units, a component chain, and a map or transient upgrade — are in *Journey 1 — A scalar equation with units*. **Examples & Tutorials** has longer guided problems and a library of verified examples. When you need the exact signature of a function, the **Reference** A–Z index is the canonical home for every symbol.

[Related: journey-scalar, journey-chain, journey-upgrade, lang-overview, fluids-overview, components-overview, examples]

[Topic: repl]
# REPL Terminal & Workspace

The **REPL terminal** is a dockable, interactive console — move and dock it anywhere like the editor. It evaluates **one line at a time** against the current **workspace** (every variable from the last solve, plus anything you define in the REPL). It's a line-oriented math REPL, not a shell: use it as a unit-aware calculator, to inspect solved values, to try `CALL` routines, and to run symbolic CAS transforms. **Up/Down** recall history; **Tab** completes variable, function, and command names.

## Meta-commands
These drive the app instead of evaluating an expression:

| Command | Action |
| --- | --- |
| `help` | show in-terminal usage |
| `clc` | clear the screen |
| `clear` | drop **all** REPL-defined overrides |
| `clear <var>` | drop one REPL variable overlay (e.g. `clear x`) |
| `vars` / `who` / `whos` | list workspace variables with values and units |
| `check` | run the document Check (DoF / solvability) |
| `solve` | solve the document with any active REPL overrides |

## Expressions
Type any expression; a bare result is stored in `ans` and is reusable on the next line. Every built-in math function works (trig, `exp`/`ln`/`sqrt`, `erf`/`gamma`, Bessel, `mod`/`gcd`, complex `real`/`imag`/`angle`, …), as do fluid-property and chemistry functions.
```
2 * sqrt(9) + 4                    { = 10 }
Enthalpy('Water', t=400, p=1e5)    { J/kg }
```

## Variables: query, assign, solve
- **Query** a workspace value (shown with units and uncertainty): `T_1` → `300 [K]`.
- **Assign** a REPL variable (persists for the session, visible to later lines and a subsequent `solve`): `x = 42 [m/s]`.
- **Implicit single-unknown solve** — give an equation with exactly one unknown and frees solves it: `P = 50000 * volume` → `volume = 5 [m^3]`.

## Matrices, vectors, ranges
```
A = [2 0; 0 3]          { = [2 0; 0 3] }
[1:2:7]                 { = [1 3 5 7] }
A * A                   { matrix product -> ans[i,j] }
Inverse(A)   Transpose(A)   Dot(u, v)
```

## The CALL library (auto-sized outputs)
The full `CALL` procedure library (eigenvalues, control-systems analysis, partial fractions, decompositions) runs in the REPL. **Output lengths are sized automatically from the inputs**, so bare output names work — no `[1:n]` annotation:
```
CALL Eigenvalues(A : lambda)            { lambda = [2 3] }
CALL Routh(den : nRHP, stable)
CALL residue(num, den : rr, ri, pr, pi, k)
CALL Bode(num, den, omega : mag, phase)
```
Only genuinely value-dependent counts take an explicit size: the finite-zero counts of `zero`/`tf2zp` (e.g. `zr[1:2]`), and the root-locus sweep resolution of `rlocus` (defaults to 100 points). This auto-sizing applies in the editor document too.

## Symbolic CAS (REPL only)
The REPL exposes the embedded **Symja** computer-algebra engine as functions that return a transformed expression as text. Free variables stay symbolic, so no solved context is needed:

| Function | Example → result |
| --- | --- |
| `Factor(expr)` | `Factor(x^2 - 1)` → `(-1+x)*(1+x)` |
| `Expand(expr)` | `Expand((x+1)^3)` → `1+3*x+3*x^2+x^3` |
| `Simplify(expr)` | algebraic simplification |
| `Together(expr)` / `Cancel(expr)` | common denominator / cancel common factors |
| `Numerator(expr)` / `Denominator(expr)` | split a rational expression |
| `Collect(expr, var)` | group by powers of a variable |
| `Diff(expr, var)` | `Diff(x^3 + x^2, x)` → `2*x+3*x^2` |
| `Integrate(expr, var)` | `Integrate(x^2, x)` → `x^3/3` |
| `Apart(expr, var)` | `Apart((s+3)/(s^2+3*s+2), s)` → `2/(1+s)-1/(2+s)` |
| `Laplace(f, t, s)` | Laplace transform |
| `InverseLaplace(F, s, t)` | `InverseLaplace(1/(s+2), s, t)` → `E^(-2*t)` |

When the CAS can't find a closed form, the REPL reports *"no closed form found"* rather than echoing the call. These symbolic functions are **REPL-only**; in the editor, symbolic work uses `SYMBOLIC` identities and `CALL residue` (see *Control Systems & Symbolic CAS*).

## What the REPL does not do
The REPL evaluates a single expression per line, so multi-line block constructs are editor-only: `FUNCTION`/`PROCEDURE`/`MODULE` definitions, `DYNAMIC` ODE systems, `TABLE` blocks, `IF`/`FOR` control flow, and the `SYMBOLIC`/`SOLVE BLOCK` directives. You can *call* a function or read `ODEValue`/`Interpolate`/table accessors that a prior solve produced — you just can't *define* the block from the REPL.

[Related: shortcuts, symbolic-cas, matrices-sys]

[Topic: shortcuts]
# Keyboard Shortcuts

| Hotkey | Action |
| --- | --- |
| `F2` or `Ctrl + Enter` | **Solve** — runs the Newton-Raphson solver |
| `F4` or `Ctrl + K` | **Check** — validates syntax, degrees of freedom, and expands blocks |
| `Ctrl + I` | Open the **Variable Information** panel (guesses & bounds) |
| `Ctrl + T` | Open the **Parametric Table** panel |
| `F9` | **Solve selected block only** — ignores all other lines |
| `F1` | **Contextual help** — opens the reference page for the symbol under the cursor |

> **Tip:** make `F4` (Check) a habit before `F2` (Solve). It reports the DoF and any unit mismatches instantly, so you fix problems before the solver runs. For parametric-table examples, use **Solve Table** in the Tables tab instead of `F2`.

[Related: gs-units-check, variables, repl]

[Topic: reports]
# Notes & Narrative

Structure a document as a readable calculation note: equations in any order, with the narrative alongside them as comments. Comments never affect solving.

## Mixing narrative and equations
- A line starting with `//`, or any text inside `{ }`, is a comment. Use them for section headings, explanations, and data provenance.
- A `{ }` comment may span several lines, so a paragraph of narrative can sit above the equations it describes.
- An inline comment documents a single equation in place: `Q = m*cp*dT { sensible heat }`.

## Named figures from code
A `PLOT ... END` block (see *Plots in Code*) declares a named figure that appears in the **Plots** window after every solve, so a document carries its own charts next to the equations they visualize.

## Example
```
{ Boiler Analysis — the pressure and firing temperature
  below fix the cycle's thermal efficiency. }

P_high = 8000 [kPa]
T_boiler = 500 [C]
eta_th = 36.9
```
Press Solve (F2): solved values appear in the Solution window, and any PLOT figures in the Plots window.

[Related: plot-code, reports]

[Topic: digitizer-fit]
# Graph Digitizer & Curve Fit

Two integrated tools turn data — measured or read off a chart — into usable equations: the **Graph Digitizer** extracts (x, y) points from an image, and the **Curve Fit Engine** fits a model to a table and writes the equation for you.

## Digitizer workflow
1. **Open** the Graph Digitizer icon in the left toolbar and upload an image of your chart.
2. **Calibrate** — mark two known points on the X-axis and two on the Y-axis to set the coordinate system.
3. **Digitize** — click points along the curve; their coordinates are computed and added to a table.
4. **Export** to an internal table (e.g. `digitized_curve`).

## Curve fit workflow
5. Open the **Curve Fit** panel, select your table, choose a model template (Linear, Polynomial, Exponential, …), and fit.
6. Copy the generated equation into the editor. The fit is returned as a plain frees expression you can paste straight in:

```
{ Fit of pump head vs flow, from a digitized catalog curve }
flow_rate = 1.25 [m^3/s]
head_loss [m] = -0.084 * flow_rate^2 + 1.54 * flow_rate + 0.12 [m]
```

> **Tip:** you can also define the data inline with a `TABLE` block (see *Custom Tables*) and fit against that — handy for reproducing a textbook table without an image. The statistics example in the Examples Library shows exactly this route.

[Related: tables-code, lookup-tables, reports]
