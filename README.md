# Mpl Core

Digital Assets

## Documentation

Read the full documentation [here](https://developers.metaplex.com)

## Programs

This project contains the following programs:

- [Mpl Core](./programs/mpl-core/README.md) `CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d`

To compile the program locally you will need a Rust toolchain matching `RUST_VERSION` in [`.github/.env`](./.github/.env). For deterministic, deploy-ready builds, see [Verifiable Builds](#verifiable-builds) below. `solana-verify` runs the build inside a Docker container whose Rust toolchain is what the on-chain hash is derived from.

## Clients

This project contains the following clients:

- [JavaScript](./clients/js/README.md)
- [Rust](./clients/rust/README.md)

## Verifiable Builds

Releases of `mpl-core` are produced with [`solana-verify`](https://github.com/Ellipsis-Labs/solana-verifiable-build), so the on-chain bytes can be independently re-derived from this repository at the deployed commit. See the [Solana verified-builds guide](https://solana.com/developers/guides/advanced/verified-builds) for background.

### Prerequisites

- [Docker](https://docs.docker.com/engine/install/) (running)
- A Rust toolchain matching `RUST_VERSION` in [`.github/.env`](./.github/.env)
- `solana-verify` matching `SOLANA_VERIFY_VERSION` in [`.github/.env`](./.github/.env):

  ```sh
  SOLANA_VERIFY_VERSION="$(grep '^SOLANA_VERIFY_VERSION=' .github/.env | cut -d '=' -f2)"
  cargo install solana-verify --locked --version "${SOLANA_VERIFY_VERSION}"
  ```

### Building Locally

```sh
pnpm programs:build
```

This wraps `solana-verify build --library-name mpl_core_program` and copies the resulting `.so` to `./programs/.bin/`. The exact same command runs in CI, so the artifact byte-matches what is uploaded to GitHub releases and deployed by `deploy-program.yml`.

### Verifying a Deployed Program

To check that a deployed program ID matches this repository at the current `HEAD`:

```sh
pnpm programs:verify -- --program-id <PROGRAM_ID> --url <RPC_URL>
```

To publish verification data to the on-chain PDA, run the same command with the program upgrade authority keypair and skip the interactive prompt:

```sh
pnpm programs:verify -- \
  --program-id <PROGRAM_ID> \
  --url <RPC_URL> \
  --keypair <UPGRADE_AUTHORITY_KEYPAIR> \
  --skip-prompt
```

Then queue the remote verification worker against the same RPC URL:

```sh
solana-verify remote submit-job \
  --url <RPC_URL> \
  --program-id <PROGRAM_ID> \
  --uploader <UPGRADE_AUTHORITY_ADDRESS>
```

The GitHub Actions [Verify Program](./.github/workflows/verify-program.yml) workflow performs both steps for one-off mainnet verification when its configured keypair is the program upgrade authority, and [Deploy Program](./.github/workflows/deploy-program.yml) performs them automatically after successful `mainnet-beta` direct deploys. Core's current `mainnet-beta` deploy path uses Squads, so publishing the verified-build PDA for that authority must be routed through the Squads transaction flow described in the Solana verified-builds guide.

By default the script reads the repo URL from `git remote get-url origin` (normalizing `git@github.com:` and `ssh://git@github.com/` to `https://`) and the commit hash from `HEAD`. Both can be overridden with `--repo-url` and `--commit-hash`. The `--mount-path` and `--library-name` flags default to `.` and `mpl_core_program` respectively, matching this repo's layout for `solana-verify` v0.4.15. Override them when verifying a different program from the same workspace.

## Contributing

Check out the [Contributing Guide](./CONTRIBUTING.md) the learn more about how to contribute to this project.

## Security

Audit Completed 2024-05-03 by Mad Shield
