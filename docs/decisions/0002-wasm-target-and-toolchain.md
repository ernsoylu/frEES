# D2 — Wasm target and toolchain

**Status:** Decided · **Date:** 2026-07-29 · Supersedes nothing

## Decision

Build for **`wasm32-unknown-unknown` with `wasm-bindgen`/`wasm-pack`**. Do not
use `wasm32-unknown-emscripten`.

## Context

`PLAN.md` §3 posed this as a genuine choice, because it is entangled with D1
(the property backend):

| Target | Upside | Downside |
|---|---|---|
| `wasm32-unknown-unknown` + `wasm-bindgen` | Mature tooling, clean Vite integration, `web-sys`/`js-sys`, small output | CoolProp (a C++ Emscripten artifact) can only be reached across a JS boundary |
| `wasm32-unknown-emscripten` | CoolProp links into the same module, callable via plain `extern "C"` with no boundary cost | `wasm-bindgen` support is poor; you inherit Emscripten's JS glue and build system for the *whole* engine |

The emscripten target only pays for itself if CoolProp sits in the Newton inner
loop. D1's recommendation — precomputed `(P,h)` property tables as the hot path,
with `coolprop.wasm` as a lazily loaded fallback — removes that pressure: the
boundary is crossed on a table miss, not per Jacobian column.

Paying the Emscripten tax across the entire engine to optimise a fallback path
is the wrong trade.

## Verified, not assumed

Installed and confirmed working on this machine:

```
rustc 1.97.1 (8bab26f4f 2026-07-14)
cargo 1.97.1
targets: wasm32-unknown-unknown, x86_64-unknown-linux-gnu
wasm-pack 0.13.1
```

`cargo build -p frees --target wasm32-unknown-unknown` succeeds on the
workspace skeleton. `rust-toolchain.toml` pins the channel and target so a fresh
clone provisions itself.

Note the starting state: the machine had a distro `rustc 1.75.0` from a source
tarball and **no cargo at all**. Anyone reproducing this needs `rustup`, not the
system package.

## Consequences

* `frees-core` carries no `wasm-bindgen` dependency and compiles for both native
  and `wasm32`. The native build runs the parity harness at full speed; a port
  testable only in a browser is a port you cannot trust.
* `crates/frees` is the only crate that knows about JS, and stays thin.
* CoolProp, when it arrives, is reached through a JS import with a Rust-side LRU
  cache in front of it (mirroring the 20k-entry caches already in
  `props/CoolProp.java`).
* **Revisit if** the D1 spike measures the JS boundary dominating solve time for
  representative thermofluid documents. That would be a real signal; nothing
  else should reopen this.
