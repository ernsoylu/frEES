[Topic: arch-async]
# How a Solve Runs

frees runs **entirely in your browser tab** via WebAssembly, with a background Web Worker compute model. Understanding the execution pipeline explains what you see in the UI — why the Solve button is gated on a green Check, how the UI remains responsive during heavy numerical computations, and how long solves can be cancelled instantly.

## The path of one solve

1. **Editor → Web Worker.** Pressing Solve (F2) packages your document text, stop criteria, and variable info into a message dispatched to a dedicated background Web Worker (`worker.ts`).
2. **Validate & Prepare.** The Web Worker parses the document using the Rust-based parser. If syntax errors or structural mismatches exist, a diagnostic envelope returns immediately without blocking.
3. **Compute.** The Web Worker executes the full engine pipeline in WebAssembly:
   - Expand component networks, matrix literals, and CALL procedures.
   - Run dimensional unit consistency analysis.
   - Perform Dulmage–Mendelsohn / Tarjan decomposition into lower triangular blocks.
   - Solve each block using Newton–Raphson, with interval scanning and multi-start root-finding when "Find all solutions" is enabled.
   - Integrate dynamic systems (`DYNAMIC ... END`) and differential-algebraic equations via IDA / Runge–Kutta when present.
   - Evaluate thermodynamic properties through the integrated `rustprop` backend.
4. **Publish & Render.** The Web Worker posts the structured solution payload back to the main thread. The React frontend updates the Variable Explorer, Diagrams, Plots, and Tables workbooks with zero network round-trip latency.

## Why in-browser Web Workers?

- **Zero backend infrastructure.** There are no remote API servers, no message brokers (RabbitMQ), and no database caches (Redis). Your models and calculations stay private on your machine.
- **Offline capability.** Once the static page and WebAssembly module are loaded, frees functions completely offline without any internet connection.
- **Responsive UI.** Heavy calculations and iterative sweeps run off the main thread in a Web Worker, ensuring smooth 60fps rendering and seamless interaction.
- **Instant cancellation.** If a simulation or stiff ODE takes too long, clicking Stop immediately terminates the Web Worker and resets the compute state safely.

## Check before Solve

Pressing Check (F4) runs validation without solving: syntax parsing, equation-variable matching, degree-of-freedom checking, and dimensional unit verification. Because it requires no numerical iterations, it completes in milliseconds and provides immediate diagnostic feedback in the editor gutter.

[Related: arch-api, deploy-docker]

[Topic: arch-api]
# The Web Worker & CLI Interface

The frees engine exposes a clean, unified JSON boundary between the Rust core and user interfaces. Whether running inside the browser Web Worker or from the terminal command line, the same document produces identical results.

## Headless CLI (`frees-cli`)

For CI/CD pipelines, automated validation, and batch calculations, frees provides a standalone headless binary: `frees-cli`.

```bash
# Solve a document and print solved variables as JSON
frees-cli solve model.frees

# Check syntax and solvability without solving
frees-cli check model.frees

# Solve with full browser request options (guesses, bounds, multiple solutions)
frees-cli solve --request '{"findAllSolutions": true}' model.frees

# Pipe document from stdin
cat model.frees | frees-cli solve
```

## JSON Request & Response Envelope

The solver communicates via structured JSON envelopes:

- **Inputs**: Document source text, `stopCriteria` (tolerances, iteration limits), `variableInfo` (guesses, bounds, display units), and optional `functionTables`.
- **Outputs**: `variables` (display-converted values with units), `blocks` (Tarjan decomposition hierarchy), `residuals` (equation-level convergence), `stats` (iterations, elapsed time), and `solutions` (multiple solutions when requested).

Because the headless CLI uses the exact same `frees_core` engine as the WebAssembly module, scripting workflows perfectly mirror browser behavior.

[Related: arch-async, deploy-docker]

[Topic: deploy-docker]
# Local Development

frees is written in Rust and TypeScript/React. Developing locally requires only standard Rust and Node.js toolchains:

## Prerequisites

- **Rust toolchain**: Managed via `rustup` with the `wasm32-unknown-unknown` target.
- **Node.js**: **Node 22 is required** (Node 20 lacks WebIDL uncloneable features required by Vitest/JSDOM).
- **wasm-pack**: Compiles Rust crates to WebAssembly.

## Building and Running

```bash
# 1. Compile the Rust WebAssembly engine
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg

# 2. Install web dependencies and start the Vite development server
cd web
npm install
npm run dev

# 3. Run test suites
cargo test --workspace -- --skip golden_corpus_parity
npm test
```

The web application is accessible at `http://localhost:5173`.

[Related: deploy-railway, arch-async]

[Topic: deploy-railway]
# Browser Deployment & Static Hosting

Because frees runs entirely client-side, deploying frees requires **no backend containers, databases, or microservices**.

## Static Web Hosting

The entire application compiles into static HTML, JavaScript, CSS, and `.wasm` files. It can be hosted on any static web server or CDN platform:

- **GitHub Pages**
- **Cloudflare Pages**
- **Vercel**
- **Netlify**
- **Nginx / Caddy / Apache**

## Production Build

```bash
# Build WebAssembly package
wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg

# Build static production assets
cd web
npm run build
```

The output in `web/dist/` contains all assets needed for deployment. Serve the directory with any web server configured with standard MIME types (specifically `application/wasm` for `.wasm` files).

[Related: deploy-docker, deploy-health]

[Topic: deploy-health]
# Worker Lifecycle & Deadlines

Because frees executes in the user's browser, execution boundaries and resource safety are handled cooperatively:

## Timeouts and Deadlines

- **Cooperative Deadlines**: For parametric sweeps, optimization loops, and Monte Carlo runs, frees enforces cooperative deadlines (default 120s budget) checked between iterations to prevent runaway computations.
- **Worker Termination**: If an equation system causes an uncooperative infinite loop, clicking the UI's Stop button immediately terminates the Web Worker thread and spawns a fresh worker instance.

## Memory and Bundle Budgets

- **WASM Memory**: WebAssembly linear memory grows dynamically as needed for large systems and matrices, bounded by browser tab memory limits.
- **Binary Size**: The release WebAssembly binary is strictly budgeted and monitored in CI (~3.1 MB uncompressed / ~1.2 MB gzipped), including complete fluid property tables and 295 physical components.

[Related: arch-async, deploy-railway]
