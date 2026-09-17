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

This runs the program's unit tests and its Mollusk integration tests with instrumentation and writes, per program, under `coverage/<program>/`:

- `lcov.info`, `coverage.json` and an HTML report at `html/index.html`;
- `summary.txt`, the per-file table;
- `missing.txt`, the same table followed by the uncovered line ranges of every file (`cargo llvm-cov report --show-missing-lines`). Use it to see exactly which lines a new test still misses, and to re-validate the exclusions file (below) at each milestone;
- `gate.json`, the thresholds in effect, the measured totals and whether the gate passed.

The report only counts `programs/<program>/src`: the test harness and the Rust client crate (which the tests pull in as a path dev-dependency for its generated instruction builders) are excluded through `--ignore-filename-regex`. That regex may only exclude test code — whether it lives outside `src` or, as dedicated in-crate test modules, inside it. Program code is never hidden that way, so a line missing from the report is always test code, never behaviour.

### Thresholds and the ratchet rule

`programs/mpl-core/coverage-thresholds.json` is the coverage gate. It holds the minimum percentages, and the checked-in values rise as coverage lands, so read the file for the numbers in force rather than this example of its shape:

```json
{ "lines": 35, "functions": 42, "regions": 32, "allowlist": [] }
```

`coverage.sh` passes these to `cargo llvm-cov report` as `--fail-under-lines`, `--fail-under-functions` and `--fail-under-regions`, so the script (and the "Coverage" workflow) fails when the program is below any of them. Regions are gated as well as lines because they are the stable-toolchain proxy for branch coverage. An optional `file_lines` value adds a per-file floor (`--fail-under-file-lines`, which needs cargo-llvm-cov 0.8.6 or newer — the script checks the installed version and says so before running when `file_lines` is set); files listed in `allowlist` (paths relative to the repository root, each citing an entry in the exclusions file) are exempt from the floor. `COVERAGE_MIN_LINES=<percent>` still works as a one-off override of the `lines` value.

The thresholds ratchet:

- **They only go up.** Mollusk tests are deterministic, so a threshold can sit at `floor(measured) - 1` with no flakiness margin.
- **Raise them with your PR when the headroom exceeds 2 points.** The script prints `ratchet: <metric> coverage is X%, more than 2 points above the Y% threshold; bump coverage-thresholds.json` for any metric in that state; a PR that leaves that warning in place should be asked to bump the file.
- **Lowering a value needs a reviewer-approved note** in [programs/mpl-core/COVERAGE_EXCLUSIONS.md](programs/mpl-core/COVERAGE_EXCLUSIONS.md), the file that records every accepted exclusion (`file:line(s) | symbol | reason | reviewer | date`) and is the only thing that separates the measured number from 100%.

### What the reported percentage includes

Unit tests written as `#[cfg(test)] mod` blocks inside `programs/mpl-core/src` are compiled into the
instrumented binary like any other code, so their own lines count as covered source and lift the
reported percentage above the program's real coverage. At the time this was written the gap is about
1.3 points: 96.97% reported, and 95.64% (8055 of 8422 lines) counting only the lines that existed
before those test modules were added.

The gate measures the reported number, which is consistent as long as everyone reads it the same way.
Two consequences worth knowing:

- Adding in-crate unit tests raises the reported number by itself. When a PR's gain comes mostly from
  new `#[cfg(test)]` code rather than from newly exercised program code, say so in the description.
- If the gap ever grows enough to matter, the fix is to move the in-crate tests into sibling
  `src/**/tests.rs` files included with `#[cfg(test)] #[path = "tests.rs"] mod tests;` and add that
  path to `IGNORE_REGEX` in `configs/scripts/program/coverage.sh`, which excludes them from the report
  without changing where the tests live logically.

### Native execution and the fast loop

Mollusk normally executes the compiled SBF ELF inside the SVM, which coverage instrumentation cannot observe. To make the Mollusk tests count, `cargo llvm-cov` sets `cfg(coverage)`, and the shared harness in `programs/mpl-core/tests/common` then registers the host-compiled program with Mollusk as a native builtin instead of loading the `.so`. The instruction accounts are still serialized with the SBF ABI, CPIs and sysvars go through the runtime, and account changes are committed back the way the BPF loader does it, so the same test code runs in both modes. The harness only changes _how_ the program is executed, not what the tests assert.

You can opt into native execution outside of coverage runs. This is the fast loop while writing tests: no SBF build, and the whole suite runs in well under a second.

```sh
MPL_CORE_NATIVE_PROGRAM=1 cargo test --manifest-path programs/mpl-core/Cargo.toml
```

The regular `pnpm programs:test` command keeps testing the real SBF binary, and CI runs both: the "Test Programs" workflow against the ELF and the "Coverage" workflow natively. The coverage workflow publishes a summary table in the job summary, posts (and keeps updating) a coverage comment on same-repository pull requests, both with the thresholds in effect and whether the gate passed, and uploads the full report (LCOV, JSON, HTML, `missing.txt`) as the `coverage-<program>` artifact.

### Porting the JS suite

The AVA suite in `clients/js/test` is the behavioural specification; the Mollusk integration tests are ported from it into the single `tests/it` crate (`programs/mpl-core/tests/it/main.rs`, one module per JS file, mirroring the `clients/js/test` tree). One crate rather than one file per JS test file, because each `tests/*.rs` file is its own crate that links the whole Solana runtime. Every ported test carries a `/// JS: <file> :: <title>` marker; the layout, naming and assertion translation rules are in [programs/mpl-core/tests/PORTING.md](programs/mpl-core/tests/PORTING.md), together with the list of JS tests that are excluded from the porting target.

The porting table is computed, not hand-maintained:

```sh
./configs/scripts/program/porting-status.sh            # counts plus the pending list
./configs/scripts/program/porting-status.sh --summary  # counts only
./configs/scripts/program/porting-status.sh --strict   # exit 1 when a marker references a JS test that no longer exists
```

The plan for taking the program to full coverage, with the per-file gaps, the tests to add, and the harness work they depend on, is in [docs/coverage-roadmap.md](docs/coverage-roadmap.md).

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
