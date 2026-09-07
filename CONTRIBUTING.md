# Contributing to Mpl Core

This is a quick guide to help you contribute to Mpl Core.

## Getting started

The root folder has a private `package.json` containing a few scripts and JavaScript dependencies that help generate IDLs; clients and start a local validator. First, [ensure you have pnpm installed](https://pnpm.io/installation) and run the following command to install the dependencies.

```sh
pnpm install
```

You will then have access to the following commands.

- `pnpm programs:build` - Build all programs and fetch all dependant programs.
- `pnpm programs:test` - Test all programs.
- `pnpm programs:debug` - Test all programs with logs enabled.
- `pnpm programs:clean` - Clean all built and fetched programs.
- `pnpm programs:coverage` - Generate a code coverage report for all programs (see [Code coverage](#code-coverage)).
- `pnpm clients:rust:test` - Run the Rust client tests.
- `pnpm clients:js:test` - Run the JS client tests.
- `pnpm generate` - Shortcut for `pnpm generate:idls && pnpm generate:clients`.
- `pnpm generate:idls` - Generate IDLs for all programs, as configured in the `configs/shank.cjs` file.
- `pnpm generate:clients` - Generate clients using Kinobi, as configured in the `configs/kinobi.cjs` file.
- `pnpm validator` - Start a local validator using Amman, as configured in the `configs/validator.cjs` file.
- `pnpm validator:debug` - Start a local validator using Amman with logs enabled, as configured in the `configs/validator.cjs` file.
- `pnpm validator:stop` - Stop the local validator.
- `pnpm validator:logs` - Show the logs of the local validator.

## Code coverage

Program coverage is measured with [`cargo llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) (LLVM source-based coverage). Install the tooling once:

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked
```

The script also needs `jq`, like the other program scripts. Then run:

```sh
pnpm programs:coverage
```

This runs the program's unit tests and its Mollusk integration tests with instrumentation and writes, per program, `coverage/<program>/lcov.info`, `coverage/<program>/coverage.json`, `coverage/<program>/summary.txt` and an HTML report at `coverage/<program>/html/index.html`. Set `COVERAGE_MIN_LINES=<percent>` to make the script fail below a line coverage threshold.

Mollusk normally executes the compiled SBF ELF inside the SVM, which coverage instrumentation cannot observe. To make the Mollusk tests count, `cargo llvm-cov` sets `cfg(coverage)`, and the shared harness in `programs/mpl-core/tests/common` then registers the host-compiled program with Mollusk as a native builtin instead of loading the `.so`. The instruction accounts are still serialized with the SBF ABI, CPIs and sysvars go through the runtime, and account changes are committed back the way the BPF loader does it, so the same test code runs in both modes. The harness only changes _how_ the program is executed, not what the tests assert.

You can also opt into native execution outside of coverage runs, for example to run the Mollusk tests without building the SBF binary first:

```sh
MPL_CORE_NATIVE_PROGRAM=1 cargo test --manifest-path programs/mpl-core/Cargo.toml
```

The regular `pnpm programs:test` command keeps testing the real SBF binary, and CI runs both: the "Test Programs" workflow against the ELF and the "Coverage" workflow natively. The coverage workflow publishes a summary table in the job summary, posts (and keeps updating) a coverage comment on same-repository pull requests, and uploads the full report (LCOV, JSON and HTML) as the `coverage-<program>` artifact.

## Managing clients

Each client has its own README with instructions on how to get started. You can find them in the `clients` folder.

- [JavaScript client](./clients/js/README.md)
- [Rust client](./clients/rust/README.md)

In order to generate the clients, run the following command.

```sh
pnpm generate
```

You will need to run `pnpm generate` to re-generate the clients when something changes in the program(s).

## Setting up CI/CD using GitHub actions

Most of the CI/CD should already be set up for you and the `.github/.env` file can be used to tweak the variables of the workflows.

However, the "Publish JS Client" workflow — configured in `.github/workflows/publish-js-client.yml` — requires a few more steps to work. See the [CONTRIBUTING.md file of the JavaScript client](./clients/js/CONTRIBUTING.md#setting-up-github-actions) for more information.

Similarly, the "Publish Rust Client" workflow — configured in `.github/workflows/publish-rust-client.yml` — requires a few more steps to work. See the [CONTRIBUTING.md file of the Rust client](./clients/rust/CONTRIBUTING.md#setting-up-github-actions) for more information.
