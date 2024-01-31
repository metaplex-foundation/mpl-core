# Contributing to the Rust client

This is a quick guide to help you contribute to the Rust client of Mpl Project Name.

## Getting started

To build and test the Rust client, you can use `cargo`.

```sh
# Build the client
cargo build

# Test the client (requires building the program first)
cargo test-sbf --sbf-out-dir ../../programs/.bin
```

When something changes in the program(s), make sure to run `pnpm generate` in the root directory, to re-generate the clients accordingly.

## Publishing the Rust client

You can publish a new version of the Rust client crate by manually dispatching the "Publish Rust Client" workflow in the GitHub Actions tab of the repository.

![Click on the "Actions" tab, then on the "Publish Rust Client" workflow, then on the "Run workflow" dropdown. Select your options before clicking on the final "Run workflow" button inside the dropdown body.](https://user-images.githubusercontent.com/3642397/235444901-6ee95f30-ed84-4eef-b1c4-8b8474ab82a4.png)

For this to work, some initial setup is required on the repository as explained below.

## Setting up GitHub actions

To publish Rust clients using GitHub actions, we first need the following secret variable to be set up on the repository.

- `CRATES_TOKEN` — An access token that can publish your packages to [crates.io](https://crates.io).
