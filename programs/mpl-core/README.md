# Mpl Core

Digital Assets

## Building

This will build the program and output a `.so` file in a non-comitted `target/deploy` directory which is used by the `config/shank.cjs` configuration file to start a new local validator with the latest changes on the program.

```sh
cargo build-sbf
```

For a deterministic, deploy-ready artifact (the same one used by CI and on-chain releases), use [`solana-verify`](https://github.com/Ellipsis-Labs/solana-verifiable-build) instead:

```sh
pnpm programs:build
```

This runs `solana-verify build --library-name mpl_core_program` inside a Docker container so the resulting `.so` hash matches a remote rebuild of the same commit. See the root [README](../../README.md#verifiable-builds) for the full verified-builds workflow, including how to verify a deployed program with `pnpm programs:verify`.

## Testing

You may run the following command to build the program and run its Rust tests.

```sh
cargo test-sbf
```
