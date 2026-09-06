[Topic: functions]
# Custom Functions & Procedures

Most of your model is declarative — equations in any order, solved simultaneously. `FUNCTION` and `PROCEDURE` are for the parts that need **sequential, imperative** logic (loops, conditionals, step-by-step algorithms). Inside them you use `:=` for assignment, just like Python or other array languages.

## Functions
A `FUNCTION` returns one or more values. Assign the return value(s) with `:=`.
- **Single output** — assign the function's own name:
```
FUNCTION poly_fit(x)
  poly_fit := 0.5 * x^2 + 2 * x + 1
END

y = poly_fit(3)          { y = 9.5 }
```
- **Multiple outputs** — declare them in brackets in the header (array-language-style):
```
FUNCTION [q, r] = DivMod(a, b)
  q := trunc(a / b)
  r := mod(a, b)
END

[quotient, remainder] = DivMod(17, 5)   { 3, 2 }
```
Discard an output you don't need with `~`, or simply leave off trailing outputs:
```
[quotient, ~] = DivMod(17, 5)   { quotient only }
```
The same `[ … ] = name( … )` destructuring works for built-in multi-output `CALL` functions too — e.g. `[A, B, C, D] = tf2ss(num, den)`. See *Control Systems & Symbolic CAS → Multi-Output Functions*.

## Procedures
A `PROCEDURE` is the same idea with inputs and outputs separated by a colon. Call it with `CALL`:
```
PROCEDURE heat_transfer(T1, T2 : Q_dot)
  Q_dot := 0.8 * 12 * (T1 - T2) / 0.25
END

CALL heat_transfer(100, 20 : heat_loss)
```

## Control flow inside functions & procedures
Sequential structures work inside function/procedure bodies (not in the declarative top level):
- **Conditional:** `IF condition THEN ... ELSE ... END`
- **While:** `WHILE condition DO ... END`
- **Repeat:** `REPEAT ... UNTIL condition`

> **Declarative vs. imperative:** the top-level solver reorders your equations freely, so `x = y + 2` and `y = x - 2` are equivalent there. Inside a `FUNCTION`/`PROCEDURE`, order matters and `:=` is a one-way assignment — read it left-to-right like a normal program.

[Related: modules, symbolic-cas, arrays]

[Topic: tables-code]
# Custom Tables (TABLE)

Define a lookup table inline with a `TABLE` block. Once compiled it is registered as a callable function — call it like `tname(x)` to interpolate.

## Syntax
Column 1 is the independent (x) column. Annotate the input and output units in the header and frees propagates them to anything computed from the table:
```
{ Pressure [Pa] vs flow rate [m^3/s] }
TABLE pump_curve(flow [m^3/s]) [Pa]
  0.0       50000
  0.001     45000
  0.002     32000
  0.003     0
END

{ Linear interpolation, with [Pa] units carried to dP }
dP = pump_curve(0.0015)
```
For a curve family (a table parameterised by a second variable, e.g. an engine map), add `: param = v1, v2, …` after the x column and call it as `tname(x, y)` for bilinear interpolation — see *Lookup Tables & Interpolation*.

[Related: lookup-tables, digitizer-fit, table-accessors]

[Topic: lookup-tables]
# Lookup Tables & Interpolation

frees provides functions to query, search, and interpolate a named `TABLE` block. In a TABLE, **column 1 is the x axis** and each further column is a y/curve column. The simplest way to interpolate is to call the table like a function — `tname(x)` (1-D) or `tname(x, y)` (bilinear across a curve family). The functions below are the classic-solver-compatible equivalents.

## Interpolation functions
- **`Interpolate('tname', x)`** — piecewise-linear interpolation at `x` (same as `tname(x)`).
- **`Interpolate1('tname', x)`** — cubic-spline interpolation (falls back to linear below 3 points).
- **`Interpolate2D('tname', x, y)`** — bilinear 2-D interpolation across a curve family (same as `tname(x, y)`).
- **`Differentiate('tname', y_col, x_col, x_val)`** — numerical $dy/dx$ at $x_{val}$ (finite difference).
- **`Differentiate1('tname', y_col, x_col, x_val)`** — cubic-spline derivative.

## Lookup functions
- **`Lookup('tname', row, col)`** — cell value by 1-based row/column.
- **`LookupRow('tname', col, val)`** — fractional 1-based row where `col` crosses `val`.
- **`NLookupRows('tname')`** — number of data rows.

## 2-D engine-map example
```
TABLE bsfc(rpm : load = 0.25, 0.50, 1.0)
  1000   320   300   290
  3000   280   260   250
  5000   300   270   255
END

g_per_kWh = Interpolate2D('bsfc', 2500, 0.6)   { same as bsfc(2500, 0.6) }
```

## Tables you don't type
A Function Table built in the **Tables** window is callable exactly like a `TABLE` block, so measured or swept data reaches the equations without being retyped:

- **Import CSV…** (the `+` menu in the Tables window) reads a `.csv` **in your browser** — nothing is uploaded. Pick the x column, the y column and a function name; blank and non-numeric cells are skipped and counted, duplicate x values keep the first row, and a recording longer than the 5 000-row table cap is thinned uniformly (first and last point always kept).
- **Create function…** does the same for two columns of a solved parametric or `DYNAMIC` table, optionally with a family-parameter column for a 2-D `name(x, param)`.
- The **Graph Digitizer** exports its digitized points the same way.

One rule holds for all three: if the document already defines a `TABLE` with that name, **the document's table wins** when the equations are solved — a GUI table never overrides it.

## What wins, and what a file carries

| Input | Origin | Precedence |
| --- | --- | --- |
| `TABLE` / `FUNCTION` in the document | equation text | **Wins** over a GUI table of the same name |
| GUI Function Table, CSV import, digitizer map | Tables window / project file | Used only when the document does not define that name |
| `GUESS` in the document | equation text | **Wins** over Variable Information on conflict |
| Variable Information | project file | Fills in where the text is silent |
| Workspace sliders | project file | Applied as overrides on each solve (last write wins among overrides) |
| REPL assignments | this session only | Override the solved workspace until `clear`; not in Export equation text |

**Save project** writes a `.frees` JSON file: equation text plus Variable Information, GUI tables, maps, plots, schematic layout, sliders, and opaque legacy slices (spreadsheets / analyzer sessions) so a reload reproduces the effective model.

**Export equation text** writes only the editor document. Anything in the table above that is not in the text is listed in the export notice — those inputs will not travel with a copy-paste of the equations.

[Related: tables-code, table-accessors, digitizer-fit]

[Topic: table-accessors]
# Table Accessors & Aggregates

Query cells or statistical summaries of the active **Parametric Table**. These are computed once per table solve and are identical in every row — handy for reporting a cycle total or average alongside each run.

## Accessor functions
- **`TableValue(run, col)`** — a cell value in the parametric table.
- **`TableRun#()`** — the current run index (1-based).
- **`NParametricRuns()`** — total configured runs.
- **`TableSum('col')` / `TableAvg('col')`** — sum / average of a column.
- **`TableMin('col')` / `TableMax('col')`** — minimum / maximum.
- **`TableStdDev('col')`** — standard deviation.
- **`IntegralValue('y_col', 'x_col')`** — trapezoid integral of one column vs. another.

## Example
```
{ Whole-cycle energy from a speed-sweep table }
E_total = IntegralValue('P', 't')      { trapezoid integral of power over time }
P_avg   = TableAvg('P')                { mean power, same in every row }
current_index = TableRun#()
```

[Related: optimization, lookup-tables, plot-code]

[Topic: modules]
# Modular Submodels (MODULE)

A `MODULE` is a reusable **declarative** sub-system — a named bag of equations solved simultaneously with the rest of your model. Unlike a `FUNCTION`, a module's equations can be solved in **either direction**: a variable you pass in as an output one call can be passed in as an input the next.

## Why use a module?
- Encapsulate a recurring sub-model (a heat exchanger, a pipe segment, a pump) once and `CALL` it many times.
- Reuse the same equations whether you're sizing (unknown is an output) or rating (unknown is an input).

## Example
```
MODULE pipe_flow(D, Q : dP)
  V  = Q / (pi# / 4 * D^2)
  dP = 0.02 * (100 / D) * (1000 * V^2 / 2)
END

CALL pipe_flow(D1, Q1 : dP1)     { rating:  find dP1 from D1, Q1 }
CALL pipe_flow(D2, Q2 : dP2)     { sizing:  find Q2  from D2, dP2 }
```
Both calls use the *same* module — frees figures out which variable is unknown in each.

> **Module vs. function:** a `MODULE` is essentially a multi-output `FUNCTION` whose body is **equations** (`=`, solved in any direction) instead of sequential assignments (`:=`, one-way). The bracket call form works here too: `[dP1] = pipe_flow(D1, Q1)`.

[Related: functions, prog-overview, arrays]
