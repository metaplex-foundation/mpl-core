<h1 align="center">
  Solana Project Template
</h1>
<p align="center">
  A template for vanilla Solana programs and their clients.
</p>
<p align="center">
  <img width="600" alt="Solana Project Template" src="https://github.com/metaplex-foundation/solana-project-template/assets/729235/aebf053a-d6fa-440b-9766-8957e843ec86" />
</p>
<p align="center">
  <a href="https://github.com/metaplex-foundation/solana-project-template/actions/workflows/main.yml"><img src="https://img.shields.io/github/actions/workflow/status/metaplex-foundation/solana-project-template/main.yml?logo=GitHub" /></a>
</p>

## Features

- **Generate IDLs** using [Shank](https://github.com/metaplex-foundation/shank)
- **Generate clients** for one or more programs using [Kinobi](https://github.com/metaplex-foundation/kinobi)
- Configure **local validators** using [Amman](https://github.com/metaplex-foundation/amman)
- **Build, test and lint** programs and clients using GitHub Actions.
- **Publish** your [Umi](https://github.com/metaplex-foundation/umi) JavaScript client and its TypeScript documentation by dispatching a GitHub workflow.
- **Publish** your Rust client SDK to [crates.io](https://crates.io) by dispatching a GitHub workflow.

## Getting started

1. [Use this template](https://github.com/new?template_name=solana-project-template&template_owner=metaplex-foundation) to create a new repository.
2. Open the `init.sh` script and update the following variables.
   ```sh
   NAME="mpl-project-name"
   DESCRIPTION="My project description"
   PUBLIC_KEY="MyProgram1111111111111111111111111111111111"
   ```
3. Run the `init.sh` script to initialize the project. This will find/replace the variable above, rename some files/folders, update the README and, finally, remove the `init.sh` script.
   ```sh
   ./init.sh
   ```
4. [Read the `CONTRIBUTING.md` file](./CONTRIBUTING.md) to learn more about how to use the project.
