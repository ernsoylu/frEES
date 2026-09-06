// GENERATED FILE - DO NOT EDIT DIRECTLY.
// Edit the markdown files in src/docs/ and compile using npm run compile-docs.

export const DOCS_CATALOG: Record<string, string> = {
  "calculus": `# Numerical Integration (ODEs & Calculus)

\`Integral(expr, var, lower, upper)\` integrates \`expr\` with respect to \`var\` from \`lower\` to \`upper\`. Use it both for plain definite integrals and — via the self-reference trick below — for scalar first-order ODEs.

## Definite integration
\`\`\`
{ Integrates 3*x^2 from 0 to 1 -> 1.0 }
area = Integral(3 * x^2, x, 0, 1)
\`\`\`

## The ODE feedback pattern (scalar, first-order)
When \`expr\` contains the result variable itself, frees detects the self-reference and integrates the corresponding initial-value ODE **starting from 0 at the lower limit**. Because the integral always starts at 0, you integrate the *change* and rebuild the quantity of interest:

\`\`\`run
{ Tank draining: dV/dt = -C*sqrt(V), V(0) = V0 }
V0 = 1.0
C = 0.02
{ integrate the DROP from 0..60 s, then rebuild V }
drop = Integral(-C * sqrt(V0 - drop), t, 0, 60)
V   = V0 - drop          { water volume at t = 60 s }
\`\`\`

> **When to use this vs. \`DYNAMIC\`:** \`Integral()\` handles a single first-order ODE. For coupled, multi-state, stiff, or event-driven systems, use the \`DYNAMIC\` block on the next page instead.

[Related: dynamic-ode, optimization, variables]`,
  "dynamic-ode": `# Transient / ODE Systems (DYNAMIC)

The \`DYNAMIC ... END\` block integrates coupled, multi-state ODE systems. A variable becomes a **state** the moment \`der(X)\` appears; each state needs one derivative equation and one initial condition. Algebraic auxiliaries (any equation without a \`der\`) are recomputed every step and become extra columns you can plot.

\`\`\`
DYNAMIC name (method = solver, t = t0 .. tf, points = n_samples)
  der(state) = rate_equation
  state(0)   = initial_value
  auxiliary  = algebraic_calc      { an extra output column }
  EVENT name: condition -> stop | record
  EVENT name: g1 = g2 | rising -> set state = expr
END
\`\`\`

An \`EVENT\` watches the zero crossing of its condition (\`| rising\` / \`| falling\` filters the direction). **\`stop\`** ends the run at the crossing; **\`record\`** logs it and continues; **\`set state = expr\`** reassigns a state at the crossing and restarts integration from the modified value — a *discrete* switch. That is what makes true hysteresis possible: a thermostat latch (\`der(q) = 0\`, one event setting \`q = 1\` at the low threshold, another setting \`q = 0\` at the high one) shows two distinct switching temperatures, which no smooth relay can. Give set events an explicit direction so the reassignment can't immediately retrigger itself.

[Diagram: GuessConvergence]

## Choosing a solver
| Need | Method | Notes |
| --- | --- | --- |
| General, non-stiff | \`ode45\` (default) | Dormand–Prince 5(4) adaptive. Start here. |
| Mildly stiff / cheaper | \`ode23\` | Bogacki–Shampine 3(2) adaptive. |
| Fixed-step teaching | \`ode1\`–\`ode5\` | Euler, Heun, RK3, RK4, Dormand–Prince. Use many points. |
| Stiff | \`ode23s\` / \`ode15s\` | Implicit Rosenbrock / BDF. Use when \`ode45\` needs tiny steps or stalls. |

Stiffness shows up when rates differ by orders of magnitude (e.g. fast chemistry alongside slow dynamics). If \`ode45\` runs slowly or the trajectory looks jagged, switch to \`ode15s\`.

## Trajectory accessors
Query columns of the compiled ODE Table from your analytic equations:
- **\`FinalValue('col')\`** — last value of column \`col\`.
- **\`MaxValue('col')\` / \`MinValue('col')\`** — peak / minimum.
- **\`TimeAt('col', val)\`** — time when \`col\` crosses \`val\`.
- **\`ODEValue('col', t)\`** — value interpolated at time \`t\`.

These let an ODE result feed back into the analytic solve — e.g. close a sizing loop with \`MaxValue('h') = h_target\`.

## Coupled two-state example
\`\`\`run
m = 1.0; k = 20.0; c = 0.5
DYNAMIC mass_spring (method = ode45, t = 0 .. 20, points = 400)
  der(x) = v
  der(v) = -(c/m) * v - (k/m) * x
  energy = 0.5*m*v^2 + 0.5*k*x^2      { auxiliary column, decays }
  x(0) = 1.0
  v(0) = 0.0
END

final_displacement = FinalValue('x')   { read back into the analytic solve }
\`\`\`

> **Name clash tip:** the time variable and a state named \`T\` are case-insensitively the same. Name the block's time axis \`time\` (or rename the state) to avoid the collision.

[Related: calculus, plot-code, symbolic-cas]`,
  "optimization": `# Optimization & Parametric Sweeps

## Parametric sweeps
A \`PARAMETRIC\` block drives one or more variables across a range. Variables listed in the header with a range are **driven** (overridden each run); the rest are **computed** outputs. Open the **Tables** tab and click **Solve Table** (not the main Solve) to fill it in.

\`\`\`
PARAMETRIC sweep_name(var1, var2, ...)
  var1 = start : step : end | Linear
END
\`\`\`

### Sweep example
\`\`\`
v0 = 50
g = 9.81
range_m = v0^2 * sin(2 * theta_deg * pi# / 180) / g

PARAMETRIC trajectory(theta_deg, range_m)
  theta_deg = 15 : 5 : 75 | Linear
END
\`\`\`
Use the \`| Log\` suffix instead of \`| Linear\` for logarithmic spacing (handy for Bode frequency sweeps). Whole-table aggregates like \`TableAvg('range_m')\` or \`IntegralValue('P','t')\` are computed once and are identical in every row.

## Single-objective optimization
**Tools → Minimize / Maximize** (or the sidebar optimization panel) finds the value of one decision variable that minimizes or maximizes an objective, subject to your equation system. Set bounds on the decision variable in Variable Info (\`Ctrl + I\`) to keep the search in a physical region.

## Multi-objective optimization (Pareto front)
When objectives conflict (minimise mass *and* maximise efficiency, say) there is no single optimum — only a **Pareto front** of non-dominated trade-offs. frees traces it with **NSGA-II**: supply two or more objectives (each flagged minimise or maximise) plus the decision variables and their bounds. Each candidate solves the equation system with the decisions fixed; the result is a list of \`(decisions, objectives)\` points where no objective improves without worsening another. Plot one objective against the other to see the trade-off curve.

[Related: table-accessors, variables, plot-code]`,
  "debugging": `# Debugging a Solve

When a model won't solve, the cause is almost always **structural** (the equation set) or a **guess** — not a bug in the solver. Work through it methodically instead of editing equations at random.

## Build incrementally
The fastest way to localize a failure is to *not* type the whole model and hit Solve. Enter a few equations, solve, then add the next group. This pins any new syntax or convergence error to the lines you just added.

- Press **F9** to *solve the selected block only*, ignoring the rest of the document — ideal for isolating one subsystem.
- Press **F4 (Check)** after each addition so the degrees of freedom stay balanced as you grow the model.

## Read the residuals and blocking order
frees groups the equations into strongly-connected **blocks** (Tarjan) and solves them in order. When a solve fails, the error message names *which block* stalled and its **residual**, and the **Diagnostics** section at the top of the Variable Explorer opens with the full picture: every equation's residual — the difference between its two sides — evaluated at the exact point the solver gave up, sorted by magnitude, with the failing block highlighted. The block that fails to converge, and the equation with the largest residual, is where to look first.

## Seed a guess for nonlinear blocks
Newton iterates from the guess in **Variable Info** (\`Ctrl + I\`); a guess near the expected magnitude is often the difference between converging and diverging. For a tightly coupled nonlinear block (radiation, simultaneous property inversions), bootstrap it with a **temporary equation**:

1. Temporarily replace the hard constraint with a rough explicit estimate, e.g. \`T = (T_hot + T_cold) / 2\`.
2. Solve — every variable now gets a physically sensible value.
3. Copy those values into the guesses (Variable Info), restore the real equation, and solve again from the good starting point.

## Common stalls
- **Singular Jacobian** — two equations are effectively the same, so the system is rank-deficient. A subtle version: two property calls from the *same* independent pair add no new information — \`h = Enthalpy(Water, T=T, P=P)\` together with \`T = Temperature(Water, h=h, P=P)\` just restate one relationship and stall the solver. Supply a genuinely independent equation instead.
- **Max iterations / diverged** — usually a guess or bound problem. Set a guess near the expected order of magnitude and bound the variable to its physical range (e.g. \`T ≥ 0\`, \`0 ≤ x ≤ 1\`).
- **DoF ≠ 0** — too few or too many equations; **F4** reports the imbalance before you solve. An accidental duplicate (the same equation written two ways) silently over-constrains the system.
- **Two-phase property lookups** — inside the vapor dome, temperature and pressure are *not* independent. Identify a saturated state by quality \`x\` with either \`T\` or \`P\`, never by \`T\` and \`P\` together.

[Related: variables, gs-units-check, api]`,
  "errors": `# Errors & Diagnostics Index

Every message the checker and solver can raise, with its cause and fix. The red status pill in the app links here — find your message below. Two habits solve most of these before they happen: **F4 before F2**, and **build incrementally** (see *Debugging a Solve*).

## Syntax error on line N

The parser could not read that line. The usual culprits:

- **Implicit multiplication** — write \`2 * x\`, never \`2x\`.
- **A stray \`==\`** — frees has only the single \`=\` equality.
- **REPL-only syntax in the editor** — range literals like \`[1 : 0.5 : 10]\` and the symbolic CAS calls work only in the REPL; in a document build ranges with \`x[1:n] = linspace(a, b, n)\`.
- **Unbalanced brackets or block keywords** — every \`FUNCTION\`/\`TABLE\`/\`DYNAMIC\`/\`COMPONENT\`/\`PLOT\` needs its \`END\`.

## Degrees of freedom ≠ 0 (equations ≠ unknowns)

The system is under- or over-determined; the Check message reports both counts.

- **Too few equations** — a variable is mentioned but never constrained. Typos create this silently: \`T_evap\` vs \`T_evp\` are *two* variables.
- **Too many equations** — the same physics written twice. A classic: two property calls that restate one relationship (\`h = Enthalpy(Water, T=T, P=P)\` **and** \`T = Temperature(Water, h=h, P=P)\`), or re-equating pressures a component junction already equalizes.
- Case-insensitivity can *merge* names you meant to be distinct: \`t\` and \`T\` are the same variable.

## Singular Jacobian

Newton's step could not be computed — the equation block is rank-deficient at the current point.

- **Dependent equations**: two equations carry the same information (see the duplicate-physics cases above).
- **A guess on a flat spot**: the derivative vanishes at the guess (e.g. \`x\` starting exactly at a function's extremum). Nudge the guess in Variable Info (\`Ctrl + I\`).
- In component networks: a **re-equated mixer pressure** or a **loop with no pinned pressure level** — see *Troubleshooting Networks*.

## Newton iteration stalled / Max iterations

The solver ran but did not converge; the diagnostic names the failing **block** and its largest **residual**.

1. Set a **guess** near the expected magnitude for the block's variables (\`Ctrl + I\`), and **bounds** to keep the iteration physical (\`T ≥ 0\`, \`0 ≤ x ≤ 1\`).
2. If the block contains property calls, check the *property error* appended to the message — the iteration usually walked a state out of the fluid's valid range (next entry).
3. Bootstrap a stubborn block with a temporary explicit estimate, solve, copy values into guesses, restore the real equation (*Debugging a Solve* shows the pattern).

## Real-fluid range / property errors ("must be in range …")

A property was evaluated outside the fluid's validity envelope — almost always a symptom, not the disease: an unseeded unknown wandered there during iteration.

- Give the offending variable a physically sensible guess and bounds.
- **Inside the vapor dome, \`T\` and \`P\` are not independent** — identify a saturated state by quality \`x\` with \`T\` *or* \`P\`, never both; use \`P_sat(fluid, T=…)\` for the saturation line.
- For humid air, every \`AirH2O\` query needs **three** coordinates including total pressure \`P\`.

## Unit warnings ("dimensionally inconsistent …")

Warnings never block a solve, but they are usually pointing at a real slip: a missing \`[unit]\` annotation on an input, a formula mixing annotated and raw numbers, or a \`TABLE\`/\`FUNCTION\` without declared argument/output units. See *Units & Consistency*. Only trust results after the warning list is empty or understood.

## Component network errors

\`connect\` domain mismatches, port-count errors, mismatched fluid families, missing \`PARAM\`/\`REQUIRE\` values, and the shared-stream limit are all **deliberate hard errors** with their own page — see *Troubleshooting Networks*.

## Job FAILED without a diagnostic / stuck in PENDING

These come from the compute tier, not your model:

- **Stuck PENDING** — no compute workers are consuming the queue. Check \`GET /api/health\`: the \`compute\` entry counts live workers (see *Health & Scaling*).
- **FAILED after a worker crash** — a redelivered task is dropped by design (the poison-message guard) so one pathological job can't crash-loop the tier. Re-submit once; if it fails again, the model itself is killing the worker — reduce it and report.
- **429 Too many requests** — the API rate-limits bursts; wait a moment and retry.

[Related: debugging, comp-troubleshooting, api]`,
  "api": `# Solver Reference & API

Knowing the execution pipeline helps you read convergence diagnostics and diagnose singular systems.

## Compilation & execution pipeline
1. **Lex/parse (ANTLR4)** — tokenizes variables, symbols, constants, and \`[unit]\` annotations.
2. **AST construction** — inlines functions/modules, expands array indices and matrix slices, and converts every unit to its SI base value.
3. **Dependency analysis (Tarjan SCC)** — builds the variable↔equation graph and groups coupled variables into minimal strongly-connected blocks.
4. **Newton–Raphson solve** — solves each block in topological order using finite-difference Jacobians and backtracking line search. Guesses (Variable Info) seed the iteration; bounds keep it physical.
5. **DYNAMIC pass** — integrates ODE blocks using the solved analytic variables as parameters. Accessor values feed back and the system re-solves until it converges globally.

## Reading convergence output
- **"Singular Jacobian"** — two equations are effectively dependent, or a guess landed on a flat region. Check for duplicate/redundant equations and adjust guesses.
- **"Max iterations"** — the solver didn't converge. Almost always a guess or bound problem; try a guess closer to the expected magnitude.
- **DoF ≠ 0** — too few or too many equations. F4 (Check) reports the imbalance before you solve.

[Related: variables, gs-units-check, uncertainty]`,
  "arch-async": `# How a Solve Runs

frees runs **entirely in your browser tab** via WebAssembly, with a background Web Worker compute model. Understanding the execution pipeline explains what you see in the UI — why the Solve button is gated on a green Check, how the UI remains responsive during heavy numerical computations, and how long solves can be cancelled instantly.

## The path of one solve

1. **Editor → Web Worker.** Pressing Solve (F2) packages your document text, stop criteria, and variable info into a message dispatched to a dedicated background Web Worker (\`worker.ts\`).
2. **Validate & Prepare.** The Web Worker parses the document using the Rust-based parser. If syntax errors or structural mismatches exist, a diagnostic envelope returns immediately without blocking.
3. **Compute.** The Web Worker executes the full engine pipeline in WebAssembly:
   - Expand component networks, matrix literals, and CALL procedures.
   - Run dimensional unit consistency analysis.
   - Perform Dulmage–Mendelsohn / Tarjan decomposition into lower triangular blocks.
   - Solve each block using Newton–Raphson, with interval scanning and multi-start root-finding when "Find all solutions" is enabled.
   - Integrate dynamic systems (\`DYNAMIC ... END\`) and differential-algebraic equations via IDA / Runge–Kutta when present.
   - Evaluate thermodynamic properties through the integrated \`rustprop\` backend.
4. **Publish & Render.** The Web Worker posts the structured solution payload back to the main thread. The React frontend updates the Variable Explorer, Diagrams, Plots, and Tables workbooks with zero network round-trip latency.

## Why in-browser Web Workers?

- **Zero backend infrastructure.** There are no remote API servers, no message brokers (RabbitMQ), and no database caches (Redis). Your models and calculations stay private on your machine.
- **Offline capability.** Once the static page and WebAssembly module are loaded, frees functions completely offline without any internet connection.
- **Responsive UI.** Heavy calculations and iterative sweeps run off the main thread in a Web Worker, ensuring smooth 60fps rendering and seamless interaction.
- **Instant cancellation.** If a simulation or stiff ODE takes too long, clicking Stop immediately terminates the Web Worker and resets the compute state safely.

## Check before Solve

Pressing Check (F4) runs validation without solving: syntax parsing, equation-variable matching, degree-of-freedom checking, and dimensional unit verification. Because it requires no numerical iterations, it completes in milliseconds and provides immediate diagnostic feedback in the editor gutter.

[Related: arch-api, deploy-docker]`,
  "arch-api": `# The Web Worker & CLI Interface

The frees engine exposes a clean, unified JSON boundary between the Rust core and user interfaces. Whether running inside the browser Web Worker or from the terminal command line, the same document produces identical results.

## Headless CLI (\`frees-cli\`)

For CI/CD pipelines, automated validation, and batch calculations, frees provides a standalone headless binary: \`frees-cli\`.

\`\`\`bash
# Solve a document and print solved variables as JSON
frees-cli solve model.frees

# Check syntax and solvability without solving
frees-cli check model.frees

# Solve with full browser request options (guesses, bounds, multiple solutions)
frees-cli solve --request '{"findAllSolutions": true}' model.frees

# Pipe document from stdin
cat model.frees | frees-cli solve
\`\`\`

## JSON Request & Response Envelope

The solver communicates via structured JSON envelopes:

- **Inputs**: Document source text, \`stopCriteria\` (tolerances, iteration limits), \`variableInfo\` (guesses, bounds, display units), and optional \`functionTables\`.
- **Outputs**: \`variables\` (display-converted values with units), \`blocks\` (Tarjan decomposition hierarchy), \`residuals\` (equation-level convergence), \`stats\` (iterations, elapsed time), and \`solutions\` (multiple solutions when requested).

Because the headless CLI uses the exact same \`frees_core\` engine as the WebAssembly module, scripting workflows perfectly mirror browser behavior.

[Related: arch-async, deploy-docker]`,
  "deploy-docker": `# Local Development

frees is written in Rust and TypeScript/React. Developing locally requires only standard Rust and Node.js toolchains:

## Prerequisites

- **Rust toolchain**: Managed via \`rustup\` with the \`wasm32-unknown-unknown\` target.
- **Node.js**: **Node 22 is required** (Node 20 lacks WebIDL uncloneable features required by Vitest/JSDOM).
- **wasm-pack**: Compiles Rust crates to WebAssembly.

## Building and Running

\`\`\`bash
# 1. Compile the Rust WebAssembly engine
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg

# 2. Install web dependencies and start the Vite development server
cd web
npm install
npm run dev

# 3. Run test suites
cargo test --workspace -- --skip golden_corpus_parity
npm test
\`\`\`

The web application is accessible at \`http://localhost:5173\`.

[Related: deploy-railway, arch-async]`,
  "deploy-railway": `# Browser Deployment & Static Hosting

Because frees runs entirely client-side, deploying frees requires **no backend containers, databases, or microservices**.

## Static Web Hosting

The entire application compiles into static HTML, JavaScript, CSS, and \`.wasm\` files. It can be hosted on any static web server or CDN platform:

- **GitHub Pages**
- **Cloudflare Pages**
- **Vercel**
- **Netlify**
- **Nginx / Caddy / Apache**

## Production Build

\`\`\`bash
# Build WebAssembly package
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg

# Build static production assets
cd web
npm run build
\`\`\`

The output in \`web/dist/\` contains all assets needed for deployment. Serve the directory with any web server configured with standard MIME types (specifically \`application/wasm\` for \`.wasm\` files).

[Related: deploy-docker, deploy-health]`,
  "deploy-health": `# Worker Lifecycle & Deadlines

Because frees executes in the user's browser, execution boundaries and resource safety are handled cooperatively:

## Timeouts and Deadlines

- **Cooperative Deadlines**: For parametric sweeps, optimization loops, and Monte Carlo runs, frees enforces cooperative deadlines (default 120s budget) checked between iterations to prevent runaway computations.
- **Worker Termination**: If an equation system causes an uncooperative infinite loop, clicking the UI's Stop button immediately terminates the Web Worker thread and spawns a fresh worker instance.

## Memory and Bundle Budgets

- **WASM Memory**: WebAssembly linear memory grows dynamically as needed for large systems and matrices, bounded by browser tab memory limits.
- **Binary Size**: The release WebAssembly binary is strictly budgeted and monitored in CI (~3.1 MB uncompressed / ~1.2 MB gzipped), including complete fluid property tables and 295 physical components.

[Related: arch-async, deploy-railway]`,
  "comp-first-network": `# Your First Component Network

frees has a library of **312 components** — reusable, parameterized blocks of physics (pumps, pipes, heat exchangers, resistors, gears, cooling coils …) with typed **ports**. You instantiate them, wire the ports together, and frees expands the network into ordinary scalar equations solved by the same Newton/Tarjan pipeline as everything else. There is no separate "simulation mode": components and plain equations mix freely in one document.

## Default authoring style

Write **named parameters**, **explicit \`connect\`**, and **single-quoted strings**. The wizard emits this shape; positional stream names are a compact alternative taught after the first network.

\`\`\`
Source SUP(fluid$='Water', mdot=2 [kg/s], P=300000 [Pa], T=298 [K])
Pipe   LINE(fluid$='Water', L=50 [m], D=0.05 [m], rough=0.0001)
connect(SUP.out, LINE.in)
\`\`\`

\`"Water"\` is a comment, not a fluid name. Prefer \`'Water'\`.

## Water through a pipe

\`\`\`run
{ Supply -> pipe -> return: what pressure is lost to friction? }
Source  SUP(fluid$='Water', mdot=2 [kg/s], P=300000 [Pa], T=298 [K])
Pipe    LINE(fluid$='Water', L=50 [m], D=0.05 [m], rough=0.0001)
Sink    RET()

connect(SUP.out, LINE.in)
connect(LINE.out, RET.in)

dP = SUP.out.P - RET.in.P     { probe: frictional pressure drop, Pa }
\`\`\`

Solve (F2) and read \`dP\` in the Solution panel. Three things happened:

1. **Instantiation** — \`Pipe LINE(...)\` stamped a copy of the \`Pipe\` template, filling in its parameters. Every parameter is named (\`L=50\`), and unit annotations work exactly as in plain equations.
2. **Connection** — each \`connect\` statement tied two ports into a node: pressures equalize, mass is conserved, enthalpy is carried through.
3. **Probing** — dotted **port members** (\`SUP.out.P\`, \`LINE.in.mdot\`) are ordinary solver variables. You can read them, plot them, or pin them with a boundary condition like \`RET.in.P = 100000 [Pa]\`.

## No causality, by design

Notice what you did *not* write: no "input" or "output" designation, no calculation order. The network is **acausal** — the \`Pipe\` doesn't know whether it is computing a pressure drop from a flow or a flow from a pressure drop. Fix any consistent set of boundary values and the solver finds the rest, exactly like swapping the unknown in an ordinary frees equation. That is the same declarative idea you met in *Your First Solve*, lifted to whole systems.

## Named outputs

Many components compute results you'll want directly — a compressor's power, an exchanger's duty. These are exposed as **named outputs** on the instance:

\`\`\`
Compressor CMP(fluid$=R134a, eta=0.72, model$=isentropic)
...
W_comp = CMP.W        { compressor power, W }
\`\`\`

Every component's ports, parameters, equations, and outputs are documented on its page in the **Reference** — see the A–Z index, or browse by library on *The Component Library* page.

[Related: comp-connections, comp-library, gs-declarative]`,
  "comp-connections": `# Connections & Junctions

There are two ways to wire a network. Both expand to exactly the same equations — pick whichever reads better.

## Style 1 — connect statements

A \`connect\` statement ties the listed ports into one **node**. It takes any number of endpoints, so branching is native:

\`\`\`
connect(PUMP.out, RAD.in, BYPASS.in)   { flow splits after the pump }
\`\`\`

At a node, frees emits the **junction rules** for the ports' domain (see *Domains & Fluid Families*): the *across* variables equalize (e.g. one pressure), and the *through* variables sum to zero (e.g. Σṁ = 0 — what flows in flows out). For fluid streams the specific enthalpy \`h\` rides along convectively: equal at a split or pass-through. Merging streams at different states needs an explicit mixer component (\`Mixer\`, \`LiquidMixer\`, \`MixingBox\`, …), which flow-weights the enthalpy properly.

Loops close the same way — connecting the last component back to the first is legal and is how closed circuits (refrigeration loops, coolant circuits) are built.

## Mixing and closing a loop

A \`connect\` of three ports is a legal **branch** (one pressure, mass conserved), but it **does not mix enthalpy**. Two streams at different states that join need an explicit mixer (\`Mixer\`, \`LiquidMixer\`, \`MixingBox\`, …):

\`\`\`
connect(SRC.out, MIX.in1)
connect(RECIRC.out, MIX.in2)
connect(MIX.out, LINE.in)
\`\`\`

A **closed loop** is \`connect\` back through the load — legal, and how a refrigeration or coolant circuit is written:

\`\`\`
connect(PUMP.out, HX.in)
connect(HX.out, PUMP.in)
\`\`\`

See *Connections & Junctions* for the two-port limit on shared stream names, and the mixer pages in the Reference for parameters.

## Style 2 — shared stream names (advanced)

For simple series chains there is a terser form: bind ports **positionally** to named streams. Two instances that name the same stream are connected. Learn \`connect\` first; use this when a two-port chain is easier to read than two statements.

\`\`\`
Source SUP(s1, fluid$=Water, mdot=2, P=300000, T=298)
Pipe   LINE(s1, s2, fluid$=Water, L=50, D=0.05, rough=0.0001)
Sink   RET(s2)
\`\`\`

Leading positional arguments bind the ports in the component's declared order (\`Pipe\` declares \`in, out\`, so \`s1\` is its inlet and \`s2\` its outlet); the trailing \`name=value\` arguments set parameters. Stream members are addressed directly: \`s2.P\`, \`s2.h\`, \`s2.mdot\`.

A stream name may join at most **two** ports — a third is a hard error, because a silent three-way tie is almost always a mistake. Use a \`connect\` statement when you need branching.

## Boundary conditions

A network needs enough pinned values to close its degrees of freedom, just like any equation system. Pin port members with plain equations:

\`\`\`
RET.in.P = 100000 [Pa]      { fix the return pressure }
\`\`\`

Source/sink components (\`Source\`, \`PressureSource\`, \`MoistAirSource\`, \`VoltageSource\`, \`ThermalSource\`, …) are pre-packaged boundary conditions; a bare equation on a port member does the same job when no component fits.

[Related: comp-first-network, comp-domains, comp-schematic, comp-troubleshooting]`,
  "comp-schematic": `# Reading the Schematic

The **Schematic** window draws the network your document describes — open it from the left rail, or from the command palette (Ctrl+K → "Schematic"). It is generated from the text on every **Check**, so it is never out of date and there is nothing to lay out by hand. Everything below is derived; the drawing is a view of the model, not a second copy of it.

It is the fastest way to answer "did I wire that the way I meant to?", because a mis-wired network usually *looks* wrong long before it solves wrong.

## Circuits are drawn apart

Each **working fluid gets its own colour and its own framed band.** This matters more than it sounds: the bond-graph domain calls a coolant loop and a refrigerant loop the same thing — both are \`fluid\` — so a drawing coloured by domain paints two independent circuits identically and they read as one tangle. frees separates them by *connector type* and *fluid*, so an EG50 coolant loop and an R1234yf refrigerant loop land in bands labelled \`EG50 · LIQUID\` and \`R1234YF · TWO-PHASE\`.

| Line | Meaning |
|---|---|
| blue | liquid (coolant, water/glycol) |
| violet | two-phase (refrigerant) |
| teal | generic thermofluid / steam |
| orange | pneumatic (\`gas\`) |
| amber | hydraulic (\`oil\`) |
| pale cyan | humid air (\`moistair\`) |
| red | heat |
| yellow | electrical |
| lime | mechanical (rotational and translational) |
| dashed cyan | signal — a causal control value, not a physical flow |

Two different fluids on the same connector type (two coolant loops, say) take different shades of the same hue, so they stay distinguishable while still reading as "both coolant". The legend above the canvas names every line in the drawing.

A **coupling band** — heat, signal, mechanical — is placed next to the circuit it links to most. In the common shape of two loops bridged by a heat exchanger, that puts the thermal band *between* them, so the couplings are short instead of crossing an unrelated circuit.

## Flow sets the left-to-right order

A component network is acausal — you may write the equations in any order — but the *port names* are not. A \`connect\` from an \`out\` port to an \`in\` port says the first feeds the second, so each circuit is laid out source → … → sink. A closed loop is drawn as a chain with its closing edge running back, which is how it would be drawn by hand.

## Every block says what it is

Blocks carry the symbol of their function, so the topology is legible before you read a single label:

| Symbol | Component kind |
|---|---|
| circle with impeller | pump, fan, blower |
| converging trapezoid | compressor |
| diverging trapezoid | turbine, expander |
| bow-tie | valve, orifice, throttle |
| crossed box | heat exchanger, coil, radiator |
| capacitor plates | storage — thermal mass, tank, accumulator, battery |
| diamond | junction — mixer, splitter, manifold |
| dashed border + chevron | **boundary condition** — a source or a sink |
| ladder | ground (fixed potential) |

Boundaries are drawn dashed on purpose: they are where your model *stops*, and seeing them makes an accidentally open circuit obvious.

## The numbers

After a solve, each block prints its most telling result — a heat exchanger's duty, a machine's power, otherwise the flow through it. **Hover** a block for the rest:

- **Ports** — the state at each wired port (\`P\`, \`T\`, \`ṁ\`, \`h\`, and the domain's own members: \`Q̇\` for heat, \`V\`/\`I\` for electrical, and so on), in SI with units.
- **Results** — the block's named outputs, the same \`CHLR.Q\` / \`CMP.W\` you can reference in equations.
- **Parameters** — what the block was built with, **and where the value came from**. A document that sizes a heat exchanger from correlations and geometry injects the answer as a parameter, so the card shows \`ua  576.79 W/K  (UA_chl_r)\` — the number *and* the variable, so you can trace it back to the correlation that produced it.

Click a block to pin the card and jump to the line that declares it in the editor.

## Moving things around

| Action | Result |
|---|---|
| drag a block | move it; wires re-route live |
| drag the background | pan |
| Ctrl/⌘ + scroll | zoom |
| **Fit to window** | frame the whole network |
| **Reset layout** | discard your arrangement, return to the automatic one |
| **Export SVG** | the whole drawing as a vector file, ready to paste into a report |

The automatic layout keeps the network framed until you first zoom, pan or drag — after that it leaves the view alone.

**Your arrangement is saved with the project.** Positions are stored as offsets from the automatic layout rather than as fixed coordinates, so when you add a component upstream everything shifts with it instead of being stranded. Positions for components you delete are dropped; positions for components that survive an edit are kept.

## Wiring by clicking

Click a **port dot** on one block, then a port dot on another, and frees appends the matching \`connect\` statement to your document. Port dots are coloured by the line they carry, so an inlet already on the coolant circuit shows blue — a quick check that you are about to join what you think you are.

## What it will not do

- It draws the network the **Check** understood. If the document has errors, the canvas says so rather than showing a stale or half-drawn network.
- Wires are routed orthogonally but do not steer around blocks, so a dense network will have lines crossing boxes.
- Grouping is automatic, by circuit. There is no manual "group these into a subsystem" — for real hierarchy, use a hierarchical \`COMPONENT\` (see *Writing Your Own Component*).

[Related: comp-connections, comp-domains, comp-cycle-plots, comp-troubleshooting]`,
  "comp-domains": `# Domains & Fluid Families

Every port belongs to a **domain** — the pair of *across* / *through* variables it carries and the junction rule a node enforces:

| Domain | Across (equal at a node) | Through (sums to zero) | Carried along |
| --- | --- | --- | --- |
| **Fluid** | pressure \`P\` | mass flow \`mdot\` | specific enthalpy \`h\` (convective) |
| **Heat** | temperature \`T\` | heat flow \`Qdot\` | — |
| **Electrical** | voltage \`V\` | current \`I\` | — |
| **Mechanical (rotational)** | speed \`omega\` | torque \`tau\` | — |
| **Mechanical (translational)** | velocity \`v\` | force \`F\` | — |
| **Signal** | value \`sig\` | *(none — causal)* | — |

The domain is inferred from the members a port carries — you never register it. A port carrying \`(P, mdot, h)\` is fluid; \`(T, Qdot)\` is heat, and so on.

## The signal domain: causal command wires

The acausal domains conserve something at a node; the **signal** domain deliberately doesn't. A port referenced as \`port.sig\` carries a bare value with *no* flow member — a \`connect\` node simply equates it everywhere, so **one writer broadcasts to any number of readers**, exactly like a control-diagram wire. That is what command inputs, setpoints, and measurements are:

\`\`\`
SigSine  CMD(amp=0.5, freq=0.2, phase=0, bias=0.5)   { 0..1 command wave }
EXVCmd   VALVE(fluid$=R134a, CdA_max=2e-6)
connect(CMD.out, VALVE.u)                             { the wire }
\`\`\`

The library ships ~30 signal blocks: **sources** (\`SigConstant\`, \`SigStep\`, \`SigRamp\`, \`SigSine\`, \`SigPulse\`, \`SigTable\` drive cycles), **math** (\`SigSum\`, \`SigGain\`, \`SigProduct\`, …), **nonlinearities** (\`SigSaturation\`, \`SigDeadband\`, \`SigRelay\`, \`SigRateLimiter\`), **dynamics** (\`SigIntegrator\`, \`SigFirstOrder\`, \`SigSecondOrder\`, \`SigLeadLag\`), the \`SigPID\` controller, and **probes** (\`SigThermalProbe\`, \`SigSpeedProbe\`, \`SigVelProbe\`) that read a physical port *into* a signal — the sanctioned way to close a loop around a plant. Signal-to-physical wiring is rejected by the same strict single-domain guard as every other mismatch: commandable actuators expose a dedicated signal port instead (the \`u\` port on \`EXVCmd\` above).

## One node, one domain — enforced

A \`connect\` node must be a single domain. Wiring a heat port to an electrical port is a **hard parse error**, not a warning — frees refuses to build a network that would silently solve the wrong physics.

Crossing domains is what **transducer components** are for: they carry one port *per* domain and the coupling physics inside. \`HeatingResistor\` has electrical terminals and a heat port (its I²R loss); \`LiquidWallHX\` has fluid ports and a \`wall\` heat port; a motor couples electrical to rotational. The pressure-cooker example in the Examples library chains electrical → thermal → two-phase fluid through exactly such components, in one solve.

## Fluid families

Several fluid-like domains share the same \`(P, mdot, h)\` bond but must never be cross-wired — a pneumatic line makes no sense feeding an oil line. A reserved string parameter, \`domain$\`, tags each fluid-family connector:

| \`domain$\` | Family | Typical components |
| --- | --- | --- |
| \`fluid\` *(default)* | General thermofluid | \`Source\`, \`Pipe\`, \`Compressor\`, \`HeatExchanger\` |
| \`liquid\` | Incompressible liquid loops | \`LiquidPump\`, \`LiquidWallHX\`, \`LiquidMixer\` |
| \`twophase\` | Evaporating / condensing refrigerant | \`TwoPhaseCompressor\`, \`TwoPhaseEvaporatorUA\` |
| \`gas\` | Pneumatics (ISO 6358) | \`PneumaticOrifice\`, \`PneumaticVolume\` |
| \`oil\` | Oil hydraulics | \`HydraulicPump\`, \`ReliefValve\` |
| \`moistair\` | Humid air (HVAC) | \`MoistAirSource\`, \`CoolingCoil\`, \`MixingBox\` |

Connecting mismatched families is, again, a hard error. The built-ins carry the right tag already; your own components opt in with \`PARAM domain$ = gas\` (see *Writing Your Own Component*).

## Humid air: the W rider

The \`moistair\` family conserves **two** masses. Its basis is \`(P, mdot_da, h, W)\`: flow is on a *dry-air* basis (Σṁ_da = 0), and the humidity ratio \`W\` rides along as a second conserved species — equal across a pass-through connection, flow-weighted only in an explicit \`MixingBox\`. That rider is what makes a cooling coil able to condense water out of the stream while dry air is conserved. The gas-mixture components use the same pattern for species fractions (\`.y\`).

[Related: comp-connections, comp-library, humidair]`,
  "comp-library": `# The Component Library

The standard library ships 312 components across thirteen domain libraries. This page is a map, not a catalog — every component's authoritative page (ports, parameters, variants, governing equations) lives in the **Reference**; find it by name in the A–Z index, or browse it from the Component Wizard. The Component Wizard and editor completion are generated from this port's parsed library, so a standalone checkout stays in agreement with the engine.

| Library | What's in it |
| --- | --- |
| **signal** | Causal control wires: sources (\`SigConstant\`/\`SigStep\`/\`SigRamp\`/\`SigSine\`/\`SigPulse\`, \`SigTable\` drive cycles), block-diagram math, saturation/deadband/relay/rate limits, transfer-function dynamics (\`SigFirstOrder\`, \`SigSecondOrder\`, \`SigLeadLag\`), \`SigPID\`, map lookups, and physical→signal probes |
| **fluid** | General thermofluid plus gas/aero breadth: \`Source\`/\`Sink\`, \`Pipe\`, \`Valve\`, \`Nozzle\`, \`Pump\`, \`Fan\`, \`Compressor\`, \`Turbine\`, \`HeatExchanger\`, \`Mixer\`/\`Splitter\`, map-driven turbomachines, ducts, regenerator, combustor, ISA atmosphere, propeller |
| **liquid** | Incompressible coolant / TMS loops: \`LiquidSource\`, \`LiquidPump\` (+ pump map), \`LiquidOrifice\`, \`LiquidWallHX\`, \`LiquidMixer\`, three-way valve, tank, thermostat, expansion tank |
| **twophase** | Evaporating/condensing refrigerant circuits: boundaries, \`TwoPhaseCompressor\`, moving-boundary heat exchangers, \`TwoPhasePipe\` (Lockhart–Martinelli), \`TXVSuperheat\`, \`ThreeZoneHX\`, charge/receiver volumes, \`BoilingVessel\`, VCC devices |
| **ac** | Application composites built on the two-phase set: \`Chiller\`, \`AirCoil\`, \`Radiator\`, \`HeaterCore\`, \`TXV\`, \`EXV\`/\`EXVCmd\` |
| **moistair** | Humid-air HVAC: \`MoistAirSource\`/\`MoistAirSink\`, \`CoolingCoil\` (wet coils), \`HeatingCoil\`, \`Humidifier\`, \`MixingBox\`, \`MoistAirWallHX\`, cabin zone |
| **pneumatic** | ISO 6358 compressible gas power: orifices, volumes, valves, cylinders, sources |
| **hydraulic** | Oil-hydraulic power: pumps, orifices, valves, cylinders, accumulators, \`ReliefValve\` |
| **heat** | Lumped heat transfer: \`ThermalSource\`, \`ThermalMass\`, \`Conduction\`/\`Convection\`/\`Radiation\`, \`ContactResistance\`, \`MassGen\` (self-heating mass), transient walls, PCM, Peltier, heat pipe |
| **electrical** | Circuits & electrification: \`VoltageSource\`, \`Ground\`, resistors (\`HeatingResistor\` couples to heat), \`Capacitor\`/\`Inductor\`, battery cells and packs with SOC, motor/inverter/DC-DC, PV, electrolyzer, \`FuelCellStack\` (PEMFC) |
| **mechanical** | Rotational & translational 1-D mechanics: \`Inertia\`, \`TransMass\`, springs, dampers, \`Gear\`, \`Planetary\`, \`Clutch\`, \`Friction\`, backlash, hard stops, kinematic pairs |
| **powertrain** | Vehicle-level: engines (\`MeanValueEngine\`), \`Transmission\`, torque converter, tire, vehicle body, \`GradeRoadLoad\`, drive cycles |
| **control** | Network-level sensors and controllers (e.g. \`PIThermostat\`, \`ThermalSensor\`, \`FlowSensor\`) — see the **signal** library for full block-diagram control |

Three conventions hold across the whole library:

- **No hidden defaults.** Every physical parameter must be given explicitly at instantiation — a missing one is an error, never a silent assumption.
- **Naming tells you the family.** \`Liquid*\`, \`TwoPhase*\`, \`Pneumatic*\`, \`Hydraulic*\`, \`MoistAir*\` prefixes mark the fluid family (and its \`domain$\` tag).
- **Fidelity is selectable, not duplicated.** Where one machine has several physics levels (a compressor with isentropic-η, volumetric, or map-based models), it is *one* component with a \`model$\` selector — see *Fidelity Variants*.

[Related: comp-variants, comp-first-network, ref-index]`,
  "comp-variants": `# Fidelity Variants (model$)

Real projects move through fidelity levels: a first-cut cycle needs only an isentropic efficiency; the sized design wants the volumetric model; the calibrated digital twin wants the manufacturer's map. In frees that is **one component, many models** — a \`model$\` parameter selects which physics body is expanded:

\`\`\`
{ concept study }
Compressor CMP(fluid$=R134a, eta=0.72, model$=isentropic)

{ sized design: same component, higher fidelity }
Compressor CMP(fluid$=R134a, eta=0.72, model$=volumetric,
               eta_v=0.92, disp=6.5e-5, rpm=2900)
\`\`\`

Because the component and its ports don't change, **the network around it doesn't change either** — you upgrade fidelity by editing one line, not rewiring the model.

## Per-variant required parameters

Each variant declares the parameters it needs (\`REQUIRE\`), validated only when that variant is selected. Choosing \`model$=volumetric\` without \`disp\` is an immediate, named error. Supplying \`rpm\` on an isentropic compressor is accepted for compatibility — Check reports a nonblocking advisory that the parameter is inactive, and the wizard omits inactive drafts from newly generated code. The reference page of every multi-model component lists its variants and their requirements under **Model Variants**, and the Component Wizard shows and requires exactly the parameters the selected variant needs. The default \`model$\` is the engine's declared default, not whichever variant happens to be documented first.

Variants of your own components use the \`VARIANT ... REQUIRE ... END\` construct — see *Writing Your Own Component*.

[Related: comp-authoring, comp-library, comp-wizard]`,
  "comp-authoring": `# Writing Your Own Component

When the library lacks a device — or you want your own correlation inside one — define a component in the document with \`COMPONENT ... END\`. The header parentheses declare the **ports** (in the order positional binding will use); \`PARAM\` lines declare parameters; everything else is acausal equations over port members, locals, and outputs.

\`\`\`run
COMPONENT Heater(in, out)
  PARAM fluid$, Q
  out.mdot = in.mdot
  out.P    = in.P
  out.h    = in.h + Q / in.mdot
  T_out    = Temperature(fluid$, P=out.P, h=out.h)   { named output }
END

Source SUP(fluid$=Water, mdot=0.5 [kg/s], P=200000 [Pa], T=290 [K])
Heater H1(fluid$=Water, Q=50000 [W])
Sink   RET()
connect(SUP.out, H1.in)
connect(H1.out, RET.in)

T_supply = H1.T_out          { read the named output }
\`\`\`

The rules:

- **Ports** carry whatever members your equations reference. Use \`(P, mdot, h)\` members and the port is a fluid port; use \`(T, Qdot)\` and it is a heat port — domain inference is automatic (see *Domains & Fluid Families*). A port referenced only as \`port.sig\` becomes a causal **signal** port: one writer, any readers — use one for every command input rather than pinning a component's internals from outside.
- **Parameters** — a trailing \`$\` marks a string parameter (\`fluid$\` is special: it names the stream's fluid for property calls and per-port fluid inference). \`PARAM x = value\` gives *your* component a default; the standard library deliberately never uses them.
- **Locals and outputs** — any bare name in the body is instance-private (auto-namespaced, like \`MODULE\` locals). Reading it from outside as \`inst.name\` makes it a named output.
- **Fluid family** — a component for a non-default family opts in with \`PARAM domain$ = gas\` (or \`oil\`, \`moistair\`, \`liquid\`, \`twophase\`), so the connector guard protects your lines too.
- **Composition** — a component body may instantiate other components and \`connect\` them: build a subsystem once, stamp it many times.
- **Time** — a body may reference the reserved global \`time\` (never namespaced) to build time-driven behavior; the \`DYNAMIC\` integrators pin it, and a steady document sets \`time = 0\` explicitly.
- **Keep closures C¹-smooth.** Newton differentiates everything, so a hard \`if\`/corner in a constitutive law stalls convergence. Use smooth surrogates — a \`tanh\` gate, the hinge \`0.5*(x + sqrt(x^2 + eps^2))\`, odd-symmetric flow laws — and expose the smoothing width as an \`eps\` parameter.

> **Contributing to the built-in library?** The end-to-end process (physics in the \`.frees\` domain files, golden-value fixture tests, generated reference pages) is documented in the repository at \`docs/component_authoring.md\`.

## Variants

Split fidelity levels with \`VARIANT\` blocks. Equations outside any variant are shared; each variant adds its own, and \`REQUIRE\` names the parameters it validates:

\`\`\`
COMPONENT MyFan(in, out)
  PARAM fluid$, model$ = simple
  out.mdot = in.mdot                  { shared by every variant }
  VARIANT simple
    out.P = in.P + 250
  END
  VARIANT curve REQUIRE dP0, kQ
    out.P = in.P + dP0 - kQ * in.mdot^2
  END
END
\`\`\`

\`MyFan F1(fluid$=Air, model$=curve, dP0=300, kQ=1.5e4)\` selects and validates the \`curve\` body.

[Related: comp-variants, comp-domains, functions]`,
  "comp-transient": `# Steady ↔ Transient from One Network

A component network describes physics, not a moment in time — the **same wiring** yields a steady operating point or a transient, depending on whether you wrap it in a \`DYNAMIC\` block.

## Storage components carry the states

Time enters through **storage** components: \`ThermalMass\` (heat capacity), \`MassGen\` (self-heating mass), \`Inertia\` / \`TransMass\` (mechanical), \`Capacitor\` / \`Inductor\` (electrical), \`Accumulator\` (hydraulic), battery SOC, \`BoilingVessel\` (two-phase mass + energy). Each contributes a state derivative and an initial condition (its \`T0\`, \`omega0\`, \`SOC0\`, … parameter). Solved without a \`DYNAMIC\` block, the network settles to its steady operating point; add one and the states integrate in time:

\`\`\`
{ A 4 kW battery module on a cold plate, warming its thermal mass }
MassGen     BATT(C=60000, Qgen=4000, T0=305 [K])
LiquidWallHX PLATE(fluid$=EG50, UA=800)
connect(PLATE.wall, BATT.port)
{ ... coolant loop around PLATE ... }

DYNAMIC warmup (method = ida, time = 0 .. 600, points = 601)
END
\`\`\`

An **empty** \`DYNAMIC\` body is enough — the storage components inject their \`der(...)\` equations and initial conditions automatically. The block produces an ODE Table (Tables tab) with every state and stream member as columns you can plot, and the trajectory accessors (\`FinalValue('...')\`, \`MaxValue('...')\`, \`TimeAt('...', v)\`) read results back into the analytic solve.

## Scheduling inputs over time

The idiomatic way to drive a transient is a **signal source**: \`SigStep\`, \`SigRamp\`, \`SigSine\`, \`SigPulse\`, or a \`SigTable\` drive cycle, wired to an actuator's command port (see *Domains & Fluid Families*). Component bodies may reference the reserved global \`time\`, which the integrators pin to the running clock — that is what makes those sources tick (in a steady document, pin it yourself with \`time = 0\`).

Raw equations inside the \`DYNAMIC\` body may also reference \`time\` directly — still handy for one-off ramps:

\`\`\`
DYNAMIC pulldown (method = ida, time = 0 .. 600, points = 1201)
  CHLR.frac = 0.05 + 0.95 * min(time/5, 1)   { capacity ramp over 5 s }
END
\`\`\`

Starting a ramp from a small floor (here 5%) rather than zero keeps the first step well-conditioned — see *Troubleshooting Networks*.

## Choosing the integrator

Component transients are usually **DAEs** — differential states coupled to a large algebraic network. Use \`method = ida\` (SUNDIALS implicit DAE integrator) for those; it is the default choice for anything with fluid loops. A pure-ODE network (thermal RC chains, mechanical trains) also runs on the stiff \`ode23s\` / \`ode15s\` methods described in *Transient / ODE Systems (DYNAMIC)*.

[Related: dynamic-ode, comp-linearize, comp-troubleshooting]`,
  "comp-linearize": `# From Plant to Controller (LINEARIZE)

A transient network is a **plant**; the control suite wants it as state-space matrices. The \`LINEARIZE\` block numerically linearizes a named \`DYNAMIC\` block about its operating point and hands you \`(A, B, C, D)\`:

\`\`\`
LINEARIZE plant (block = warmup, a = A, b = B, c = C, d = D)
  INPUT  Q_load
  OUTPUT BATT.port.T
END
\`\`\`

- **States** are the \`DYNAMIC\` block's \`der()\` variables (the storage components' states).
- **INPUT** names the exogenous inputs to perturb; **OUTPUT** the observed quantities — both accept dotted member accessors like \`BATT.port.T\`.
- The matrix names in the header default to \`A\`, \`B\`, \`C\`, \`D\`.

The result is an ordinary set of matrices, so the whole control toolbox applies directly:

\`\`\`
CALL ss2tf(A, B, C, D : num, den)          { transfer function of the plant }
CALL bode(num, den, omega : mag, phase)    { frequency response }
CALL lqr(A, B, Q_w, R_w : K)               { optimal state feedback }
\`\`\`

Close the loop back in the time domain with controller components (\`PIThermostat\` and friends) inside the same \`DYNAMIC\` network — design in the frequency domain, verify in the transient, all in one document.

[Related: comp-transient, symbolic-cas, plot-code]`,
  "comp-cycle-plots": `# Cycle Plots & Diagnostics

## Source-mapped diagnostics

Expansion never leaks into your error messages. Diagnostics and residual reports name **components, ports, and streams** (\`CMP.out.P\`, stream \`s2\`) — never the internal flattened variables — so a convergence failure points at a device you recognize, and the *Debugging a Solve* workflow (F9 block-solve, residual reading, guess seeding) applies unchanged.

## Cycle overlays on property charts

Stream members are first-class citizens of the plotting system. A \`PLOT\` block of kind \`property\` recognizes component stream states, so a refrigeration loop drawn through \`s1 … s4\` overlays as a cycle path on a P-h or T-s chart:

\`\`\`
PLOT 'Cycle'
  kind = property
  fluid = R134a
  diagram = 'P-h'
  overlaystates = true
  connectstates = true
END
\`\`\`

See *Plots in Code (PLOT)* for the full attribute set, and *Fluid State Tables* for the STATE TABLE route to the same overlay.

[Related: plot-code, state-tables]`,
  "comp-wizard": `# The Component Wizard

The **Component Wizard** builds an instantiation line for you — useful while you are still learning a component's parameter surface, and for the map-driven components whose setup is more than one line.

Open it from the editor toolbar, pick a component, and the wizard presents:

- **Every parameter with its meaning and unit**, validated as you type — string parameters (\`fluid$\`) offer the known fluid lists.
- **Variant gating** — selecting a \`model$\` variant shows (and requires) exactly the parameters that variant \`REQUIRE\`s, so you cannot assemble an invalid combination (see *Fidelity Variants*).
- **UA from correlations** — for heat-exchanger components, a helper computes the conductance from geometry and film-coefficient correlations instead of a guessed number, and writes the supporting equations for you.
- **Map ingestion** — for map-based machines (\`CompressorMap\`, \`FanMap\`, \`PumpMap\`), paste or import tabulated curve data and the wizard emits the backing \`TABLE\` block wired to the component's map parameter.

The output is plain frees text inserted at the cursor — the wizard is a typing aid, not a separate model format; everything it writes you could have written by hand.

[Related: comp-variants, comp-library, tables-code]`,
  "comp-troubleshooting": `# Troubleshooting Networks

Everything in *Debugging a Solve* applies to component networks. This page adds the failure modes specific to them.

## Errors at parse time (by design)

frees rejects a malformed network **loudly, before solving** — a hard error beats a silently wrong answer:

- **Port count mismatch** — a shared-name instantiation must bind *all* ports or *none* (none = wire with \`connect\`). \`Component 'LINE' binds 1 port(s) but COMPONENT Pipe declares 2\` means a stream is missing.
- **Mixed domains at a node** — connecting, say, a heat port to a fluid port. Cross domains through a transducer component, never a wire (*Domains & Fluid Families*).
- **Mismatched fluid families** — a \`gas\` line wired to an \`oil\` or \`moistair\` line. Check the components' \`domain$\` tags.
- **Three ports on one shared stream** — the shared-name form is strictly point-to-point; use a \`connect\` node for branches.
- **Missing parameters** — library components have no defaults; every parameter (and every \`REQUIRE\` of the selected \`model$\` variant) must be supplied.

## Convergence: cold-start patterns

A coupled cycle (a refrigeration loop, a pump network) can be structurally perfect and still diverge from a cold start. Three patterns fix most of it:

1. **Seed the pressure level explicitly.** Give every closed loop one component that *pins* pressure — a \`PressureSource\`-style feed or a pinned port member (\`PUMPOUT.in.P = 200000 [Pa]\`). A loop with only relative pressure drops has a floating level the solver must guess.
2. **Don't re-equate mixer pressures.** A mixer's node already equalizes the joining pressures; adding your own \`MIX.in1.P = MIX.in2.P\` duplicates an equation and makes the Jacobian singular.
3. **Floor the capacity, then ramp.** Starting a compressor or valve at exactly zero flow puts property calls at degenerate states. Hold a small floor (\`frac = 0.05\`) for the steady solve, or ramp from it in a transient (\`frac = 0.05 + 0.95 * min(time/5, 1)\`).

## Working method

Build the network **one leg at a time**: source → component → sink, solve, extend. Select a subsystem and press **F9** to solve only it. Diagnostics are source-mapped (component and stream names), so the failing block names the device to look at. Set guesses on stream members (they appear under their display names, e.g. \`s2.P\`) in Variable Info exactly as for scalar variables. And inside the vapor dome, remember the two-phase rule: identify a state by quality \`x\` with \`T\` *or* \`P\`, never both.

[Related: debugging, comp-connections, comp-transient]`,
  "plot-code": `# Plots in Code (PLOT)

Declare figures directly in your code with a \`PLOT ... END\` block. Each block names a figure (quoted title) and sets \`kind\` plus the data attributes for that kind. The figure appears in the Plots panel and can be embedded in reports (see below).

## XY plot (solved arrays)
\`\`\`
PLOT 'Speed vs Distance'
  kind = xy
  x = speed[1:N]
  y = distance[1:N]
  xlabel = 'Speed [m/s]'
  ylabel = 'Distance [m]'
END
\`\`\`

## Thermodynamic property plot
Overlays state points from a \`STATE TABLE\` of the named fluid onto a T-s or P-h chart. Set \`overlaystates\` to draw the points and \`connectstates\` to connect them as a cycle path.
\`\`\`
PLOT 'Boiler Cycle'
  kind = property
  fluid = Water
  diagram = 'T-s'
  overlaystates = true
  connectstates = true
END
\`\`\`

## Control-system plot kinds
Feed these the arrays produced by \`bode\`, \`nyquist\`, \`pole\`/\`zero\` (see *Control Systems & Symbolic CAS*):
- **Bode** — stacked magnitude (dB) and phase (deg) vs. frequency:
\`\`\`
PLOT 'Bode Diagram'
  kind = bode
  omega = omega
  mag = mag
  phase = phase
END
\`\`\`
- **Nyquist** — real vs. imaginary, with the \`-1 + j0\` critical point marked:
\`\`\`
PLOT 'Nyquist Diagram'
  kind = nyquist
  real = re
  imag = im
END
\`\`\`
- **Pole-zero map** — s-plane scatter (poles \`x\`, zeros \`o\`):
\`\`\`
PLOT 'Pole-Zero Map'
  kind = polezero
  pr = pr
  pi = pi
  zr = zr
  zi = zi
END
\`\`\`

Time responses (\`step\`, \`impulse\`, \`lsim\`) reuse the standard **xy** kind with the time vector on \`x\`. The root-locus and Nichols kinds take the matrices/arrays returned by \`rlocus\` and \`nichols\`.

## Embedding plots in reports
Reference any code-defined plot in your narrative with a graph tag and it renders as a live, interactive chart in the **Formatted** view:
\`\`\`
[Graph="Boiler Cycle"] Temperature–entropy diagram of the power cycle [/Graph]
\`\`\`
The name inside the quotes must match a \`PLOT\` block's title.

[Related: reports, symbolic-cas]`,
  "thermo": `# Thermophysical Properties Reference (Real Fluids & Gas)

frees ships a high-precision fluid-properties database. **Every property function returns a value in SI base units** (J/kg, K, Pa, kg/m³, …) regardless of the units you annotate inputs with — annotate inputs for convenience, then convert outputs if you need engineering units.

## Material classes
1. **Real fluids** — multi-phase fluids such as \`Water\`, \`R134a\`, \`Ammonia\`, \`CarbonDioxide\`. Each lookup needs exactly **two** independent properties.
2. **Ideal gases** — spelled formulas (\`CO2\`, \`N2\`, \`CH4\`, \`O2\`, \`Air\`) evaluated with NASA thermodynamic polynomials. Use these for gas-cycle work where real-fluid effects are negligible.
3. **Aqueous glycols** — incompressible coolants written as base + mass percent: \`EG50\` (50% ethylene glycol), \`PG30\` (30% propylene glycol). Queries need Temperature (\`T\`) and Pressure (\`P\`).

## Specifying a state
Pass the fluid name first, then **two** named coordinates. The recognized coordinate keys are \`T\` (temperature), \`P\` (pressure), \`h\` (enthalpy), \`s\` (entropy), \`v\` (specific volume), \`x\` (quality: 0 saturated liquid, 1 saturated vapour), \`u\` (internal energy), \`D\` (density).

\`\`\`
{ Boiler exit: superheated steam }
h3 = Enthalpy(Water, P=P_boiler, T=480 [C])     { J/kg, from kPa + C inputs }
s3 = Entropy(Water, P=P_boiler, T=480 [C])

{ Condenser exit: saturated liquid (x = 0) }
h1 = Enthalpy(Water, P=P_cond, x=0)
v1 = Volume(Water, P=P_cond, x=0)

{ Turbine exit: isentropic, so s4 = s3 -> P and s fix the state }
h4 = Enthalpy(Water, P=P_cond, s=s3)
\`\`\`

## Standard property functions
Every function takes the fluid name first, then coordinates:

| Function | Property | SI Unit | Example |
| --- | --- | --- | --- |
| \`Temperature\` | Absolute temperature | K | \`T = Temperature(Water, P=P1, h=h1)\` |
| \`Pressure\` | Absolute pressure | Pa | \`P = Pressure(R134a, T=T1, x=1)\` |
| \`Enthalpy\` | Specific enthalpy | J/kg | \`h = Enthalpy(Steam, T=T1, P=P1)\` |
| \`Entropy\` | Specific entropy | J/kg-K | \`s = Entropy(Nitrogen, T=T1, P=P1)\` |
| \`Density\` | Mass density | kg/m³ | \`rho = Density(EG50, T=T1, P=P1)\` |
| \`Volume\` | Specific volume | m³/kg | \`v = Volume(Water, P=P1, x=0)\` |
| \`IntEnergy\` | Specific internal energy | J/kg | \`u = IntEnergy(Water, T=T1, P=P1)\` |
| \`Gibbs\` | Specific Gibbs free energy | J/kg | \`g = Gibbs(Water, T=T1, P=P1)\` |
| \`Cp\` / \`Specheat\` | Specific heat $C_p$ | J/kg-K | \`c = Cp(Air, T=T1, P=P1)\` |
| \`Cv\` | Specific heat $C_v$ | J/kg-K | \`c_v = Cv(Air, T=T1, P=P1)\` |
| \`Viscosity\` | Dynamic viscosity | Pa-s | \`mu = Viscosity(Water, T=T1, P=P1)\` |
| \`Conductivity\` | Thermal conductivity | W/m-K | \`k = Conductivity(Water, T=T1, P=P1)\` |
| \`SoundSpeed\` | Speed of sound | m/s | \`c = SoundSpeed(Air, T=T1, P=P1)\` |

[Diagram: DependentProperties]

> **Common pitfall:** because outputs are SI, a Rankine-cycle efficiency \`eta = (h3-h4-w_pump)/(h3-h2)\` is dimensionless and correct as-is — but if you want results in kJ/kg for display, divide the enthalpies by 1000 (or annotate a derived variable \`[kJ/kg]\`).

## Thermophysical utility functions
- **\`P_sat(Fluid, T=t)\`** — saturation pressure at temperature $T$.
- **\`T_sat(Fluid, P=p)\`** — saturation temperature at pressure $P$.
- **\`MolarMass(Fluid)\`** — molar mass (kg/mol) of real fluids, ideal-gas species, or arbitrary formulas (\`C8H18\`, \`Ca(OH)2\`).
- **\`HeatingValue(Fuel, 'LHV'|'HHV')\`** — lower or higher heating value (J/kg).
- **\`StoichAFR(Fuel)\`** — stoichiometric air-fuel ratio (mass basis).
- **\`IsIdealGas(Fluid)\`** — \`1\` if treated as ideal, else \`0\`.
- **\`Phase$(Fluid, T=t, P=p)\`** — phase string: \`'liquid'\`, \`'gas'\`, \`'twophase'\`, \`'supercritical'\`.
- **\`P_crit\` / \`T_crit\` / \`v_crit\` / \`T_triple\`** — critical and triple-point constants.
- **\`CompressibilityFactor(Fluid, T=t, P=p)\`** — $Z = Pv/(RT)$.
- **\`StagnationTemp(T, V, cp)\`** — $T_0 = T + V^2/(2c_p)$ (K).
- **\`StagnationPres(P, T, T0, k)\`** — $P_0 = P(T_0/T)^{k/(k-1)}$ (Pa).
- **\`SurfaceTension(Fluid, T=t)\`** — surface tension (N/m).

[Related: humidair, state-tables, chemistry]`,
  "solid-materials": `# Solid Material Properties Reference

frees carries bulk (room-temperature) physical properties for common engineering solids, so you don't have to look up a conductivity or Young's modulus by hand. Available materials: \`Aluminum\`, \`Copper\`, \`Steel\`, \`StainlessSteel\`, \`Iron\`, \`Brass\`, \`Bronze\`, \`Gold\`, \`Silver\`, \`Lead\`, \`Nickel\`, \`Titanium\`, \`Tungsten\`, \`Zinc\`, \`Magnesium\`, \`Concrete\`, \`Glass\`, \`Brick\`, \`Wood\`, \`Ice\`. Values are representative constants.

## Property functions
Each takes the material name as its first argument. The trailing underscore is part of the name.

| Function | Property | Unit |
| --- | --- | --- |
| \`k_(Material)\` / \`k_(Material, T=t)\` | thermal conductivity | W/m-K |
| \`c_(Material)\` / \`c_(Material, T=t)\` | specific heat | J/kg-K |
| \`rho_(Material)\` | density | kg/m³ |
| \`E_(Material)\` | Young's modulus | Pa |
| \`nu_(Material)\` | Poisson's ratio | — |

\`k_\` and \`c_\` accept an optional temperature \`T\` (kelvin). For the well-characterised metals (aluminum, copper, steel, iron, nickel, titanium, tungsten) a linear correction about the 300 K reference is applied; for other materials, or when \`T\` is omitted, the room-temperature value is returned. \`rho_\`, \`E_\`, and \`nu_\` are constants.

> A material that doesn't carry a requested property (e.g. \`E_(Ice)\` exists but \`nu_(Brick)\` doesn't) raises a clear error rather than returning a wrong value.

## Example
\`\`\`
{ Steady conduction through an aluminum slab }
T_hot = 400 [K];  T_cold = 300 [K]
L = 0.1 [m];  A = 2 [m^2]
k = k_(Aluminum)                 { ~237 W/m-K }
q = k * A * (T_hot - T_cold) / L { watts }
\`\`\`

[Related: thermo, chemistry, ref-index]`,
  "chemistry": `# Chemistry & Combustion

Chemical calculations and combustion analysis for hydrocarbons, alcohols, and common species.

## Molar mass
\`MolarMass\` resolves real fluids, ideal-gas species, **or** arbitrary chemical formulas straight from the periodic table:
\`\`\`
M  = MolarMass(CarbonDioxide)   { 0.04401 kg/mol }
M2 = MolarMass(C8H18)           { 0.11423 kg/mol }
M3 = MolarMass('Al2(SO4)3')     { quote formulas containing parentheses }
\`\`\`
Formulas are **case-sensitive** (element symbols matter): \`Co\` is cobalt, \`CO\` is carbon monoxide. Quote any formula containing parentheses.

## Combustion functions
- **\`HeatingValue(Fuel, mode)\`** — heating value in J/kg. \`mode\` is \`'LHV'\` (water as vapour) or \`'HHV'\` (water as liquid).
- **\`StoichAFR(Fuel)\`** — stoichiometric air-fuel ratio on a mass basis.

\`\`\`
{ Stoichiometric combustion of octane }
LHV = HeatingValue(C8H18, 'LHV')   { ~44.4 MJ/kg }
afr = StoichAFR(C8H18)             { ~15.0 }
\`\`\`

## Radiation view factors
Closed-form diffuse view factors (Howell catalog) for the three configurations textbooks usually read off charts. Arguments are lengths in consistent units; the result is the dimensionless \`F_12\`.
- **\`viewfactor_perp(w1, w2, L)\`** — two perpendicular rectangles sharing an edge of length \`L\` (Howell C-14).
- **\`viewfactor_plates(a, b, L)\`** — two aligned parallel rectangles \`a × b\` separated by \`L\` (Howell C-11).
- **\`viewfactor_disks(r1, r2, L)\`** — coaxial parallel disks, radius \`r1\` → \`r2\`, separated by \`L\` (Howell C-41).

\`\`\`
F_12 = viewfactor_perp(1 [m], 1 [m], 1 [m])   { ~0.2000 }
F_21 = viewfactor_disks(1 [m], 0.5 [m], 0.4 [m])
\`\`\`

## Transient conduction (Heisler charts)
When a solid is suddenly exposed to convection and the Biot number is large enough that internal gradients matter (\`Bi > 0.1\` — where lumped capacitance fails), frees gives the one-term approximation, the computational replacement for reading Heisler/Gröber charts. Accurate for Fourier number \`Fo ≥ 0.2\`.
- **\`heisler_temp(geometry$, Bi, Fo, xstar)\`** — dimensionless temperature $\\theta^* = (T - T_\\infty)/(T_i - T_\\infty)$ at position \`xstar\` (0 centre, 1 surface).
- **\`heisler_q(geometry$, Bi, Fo)\`** — fraction of maximum heat transfer, $Q/Q_0$.

\`geometry$\` is \`'wall'\` (half-thickness L), \`'cylinder'\` (radius r0), or \`'sphere'\` (radius r0). The characteristic length \`Lc\` is \`L\` for the wall and \`r0\` for cylinder/sphere, so \`Bi = h·Lc/k\` and \`Fo = α·t/Lc²\`.

\`\`\`
h = 100 [W/m^2-K];  k = 0.6 [W/m-K]
alpha = 0.15e-6 [m^2/s];  L = 0.02 [m];  t = 600 [s]

Bi = h * L / k
Fo = alpha * t / L^2
theta_c = heisler_temp('wall', Bi, Fo, 0)   { centre }
theta_s = heisler_temp('wall', Bi, Fo, 1)   { surface }
Q_ratio = heisler_q('wall', Bi, Fo)         { heat removed fraction }
\`\`\`

[Related: thermo, solid-materials, ref-index]`,
  "humidair": `# Psychrometrics (AirH2O / Humid Air)

Humid-air calculations use the special fluid name \`AirH2O\`. Unlike pure fluids, every query needs **three** independent coordinates, one of which must be total pressure (\`P\`).

## Coordinate indicators
| Key | Meaning | Unit |
| --- | --- | --- |
| \`T\` | Dry-bulb temperature | K |
| \`P\` | Total (atmospheric) pressure | Pa |
| \`R\` | Relative humidity | 0–1 |
| \`W\` | Humidity ratio | kg water / kg dry air |
| \`D\` | Dew-point temperature | K |
| \`B\` | Wet-bulb temperature | K |
| \`H\` | Specific enthalpy of moist air | J/kg dry air |

> **Unit note:** \`AirH2O\` works in SI internally. If your problem is in °F/psia, convert inputs to K/Pa (and enthalpy outputs back to Btu/lb by dividing by 2326). Several HVAC examples in the Examples Library show this conversion explicitly.

## Dedicated psychrometric functions
- **\`HumRat(AirH2O, T=t, P=p, R=phi)\`** — humidity ratio $\\omega$.
- **\`RelHum(AirH2O, T=t, P=p, W=w)\`** — relative humidity $\\phi$.
- **\`WetBulb(AirH2O, T=t, P=p, R=phi)\`** — wet-bulb temperature.
- **\`DewPoint(AirH2O, T=t, P=p, R=phi)\`** — dew-point temperature.

## Worked example
\`\`\`
T_db = 25 [C]            { dry-bulb }
P_atm = 101325 [Pa]
phi = 0.60               { 60% relative humidity }

w       = HumRat(AirH2O, T=T_db, P=P_atm, R=phi)
T_dew   = DewPoint(AirH2O, T=T_db, P=P_atm, R=phi)
T_wet   = WetBulb(AirH2O, T=T_db, P=P_atm, R=phi)
h_moist = Enthalpy(AirH2O, T=T_db, P=P_atm, R=phi)   { J/kg dry air }
\`\`\`

[Related: thermo, state-tables, ref-fluids]`,
  "state-tables": `# Fluid State Tables (STATE TABLE)

A \`STATE TABLE\` groups the variables that make up one thermodynamic circuit and binds them to a single fluid. Use it whenever a model has state points — it keeps related variables together and unlocks the Fluid States window.

## Declaring a state table
List the state variables in the header, then declare the fluid inside the block:
\`\`\`
STATE TABLE WaterLoop(P1, T1, h1, P2, T2, h2)
  FLUID = Water
END
\`\`\`

## Why use state tables?
- **Fluid isolation** — a \`P1\` in a water loop and a \`P1\` in an R134a loop stay separate; lookups never mix fluids.
- **Auto-fill** — after solving, click **Fill Missing Values** in the Fluid States window to compute every other property (\`s\`, \`v\`, \`x\`, …) from the declared fluid.
- **Cycle overlay** — lets you overlay the whole cycle as a connected path on a property chart (T-s, P-h).

## Two-circuit example
\`\`\`
STATE TABLE WaterLoop(Pw_1, Tw_1, hw_1, Pw_2, Tw_2, hw_2)
  FLUID = Water
END

STATE TABLE RefrigerantLoop(Pref_1, xref_1, href_1, Pref_2, Tref_2, href_2)
  FLUID = R134a
END
\`\`\`

[Related: thermo, plot-code]`,
  "started": `Welcome to **frees** — a declarative equation-solving environment for engineering problems: thermodynamics, fluid mechanics, heat transfer, control systems, structural analysis, and multi-domain simulation.

You write equations the way they appear in a textbook; frees figures out what is unknown and in what order to solve it. This **Get Started** path is the fastest way in — work through the seven steps below in order, then use *Where to go next* to branch into the area you need.

[Diagram: SolverPipeline]

**New here? Start with step 1 below and press *Next* at the bottom of each page.** The rest of the portal is organized as: **Language Fundamentals** (the grammar), **Matrices**, **Programming & Tables**, **Fluids & Materials** (property data), **Solving & Optimization** (how the solver works, debugging, uncertainty), **Dynamic Systems & Control** (ODEs, transfer functions, Bode), **System Modeling with Components** (the acausal component library), **Tools & Workflow** (the REPL, shortcuts, reports), **Examples & Tutorials**, **Architecture & Deployment**, and a per-symbol **Reference**.`,
  "gs-first-solve": `# 1. Your First Solve

The quickest way to understand frees is to solve something. Type this into the editor and press **F2** (Solve):

\`\`\`run
{ Mass of air in a rigid tank }
P = 500 [kPa]
Vol = 0.05 [m^3]
T = 25 [C]
R = 0.287 [kJ/kg-K]
P * Vol = m * R * T      { frees solves this for m }
\`\`\`

You never told frees to "compute \`m\`". It read the five equations, saw that \`m\` was the only unknown, and rearranged the ideal-gas relation to find it. The result appears in the **Solution** panel, in SI units, with any propagated uncertainty.

## Any variable can be the unknown
Swap one line — change \`T = 25 [C]\` to \`m = 0.3 [kg]\` — and the *same* equation now solves for temperature instead. You describe the physics; frees decides the calculation order. That is the whole idea, and the next page explains why it matters.

## The four-step loop
Every model follows the same rhythm:

1. **Describe** the system — algebraic, matrix, or differential equations, in any order.
2. **Check (F4)** — validates syntax and the degrees of freedom (see step 3).
3. **Solve (F2)** — runs the Newton–Raphson solver; results land in the Solution panel.
4. **Sweep** — optionally build a **Parametric Table** (\`Ctrl + T\`) to vary an input and plot the response.

[Related: gs-declarative, shortcuts, variables]`,
  "gs-declarative": `# 2. Thinking Declaratively

In a traditional language you write *assignments*: \`x = y + 2\` means "compute \`x\` from \`y\`". frees is **declarative** — an \`=\` is a mathematical **equality**, a constraint that must hold once solved. The solver looks at the whole system at once and finds the values that satisfy every equation simultaneously.

\`\`\`
P * V = m * R * T      { solve for m, or for V, or for T — all valid }
\`\`\`

Because equations are constraints, **order does not matter** and **any variable on either side can be the unknown**. A consequence: you can transcribe a textbook problem line for line without first rearranging it to isolate the answer.

## Rules that follow from this
- A single \`=\` is equality, never assignment. There is no \`==\`.
- Names are **case-insensitive** — \`Temp\`, \`TEMP\`, and \`temp\` are one variable.
- Implicit multiplication is **not** allowed — write \`2 * x\`, not \`2x\`.
- Everything is computed in **SI base units** internally; you annotate inputs with \`[unit]\`.

The full grammar — operators, comments, constants — is in *Equation Syntax & Rules*. The next page covers the two things that most often decide whether a solve succeeds: units and the Check.

[Related: gs-units-check, syntax, variables]`,
  "gs-units-check": `# 3. Units & Checking the Model

### Annotate inputs; read SI results
frees runs every calculation in SI base units. You annotate **inputs** for convenience and the compiler converts them at parse time:

\`\`\`
P = 500 [kPa]      { stored as 500000 Pa }
T = 25 [C]         { stored as 298.15 K }
m = 120 [lb]       { stored as 54.43 kg }
\`\`\`

Results come back in SI; convert or label them for display (see *Units & Consistency*). Mixing inconsistent units is reported as a warning, never a silent error — and warnings never block a solve.

### Check before you solve (F4)
A system is solvable only when the number of equations equals the number of unknowns — the **degrees of freedom (DoF)** are zero. Press **F4** (Check) to verify this *before* solving: it reports the DoF and any unit mismatches instantly, so you fix structural problems before the solver ever runs. Make F4-before-F2 a habit.

### Guesses make nonlinear solves converge
For nonlinear or transcendental equations, the Newton solver iterates from a **guess**. Open **Variable Info** (\`Ctrl + I\`) to set a starting guess and physical bounds (e.g. \`T ≥ 0\`, \`0 ≤ x ≤ 1\`). A good guess is usually the difference between convergence and divergence.

> **Tip:** If a solve fails to converge, the cause is almost always a missing guess or a wrong unit annotation — not a bug. Check the Solution panel's diagnostics and the Variable Info guesses first.

[Related: gs-plots, units, variables]`,
  "gs-plots": `# 4. See It: Tables & Plots

A single answer is rarely the goal — engineers want the *response*: how the answer moves when an input does. In frees that is a **parametric sweep**, and it takes four lines more than your first solve:

\`\`\`run vary=P=100000:25000:900000
P = 500 [kPa]
Vol = 0.05 [m^3]
T = 25 [C]
R = 0.287 [kJ/kg-K]
P * Vol = m * R * T

PARAMETRIC tank_sweep(T, m)
  T = 275 : 5 : 375 | Linear
END
\`\`\`

*(Drag the \`P\` slider above the code — in pascals — and the whole system re-solves live, the same override mechanism the REPL uses.)*

The \`PARAMETRIC\` block **drives** \`T\` across the range (overriding any fixed value each run) and records \`m\` as a computed output. Open the **Tables** tab and click **Solve Table** — one solve per row fills the grid.

## From table to plot
Select the columns in the table and click **Plot curve** — the figure opens in the **Plots** panel. For figures you want built every solve, declare them in code with a \`PLOT\` block instead (see *Plots in Code*). Property plots (T-s, P-h, psychrometric) with your state points overlaid come later in the *Fluids & Materials* group.

That is the everyday loop: model → sweep → curve. The next two steps add the interactive console and the component library.

[Related: gs-repl, optimization, plot-code]`,
  "gs-repl": `# 5. Ask Questions: the REPL

After a solve, the **REPL terminal** (a dockable console window) holds the whole solved session as a live **workspace**. Instead of editing the document to ask a side question, ask it directly:

\`\`\`
>> m                                  { query a solved value -> with units }
>> m * 3600                           { unit-aware calculator; result stored in ans }
>> Enthalpy('Water', T=400, P=1e5)    { any property or math function }
>> vars                               { list the workspace }
\`\`\`

Three things make it more than a calculator:

- **Implicit solve** — type an equation with one unknown and the REPL solves it on the spot.
- **The CALL library** — eigenvalues, Bode data, partial fractions: \`CALL bode(num, den, omega : mag, phase)\` works interactively, with output sizes inferred for you.
- **Symbolic CAS** — \`Factor(x^2 - 1)\`, \`Apart(...)\`, \`Laplace(...)\` return transformed expressions (REPL-only).

The full command set is on the *REPL Terminal & Workspace* page. One step left: components.

[Related: gs-components, repl, shortcuts]`,
  "gs-components": `# 6. Wire Components

For system problems — loops, circuits, networks — frees has a library of ~295 **components**: parameterized, connectable blocks of physics. You wire them; frees turns the network into equations and solves it like everything else:

\`\`\`run
{ What pressure does 50 m of pipe cost? }
Source  SUP(fluid$=Water, mdot=2 [kg/s], P=300000 [Pa], T=298 [K])
Pipe    LINE(fluid$=Water, L=50 [m], D=0.05 [m], rough=0.0001)
Sink    RET()

connect(SUP.out, LINE.in)
connect(LINE.out, RET.in)

dP = SUP.out.P - RET.in.P
\`\`\`

Solve, and read \`dP\` — the \`Pipe\` computed density, Reynolds number, and friction factor internally. Port members like \`LINE.out.P\` are ordinary variables you can probe or pin.

This scales a long way: pumps, heat exchangers, refrigerant circuits, electrical and mechanical elements, humid-air HVAC — including transients, from the same wiring. The **System Modeling with Components** group teaches it properly, starting with *Your First Component Network*.

[Related: gs-next, comp-first-network, comp-library]`,
  "gs-next": `# 7. Where to Go Next

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

[Related: journey-scalar, journey-chain, journey-upgrade, lang-overview, fluids-overview, components-overview, examples]`,
  "repl": `# REPL Terminal & Workspace

The **REPL terminal** is a dockable, interactive console — move and dock it anywhere like the editor. It evaluates **one line at a time** against the current **workspace** (every variable from the last solve, plus anything you define in the REPL). It's a line-oriented math REPL, not a shell: use it as a unit-aware calculator, to inspect solved values, to try \`CALL\` routines, and to run symbolic CAS transforms. **Up/Down** recall history; **Tab** completes variable, function, and command names.

## Meta-commands
These drive the app instead of evaluating an expression:

| Command | Action |
| --- | --- |
| \`help\` | show in-terminal usage |
| \`clc\` | clear the screen |
| \`clear\` | drop **all** REPL-defined overrides |
| \`clear <var>\` | drop one REPL variable overlay (e.g. \`clear x\`) |
| \`vars\` / \`who\` / \`whos\` | list workspace variables with values and units |
| \`check\` | run the document Check (DoF / solvability) |
| \`solve\` | solve the document with any active REPL overrides |

## Expressions
Type any expression; a bare result is stored in \`ans\` and is reusable on the next line. Every built-in math function works (trig, \`exp\`/\`ln\`/\`sqrt\`, \`erf\`/\`gamma\`, Bessel, \`mod\`/\`gcd\`, complex \`real\`/\`imag\`/\`angle\`, …), as do fluid-property and chemistry functions.
\`\`\`
2 * sqrt(9) + 4                    { = 10 }
Enthalpy('Water', t=400, p=1e5)    { J/kg }
\`\`\`

## Variables: query, assign, solve
- **Query** a workspace value (shown with units and uncertainty): \`T_1\` → \`300 [K]\`.
- **Assign** a REPL variable (persists for the session, visible to later lines and a subsequent \`solve\`): \`x = 42 [m/s]\`.
- **Implicit single-unknown solve** — give an equation with exactly one unknown and frees solves it: \`P = 50000 * volume\` → \`volume = 5 [m^3]\`.

## Matrices, vectors, ranges
\`\`\`
A = [2 0; 0 3]          { = [2 0; 0 3] }
[1:2:7]                 { = [1 3 5 7] }
A * A                   { matrix product -> ans[i,j] }
Inverse(A)   Transpose(A)   Dot(u, v)
\`\`\`

## The CALL library (auto-sized outputs)
The full \`CALL\` procedure library (eigenvalues, control-systems analysis, partial fractions, decompositions) runs in the REPL. **Output lengths are sized automatically from the inputs**, so bare output names work — no \`[1:n]\` annotation:
\`\`\`
CALL Eigenvalues(A : lambda)            { lambda = [2 3] }
CALL Routh(den : nRHP, stable)
CALL residue(num, den : rr, ri, pr, pi, k)
CALL Bode(num, den, omega : mag, phase)
\`\`\`
Only genuinely value-dependent counts take an explicit size: the finite-zero counts of \`zero\`/\`tf2zp\` (e.g. \`zr[1:2]\`), and the root-locus sweep resolution of \`rlocus\` (defaults to 100 points). This auto-sizing applies in the editor document too.

## Symbolic CAS (REPL only)
The REPL exposes the embedded **Symja** computer-algebra engine as functions that return a transformed expression as text. Free variables stay symbolic, so no solved context is needed:

| Function | Example → result |
| --- | --- |
| \`Factor(expr)\` | \`Factor(x^2 - 1)\` → \`(-1+x)*(1+x)\` |
| \`Expand(expr)\` | \`Expand((x+1)^3)\` → \`1+3*x+3*x^2+x^3\` |
| \`Simplify(expr)\` | algebraic simplification |
| \`Together(expr)\` / \`Cancel(expr)\` | common denominator / cancel common factors |
| \`Numerator(expr)\` / \`Denominator(expr)\` | split a rational expression |
| \`Collect(expr, var)\` | group by powers of a variable |
| \`Diff(expr, var)\` | \`Diff(x^3 + x^2, x)\` → \`2*x+3*x^2\` |
| \`Integrate(expr, var)\` | \`Integrate(x^2, x)\` → \`x^3/3\` |
| \`Apart(expr, var)\` | \`Apart((s+3)/(s^2+3*s+2), s)\` → \`2/(1+s)-1/(2+s)\` |
| \`Laplace(f, t, s)\` | Laplace transform |
| \`InverseLaplace(F, s, t)\` | \`InverseLaplace(1/(s+2), s, t)\` → \`E^(-2*t)\` |

When the CAS can't find a closed form, the REPL reports *"no closed form found"* rather than echoing the call. These symbolic functions are **REPL-only**; in the editor, symbolic work uses \`SYMBOLIC\` identities and \`CALL residue\` (see *Control Systems & Symbolic CAS*).

## What the REPL does not do
The REPL evaluates a single expression per line, so multi-line block constructs are editor-only: \`FUNCTION\`/\`PROCEDURE\`/\`MODULE\` definitions, \`DYNAMIC\` ODE systems, \`TABLE\` blocks, \`IF\`/\`FOR\` control flow, and the \`SYMBOLIC\`/\`SOLVE BLOCK\` directives. You can *call* a function or read \`ODEValue\`/\`Interpolate\`/table accessors that a prior solve produced — you just can't *define* the block from the REPL.

[Related: shortcuts, symbolic-cas, matrices-sys]`,
  "shortcuts": `# Keyboard Shortcuts

| Hotkey | Action |
| --- | --- |
| \`F2\` or \`Ctrl + Enter\` | **Solve** — runs the Newton-Raphson solver |
| \`F4\` or \`Ctrl + K\` | **Check** — validates syntax, degrees of freedom, and expands blocks |
| \`Ctrl + I\` | Open the **Variable Information** panel (guesses & bounds) |
| \`Ctrl + T\` | Open the **Parametric Table** panel |
| \`F9\` | **Solve selected block only** — ignores all other lines |
| \`F1\` | **Contextual help** — opens the reference page for the symbol under the cursor |

> **Tip:** make \`F4\` (Check) a habit before \`F2\` (Solve). It reports the DoF and any unit mismatches instantly, so you fix problems before the solver runs. For parametric-table examples, use **Solve Table** in the Tables tab instead of \`F2\`.

[Related: gs-units-check, variables, repl]`,
  "reports": `# Notes & Narrative

Structure a document as a readable calculation note: equations in any order, with the narrative alongside them as comments. Comments never affect solving.

## Mixing narrative and equations
- A line starting with \`//\`, or any text inside \`{ }\`, is a comment. Use them for section headings, explanations, and data provenance.
- A \`{ }\` comment may span several lines, so a paragraph of narrative can sit above the equations it describes.
- An inline comment documents a single equation in place: \`Q = m*cp*dT { sensible heat }\`.

## Named figures from code
A \`PLOT ... END\` block (see *Plots in Code*) declares a named figure that appears in the **Plots** window after every solve, so a document carries its own charts next to the equations they visualize.

## Example
\`\`\`
{ Boiler Analysis — the pressure and firing temperature
  below fix the cycle's thermal efficiency. }

P_high = 8000 [kPa]
T_boiler = 500 [C]
eta_th = 36.9
\`\`\`
Press Solve (F2): solved values appear in the Solution window, and any PLOT figures in the Plots window.

[Related: plot-code, reports]`,
  "digitizer-fit": `# Graph Digitizer & Curve Fit

Two integrated tools turn data — measured or read off a chart — into usable equations: the **Graph Digitizer** extracts (x, y) points from an image, and the **Curve Fit Engine** fits a model to a table and writes the equation for you.

## Digitizer workflow
1. **Open** the Graph Digitizer icon in the left toolbar and upload an image of your chart.
2. **Calibrate** — mark two known points on the X-axis and two on the Y-axis to set the coordinate system.
3. **Digitize** — click points along the curve; their coordinates are computed and added to a table.
4. **Export** to an internal table (e.g. \`digitized_curve\`).

## Curve fit workflow
5. Open the **Curve Fit** panel, select your table, choose a model template (Linear, Polynomial, Exponential, …), and fit.
6. Copy the generated equation into the editor. The fit is returned as a plain frees expression you can paste straight in:

\`\`\`
{ Fit of pump head vs flow, from a digitized catalog curve }
flow_rate = 1.25 [m^3/s]
head_loss [m] = -0.084 * flow_rate^2 + 1.54 * flow_rate + 0.12 [m]
\`\`\`

> **Tip:** you can also define the data inline with a \`TABLE\` block (see *Custom Tables*) and fit against that — handy for reproducing a textbook table without an image. The statistics example in the Examples Library shows exactly this route.

[Related: tables-code, lookup-tables, reports]`,
  "journey-scalar": `# Journey 1 — A scalar equation with units

**Goal.** Solve one textbook equation, read the SI result, and recover from a missing unit.

**Do this.** Type the block, press **F4** then **F2**. Expected: \`m ≈ 0.293 kg\`.

\`\`\`run
{ Mass of air in a rigid tank }
P = 500 [kPa]
Vol = 0.05 [m^3]
T = 25 [C]
R = 0.287 [kJ/kg-K]
P * Vol = m * R * T
\`\`\`

\`T = 25 [C]\` is an absolute temperature (298.15 K). A 10 °C *rise* is 10 K — see *Units & Dimensional Consistency*.

**Deliberate error.** Change \`P = 500 [kPa]\` to \`P = 500\`. Check still reports the system solvable; the unknown-unit / unconverted-value warning is the recovery, not a failed solve. Put \`[kPa]\` back and Solve — \`m\` returns to ~0.293 kg.

[Related: gs-first-solve, units, variables]`,
  "journey-chain": `# Journey 2 — A component chain

**Goal.** Build source → pipe → sink with named parameters and \`connect\`, then recover a missing boundary.

**Do this.** Insert from the Component Wizard (or type). Expected: \`dP\` is a positive frictional drop, on the order of kilopascals for this pipe.

\`\`\`run
Source SUP(fluid$='Water', mdot=2 [kg/s], P=300000 [Pa], T=298 [K])
Pipe   LINE(fluid$='Water', L=50 [m], D=0.05 [m], rough=0.0001)
Sink   RET()
connect(SUP.out, LINE.in)
connect(LINE.out, RET.in)
dP = SUP.out.P - RET.in.P
\`\`\`

**Deliberate error.** Delete \`mdot=2 [kg/s]\` from the Source (or comment out the SUP.out–LINE.in \`connect\`). Check reports the free quantity; the Schematic highlights the open port. Restore the boundary — do not invent a pressure. See *Your First Component Network* and *Connections & Junctions*.

Positional streams (\`Source SUP(s1, …)\`) are an advanced shorthand for two-port chains; stay on \`connect\` until this journey is fluent.

[Related: comp-first-network, comp-connections, journey-loop]`,
  "journey-upgrade": `# Journey 3 — Upgrade: map or transient

**Goal.** Replace a constant-rise fan with a map, or add a \`DYNAMIC\` block, without losing the wiring.

**Map path.** Start from a \`Fan\` (needs \`dP0\`, \`Q0\`, \`eta\`). In the wizard, open **Related models** and pick \`FanMap\` when you have a ΔP vs Q table. The ports stay \`in, out\`; only the required data change. Expected: \`connect\` lines do not need rewriting. A missing \`map$\` is a Check error naming the parameter — recover by inserting the TABLE (or the map builder), not by switching variants silently.

**Transient path.** Keep the algebraic network and add a first-order lag on a probe, or follow *Tutorial: Mass–Spring–Damper* for a full \`DYNAMIC\` document. Name the independent axis \`time\`, not \`t\`, so it does not collide with a temperature \`T\`.

[Related: tut-msd, journey-chain, variables]`,
  "journey-loop": `# Mixers and closed loops

After the chain: two streams at different states that join need a **mixer** (\`Mixer\`, \`LiquidMixer\`, \`MixingBox\`). A three-port \`connect\` is a legal *branch* (equal pressure, mass conserved) but it does not mix enthalpy.

A **closed loop** is \`connect\` back through the load, not through a second inlet on the source:

\`\`\`
connect(PUMP.out, HX.in)
connect(HX.out, PUMP.in)
\`\`\`

Parameter tables live on each component's Reference page — this journey does not copy them.

[Related: journey-chain, comp-connections, comp-library]`,
  "syntax": `# Equation Syntax & Rules

frees parses standard mathematical notation with a few rules worth knowing up front.

## Core rules
- **Equality (\`=\`)** — a single \`=\` is mathematical equality, never assignment. \`P * V = m * R * T\` is valid; the solver rearranges it to find whichever variable is unknown. There is no \`==\` or \`:=\` at the top level (those are for \`FUNCTION\`/\`PROCEDURE\` bodies).
- **Case insensitivity** — \`Temp\`, \`TEMP\`, and \`temp\` are one variable. Watch for accidental clashes: a state \`T\` and a time \`t\` are the same name (rename one).
- **No implicit multiplication** — write \`2 * x\`, not \`2x\`. Likewise \`a(b+c)\` is a function call, not \`a*(b+c)\`.
- **Operators** — \`+\`, \`-\`, \`*\`, \`/\`, \`^\` (exponentiation), and \`%\` (modulo). \`^\` is right-binding: \`2^3^2 = 2^9\`.
- **Comments** — \`{ … }\` or \`"…"\` are inline comments; \`//\` at the start of a line makes the whole line narrative (markdown). Use comments to label states and document assumptions.
- **Strings** — prefer **single quotes** for values the solver reads (\`fluid$='Water'\`, \`INCOMP::MEG[0.50]\`). A double-quoted \`"…"\` span is a comment, the same as \`{ … }\`, so \`"Water"\` is not a fluid name.

## Built-in constants
Physical constants are available with a trailing \`#\` (by long-standing convention) and substituted at parse time:

| Name | Meaning |
| --- | --- |
| \`pi#\` | $\\pi$ |
| \`g#\` | Standard gravity, $9.80665$ m/s² |
| \`R#\` | Universal gas constant, $8.31446$ J/mol·K |
| \`N#a\` | Avogadro's number |
| \`k#\` | Boltzmann constant |
| \`h#\` | Planck constant |
| \`c#\` | Speed of light |
| \`sigma#\` | Stefan–Boltzmann constant |
| \`epsilon0#\` | Vacuum permittivity |

\`\`\`
{ Free-fall distance in 3 s }
d = 0.5 * g# * t^2
\`\`\`

[Related: gs-declarative, variables, units]`,
  "math-funcs": `# Mathematical Functions

frees provides a full set of scalar math functions. Built-in functions are supported by the solver's Jacobian engine via symbolic automatic differentiation for smooth expressions, with numerical finite-difference fallbacks for non-smooth or complex operations.

## Trigonometric (angles in radians)
\`sin\`, \`cos\`, \`tan\` and their inverses \`arcsin\`, \`arccos\`, \`arctan\` take and return **radians**. Work in degrees with a unit annotation or \`Convert\`:
\`\`\`
theta = 30 [deg]          { stored as radians internally }
y = sin(theta)            { 0.5 }
deg = theta * Convert('rad', 'deg')
\`\`\`
\`atan2(y, x)\` returns the quadrant-correct angle of the point \`(x, y)\`:
\`\`\`
phi = atan2(1, -1)        { 2.356 rad = 135 deg }
\`\`\`

## Logarithms, exponentials, powers
| Function | Description |
| --- | --- |
| \`exp(x)\`, \`ln(x)\`, \`log10(x)\`, \`log2(x)\` | exponential and logs (natural / base-10 / base-2) |
| \`sqrt(x)\`, \`cbrt(x)\` | square / cube root |
| \`abs(x)\` | absolute value |
| \`min(a,b,…)\`, \`max(a,b,…)\` | element selection |
| \`mod(a, b)\`, \`gcd(a, b)\`, \`lcm(a, b)\` | modulo, greatest common divisor, least common multiple |
| \`factorial(n)\` | $n!$ (integer) |

## Hyperbolic
\`sinh\`, \`cosh\`, \`tanh\` and \`arcsinh\`, \`arccosh\` (x ≥ 1), \`arctanh\` (|x| < 1). Note \`sinh(x) + cosh(x) = exp(x)\`.

## Rounding & integer
| Function | Description |
| --- | --- |
| \`round(x, decimals)\` | round to \`decimals\` places |
| \`floor(x)\` / \`ceil(x)\` / \`trunc(x)\` | round down / up / toward zero |
| \`sign(x)\` | -1, 0, or 1 |
| \`step(x)\` | unit step (1 if x ≥ 0, else 0) |

\`\`\`
val1 = round(3.14159, 3)   { 3.142 }
val2 = floor(2.7)          { 2 }
val3 = step(0.5)           { 1 }
\`\`\`

## Conditional selection & series
| Function | Description |
| --- | --- |
| \`If(a, b, lt, eq, gt)\` | returns \`lt\` if \`a<b\`, \`eq\` if \`a=b\`, \`gt\` if \`a>b\` |
| \`Sum(i, start, end, term)\` | $\\sum_{i=start}^{end} term$ |
| \`Product(i, start, end, term)\` | $\\prod_{i=start}^{end} term$ |
| \`average(a, b, …)\` | arithmetic mean |

\`If\` is the inline branch for the declarative top level (use \`IF…THEN…ELSE\` inside \`FUNCTION\`/\`PROCEDURE\` bodies):
\`\`\`
{ Pick k = 1.8 above 300 K, else 1.2 }
temp = 350 [K]
k = If(temp, 300, 1.2, 1.5, 1.8)   { k = 1.8 }

{ Sum of squares 1+4+9+16 = 30 }
s = Sum(i, 1, 4, i^2)
\`\`\`

> **Looking for one function?** This page teaches the families; every built-in has its own page with full syntax, arguments, and examples. Browse them all in **Reference → A–Z Function Index**.

[Related: special-funcs, ref-index, complex]`,
  "special-funcs": `# Special & Statistical Functions

Transcendental and statistical distribution functions for less common but important calculations.

## Statistical distributions
- **\`Probability(x1, x2, mean, stddev)\`** — probability that a normal variate lies in the interval \`[x1, x2]\`.
- **\`NormalCDF(x, mean, stddev)\`** — cumulative normal probability \`Pr(X ≤ x)\`.
- **\`Chi_Square(x, df)\`** — cumulative chi-square CDF at \`x\` with \`df\` degrees of freedom.
- **\`Random(a, b[, seed])\`** — uniform random number in \`[a, b]\`.
- **\`RandG(mean, stddev[, seed])\`** — Gaussian random number.

\`\`\`
prob = Probability(75, 85, 80, 5)   { 0.6827 — within ±1σ of N(80, 5) }
\`\`\`

## Special mathematical functions
- **\`Gamma(x)\`** — $\\Gamma(x)$, with $\\Gamma(n+1)=n!$.
- **\`LogGamma(x)\`** — $\\ln \\Gamma(x)$ (avoids overflow for large x).
- **\`Digamma(x)\`** — $\\psi(x) = \\frac{d}{dx}\\ln\\Gamma(x)$.
- **\`Beta(a, b)\`** — $B(a,b) = \\frac{\\Gamma(a)\\Gamma(b)}{\\Gamma(a+b)}$.
- **\`Erf(x)\` / \`Erfc(x)\` / \`ErfInv(x)\`** — error function, complementary, and inverse.
- **\`BesselJ(n, x)\` / \`BesselY(n, x)\`** — Bessel functions of the first and second kind, order $n$.
- **\`BesselI(n, x)\` / \`BesselK(n, x)\`** — modified Bessel functions of the first and second kind.

> Each function above has a dedicated reference page with its mathematical definition and worked examples — see **Reference → A–Z Function Index**.

[Related: math-funcs, ref-index, uncertainty]`,
  "variables": `# Variables, Guesses & Bounds

A system is solvable only when the number of equations equals the number of unknowns — the **degrees of freedom (DoF)** are zero. Press **F4** (Check) to see the DoF and confirm the system is well-posed before solving.

[Diagram: DoF]

## Bounds versus units
\`T = 25 [C]\` is a **unit annotation**: the compiler converts the literal to SI (298.15 K) before solving. \`GUESS T [250, 400]\` is a **solver bound** on that same SI value — it does not convert, and it is not a unit. Mixing the two (\`GUESS T [0, 100]\` intending Celsius) boxes the Newton iterate in kelvin.

In-text \`GUESS\` wins over Variable Information on conflict; the window is a view of the same scalars, including public member paths such as \`HX.in.P\`.

## The Variable Information panel
Open it with \`Ctrl + I\`. For every variable you can set:

- **Guess** — the starting point for the Newton-Raphson solver. Required for nonlinear equations; a poor guess is the most common cause of non-convergence.
- **Lower / Upper bounds** — physical limits that keep the solver out of invalid domains (e.g. \`T ≥ 0\`, \`0 ≤ x ≤ 1\` for a quality or fraction, \`P > 0\`).
- **Fixed** — locks the variable to its guess, removing it from the unknowns. Handy for "what if I hold this constant" studies.

## \`time\` versus temperature names
Names are case-insensitive, so a state \`T\` and a time \`t\` are **one variable**. In a \`DYNAMIC\` block, name the independent axis \`time\` (the default) and keep temperatures as \`Temp\` / \`T_wall\` / \`T_inf\`. See *Transient / ODE Systems* for the collision that follows from calling both \`t\`.

## Why guesses matter
The Colebrook friction equation is transcendental — it has no closed form, so frees iterates from a guess. Without a guess it may diverge or land on the wrong branch:

\`\`\`
Re = 1e5
eps = 0.00015
D = 0.25 [m]
{ ff is unknown — set a guess of ~0.02 in Variable Info }
1/sqrt(ff) = -2*log10(eps/(3.7*D) + 2.51/(Re*sqrt(ff)))
\`\`\`
A guess near \`0.02\` converges in a few iterations; a guess of \`0.5\` may stall. As a rule of thumb, guess dimensionless ratios near \`0.5\`, temperatures/pressures near the expected magnitude, and flow rates near the order of magnitude you expect.

> **Tip:** For implicit property lookups (e.g. \`h = Enthalpy(Water, P=P, s=s)\`), a guess on the unknown output variable also helps the solver pick the right two-phase region.

[Related: gs-units-check, uncertainty, api]`,
  "uncertainty": `# Uncertainty Propagation

frees does automatic **first-order** uncertainty propagation: declare the tolerance of each measured input, and it propagates the uncertainty to every dependent result using the root-sum-of-squares (RSS) rule. You don't write the partial derivatives — the solver computes them from the numerical Jacobian.

## How to use it
1. **Declare input uncertainties** with \`UncertaintyOf(var) = value\` on your independent (measured) variables.
2. **Query output uncertainties** with \`UncertaintyOf(var)\` on any dependent variable — frees returns its propagated absolute uncertainty.

The combination rule is:
$$u_y = \\sqrt{\\sum \\left(\\frac{\\partial y}{\\partial x_i} u_{x_i}\\right)^2}$$

## Worked example
\`\`\`
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
\`\`\`
Only independent inputs should carry a declared uncertainty; assigning one to a computed output that you also query is redundant. Uncertainties are shown alongside each value in the Solution panel.

[Related: variables, units, api]`,
  "units": `# Units & Dimensional Consistency

frees checks every equation for dimensional consistency and runs all calculations in SI base units internally. You annotate **inputs** for convenience; **results** come back in SI and you convert or label them as needed.

## Annotating inputs
Bracket a numeric literal to tag it with a unit; the compiler converts it to SI at parse time:
\`\`\`
P = 140 [kPa]      { stored as 140000 Pa }
m = 120 [lb]       { stored as 54.43 kg }
T = 25 [C]         { stored as 298.15 K }
\`\`\`
The full list of recognized units and built-in constants lives in the reference table below (\`[Component: UnitsReference]\`).

## Results are SI
A computed result has the SI unit of its expression — \`P * Vol = m * R * T\` gives \`m\` in kg, \`q = k*A*dT/L\` gives watts. To display in engineering units, either convert explicitly, set display units in the Variable Information window, or use the \`Convert\` function:
\`\`\`
P_kPa  = P / 1000                { kPa, by division }
P_kPa2 = P * Convert(Pa, kPa)    { using Convert }
\`\`\`

## The Convert() function
\`Convert(From, To)\` returns the pure scaling factor between two units of the **same dimension** (no offset):
\`\`\`
area_in2 = area_ft2 * Convert(ft^2, in^2)     { ×144 }
\`\`\`

## Temperature conversions
Temperature scales have offsets as well as scaling, so use \`ConvertTemp(From, To, value)\` instead of \`Convert\`:
\`\`\`
T_f = ConvertTemp(C, F, 100)   { 212 F }
T_k = ConvertTemp(F, K, 32)    { 273.15 K }
\`\`\`

## Worked example
\`\`\`
{ Pressure input in psi, converted to kPa }
P = 100 [psi]                 { stored as 689475.7 Pa internally }
P_kPa = P * Convert(Pa, kPa)  { converts SI Pa to kPa: 689.5 kPa }
\`\`\`

> **Common pitfall:** \`Convert\` works for differences and ratios (kPa, ft², mph); it does **not** handle temperature offsets. Mixing them — e.g. \`Convert(C, K)\` — gives a wrong result. Always use \`ConvertTemp\` for absolute temperatures.

## Absolute temperatures versus differences
\`T = 25 [C]\` is an **absolute** temperature (stored as 298.15 K). A *difference* such as a 10 °C rise is 10 K — annotate it \`[deltaC]\` / \`[deltaK]\`, or write the kelvin difference directly. \`Convert(C, K)\` is the wrong tool for either: it has no offset, so it cannot turn 25 °C into 298.15 K, and it is unnecessary for a difference that is already 10 K.

[Component: UnitsReference]

[Related: syntax, variables, ref-units]`,
  "arrays": `# Arrays & For Loops

An array element is written with a 1-based index in square brackets: \`T[1]\`, \`P[5]\`. Declare the array's size with a slice suffix when you first use it (\`T[1:5]\`), so the compiler can allocate it; afterwards the bare name works.

## The FOR loop = equation expansion
A \`FOR ... END\` block isn't a runtime loop — the compiler **expands** it into one equation per index at compile time. This is the idiomatic way to generate a family of equations (one per state point, node, or time step):
\`\`\`
{ One enthalpy equation per state }
P[1:3] = [8000, 2000, 10]      { kPa }
T[1:3] = [480, 200, 45]        { C }
FOR i = 1 TO 3
  h[i] = Enthalpy(Water, P=P[i], T=T[i])
END
\`\`\`
The loop variable (\`i\`) is local to the block and must not clash with a model variable (names are case-insensitive).

## Array helper functions
- **\`ArrayElmt(array[1:N], index)\`** — element at a **dynamically computed** index. Use it when the index is itself a variable (a plain \`T[idx]\` only works with a literal index):
\`\`\`
idx = 3
val = ArrayElmt(T[1:10], idx)   { the value of T[3] }
\`\`\`

> **Tip:** for matrices and vectors declared with literals, see *Declaring Matrices & Vectors*. For column-by-column access to solved tables, see *Table Accessors & Aggregates*.

[Related: matrices-decl, functions, table-accessors]`,
  "complex": `# Complex Numbers

frees supports complex numbers natively. A complex variable is stored as two real scalars: append \`_r\` for the real part and \`_i\` for the imaginary part. So \`Z_r = 10\` and \`Z_i = 5\` together represent $Z = 10 + 5j$.

Arithmetic operators (\`+\`, \`-\`, \`*\`, \`/\`, \`^\`) work on the paired \`_r\`/\`_i\` variables automatically — you write \`Z = A * B\` and frees keeps both components in step.

## Helper functions
| Function | Returns |
| --- | --- |
| \`Real(z)\` / \`Imag(z)\` | real / imaginary part |
| \`Conj(z)\` | complex conjugate |
| \`Magnitude(z)\` | modulus $\\lvert z \\rvert$ |
| \`Angle(z)\` / \`AngleDeg(z)\` | argument in radians / degrees |
| \`Cis(theta)\` | $e^{j\\theta} = \\cos\\theta + j\\sin\\theta$ |

\`\`\`
{ Build a phasor from magnitude and angle }
Z_r = Magnitude_r      { reuse the real parts }
Z_i = 0
A = Cis(phi)           { unit phasor at angle phi }
\`\`\`

[Related: math-funcs, symbolic-cas, matrices-sys]`,
  "strings": `# String Variables & Functions

A string variable ends with \`$\` (e.g. \`fluid$\`), and string literals use **single** quotes (\`'R134a'\`, \`'wall'\`). Strings are resolved at compile time — most often as a fluid name in a property call, or as a geometry label for the Heisler functions.

\`\`\`
fluid$ = 'R134a'
h = Enthalpy(fluid$, P=P1, x=1)     { fluid name from a string variable }
\`\`\`

## String functions
These take string-literal (or string-variable) arguments and return a number:
- **\`StringLen(s)\`** — number of characters.
- **\`StringPos(s, sub)\`** — 1-based index of the first occurrence of \`sub\` in \`s\` (0 if not found).
- **\`StringVal(s)\`** — convert a numeric string to a scalar (\`StringVal('3.14')\` → \`3.14\`).

String-returning functions (also suffixed \`$\`) include \`LowerCase$\`, \`UpperCase$\`, \`Trim$\`, \`Concat$\`, \`Copy$\`, \`Chr$\`, \`Date$\`, \`Time$\`, \`TimeStamp$\`, \`UnitSystem$\`, and \`UnitsOf$\`.

[Related: syntax, thermo, ref-index]`,
  "matrices-decl": `# Declaring Matrices & Vectors

frees uses array-language-like syntax for matrices and vectors. **You must declare the shape with a slice suffix** so the compiler can size the variable — the literal alone isn't enough.

## Notation
- **Vector:** \`v[1:3] = [1, 2, 3]\` — elements in brackets, comma-separated. The \`[1:3]\` declares a 3-element vector.
- **Matrix:** columns separated by commas, rows by semicolons:
\`\`\`
A[1:2, 1:2] = [1, 2; 3, 4]
\`\`\`
- **Slice suffixes:** \`[start:end]\` (1-based) tell the compiler the dimensions. Always include them on the left of an assignment when you first define an array; once sized, the bare name works downstream.

## Generation helpers
- **\`zeros(m, n)\`** — $m \\times n$ zero matrix.
- **\`ones(m, n)\`** — $m \\times n$ ones matrix.
- **\`eye(n)\` / \`identity(n)\`** — $n \\times n$ identity.
- **\`diag(v)\`** — diagonal matrix from a vector, or the diagonal of a matrix.
- **\`linspace(a, b, n)\`** — $n$ values linearly spaced from $a$ to $b$.

## Examples
\`\`\`
I[1:3, 1:3] = eye(3)                 { 3x3 identity }
grid[1:11] = linspace(0, 1, 11)      { 0, 0.1, …, 1.0 }
\`\`\`

> **Tip:** for control-systems work, transfer-function coefficient arrays are just vectors — \`num = [0, 0, 1]\` and \`den = [1, 3, 2]\` represent $1/(s^2+3s+2)$. See *Control Systems & Symbolic CAS*.

[Related: matrices-ops, matrices-sys, arrays]`,
  "matrices-ops": `# Matrix Operators

Standard algebraic operators work element-wise or as matrix operations depending on shape, sized automatically.

## Operators
- **\`+\` / \`-\`** — add/subtract same-shaped matrices (element-wise).
- **\`*\`** — matrix multiplication (sized automatically). A scalar times a matrix scales every element.
- **\`'\`** (postfix apostrophe) — transpose: \`At = A'\`.
- **\`\\\`** (left division) — solves $A x = b$ directly: \`x = A \\ b\`. Equivalent to \`SolveLinear(A, b)\`.

## Example: solve a 2×2 system
\`\`\`
A[1:2, 1:2] = [1, 2; 3, 4]
b[1:2] = [5, 6]
x[1:2] = A \\ b      { solves A * x = b }
\`\`\`

[Related: matrices-sys, matrices-blas, matrices-decl]`,
  "matrices-blas": `# OpenBLAS Algebra Functions

For high-performance vector/matrix operations, frees binds directly to BLAS (Basic Linear Algebra Subprograms). These are useful when you want explicit control over scaled updates.

## BLAS routines
- **\`axpy(alpha, x, y)\`** — $\\alpha x + y$ (Level 1).
- **\`scal(alpha, x)\`** — $\\alpha x$.
- **\`asum(x)\`** — $L_1$ norm (sum of absolute values).
- **\`nrm2(x)\`** — Euclidean $L_2$ norm.
- **\`copy(x)\`** — symbolic copy of a vector.
- **\`gemv(alpha, A, x, beta, y)\`** — $\\alpha A x + \\beta y$ (Level 2).
- **\`ger(alpha, x, y, A)\`** — outer-product update $\\alpha x y^T + A$.
- **\`gemm(alpha, A, B, beta, C)\`** — $\\alpha A B + \\beta C$ (Level 3).

## Example
\`\`\`
v1[1:3] = [1, 2, 3]
v2[1:3] = [4, 5, 6]
result[1:3] = axpy(2.5, v1[1:3], v2[1:3])   { 2.5*v1 + v2 }
\`\`\`

[Related: matrices-sys, matrices-ops, ref-index]`,
  "matrices-sys": `# Linear Systems & Decomposition

Dedicated routines for linear systems, decompositions, and structural analysis.

## Functions
- **\`SolveLinear(A, b)\`** — solve $A x = b$ (same as \`A \\ b\`).
- **\`Inverse(A)\`** — $A^{-1}$.
- **\`Determinant(A)\`** — determinant of a square matrix.
- **\`Dot(a, b)\`** — vector dot product.
- **\`Cross(a, b)\`** — cross product of two 3-vectors.
- **\`Norm(v)\`** — Euclidean length of a vector.
- **\`Eigenvalues(A)\`** — eigenvalues of a square matrix.
- **\`Eigen(A)\`** — eigenvalues and eigenvectors.
- **\`LUDecompose(A)\`** — LU decomposition.
- **\`EulerRotate(phi, theta, psi : R)\`** — $3 \\times 3$ rotation matrix from Euler angles (rad, ZYX), inside a \`CALL\`.

> **Control systems:** state-space models build directly on these matrix variables. LTI conversions (\`tf2ss\`, \`ss2tf\`), interconnection (\`series\`, \`parallel\`, \`feedback\`), analysis (\`pole\`, \`zero\`, \`bode\`, \`nyquist\`, \`margin\`, \`step\`, \`impulse\`, \`lsim\`), and controller design (\`lqr\`, \`place\`, \`pidtune\`) are documented under *Control Systems & Symbolic CAS*.

## Example: solve a 3×3 system
\`\`\`
A[1:3, 1:3] = [2, 1, -1; -3, -1, 2; -2, 1, 2]
b[1:3] = [8, -11, -3]
x[1:3] = SolveLinear(A[1:3,1:3], b[1:3])
\`\`\`

[Related: matrices-decl, symbolic-cas, ref-index]`,
  "lang-overview": `The frees language is small and declarative: equations are constraints, names are case-insensitive, and everything is computed in SI. These pages cover the grammar and the everyday building blocks — variables and the guesses that make nonlinear solves converge, units, arrays, complex numbers, strings, and the differentiable math functions. Start with *Equation Syntax & Rules* if you are new; the function pages list the most-used calls and link to the per-symbol Reference for full signatures.`,
  "matrix-overview": `frees uses an array-language-style syntax for matrices and vectors. Declare a shape with a slice suffix, then add, multiply, transpose, or solve linear systems with the standard operators. For heavy numerics there are low-level OpenBLAS primitives and higher-level decompositions (LU, eigenvalues). Transfer-function coefficient arrays for control work are just vectors — see *Dynamic Systems & Control*.`,
  "prog-overview": `When a model repeats or grows, factor it out. \`FUNCTION\` and \`PROCEDURE\` blocks add reusable, imperative-bodied routines; \`MODULE\` encapsulates a whole equation subsystem you can instantiate many times. \`TABLE\` blocks hold tabulated data callable like a function, and the lookup/interpolation and parametric-table accessors read that data back into a solve.`,
  "fluids-overview": `frees ships high-precision property data so you never hand-look-up a state. Real fluids (water, refrigerants, ammonia, …) are computed in the browser by rustprop, a pure-Rust port of CoolProp 8.0.0; ideal-gas species use NASA polynomials; \`AirH2O\` handles humid air from three coordinates; and a built-in database carries bulk properties for common solids. Every property function returns SI base units. Group a circuit's state points with a \`STATE TABLE\` to isolate fluids and overlay cycles on property charts.`,
  "solving-overview": `How frees actually solves — and what to do when it doesn't. These pages explain the pipeline (Tarjan blocking, then Newton's method per block), the guesses and bounds that make nonlinear systems converge, and a methodical debugging workflow for solves that stall. The same solved state powers two system-level analyses: first-order **uncertainty propagation** (\`val ± unc\` on every result) and **optimization** — parametric sweeps, single-objective search, and NSGA-II Pareto fronts.`,
  "dynamics-overview": `Models that move. A \`DYNAMIC\` block integrates coupled, stiff, even event-driven ODE/DAE systems in time; \`LINEARIZE\` extracts state-space matrices about an operating point; and the control suite takes it from there — transfer functions, frequency response (Bode, Nyquist), pole placement, LQR, and PID tuning, with figures declared in code via \`PLOT\`. The symbolic CAS pages cover the Laplace-domain algebra that backs the control work.`,
  "components-overview": `Model whole systems, not just equations: instantiate parameterized **components** (pumps, pipes, heat exchangers, resistors, gears, cooling coils, signal blocks — ~295 shipped), connect their ports, and frees expands the network into ordinary scalar equations for the same solver. The modeling is **acausal** (no inputs or outputs — fix any consistent boundary values), spans five physical domains plus a causal **signal** domain for command wires and six fluid families with strict cross-wiring guards, selects physics fidelity per component with \`model$\` variants, and turns the *same wiring* into a steady operating point or a transient. Start with *Your First Component Network*.`,
  "deploy-overview": `frees is a client–server system you can run anywhere Docker runs. These pages explain the asynchronous compute model (API → queue → compute workers → job store) and why it makes solves robust and scalable, document the REST API so scripts can drive frees directly, and walk through both deployment paths: local Docker via \`frees.sh\`, and Railway (or any container platform) with the hard-won production configuration already baked in.`,
  "tools-overview": `These are the tools around the editor that make modeling faster: a dockable **REPL** console that evaluates expressions against the last solved session (with the full \`CALL\` library and symbolic CAS), the **keyboard shortcuts** for Solve/Check, the **Markdown report** system that weaves live values and plots into a formatted document, and the **Graph Digitizer & Curve Fit** tools that turn a chart image or a table into a fitted equation.`,
  "functions": `# Custom Functions & Procedures

Most of your model is declarative — equations in any order, solved simultaneously. \`FUNCTION\` and \`PROCEDURE\` are for the parts that need **sequential, imperative** logic (loops, conditionals, step-by-step algorithms). Inside them you use \`:=\` for assignment, just like Python or other array languages.

## Functions
A \`FUNCTION\` returns one or more values. Assign the return value(s) with \`:=\`.
- **Single output** — assign the function's own name:
\`\`\`
FUNCTION poly_fit(x)
  poly_fit := 0.5 * x^2 + 2 * x + 1
END

y = poly_fit(3)          { y = 9.5 }
\`\`\`
- **Multiple outputs** — declare them in brackets in the header (array-language-style):
\`\`\`
FUNCTION [q, r] = DivMod(a, b)
  q := trunc(a / b)
  r := mod(a, b)
END

[quotient, remainder] = DivMod(17, 5)   { 3, 2 }
\`\`\`
Discard an output you don't need with \`~\`, or simply leave off trailing outputs:
\`\`\`
[quotient, ~] = DivMod(17, 5)   { quotient only }
\`\`\`
The same \`[ … ] = name( … )\` destructuring works for built-in multi-output \`CALL\` functions too — e.g. \`[A, B, C, D] = tf2ss(num, den)\`. See *Control Systems & Symbolic CAS → Multi-Output Functions*.

## Procedures
A \`PROCEDURE\` is the same idea with inputs and outputs separated by a colon. Call it with \`CALL\`:
\`\`\`
PROCEDURE heat_transfer(T1, T2 : Q_dot)
  Q_dot := 0.8 * 12 * (T1 - T2) / 0.25
END

CALL heat_transfer(100, 20 : heat_loss)
\`\`\`

## Control flow inside functions & procedures
Sequential structures work inside function/procedure bodies (not in the declarative top level):
- **Conditional:** \`IF condition THEN ... ELSE ... END\`
- **While:** \`WHILE condition DO ... END\`
- **Repeat:** \`REPEAT ... UNTIL condition\`

> **Declarative vs. imperative:** the top-level solver reorders your equations freely, so \`x = y + 2\` and \`y = x - 2\` are equivalent there. Inside a \`FUNCTION\`/\`PROCEDURE\`, order matters and \`:=\` is a one-way assignment — read it left-to-right like a normal program.

[Related: modules, symbolic-cas, arrays]`,
  "tables-code": `# Custom Tables (TABLE)

Define a lookup table inline with a \`TABLE\` block. Once compiled it is registered as a callable function — call it like \`tname(x)\` to interpolate.

## Syntax
Column 1 is the independent (x) column. Annotate the input and output units in the header and frees propagates them to anything computed from the table:
\`\`\`
{ Pressure [Pa] vs flow rate [m^3/s] }
TABLE pump_curve(flow [m^3/s]) [Pa]
  0.0       50000
  0.001     45000
  0.002     32000
  0.003     0
END

{ Linear interpolation, with [Pa] units carried to dP }
dP = pump_curve(0.0015)
\`\`\`
For a curve family (a table parameterised by a second variable, e.g. an engine map), add \`: param = v1, v2, …\` after the x column and call it as \`tname(x, y)\` for bilinear interpolation — see *Lookup Tables & Interpolation*.

[Related: lookup-tables, digitizer-fit, table-accessors]`,
  "lookup-tables": `# Lookup Tables & Interpolation

frees provides functions to query, search, and interpolate a named \`TABLE\` block. In a TABLE, **column 1 is the x axis** and each further column is a y/curve column. The simplest way to interpolate is to call the table like a function — \`tname(x)\` (1-D) or \`tname(x, y)\` (bilinear across a curve family). The functions below are the classic-solver-compatible equivalents.

## Interpolation functions
- **\`Interpolate('tname', x)\`** — piecewise-linear interpolation at \`x\` (same as \`tname(x)\`).
- **\`Interpolate1('tname', x)\`** — cubic-spline interpolation (falls back to linear below 3 points).
- **\`Interpolate2D('tname', x, y)\`** — bilinear 2-D interpolation across a curve family (same as \`tname(x, y)\`).
- **\`Differentiate('tname', y_col, x_col, x_val)\`** — numerical $dy/dx$ at $x_{val}$ (finite difference).
- **\`Differentiate1('tname', y_col, x_col, x_val)\`** — cubic-spline derivative.

## Lookup functions
- **\`Lookup('tname', row, col)\`** — cell value by 1-based row/column.
- **\`LookupRow('tname', col, val)\`** — fractional 1-based row where \`col\` crosses \`val\`.
- **\`NLookupRows('tname')\`** — number of data rows.

## 2-D engine-map example
\`\`\`
TABLE bsfc(rpm : load = 0.25, 0.50, 1.0)
  1000   320   300   290
  3000   280   260   250
  5000   300   270   255
END

g_per_kWh = Interpolate2D('bsfc', 2500, 0.6)   { same as bsfc(2500, 0.6) }
\`\`\`

## Tables you don't type
A Function Table built in the **Tables** window is callable exactly like a \`TABLE\` block, so measured or swept data reaches the equations without being retyped:

- **Import CSV…** (the \`+\` menu in the Tables window) reads a \`.csv\` **in your browser** — nothing is uploaded. Pick the x column, the y column and a function name; blank and non-numeric cells are skipped and counted, duplicate x values keep the first row, and a recording longer than the 5 000-row table cap is thinned uniformly (first and last point always kept).
- **Create function…** does the same for two columns of a solved parametric or \`DYNAMIC\` table, optionally with a family-parameter column for a 2-D \`name(x, param)\`.
- The **Graph Digitizer** exports its digitized points the same way.

One rule holds for all three: if the document already defines a \`TABLE\` with that name, **the document's table wins** when the equations are solved — a GUI table never overrides it.

## What wins, and what a file carries

| Input | Origin | Precedence |
| --- | --- | --- |
| \`TABLE\` / \`FUNCTION\` in the document | equation text | **Wins** over a GUI table of the same name |
| GUI Function Table, CSV import, digitizer map | Tables window / project file | Used only when the document does not define that name |
| \`GUESS\` in the document | equation text | **Wins** over Variable Information on conflict |
| Variable Information | project file | Fills in where the text is silent |
| Workspace sliders | project file | Applied as overrides on each solve (last write wins among overrides) |
| REPL assignments | this session only | Override the solved workspace until \`clear\`; not in Export equation text |

**Save project** writes a \`.frees\` JSON file: equation text plus Variable Information, GUI tables, maps, plots, schematic layout, sliders, and opaque legacy slices (spreadsheets / analyzer sessions) so a reload reproduces the effective model.

**Export equation text** writes only the editor document. Anything in the table above that is not in the text is listed in the export notice — those inputs will not travel with a copy-paste of the equations.

[Related: tables-code, table-accessors, digitizer-fit]`,
  "table-accessors": `# Table Accessors & Aggregates

Query cells or statistical summaries of the active **Parametric Table**. These are computed once per table solve and are identical in every row — handy for reporting a cycle total or average alongside each run.

## Accessor functions
- **\`TableValue(run, col)\`** — a cell value in the parametric table.
- **\`TableRun#()\`** — the current run index (1-based).
- **\`NParametricRuns()\`** — total configured runs.
- **\`TableSum('col')\` / \`TableAvg('col')\`** — sum / average of a column.
- **\`TableMin('col')\` / \`TableMax('col')\`** — minimum / maximum.
- **\`TableStdDev('col')\`** — standard deviation.
- **\`IntegralValue('y_col', 'x_col')\`** — trapezoid integral of one column vs. another.

## Example
\`\`\`
{ Whole-cycle energy from a speed-sweep table }
E_total = IntegralValue('P', 't')      { trapezoid integral of power over time }
P_avg   = TableAvg('P')                { mean power, same in every row }
current_index = TableRun#()
\`\`\`

[Related: optimization, lookup-tables, plot-code]`,
  "modules": `# Modular Submodels (MODULE)

A \`MODULE\` is a reusable **declarative** sub-system — a named bag of equations solved simultaneously with the rest of your model. Unlike a \`FUNCTION\`, a module's equations can be solved in **either direction**: a variable you pass in as an output one call can be passed in as an input the next.

## Why use a module?
- Encapsulate a recurring sub-model (a heat exchanger, a pipe segment, a pump) once and \`CALL\` it many times.
- Reuse the same equations whether you're sizing (unknown is an output) or rating (unknown is an input).

## Example
\`\`\`
MODULE pipe_flow(D, Q : dP)
  V  = Q / (pi# / 4 * D^2)
  dP = 0.02 * (100 / D) * (1000 * V^2 / 2)
END

CALL pipe_flow(D1, Q1 : dP1)     { rating:  find dP1 from D1, Q1 }
CALL pipe_flow(D2, Q2 : dP2)     { sizing:  find Q2  from D2, dP2 }
\`\`\`
Both calls use the *same* module — frees figures out which variable is unknown in each.

> **Module vs. function:** a \`MODULE\` is essentially a multi-output \`FUNCTION\` whose body is **equations** (\`=\`, solved in any direction) instead of sequential assignments (\`:=\`, one-way). The bracket call form works here too: \`[dP1] = pipe_flow(D1, Q1)\`.

[Related: functions, prog-overview, arrays]`,
  "symbolic-cas": `# Control Systems & Symbolic CAS

frees brings control-toolbox-style workflows in as native, order-independent equations: LTI modeling and conversions, system interconnection, poles/zeros and stability margins, Bode/Nyquist frequency response, step/impulse/forced time response, and state-feedback/PID controller design. Underneath, two engines meet at the \`num\`/\`den\` coefficient arrays — an embedded **Symja** computer-algebra system (CAS) for symbolic work, and Apache Commons Math for numeric analysis (companion-matrix eigenvalues, Riccati via the matrix sign function) that stays robust on high-order, floating-point systems.

This page starts with the symbolic CAS layer (symbolic identities and Laplace partial fractions), then covers the LTI model representations and every control-systems \`CALL\` function.

## Symbolic identities

frees can solve **symbolic identities** — equations that must hold for *all* values of an independent variable — using the embedded CAS. The classic use is decomposing a Laplace transfer function into partial fractions and reading off the residues, which then appear in the Solution window like any other variable.

## Declaring a symbolic variable

Use \`SYMBOLIC\` to mark one or more independent variables (for control work this is usually the Laplace variable \`s\`):

\`\`\`
SYMBOLIC s
\`\`\`

A \`SYMBOLIC\` variable is **not** solved for. Instead, any equation that contains it is treated as an identity: frees brings both sides over a common denominator, requires every power of the variable to match, and solves the resulting system for the remaining unknown coefficients.

## Partial-fraction decomposition

Write the decomposition you want as an ordinary equation, naming the residues yourself:

\`\`\`
SYMBOLIC s
(s + 3)/(s^2 + 3*s + 2) = A/(s+1) + B/(s+2)
\`\`\`

frees solves this for **A = 2** and **B = -1**. Because you name the residues against the poles you chose, there is never any ambiguity about which residue is which. \`A\` and \`B\` are now ordinary variables — use them in downstream equations (for example, the inverse Laplace transform \`y(t) = A*exp(-t) + B*exp(-2*t)\`).

This partial-fraction route is the recommended way to take an inverse Laplace transform in frees: the residues land in the Solution window as numeric variables, and you write the time-domain reconstruction yourself. (The underlying CAS engine also exposes forward and inverse Laplace transforms directly, used internally to ground these workflows.)

### Automatic residues: residue

When you don't want to write the decomposition template by hand — or the poles aren't obvious — use the numeric \`residue\` dispatch. It factors \`num/den\` automatically and returns the residues, the matching poles, and the scalar direct term as ordinary solved variables:
\`\`\`
num = [1, 3]          # s + 3
den = [1, 3, 2]       # s^2 + 3s + 2
CALL residue(num[1:2], den[1:3] : r_r[1:2], r_i[1:2], p_r[1:2], p_i[1:2], k)
\`\`\`
This yields poles \`p = -2, -1\` with residues \`r = -1, 2\` (and \`k = 0\`), so the inverse Laplace transform is \`y(t) = r_r[1]*exp(p_r[1]*t) + r_r[2]*exp(p_r[2]*t)\`. Residues and poles are complex (real/imag pairs) and sorted together, so \`r_r[i]\`/\`r_i[i]\` always pairs with \`p_r[i]\`/\`p_i[i]\`. A bi-proper \`num/den\` (equal degree) puts its constant term in \`k\`.

**Repeated poles.** Add a sixth output \`ord\` to handle repeated poles — it carries the power \`k\` of each \`A/(s-p)^k\` term:
\`\`\`
num = [1]
den = [1, 2, 1, 0]   # 1 / (s (s+1)^2)
CALL residue(num[1:1], den[1:4] : r_r[1:3], r_i[1:3], p_r[1:3], p_i[1:3], ord[1:3], k)
\`\`\`
gives \`1/s - 1/(s+1) - 1/(s+1)^2\`, i.e. the terms \`(p=-1, ord=1, r=-1)\`, \`(p=-1, ord=2, r=-1)\`, \`(p=0, ord=1, r=1)\`. The time-domain term for order \`k\` is \`r · t^(k-1)/(k-1)! · exp(p·t)\`. The 5-output form raises an error if the system has repeated poles, since they cannot be disambiguated without \`ord\`.

## Transfer functions: tf(num, den)

\`tf(num, den)\` builds a transfer function \`num(s)/den(s)\` from coefficient arrays in **descending powers** (array-language-style): \`[1, 3]\` is \`s + 3\` and \`[1, 3, 2]\` is \`s^2 + 3*s + 2\`. Use it on the left of an identity instead of writing the fraction out:

\`\`\`
SYMBOLIC s
tf([1, 3], [1, 3, 2]) = A/(s+1) + B/(s+2)
\`\`\`

This is equivalent to the explicit form above and also yields \`A = 2\`, \`B = -1\`.

## Notes and limits

- An identity may involve **only one** \`SYMBOLIC\` variable — it is solved with respect to that single independent variable.
- The coefficient arrays passed to \`tf(...)\` must be **constant** (numeric array literals such as \`[1, 3, 2]\`).
- An identity that cannot hold for all values of the symbolic variable (an inconsistent or under-determined decomposition) is reported as an error.
- The residues are solved numerically and shown with their units (dimensionless here) in the Solution window.

## LTI Model Representations

frees represents LTI systems using standard array/matrix variables rather than introducing custom data types. This integrates seamlessly with the existing matrix/array algebra and unit checker.

- **Transfer Function (TF)**: Represented as a pair of coefficient arrays in descending powers (array-language-style). E.g. \`num = [0, 0, 1]\` and \`den = [1, 3, 2]\` represents the system:
  $$G(s) = \\frac{1}{s^2 + 3s + 2}$$
- **State Space (SS)**: Represented as matrices \`A\` ($n \\times n$), \`B\` ($n \\times 1$), \`C\` ($1 \\times n$), and scalar \`D\` ($1 \\times 1$):
  $$\\dot{x} = A x + B u$$
  $$y = C x + D u$$
- **Zero-Pole-Gain (ZPK)**: Represented as real and imaginary components of zeros (\`zr\`, \`zi\`) and poles (\`pr\`, \`pi\`), plus a scalar gain \`k\`:
  $$H(s) = k \\frac{\\prod (s - z_i)}{\\prod (s - p_i)}$$

## Model Conversions

Use \`CALL\` dispatches to convert between representations. The solver automatically registers output shapes so variables can be used as bare names downstream.

> **Output sizes are inferred.** You may write \`CALL\` outputs as **bare names** — frees sizes each output array from the inputs (e.g. \`num\`/\`den\` get length \`n+1\`, a Bode \`mag\` matches \`omega\`). Explicit slices like \`num[1:3]\` still work and are shown in the examples for clarity. Only value-dependent counts need an explicit size: the finite-zero counts of \`zero\`/\`tf2zp\` (e.g. \`zr[1:2]\`) and the \`rlocus\` sweep length. The same control-systems \`CALL\` functions, and the symbolic transforms below, are also available in the **REPL terminal** (see *REPL Terminal & Workspace*), where \`Factor\`, \`Expand\`, \`Apart\`, \`Laplace\`, \`InverseLaplace\`, \`Diff\` and \`Integrate\` run interactively.

## Multi-Output Functions (array-language-style)

Every multi-output \`CALL\` function below also has a **destructuring** form — the same syntax array languages use. Write the outputs in brackets on the left and call the function on the right; it is exactly equivalent to the \`CALL name(inputs : outputs)\` form, with output sizes still inferred:

\`\`\`
{ These two lines are identical }
[A, B, C, D] = tf2ss(num, den)
CALL tf2ss(num, den : A, B, C, D)
\`\`\`

**Discard outputs with \`~\`.** Use a tilde in any slot you don't need — that output is computed but never assigned to a variable, so it never appears in the Solution window:

\`\`\`
[~, ~, V] = svd(M)        { keep only the right singular vectors }
[mag, ~]  = bode(num, den, omega)   { magnitude only }
\`\`\`

**Omit trailing outputs.** You can simply leave off outputs you don't want from the end of the list:

\`\`\`
[A, B] = tf2ss(num, den)   { state and input matrices only — C, D dropped }
\`\`\`

Both \`~\` and trailing omission work in the \`CALL … : …\` colon form too. The discarded values are still solved internally (so the result is identical), they are just hidden from the results. This destructuring form works for user-defined multi-output \`FUNCTION\`s as well — see *Custom Functions & Procedures*.

### 1. State Space to Transfer Function: ss2tf
\`\`\`
CALL ss2tf(A, B, C, D : num[1:3], den[1:3])
\`\`\`

### 2. Transfer Function to State Space: tf2ss
\`\`\`
CALL tf2ss(num, den : A[1:2,1:2], B[1:2], C[1:2], D)
\`\`\`

### 3. Zero-Pole-Gain to Transfer Function: zp2tf
\`\`\`
CALL zp2tf(zr, zi, pr, pi, k : num[1:3], den[1:3])
\`\`\`

### 4. Transfer Function to Zero-Pole-Gain: tf2zp
\`\`\`
CALL tf2zp(num, den : zr[1:1], zi[1:1], pr[1:2], pi[1:2], k)
\`\`\`

## Model Interconnection

Use \`CALL\` dispatches to connect multiple systems in series, parallel, or feedback. Systems can be represented either as transfer functions (numerator and denominator arrays) or as state-space systems (matrices A, B, C, D).

For two systems $G_1(s)$ (of order $n_1$) and $G_2(s)$ (of order $n_2$), the connected system has order $n_1 + n_2$.

### 1. Series Connection: series
Connects $G_1(s)$ and $G_2(s)$ in series: $G(s) = G_1(s) \\cdot G_2(s)$.
\`\`\`
# Transfer Function series:
CALL series(num1, den1, num2, den2 : num[1:3], den[1:3])

# State Space series:
CALL series(A1, B1, C1, D1, A2, B2, C2, D2 : A[1:3,1:3], B[1:3], C[1:3], D)
\`\`\`

### 2. Parallel Connection: parallel
Connects $G_1(s)$ and $G_2(s)$ in parallel: $G(s) = G_1(s) + G_2(s)$.
\`\`\`
# Transfer Function parallel:
CALL parallel(num1, den1, num2, den2 : num[1:3], den[1:3])

# State Space parallel:
CALL parallel(A1, B1, C1, D1, A2, B2, C2, D2 : A[1:3,1:3], B[1:3], C[1:3], D)
\`\`\`

### 3. Feedback Connection: feedback
Connects $G_1(s)$ (forward path) and $G_2(s)$ (feedback path) in a closed loop.
\`\`\`
# Transfer Function feedback:
CALL feedback(num1, den1, num2, den2, sign : num[1:3], den[1:3])

# State Space feedback:
CALL feedback(A1, B1, C1, D1, A2, B2, C2, D2, sign : A[1:3,1:3], B[1:3], C[1:3], D)
\`\`\`
- \`sign\` is optional and defaults to \`1.0\` (negative feedback, i.e., $T(s) = \\frac{G_1}{1 + G_1 G_2}$). Use \`-1.0\` for positive feedback.

## Time Delay Modeling

### 1. Padé Approximation: pade
Generates the numerator and denominator polynomials of a Padé rational approximation of a dead time delay $T_d$ of a given \`order\`. For a Padé approximation of order $m$, the output polynomials have $m+1$ coefficients (descending powers of $s$).
\`\`\`
CALL pade(Td, order : num_delay[1:3], den_delay[1:3])
\`\`\`

## State-Space Analysis & Transformations

Use the following dispatches to compute controllability and observability, verify system rank, and apply similarity transformations.

### 1. Controllability Matrix: ctrb
Computes the controllability matrix $C_{trb} = [B, A B, A^2 B, \\ldots, A^{n-1} B]$ for state-space matrices A ($n \\times n$) and B ($n \\times 1$).
\`\`\`
CALL ctrb(A, B : Co[1:3,1:3])
\`\`\`

### 2. Observability Matrix: obsv
Computes the observability matrix $O_{bsv} = [C; C A; C A^2; \\ldots; C A^{n-1}]$ for state-space matrices A ($n \\times n$) and C ($1 \\times n$).
\`\`\`
CALL obsv(A, C : Ob[1:3,1:3])
\`\`\`

### 3. Matrix Rank: rank
Computes the numerical rank of a matrix $M$ using Singular Value Decomposition (SVD) tolerance comparisons.
\`\`\`
CALL rank(M : r)
\`\`\`

### 4. Similarity Transformation: ss2ss
Applies similarity transformation matrix $P$ to a state-space system (A, B, C, D) such that $x = P z$, yielding transformed matrices $A_n = P^{-1} A P, B_n = P^{-1} B, C_n = C P, D_n = D$.
\`\`\`
CALL ss2ss(A, B, C, D, P : An[1:3,1:3], Bn[1:3], Cn[1:3], Dn)
\`\`\`

## Frequency Analysis & Poles/Zeros

Use the following \`CALL\` dispatches to analyze system poles, zeros, Bode/Nyquist responses, and gain/phase margins.

### 1. Poles: pole
Computes system poles (real part \`pr\`, imaginary part \`pi\`) for a transfer function or a state-space matrix \`A\`.
\`\`\`
CALL pole(num, den : pr[1:2], pi[1:2])
# OR
CALL pole(A : pr[1:2], pi[1:2])
\`\`\`

### 2. Zeros: zero
Computes system zeros (real part \`zr\`, imaginary part \`zi\`) for a transfer function or a state-space system \`(A, B, C, D)\`.
\`\`\`
CALL zero(num, den : zr[1:1], zi[1:1])
# OR
CALL zero(A, B, C, D : zr[1:1], zi[1:1])
\`\`\`

### 3. Bode Frequency Response: bode
Computes magnitude (in dB) and unwrapped phase (in degrees) at a vector of frequencies \`omega\`.
\`\`\`
CALL bode(num, den, omega : mag[1:50], phase[1:50])
# OR
CALL bode(A, B, C, D, omega : mag[1:50], phase[1:50])
\`\`\`

### 4. Nyquist Frequency Response: nyquist
Computes real and imaginary parts at a vector of frequencies \`omega\`.
\`\`\`
CALL nyquist(num, den, omega : real[1:50], imag[1:50])
# OR
CALL nyquist(A, B, C, D, omega : real[1:50], imag[1:50])
\`\`\`

### 5. Gain and Phase Margins: margin
Computes gain margin \`gm\` (in dB), phase margin \`pm\` (in degrees), gain crossover frequency \`w_cg\`, and phase crossover frequency \`w_cp\`.
\`\`\`
CALL margin(num, den : gm, pm, w_cg, w_cp)
# OR
CALL margin(A, B, C, D : gm, pm, w_cg, w_cp)
\`\`\`

### 6. Root Locus Trajectories: rlocus
Computes closed-loop s-plane poles over a swept range of $M$ gain values \`K\`. Outputs are the gain values \`K\` (length \`M\`), and the closed-loop pole real parts \`cpr\` and imaginary parts \`cpi\` (matrices of size \`M x N\` where \`N\` is the order of the open-loop denominator).
\`\`\`
CALL rlocus(num, den : K[1:100], cpr[1:100, 1:4], cpi[1:100, 1:4])
\`\`\`
To plot the root locus s-plane trajectories along with open-loop poles and zeros, use the \`rootlocus\` plot kind:
\`\`\`
PLOT 'Root Locus'
  kind = rootlocus
  pr = cpr
  pi = cpi
  zr = zr  # optional: open-loop zeros real parts
  zi = zi  # optional: open-loop zeros imaginary parts
END
\`\`\`

### 7. Routh-Hurwitz Stability: routh
Runs the Routh-Hurwitz test on a characteristic polynomial \`den\` (descending powers) and reports \`nRHP\`, the number of closed-loop poles in the right half-plane (sign changes in the first column of the Routh array), and \`stable\` (\`1\` when \`nRHP = 0\`, else \`0\`). The two textbook special cases are handled automatically: a zero in the first column is resolved with the epsilon method, and an entire row of zeros is replaced by the derivative of the auxiliary polynomial.
\`\`\`
den = [1, 1, 2, 8]
CALL routh(den[1:4] : nRHP, stable)   # nRHP = 2, stable = 0
\`\`\`
To find the range of a free gain \`K\` for stability, sweep \`K\` over a \`PARAMETRIC\` table and read where \`nRHP\` drops to \`0\`.

### 8. Nichols Chart Data: nichols
Computes the open-loop magnitude (dB) and unwrapped phase (deg) at a vector of frequencies \`omega\` — the same data as \`bode\`, arranged for a Nichols chart.
\`\`\`
CALL nichols(num, den, omega : mag[1:50], phase[1:50])
# OR
CALL nichols(A, B, C, D, omega : mag[1:50], phase[1:50])
\`\`\`
Plot the result with the dedicated **\`nichols\`** plot kind, which draws the locus on the standard Nichols grid (constant closed-loop magnitude *M* and phase *N* contours) with the −1 critical point marked:
\`\`\`
PLOT 'Nichols'
  kind = nichols
  mag = mag
  phase = phase
END
\`\`\`

### 9. Static Error Constants: errorconst
Computes the steady-state (static) error constants for an open-loop \`G(s) = num/den\` given in lowest terms: position \`Kp = lim G(s)\`, velocity \`Kv = lim s·G(s)\`, and acceleration \`Ka = lim s²·G(s)\` as \`s → 0\`. Constants that are infinite for the system type are returned as \`Infinity\`.
\`\`\`
num = [0, 0, 20]
den = [1, 6, 5]            # type 0 system
CALL errorconst(num[1:3], den[1:3] : Kp, Kv, Ka)   # Kp = 4, Kv = 0, Ka = 0
\`\`\`

### 10. Signal-Flow Graphs: mason
Computes the overall transmittance of a scalar signal-flow graph by **Mason's gain formula**. \`G\` is a square node-gain matrix where \`G[i,j]\` is the branch gain from node \`i\` to node \`j\` (\`0\` means no branch); \`source\` and \`sink\` are 1-based node numbers. The solver enumerates the forward paths and loops, builds the graph determinant from the non-touching loop combinations, and returns \`T = Y(sink)/X(source)\`.
\`\`\`
G = [0, 2, 0; 0, 0, 3; 0, 0.5, 0]   # 1->2 (2), 2->3 (3), feedback 3->2 (0.5)
CALL mason(G[1:3,1:3], 1, 3 : T)    # T = 6/(1 - 1.5) = -12
\`\`\`
For transfer-function-valued block diagrams, use the \`series\`/\`parallel\`/\`feedback\` interconnection functions instead, which carry full \`num/den\` polynomials.

## Digital Control (z-domain)

Convert between continuous (s-domain) and discrete (z-domain) transfer functions. Coefficient arrays are in descending powers; outputs are normalized to a monic denominator.

### 1. Continuous to Discrete: c2d
Discretizes \`num/den\` at sample time \`Ts\`. The method is a quoted \`'tustin'\` (bilinear, the default) or \`'zoh'\` (zero-order hold, exact for a piecewise-constant input via the state-space matrix exponential). \`num\` and \`den\` must be the same length (pad the numerator with leading zeros); \`numz\`/\`denz\` share that length.
\`\`\`
num = [0, 2]
den = [1, 2]
Ts = 0.1
CALL c2d(num[1:2], den[1:2], Ts, 'zoh' : numz[1:2], denz[1:2])
\`\`\`

### 2. Discrete to Continuous: d2c
Inverts the bilinear mapping back to continuous time using the inverse Tustin transform (\`'tustin'\`).
\`\`\`
CALL d2c(numz[1:2], denz[1:2], Ts, 'tustin' : num[1:2], den[1:2])
\`\`\`

## Time-Domain Responses

Time responses are integrated through the same tested ODE solver used by \`DYNAMIC\` blocks: a transfer function is converted to controllable canonical state space, the state equation \`x' = A x + B u(t)\` is integrated, and the output \`y = C x + D u\` is sampled at the supplied time vector \`t\`. Each output \`y\` is the same length as \`t\`, so it plots directly with the **xy** plot kind.

### 1. Step Response: step
Unit step response \`y(t)\` (input \`u(t) = 1\`, zero initial state).
\`\`\`
CALL step(num, den, t : y[1:N])
# OR
CALL step(A, B, C, D, t : y[1:N])
\`\`\`

### 2. Impulse Response: impulse
Impulse response \`y(t) = C e^{At} B\` (the direct-feedthrough delta term from a non-zero \`D\` is omitted, as it cannot be represented on a sampled grid).
\`\`\`
CALL impulse(num, den, t : y[1:N])
# OR
CALL impulse(A, B, C, D, t : y[1:N])
\`\`\`

### 3. Forced Response: lsim
Response to an arbitrary input signal \`u\`, linearly interpolated between samples. The input \`u\` and time \`t\` must have the same length \`N\`.
\`\`\`
CALL lsim(num, den, u, t : y[1:N])
# OR
CALL lsim(A, B, C, D, u, t : y[1:N])
\`\`\`

### 4. Transient Response Metrics: stepinfo
Extracts transient response metrics (Rise Time \`Tr\` from 10% to 90%, Peak Time \`Tp\`, Settling Time \`Ts\` using the 2% criterion, and Percent Overshoot \`OS\`) from numerical step response outputs \`y\` at time points \`t\`.
\`\`\`
CALL stepinfo(t, y : Tr, Tp, Ts, OS)
\`\`\`

## Controller Design

State-feedback and PID design solvers. Numeric methods (Riccati / eigenvalues) keep these robust on floating-point, high-order systems.

### 1. LQR Optimal Gain: lqr
Continuous-time linear-quadratic regulator. Returns the optimal state-feedback gain \`K\` that minimizes \`∫ (x'Qx + u'Ru) dt\`, computed by solving the algebraic Riccati equation via the matrix sign function of the Hamiltonian. Single-input form: \`A\` and \`Q\` are \`n×n\`, \`B\` is an \`n\`-vector, \`R\` is a scalar, and \`K\` is an \`n\`-vector. The closed-loop \`A - B K\` is stable.
\`\`\`
CALL lqr(A, B, Q, R : K[1:n])
\`\`\`

### 2. Pole Placement: place
SISO pole placement by Ackermann's formula. Returns the gain \`K\` that relocates the poles of \`A - B K\` to the requested locations, supplied as real/imaginary arrays \`pr\`, \`pi\` (each length \`n\`, complex poles in conjugate pairs).
\`\`\`
CALL place(A, B, pr, pi : K[1:n])
\`\`\`

### 3. PID Auto-Tuning: pidtune
Loop-shaping tuning of a P/PI/PID controller for a SISO plant \`num/den\`. The controller is designed so the open loop crosses over (gain = 1) at frequency \`wc\` with a 60° phase-margin target (a common default). The type is a quoted \`'P'\`, \`'PI'\`, or \`'PID'\`; unused gains are returned as \`0\`. A pure \`P\` controller only sets the crossover — it cannot reshape phase.
\`\`\`
CALL pidtune(num, den, 'PID', wc : Kp, Ki, Kd)
\`\`\`

[Related: matrices-sys, plot-code, dynamic-ode]`,
  "tut-msd": `# Tutorial: Mass–Spring–Damper, from Time Domain to Bode Plot

**The problem.** A 2 kg carriage on a spring (k = 800 N/m) with a viscous damper (c = 8 N·s/m) is released 5 cm from equilibrium. How does it ring down — and what does it look like as a plant, in the frequency domain?

**What you'll use:** \`DYNAMIC\` integration, ODE trajectory accessors, transfer functions, \`CALL bode\`, and \`PLOT\`. Build it in stages and solve after each one — that habit (from *Debugging a Solve*) pins any mistake to the lines you just added.

## Stage 1 — the parameters, and what to expect

\`\`\`run
m = 2 [kg]
k = 800 [N/m]
c = 8 [N-s/m]

wn   = sqrt(k / m)              { natural frequency -> 20 rad/s }
zeta = c / (2 * sqrt(k * m))    { damping ratio     -> 0.1     }
\`\`\`

Solve. With ζ = 0.1 the system is lightly underdamped: expect a slow ring-down at ~20 rad/s. Writing the analytic expectations *first* gives you numbers to check the simulation against — the habit that catches modeling errors.

## Stage 2 — free response in time

Newton's second law, as two first-order states:

\`\`\`
DYNAMIC msd (method = ode45, time = 0 .. 5, points = 500)
  der(x) = v
  der(v) = -(c/m) * v - (k/m) * x
  energy = 0.5*m*v^2 + 0.5*k*x^2     { auxiliary output column }
  x(0) = 0.05
  v(0) = 0
END
\`\`\`

Solve, open the **Tables** tab to see the ODE table, select \`time\` and \`x\`, and click **Plot curve**: a decaying oscillation. The \`energy\` column decays monotonically — a physical sanity check the plot gives you for free.

## Stage 3 — read the trajectory back

Trajectory accessors pull scalar answers out of the run, back into the analytic solve:

\`\`\`
x_peak  = MaxValue('x')            { should be the 0.05 release }
t_cross = TimeAt('x', 0)           { first zero crossing }
E_final = FinalValue('energy')     { how much energy is left at t = 5 s }
\`\`\`

Check \`t_cross\` against theory: the damped frequency is ωd = ωn·√(1−ζ²) ≈ 19.9 rad/s, so the first zero crossing lands near a quarter period, π/(2·ωd) ≈ 0.079 s.

## Stage 4 — the same plant in the frequency domain

Force-to-displacement, the plant is \`X(s)/F(s) = 1/(m·s² + c·s + k)\`. In frees a transfer function is just its coefficient vectors:

\`\`\`
num = [1]
den = [m, c, k]
omega[1:400] = linspace(0.5, 100, 400)

CALL bode(num, den, omega : mag, phase)

PLOT 'MSD Bode'
  kind = bode
  omega = omega
  mag = mag
  phase = phase
END
\`\`\`

Solve and open the **Plots** panel: the magnitude peaks at the resonance you predicted in Stage 1 (≈ 20 rad/s — for ζ = 0.1 the peak sits within a percent of ωn), and the phase falls through −90° there. One document now holds the physics, the transient, and the frequency response — all consistent because they share \`m\`, \`c\`, \`k\`.

## Pitfalls

- **Name your time axis \`time\`** if any state is named \`t\` or \`T\` — names are case-insensitive, and a collision between the time variable and a state is the classic first mistake (see *Transient / ODE Systems*).
- **No implicit multiplication:** \`2*zeta*wn\`, never \`2 zeta wn\`.
- If you stiffen the damper by orders of magnitude, switch \`method = ode45\` to \`ode23s\`.

## Complete listing

\`\`\`run
{ Mass-spring-damper: ring-down, accessors, and Bode -- Tutorial 1 }
m = 2 [kg]
k = 800 [N/m]
c = 8 [N-s/m]
wn   = sqrt(k / m)
zeta = c / (2 * sqrt(k * m))

DYNAMIC msd (method = ode45, time = 0 .. 5, points = 500)
  der(x) = v
  der(v) = -(c/m) * v - (k/m) * x
  energy = 0.5*m*v^2 + 0.5*k*x^2
  x(0) = 0.05
  v(0) = 0
END

x_peak  = MaxValue('x')
t_cross = TimeAt('x', 0)
E_final = FinalValue('energy')

num = [1]
den = [m, c, k]
omega[1:400] = linspace(0.5, 100, 400)
CALL bode(num, den, omega : mag, phase)

PLOT 'MSD Bode'
  kind = bode
  omega = omega
  mag = mag
  phase = phase
END
\`\`\`

## Go further

- Step and impulse responses: \`CALL step(num, den, t : y)\` on a time vector, plotted with the \`xy\` kind.
- Build the same oscillator from mechanical components (\`TransMass\`, a spring, a damper) and extract its state space with \`LINEARIZE\` — see *From Plant to Controller*.
- Close the loop: pick gains with \`pidtune\` or \`lqr\` and verify with \`pole\` and \`margin\`.

[Related: dynamic-ode, symbolic-cas, comp-linearize]`,
  "tut-coil": `# Tutorial: Psychrometric Analysis of an AC Cooling Coil

**The problem.** An air handler draws 1.5 kg/s of dry air at 30 °C, 50 % relative humidity, and must deliver it at 12 °C, saturated. How much cooling does the coil need, how much of it is latent, and how much water condenses out?

**What you'll use:** the \`AirH2O\` psychrometric functions, energy and moisture balances — and then the **moist-air component library**, to build the same coil in four lines and see why the component layer exists. Solve after every stage.

## Stage 1 — the inlet state

Humid-air properties need **three** coordinates, one of which is total pressure (see *Psychrometrics*):

\`\`\`
P_atm  = 101325 [Pa]
T_in   = 30 [C]
phi_in = 0.50
mdot   = 1.5 [kg/s]          { dry-air basis }

w_in  = HumRat(AirH2O, T=T_in, P=P_atm, R=phi_in)
h_in  = Enthalpy(AirH2O, T=T_in, P=P_atm, R=phi_in)
T_dew = DewPoint(AirH2O, T=T_in, P=P_atm, R=phi_in)
\`\`\`

Solve: ω ≈ 0.0133 kg/kg, and the dew point lands near 18 °C. The coil surface will be far below that — so this coil dehumidifies, and the moisture balance in Stage 2 is not optional.

## Stage 2 — outlet state and coil duty

The air leaves at 12 °C saturated (relative humidity 1). Balances are on the **dry-air** mass basis, which is why psychrometric enthalpies are per kg of *dry air*:

\`\`\`
T_out = 12 [C]
w_out = HumRat(AirH2O, T=T_out, P=P_atm, R=1)
h_out = Enthalpy(AirH2O, T=T_out, P=P_atm, R=1)

Q_total  = mdot * (h_in - h_out)              { total coil duty, W }
mdot_w   = mdot * (w_in - w_out)              { condensate, kg/s }
Q_latent = mdot * 2.501e6 * (w_in - w_out)    { latent share, W }
Q_sens   = Q_total - Q_latent
SHR      = Q_sens / Q_total                   { sensible heat ratio }
\`\`\`

Solve. Expect a total duty around 45 kW with roughly 17 kW of it latent (SHR ≈ 0.6) and about 25 g/s of condensate — typical numbers for a deeply dehumidifying coil.

## Stage 3 — the same coil, as components

Now rebuild it from the moist-air library:

\`\`\`
MoistAirSource AHU(P=P_atm, T=303.15 [K], W=w_in, mdot=1.5 [kg/s])
CoolingCoil    COIL(Tout=285.15 [K])
MoistAirSink   RET()

connect(AHU.out, COIL.in)
connect(COIL.out, RET.in)

Q_coil     = COIL.Q          { total duty, from the component }
Q_coil_lat = COIL.Q_lat      { latent share }
\`\`\`

Solve: \`COIL.Q\` and \`COIL.Q_lat\` reproduce your Stage-2 numbers. The component carries the same physics you just wrote — saturated outlet, dry-air-basis balances — plus the connector bookkeeping: its ports conserve dry air (Σṁ_da = 0) and carry the humidity ratio \`W\` as a conserved rider (see *Domains & Fluid Families*).

The payoff is what happens next. Hand-written balances grow quadratically as the network grows; components don't:

## Stage 4 — grow it

- **Mixing box:** feed the coil 80 % return air at 26 °C / 55 % RH and 20 % outdoor air at 35 °C / 60 % RH through a \`MixingBox\` — it flow-weights both enthalpy *and* moisture at the junction, the step where hand calculations start sprouting errors.
- **Winter mode:** swap in \`HeatingCoil\` and \`Humidifier\` for the heating season.
- **Reheat:** add a \`HeatingCoil\` after the cooling coil to hit a supply setpoint at the dehumidified moisture level, and read both duties as named outputs.

## Complete listing

\`\`\`run
{ AC cooling coil: psychrometrics by hand, then as components -- Tutorial 2 }
P_atm  = 101325 [Pa]
T_in   = 30 [C]
phi_in = 0.50
mdot   = 1.5 [kg/s]

w_in  = HumRat(AirH2O, T=T_in, P=P_atm, R=phi_in)
h_in  = Enthalpy(AirH2O, T=T_in, P=P_atm, R=phi_in)
T_dew = DewPoint(AirH2O, T=T_in, P=P_atm, R=phi_in)

T_out = 12 [C]
w_out = HumRat(AirH2O, T=T_out, P=P_atm, R=1)
h_out = Enthalpy(AirH2O, T=T_out, P=P_atm, R=1)

Q_total  = mdot * (h_in - h_out)
mdot_w   = mdot * (w_in - w_out)
Q_latent = mdot * 2.501e6 * (w_in - w_out)
Q_sens   = Q_total - Q_latent
SHR      = Q_sens / Q_total

{ The same coil from the moist-air library }
MoistAirSource AHU(P=P_atm, T=303.15 [K], W=w_in, mdot=1.5 [kg/s])
CoolingCoil    COIL(Tout=285.15 [K])
MoistAirSink   RET()
connect(AHU.out, COIL.in)
connect(COIL.out, RET.in)

Q_coil     = COIL.Q
Q_coil_lat = COIL.Q_lat
\`\`\`

## Pitfalls

- **Every \`AirH2O\` call needs three coordinates including \`P\`** — two won't resolve, and inside a query \`T\` means *dry-bulb* (wet-bulb is the \`B\` coordinate).
- **Component temperatures are SI:** parameters like \`Tout=285.15 [K]\` — annotate a Celsius input on a plain variable and pass the variable if you prefer to think in °C.
- **Don't equate moist-air enthalpy across a coil that condenses** — water leaves the stream; that is exactly what the \`W\` bookkeeping is for.

[Related: humidair, comp-domains, comp-first-network]`,
  "tut-rlc": `# Tutorial: Frequency Response of an RLC Filter

**The problem.** A series RLC circuit (R = 220 Ω, L = 0.1 H, C = 1 µF) driven by a 5 V source, with the output taken across the capacitor, is a second-order low-pass filter. What does it pass, what does it reject, and how peaked is it?

**What you'll use:** phasor (impedance) analysis with plain algebra, then the transfer-function route with \`CALL bode\` — the same circuit two ways, so you can check one against the other.

## Stage 1 — the numbers that shape the response

\`\`\`run
R = 220 [ohm]
L = 0.1 [H]
C = 1e-6 [F]

w0 = 1 / sqrt(L * C)         { resonance -> 3162 rad/s (~503 Hz) }
Q  = w0 * L / R              { quality factor -> ~1.44 }
\`\`\`

Solve. A \`Q\` above 1/√2 means the magnitude response will show a resonant peak just below ω₀ before rolling off at −40 dB/decade — worth knowing *before* you plot.

## Stage 2 — phasor analysis at one frequency

At a single frequency, the circuit is an impedance divider. The reactances and the divider work out with ordinary real algebra:

\`\`\`
f  = 200 [Hz]
w  = 2 * pi# * f
X_L = w * L                       { inductive reactance }
X_C = 1 / (w * C)                 { capacitive reactance }
Z_mag = sqrt(R^2 + (X_L - X_C)^2) { series impedance magnitude }
phi   = arctan((X_L - X_C) / R)   { impedance angle, rad }

V_s   = 5 [V]
I_mag = V_s / Z_mag
V_out = I_mag * X_C               { amplitude across the capacitor }
gain  = V_out / V_s
\`\`\`

At 200 Hz (well below resonance) the gain should come out near 1 — the filter passes it. Re-solve with \`f = 2000 [Hz]\` and the gain collapses; that is the roll-off, one point at a time. (frees also has native complex variables — the \`_r\`/\`_i\` pair mechanism in *Complex Numbers* — but for a single divider, real algebra is the shortest path.)

## Stage 3 — the whole response at once

Point-by-point is how you check; the transfer function is how you *see*. For the output across \`C\`:

$$ H(s) = \\frac{1}{L C\\,s^2 + R C\\,s + 1} $$

\`\`\`
num = [1]
den = [L * C, R * C, 1]
omega[1:400] = linspace(100, 30000, 400)
CALL bode(num, den, omega : mag, phase)

PLOT 'RLC Low-Pass Bode'
  kind = bode
  omega = omega
  mag = mag
  phase = phase
END
\`\`\`

Solve and open the **Plots** panel: flat passband, the modest \`Q\` peak near 3162 rad/s, then −40 dB/decade. Read the Stage-2 spot check against the curve — they must agree, because they are the same physics.

## Complete listing

\`\`\`run
{ Series RLC low-pass filter -- Tutorial 3 }
R = 220 [ohm]
L = 0.1 [H]
C = 1e-6 [F]

w0 = 1 / sqrt(L * C)
Q  = w0 * L / R

f  = 200 [Hz]
w  = 2 * pi# * f
X_L = w * L
X_C = 1 / (w * C)
Z_mag = sqrt(R^2 + (X_L - X_C)^2)
phi   = arctan((X_L - X_C) / R)
V_s   = 5 [V]
I_mag = V_s / Z_mag
V_out = I_mag * X_C
gain  = V_out / V_s

num = [1]
den = [L * C, R * C, 1]
omega[1:400] = linspace(100, 30000, 400)
CALL bode(num, den, omega : mag, phase)

PLOT 'RLC Low-Pass Bode'
  kind = bode
  omega = omega
  mag = mag
  phase = phase
END
\`\`\`

## Pitfalls

- **Radians vs hertz:** \`bode\` and the plot axis work in ω (rad/s); convert with \`w = 2 * pi# * f\` and don't mix the two.
- **No implicit multiplication:** \`2 * pi# * f\`, never \`2 pi# f\`.

## Go further

- Take the output across \`R\` instead — the numerator becomes \`[R*C, 0]\` and the same circuit is a band-pass.
- Build the circuit from electrical components (\`VoltageSource\`, \`Resistor\`, \`Capacitor\`, \`Inductor\`, \`Ground\`) and watch the transient charging response in a \`DYNAMIC\` block.
- Check margins and poles with \`margin\` and \`pole\` as the start of a control loop around the filter.

[Related: complex, symbolic-cas, plot-code]`,
  "tut-vccycle": `# Tutorial: A Refrigeration Cycle with Real Uncertainty

**The problem.** An R134a vapor-compression cycle evaporates at −10 °C and condenses at 40 °C with a 72 % isentropic compressor. What is the COP — and how well do you actually *know* that COP, given that the two temperatures come from ±0.5 K probes and the efficiency from a ±0.03 datasheet figure?

**What you'll use:** real-fluid property calls at the four cycle states, and the **uncertainty propagation engine** (\`UncertaintyOf\`) that turns instrument specs into error bars on the result — the calculation every test report needs and almost nobody does by hand.

## Stage 1 — the four states

Work around the loop, one state per pair of lines. Saturated states are identified by quality \`x\` plus one of \`T\` or \`P\` — never both (see *Debugging a Solve*):

\`\`\`run
T_evap = 263.15 [K]     { -10 C }
T_cond = 313.15 [K]     {  40 C }
eta_c  = 0.72

{ 1: saturated vapor leaving the evaporator }
P1 = P_sat(R134a, T=T_evap)
h1 = Enthalpy(R134a, T=T_evap, x=1)
s1 = Entropy(R134a, T=T_evap, x=1)

{ 2: compressor discharge at condenser pressure }
P2  = P_sat(R134a, T=T_cond)
h2s = Enthalpy(R134a, P=P2, s=s1)        { isentropic ideal }
h2  = h1 + (h2s - h1) / eta_c            { real discharge enthalpy }

{ 3: saturated liquid off the condenser; 4: after the expansion valve }
h3 = Enthalpy(R134a, P=P2, x=0)
h4 = h3                                  { throttling: isenthalpic }
\`\`\`

Solve. Check the pressure ratio \`P2/P1\` (should be near 5) — a sanity anchor before performance numbers.

## Stage 2 — performance

\`\`\`
q_evap = h1 - h4          { refrigerating effect, J/kg }
w_comp = h2 - h1          { specific compressor work, J/kg }
COP    = q_evap / w_comp
\`\`\`

Expect a COP a little above 3 for these conditions.

## Stage 3 — how well do you know it?

Attach the instrument specs directly in code. \`UncertaintyOf(X) = value\` declares the measurement uncertainty of an input; frees then propagates all of them through the whole system (finite-difference Jacobian, root-sum-square) and every computed variable in the Solution panel gains a \`± band\`:

\`\`\`
UncertaintyOf(T_evap) = 0.5
UncertaintyOf(T_cond) = 0.5
UncertaintyOf(eta_c)  = 0.03
\`\`\`

Solve again and read \`COP\` — now shown as \`value ± uncertainty\`. The dominant contributor is the efficiency figure, not the probes: a conclusion you get for free here, and the reason to buy a better datasheet before better thermometers.

## Complete listing

Drag the sliders to move the cycle's boundary temperatures and watch the COP re-solve — the falloff with condensing temperature is the whole story of air-conditioning on a hot day:

\`\`\`run vary=T_evap=253.15:1:273.15 vary=T_cond=303.15:1:323.15
{ R134a vapor-compression cycle with uncertainty -- Tutorial 4 }
T_evap = 263.15 [K]     { -10 C }
T_cond = 313.15 [K]     {  40 C }
eta_c  = 0.72

P1 = P_sat(R134a, T=T_evap)
h1 = Enthalpy(R134a, T=T_evap, x=1)
s1 = Entropy(R134a, T=T_evap, x=1)

P2  = P_sat(R134a, T=T_cond)
h2s = Enthalpy(R134a, P=P2, s=s1)
h2  = h1 + (h2s - h1) / eta_c

h3 = Enthalpy(R134a, P=P2, x=0)
h4 = h3

q_evap = h1 - h4
w_comp = h2 - h1
COP    = q_evap / w_comp
PR     = P2 / P1

UncertaintyOf(T_evap) = 0.5
UncertaintyOf(T_cond) = 0.5
UncertaintyOf(eta_c)  = 0.03
\`\`\`

## Pitfalls

- **Inside the dome, \`T\` and \`P\` are not independent** — identify saturated states with \`x\` plus one of them, or the solve stalls on a singular system.
- **Enthalpies are absolute J/kg** in SI — differences (\`h2 - h1\`) are what carry meaning, not the raw values.
- \`UncertaintyOf\` values are in the variable's **SI unit** (kelvin for the temperatures here); a relative spec must be converted first.

## Go further

- Overlay the cycle on a P-h chart: group the states with a \`STATE TABLE\` and add a \`PLOT\` of kind \`property\` (see *Fluid State Tables*).
- Rebuild the loop from two-phase components (\`TwoPhaseCompressor\`, \`TwoPhaseCondenserFloat\`, \`TwoPhaseEvaporatorUA\`) and let the pressures float with the boundary conditions.
- Sweep \`T_cond\` with a \`PARAMETRIC\` table to see the COP fall as ambient rises.

[Related: thermo, uncertainty, state-tables]`,
  "tut-pump": `# Tutorial: Pump Selection from a Manufacturer's Curve

**The problem.** A cooling loop needs water lifted 18 m through a piping run with known friction. The candidate pump's head curve exists only as a picture in a datasheet. Find the operating point, and the shaft power it implies.

**What you'll use:** the **Graph Digitizer** to turn the datasheet picture into numbers, a unit-annotated \`TABLE\` as the pump model, and an implicit solve for the intersection with the system curve — the everyday workflow of turning *vendor paper* into *engineering numbers*.

## Stage 1 — digitize the curve

Open the **Graph Digitizer** (left toolbar), load the datasheet image, calibrate the axes with two known points on each, and click along the head curve (see *Graph Digitizer & Curve Fit* for the full workflow). Export the points, then paste them into a \`TABLE\` block — column 1 is flow, the header annotates the units, and the table becomes a callable function:

\`\`\`
TABLE pump_curve(flow [m^3/s]) [Pa]
  0.000   520000
  0.010   505000
  0.020   470000
  0.030   415000
  0.040   340000
  0.050   245000
  0.060   130000
END
\`\`\`

Anything computed from the \`pump_curve\` table now carries pascals, because the header says so.

## Stage 2 — the system curve

The circuit resists flow with a static head plus friction that grows with the square of flow:

\`\`\`
rho = 998 [kg/m^3]
g   = 9.81 [m/s^2]
H_static  = 18 [m]
dP_static = rho * g * H_static     { ~176 kPa of static lift }
K = 1.1e8                          { friction coefficient, Pa/(m^3/s)^2 }
\`\`\`

(\`K\` comes from your piping model — the Darcy losses of *Your First Component Network*, or a measured pressure drop at a known flow.)

## Stage 3 — the operating point

The pump runs where the curves cross. In frees that is one declarative line — the equation *is* the intersection:

\`\`\`
pump_curve(V_op) = dP_static + K * V_op^2
\`\`\`

No rearranging, no iteration loop of your own: the solver finds \`V_op\` (expect ≈ 0.039 m³/s). From there the engineering answers are arithmetic:

\`\`\`
dP_op   = dP_static + K * V_op^2
eta_pump = 0.68
P_hyd   = dP_op * V_op             { hydraulic power, W }
P_shaft = P_hyd / eta_pump         { what the motor must supply }
\`\`\`

## Complete listing

\`\`\`run
{ Pump selection from a digitized head curve -- Tutorial 5 }
TABLE pump_curve(flow [m^3/s]) [Pa]
  0.000   520000
  0.010   505000
  0.020   470000
  0.030   415000
  0.040   340000
  0.050   245000
  0.060   130000
END

rho = 998 [kg/m^3]
g   = 9.81 [m/s^2]
H_static  = 18 [m]
dP_static = rho * g * H_static
K = 1.1e8

pump_curve(V_op) = dP_static + K * V_op^2

dP_op    = dP_static + K * V_op^2
eta_pump = 0.68
P_hyd    = dP_op * V_op
P_shaft  = P_hyd / eta_pump
\`\`\`

## Pitfalls

- **Digitize past the region you expect to operate in** — interpolation is honest, extrapolation off the end of a table is not.
- **If the intersection solve stalls**, set a guess for \`V_op\` near the middle of the table's flow range in Variable Info (\`Ctrl + I\`) — an intersection far from the default guess is the classic case for one.
- **Keep the table in SI-consistent columns** (or annotate the units, as here) so the system-curve pascals and the table pascals actually meet.

## Go further

- Fit a quadratic to the digitized points with the **Curve Fit** panel and compare the smooth fit against the raw table.
- Sweep \`H_static\` with a \`PARAMETRIC\` table to see the operating point walk down the curve.
- Wrap the loop in components — \`Pump\` and \`Pipe\` — and let the network compute \`K\` from geometry instead of assuming it.

[Related: digitizer-fit, tables-code, optimization]`,
  "verification": `# Verification Suite

Engineers should not have to take a solver's word for it. Every case on this page ships in the repository as a test fixture (\`backend/core/src/test/resources/validation/\`) and runs as part of the backend test suite **on every commit** — the values below are enforced by CI, not curated by hand. Each fixture's header states its **basis**: the closed-form derivation, exact arithmetic, or public-standard table value the expectation rests on, so every number can be audited without trusting frees itself. Property-model comparisons are deliberately excluded so no expectation depends on the property backend.

Reproduce locally:

\`\`\`text
cd backend && ./gradlew :core:test --tests "com.frees.backend.core.ValidationSuiteTest"
\`\`\`

## Nonlinear algebra

| Case | Verified result | Basis |
| --- | --- | --- |
| Coupled power-ratio pair | x = 4.6940124, y = 3.8021744 (±1e-6) | Direct substitution into x² + y³ = 77 with y = x/1.23456 |
| Monotone cubic | x = 2 exactly | 8 + 2 = 10; the root is unique (derivative 3x² + 1 > 0) |
| Transcendental x·eˣ = 2e² | x = 2 exactly | x·eˣ strictly increasing for x > −1 |

## Thermodynamics

| Case | Verified result | Basis |
| --- | --- | --- |
| Carnot efficiency | η = 0.5 exactly | 1 − 300/600 |
| Air-standard Otto cycle | η = 0.5647247 (±1e-6) | 1 − 8^(−0.4); 8^0.4 = 2^1.2 = 2.2973967 |
| Isentropic compression | T₂ = 579.209 K (±0.01) | 300 · 10^(1/3.5) |
| Ideal Brayton cycle | η = 0.4820525 (±1e-6) | 1 − 10^(−1/3.5) |
| Isothermal expansion work | W = 59 679.97 J (±0.05) | 1 · 287 · 300 · ln 2 |

## Heat transfer

| Case | Verified result | Basis |
| --- | --- | --- |
| Lumped-capacitance cooling | T = 360.653 K (±1e-3) | τ = mc/(hA) = 200 s; e^(−0.5) = 0.60653066 |
| Straight-fin efficiency | η = 0.48201379 (±1e-6) | tanh(2)/2 |
| Parallel-plate radiation | q = 13 121.25 W/m² (±0.05) | Exact fourth powers; resistance denominator 1.5 |
| Critical insulation radius | r = 0.02 m exactly | k/h |
| Counterflow ε-NTU | ε = 0.7746003 (±1e-5) | Closed form at NTU = 2, Cr = 0.5 |

## Fluid mechanics & atmosphere

| Case | Verified result | Basis |
| --- | --- | --- |
| Hagen–Poiseuille pressure drop | Δp = 40.743665 Pa (±1e-4) | 1.28e−6 / (π · 1e−8) |
| Reynolds number | Re = 99 800 exactly | 998 · 2 · 0.05 / 0.001 |
| Hydrostatic column | P = 100 959.07 Pa (±0.05) | ρgh, exact multiplication |
| Isentropic flow at Mach 2 | T₀/T = 1.8 exact; P₀/P = 7.824449 (±1e-4) | 1 + 0.2M²; 1.8^3.5 = 5.832·√1.8 |
| Standard atmosphere, 11 km | T = 216.65 K, P = 22 632 Pa, ρ = 0.36392 kg/m³ | U.S. Standard Atmosphere 1976 published tropopause values, against the built-in \`isa_T\`/\`isa_P\`/\`isa_rho\` |

## Dynamics (ODE integration)

| Case | Verified result | Basis |
| --- | --- | --- |
| Exponential decay | y(1) = 0.36787944 (±1e-4) | Exact solution e^(−t) |
| RC step response | V(1) = 6.3212056 V (±1e-4) | 10 · (1 − e^(−1)), τ = RC = 1 s |
| Harmonic oscillator | x(1) = 1 (±2e-3) | Return after exactly one period, ω = 2π |
| Logistic growth | y(5) = 0.9428256 (±1e-4) | Closed form 1/(1 + 9e^(−5)) |

## Control systems

| Case | Verified result | Basis |
| --- | --- | --- |
| Routh–Hurwitz, stable cubic | 0 RHP poles, stable | First column 1, 2, 2.5, 1 — no sign change |
| Routh–Hurwitz, unstable cubic | 2 RHP poles, unstable | Factors as (s+2)(s² − s + 1); pair at Re +0.5 |
| Static error constants (type 0) | Kp = 4, Kv = 0, Ka = 0 exactly | G(0) = 16/4 |
| Tustin discretization of 1/s | [0.05, 0.05]/[1, −1] exactly | (Ts/2)(z+1)/(z−1) at Ts = 0.1 |

## Linear algebra, signals & statistics

| Case | Verified result | Basis |
| --- | --- | --- |
| 2×2 linear system | x = 1, y = 3 exactly | Elimination/Cramer, det = 5 |
| Triangular 5×5 determinant | det = 120 exactly | Diagonal product 1·2·3·4·5 (exercises the runtime LU path) |
| Symmetric 2×2 eigenvalues | λ = 1, 3 exactly | Characteristic roots 2 ± 1 |
| FFT of a unit impulse | Flat unit spectrum exactly | DFT of [1,0,0,0] is 1 in every bin |
| Least squares on collinear points | slope 2, intercept 1, R² = 1 exactly | Points lie exactly on y = 2x + 1 |

## Uncertainty propagation

| Case | Verified result | Basis |
| --- | --- | --- |
| Product RSS | f = 12; σ_f = 0.7211103 (±1e-4) | √(y²σx² + x²σy²) = √0.52 |

## Component networks

| Case | Verified result | Basis |
| --- | --- | --- |
| Resistive voltage divider | V_mid = 5 V exactly | E · R₂/(R₁+R₂) with equal resistors |
| Series conduction chain | T_interface = 333.333 K (±1e-3) | Q = ΔT/(R₁+R₂) = 133.33 W; 400 − 0.5·Q |

## Adding a case

A validation case is one \`.frees\` file: the problem, a \`// BASIS:\` header explaining how the expected value is derived *independently of frees*, and one \`// EXPECT <var> = <value> tol <abs>\` directive per asserted quantity (\`// EXPECT-UNC\` for a propagated uncertainty). Drop the file in \`backend/core/src/test/resources/validation/\` and the suite picks it up automatically — a case with no directive fails, because an unasserted case verifies nothing.

[Related: started, gs-units-check]`,
};
