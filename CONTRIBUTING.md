# Contributing to Mpl Project Name

This is a quick guide to help you contribute to Mpl Project Name.

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
- `pnpm clients:rust:test` -  Run the Rust client tests.
- `pnpm clients:js:test` -  Run the JS client tests.
- `pnpm generate` - Shortcut for `pnpm generate:idls && pnpm generate:clients`.
- `pnpm generate:idls` - Generate IDLs for all programs, as configured in the `configs/shank.cjs` file.
- `pnpm generate:clients` - Generate clients using Kinobi, as configured in the `configs/kinobi.cjs` file.
- `pnpm validator` - Start a local validator using Amman, as configured in the `configs/validator.cjs` file.
- `pnpm validator:debug` - Start a local validator using Amman with logs enabled, as configured in the `configs/validator.cjs` file.
- `pnpm validator:stop` - Stop the local validator.
- `pnpm validator:logs` - Show the logs of the local validator.

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
