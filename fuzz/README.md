# Parser and evaluator fuzzing

These native-only libFuzzer targets exercise the same lexer, parser and scalar
evaluator used by WASM. Any normal `Ok` or structured `Err` is accepted; panics,
sanitizer failures and timeouts fail the run. Panics are deliberately not caught.
Property-based solver tests remain in `crates/frees-core/tests/fuzz_properties.rs`.

From the repository root:

```sh
rustup toolchain install nightly --profile minimal
cargo install cargo-fuzz --version 0.13.2 --locked
cargo +nightly fuzz run parse_document -- -max_total_time=60 -max_len=4096 -timeout=10 -rss_limit_mb=2048
cargo +nightly fuzz run eval_expression -- -max_total_time=60 -max_len=4096 -timeout=10 -rss_limit_mb=2048
```

CI runs both targets for 60 seconds each using the committed lockfile. This is
a smoke check, not exhaustive coverage. For a longer local soak, increase
`-max_total_time` and `-max_len`; the 4096-byte CI limit is a fuzzing budget,
not a product input limit. Curated `.frees` seeds are committed; generated
corpus entries, coverage and crash artifacts are ignored.

Replay a failure with `cargo +nightly fuzz run TARGET fuzz/artifacts/TARGET/FILE`.
Minimize it with `cargo +nightly fuzz tmin TARGET fuzz/artifacts/TARGET/FILE`.
Fix the shared parser/evaluator, then preserve the minimized input as a named
native regression test and a `.frees` corpus seed. Do not filter out the input
or catch its panic to make the fuzzer pass.

The expression target evaluates parsed expressions with `x = 2`, `y = -1` and
an otherwise empty scope. Document-defined functions, full solves and property
backends remain covered by the existing native robustness/property tests.
