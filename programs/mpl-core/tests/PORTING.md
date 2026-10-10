# Porting the JS test suite to Mollusk

The AVA suite in `clients/js/test` is the behavioural specification of the
program: every processor and plugin has positive and negative cases with named
`MplCoreError`s. The Mollusk integration tests in this directory are ported
from it one test at a time, and this file defines how, so that the mapping
between the two suites can be computed instead of hand-maintained. The
rationale is in [docs/coverage-roadmap.md](../../../docs/coverage-roadmap.md),
section 14, item 4.

## Layout

- **One Rust module per JS file, same tree.** All ported tests live in the
  single integration crate `tests/it/main.rs` (one crate, because every
  `tests/*.rs` file links the whole Solana runtime and would add tens of
  seconds of link time). The module tree mirrors `clients/js/test/**` exactly:

  | JS file                                   | Rust module                                 |
  | ----------------------------------------- | ------------------------------------------- |
  | `transfer.test.ts`                        | `tests/it/transfer.rs`                      |
  | `plugins/asset/royalties.test.ts`         | `tests/it/plugins/asset/royalties.rs`       |
  | `externalPlugins/oracle.test.ts`          | `tests/it/external_plugins/oracle.rs`       |
  | `signers/create.test.ts`                  | `tests/it/signers/create.rs`                |

  Directory and file names are converted to snake_case; `main.rs` declares
  `mod transfer; mod plugins { pub mod asset { pub mod royalties; } }` and so
  on.

- **Same test names.** The Rust `fn` is the JS title in snake_case, with
  punctuation dropped (`it can transfer an asset as the owner` becomes
  `it_can_transfer_an_asset_as_the_owner`). Keep the `it_` prefix: it keeps
  the mapping mechanical and greppable.

- **Marker on every ported test.** Each ported test carries a doc-comment
  marker naming the JS file (relative to `clients/js/test/`) and the JS title
  verbatim:

  ```rust
  /// JS: transfer.test.ts :: it can transfer an asset as the owner
  #[test]
  fn it_can_transfer_an_asset_as_the_owner() { ... }
  ```

  The marker is the source of truth for "ported". A test that covers several
  JS tests carries one marker line per JS test (the two `sdkv1.test.ts` "all
  plugins" tests are ported as one Mollusk test, for example). Template-literal
  JS titles keep their `${...}` placeholders in the marker, verbatim, because
  the JS suite declares them once inside a loop.

- **Excluded tests** are listed in the fenced block at the end of this file,
  not marked in the Rust code.

## Assertion translation rules

| JS                                                          | Rust (`tests/common`)                                               |
| ----------------------------------------------------------- | ------------------------------------------------------------------- |
| `t.throwsAsync(..., { name: 'InvalidAuthority' })`          | `assert_core_err(&result, MplCoreError::InvalidAuthority)`          |
| `t.throwsAsync` on a non-`MplCoreError` program error       | `assert_program_err(&result, ProgramError::...)`                    |
| a panic or `unreachable!()` in the program                  | `assert_instruction_err(&result, InstructionError::ProgramFailedToComplete)` |
| `fetchAsset(umi, k)` / `fetchCollection`                    | `f.asset(&k)` / `f.collection(&k)`                                  |
| `assertAsset(t, umi, { ...plugins })`                       | `read_plugin::<T>(&data, PluginType::T)` plus field assertions      |
| `assertBurned`                                              | `assert_burned(&result, &k)` (0 lamports, one `Uninitialized` byte, still program-owned; Mollusk's `Check::closed()` does not match) |
| `umi.rpc.getAccount(k).lamports`                            | `lamports_of(&result, &k)`                                          |
| missing signer (transaction-level failure in JS)            | set `is_signer = false` on the `AccountMeta`; Mollusk skips signature verification, so assert the program-level `MissingRequiredSignature` / `InvalidAuthority` |
| `generateSigner(umi)`                                       | `Pubkey::new_unique()` (any `AccountMeta` with `is_signer = true` signs) |
| `umi.rpc.airdrop(k, sol(1))`                                | `Account { lamports: 1_000_000_000, .. }` seeded in the fixture      |
| `createAsset` / `createCollection` / `createGroup` / `createAssetWithCollection` (`_setupRaw.ts`) | `f.create_asset(..)` / `f.create_collection(..)` / `f.create_group(..)` / `f.create_asset_with_collection(..)` |
| `SPL_NOOP`, oracle example program, Bubblegum signer        | the noop builtin; raw `OracleValidation` bytes written into the account; the fixed pubkey marked as signer in the meta |
| CU assertions                                               | only when `!common::native_program_enabled()` (native runs charge 1 CU) |

Rules of thumb:

- Assert exact errors. Never accept "any error".
- Prefer Mollusk's `process_and_validate_instruction(&ix, &accounts, &[Check::...])`
  for single-instruction tests; use the `assert_*` helpers in multi-step
  chains for readability.
- Read post-state through the program's own types (`AssetV1`, registry,
  plugins) so the tests do not depend on the client's deserializers.
- Tests that need harness support the JS suite gets from the validator
  (`bubblegumV2.test.ts`, `update_collection_info`, `oracle.test.ts`) are
  portable; see section 14, item 5 of the roadmap for the fixture each needs.

## Status script

```sh
./configs/scripts/program/porting-status.sh            # counts plus the pending list
./configs/scripts/program/porting-status.sh --summary  # counts only
./configs/scripts/program/porting-status.sh --strict   # exit 1 on stale markers
```

It extracts every `test(` / `test.serial(` / `test.skip(` title from
`clients/js/test/**/*.test.ts`, every `/// JS:` marker from
`programs/mpl-core/tests/**/*.rs`, and the excluded entries below, and prints
the totals, the ported percentage (ported over portable, where portable is
total minus excluded) and the pending list as `file :: title`. It warns about
excluded entries that match no JS test and about tests that are both excluded
and marked. With `--strict` it exits non-zero when a marker references a JS
file or title that no longer exists; CI runs it that way so renamed JS tests
are noticed. Only bash, find, grep, sed, awk and sort are needed.

Known limitation: titles are only recognised as a quoted string on the
declaration line (or on the next line for a bare `test.serial(`). A file that
aliases the runner (`const serial = test.serial; serial(...)`) is not scanned;
`helps/fetch.test.ts` is the only such file and it is excluded anyway.

## Excluded from the porting target

Tests that are JS-only by nature. One entry per line inside the fenced block:
`<file> :: <title>` excludes one test, `<file>` a whole file, `<dir>/*` a
directory. Lines starting with `#` are comments.

```excluded
# Measures compute units of group stress transactions over RPC. Under native
# coverage every invocation costs 1 CU, so a port is only meaningful in SBF
# mode and is not counted toward the target.
compute.test.ts

# Umi SDK helper logic (client-side state derivation, lifecycle simulation,
# plugin key conversions). No program code involved.
getProgram.test.ts
helps/*

# getProgramAccounts filters through RPC.
asset.test.ts
collection.test.ts

# Prints account sizes.
info.test.ts

# SDK v1 wrapper; the program behaviour is covered by the raw suites. The two
# "all plugins" tests (lines 38 and 270) are NOT excluded: port them as a
# single Mollusk test carrying both markers, since they are the only place
# every plugin is initialised at once.
sdkv1.test.ts :: it can transfer asset
sdkv1.test.ts :: it can transfer asset in collection
sdkv1.test.ts :: it can update asset
sdkv1.test.ts :: it can update collection
sdkv1.test.ts :: it can burn asset
sdkv1.test.ts :: it can burn asset in collection
sdkv1.test.ts :: it can fetch asset which correctly derived plugins
```
