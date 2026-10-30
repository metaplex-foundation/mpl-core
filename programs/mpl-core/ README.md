# Mpl Core

Digital Assets

## Building

This will build the program and output a `.so` file in a non-comitted `target/deploy` directory which is used by the `config/shank.cjs` configuration file to start a new local validator with the latest changes on the program.

```sh
cargo build-bpf
```

## Testing

You may run the following command to build the program and run its Rust tests.

```sh
cargo test-bpf
```

The Mollusk integration tests live in the single `tests/it` crate (`tests/it/main.rs`, one module per area) and share the library in `tests/common`: `Fixture` creates assets and collections through the program and keeps them between instructions, `ix` wraps the generated client's instruction builders, `AssetSpec` / `CollectionSpec` lay out accounts by hand for adversarial cases, and `read` / `assert` inspect the results. Run them natively without an SBF toolchain with `MPL_CORE_NATIVE_PROGRAM=1 cargo test`.

## Coverage

To measure which lines the unit tests and the Mollusk tests exercise, run the following command from the repository root (requires `cargo-llvm-cov` and the `llvm-tools-preview` component).

```sh
pnpm programs:coverage
```

Under `cargo llvm-cov` the Mollusk tests execute the host-compiled program through the shared harness in `tests/common`, so the report covers the code they reach. See the root [CONTRIBUTING.md](../../CONTRIBUTING.md#code-coverage) for details.
