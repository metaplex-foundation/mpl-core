# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Essential Commands

### Building and Development
- `pnpm install` - Install workspace dependencies
- `pnpm programs:build` - Build all Rust programs and fetch dependencies
- `pnpm programs:test` - Run Rust program tests
- `pnpm programs:debug` - Run Rust program tests with logs enabled
- `pnpm clients:js:test` - Run JavaScript client tests
- `pnpm clients:rust:test` - Run Rust client tests
- `pnpm generate` - Generate IDLs and clients (shortcut for both commands below)
- `pnpm generate:idls` - Generate IDLs from Rust programs using Shank
- `pnpm generate:clients` - Generate client libraries using Kinobi

### Testing and Quality
- `pnpm lint` - Run linting for both JS client and Rust programs
- `pnpm lint:fix` - Auto-fix linting issues and format code
- `cd clients/js && pnpm test` - Run specific JS client tests
- `cd clients/js && pnpm build` - Build JS client
- `cargo test-bpf` - Run Rust program tests (from program directory)
- `cargo build-bpf` - Build Rust program (from program directory)

### Local Development
- `pnpm validator` - Start local validator with program deployed
- `pnpm validator:debug` - Start local validator with logs
- `pnpm validator:stop` - Stop local validator
- `pnpm validator:logs` - Show validator logs

## Architecture Overview

### Multi-Language Workspace Structure
This repository contains both Rust programs and multiple client libraries:
- **Rust Program**: `programs/mpl-core/` - The on-chain Solana program
- **JavaScript Client**: `clients/js/` - Umi-compatible JS library 
- **Rust Client**: `clients/rust/` - Rust client library
- **Python Client**: `clients/python/` - Python client (minimal)

### Program Architecture (Rust)
The core program follows a modular plugin-based architecture:
- **State Management**: `state/` - Asset, Collection, and other core data structures
- **Plugin System**: `plugins/` - Extensible plugin architecture with three categories:
  - `internal/authority_managed/` - Collection-level plugins (royalties, attributes, etc.)
  - `internal/owner_managed/` - Asset-level plugins (freeze, burn, transfer delegates)
  - `internal/permanent/` - Immutable plugins (edition, permanent delegates)
  - `external/` - External plugin adapters for third-party extensions
- **Processors**: `processor/` - Instruction handlers for all program operations
- **Instructions**: Core operations like create, transfer, burn, plugin management

### Client Generation Workflow
1. Rust program code defines the on-chain interface using Shank annotations
2. `pnpm generate:idls` extracts IDL (Interface Definition Language) files
3. `pnpm generate:clients` uses Kinobi to generate client code from IDLs
4. Generated client code appears in `clients/js/src/generated/`
5. Hand-written client code in `clients/js/src/` provides higher-level APIs

### Key Plugin Types
- **FreezeDelegate**: Allows freezing/unfreezing assets
- **BurnDelegate**: Permits burning assets
- **TransferDelegate**: Enables transfers on behalf of owner
- **Royalties**: Collection-level royalty enforcement
- **Attributes**: On-chain metadata storage
- **MasterEdition**: Print/edition functionality
- **External Adapters**: Third-party plugin integration (Oracle, AppData, etc.)

## Development Patterns

### Code Generation
- Never edit files in `clients/js/src/generated/` - they are auto-generated
- Run `pnpm generate` after making changes to Rust program interfaces
- Use Kinobi visitors in `configs/kinobi.cjs` for client customization

### Testing
- Program tests are written in Rust using the standard test framework
- Client tests use AVA framework and require a running validator
- Tests often use generated keypairs and work with both individual assets and collections

### Plugin Development
- New internal plugins go in appropriate `plugins/internal/` subdirectory
- External plugin adapters use the external plugin system
- Plugins have distinct authority models (owner-managed vs authority-managed)

### Asset Hierarchy
- **Collections**: Optional grouping mechanism with shared plugins
- **Assets**: Core NFT-like digital assets that can belong to collections
- **Plugin Inheritance**: Assets inherit authority-managed plugins from their collection