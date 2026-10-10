# mpl-core program: roadmap to full code coverage

Measured on `main` at commit `89a33e3` (2026-09-07) with `pnpm programs:coverage` (`cargo llvm-cov`, unit tests plus the Mollusk integration tests executed natively). The local run reproduces the CI number exactly.

| Metric | Covered | Total | Percent |
| --- | ---: | ---: | ---: |
| Lines | 2479 | 8422 | 29.43% |
| Functions | 202 | 530 | 38.11% |
| Regions | 3014 | 10889 | 27.68% |

This document has two parts. Part 1 (sections 1 to 7) is the roadmap: where coverage stands, why it is low, the phased plan, the shared test infrastructure every phase depends on, the findings that surfaced while reading the uncovered code, and the exclusion policy that defines "full". Part 2 (sections 8 to 14) is the per-area detail: for every source file, the uncovered code paths grouped by trigger, the existing JS or Rust-client test that documents the behaviour (so it can be ported), dead or unreachable code, and a concrete test plan with sizes and ordering.

## 1. Executive summary

Coverage is 29% because the three Mollusk test files (53 tests) exercise only a handful of instructions: `CreateV2`, `CreateCollectionV2`, `TransferV1`, `ExecuteV1`, and the AgentIdentity add/update/remove adapter instructions. Nothing in the counted suite runs `AddPluginV1`, `RemovePluginV1`, `UpdatePluginV1`, approve or revoke plugin authority, `UpdateV1`/`UpdateV2`, `BurnV1`, any group instruction, any Oracle/AppData/LinkedAppData adapter, any `Write*ExternalPluginAdapterDataV1`, or any collection-level plugin or adapter instruction. That leaves 31 of 77 source files at 0% and every `CollectionV1` code path untouched.

Four facts shape the plan:

1. **Most of `tests/account_ownership.rs` never reaches the program.** 21 of its 25 tests pass a default (non-executable, system-owned) account under the program's own ID as the optional-account placeholder, so the SVM rejects the instruction before dispatch, and `assert_failure` accepts any error. Verified against the LCOV data: `BurnV1`, `UpdateV1` and `UpdateCollectionV1` have zero dispatches despite seven dedicated tests, and running one of these tests alone under coverage produces zero hits in `entrypoint.rs`. These are security tests (fake owners, wrong discriminators, frozen assets, permanent delegates) and they currently verify nothing. Fixing the placeholder and asserting exact error codes is a half-day change and the first item on the roadmap.
2. **The behavioural specification already exists.** About 700 JS AVA tests and 61 Rust-client tests document every processor and plugin with named `MplCoreError` outcomes. They do not count toward coverage because they run against a validator or the SBF binary. Roughly 80% of the roadmap is porting them to Mollusk, which is mechanical once a shared fixture library exists. The remaining 20% is new tests for branches the JS suite never reaches (argument and account-count mismatches, non-writable accounts, crafted or corrupted accounts, vector-limit checks that exceed the on-chain transaction size, buffer-account data sources).
3. **No external program stubs are needed for almost anything.** Lifecycle hooks never CPI (they are blocked with `NotAvailable` and their validators are constant abstains). Oracle accounts are read-only and can be fabricated as raw bytes. Bubblegum and mpl-agent-tools are only checked as signer addresses, which Mollusk does not verify. The only executable stub required is an SPL noop builtin for the dormant compression paths, plus an optional "recorder" builtin to assert what `ExecuteV1` CPIs into.
4. **Full coverage is realistically 95%+ lines and 97%+ functions.** The genuinely unreachable set is small and enumerable: five `ForceApproved => unreachable!()` guards, a few exhaustiveness arms, two defensive panics, and a dozen `None`-argument error arms that no processor can produce. Everything else, including the dormant compression code and corrupted-account branches, is reachable from Mollusk with fabricated accounts.

The plan is five milestones. M0 builds the shared harness and fixes the vacuous tests. M1 ports the core instruction suites (update, plugin management, burn, transfer, create) and reaches 50%. M2 ports the per-plugin and external-adapter suites and reaches 70%. M3 covers groups, compression, collect, and adversarial account layouts and reaches 85%. M4 closes with in-crate unit tests for pure logic and applies the exclusion policy to reach 95%+. Estimated volume across all sections, after removing overlap: about 250 Mollusk test functions (many parameterised, roughly 600 scenarios) and about 65 in-crate unit tests. Thresholds ratchet upward at each milestone and are enforced by the existing coverage script, which already supports `COVERAGE_MIN_LINES` but has never had it set.

While reading the uncovered code, the reviewers noted 30 or so behaviours worth a second look (section 6). None is an obvious exploit. The most relevant for a security fork are two panics on caller-supplied accounts instead of clean errors, an Oracle `Execute` check that is silently never enforced, a collection-level write path that creates a `DataSection` on the collection itself, an unauthenticated no-op write path in `VerifiedCreators`, and `UpdateCollectionInfoV1` not requiring the BubblegumV2 plugin on the target collection.

## 2. Where coverage stands

Per-area rollup of the 77 source files (line coverage from the current report):

| Area | Files | Lines covered | Lines total | Line % | Missed |
| --- | ---: | ---: | ---: | ---: | ---: |
| Lifecycle processors (create, burn, transfer, update, compress, collect, execute, dispatch) | 11 | 342 | 1259 | 27.2% | 917 |
| Plugin management processors | 9 | 277 | 1383 | 20.0% | 1106 |
| Groups (9 processors, utils, state, plugin) | 12 | 38 | 952 | 4.0% | 914 |
| Plugin engine and external adapters | 13 | 986 | 2406 | 41.0% | 1420 |
| Internal plugin implementations | 18 | 385 | 1064 | 36.2% | 679 |
| State types and utilities | 14 | 444 | 1300 | 34.2% | 856 |

Files at 0% (31): all nine group processors and `groups_plugin_utils.rs`; `update.rs`, `burn.rs`, `collect.rs`, `compress.rs`, `decompress.rs`, `update_collection_info.rs`; `add_plugin.rs`, `remove_plugin.rs`, `update_plugin.rs`, `approve_plugin_authority.rs`, `revoke_plugin_authority.rs`, `write_external_plugin_adapter_data.rs`; `external/oracle.rs`, `lifecycle_hook.rs`, `linked_lifecycle_hook.rs`, `linked_app_data.rs`, `data_section.rs`; `utils/compression.rs`; `state/compression_proof.rs`, `hashable_plugin_schema.rs`, `update_authority.rs`.

Files at 100% (6): `entrypoint.rs`, `error.rs`, `plugin_header.rs`, `attributes.rs`, `master_edition.rs`, `state/collect.rs`.

The largest absolute gaps are in the plugin engine: `plugins/utils.rs` (419 missed lines), `plugins/lifecycle.rs` (336), `plugins/external_plugin_adapters.rs` (329), and `utils/mod.rs` (309). These are not covered by any single instruction; they light up progressively as the processor suites are ported, which is why the milestone order below front-loads the processor ports.

What the current 53 Mollusk tests actually exercise, per the LCOV function-hit data: `process_instruction` ran 32 times in total, taken by 8 of the 42 dispatch arms. Every hit on `burn.rs`, `update.rs`, and the internal-plugin validators except `FreezeDelegate::validate_transfer` is zero.

## 3. Why coverage is low

- **The counted suite is three narrowly scoped security regression files**, written for specific findings (account ownership, agent identity, execution delegates). They were never meant to be a behavioural suite.
- **21 of the 25 account-ownership tests are vacuous** (section 1, fact 1). The suite reports green while the processors it names have zero hits.
- **The behavioural suites do not count.** The JS suite needs a validator; the Rust-client suite loads the SBF binary through `solana-program-test`. Neither is observable by LLVM instrumentation.
- **Every test hand-builds account bytes and hand-encodes instructions.** Helpers are duplicated across the three files with three different `assert_failure` signatures, and the internal-plugin account builder supports only four plugin types and panics otherwise. Adding a test for a new instruction today means writing a new builder from the IDL by hand, which is why nobody has.
- **No test reads resulting accounts back.** Post-state assertions (registry offsets, plugin bytes, lamport movements, account closure) do not exist, so grow/shrink/memmove paths cannot be verified even where they run.
- **No threshold is enforced.** `coverage.sh` supports `COVERAGE_MIN_LINES` but CI never sets it, so regressions are invisible.

## 4. Roadmap

Targets are line coverage on the stable `cargo llvm-cov` report. Each milestone lists what it contains at a high level; the per-test detail with source line ranges and port sources is in sections 8 to 14. Sizes: S is one instruction against existing fixtures, M needs a new fixture or a multi-step flow, L needs several post-state layout assertions or a new foreign-account fixture.

### M0: infrastructure and the vacuous-test fix (29% to about 30%, or about 40% if the Rust-client suite is plumbed in)

Prerequisite for everything else. Details in section 5 and section 14.

- Fix `tests/account_ownership.rs`: map `MPL_CORE_ID` to `core_program_account()` in its account builder and replace the any-error assertion with exact `MplCoreError` assertions. This alone lights up about 180 lines in `burn.rs`, `update.rs`, `transfer.rs`, the dispatcher, and the `AssetV1` burn/update validators from tests that already exist.
- Build the shared fixture library under `tests/common/` (accounts, fixture, instruction builders, asserts, readers, programs). Generalise the account builders to any plugin type and any `UpdateAuthority`, keep raw builders for adversarial layouts, and add a post-state parser with a registry-consistency check.
- Add `clients/rust` as a path dev-dependency for generated instruction builders (verified to compile with no cycle). Widen the coverage script's ignore regex to exclude `clients/rust/`.
- Register an SPL noop builtin in the harness (native and SBF modes) and optionally a recorder builtin for `ExecuteV1` targets.
- Consolidate integration tests into one `tests/it/main.rs` crate mirroring the `clients/js/test` tree, with a `PORTING.md` mapping table and a status script.
- Commit a thresholds file and turn on the gate at current minus one point for lines, functions, and regions; adopt the ratchet rule.
- Optional: plumb `clients/rust/tests` under coverage via `solana-program-test`'s `processor!` (about one day, +8 to 12 points), then freeze that suite and delete files as their Mollusk ports land.

**Status (2026-09-07): M0 landed** on the branch that carries this document, except the optional program-test plumbing. Measured after M0, with no new behavioural tests: 36.44% lines, 43.77% functions, 33.89% regions (from 29.43 / 38.11 / 27.68). The account-ownership tests now reach the program and assert exact errors, which took `burn.rs` and `update.rs` from 0% to about a third covered; the shared library lives in `programs/mpl-core/tests/common`, the single crate in `tests/it`, and the gate is set at 35 / 42 / 32 in `programs/mpl-core/coverage-thresholds.json`. The porting status script reports 13 of 666 portable JS tests ported.

### M1: core instruction suites (target 50%)

Ports of the highest-value JS files, in this order because each unlocks the next: `update.test.ts` and `updateV2.test.ts` (271 lines of `update.rs` at 0%, plus the asset-in-collection and re-parenting paths in `utils/mod.rs` and `collection.rs`); `addPlugin`, `removePlugin`, `updatePlugin`, `approveAuthority`, `revokeAuthority` (five processors totalling about 690 lines at 0%, plus most of `plugins/lifecycle.rs` and the internal-plugin mutators in `plugins/utils.rs`); `burn`, `burnCollection`, `transfer`, `create`, `createCollection`, `signers/*`. One parameterised guard test (signer, system program, log wrapper, hashed-asset placeholder, Groups plugin) covers about 120 lines across all nine plugin-management processors at once.

Gate: lines at or above 50; no file under `src/processor/` at 0% except groups, compress, decompress, and collect.

### M2: plugins and external adapters (target 70%)

Ports of `plugins/asset/*`, `plugins/collection/*` (about 230 JS tests) covering `update_delegate.rs` (the largest single internal-plugin gap at 18%), `verified_creators.rs`, `royalties.rs`, `autograph.rs`, the freeze and permanent delegates, `edition.rs`, `bubblegum_v2.rs`, and the per-plugin arms of `lifecycle.rs`. Then `externalPlugins/*` (75 JS tests, 47 of them oracle) with a raw oracle-account fixture: all of `oracle.rs`, the `ExtraAccount` derivation arms, the external reject path, AppData and LinkedAppData writes including the `DataSection` creation path and the PR #19 membership check, and every `CollectionV1` monomorphisation of the adapter processors.

In parallel, the in-crate unit tests for pure plugin logic (`validate_royalties`, `calculate_signature_changes`, `validate_autograph`, the stateless reject/abstain arms, check tables, dispatch tables, `bump_offsets`, `ExtraAccount::derive`, `Oracle::validate_helper`): about 45 cases, roughly one day, taking the twelve small plugin files to about 100% in isolation.

Gate: lines at or above 70, regions at or above 60, a per-file floor of 40 with an allowlist.

### M3: groups, compression, collect, adversarial layouts (target 85%)

- Groups: about 45 test functions (about 120 instruction executions) across the nine processors, `groups_plugin_utils.rs`, `state/group.rs`, and the `Groups` plugin. Mollusk makes the expensive JS cases trivial: 256-entry vectors, 8-parent depth, and inconsistent bidirectional state become one-line fabricated `GroupV1` accounts, and `CreateGroupV1`'s vector-full check is only testable here because 257 entries exceed the on-chain transaction size. About half the group branches (account-count and key mismatches, non-writable accounts, per-target authority failures) have no JS test at all.
- Compression: `CompressV1`, `DecompressV1`, and the hashed-asset branches of burn, transfer, update, and execute all run real state transitions (realloc, hashing, noop CPI) before returning `NotAvailable`. One hashed-asset fixture (hash computed in-test from the crate's public `state` types) plus the noop builtin covers `utils/compression.rs` (92 lines), `compression_proof.rs`, `hashable_plugin_schema.rs`, and the `Wrappable::wrap` CPI.
- `collect.rs` and `update_collection_info.rs`: 79 lines, all new tests, all small.
- Adversarial and corrupted-account tests from the raw builders: registry offsets pointing at garbage, wrong discriminators in the collection slot, truncated payloads, underfunded accounts. These are the highest-value tests for a security fork and are only possible in Mollusk.
- Delete the `clients/rust/tests` files whose Mollusk ports have landed, if M0 plumbed them in.

Gate: lines at or above 85, functions at or above 90, file floor 60.

### M4: closure (target 95%+ lines, 97%+ functions, 90%+ regions)

- Unit tests for the remaining pure logic in `state/*` and `utils/mod.rs`: `CompressionProof` round trip, `UpdateAuthority::key`, collection counters with overflow, `SolanaAccount::load`/`save` error arms, the `CoreAsset` impls, `assert_authority`, the check-result matrices, and the validators that no processor can reach.
- Crafted-state tests for the defensive branches that program-produced state cannot reach (inconsistent group relationships, registry records of the wrong type, pre-existing lifecycle-hook records).
- Apply the exclusion policy (section 7): record every accepted exclusion with a reason, annotate whole-function exclusions with `#[cfg_attr(coverage_nightly, coverage(off))]`, and add the optional nightly branch-coverage job as a non-gating artifact for reviewers of `lifecycle.rs`, `utils/mod.rs`, and `plugins/utils.rs`.

Gate: file floor 90 with allowlist; `--fail-uncovered-functions` at or below the allowlist size.

### Ordering by coverage gained per unit of effort, across all sections

1. The account-ownership placeholder fix (half a day, about 180 lines).
2. Fixture library plus six small plugin-management happy paths: `create_plugin_meta`, `initialize_plugin`, `fetch_wrapped_plugin`, `delete_plugin`, approve and revoke on plugin, internal `bump_offsets` (about 250 lines in `plugins/utils.rs`, `lifecycle.rs`, `mod.rs`, plus about 500 lines in the five 0% processors).
3. The `update.rs` block (271 lines, mostly ports from `updateV2.test.ts`, which already enumerates every branch).
4. `burn.rs` and the delegate matrices (burn, transfer, execute with owner and permanent delegates, ForceApproved short-circuit).
5. Collection-targeted instructions: all 150 lines of `validate_collection_permissions` plus most of `collection.rs`.
6. Groups happy paths (ten tests take every group processor from 0% to about 60%), then the fabricated-limit tests.
7. Oracle fixture and the oracle suite (92 lines of `oracle.rs` plus the external reject path and `ExtraAccount` arms).
8. AppData, LinkedAppData, DataSection writes (194 lines of `write_external_plugin_adapter_data.rs`, plus `update_external_plugin_adapter_data` in `plugins/utils.rs`).
9. Compression fixture and the dormant compression paths.
10. Unit tests, crafted-state tests, layout-integrity (grow/shrink with trailing plugins) tests, and exclusions.

## 5. Shared harness prerequisites (deduplicated across sections)

Every section lists its own prerequisites; this is the union, in build order. Detail and design in section 14.

1. **Program-account placeholder fix** in `tests/account_ownership.rs` and a shared `keyed_program_accounts()` so the optional-account sentinel is always `core_program_account()`.
2. **Exact-error assertion helpers**: `assert_core_err(&result, MplCoreError::X)`, `assert_program_err`, `assert_instruction_err` for panics. The three existing files have three different signatures; `execution_delegate.rs` and `agent_identity.rs` already assert exact errors and `account_ownership.rs` must be brought in line.
3. **Generic account builders**: `AssetSpec` and `CollectionSpec` producing the exact on-chain layout (core, `PluginHeaderV1`, plugins and adapters with appended data, `PluginRegistryV1`) for any `Plugin` via `PluginType::from(&plugin)`, any `Authority` per plugin, any `UpdateAuthority` including `Collection(c)`, optional `seq: Some(n)`, and `ExternalRegistryRecord`s with `data_offset`/`data_len`. Plus `GroupV1` builders (with helpers for 256-entry and 8-parent vectors), a one-byte `HashedAssetV1` placeholder, bare (no-meta) asset and collection, an empty 0-lamport system account for `CreateGroupV1`, and a buffer account for the write-data instructions.
4. **Instruction builders** for all 42 discriminators. Preferred: `clients/rust` as a path dev-dependency (generated builders fill the optional-account sentinel and support remaining accounts). Fallback: hand-Borsh, since `MplAssetInstruction` and all `*Args` structs are `pub(crate)`. Keep a raw builder for malformed-input tests.
5. **Post-state readers and invariants**: parse `AssetV1`/`CollectionV1`/`GroupV1`, header, registry, each plugin and adapter and its data slice; `assert_registry_consistent`; `assert_burned` (Mollusk's `Check::closed()` does not match burned mpl-core accounts, which keep one byte and their owner); lamport-flow helpers for execute fees, collect, burn refunds, and realloc top-ups.
6. **Fixture flow on `MolluskContext`** for create-then-mutate sequences, mirroring `_setupRaw.ts` helper for helper (`create_asset`, `create_collection`, `create_group`, `create_asset_with_collection`, `assert_asset`, `assert_collection`, `assert_group`, `assert_burned`).
7. **Program stubs**: SPL noop builtin (native mode via `declare_process_instruction!`, SBF mode via a dumped ELF or the same builtin); optional recorder builtin for `ExecuteV1` CPI targets; enable Mollusk's `inner-instructions` feature to count CPIs.
8. **Foreign-account fixtures**: oracle account (Borsh `OracleValidation` at a configurable offset, any owner) plus PDA derivation helpers for every `ExtraAccount` variant; `ExecutionDelegateRecordV1` built from the `mpl_agent_tools` type instead of hand-packed bytes; agent-identity PDA marked as signer (existing trick, add the unsigned negative).
9. **Hashed-asset fixture**: `HashedAssetV1` account plus matching `CompressionProof`, hash computed from `state::{Compressible, HashedAssetSchema, HashablePluginSchema}`.
10. **In-crate unit-test scaffolding** (`#[cfg(test)]` under `src/plugins/`): `fake_account_info` over a local buffer and a `default_ctx()` producing a `PluginValidationContext` with all-`None` fields and setters. Required because `PluginValidationContext` and most `validate_*` are `pub(crate)`.
11. **Program-owned "new owner" accounts** for Royalties allow and deny lists (the rules key on `AccountInfo::owner`, not the pubkey).

## 6. Findings that surfaced while mapping uncovered code

These are factual observations from reading the uncovered paths, consolidated from all sections and deduplicated. None was verified as exploitable; each is listed because the roadmap should pin the current behaviour with a test so any change is deliberate. Section numbers point to the detailed discussion.

Test-suite defects:

- 21 of 25 tests in `tests/account_ownership.rs` never reach the program (sections 8, 13).
- The existing `to_mollusk_accounts` adds a loader account for SPL noop but never registers it as a program; no current test reaches the noop CPI, which is why it has not failed (section 14).

Panics on caller-supplied input instead of clean errors:

- `RevokePluginAuthorityV1` and its collection variant on an account without plugin metadata index out of bounds in `PluginHeaderV1::load` and surface as `ProgramFailedToComplete` rather than `PluginNotFound`; remove and remove-external guard this, approve and update do not hit it (section 9).
- `load_key` reads `data[0]` unguarded; `CreateGroupV1`, `AddAssetsToGroupV1`, and `RemoveAssetsFromGroupV1` call it on caller-supplied remaining accounts, so a zero-byte account panics (section 10).
- `check_plugin_key` and `ExternalPluginAdapterKey::from_record` slice `data[offset..]` unchecked; a registry record with `offset >= data_len` panics instead of returning `DeserializationError` (section 11).
- `close_program_account` debits with an unchecked subtraction from a rent-derived (not actual) balance; with overflow checks on, an underfunded account panics (section 13).

Enforcement gaps and asymmetries:

- An Oracle may register `Execute` with `can_reject`, but `ExternalPluginAdapter::validate_execute` routes Oracle to the abstaining default; the `Execute` arm in `validate_helper` is dead and the check is silently never enforced (section 11).
- `WriteCollectionExternalPluginAdapterDataV1` with a `LinkedAppData` key creates a `DataSection` on the collection itself; the asset variant restricts linked keys, the collection variant does not (section 9).
- The PR #19 membership check in `write_external_plugin_adapter_data.rs` is the only defence for assets not in any collection; assets in a different collection already fail earlier with the same error. Tests should cover both shapes so a refactor cannot silently remove the protection (section 9).
- `update_external_plugin_adapter` never calls the permission validators; an asset not in a collection accepts any collection account silently, unlike add and remove (section 9).
- `VerifiedCreators::validate_update_plugin` returns `Approved` for a no-op update from any signer, so an unrelated wallet can execute a state-preserving `UpdatePluginV1` that still bumps `seq` (section 12).
- `UpdateCollectionInfoV1` does not require the BubblegumV2 plugin on the target collection, and `Remove` saturates to zero, which then unlocks `BurnCollectionV1` and makes `decrement_size` underflow for remaining members (sections 8, 13).
- `validate_plugin_checks` accepts `Approved` from plugins registered as `CanReject` (Autograph and VerifiedCreators return `Approved` from update and add paths) (section 12).
- Asset-level plugins shadow same-type collection plugins: a collection `UpdateDelegate` cannot act on an asset carrying its own, and an unfrozen asset-level permanent freeze disables a frozen collection-level one (JS treats this as intended) (section 12).
- `create_collection.rs` validates `validate_create` against the payer rather than the update authority, which changes `VerifiedCreators` outcomes on collection creation (sections 8, 12).
- Group authority helpers honour only `additional_delegates` and ignore the `UpdateDelegate` record's own authority, whereas `UpdateV2` honours both; groups themselves have no delegate mechanism and `UpdateGroupV1`'s non-signing `new_update_authority` can permanently orphan a group (sections 10, 13).
- `AddGroupsToGroupV1` only rejects self-linking; cycles are accepted, and `MAX_GROUP_NESTING_DEPTH` caps parents per group, not chain depth (section 10).
- `Add*ToGroupV1` with zero remaining accounts is a silent success (section 10).
- `Collect` permissionlessly reassigns burned asset accounts to the system program (section 8).
- The write-data `buffer` account has no owner, size, or type check and zero coverage in any client suite (section 9).
- `MplCoreError::AssetIsFrozen` is defined but never emitted; frozen assets fail with `InvalidAuthority`. The asset path returns `NoApprovals` when nothing approved, the collection path returns `InvalidAuthority` (section 13).
- The collection update authority is granted `Authority::Owner` via `CollectionV1::owner()`; a crafted collection with an `Owner` registry record would be satisfied by the update authority (section 13).

Latent or dead code worth a decision (remove, or test and document):

- `transfer.rs:118-136,142`: hashed-asset re-serialisation after an unconditional early return (section 8).
- `create.rs:220-221` and `create_collection.rs:182-183,251-253`: `Rejected`/`ForceApproved` bookkeeping for internal create validators that only error or abstain; collection external adapters are never validated at create (section 8).
- `UpdateDelegate::validate_create` is inert; half of it can never execute (section 12).
- `Royalties::validate_add_plugin` validates `self` rather than the target (masked by `PluginAlreadyExists`) (section 12).
- `update_external_plugin_adapter_data` would write the plugin header at offset 0 if called with `core: None`; all current callers pass `Some` (section 11).
- `ExternalRegistryRecord::update` ignores `LinkedLifecycleHook` lifecycle-check updates while honouring `LifecycleHook`'s; moot while both are blocked (section 11).
- `assert_authority`, `fetch_plugins`, `assert_plugins_initialized`, `ExternalPluginAdapter::check_execute`, the `*::{check,validate}_update_external_plugin_adapter` impls, and `CoreAsset for GroupV1` have no callers (sections 10, 11, 13).
- `burn_collection` ignores its `compression_proof` argument (section 8).
- Compressed branches of burn, transfer, and decompress do payer-funded reallocs and a noop CPI before returning `NotAvailable`; `DecompressV1` rebuilds the account before validating permissions (sections 8, 13).

## 7. What "full coverage" means here

Definition: at least 95% lines, 97% functions, and 90% regions on the stable `cargo llvm-cov` report, with `--ignore-filename-regex` limited to non-source paths (`programs/mpl-core/tests/`, `clients/rust/`). Source files are never hidden by the regex.

The program has no feature-gated code and no deprecated items, so the accepted-unreachable set is small:

| Category | Locations | Disposition |
| --- | --- | --- |
| `ForceApproved => unreachable!()` guards (external adapters never force-approve) | `utils/mod.rs:309,465`; `external_plugin_adapters.rs:424,426`; `lifecycle.rs:816` | accepted exclusion |
| Exhaustiveness `unreachable!()` arms after prior matching | `transfer.rs:142`; `lifecycle.rs:705,779`; `add_external_plugin_adapter.rs:76,190` | accepted exclusion; making the matches exhaustive is a reviewed refactor, not a coverage exercise |
| Defensive `panic!` on mismatched internal parameters | `utils/mod.rs:165,363` | accepted exclusion |
| `None`-argument error arms that no processor can produce (`target_plugin`, `resolved_authorities`, `new_owner`) | about a dozen across the plugin files, listed per section | unit-test them (cheap) rather than exclude |
| Compression-only types and paths | `utils/compression.rs`, `compression_proof.rs`, `hashable_plugin_schema.rs`, `hashed_asset.rs` | in scope (M3) |
| Corrupted-account and deserialisation-failure branches | `plugins/utils.rs`, `plugin_registry.rs`, `state/*`, `utils/account.rs` | in scope, highest value for a security fork |
| Dead code discovered while porting | section 6 | remove or justify through the normal review path; never allowlist silently |

Policy: every accepted exclusion is recorded in `programs/mpl-core/COVERAGE_EXCLUSIONS.md` as `file:line | symbol | reason | reviewer | date`, and the sum of excluded lines is what separates the measured number from 100%. Whole-function exclusions carry `#[cfg_attr(coverage_nightly, coverage(off))]` (already warning-free on stable via the crate's `check-cfg`; honoured by the optional nightly job). No code changes are made purely to raise the number. Exclusions are re-validated at each milestone against `--show-missing-lines` output.

---

# Part 2: per-area detail

Each section below was produced by a reviewer assigned to one area, working from the per-file uncovered-line data in the coverage report and reading the source. Line references are relative to `programs/mpl-core`. Test-plan tables use S/M/L sizes as defined in section 4.


## 8. Asset and collection lifecycle processors

Scope: `src/processor/{mod,create,create_collection,burn,transfer,update,update_collection_info,compress,decompress,collect,execute}.rs` and `src/instruction.rs`. All line references are relative to `programs/mpl-core`. Coverage numbers come from the per-file coverage index and the per-file uncovered listings; hit counts come from `coverage/mpl-core/lcov.info` (`FNDA`/`DA` records).

| File | Lines covered | Functions never executed |
|---|---|---|
| `src/processor/mod.rs` | 32/132 (24.2%) | none (dispatcher ran 32 times; 34 of 42 match arms never taken) |
| `src/processor/create.rs` | 110/218 (50.5%) | `create_v1`, `From<CreateV1Args>` |
| `src/processor/create_collection.rs` | 62/166 (37.3%) | `create_collection_v1`, `From<CreateCollectionV1Args>`, the BubblegumV2 `any()` closure |
| `src/processor/burn.rs` | 0/106 (0%) | `burn`, `burn_collection`, `process_burn` |
| `src/processor/transfer.rs` | 47/83 (56.6%) | seq-increment closure (`transfer.rs:139`) |
| `src/processor/update.rs` | 0/271 (0%) | everything: `update_v1`, `update_v2`, `update`, `update_collection`, both `process_update` monomorphizations |
| `src/processor/update_collection_info.rs` | 0/24 (0%) | `update_collection_info` |
| `src/processor/compress.rs` | 0/50 (0%) | `compress` |
| `src/processor/decompress.rs` | 0/50 (0%) | `decompress`, seq closure |
| `src/processor/collect.rs` | 0/55 (0%) | `collect`, `collect_from_account` |
| `src/processor/execute.rs` | 91/104 (87.5%) | none (13 lines of error/PDA-payer branches) |
| `src/instruction.rs` | no coverage record in lcov | n/a |

### Cross-cutting finding: 21 of the 25 `tests/account_ownership.rs` tests never reach the program

`tests/account_ownership.rs` passes `(MPL_CORE_ID, Account::default())` as the account backing the optional-account sentinel in 21 tests (only lines 665 and 1252 use `core_program_account()`). Because that entry also *is* the invoked program's account, the SVM loads a non-executable, system-owned account for the program id and fails before `process_instruction` runs. `assert_failure` (account_ownership.rs:382) accepts any failure, so those tests pass vacuously. lcov confirms it: `process_instruction` ran 32 times in total = 3 `CreateV2` + 1 `CreateCollectionV2` + 14 `ExecuteV1` + 4 `TransferV1` + 10 external-adapter add/update/remove calls (all from `agent_identity.rs`/`execution_delegate.rs`); the `BurnV1`, `UpdateV1` and `UpdateCollectionV1` arms (mod.rs:121-127, 133-139) have 0 hits, and `burn.rs`/`update.rs` are at 0% despite having 7 dedicated tests. The 4 `TransferV1` hits are the two tests that use `core_program_account()` plus the two tests that use `transfer_v1_with_collection_instruction` (which does not put `MPL_CORE_ID` in the account list).

Fix (S, highest value per effort in this scope): make `to_mollusk_accounts` (account_ownership.rs:360) substitute `core_program_account()` for `MPL_CORE_ID` the way it already does for the system program and SPL noop, and replace `assert_failure` with an assertion on the specific `MplCoreError`/`ProgramError` (as `execution_delegate.rs:67` and `agent_identity.rs:92` already do). This immediately exercises `burn.rs:21-107`, `update.rs:55-132`, `update.rs:277-313`, `transfer.rs:75` and the corresponding `mod.rs` arms, and turns 21 vacuous security tests into real ones.

### `src/processor/mod.rs`

`process_instruction` (mod.rs:66) deserializes with `try_from_slice` (line 71, executed 32 times, error path never separately tested) and dispatches. Arms taken so far: `TransferV1`, `CreateV2`, `CreateCollectionV2`, `AddExternalPluginAdapterV1`, `AddCollectionExternalPluginAdapterV1`, `RemoveExternalPluginAdapterV1`, `UpdateExternalPluginAdapterV1`, `ExecuteV1`.

Uncovered arms in this scope (each is 3 lines, covered by any test that sends the instruction): `CreateV1` (73-75), `CreateCollectionV1` (77-79), `BurnV1` (121-123), `BurnCollectionV1` (125-127), `UpdateV1` (133-135), `UpdateCollectionV1` (137-139), `CompressV1` (141-143), `DecompressV1` (145-147), `Collect` (149), `UpdateV2` (182-184), `UpdateCollectionInfoV1` (192-194).

Uncovered arms owned by other sections (listed so the roadmap is complete): `AddPluginV1`/`AddCollectionPluginV1` (81-87), `RemovePluginV1`/`RemoveCollectionPluginV1` (89-95), `UpdatePluginV1`/`UpdateCollectionPluginV1` (97-103), `Approve*/Revoke*PluginAuthorityV1` (105-119), `RemoveCollectionExternalPluginAdapterV1` (170-172), `Write*ExternalPluginAdapterDataV1` (174-180), `UpdateCollectionExternalPluginAdapterV1` (229-231), and all group instructions (197-223, 233-239).

One extra S test: instruction data `[0xFF]` (no such variant) must fail with `ProgramError::BorshIoError` — cheap and pins the discriminator table (see `src/instruction.rs` below).

### `src/processor/create.rs`

`process_create` ran 3 times (all `CreateV2` with an `AgentIdentity` adapter and no collection, no internal plugins).

| Path | Lines | Trigger | Existing test to port |
|---|---|---|---|
| `CreateV1` wrapper | 45-58 | discriminator 0, `CreateV1Args` (no `external_plugin_adapters`) | JS `sdkv1.test.ts`, Rust `clients/rust/tests/create.rs::create_asset_in_account_state` |
| `InvalidSystemProgram` | 77 | account 6 is not `11111111111111111111111111111111` | JS `create.test.ts:266` |
| `InvalidLogWrapperProgram` | 81-83 | account 7 present and not `SPL_NOOP_ID` | JS `create.test.ts:282` |
| `ConflictingAuthority` | 87 | both `update_authority` (acct 5) and `collection` (acct 1) supplied | JS `create.test.ts:253`, `createCollection.test.ts:131` |
| Create into collection | 91-93, 310-312 | acct 1 is a `CollectionV1`; `authority` must be the collection update authority (`CollectionV1::validate_create`, state/collection.rs:138) or an `UpdateDelegate`/additional delegate; afterwards `num_minted` and `current_size` +1 | JS `createCollection.test.ts:62` (auth), `:97` and `plugins/collection/updateDelegate.test.ts:35,88` (delegate), `:135` (wrong auth -> `InvalidAuthority` via line 176) |
| `DataState::LedgerState` | 122-123 | `data_state = 1` -> `NotAvailable` (before the system CPI) | JS `create.test.ts:95` |
| system `create_account` CPI failure | 143 | asset account already has lamports/data or is not system-owned | JS `create.test.ts:196`, `:215` |
| Internal plugins at create | 183-236 | `plugins = Some(vec![...])` non-empty: `create_plugin_meta` + `initialize_plugin` per entry | JS `create.test.ts:112`, `:167`; Rust `create.rs::create_asset_with_plugins` |
| `InvalidPlugin` for asset-forbidden plugins | 194-198 | `MasterEdition`, `BubblegumV2` or `Groups` in `plugins` | JS `plugins/collection/masterEdition.test.ts:135`, `plugins/asset/bubblegumV2.test.ts:6` |
| create-checked plugins | 200-224 | `Royalties` (bad config -> `Err(InvalidPluginSetting)` at 219), `Autograph`/`VerifiedCreators` (unauthorized signature -> `Err(InvalidPluginOperation|MissingSigner)`), `UpdateDelegate` (-> `Approved`, falls to `_`) | JS `plugins/asset/royalties.test.ts:547,571,591`, `autograph.test.ts:9,35,58`, `verifiedCreators.test.ts:9,35,58`, `updateDelegate.test.ts:16,42` |
| `create_meta_idempotent` error | 246 | only via realloc/rent failure (payer under-funded) | new (S) or skip |
| `InvalidPluginAdapterTarget` | 256 | `LinkedLifecycleHook` or `LinkedAppData` in `external_plugin_adapters` | JS `externalPlugins/linkedAppData.test.ts:709` |
| `CannotAddDataSection` | 259 | `DataSection` init info | JS `externalPlugins/dataSection.test.ts:14` |
| external adapter that can reject create | 265-288 | `Oracle`/`LifecycleHook`/`AgentIdentity` init info with a `(Create, {can_reject})` lifecycle check; `Oracle` needs the oracle account in remaining accounts and returns `Rejected` when its validation byte says so -> `approved=false` -> `InvalidAuthority` at 305 | JS `externalPlugins/oracle.test.ts:1312` (deny), `:3216` (missing oracle account), Rust `create_with_external_plugins.rs::test_temporarily_cannot_create_lifecycle_hook` |
| `initialize_external_plugin_adapter` error | 299 | duplicate lifecycle checks in one adapter | Rust `create_with_external_plugins.rs::test_cannot_create_oracle_with_duplicate_lifecycle_checks` |

Dead / effectively unreachable: `ValidationResult::Rejected => approved = false` (220) and `ForceApproved => force_approved = true` (221) for internal plugins. No internal `validate_create` returns `Rejected` or `ForceApproved` (royalties.rs:79-104, autograph.rs:45-95, verified_creators.rs:139-175 return `Err` or `abstain!`; update_delegate.rs:49-64 returns `Approved`). Line 305 is reachable only through the external-adapter path (287).

### `src/processor/create_collection.rs`

`process_create_collection` ran once (`agent_identity.rs::cannot_create_collection_with_agent_identity`, which returns at line 232). No collection has ever been created with internal plugins in Mollusk, and the successful external-adapter init (237-246) has never run.

| Path | Lines | Trigger | Existing test to port |
|---|---|---|---|
| `CreateCollectionV1` wrapper | 39-54 | discriminator 1 | Rust `create_collection.rs::test_create_collection`, JS `sdkv1.test.ts` |
| `InvalidSystemProgram` | 76 | account 3 wrong | JS `createCollection.test.ts:170` |
| system CPI failure | 106 | collection key already in use | new (S) |
| plugins path + BubblegumV2 detection | 122-144 | `plugins = Some(non-empty)`; `has_bubblegum_v2` closure (134) | JS `createCollection.test.ts:37`; Rust `create_collection.rs::create_collection_with_plugins` |
| owner-managed plugin -> `InvalidAuthority` | 146-148 | e.g. `FreezeDelegate` in collection plugins | JS `createCollection.test.ts:151` |
| `Edition`/`Groups` -> `InvalidPlugin` | 151-154 | those plugin types in `plugins` | new (S) |
| BubblegumV2 allow-list -> `BlockedByBubblegumV2` | 157-162 | `BubblegumV2` + a plugin outside `BubblegumV2::ALLOW_LIST` (bubblegum_v2.rs:24) | JS `plugins/collection/bubblegumV2.test.ts:24,120,165` |
| create-checked collection plugins | 164-186 | `Royalties` (`Err(InvalidPluginSetting)` at 181), `UpdateDelegate` (`Approved`), `VerifiedCreators` | JS `plugins/collection/updateDelegate.test.ts:166`; new for Royalties/VerifiedCreators on collection |
| BubblegumV2 fixed authority | 189-198 | `BubblegumV2` with `authority: Some(x)`, `x != manager()` -> `InvalidAuthority` (192) | JS `bubblegumV2.test.ts:331` |
| `initialize_plugin` | 200-211 | any successful plugin | as above |
| external adapters blocked by BubblegumV2 | 217 | `has_bubblegum_v2 && external_plugin_adapters non-empty` | JS `bubblegumV2.test.ts:273` |
| `CannotAddDataSection` | 229 | `DataSection` init info | JS `externalPlugins/dataSection.test.ts:40` |
| external adapter init on collection | 234, 237-246 | `LifecycleHook`/`Oracle`/`AppData`/`LinkedLifecycleHook`/`LinkedAppData` init info | Rust `create_collection_with_external_plugins.rs::{test_create_oracle_on_collection,test_create_app_data_on_collection,test_create_lifecycle_hook_on_collection}`, JS `oracle.test.ts:3857` |

Dead code: 182-183 (`Rejected`/`ForceApproved` arms) and 251-253 (`InvalidAuthority` when `!(approved || force_approved)`). No collection-eligible plugin's `validate_create` returns `Rejected`, and external adapters are not validated at all in this processor (no `validate_create` call in 226-247), so `approved` can never become `false`.

Observation (not a bug): the plugin validation context uses `authority_info: ctx.accounts.payer` (171) while `create.rs:209` uses the resolved `authority`. Both are signers, so the `VerifiedCreators`/`Autograph` "verified signer" invariant holds, but a `VerifiedCreators` entry verified for the *update authority* (account 1, a non-signer) is rejected on collection create even though the same layout is accepted on asset create when `authority` (account 2) signs.

### `src/processor/burn.rs` (0%)

| Path | Lines | Trigger | Existing test to port |
|---|---|---|---|
| entry, optional collection load | 21-32 | acct 1 present -> `CollectionV1::load` (wrong key -> `DeserializationError`, wrong owner -> `InvalidAccountOwner`); payer signer; `resolve_authority` | account_ownership.rs burn tests once un-vacuous; JS `burn.test.ts:26` |
| `InvalidSystemProgram` | 34-38 | acct 4 present and wrong | JS `burn.test.ts:207` |
| `InvalidLogWrapperProgram` | 40-44 | acct 5 present and not noop | JS `burn.test.ts:226` |
| `HashedAssetV1` branch | 47-80 | fabricated `HashedAssetV1` account (key byte 2 + 32-byte hash, owned by program): `compression_proof: None` -> `MissingCompressionProof` (50); no system program -> `MissingSystemProgram` (55); wrong hash -> `IncorrectAssetHash` from `verify_proof`; correct proof -> `rebuild_account_state_from_proof_data`, `wrap()` CPI to SPL noop, then `NotAvailable` (79) | new (M); JS/Rust have no compressed-burn test |
| `IncorrectAccount` | 82 | key byte is `Uninitialized`/`CollectionV1`/`PluginHeaderV1`... | account_ownership.rs:543 `burn_rejects_account_with_wrong_discriminator` (currently vacuous) |
| `validate_asset_permissions` for burn | 86-105 | owner burns (ok); update authority -> `InvalidAuthority`; non-owner; frozen (`FreezeDelegate`/`PermanentFreezeDelegate` on asset or collection); asset in collection with acct 1 missing -> `MissingCollection`; wrong collection -> `InvalidCollection`; `BurnDelegate`/`PermanentBurnDelegate` approve; `Groups` plugin rejects | JS `burn.test.ts:45,71,97,141,159,245,268,313,372`, `plugins/asset/burnDelegate.test.ts`, `plugins/asset/permanentBurn.test.ts`, `plugins/collection/permanentBurn.test.ts`; Groups cases need a `GroupV1` fixture (other section) |
| `process_burn` + collection `decrement_size` | 107-113, 171-173 | success: account resized to 1 byte, key = `Uninitialized`, `rent(len)-rent(1)` lamports moved to payer; if in collection `current_size -= 1` (underflow -> `NumericalOverflowError` with a fabricated `current_size = 0` collection) | JS `collectionSize.test.ts:5`, `burn.test.ts:283` (different payer) |
| `burn_collection` guards | 120-135 | payer signer, `resolve_authority`, `InvalidLogWrapperProgram` (133) | JS `burnCollection.test.ts:118` |
| `CollectionMustBeEmpty` | 137-140 | `current_size > 0` | JS `burnCollection.test.ts:64` |
| `InvalidAuthority` | 143-145 | authority != `collection.update_authority` | JS `burnCollection.test.ts:39` |
| `validate_collection_permissions` for burn | 151-166 | uses `CollectionV1::check_update/validate_update` with `PluginType::check_burn`; `Groups` plugin rejects (`burnCollection.test.ts:137`); external adapters with a Burn hook | JS `burnCollection.test.ts:20,88,137,170` |

Note: `burn_collection` ignores `_args.compression_proof` entirely; `BurnCollectionV1Args` is dead payload.

### `src/processor/transfer.rs` (56.6%)

| Path | Lines | Trigger | Existing test to port |
|---|---|---|---|
| `InvalidSystemProgram` / `InvalidLogWrapperProgram` | 32-40 | acct 5 / acct 6 wrong | JS `transfer.test.ts:298,320` |
| `HashedAssetV1` branch | 45-73 | same fabricated hashed asset as burn; `MissingCompressionProof` (48), `MissingSystemProgram` (53), `verify_proof`, owner overwrite (59), rebuild, `NotAvailable` (72) | new (M) |
| `IncorrectAccount` | 75 | non-asset discriminator | account_ownership.rs:504 (vacuous today) |
| seq increment closure | 139 | asset with `seq: Some(n)`; only produced by compress/decompress (both `NotAvailable`), so fabricate `AssetV1 { seq: Some(5), .. }` and assert `seq == 6` after transfer | new (S) |
| owner-managed authority reset | 104-108 | covered; add an assertion-bearing test: asset with `FreezeDelegate`/`TransferDelegate` authority `Address(x)` -> after transfer both are `Authority::Owner`, while `PermanentFreezeDelegate`/`Attributes` keep theirs | JS `transfer.test.ts:124,175,227` |

Dead code: 118-136 (`Key::HashedAssetV1` re-serialization arm) and 142 (`unreachable!()`) cannot execute because the hashed branch returns at 72. The `compress_into_account_space` call here is the only remaining reference to compressed *transfer*; if compression is not coming back this block should be removed rather than tested.

### `src/processor/update.rs` (0%)

| Path | Lines | Trigger | Existing test to port |
|---|---|---|---|
| `update_v1` wrapper (`new_collection = None`) | 41-78 | discriminator 15 | account_ownership.rs:449,718 (vacuous today); JS `update.test.ts:19,67`, `signers/update.test.ts` |
| `update_v2` | 80-83 | discriminator 30 (7 accounts, acct 4 = `new_collection`) | JS `updateV2.test.ts:27,75` |
| guards | 92-103 | `InvalidSystemProgram` (96), `InvalidLogWrapperProgram` (101) | JS `update.test.ts:292,308`, `updateV2.test.ts:217,233` |
| compressed -> `NotAvailable` | 105-108 | fabricated `HashedAssetV1` | new (S) |
| `validate_asset_permissions` (update) | 110-129 | update authority ok; owner -> `InvalidAuthority` (`update.test.ts:264`); asset key as authority (`:39`); `ImmutableMetadata` rejects (`plugins/asset/immutableMetadata.test.ts`); `UpdateDelegate` approves; `new_update_authority` passed for delegate checks | JS as listed |
| `increment_seq_and_save` | 132 | `seq: Some` fabricated asset | new (S), share fixture with transfer |
| remove from collection, V1 | 141-148 | asset `UpdateAuthority::Collection`, args set `new_update_authority` -> `NotAvailable` | JS `update.test.ts:248` |
| remove from collection, V2 | 149-160 | `existing_collection.decrement_size()` + save; the `.ok_or(MissingCollection)` at 153 is unreachable because `validate_asset_permissions` (utils/mod.rs:174) already fails with `MissingCollection` | JS `updateV2.test.ts:281,325,375,1051,1135,1211,1266,1338` |
| add to collection, V1 | 171-174 | `new_update_authority = Collection(_)` -> `NotAvailable` | JS `update.test.ts:230` |
| add to collection, V2: missing / mismatched account | 177-185 | acct 4 absent -> `MissingCollection`; key != arg -> `InvalidCollection` | JS `updateV2.test.ts:488,533` |
| plugin set of new collection | 188-197 | new collection with plugins (list_plugins) vs bare collection (`HashSet::new()`) | JS `updateV2.test.ts:443` (bare), `:857` (with `UpdateDelegate`) |
| `PermanentDelegatesPreventMove` | 200-202 | new collection has `PermanentFreezeDelegate`/`PermanentTransferDelegate`/`PermanentBurnDelegate` | JS `updateV2.test.ts:1442,1507,1569` |
| new-collection `UpdateDelegate` branch | 208-230 | delegate authority / `additional_delegates` approve; neither -> `InvalidAuthority` (229) | JS `updateV2.test.ts:857,954,1388,737` |
| no-`UpdateDelegate` branch | 231-236 | authority != new collection UA -> `InvalidAuthority`; note `plugin.additional_delegates` is the default (empty) here so the second clause is always false | JS `updateV2.test.ts:556,583` |
| `increment_size` + field updates | 238-254 | success paths; name/uri only (`dirty`) | JS `updateV2.test.ts:443,660` |
| `update_collection` guards | 285-296 | `InvalidSystemProgram`/`InvalidLogWrapperProgram` | JS `update.test.ts:324,340` |
| `validate_collection_permissions` (update) | 298-313 | UA ok; `UpdateDelegate` on collection; `ImmutableMetadata` rejects; `new_update_authority` acct 3 passed for delegate checks | JS `plugins/collection/updateDelegate.test.ts:678,706,732`, `plugins/collection/immutableMetadata.test.ts:142`, `accountOwnership.test.ts:169` |
| new collection update authority | 318-321 | acct 3 present | JS `plugins/collection/updateDelegate.test.ts:678` |
| `process_update` with plugins, grow | 354-389, 416-421 | asset/collection with plugins, longer name -> realloc before memmove, header/registry offsets bumped | JS `update.test.ts:87`, `updateV2.test.ts:95` |
| `process_update` with plugins, shrink | 391-414 | shorter name -> memmove then realloc | JS `update.test.ts:128`; Rust `plugin_shrink_corruption.rs::test_update_v1_shrink_name_uri_preserves_plugin` (:796) and `::test_update_collection_v1_shrink_name_uri_preserves_plugin` (:898) — these assert trailing plugin bytes, port them |
| `process_update` same size / `copy_len == 0` | 386, 393, 410 (false branches) | same-length name; plugin header present but empty registry (all plugins removed) | new (S) |
| `process_update` without plugins | 422-424 | bare asset/collection | JS `update.test.ts:19,67` |
| `NumericalOverflow` arms | 363, 368, 374, 380, 384 | not reachable with realistic sizes | skip |

Both monomorphizations (`process_update::<AssetV1>` and `::<CollectionV1>`) are separate functions in the coverage data; each needs a grow, a shrink and a no-plugin case.

### `src/processor/update_collection_info.rs` (0%)

Only callable by the Bubblegum PDA `BUBBLEGUM_SIGNER` (`CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk`). Mollusk does not verify signatures, so a test can list that pubkey with `is_signer = true` — no Bubblegum program stub needed.

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| signer check | 41 | acct 1 not a signer -> `MissingRequiredSignature` | new |
| `InvalidAuthority` | 44-46 | signer is any other key | Rust `update_collection_info.rs::test_cannot_update_collection_info_with_incorrect_signer` |
| `fetch_core_data` | 48 | acct 0 not a `CollectionV1` -> error | new |
| `Mint` / `Add` / `Remove` | 50-61 | `update_type` 0/1/2, `amount` u32; saturating arithmetic (`Remove` beyond `current_size` clamps to 0; `Mint` with `u32::MAX` clamps) | new (S each) |
| save | 63 | success | new |

Observation: the instruction does not check that the collection carries the `BubblegumV2` plugin; the Bubblegum signer can adjust `num_minted`/`current_size` on any collection. Whether that is intended depends on the Bubblegum integration contract; noting it because `current_size` gates `BurnCollectionV1` (burn.rs:138).

### `src/processor/compress.rs` and `src/processor/decompress.rs` (both 0%)

Both are feature-flagged off at the end (`NotAvailable` at compress.rs:81 / decompress.rs:85) but execute real state transitions first, so they are worth a small set of tests and are cheap once a hashed-asset fixture exists.

`compress`:
- 28-39 guards: `InvalidSystemProgram` (32), `InvalidLogWrapperProgram` (37) — JS `compress.test.ts:124,160`.
- 42-82 `AssetV1`: `fetch_core_data`, `validate_asset_permissions` with `check_compress` (owner only), `compress_into_account_space` (asset+registry hashed, account shrunk to 33 bytes, `seq` set), `wrap()` = CPI to SPL noop with **no accounts**, then `NotAvailable` — JS `compress.test.ts:90`. The noop CPI requires the noop program to be loadable in Mollusk and present in the instruction account list (acct 5 `log_wrapper`); if `log_wrapper` is omitted the failure is a runtime missing-account error, not `NotAvailable`.
- 83 `AlreadyCompressed` (fabricated hashed asset), 84 `IncorrectAccount` (collection or uninitialized account) — new, S.

`decompress`:
- 30-41 guards — JS `decompress.test.ts:96,128` (these pass an arbitrary proof against a normal asset; they fail on the guard before the key check).
- 44-86 `HashedAssetV1`: `verify_proof` (`IncorrectAssetHash` on mismatch), seq closure (50), `rebuild_account_state_from_proof_data` (realloc paid by payer; with plugins in the proof it re-initialises them via `initialize_plugin`), `validate_asset_permissions` with `check_decompress`, `NotAvailable`. Two tests: matching proof with 0 plugins, matching proof with 1-2 plugins (exercises `utils/compression.rs` too). New, M.
- 87 `AlreadyDecompressed` (plain asset + any proof) — new, S. 88 `IncorrectAccount` — new, S.

Fixture: `HashedAssetV1 { key: HashedAssetV1, hash }` with `hash = keccak(HashedAssetSchema { asset_hash: keccak(borsh(AssetV1)), plugin_hashes })` computed in the test with the crate's public `state::{Compressible, HashedAssetSchema, HashablePluginSchema, CompressionProof}` (`pub mod state` in lib.rs).

### `src/processor/collect.rs` (0%)

Permissionless fee sweep. All paths are new tests (JS `collect.test.ts` uses the real recipients on a validator; the Mollusk version must use the hard-coded `COLLECT_RECIPIENT1/2` from `state/collect.rs:5-8`).

| Path | Lines | Trigger | JS reference |
|---|---|---|---|
| wrong recipient 1 / 2 -> `IncorrectAccount` | 14-20 | any other pubkey in acct 0 / 1 | none |
| remaining account not program-owned -> `IncorrectAccount` | 25-28 | system-owned account in remaining accounts | none |
| `Uninitialized` (burned asset) | 44-53 | 1-byte account with key 0: assigned to system program, keeps `rent(1)`, remainder split | `collect.test.ts:404,277,326` |
| `AssetV1`/`HashedAssetV1` | 54-62 | account with lamports > `rent(data_len)` (created with the create fee) | `collect.test.ts:155,174,211` |
| other key -> `IncorrectAccount` | 63 | a `CollectionV1` in remaining accounts (collections carry no fee) | none |
| split and write | 66-83 | odd fee amount -> recipient2 gets the extra lamport; zero fee (already collected) -> idempotent | `collect.test.ts:430` |
| `NumericalOverflowError` | 51, 59 | account lamports below rent-exempt minimum (fabricate) | none |

### `src/processor/execute.rs` (87.5%)

Remaining gaps:
- 36 `InvalidAsset`: acct 0 not owned by the program — new, S (same shape as the account_ownership fake-asset tests).
- 49-51 + 97-101: payer *is* the `mpl-core-execute` PDA (`payer_is_pda`): `authority` must be supplied and sign (else `MissingSigner` at 49); the fee is paid by `invoke_signed` from the PDA, so the PDA account must hold `get_execute_fee()` + the CPI's needs — JS `execute.test.ts:377,427,466`. Two tests: success, and missing authority -> `MissingSigner`.
- 58 `InvalidSystemProgram` — JS `execute.test.ts:511`.
- 62-63 compressed -> `NotAvailable` — fabricated hashed asset, S.
- 106 `invoke` fee transfer failure: payer with fewer lamports than `get_execute_fee()` — new, S.

### `src/instruction.rs`

lcov has no `SF:` record for this file: it contains only the `MplAssetInstruction` enum with Shank attributes plus the derive-generated `accounts::*Accounts::context()` helpers, and those expansions are attributed to the macro crate, not to this file. Nothing to cover directly; the discriminator table (0 `CreateV1` ... 32 `UpdateCollectionInfoV1`, 30 `UpdateV2`, 31 `ExecuteV1`, 33-41 group instructions) is what the hand-rolled instruction builders in the Mollusk tests depend on. Note that `MplAssetInstruction` and every `*Args` struct are `pub(crate)`, so integration tests cannot serialize the enum directly; the shared builder must either hand-encode Borsh (as account_ownership.rs:277-357 does today) or use the generated builders from `clients/rust` as a dev-dependency (see the infrastructure section, which verified that this compiles).

### Bugs, suspicious logic and security-relevant paths noticed

1. **Vacuous security tests** (`tests/account_ownership.rs`, 21 tests): see the cross-cutting finding. The fake-owner, wrong-discriminator, permanent-delegate and frozen-collection rejections for `BurnV1`, `UpdateV1`, `UpdateCollectionV1` and most `TransferV1` cases are currently not verified by anything that counts toward coverage.
2. `create.rs:220-221`, `create_collection.rs:182-183, 251-253`: `Rejected`/`ForceApproved` handling for internal plugins at create time is dead; internal create validators only `Err` or `abstain`/`approve`. Not exploitable, but the `approved` flag gives a false sense that a plugin can veto creation.
3. `transfer.rs:118-136`: dead re-serialization branch for `HashedAssetV1` after an unconditional early return at 72; `unreachable!()` at 142 is genuinely unreachable.
4. `burn.rs`/`transfer.rs`/`decompress.rs` compressed branches do a payer-funded `rebuild_account_state_from_proof_data` (realloc) and a noop CPI *before* returning `NotAvailable`. The transaction reverts, so there is no persisted effect, but the compute/rent path is live and untested.
5. `update.rs:150-153`: `.ok_or(MissingCollection)` is unreachable (already enforced by `validate_asset_permissions`); `update.rs:231-232`: `plugin.additional_delegates` is always the default empty vector in the else-branch, so that clause is a no-op.
6. `update_collection_info.rs`: no `BubblegumV2` plugin requirement on the target collection; `Remove` uses `saturating_sub`, so over-removal silently zeroes `current_size` (which then permits `BurnCollectionV1`).
7. `collect.rs:45`: `Collect` reassigns burned (`Uninitialized`) asset accounts to the system program with 1 byte of data and `rent(1)` lamports; any caller can do this. Not a vulnerability by itself, but it is a permissionless state change on program-owned accounts that no Mollusk test observes.
8. `burn_collection` ignores its `compression_proof` argument (dead field).

### Test plan

Sizes: S = single instruction, existing fixtures; M = needs a new fixture (collection with plugins, hashed asset, PDA-funded payer); L = needs external program or multi-instruction setup.

| # | Test case | Sets up | Instruction(s) | Expected | Source lines covered | New / port from | Size |
|---|---|---|---|---|---|---|---|
| 1 | `fix_account_ownership_sentinel` (harness fix, not a test) | `to_mollusk_accounts` substitutes `core_program_account()`; `assert_failure` -> specific error | BurnV1, UpdateV1, UpdateCollectionV1, TransferV1 | 21 tests now hit the program | burn.rs:21-46,82; update.rs:55-129,277-313; transfer.rs:75; mod.rs:121-139 | existing Mollusk tests | S |
| 2 | `dispatch_rejects_unknown_discriminator` | none | data `[0xFF]` | `BorshIoError` | mod.rs:71 | new | S |
| 3 | `create_v1_account_state` | payer, fresh asset key | CreateV1 | asset created, owner = payer, UA = payer | create.rs:45-58, mod.rs:73-75 | Rust `create.rs::create_asset_in_account_state` | S |
| 4 | `create_rejects_invalid_system_program` / `invalid_log_wrapper` | wrong acct 6 / acct 7 | CreateV2 | `InvalidSystemProgram` / `InvalidLogWrapperProgram` | create.rs:77,81-83 | JS `create.test.ts:266,282` | S |
| 5 | `create_rejects_conflicting_authority` | collection + update_authority both set | CreateV2 | `ConflictingAuthority` | create.rs:87 | JS `create.test.ts:253` | S |
| 6 | `create_ledger_state_not_available` | `data_state = LedgerState` | CreateV2 | `NotAvailable` | create.rs:122-123 | JS `create.test.ts:95` | S |
| 7 | `create_rejects_address_in_use` | asset account pre-funded / pre-owned | CreateV2 | system-program CPI error | create.rs:143 | JS `create.test.ts:196,215` | S |
| 8 | `create_in_collection_as_authority` | bare `CollectionV1` (UA = payer) | CreateV2 with acct 1 | asset UA = Collection, `num_minted = current_size = 1` | create.rs:91-93,310-312 | JS `createCollection.test.ts:62` | M |
| 9 | `create_in_collection_as_update_delegate` / `additional_delegate` | collection with `UpdateDelegate` (authority `Address(d)` / `additional_delegates=[d]`) | CreateV2 as `d` | success | create.rs:91-93,176,310-312; utils | JS `plugins/collection/updateDelegate.test.ts:35,88` | M |
| 10 | `create_in_collection_rejects_wrong_authority` | collection UA != signer | CreateV2 | `InvalidAuthority` | create.rs:176 | JS `createCollection.test.ts:135` | M |
| 11 | `create_with_internal_plugins` | plugins `[FreezeDelegate, Attributes, UpdateDelegate]` | CreateV2 | registry has 3 records, authorities as given | create.rs:183-236 | JS `create.test.ts:112`; Rust `create.rs::create_asset_with_plugins` | S |
| 12 | `create_rejects_collection_only_plugins` | plugins `[MasterEdition]`, `[BubblegumV2]`, `[Groups]` (3 cases) | CreateV2 | `InvalidPlugin` | create.rs:194-198 | JS `masterEdition.test.ts:135`, `bubblegumV2.test.ts:6` | S |
| 13 | `create_rejects_invalid_royalties` | `Royalties` with percentages != 100 / bps > 10000 / dup creators | CreateV2 | `InvalidPluginSetting` | create.rs:200-224 | JS `royalties.test.ts:547,571,591` | S |
| 14 | `create_autograph_and_verified_creators` | `Autograph`/`VerifiedCreators` signed by payer (ok) and by someone else (reject) | CreateV2 | success / `InvalidPluginOperation`/`MissingSigner` | create.rs:200-224 | JS `autograph.test.ts:9,35,58`, `verifiedCreators.test.ts:9,35,58` | S |
| 15 | `create_rejects_linked_adapters_and_data_section` | `LinkedAppData`, `LinkedLifecycleHook`, `DataSection` init infos | CreateV2 | `InvalidPluginAdapterTarget` / `CannotAddDataSection` | create.rs:256,259 | JS `linkedAppData.test.ts:709`, `dataSection.test.ts:14` | S |
| 16 | `create_with_oracle_denying_create` | `Oracle` init with `(Create, can_reject)`, oracle account (validation byte = reject) in remaining accounts; second case without the oracle account | CreateV2 | `InvalidAuthority` / missing-account error | create.rs:264-288,305 | JS `oracle.test.ts:1312,3216` | M |
| 17 | `create_with_agent_identity_create_check_approving` | `AgentIdentity` init with `(Create, can_reject)` + PDA signer | CreateV2 | success | create.rs:264-289 | extend `agent_identity.rs` | S |
| 18 | `create_collection_v1` | fresh collection key | CreateCollectionV1 | collection created | create_collection.rs:39-54, mod.rs:77-79 | Rust `create_collection.rs::test_create_collection` | S |
| 19 | `create_collection_rejects_invalid_system_program` / `address_in_use` | wrong acct 3 / pre-owned key | CreateCollectionV2 | `InvalidSystemProgram` / CPI error | create_collection.rs:76,106 | JS `createCollection.test.ts:170` | S |
| 20 | `create_collection_with_plugins` | `[Royalties(valid), Attributes, UpdateDelegate{additional_delegates}]` | CreateCollectionV2 | registry populated | create_collection.rs:122-144,164-186,200-211 | JS `createCollection.test.ts:37`, `updateDelegate.test.ts:166`; Rust `create_collection_with_plugins` | S |
| 21 | `create_collection_rejects_owner_managed_and_asset_only_plugins` | `[FreezeDelegate]`, `[Edition]`, `[Groups]` | CreateCollectionV2 | `InvalidAuthority` / `InvalidPlugin` | create_collection.rs:146-154 | JS `createCollection.test.ts:151`; new for Edition/Groups | S |
| 22 | `create_collection_bubblegum_v2_allow_list` | `[BubblegumV2, Attributes]` (ok); `[BubblegumV2, MasterEdition]` (reject); `[BubblegumV2 with authority Some(Address)]` (reject); `BubblegumV2` + external adapter (reject) | CreateCollectionV2 | success / `BlockedByBubblegumV2` / `InvalidAuthority` / `BlockedByBubblegumV2` | create_collection.rs:132-142,157-162,189-198,217 | JS `plugins/collection/bubblegumV2.test.ts:24,120,165,331,273` | S |
| 23 | `create_collection_with_external_adapters` | `Oracle`, `AppData`, `LifecycleHook` init infos; `DataSection` (reject) | CreateCollectionV2 | adapters initialised / `CannotAddDataSection` | create_collection.rs:214-249 | Rust `create_collection_with_external_plugins.rs` (3 tests), JS `dataSection.test.ts:40` | S |
| 24 | `burn_as_owner` (+ different payer) | valid asset, payer = owner / separate payer with authority | BurnV1 | account 1 byte, key 0, lamports moved to payer | burn.rs:21-32,81,86-113,171-173 | JS `burn.test.ts:26,283` | S |
| 25 | `burn_rejects_invalid_system_program` / `log_wrapper` | wrong acct 4 / acct 5 | BurnV1 | `InvalidSystemProgram` / `InvalidLogWrapperProgram` | burn.rs:34-44 | JS `burn.test.ts:207,226` | S |
| 26 | `burn_rejects_non_owner_and_update_authority` | authority = UA / stranger | BurnV1 | `InvalidAuthority` | burn.rs:86-105 | JS `burn.test.ts:45,71` | S |
| 27 | `burn_in_collection` | asset UA = Collection, collection `current_size = 1`; cases: ok, missing collection, wrong collection, `current_size = 0` | BurnV1 | success (`current_size = 0`) / `MissingCollection` / `InvalidCollection` / `NumericalOverflowError` | burn.rs:24-25,108-111 | JS `burn.test.ts:141,268`, `collectionSize.test.ts:5`; new for underflow | M |
| 28 | `burn_frozen_and_delegates` | `FreezeDelegate{frozen}`; collection `PermanentFreezeDelegate{frozen}`; `BurnDelegate` authority `Address(d)` burning as `d`; `PermanentBurnDelegate` on collection | BurnV1 | reject / reject / success / success | burn.rs:86-105 (+ plugin validators) | JS `burn.test.ts:97,159,245`, `burnDelegate.test.ts`, `permanentBurn.test.ts` | M |
| 29 | `burn_compressed_paths` | fabricated `HashedAssetV1`: no proof; proof + no system program; wrong hash; correct proof | BurnV1 | `MissingCompressionProof` / `MissingSystemProgram` / `IncorrectAssetHash` / `NotAvailable` | burn.rs:47-80 | new | M |
| 30 | `burn_collection_as_authority` (+ different payer) | empty collection | BurnCollectionV1 | closed | burn.rs:120-169 | JS `burnCollection.test.ts:20,88` | S |
| 31 | `burn_collection_rejections` | non-empty (`current_size = 1`); wrong authority; bad log wrapper | BurnCollectionV1 | `CollectionMustBeEmpty` / `InvalidAuthority` / `InvalidLogWrapperProgram` | burn.rs:131-145 | JS `burnCollection.test.ts:64,39,118` | S |
| 32 | `transfer_rejects_invalid_system_program` / `log_wrapper` | wrong acct 5 / 6 | TransferV1 | errors | transfer.rs:32-40 | JS `transfer.test.ts:298,320` | S |
| 33 | `transfer_compressed_paths` | as #29 | TransferV1 | `MissingCompressionProof` / `MissingSystemProgram` / `IncorrectAssetHash` / `NotAvailable` | transfer.rs:46-72 | new | M (shares fixture with #29) |
| 34 | `transfer_increments_seq_when_present` | asset with `seq: Some(5)` | TransferV1 | owner changed, `seq == 6` | transfer.rs:139 | new | S |
| 35 | `transfer_resets_owner_managed_authorities` | `FreezeDelegate{Address}`, `TransferDelegate{Address}`, `PermanentFreezeDelegate{Address}`, `Attributes{UA}` | TransferV1 as delegate | owner-managed -> `Owner`, others unchanged | transfer.rs:100-112 | JS `transfer.test.ts:124,175,227` | M |
| 36 | `update_v1_name_uri_no_plugins` (grow + shrink) | bare asset | UpdateV1 | fields updated, account resized | update.rs:41-78,92-137,247-268,422-426 | JS `update.test.ts:19,67` | S |
| 37 | `update_v1_with_plugins_grow_and_shrink` | asset with 2 plugins | UpdateV1 | plugins intact, offsets bumped | update.rs:354-421 (`AssetV1`) | JS `update.test.ts:87,128`; Rust `plugin_shrink_corruption.rs:796` | M |
| 38 | `update_rejections` | wrong authority; asset key as authority; bad system program; bad log wrapper; `ImmutableMetadata` | UpdateV1/V2 | `InvalidAuthority` x2 / `InvalidSystemProgram` / `InvalidLogWrapperProgram` / reject | update.rs:95-103,110-129 | JS `update.test.ts:264,39,292,308`, `immutableMetadata.test.ts` | S |
| 39 | `update_compressed_not_available` + `seq_increment` | hashed asset; asset with `seq: Some` | UpdateV2 | `NotAvailable`; seq +1 | update.rs:105-108,132 | new | S |
| 40 | `update_v1_cannot_change_collection` | asset in collection + `new_update_authority = Address`; bare asset + `Collection(c)` | UpdateV1 | `NotAvailable` x2 | update.rs:141-148,171-174 | JS `update.test.ts:248,230` | M |
| 41 | `update_v2_remove_from_collection` | asset in collection (size 1), authority = collection UA / `UpdateDelegate` / additional delegate / asset-level delegate (reject) | UpdateV2 `new_update_authority = Address` | UA changed, `current_size = 0` / `InvalidAuthority` | update.rs:149-160,244-246 | JS `updateV2.test.ts:281,1051,1135,1211,1266,325` | M |
| 42 | `update_v2_add_to_collection` | bare asset + new collection (bare / with `UpdateDelegate` / with additional delegate / with permanent delegate); missing acct 4; mismatched acct 4; wrong authority | UpdateV2 `Collection(c)` | success (`current_size` +1) / `MissingCollection` / `InvalidCollection` / `PermanentDelegatesPreventMove` / `InvalidAuthority` | update.rs:177-242 | JS `updateV2.test.ts:443,857,1388,1442,488,533,556,737` | M |
| 43 | `update_v2_change_collection` | asset in collection A, new collection B, same UA / delegate | UpdateV2 | A size -1, B size +1 | update.rs:149-160,177-242 | JS `updateV2.test.ts:660,857,954` | M |
| 44 | `update_collection_name_uri_and_authority` | bare collection; collection with plugins (grow, shrink); acct 3 new UA | UpdateCollectionV1 | fields/UA updated, plugins intact | update.rs:277-343, 354-426 (`CollectionV1`) | JS `accountOwnership.test.ts:169`, `updateDelegate.test.ts:678`; Rust `plugin_shrink_corruption.rs:898` | M |
| 45 | `update_collection_rejections` | bad system program / log wrapper; non-authority; `ImmutableMetadata` | UpdateCollectionV1 | errors | update.rs:288-313 | JS `update.test.ts:324,340`, `immutableMetadata.test.ts:142` | S |
| 46 | `update_collection_info_mint_add_remove` | collection, `BUBBLEGUM_SIGNER` as signer; amounts incl. over-remove and `u32::MAX` | UpdateCollectionInfoV1 x3 | counters updated with saturation | update_collection_info.rs:33-66, mod.rs:192-194 | new | S |
| 47 | `update_collection_info_rejects_wrong_signer` / `non_signer` / `non_collection` | other key / `is_signer=false` / asset account | UpdateCollectionInfoV1 | `InvalidAuthority` / `MissingRequiredSignature` / deserialization error | update_collection_info.rs:41-48 | Rust `update_collection_info.rs` | S |
| 48 | `compress_not_available_after_state_change` | asset (with and without plugins), noop program registered + passed as acct 5 | CompressV1 | `NotAvailable` | compress.rs:20-82, mod.rs:141-143, utils/compression.rs:64-122 | JS `compress.test.ts:90` | M |
| 49 | `compress_rejections` | bad system program; bad log wrapper; hashed asset; collection as acct 0 | CompressV1 | `InvalidSystemProgram` / `InvalidLogWrapperProgram` / `AlreadyCompressed` / `IncorrectAccount` | compress.rs:31-39,83-84 | JS `compress.test.ts:124,160`; new | S |
| 50 | `decompress_not_available_after_rebuild` | hashed asset + matching proof (0 plugins; 2 plugins) | DecompressV1 | `NotAvailable` | decompress.rs:22-86, utils/compression.rs:21-62,124-158 | new | M |
| 51 | `decompress_rejections` | bad system program; bad log wrapper; wrong hash; plain asset; collection | DecompressV1 | `InvalidSystemProgram` / `InvalidLogWrapperProgram` / `IncorrectAssetHash` / `AlreadyDecompressed` / `IncorrectAccount` | decompress.rs:33-41,46,87-88 | JS `decompress.test.ts:96,128`; new | S |
| 52 | `collect_from_assets_and_burned_assets` | `COLLECT_RECIPIENT1/2` writable; remaining: asset with fee, hashed asset with fee, 1-byte `Uninitialized` account with fee | Collect | recipients split fee, accounts left at rent, burned account reassigned to system | collect.rs:10-86, mod.rs:149 | JS `collect.test.ts:155,174,404` | S |
| 53 | `collect_idempotent_and_odd_split` | already-collected asset; fee of 3 lamports | Collect x2 | no-op; recipient2 gets 2 | collect.rs:66-83 | JS `collect.test.ts:430` | S |
| 54 | `collect_rejections` | wrong recipient1; wrong recipient2; system-owned remaining account; collection as remaining account; asset below rent | Collect | `IncorrectAccount` x4 / `NumericalOverflowError` | collect.rs:14-28,63,51,59 | new | S |
| 55 | `execute_rejects_foreign_owned_asset` / `invalid_system_program` / `compressed` | asset owned by other program; wrong acct 5; hashed asset | ExecuteV1 | `InvalidAsset` / `InvalidSystemProgram` / `NotAvailable` | execute.rs:36,58,62-63 | JS `execute.test.ts:511`; new | S |
| 56 | `execute_with_pda_payer` | payer = asset-signer PDA funded with fee; authority = owner signer; second case without authority | ExecuteV1 | success / `MissingSigner` | execute.rs:46-51,94-101 | JS `execute.test.ts:377` | M |
| 57 | `execute_fee_transfer_fails_when_payer_underfunded` | payer with 0 lamports | ExecuteV1 | system transfer error | execute.rs:106 | new | S |

### Harness prerequisites

1. **Program-account sentinel fix** in `tests/account_ownership.rs::to_mollusk_accounts` (or a shared helper in `tests/common/mod.rs`): always map `MPL_CORE_ID` to `core_program_account()`. Without this every test that uses the program id as the optional-account placeholder is vacuous.
2. **Shared instruction builders**: `MplAssetInstruction` and the `*Args` structs are `pub(crate)`, so the builders must hand-encode Borsh or wrap the generated `clients/rust` builders (preferred, see the infrastructure section); either way one shared helper removes the per-file byte assembly and pins discriminators (`CreateV1=0`, `CreateCollectionV1=1`, `BurnV1=12`, `BurnCollectionV1=13`, `TransferV1=14`, `UpdateV1=15`, `UpdateCollectionV1=16`, `CompressV1=17`, `DecompressV1=18`, `Collect=19`, `CreateV2=20`, `CreateCollectionV2=21`, `UpdateV2=30`, `ExecuteV1=31`, `UpdateCollectionInfoV1=32`).
3. **Account fixture builders** (extend `account_ownership.rs::build_asset_with_plugins` / `build_collection_with_plugins` and move them to `tests/common`): `AssetV1` with arbitrary `update_authority` (`Address`/`Collection`/`None`), optional `seq: Some(n)`, plugin list with per-plugin `Authority`; `CollectionV1` with `num_minted`/`current_size` and plugin list (`UpdateDelegate` with `additional_delegates`, permanent delegates, `Royalties`, `ImmutableMetadata`, `BubblegumV2`); external-adapter registry records (`Oracle`, `AppData`, `LifecycleHook`) for #16/#23.
4. **Hashed-asset fixture**: `HashedAssetV1` account plus the matching `CompressionProof` (hash computed via `state::{Compressible, HashedAssetSchema, HashablePluginSchema}`), used by #29, #33, #39, #49-51, #55.
5. **SPL noop stub**: `wrap()` CPIs to `noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV` with no accounts. In native mode register a `Builtin` whose entrypoint returns `Ok(())`; in SBF mode load a noop ELF into `SBF_OUT_DIR` (or reuse `mollusk_svm::program::create_program_account_loader_v3` plus a stub in the program cache). Needed by #29, #33, #48; `to_mollusk_accounts` already adds the account but not the executable.
6. **Oracle account fixture** for #16: a plain account whose data is the oracle validation struct at the configured offset (see `plugins/external/oracle.rs::validate_helper`); no external program needed.
7. **Rent sysvar / fees**: `get_create_fee()` and `get_execute_fee()` derive from `Rent::get()`; tests asserting lamport deltas (#24, #52, #56) should compute expected values with `mollusk.sysvars.rent`.
8. **Execute PDA payer** (#56): derive `["mpl-core-execute", asset]` and pre-fund it; `execution_delegate.rs::asset_signer_pda` already does the derivation.
9. **Group fixtures** (`GroupV1`, `Groups` plugin) are needed for the Groups-rejection cases in `burn.test.ts:313-395` and `burnCollection.test.ts:137,170`; they belong to the groups section and are excluded from the estimates below.

### Estimate and ordering

About 57 test functions (some with 2-5 sub-cases), ~35 S, ~20 M, 0 L. Ordering by coverage gained per unit of effort:

1. Harness fix #1 — unlocks ~180 lines in `burn.rs`/`update.rs`/`transfer.rs`/`mod.rs` from tests that already exist; half a day.
2. `update.rs` block (#36-#45) — 271 lines at 0%, the largest single file in scope; mostly ports from `updateV2.test.ts`, which already enumerates every branch. Needs fixture builder (prereq 3).
3. `burn.rs` (#24-#31) — 106 lines at 0%, simple instruction, ports from `burn.test.ts`/`burnCollection.test.ts`.
4. `create.rs`/`create_collection.rs` internal-plugin and collection paths (#8-#15, #18-#23) — ~200 lines, all S once builders exist; ports from `create.test.ts`, `createCollection.test.ts`, `bubblegumV2.test.ts`.
5. `collect.rs` (#52-#54) and `update_collection_info.rs` (#46-#47) — 79 lines, all new but trivial (S).
6. `compress.rs`/`decompress.rs` + compressed branches of burn/transfer/update/execute (#29, #33, #39, #48-#51, #55) — ~130 lines in this scope plus most of `utils/compression.rs` (92 lines); gated on the hashed-asset fixture and noop stub (prereqs 4-5).
7. Remaining `execute.rs` and `transfer.rs` gaps (#32, #34, #35, #56, #57) — ~25 lines, S/M.
8. Oracle-at-create (#16) — 24 lines in `create.rs` plus a chunk of `plugins/external/oracle.rs` (92 lines at 0%); shares the oracle fixture with the external-adapter section.

Not worth writing tests for (dead or unreachable): `transfer.rs:118-136,142`; `create.rs:220-221`; `create_collection.rs:182-183,251-253`; `update.rs:150-153` error arm and the `NumericalOverflow` arms in `process_update`; `update.rs:232`. Removing the transfer dead branch and the create `approved`/`force_approved` bookkeeping would reduce the uncovered total by ~30 lines without losing behavior.


## 9. Plugin and external adapter management processors

Scope: `src/processor/{add_plugin,remove_plugin,update_plugin,approve_plugin_authority,revoke_plugin_authority,add_external_plugin_adapter,remove_external_plugin_adapter,update_external_plugin_adapter,write_external_plugin_adapter_data}.rs`.

| File | Lines covered | Functions covered | Instructions (discriminator) |
|---|---|---|---|
| src/processor/add_plugin.rs | 0 / 156 (0.0%) | 0 / 3 | AddPluginV1 (2), AddCollectionPluginV1 (3) |
| src/processor/remove_plugin.rs | 0 / 117 (0.0%) | 0 / 3 | RemovePluginV1 (4), RemoveCollectionPluginV1 (5) |
| src/processor/update_plugin.rs | 0 / 166 (0.0%) | 0 / 3 | UpdatePluginV1 (6), UpdateCollectionPluginV1 (7) |
| src/processor/approve_plugin_authority.rs | 0 / 118 (0.0%) | 0 / 3 | ApprovePluginAuthorityV1 (8), ApproveCollectionPluginAuthorityV1 (9) |
| src/processor/revoke_plugin_authority.rs | 0 / 136 (0.0%) | 0 / 3 | RevokePluginAuthorityV1 (10), RevokeCollectionPluginAuthorityV1 (11) |
| src/processor/add_external_plugin_adapter.rs | 101 / 180 (56.1%) | asset fn partially, collection fn partially, process fn (asset) | AddExternalPluginAdapterV1 (22), AddCollectionExternalPluginAdapterV1 (23) |
| src/processor/remove_external_plugin_adapter.rs | 55 / 112 (49.1%) | asset fn partially, collection fn 0 | RemoveExternalPluginAdapterV1 (24), RemoveCollectionExternalPluginAdapterV1 (25) |
| src/processor/update_external_plugin_adapter.rs | 121 / 204 (59.3%) | asset fn partially, collection fn 0 | UpdateExternalPluginAdapterV1 (26), UpdateCollectionExternalPluginAdapterV1 (27) |
| src/processor/write_external_plugin_adapter_data.rs | 0 / 194 (0.0%) | 0 / 3 | WriteExternalPluginAdapterDataV1 (28), WriteCollectionExternalPluginAdapterDataV1 (29) |

The three partially covered files owe their coverage entirely to `tests/agent_identity.rs` (AgentIdentity add/update/remove on assets). Every internal-plugin instruction (2-11) and every collection-side external-adapter instruction (23, 25, 27, 29) has zero Mollusk coverage. All of these have extensive JS (AVA) coverage and, for external adapters, Rust-client (`solana-program-test`) coverage, so the bulk of this work is porting.

Shared structure. All nine files follow the same skeleton, so the same "guard" paths recur in each and can be covered by one parameterised test per instruction:

| Guard path (present in every asset/collection fn) | Trigger | Error |
|---|---|---|
| `assert_signer(payer)` | payer account meta `is_signer=false` | `ProgramError::MissingRequiredSignature` |
| `resolve_authority` with explicit authority not signer | authority account present but not signer | `MissingRequiredSignature` |
| `system_program.key != system::ID` | pass any other pubkey at the system-program index | `MplCoreError::InvalidSystemProgram` |
| `log_wrapper` present and `!= SPL_NOOP_ID` | pass a random pubkey (not the program-id sentinel) at the log-wrapper index | `MplCoreError::InvalidLogWrapperProgram` |
| `load_key(asset) == HashedAssetV1` (asset fns only) | account whose byte 0 is `Key::HashedAssetV1` (=2). `load_key` only reads byte 0 and does not check owner, so a 1-byte account suffices | `MplCoreError::NotAvailable` |
| `plugin_type == Groups` (internal-plugin fns only) | `PluginType::Groups` / `Plugin::Groups(..)` in args | `MplCoreError::InvalidPlugin` |

JS tests `it cannot use an invalid system program for assets/collections` and `it cannot use an invalid noop program for assets/collections` exist in `addPlugin.test.ts`, `removePlugin.test.ts`, `updatePlugin.test.ts`, `approveAuthority.test.ts`, `revokeAuthority.test.ts` and can be ported 1:1. The `HashedAssetV1` guard has no JS test (CompressV1 is `NotAvailable`, `src/processor/compress.rs:81`), so it is only reachable with a crafted account; Mollusk makes this trivial.

Authority-resolution semantics that determine success vs. failure in every test below (`src/utils/mod.rs:124-323`, `:324-474`):
- Asset in a collection (`UpdateAuthority::Collection(c)`) must be passed with `collection == c`, else `MissingCollection` / `InvalidCollection`; an asset not in a collection passed with any collection gives `InvalidCollection`.
- `validate_asset_permissions` returns `InvalidAuthority` if any check rejected, `NoApprovals` if nothing approved. `validate_collection_permissions` returns `InvalidAuthority` in both cases.
- The asset/collection core approves add/remove/approve/revoke when the signer is the owner (owner-managed plugin) or the update authority (UA-managed plugin) (`src/state/asset.rs:160-260`). The core never approves update-plugin (`check_update_plugin == None`); update-plugin is approved only by `Plugin::validate_update_plugin` when the signer resolves to the plugin's registry authority (`src/plugins/lifecycle.rs:335-375`).

### src/processor/add_plugin.rs (0/156)

Paths (asset fn `add_plugin`, lines 25-112; collection fn `add_collection_plugin`, 121-198; `process_add_plugin`, 200-220):

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 29-48 | guards (see shared table) | see above | addPlugin.test.ts `it cannot use an invalid system program for assets`, `...noop program for assets` |
| 51-54 | `MasterEdition` or `Groups` on an asset -> `InvalidPlugin` | args.plugin = MasterEdition / Groups | plugins/collection/masterEdition.test.ts `it cannot add masterEdition to asset`; groupsPluginBlocking.test.ts |
| 56, 60-78 | pre-validation: the *new* plugin's own `validate_add_plugin` with `resolved_authorities: None`; `Rejected` -> `InvalidAuthority` (77). Rejecting plugins: PermanentFreezeDelegate, PermanentTransferDelegate, PermanentBurnDelegate, PermanentFreezeExecute, Edition, BubblegumV2 (all "creation-time only"). Error-returning: Royalties (invalid basis points / creators), Autograph / VerifiedCreators (signature not by signer) | permanentFreeze.test.ts `it cannot add permanentFreeze after creation`; permanentTransfer/permanentBurn/permanentFreezeExecute equivalents; edition.test.ts `it cannot add edition plugin after mint`; bubblegumV2.test.ts `it cannot add BubblegumV2 to asset`; royalties.test.ts `it cannot add royalty basis points greater than 10000`; autograph.test.ts `it cannot add autograph plugin to asset by creator` | 
| 81-100 | `validate_asset_permissions` success: owner adds owner-managed plugin; UA adds UA-managed plugin; UA of the collection adds to a member asset; UpdateDelegate (root or additional delegate) adds UA-managed plugin (`update_delegate.rs:66-86`) | addPlugin.test.ts `it can add a plugin to an asset`, `...via update auth`, `...with the collection update authority`, `...via delegate authority`; updateDelegate.test.ts `an updateDelegate can add a plugin to an asset` |
| 81-100 (error) | `NoApprovals`: owner adds UA-managed plugin, or UA adds owner-managed; `InvalidAuthority`: AddBlocker present on asset or collection (`add_blocker.rs:26`), BubblegumV2 collection with non-allow-listed plugin (`bubblegum_v2.rs:42`); `MissingCollection`/`InvalidCollection` | addPlugin.test.ts `it cannot add authority-managed plugin to an asset by owner`, `...if the collection is wrong`, `...if the collection is missing`; plugins/asset/addBlocker.test.ts `it cannot add UA-managed plugin if addBlocker had been added on creation`, `it can add owner-managed plugins even if AddBlocker had been added`; plugins/collection/bubblegumV2.test.ts |
| 103 | `increment_seq_and_save` (no-op unless `asset.seq.is_some()`) | asset built with `seq: Some(n)` to exercise the save branch (`src/state/asset.rs:61-68`) | none (needs crafted asset with seq) |
| 105-111, 200-220 | `process_add_plugin`: `create_meta_idempotent` creates header+registry when `asset.len()==data_len` (first plugin) or loads them (subsequent), then `initialize_plugin` (realloc + append) | first plugin on a bare asset AND second plugin on an asset that already has one | addPlugin.test.ts `it can add a plugin to an asset`, `it can add plugin to asset with a plugin` |
| 105-111 (error) | `PluginAlreadyExists` (`plugins/utils.rs:279-285`) | add FreezeDelegate to an asset that already has one | freeze.test.ts `it cannot add multiple freeze plugins to an asset` |
| 125-139 | collection guards | see shared | addPlugin.test.ts `...for collections` |
| 145-147 | Groups on collection -> `InvalidPlugin` | | groupsPluginBlocking.test.ts |
| 148-166 | pre-validation, `Rejected` -> `InvalidAuthority`: same rejecting plugins as above, plus BubblegumV2 on a collection rejects any non-allow-listed plugin | plugins/collection/permanentFreeze.test.ts `it cannot add permanentFreezeDelegate to collection after creation` (+ permanentBurn/permanentTransfer/permanentFreezeExecute/bubblegumV2 equivalents); bubblegumV2.test.ts `it cannot add non-allow-listed plugins to collection with BubblegumV2 plugin` |
| 169-171 | owner-managed plugin on collection -> `InvalidAuthority` (e.g. FreezeDelegate) | | addPlugin.test.ts `it cannot add an owner-managed plugin to a collection` |
| 174-189 | `validate_collection_permissions`: UA or UpdateDelegate approves; anything else -> `InvalidAuthority`; AddBlocker on collection rejects | addPlugin.test.ts `it can add a plugin to a collection`, `it can add a plugin to a collection with a plugin`; plugins/collection/addBlocker.test.ts; collection/updateDelegate.test.ts `it can add updateDelegate to collection and then approve` |
| 191-197 | process for CollectionV1 (both the fresh-meta and the existing-meta branch) | first plugin on bare collection; second plugin | as above |

Note on line 110 / 196: `init_authority.unwrap_or(manager())` is recomputed rather than reusing `target_plugin_authority`; harmless. Note the `init_authority` path (a non-default plugin authority, e.g. `Authority::Address`) is a distinct branch of `unwrap_or` worth one test: addPlugin.test.ts `it can add a plugin to an asset with a different authority than the default`.

### src/processor/remove_plugin.rs (0/117)

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 22-45 | guards | shared | removePlugin.test.ts `it cannot use an invalid system/noop program for assets` |
| 48-50 | Groups -> `InvalidPlugin` | | groupsPluginBlocking.test.ts |
| 52-58 | asset has no plugin meta -> `PluginNotFound` | bare AssetV1 (no header) | none; new (trivial) |
| 60-61 | `fetch_wrapped_plugin` with `Some(&asset)`: plugin type not in registry -> `PluginNotFound` | asset with Attributes, remove FreezeDelegate | none; new |
| 64-83 | `validate_asset_permissions(check_remove_plugin ...)`. Success: owner removes owner-managed; UA removes UA-managed (with or without collection); collection UA removes from member asset; UpdateDelegate removes UA-managed. Failure: `InvalidAuthority` when plugin authority is `Authority::None` (`lifecycle.rs:249-261`), FreezeDelegate frozen (`freeze_delegate.rs:99`), PermanentFreezeDelegate / PermanentFreezeExecute frozen, Edition or BubblegumV2 (never removable); `NoApprovals` for wrong signer | removePlugin.test.ts `it can remove a plugin from an asset`, `it cannot remove an owner plugin from an asset if not the owner`, `it can remove authority managed plugin from asset in collection using update auth`, `...not in collection...`, `it cannot remove a plugin from a frozen asset`, `it cannot remove a plugin from an asset with a frozen collection`, `it cannot remove an authority managed plugin when the authority is None`, `it cannot remove an owner managed plugin when the authority is None`, `it can remove an owner managed plugin ... after transferring`, `it cannot use an invalid collection to remove a plugin on an asset`; edition.test.ts `it cannot remove edition plugin`; freezeExecuteRemoval.test.ts `it cannot remove FreezeExecute while frozen`; updateDelegate.test.ts `an updateDelegate can remove a plugin from an asset` |
| 86 | seq increment | crafted `seq: Some` | none |
| 88-94, 170-178 | `delete_plugin`: memmove of trailing plugins + registry, `bump_offsets`, shrink realloc. Needs the removed plugin to be *not last* to exercise `data_to_move > 0` and offset bumping | asset with [Attributes, FreezeDelegate, TransferDelegate], remove the first | removePlugin.test.ts `it can remove a plugin from asset with existing plugins`; appData.test.ts `Data offsets are correctly bumped when removing other plugins` (also checks external records get bumped) |
| 103-121 | collection guards | | removePlugin.test.ts `...for collections` |
| 124-126 | Groups -> `InvalidPlugin` | | |
| 128-134 | no meta -> `PluginNotFound` | bare CollectionV1 | new |
| 136-140 | plugin not present -> `PluginNotFound` | new |
| 143-158 | `validate_collection_permissions`: UA or UpdateDelegate approve; PermanentFreezeDelegate/PermanentFreezeExecute frozen reject; BubblegumV2 rejects self-removal; UpdateDelegate additional delegate cannot remove UpdateDelegate | removePlugin.test.ts `it can remove authority managed plugin from collection`, `...using delegate auth`, `it cannot remove authority managed collection plugin if the delegate authority is not update authority`; plugins/collection/permanentFreeze.test.ts `it cannot remove permanentFreezeDelegate from collection when frozen`, `it can remove permanentFreezeDelegate from collection`; bubblegumV2.test.ts `Update Authority cannot remove BubblegumV2 from collection` |
| 160-167 | process for CollectionV1 | | as above |

### src/processor/update_plugin.rs (0/166)

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 24-48 | guards | shared | updatePlugin.test.ts `it cannot use an invalid system/noop program for assets` |
| 51-53 | Groups -> `InvalidPlugin` | | groupsPluginBlocking.test.ts |
| 55-56 | `fetch_wrapped_plugin(asset, None, ..)`: bare asset or missing plugin -> `PluginNotFound` | new (trivial) |
| 58-77 | `validate_asset_permissions(check_update_plugin ...)`. Core abstains; only `Plugin::validate_update_plugin` approves, when `resolved_authorities` contains the registry authority of the plugin being updated. Success: owner updates FreezeDelegate (Owner authority); UA updates Attributes; delegate `Address(x)` updates the plugin it holds; UpdateDelegate (root or additional) updates any plugin whose authority is `UpdateAuthority` (`update_delegate.rs:208-236`). Failure: `NoApprovals` for anyone else; `InvalidAuthority` when a *different* plugin on the asset or collection rejects (Royalties invalid data `royalties.rs:153`; VerifiedCreators / Autograph signature rules; UpdateDelegate additional delegate trying to change other delegates `update_delegate.rs:217-225`) | updatePlugin.test.ts (all 11 tests, incl. the four "Owner authority plugin vs UpdateAuthority plugin present on asset/collection" cases and `it cannot use an invalid collection to update a plugin on an asset`); freeze.test.ts `it can freeze and unfreeze an asset`, `it owner cannot unfreeze frozen asset`, `it update authority cannot unfreeze frozen asset`; attributes.test.ts; royalties.test.ts `it cannot update royalty basis points greater than 10000`, `...duplicate creators`; edition.test.ts `it can update edition plugin`, `it cannot update edition plugin as owner`; updateDelegate.test.ts `it can update updateDelegate on asset with additional delegates`, `it can remove additional delegate as additional delegate if self`, `it cannot remove another additional delegate as additional delegate`, `it can update a non-updateDelegate plugin as additional delegate`; autograph.test.ts `it can add additional autograph to asset via update by 3rd party`, `it cannot modify autograph message as signer`; verifiedCreators.test.ts `it can unverify signature verified creator plugin` |
| 80 | seq increment | crafted `seq: Some` | none |
| 82-90, 160-243 | `process_update_plugin`. Branches: `size_diff > 0` grow (211-214: realloc first, then memmove 216-228); `size_diff < 0` shrink (216-228 memmove, then 230-233 realloc); `size_diff == 0` (no realloc, memmove no-op). `copy_len > 0` (219) requires a plugin *after* the updated one. `bump_offsets` (237) must move both internal and external records | Attributes grown / shrunk / same size with FreezeDelegate and an AppData adapter behind it | clients/rust/tests/plugin_shrink_corruption.rs `test_update_plugin_shrink_attributes_preserves_trailing_plugins`, `test_update_plugin_shrink_attributes_preserves_external_plugin`; appData.test.ts `updating a plugin before a secure app data does not corrupt the data`; oracle.test.ts `it can update asset to different size name with oracle` (asset-level, different processor) |
| 169-170, 174-178 | `PluginsNotInitialized` / `PluginNotFound` inside `process_update_plugin` | dead: `fetch_wrapped_plugin` at line 55/125 already fails with `PluginNotFound` when meta or plugin is absent | n/a |
| 99-158 | collection variant: guards (104-118), Groups (121-123), fetch (125-129), `validate_collection_permissions` (132-147; only the plugin's own authority or UpdateDelegate approve, else `InvalidAuthority`), process (149-157) | UA updates Attributes on collection; UpdateDelegate on collection updates; wrong signer | updatePlugin.test.ts `...for collections` guards; plugins/collection/updateDelegate.test.ts `it can update updateDelegate on collection with additional delegates`; royalties.test.ts collection variants; plugin_shrink_corruption.rs (collection asset-update analogue only) |

### src/processor/approve_plugin_authority.rs (0/118)

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 25-48 | guards | shared | approveAuthority.test.ts `it cannot use an invalid system/noop program for assets` |
| 51-53 | Groups -> `InvalidPlugin` | | groupsPluginBlocking.test.ts |
| 55-56 | `fetch_wrapped_plugin(asset, None, type)` -> `PluginNotFound` | bare asset / missing plugin | new |
| 59-78 | `validate_asset_permissions(check_approve_plugin_authority ...)`. `Plugin::validate_approve_plugin_authority` (`lifecycle.rs:264-280`) returns `CannotRedelegate` when the target plugin's registry authority != its manager (already delegated). Core approves owner->owner-managed / UA->UA-managed. FreezeDelegate rejects while frozen (`freeze_delegate.rs:65`); FreezeExecute likewise; UpdateDelegate approves for UA-managed plugins except UpdateDelegate itself (`update_delegate.rs:111-136`) | approveAuthority.test.ts `it can add an authority to a plugin`, `it cannot reassign authority of a plugin while already delegated`, `...as delegate while already delegated`, `it cannot approve to reassign authority back to owner`; freeze.test.ts `owner cannot approve to reassign authority back to owner if frozen`; freezeExecuteRemoval.test.ts `it cannot approve a new authority for FreezeExecute as the owner while frozen`; updateDelegate.test.ts `it cannot approve the update delegate plugin authority as additional delegate`, `it can approve/revoke the plugin authority of non-updateDelegate plugins as additional delegate`, `it can approve/revoke the plugin authority of other plugins`; delegate.test.ts `it can delegate a new authority` |
| 81 | seq increment | crafted | none |
| 83-89, 154-182 | `process_approve_plugin_authority`: re-fetch, then `approve_authority_on_plugin` (registry authority swap; realloc only when `size_diff != 0`, i.e. Owner/UA/None (1 byte) <-> Address (33 bytes)) | approve `Address(x)` (grow) and approve `UpdateAuthority` over an Owner-managed plugin (same size, no realloc) | approveAuthority.test.ts `it can add an authority to a plugin`; freeze.test.ts `it can delegate then freeze an asset` |
| 163-171 | `PluginsNotInitialized` in process fn | dead: fetch at 55-56 fails first | n/a |
| 99-152 | collection variant (guards 106-117, Groups 120-122, fetch 124-125, `validate_collection_permissions` 128-143, process 145-151) | UA approves Address on collection Attributes; wrong signer -> `InvalidAuthority`; already delegated -> `CannotRedelegate` | approveAuthority.test.ts `...for collections`; plugins/collection/updateDelegate.test.ts `it can add updateDelegate to collection and then approve` |

### src/processor/revoke_plugin_authority.rs (0/136)

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 28-51 | guards | shared | revokeAuthority.test.ts `it cannot use an invalid system/noop program for assets` |
| 54-56 | Groups -> `InvalidPlugin` | | groupsPluginBlocking.test.ts |
| 58-62 | `fetch_core_data` then `fetch_wrapped_plugin(asset, Some(&asset), type)`. Missing plugin -> `PluginNotFound`. **Bare asset (no meta) panics** (see findings) | new |
| 65-84 | `validate_asset_permissions(check_revoke_plugin_authority ...)`. `Plugin::validate_revoke_plugin_authority` (`lifecycle.rs:282-316`): rejects if the plugin's authority is `None`; approves if the signer resolves to the plugin's current authority (a delegate can revoke itself). Core approves owner/UA for their managed plugins. FreezeDelegate: rejects while frozen, approves the delegate when unfrozen (`freeze_delegate.rs:78-96`). UpdateDelegate: approves only for UA-managed plugins (security fix, `update_delegate.rs:138-163`) | revokeAuthority.test.ts (all 12 tests: `it can remove an authority from a plugin`, `...default authority ... immutable`, `...pubkey authority from an owner-managed plugin if that pubkey is the signer`, `...update authority from an owner-managed plugin...`, `...pubkey authority from an authority-managed plugin...`, `...owner authority from an authority-managed plugin...`, `it cannot remove a none authority from a plugin`, `it can revoke an authority from a plugin if another plugin is None`); freeze.test.ts `owner cannot undelegate a freeze plugin with a delegate`, `it delegate cannot freeze after delegate has been revoked`; updateDelegateRevokeBug.test.ts (all 5: UpdateDelegate must NOT revoke owner-managed FreezeDelegate/TransferDelegate, may revoke UA-managed); updateDelegate.test.ts `it cannot revoke the update delegate plugin authority as additional delegate`; freezeExecuteRemoval.test.ts `it cannot revoke FreezeExecute as the owner while frozen` |
| 87 | seq increment | crafted | none |
| 89-95 | payer selection: if signer resolves to `plugin.manager()` the payer receives the rent refund (91-92), else `ctx.accounts.asset` is used as "payer" (93-94) - i.e. the lamports freed by shrinking `Address` -> 1-byte authority stay in the asset | (a) owner revokes delegate on FreezeDelegate; (b) delegate `Address(x)` revokes itself | revokeAuthority.test.ts `it can remove a pubkey authority from an owner-managed plugin if that pubkey is the signer authority` (b), `it can remove an authority from a plugin` (a); add lamport assertions |
| 97-104, 184-210 | `process_revoke_plugin_authority` -> `revoke_authority_on_plugin` (authority reset to manager, realloc if size changed) | as above; also revoke of a same-size authority (`UpdateAuthority` on owner-managed) to hit the no-op realloc | revokeAuthority.test.ts `it can remove an update authority from an owner-managed plugin...` |
| 192-200 | `PluginsNotInitialized` in process fn | dead: header is always `Some` here because a bare asset panics at 61-62 first | n/a |
| 113-181 | collection variant (guards 118-131, Groups 134-136, fetch 138-145, `validate_collection_permissions` 148-163, payer selection 165-171, process 173-180) | UA revokes Address on collection plugin; delegate revokes itself (asset-as-payer branch 170); `None` authority -> `InvalidAuthority` | revokeAuthority.test.ts `...for collections` guards; plugins/collection/updateDelegate.test.ts `an updateDelegate on collection cannot update an asset after delegate authority revoked` (revoke step) |

### src/processor/add_external_plugin_adapter.rs (101/180)

Covered today (by `tests/agent_identity.rs`): asset happy path with AgentIdentity, `validate_asset_permissions` success, `process_add_external_plugin_adapter::<AssetV1>`, and the collection fn up to the AgentIdentity rejection at 172-174.

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 39, 43-45, 49-50 | guards (system program, log wrapper, HashedAssetV1) | shared | none for adapters; reuse the shared guard test |
| 57 | `LinkedLifecycleHook` / `LinkedAppData` init on an asset -> `InvalidPluginAdapterTarget` | | linkedAppData.test.ts `it cannot add linked app data to an asset` |
| 60 | `DataSection` init on an asset -> `CannotAddDataSection` | | dataSection.test.ts `it cannot add a DataSection to an asset` |
| 67-68 | `LifecycleHook` init authority extraction; proceeds to `initialize_external_plugin_adapter` which returns `NotAvailable` (`plugins/utils.rs:335-341`) | add LifecycleHook to asset | clients/rust/tests/add_external_plugins.rs `test_temporarily_cannot_add_lifecycle_hook` |
| 70 | `Oracle` init authority | add Oracle to asset (success; also `RequiresLifecycleCheck` / `OracleCanRejectOnly` / `DuplicateLifecycleChecks` failures from `validate_lifecycle_checks`) | oracle.test.ts `it can add oracle to asset for multiple lifecycle events`, `it cannot add oracle with no lifecycle checks to asset`, `it cannot add oracle to asset that can approve`, `...that can listen`; add_external_plugins.rs `test_add_oracle`, `test_cannot_add_oracle_with_duplicate_lifecycle_checks` |
| 71 | `AppData` init authority | add AppData to asset; duplicate -> `ExternalPluginAdapterAlreadyExists` | add_external_plugins.rs `test_add_app_data`, `test_cannot_add_duplicate_external_plugin_adapter`; appData.test.ts (`DATA_AUTHORITIES x SCHEMAS` create loop) |
| 72-76 | `LinkedLifecycleHook` / `LinkedAppData` / `DataSection` arms of the authority match | **dead**: all three returned at 55-61 (`unreachable!()` at 76 is correct) | n/a |
| 104 | `validate_add_external_plugin_adapter == Rejected` -> `InvalidAuthority` | **effectively unreachable on the asset path**: LifecycleHook/Oracle/AppData abstain, AgentIdentity errors (never rejects) when `asset_info` is `Some`, DataSection is rejected earlier at 60 | n/a |
| 128 | `validate_asset_permissions` failure: `NoApprovals` (signer is not UA), `MissingCollection`/`InvalidCollection`; BubblegumV2 on the *collection* abstains for assets (`bubblegum_v2.rs:82-90`) so no `InvalidAuthority` source | non-UA signer adds Oracle | oracle.test.ts (indirect); new test is simplest |
| 159, 163-165 | collection guards | | new |
| 170 | `DataSection` on collection -> `CannotAddDataSection` | | dataSection.test.ts `it cannot add a DataSection to a collection` |
| 175-190 | collection init-authority match: LifecycleHook (180-181, then `NotAvailable`), Oracle (183), AppData (184), LinkedLifecycleHook (185-186, then `NotAvailable`), LinkedAppData (188). 189-190 `unreachable!()` is dead (returned at 168-176) | add each type to a collection | add_external_plugins.rs `test_temporarily_cannot_add_lifecycle_hook_on_collection`; oracle.test.ts `it can create an oracle on a collection with create set to reject` (create path; add variant new); linkedAppData.test.ts `it can update linked app data on collection...` (uses createCollection; add variant new) |
| 192-216 | collection pre-validation; `Rejected` -> `InvalidAuthority` (215) | BubblegumV2 on the collection rejects every adapter (`bubblegum_v2.rs:82-99`) - this is the *only* reachable `Rejected` source and it is reached via `validate_collection_permissions` (221-236) not this pre-check, because the pre-check calls the *new adapter's* validate, which abstains. So 215-216 is effectively unreachable | plugins/collection/bubblegumV2.test.ts `it cannot add external plugin to collection with BubblegumV2 plugin` covers 236 (error branch) |
| 218, 221-242 | `validate_collection_permissions` success (UA) and failure; `process_add_external_plugin_adapter::<CollectionV1>` (both fresh-meta and existing-meta branches) | UA adds Oracle / AppData / LinkedAppData to a bare collection and to one with an internal plugin | create_collection_with_external_plugins.rs + add analogues (new) |

### src/processor/remove_external_plugin_adapter.rs (55/112)

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 39, 43-45, 49-50 | guards | shared | new (shared guard test) |
| 57 | no plugin meta -> `PluginNotFound` | bare asset | new |
| 64 | key not found -> `ExternalPluginAdapterNotFound` | asset with Oracle(a), remove Oracle(b) / AppData(Owner) | new |
| 86 | `validate_asset_permissions` failure (`NoApprovals` non-UA signer; `MissingCollection`/`InvalidCollection`) | owner (not UA) removes AppData | new; remove_external_plugins.rs `test_remove_*` are success-only |
| 88-94 (covered) plus `delete_external_plugin_adapter` variants | removing an adapter *with data* (AppData with written data: `data_len` added to the moved size, `plugins/utils.rs:708-712`) and one that is not last (offset bumping) | asset with [AppData(x)+data, Oracle], remove AppData | appData.test.ts `Data offsets are correctly bumped when removing other external plugins with data`; linkedAppData.test.ts `Data offsets are correctly bumped when removing Data Section with data`; remove_external_plugins.rs `test_remove_oracle`, `test_remove_app_data` |
| 104-163 | collection variant: guards, no-meta -> `PluginNotFound` (132-134), not found (136-140), `validate_collection_permissions` (143-158: UA approves, else `InvalidAuthority`), process for CollectionV1 (160-166) | UA removes Oracle/AppData/LinkedAppData from collection; owner-signer fails | remove_external_plugins_on_collection.rs `test_remove_oracle_on_collection`, `test_remove_app_data_on_collection` |

### src/processor/update_external_plugin_adapter.rs (121/204)

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 45, 49-51, 55-56 | guards | shared | new |
| 59-66 (covered) error branches | `fetch_wrapped_external_plugin_adapter(asset, None, key)` -> `ExternalPluginAdapterNotFound`; `incoming.update(&update_info)` -> `InvalidPlugin` when key variant and update_info variant differ (`external_plugin_adapters.rs:151-194`) | key Oracle(x) with `AppData` update info | new |
| 90 | `validate_update_external_plugin_adapter != Approved` -> `InvalidAuthority`: signer does not resolve to the registry record's authority (`external_plugin_adapters.rs:368-428`) | asset UA tries to update an adapter whose authority is `Address(other)` | oracle.test.ts `it cannot update oracle using update authority when different from external plugin authority`; appData.test.ts `it cannot update app data using update authority when different from external plugin authority` |
| 96-106 (covered) `process_update_external_plugin_adapter` error branches | `registry_record.update` -> `RequiresLifecycleCheck` / `OracleCanRejectOnly` / `DuplicateLifecycleChecks` (`plugin_registry.rs:186-214`) | oracle update with empty / approving / duplicate checks | oracle.test.ts `it cannot update oracle to have no lifecycle checks`, `it cannot update oracle to approve`, `it cannot update oracle to listen`; update_external_plugins.rs `test_cannot_update_oracle_to_have_duplicate_lifecycle_checks` |
| 269-276 | shrink: `plugin_size_diff < 0 && copy_len > 0` memmove before realloc | leading Oracle shrunk (e.g. drop `base_address_config`) with a trailing Oracle/AppData | oracle.test.ts `it can shrink a leading oracle without corrupting trailing oracle metadata`, `it can update oracle to smaller registry record`; appData.test.ts `Data offsets are correctly bumped when rewriting other external plugins to be smaller` |
| 283-290 | grow: `plugin_size_diff > 0 && copy_len > 0` memmove after realloc | leading Oracle grown with trailing adapter | oracle.test.ts `it can grow a leading oracle without corrupting trailing oracle metadata`, `it can update oracle to larger registry record`; appData.test.ts `...rewriting other external plugins to be larger` |
| 118-188 | collection variant: guards (126-137), fetch (139-150), validation ctx (152-167), `InvalidAuthority` (169-175), process for CollectionV1 (177-187) | UA / plugin-authority updates Oracle, AppData (schema), LinkedAppData (schema) on a collection; wrong signer fails | update_external_plugins_on_collection.rs `test_update_oracle_on_collection`, `test_update_app_data_on_collection`, `test_cannot_update_oracle_to_have_duplicate_lifecycle_checks_on_collection`; oracle.test.ts `it can update oracle on collection with external plugin authority different...`, `it cannot update oracle on collection using update authority...`; appData.test.ts / linkedAppData.test.ts collection update tests |

Note: `registry_record_size_diff` (208-214) is non-zero only when `lifecycle_checks` length changes (Oracle/LifecycleHook/AgentIdentity); AgentIdentity tests already hit the non-zero case (`update_agent_identity_lifecycle_checks`). An Oracle test that changes the number of checks *and* the plugin body size covers both diffs together.

### src/processor/write_external_plugin_adapter_data.rs (0/194)

Only `AppData` (asset/collection) and `LinkedAppData` (asset, plugin on collection) are writable in production, because `LifecycleHook`/`LinkedLifecycleHook` cannot be initialised (`NotAvailable`). Guard order in this file differs from the others: `fetch_core_data`/`resolve_pubkey_to_authorities` (46-48) run *before* the system-program / log-wrapper / HashedAssetV1 checks.

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 36-44 | signer guards | shared | new |
| 46 | `fetch_core_data::<AssetV1>` failure (wrong key/owner) | non-asset account | account_ownership-style test |
| 48 | `resolve_pubkey_to_authorities`: `MissingCollection` (asset in collection, none passed) / `InvalidCollection` (wrong collection) | | new |
| 50-52, 54-58 | system program / log wrapper guards | shared | new |
| 60-63 | HashedAssetV1 -> `NotAvailable` | **dead**: `AssetV1::load` at line 46 already fails with `DeserializationError` for key `HashedAssetV1` | n/a |
| 65-68 | key `LifecycleHook(_)` / `AppData(_)`: fetch from asset; `ExternalPluginAdapterNotFound` when absent | write AppData(Owner) on asset lacking it | appData.test.ts write loop (success); not-found is new |
| 69-83 | key `LinkedLifecycleHook`/`LinkedAppData`: collection required (`MissingCollection`, 74-77); membership check (79-80, PR #19) -> `InvalidCollection`; fetch from collection (82) | (a) asset *not in any collection*, no collection passed -> `MissingCollection` at 77 (an asset in a collection with no collection passed already fails at line 48 with the same error); (b) asset *not in any collection* + attacker collection with LinkedAppData -> `InvalidCollection` at 80 (an asset in a *different* collection already fails at line 48 with the same error, so 77 and 80 are reachable only for non-collection assets); (c) member asset -> success | linkedAppDataMembership.test.ts (all 3); linkedAppData.test.ts write loop |
| 84-85 | key `Oracle` / `DataSection` / `AgentIdentity` -> `UnsupportedOperation` | | new |
| 87-100, 160-175 | dispatch into `process_write_external_plugin_data::<AssetV1>` | | |
| 176-188 | data-authority check: `AppData`/`LinkedAppData` (`data_authority` any) and `LifecycleHook`/`LinkedLifecycleHook` (only when `data_authority: Some`); `!authorities.contains(data_authority)` -> `InvalidAuthority`. `Authority::None` never matches. `Owner`/`UpdateAuthority`/`Address` all need one success case each | appData.test.ts `it cannot write data to a secure app data with X data authority using Y data authority`, `...if the data authority is None`; same in linkedAppData.test.ts; the `DATA_AUTHORITIES` loop gives Owner/UpdateAuthority/Address success |
| 190 | `_ => UnsupportedOperation` (Oracle/DataSection/AgentIdentity, or hook with `data_authority: None`) | reachable in production only via the *collection* instruction (line 137-141 fetches any key from the collection): write with key `Oracle(x)` on a collection holding that oracle | new |
| 195-223 | `LifecycleHook | AppData`: `PluginsNotInitialized` (197-198, unreachable: adapter was just fetched), then `(data, buffer)` match: inline data (200-209), buffer account (210-219), both -> `TwoDataSources` (220), neither -> `NoDataSources` (221). `update_external_plugin_adapter_data` grows / shrinks / keeps size | write 0 -> N bytes, N -> M<N, N -> N; via buffer; both; neither | appData.test.ts `it can write ... data ... multiple times` (grow+shrink), Rust plugin_shrink_corruption.rs `test_write_external_plugin_adapter_data_shrink_preserves_second_plugin`, `..._single_plugin_shrink`; buffer / TwoDataSources / NoDataSources: new (JS SDK always uses inline data) |
| 224-226 | `LinkedAppData`: `create_meta_idempotent::<T>` on the asset (creates header+registry when the asset has no plugins yet - the DataSection is the first "plugin") | member asset with no plugins; member asset with an existing plugin | linkedAppData.test.ts write loop covers the first; second is new |
| 228-258 | DataSection already exists -> update data (inline 236-245 / buffer 246-255 / Two 256 / No 257) | second write | linkedAppData.test.ts `...multiple times`; `Data offsets are correctly bumped when rewriting Data Section to be smaller/larger` |
| 259-295 | DataSection missing (`ExternalPluginAdapterNotFound`) -> `initialize_external_plugin_adapter` with `DataSectionInitInfo { parent_key: LinkedAppData(data_authority), schema }` and appended data (inline 266-278 / buffer 279-291 / Two 292 / No 293) | first write | linkedAppData.test.ts write loop; buffer/Two/No new |
| 296 | other fetch error propagated | corrupt registry only; skip | n/a |
| 299 | `_ => UnsupportedOperation` (`LinkedLifecycleHook` with `data_authority: Some`) | only with a crafted collection holding a LinkedLifecycleHook | crafted-account test or leave uncovered |
| 112-157 | collection variant: guards; `fetch_wrapped_external_plugin_adapter::<CollectionV1>` any key (137-141); process for CollectionV1 | write AppData on collection (success, wrong data authority, buffer/Two/No); write LinkedAppData key on collection (see finding 3) | appData.test.ts `it can update app data on collection...` is an *update* test; collection *write* tests: none in JS (`generateTestContext` is asset-only) - new; Rust `create_collection_with_external_plugins.rs::test_create_and_fetch_app_data_on_collection` is create-only |

### Dead / special-harness code summary

- Dead (unreachable by construction): add_external_plugin_adapter.rs:72-76, :189-190, :104, :215-216; write_external_plugin_adapter_data.rs:60-63, :197-198; update_plugin.rs:169-170,178; approve_plugin_authority.rs:163-171; revoke_plugin_authority.rs:192-200.
- Reachable only with crafted accounts (no production path): every `HashedAssetV1` guard (CompressV1 is disabled); all `LifecycleHook`/`LinkedLifecycleHook` arms in write_external_plugin_adapter_data.rs:176-183, :196, :299 (hooks return `NotAvailable` at init). Mollusk can craft these accounts directly, so coverage is attainable; decide per line whether it is worth asserting behaviour that cannot occur on chain.
- Needs no CPI into another program: none of these processors invoke lifecycle-hook programs or read oracle accounts (only the System Program for realloc, which the harness already supports).
- AgentIdentity paths in add/update/remove are already covered by `tests/agent_identity.rs`.

### Findings (bugs / suspicious logic / security-relevant)

1. **Panic instead of error in RevokePluginAuthorityV1 / RevokeCollectionPluginAuthorityV1 on an account without plugin meta** (`revoke_plugin_authority.rs:61-62`, `:141-145`). `fetch_wrapped_plugin(.., Some(&asset), ..)` skips the `asset.len() == data_len` check (`plugins/utils.rs:156-170`) and calls `PluginHeaderV1::load(account, asset.len())`, which indexes `data[offset]` at `offset == data_len` (`utils/mod.rs:29-34`) - out-of-bounds panic, surfacing as `ProgramFailedToComplete` rather than `PluginNotFound`. Remove/RemoveExternal guard this with `plugin_header.is_none()` first; Approve/Update/UpdateExternal pass `None` and get the clean error. Not exploitable (caller's own tx fails), but inconsistent, and a good Mollusk regression target.
2. **WriteCollectionExternalPluginAdapterDataV1 with a `LinkedAppData` key writes a DataSection onto the collection itself** (`write_external_plugin_adapter_data.rs:137-141` fetches any key from the collection, then `:224-295` runs `create_meta_idempotent`/`initialize_external_plugin_adapter(DataSection)` on the collection account). The asset variant restricts Linked keys to the collection-plugin/asset-data pattern; the collection variant has no such restriction. Requires the LinkedAppData `data_authority`, so not an auth bypass, but it produces a collection-level DataSection that no reader expects. Worth an explicit test either way (document or reject).
3. **Membership check ordering** (`write_external_plugin_adapter_data.rs:48` vs `:79-80`): `resolve_pubkey_to_authorities` already rejects a mismatched collection for assets that *are* in a collection, so the PR #19 check at line 80 is the only defence for assets *not* in any collection (`UpdateAuthority::Address`/`None`). Both return `InvalidCollection`; tests should cover both shapes so a future refactor of line 48 does not silently remove the protection.
4. `update_external_plugin_adapter` (both variants) never calls `validate_asset_permissions`/`validate_collection_permissions`; authority is decided solely by the adapter's registry authority (`:85-91`). Collection-level internal plugins therefore cannot veto adapter updates. Consistent with `check_update_external_plugin_adapter == None` for all plugins (`lifecycle.rs:231-237`), but note that an asset not in a collection accepts any `collection` account here without complaint (ignored by `resolve_pubkey_to_authorities`), unlike the add/remove paths which return `InvalidCollection`.
5. `revoke_plugin_authority.rs:89-95`: when a delegate revokes itself, the rent freed by shrinking the registry record (`Address` 33 B -> 1 B) is "paid" to the asset account (`resize_or_reallocate_account` adds and subtracts the same lamports on the same account, `utils/account.rs:64-70`), so the lamports remain locked in the asset until burn. Behavioural quirk, not a vulnerability; assert it in the test so it is deliberate.
6. `add_plugin.rs:60-78` pre-validation runs the new plugin's own `validate_add_plugin` with `resolved_authorities: None`. This is the only place Royalties/Autograph/VerifiedCreators data is validated on add (the registry-driven `validate_plugin_checks` only consults plugins already present). Tests for invalid Royalties (basis points > 10000, creators != 100%, duplicate creators) on both asset and collection are therefore load-bearing.
7. `write_external_plugin_adapter_data.rs:210-219, :246-255, :279-291`: the `buffer` account is read with no owner, size, or type check. Intended (caller-supplied bytes), but the buffer path has zero coverage in any client test suite.

### Test plan

Naming: `pm_` = plugin management. "Setup" abbreviations: A = bare AssetV1 (no plugin meta), A+[..] = asset with listed internal plugins (authority in parentheses), C = bare CollectionV1, C+[..], A@C = asset whose update_authority is `Collection(C)`, X = external adapter. Sizes: S = one instruction, one crafted account, error assertion; M = success path with post-state assertions on header/registry/plugin bytes; L = multi-plugin layout with offset/bytes integrity assertions.

| Test case | Setup | Instruction(s) | Expected | Source paths covered | New / port from | Size |
|---|---|---|---|---|---|---|
| pm_guards_all_instructions (parameterised over 2-11, 22-29) | A, C | each ix with (a) payer non-signer, (b) bad system program, (c) bad log wrapper, (d) 1-byte `[2]` account as asset (asset ixs) | MissingRequiredSignature / InvalidSystemProgram / InvalidLogWrapperProgram / NotAvailable (ix 28 gives DeserializationError instead, see dead code) | add_plugin.rs:29-48,125-139; remove_plugin.rs:26-45,107-121; update_plugin.rs:29-48,104-118; approve:29-48,103-117; revoke:32-51,117-131; add_ext:35-50,155-165; remove_ext:35-51,109-119; update_ext:41-57,126-137; write:39-58,119-135 | port: `it cannot use an invalid system/noop program` x5 files; HashedAssetV1 new | M (one helper, many cases) |
| pm_groups_rejected (parameterised over 2-11) | A+[Attributes], C+[Attributes] | Groups plugin/type | InvalidPlugin | add_plugin.rs:51-54,145-147; remove:48-50,124-126; update:51-53,121-123; approve:51-53,120-122; revoke:54-56,134-136 | port: groupsPluginBlocking.test.ts | S |
| pm_add_master_edition_to_asset | A | AddPluginV1 MasterEdition | InvalidPlugin | add_plugin.rs:51-54 | port: masterEdition.test.ts `it cannot add masterEdition to asset` | S |
| pm_add_first_plugin_owner | A (owner=payer) | AddPluginV1 FreezeDelegate | Ok; header+registry created, record authority Owner | add_plugin.rs:56-112,200-220 (fresh-meta branch) | port: `it can add a plugin to an asset` | M |
| pm_add_second_plugin_ua_with_init_authority | A+[FreezeDelegate(Owner)], UA=payer | AddPluginV1 Attributes, init_authority=Address(x) | Ok; existing-meta branch; record authority Address(x) | add_plugin.rs:105-111,200-220 | port: `it can add plugin to asset with a plugin`, `...different authority than the default` | M |
| pm_add_plugin_wrong_manager | A | owner adds Attributes; UA adds FreezeDelegate | NoApprovals (x2) | add_plugin.rs:81-100 error | port: `it cannot add authority-managed plugin to an asset by owner`, `it cannot add a owner-managed plugin to an asset via delegate authority` | S |
| pm_add_plugin_asset_in_collection | A@C, C UA=payer; also missing / wrong collection | AddPluginV1 Attributes | Ok; MissingCollection; InvalidCollection | add_plugin.rs:81-100 | port: `it can add a plugin to an asset that is part of a collection`, `...if the collection is wrong`, `...missing` | M |
| pm_add_plugin_via_update_delegate | A+[UpdateDelegate(UA, additional=[d])] | AddPluginV1 Attributes signed by d; then FreezeDelegate signed by d | Ok; NoApprovals | add_plugin.rs:81-100; update_delegate.rs:66-86 | port: updateDelegate.test.ts `an updateDelegate can add a plugin to an asset` | M |
| pm_add_plugin_rejected_by_add_blocker | A+[AddBlocker], C+[AddBlocker] with A@C | AddPluginV1 Attributes (UA); AddPluginV1 FreezeDelegate (owner) | InvalidAuthority; Ok | add_plugin.rs:81-100; add_blocker.rs:26-40 | port: addBlocker.test.ts (asset+collection) | M |
| pm_add_creation_only_plugins_rejected (param: PermanentFreeze/Transfer/Burn/FreezeExecute, Edition, BubblegumV2) | A, C | AddPluginV1 / AddCollectionPluginV1 | InvalidAuthority (line 77 / 165) | add_plugin.rs:60-78,148-166 | port: `it cannot add permanentX after creation`, `it cannot add edition plugin after mint`, `it cannot add BubblegumV2 to asset/collection` | S |
| pm_add_royalties_invalid_data | A, C | Royalties bps 10001; creators sum != 100; duplicate creators | error from validate_royalties | add_plugin.rs:76 (`?`), :164 | port: royalties.test.ts `it cannot add royalty ...` x3 | S |
| pm_add_duplicate_plugin | A+[FreezeDelegate] | AddPluginV1 FreezeDelegate | PluginAlreadyExists | add_plugin.rs:105-111 error | port: freeze.test.ts `it cannot add multiple freeze plugins to an asset` | S |
| pm_add_collection_plugin_ua | C (bare), then C+[Attributes] | AddCollectionPluginV1 Attributes; then Royalties | Ok both; header creation then existing meta | add_plugin.rs:141-198,200-220 (CollectionV1) | port: `it can add a plugin to a collection`, `...with a plugin` | M |
| pm_add_collection_plugin_owner_managed | C | AddCollectionPluginV1 FreezeDelegate | InvalidAuthority (line 170) | add_plugin.rs:169-171 | port: `it cannot add an owner-managed plugin to a collection` | S |
| pm_add_collection_plugin_wrong_signer / bubblegum | C (random signer); C+[BubblegumV2] add Attributes (non-allow-listed) | AddCollectionPluginV1 | InvalidAuthority | add_plugin.rs:174-189 error, :164-166 | port: bubblegumV2.test.ts `it cannot add non-allow-listed plugins...`; new for wrong signer | S |
| pm_remove_plugin_no_meta / not_found | A; A+[Attributes] | RemovePluginV1 FreezeDelegate | PluginNotFound (56-58); PluginNotFound (60-61) | remove_plugin.rs:52-61 | new | S |
| pm_remove_plugin_owner_and_ua | A+[FreezeDelegate(Owner), Attributes(UA)] | owner removes FreezeDelegate; UA removes Attributes; owner removes Attributes | Ok; Ok; NoApprovals | remove_plugin.rs:64-94,170-178 | port: `it can remove a plugin from an asset`, `it cannot remove an owner plugin from an asset if not the owner`, `...not in collection using update auth` | M |
| pm_remove_plugin_middle_with_external | A+[Attributes, FreezeDelegate, TransferDelegate] + X AppData with data | RemovePluginV1 Attributes | Ok; trailing plugins and external record offsets/data intact | remove_plugin.rs:88-94; delete_plugin memmove+bump | port: `it can remove a plugin from asset with existing plugins`; appData.test.ts `Data offsets are correctly bumped when removing other plugins` | L |
| pm_remove_plugin_frozen / none_authority | A+[FreezeDelegate(frozen)]; A+[Attributes(None)]; A@C with C+[PermanentFreezeDelegate(frozen)] | RemovePluginV1 | InvalidAuthority x3 | remove_plugin.rs:64-83 error | port: `it cannot remove a plugin from a frozen asset`, `...with a frozen collection`, `...when the authority is None` | S |
| pm_remove_plugin_in_collection_and_delegate | A@C (C UA=payer); A+[UpdateDelegate(UA, additional=[d])] | RemovePluginV1 Attributes by collection UA; by d | Ok; Ok | remove_plugin.rs:64-94 | port: `...asset in collection using update auth`; updateDelegate.test.ts `an updateDelegate can remove a plugin from an asset` | M |
| pm_remove_collection_plugin | C (bare); C+[Attributes]; C+[PermanentFreezeDelegate(frozen)]; C+[BubblegumV2] | RemoveCollectionPluginV1 | PluginNotFound; Ok; InvalidAuthority; InvalidAuthority | remove_plugin.rs:103-167,170-178 (CollectionV1) | port: `it can remove authority managed plugin from collection`, permanentFreeze `...when frozen`, bubblegumV2 `Update Authority cannot remove BubblegumV2` | M |
| pm_update_plugin_not_found | A; A+[Attributes] | UpdatePluginV1 FreezeDelegate | PluginNotFound | update_plugin.rs:55-56 | new | S |
| pm_update_plugin_owner_freeze | A+[FreezeDelegate(Owner)] | owner freezes; UA tries to unfreeze; owner unfreezes | Ok; NoApprovals; Ok | update_plugin.rs:58-90,160-243 (size_diff == 0) | port: freeze.test.ts `it can freeze and unfreeze an asset`, `it update authority cannot unfreeze frozen asset` | M |
| pm_update_plugin_grow_shrink_with_trailing | A+[Attributes(UA), FreezeDelegate(Owner)] + X AppData with data | UpdatePluginV1 Attributes larger; then smaller; then equal | Ok x3; trailing plugin bytes, external record offsets and data intact after each | update_plugin.rs:180-243 (both realloc branches, memmove copy_len>0, bump_offsets) | port: plugin_shrink_corruption.rs `test_update_plugin_shrink_attributes_preserves_trailing_plugins`, `..._preserves_external_plugin`; appData.test.ts `updating a plugin before a secure app data does not corrupt the data` | L |
| pm_update_plugin_delegate_and_update_delegate | A+[FreezeDelegate(Address d)]; A+[UpdateDelegate(UA, additional=[d]), Attributes(UA)] | d updates FreezeDelegate; d updates Attributes; d removes another additional delegate; d removes itself | Ok; Ok; NoApprovals; Ok | update_plugin.rs:58-90; update_delegate.rs:208-236 | port: updateDelegate.test.ts `it can update a non-updateDelegate plugin as additional delegate`, `it can remove additional delegate as additional delegate if self`, `it cannot remove another additional delegate...` | M |
| pm_update_plugin_owner_vs_ua_plugins | the four updatePlugin.test.ts matrices (asset-only and with collection) | UpdatePluginV1 | per JS expectations | update_plugin.rs:58-77 | port: updatePlugin.test.ts lines 219-460 | M |
| pm_update_plugin_invalid_royalties | A+[Royalties(UA)] | UpdatePluginV1 Royalties bps 10001 / duplicate creators | InvalidAuthority (royalties rejects) | update_plugin.rs:58-77 error | port: royalties.test.ts `it cannot update royalty ...` | S |
| pm_update_collection_plugin | C+[Attributes(UA), Royalties(UA)] | UpdateCollectionPluginV1 Attributes larger/smaller by UA; by random signer; not-found | Ok, Ok (trailing Royalties intact); InvalidAuthority; PluginNotFound | update_plugin.rs:99-158,160-243 (CollectionV1) | new (guards port from updatePlugin.test.ts collections) | M |
| pm_approve_plugin_authority_asset | A+[FreezeDelegate(Owner), Attributes(UA)] | owner approves Address(d) on FreezeDelegate; UA approves Address(e) on Attributes; owner approves UpdateAuthority on FreezeDelegate (same-size path); owner approves `Authority::None` (immutable) | Ok x4; registry authority updated; realloc only when size changes | approve_plugin_authority.rs:55-90,154-182 | port: approveAuthority.test.ts `it can add an authority to a plugin`; revokeAuthority.test.ts `it can remove the default authority from a plugin to make it immutable` (uses approve with None); delegate.test.ts `it can delegate a new authority` | M |
| pm_approve_plugin_authority_errors | A+[FreezeDelegate(Address d)]; A+[FreezeDelegate(Owner, frozen)]; A; A+[UpdateDelegate(UA, additional=[d])] | owner re-approves (CannotRedelegate); owner approves while frozen (InvalidAuthority); on bare asset (PluginNotFound); d approves UpdateDelegate authority (NoApprovals) | as listed | approve_plugin_authority.rs:55-78 error paths | port: approveAuthority.test.ts `it cannot reassign authority ... while already delegated`, freeze.test.ts `owner cannot approve to reassign authority back to owner if frozen`, updateDelegate.test.ts `it cannot approve the update delegate plugin authority as additional delegate` | S |
| pm_approve_collection_plugin_authority | C+[Attributes(UA)] | UA approves Address(d); random signer; re-approve | Ok; InvalidAuthority; CannotRedelegate | approve_plugin_authority.rs:99-152,154-182 (CollectionV1) | port: collection/updateDelegate.test.ts `it can add updateDelegate to collection and then approve` (+ new errors) | M |
| pm_revoke_plugin_authority_asset | A+[FreezeDelegate(Address d), Attributes(Address e)] | owner revokes FreezeDelegate (payer refund branch); d revokes itself (asset-as-payer branch, assert lamports stay in asset); UA revokes Attributes; e revokes itself | Ok x4; authority back to manager; account shrinks by 32 each | revoke_plugin_authority.rs:58-104,184-210 | port: revokeAuthority.test.ts `it can remove an authority from a plugin`, `...pubkey authority from an owner-managed plugin if that pubkey is the signer`, `...from an authority-managed plugin...` | M |
| pm_revoke_plugin_authority_errors | A+[Attributes(None)]; A+[FreezeDelegate(Address d, frozen)]; A+[UpdateDelegate(UA), FreezeDelegate(Address d)] UA revokes FreezeDelegate; A (no plugin) | InvalidAuthority; InvalidAuthority; InvalidAuthority (updateDelegateRevokeBug); PluginNotFound (registry present but type absent: use A+[Attributes]) | revoke_plugin_authority.rs:65-84 error paths | port: `it cannot remove a none authority from a plugin`, freeze.test.ts `owner cannot undelegate a freeze plugin with a delegate`, updateDelegateRevokeBug.test.ts `it should NOT allow update authority to revoke authority on owner-managed plugins via UpdateDelegate` | S |
| pm_revoke_plugin_authority_bare_asset_panics (regression for finding 1) | A (no meta) | RevokePluginAuthorityV1 FreezeDelegate | currently ProgramFailedToComplete; desired PluginNotFound | revoke_plugin_authority.rs:58-62 | new | S |
| pm_revoke_collection_plugin_authority | C+[Attributes(Address d)] | UA revokes; d revokes itself (collection-as-payer branch); random signer | Ok; Ok; InvalidAuthority | revoke_plugin_authority.rs:113-181,184-210 | new (guards port from revokeAuthority.test.ts collections) | M |
| pm_add_ext_asset_rejections | A | AddExternalPluginAdapterV1 with LinkedAppData / LinkedLifecycleHook / DataSection / LifecycleHook | InvalidPluginAdapterTarget x2; CannotAddDataSection; NotAvailable | add_external_plugin_adapter.rs:53-63,67-68 | port: linkedAppData.test.ts `it cannot add linked app data to an asset`, dataSection.test.ts `it cannot add a DataSection to an asset`, add_external_plugins.rs `test_temporarily_cannot_add_lifecycle_hook` | S |
| pm_add_ext_asset_oracle_appdata | A (UA=payer); then with existing adapter | Oracle (valid checks); AppData(Owner); duplicate AppData; Oracle with empty / approving / duplicate checks; non-UA signer | Ok; Ok; ExternalPluginAdapterAlreadyExists; RequiresLifecycleCheck / OracleCanRejectOnly / DuplicateLifecycleChecks; NoApprovals | add_external_plugin_adapter.rs:65-139 (Oracle/AppData arms 70-71, line 128 error) | port: add_external_plugins.rs `test_add_oracle`, `test_add_app_data`, `test_cannot_add_duplicate_external_plugin_adapter`, `test_cannot_add_oracle_with_duplicate_lifecycle_checks`; oracle.test.ts `it cannot add oracle with no lifecycle checks to asset`, `...that can approve` | M |
| pm_add_ext_collection | C bare, then C+[Attributes] | AddCollectionExternalPluginAdapterV1 Oracle; AppData; LinkedAppData; LifecycleHook; LinkedLifecycleHook; DataSection; random signer | Ok x3 (fresh-meta then existing-meta); NotAvailable x2; CannotAddDataSection; InvalidAuthority | add_external_plugin_adapter.rs:148-244,246-265 (CollectionV1) | port: add_external_plugins.rs `test_temporarily_cannot_add_lifecycle_hook_on_collection`, dataSection.test.ts `it cannot add a DataSection to a collection`; success cases new | M |
| pm_add_ext_collection_bubblegum | C+[BubblegumV2] | AddCollectionExternalPluginAdapterV1 Oracle | InvalidAuthority | add_external_plugin_adapter.rs:221-236 error | port: bubblegumV2.test.ts `it cannot add external plugin to collection with BubblegumV2 plugin` | S |
| pm_remove_ext_asset_errors | A; A+X Oracle(a); A+X AppData(Owner) with owner != UA | RemoveExternalPluginAdapterV1 key AppData(Owner); key Oracle(b); by owner | PluginNotFound; ExternalPluginAdapterNotFound; NoApprovals | remove_external_plugin_adapter.rs:53-58,60-64,67-86 | new | S |
| pm_remove_ext_asset_with_data_and_trailing | A+X [AppData(Owner) with 40 B data, Oracle] + internal Attributes | RemoveExternalPluginAdapterV1 AppData(Owner) | Ok; Oracle record offset bumped by plugin+data size; Attributes intact | remove_external_plugin_adapter.rs:88-94; delete_external_plugin_adapter | port: appData.test.ts `Data offsets are correctly bumped when removing other external plugins with data`; remove_external_plugins.rs `test_remove_app_data` | L |
| pm_remove_ext_collection | C bare; C+X [Oracle, AppData(UA), LinkedAppData(Address d)] | RemoveCollectionExternalPluginAdapterV1 each key by UA; unknown key; random signer | PluginNotFound; Ok x3; ExternalPluginAdapterNotFound; InvalidAuthority | remove_external_plugin_adapter.rs:104-163,165-172 (CollectionV1) | port: remove_external_plugins_on_collection.rs `test_remove_oracle_on_collection`, `test_remove_app_data_on_collection` | M |
| pm_update_ext_asset_errors | A+X Oracle(a) authority Address(x) | UpdateExternalPluginAdapterV1 by UA (not x); key Oracle(b); Oracle key with AppData update info; empty checks; approving checks | InvalidAuthority; ExternalPluginAdapterNotFound; InvalidPlugin; RequiresLifecycleCheck; OracleCanRejectOnly | update_external_plugin_adapter.rs:59-91 error branches (line 90) | port: oracle.test.ts `it cannot update oracle using update authority when different from external plugin authority`, `it cannot update oracle to have no lifecycle checks`, `it cannot update oracle to approve` | S |
| pm_update_ext_asset_shrink_grow_leading | A+X [Oracle(a) with base_address_config, Oracle(b)] + AppData with data | Update Oracle(a) to drop config (shrink), then add larger config and an extra lifecycle check (grow, registry diff != 0) | Ok x2; Oracle(b) bytes, AppData data and all record offsets intact | update_external_plugin_adapter.rs:269-276, 283-290, 208-214 | port: oracle.test.ts `it can shrink a leading oracle without corrupting trailing oracle metadata`, `it can grow a leading oracle...`, `it can update oracle to smaller/larger registry record` | L |
| pm_update_ext_collection | C+X [Oracle(a) auth Address(x), AppData(UA), LinkedAppData(Address d)] | UpdateCollectionExternalPluginAdapterV1: x updates Oracle; UA updates AppData schema; UA updates LinkedAppData schema; UA updates Oracle (not x) | Ok x3; InvalidAuthority | update_external_plugin_adapter.rs:118-188,191-303 (CollectionV1) | port: update_external_plugins_on_collection.rs `test_update_oracle_on_collection`, `test_update_app_data_on_collection`; oracle.test.ts / appData.test.ts / linkedAppData.test.ts `...on collection ...` tests | M |
| pm_write_appdata_asset_matrix (param: data_authority Owner/UpdateAuthority/Address x schema Binary/Json/MsgPack) | A+X AppData(auth) | WriteExternalPluginAdapterDataV1 inline data (first write, grow), second write smaller, third same size; wrong data authority; data authority None | Ok x3 with bytes + data_len assertions; InvalidAuthority x2 | write_external_plugin_adapter_data.rs:36-100,160-223 | port: appData.test.ts write loop, `it cannot write data ... using Y data authority`, `...if the data authority is None`; plugin_shrink_corruption.rs `test_write_external_plugin_adapter_data_single_plugin_shrink` | M |
| pm_write_appdata_asset_trailing_plugin | A+X [AppData(Owner) with data, AppData(Address d)] + Attributes | write AppData(Owner) smaller then larger | Ok; trailing adapter + data + Attributes intact | write:196-223; update_external_plugin_adapter_data both realloc orders | port: plugin_shrink_corruption.rs `test_write_external_plugin_adapter_data_shrink_preserves_second_plugin`; appData.test.ts `Data offsets are correctly bumped when rewriting other external plugins to be smaller/larger` | L |
| pm_write_appdata_buffer_and_source_errors | A+X AppData(Owner); buffer account with 64 B | write with buffer only; with data+buffer; with neither; write with key Oracle(a) / AgentIdentity | Ok (buffer bytes copied); TwoDataSources; NoDataSources; UnsupportedOperation x2 | write:84-85, 210-221 | new | S |
| pm_write_asset_not_found_and_collection_guards | A (no adapter); A@C with wrong / missing collection; A (not in collection) with no collection | write AppData(Owner); write LinkedAppData(Address d) x3 | ExternalPluginAdapterNotFound; InvalidCollection (line 48); MissingCollection (line 48); MissingCollection (line 77) | write:46-48, 65-68, 71-77 | new; linkedAppDataMembership.test.ts | S |
| pm_write_linked_appdata_membership | C+X LinkedAppData(Address d); (i) A not in any collection; (ii) A@C2 (other collection); (iii) A@C | WriteExternalPluginAdapterDataV1 key LinkedAppData(Address d) signed by d, collection=C | (i) InvalidCollection at line 80; (ii) InvalidCollection at line 48; (iii) Ok | write:69-83 | port: linkedAppDataMembership.test.ts (all 3) | M |
| pm_write_linked_appdata_asset_lifecycle (param: data_authority x schema) | C+X LinkedAppData(auth); A@C bare; A@C+[FreezeDelegate] | first write (creates meta + DataSection, inline / buffer); second write smaller; larger; wrong data authority; None; both sources; neither | Ok (DataSection record with parent_key LinkedAppData(auth), data_len); Ok; Ok; InvalidAuthority; InvalidAuthority; TwoDataSources; NoDataSources | write:224-295 (both Ok and NotFound arms, all four source combos) | port: linkedAppData.test.ts write loop + `...multiple times`, `Data offsets are correctly bumped when rewriting Data Section to be smaller/larger`; buffer/Two/No new | L |
| pm_write_collection_appdata | C+X [AppData(UA), Oracle(a)] | WriteCollectionExternalPluginAdapterDataV1 AppData(UA) by UA (grow, shrink, buffer); by owner-only signer; key Oracle(a) | Ok x3; InvalidAuthority; UnsupportedOperation (line 190) | write:112-157, 160-223 (CollectionV1) | new | M |
| pm_write_collection_linked_appdata_key (documents finding 2) | C+X LinkedAppData(Address d) | WriteCollectionExternalPluginAdapterDataV1 key LinkedAppData(Address d) by d | currently Ok and creates a DataSection on C; assert whichever behaviour is decided | write:137-141, 224-295 (CollectionV1) | new | M |
| pm_seq_increment (param over 2, 4, 6, 8, 10, 22, 26) | A with `seq: Some(5)` + relevant plugin | each asset instruction success path | Ok; `seq == 6` | add_plugin.rs:103; remove:86; update:80; approve:81; revoke:87; add_ext:131; update_ext:94 | new (crafted asset; `seq` is normally set only by compression) | S |
| pm_crafted_lifecycle_hook_write (optional) | A+X LifecycleHook(hook, data_authority Some(Owner)) with data; C+X LinkedLifecycleHook(hook, data_authority Some) | WriteExternalPluginAdapterDataV1 key LifecycleHook(hook); key LinkedLifecycleHook(hook) via asset; LifecycleHook with data_authority None | Ok; UnsupportedOperation (line 299); UnsupportedOperation (line 190) | write:176-183, 196, 299 | new; only if covering non-production adapters is desired | M |

### Harness prerequisites

1. Generic account builders in `tests/common` (today's helpers in `account_ownership.rs`/`agent_identity.rs` are file-local and the internal-plugin builder panics on anything but 4 plugin types): `build_asset(owner, update_authority, seq, name, uri, internal: &[(Plugin, Authority)], external: &[(ExternalPluginAdapter, ExternalRegistryRecord fields, Option<data>)])` and `build_collection(..)` producing the exact on-chain layout (core | PluginHeaderV1 | plugins+adapters+appended data | PluginRegistryV1) with correct `offset`/`data_offset`/`data_len`. Use `PluginType::from(&plugin)` instead of a match. Support `UpdateAuthority::Collection(c)` for A@C.
2. A one-byte `[Key::HashedAssetV1]` account helper (NotAvailable guards) and a `bare` (no meta) asset/collection helper.
3. Instruction builders for discriminators 2-11 and 22-29 following the layouts in `src/instruction.rs:45-130, 219-296` (borsh-serialize the `*Args` structs from `mpl_core_program::processor::*`, or hand-serialize Plugin / PluginType / Authority / ExternalPluginAdapterInitInfo / UpdateInfo / Key). Optional accounts use the program-id sentinel as in `agent_identity.rs`. Write ixs need the extra `buffer` slot (index 4 on asset, 3 on collection).
4. Post-state readers: parse `PluginHeaderV1`, `PluginRegistryV1` and each plugin / adapter / data slice back out of the resulting account so tests can assert offsets, authorities, `data_len`, and byte-for-byte integrity of untouched neighbours (the L-size tests depend on this).
5. `assert_failure(result, MplCoreError::X)` shared helper (exists in `execution_delegate.rs`/`agent_identity.rs`; move to common), plus an `assert_panic` (ProgramFailedToComplete) helper for finding 1.
6. Payer account with ample lamports and the System Program in the Mollusk account list (already done in existing tests); Rent sysvar is provided by Mollusk. No lifecycle-hook program stub, no oracle account, and no Bubblegum CPI are needed for anything in this scope.
7. For the write tests: a plain data account to act as `buffer` (any owner, any size).

### Estimate and ordering

Roughly 55 test functions (many parameterised internally): ~14 S, ~30 M, ~7 L, plus the optional crafted-hook test. Expected to take all nine files from their current state to >90% line coverage; the remaining lines are the dead/defensive branches listed above.

Order by coverage gained per unit of effort:
1. Builders + guard/Groups parameterised tests (`pm_guards_all_instructions`, `pm_groups_rejected`) - one helper unlocks ~120 lines across all nine files.
2. Happy paths for internal plugins on assets (`pm_add_first_plugin_owner`, `pm_add_second_plugin_ua_with_init_authority`, `pm_remove_plugin_owner_and_ua`, `pm_update_plugin_owner_freeze`, `pm_approve_plugin_authority_asset`, `pm_revoke_plugin_authority_asset`) - covers the bulk of the five 0% files (~500 lines).
3. Collection variants of the same six (`pm_add_collection_plugin_ua`, `pm_remove_collection_plugin`, `pm_update_collection_plugin`, `pm_approve_collection_plugin_authority`, `pm_revoke_collection_plugin_authority`).
4. Write-data suite (`pm_write_appdata_asset_matrix`, `pm_write_linked_appdata_asset_lifecycle`, `pm_write_linked_appdata_membership`, `pm_write_collection_appdata`, `pm_write_appdata_buffer_and_source_errors`) - 194 lines at 0%, security-relevant (PR #19), and the buffer path has never been tested anywhere.
5. External adapter collection variants and error paths (`pm_add_ext_collection`, `pm_remove_ext_collection`, `pm_update_ext_collection`, `pm_add_ext_asset_oracle_appdata`, `pm_remove_ext_asset_errors`, `pm_update_ext_asset_errors`).
6. Layout-integrity L tests (grow/shrink with trailing plugins, removal with data) - fewer new lines but they guard the memmove/realloc ordering fixes and `bump_offsets`.
7. Authority-matrix ports (updatePlugin owner-vs-UA matrix, updateDelegate / updateDelegateRevokeBug, freeze/freezeExecute frozen rejections), `pm_seq_increment`, the finding-1 panic regression, and finally the optional crafted-hook coverage.


## 10. Groups

Scope: the nine Group instruction processors, the shared `groups_plugin_utils.rs`, `state/group.rs`, and the `Groups` plugin. Every processor is at **0 % line coverage** (12 files, 914 uncovered lines in scope, plus ~120 uncovered lines in `utils/mod.rs`, `utils/account.rs`, `plugins/utils.rs` and `plugins/plugin_registry.rs` that only the group paths reach).

There are **no Mollusk or Rust-client tests** for any group instruction. The JS suite has 6 dedicated files (`createGroup`, `closeGroup`, `group`, `groupComplexRelations`, `groupsPluginBlocking`, `updateGroupAuthority`; 38 tests) plus 6 group-related tests in `burn.test.ts` / `burnCollection.test.ts` and 2 stress tests in `compute.test.ts`. Roughly half of the uncovered branches (argument/account mismatches, non-writable accounts, wrong system program, wrong-account-type, per-target authority failures) have no JS test at all.

Instruction discriminators (first byte of ix data, Borsh args follow): `AddCollectionsToGroupV1=33`, `RemoveCollectionsFromGroupV1=34`, `AddAssetsToGroupV1=35`, `RemoveAssetsFromGroupV1=36`, `AddGroupsToGroupV1=37`, `RemoveGroupsFromGroupV1=38`, `CreateGroupV1=39`, `CloseGroupV1=40`, `UpdateGroupV1=41`. The `*Args` structs are `pub(crate)`, so Mollusk tests must hand-serialize (`u8` discriminator + Borsh fields); `Key::GroupV1 = 6`.

Why Mollusk is a big win here: the expensive JS cases (256-entry vectors, 8-parent nesting, inconsistent bidirectional state, "unreachable" defensive branches) become one-line fixtures because a `GroupV1` account is flat Borsh (`key, update_authority, name, uri, collections, groups, parent_groups, assets`) and can be fabricated with any vector contents. Mollusk also has no 1232-byte transaction limit, so `CreateGroupV1` with 257 relationship entries is testable natively.

### Cross-cutting helpers reached only through group instructions (outside scope, but freed by these tests)

| Helper | Lines (uncovered) | Reached by |
|---|---|---|
| `utils::save_flat_group` | src/utils/mod.rs:105-120 | every group mutation; the `resize` branch (112-114) only when the group's serialized length changes (any add/remove, or `UpdateGroupV1` name/uri length change) |
| `utils::resolve_authority` | src/utils/mod.rs:532-542 | all nine processors (`Some` = separate signer, `None` = payer) |
| `utils::is_valid_group_authority` | src/utils/mod.rs:607-614 | all except `CreateGroupV1` without child/parent rels |
| `utils::is_valid_collection_authority` | src/utils/mod.rs:617-648 | collection links (direct UA at 623, `UpdateDelegate.additional_delegates` at 627-638, `PluginNotFound`/`PluginsNotInitialized` fall-through at 640-644) |
| `utils::is_valid_asset_authority` | src/utils/mod.rs:553-604 | asset links (`Address` 560-564, `Collection` found 566-571, `Collection` missing 573-579 msg path, `UpdateDelegate` 586-600) |
| `utils::account::close_program_account` | src/utils/account.rs:10-35 | `CloseGroupV1` only (no other instruction in the program calls it) |
| `utils::account::resize_or_reallocate_account` shrink branch | src/utils/account.rs:63 | `UpdateGroupV1` with a shorter name/uri, `Remove*FromGroupV1` (group shrinks by 32 bytes), plugin shrink in `save_updated_groups_plugin` |
| `plugins::create_meta_idempotent::<CollectionV1>` / `::<AssetV1>` | src/plugins/utils.rs:26-66 | plugin add/remove on a member; the "create header" branch (35-59) when the member has no plugins yet, the "load" branch (60-65) when it does |
| `plugins::initialize_plugin::<CollectionV1/AssetV1>` | src/plugins/utils.rs:265+ | first time a member joins any group |
| `PluginRegistryV1::bump_offsets` | src/plugins/plugin_registry.rs:82-108 | `save_updated_groups_plugin` when `size_diff != 0` |

### src/processor/create_group.rs — 0/193 lines

Accounts: `[0] group (writable, signer, must be an empty 0-lamport system account)`, `[1] update_authority (optional signer)`, `[2] payer (writable, signer)`, `[3] system_program`, then remaining accounts in the order collections → child groups → parent groups → assets, then optional read-only `CollectionV1` supplemental accounts. Args: `name: String, uri: String, relationships: Vec<RelationshipEntry{kind:u8, key}>`.

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| Guards | 43-57, 61 | any successful call | JS `createGroup.test.ts` "it can create a new group" |
| `InvalidSystemProgram` | 57-59 | account[3] != system program id | none — new |
| group not writable → `ProgramError::InvalidAccountData` | 61-63 | group meta `is_writable=false` (must still sign) | none — new |
| group / payer not signer → `MissingRequiredSignature` | 52-53 | drop signer flag | none — new (cheap, also covers `mpl_utils::assert_signer` failure) |
| `update_authority` supplied → group UA = that key | 54 | pass account[1] as signer | JS "it allows collection authority to link collection-managed assets" (uses `updateAuthority: sharedAuthority`) |
| `DuplicateEntry` | 84-86 | same key appears twice in `relationships` (any kind combination) | none in createGroup tests — new |
| self-reference as Child/Parent → `IncorrectAccount` | 88-96 | `rel.key == group.key` with kind ChildGroup or ParentGroup | JS "it rejects creating a group with itself as a child/parent relationship" (2 tests) |
| kind dispatch | 98-102 | one entry of each kind | JS "it can createGroupV1 with all four relationship kinds in one call" |
| `GroupVectorFull` at creation | 106-111 | ≥257 entries of kind Collection (or ChildGroup, or Asset); checked *before* account creation so keys can be random and no remaining accounts are needed | none — new, **Mollusk-only** (257×33 B of ix data exceeds the on-chain tx size, which is why no JS test exists). Three variants needed for full region coverage of the `||` chain |
| `GroupNestingDepthExceeded` at creation | 113-115 | 9 ParentGroup entries | JS "it rejects createGroupV1 when parent relationships exceed nesting depth" |
| account creation + initial save | 117-151 | any success; `save_flat_group` takes the no-resize branch here (sizes equal) | any happy test |
| `NotEnoughAccountKeys` | 156-167 | fewer remaining accounts than relationship entries (error occurs *after* the CPI created the account; tx reverts) | none — new |
| supplemental account not `CollectionV1` → `IncorrectAccount` | 172-176 | extra remaining account whose byte 0 is not `Key::CollectionV1` (e.g. an `AssetV1`) | none — new |
| supplemental collection accepted | 172-177 (loop, pass) | extra read-only collection | JS "it allows collection authority to link collection-managed assets in createGroupV1" |
| collection key mismatch | 187-190 | remaining account at index ≠ `relationships[i].key` | none — new |
| collection not writable | 192-195 | | none — new |
| collection `InvalidAuthority` | 198-201 | collection UA ≠ resolved authority and no `UpdateDelegate` listing it | none — new |
| collection linked via `UpdateDelegate.additional_delegates` | 198 (true via utils 627-638) | collection fabricated with `UpdateDelegate{additional_delegates:[authority]}` | none — new |
| collection success → plugin add | 204-209 | | JS "all four relationship kinds" |
| child group key mismatch / not writable | 218-226 | | none — new |
| child `GroupV1::load` failure | 228 | pass an `AssetV1`/`CollectionV1` (→ `DeserializationError`) or a GroupV1 owned by another program (→ `InvalidAccountOwner`) | none — new (pattern in `tests/account_ownership.rs`) |
| child `InvalidAuthority` | 230-233 | child group with different `update_authority` | none — new |
| child depth full → `GroupNestingDepthExceeded` | 235-239 | child fabricated with 8 `parent_groups` | JS serial "it rejects createGroupV1 when linking a child group that is already at max nesting depth" (needs 9 txs there; one fixture here) |
| child push + save (resize +32) | 241-247 | | JS "all four relationship kinds" |
| child already lists new group (skip) | 235 false | only by fabricating a child whose `parent_groups` already contains the not-yet-created group key | fabricated-state only |
| parent key mismatch / not writable / load failure / `InvalidAuthority` | 257-272 | mirror of child | none — new |
| parent `groups` full → `GroupVectorFull` | 274-278 | parent fabricated with 256 `groups` | none — new |
| parent push + save | 280-287 | | JS "all four relationship kinds" |
| parent already lists new group (skip) | 274 false | fabricated-state only | — |
| asset key mismatch / not writable | 296-304 | | none — new |
| asset `InvalidAuthority` | 307-310 | asset `UpdateAuthority::Address(other)`; or `UpdateAuthority::Collection(c)` with `c` **not** in the tx (utils 573-579 msg path); or collection present but authority is neither its UA nor delegate | none — new (2-3 variants) |
| asset via collection authority | 307 true via utils 566-571 | collection-managed asset + supplemental collection | JS "it allows collection authority to link collection-managed assets in createGroupV1" |
| asset via `UpdateDelegate` on the asset | 307 true via utils 586-600 | asset fabricated with `UpdateDelegate{additional_delegates:[authority]}` | none — new |
| asset success → plugin add | 313-318 | | JS "all four relationship kinds" |

Notes:
- `create_account` CPI (131-144) requires the `group` account to have 0 lamports and no data; the Mollusk fixture is `(group_pk, Account::default())`.
- `load_key` (line 173) indexes `data[0]` without a length check: a supplemental remaining account with **empty data** (e.g. a plain system wallet) panics → `ProgramFailedToComplete` rather than a clean error. Same in `add_assets_to_group.rs:69` and `remove_assets_from_group.rs:62`. Not exploitable (only aborts the caller's own tx), but worth a defensive `data_len()` check.
- Self-reference with kind `Collection` or `Asset` is *not* caught by 88-96; it fails later at `CollectionV1::load`/`AssetV1::load` with `DeserializationError` because the freshly created account has `Key::GroupV1`. Harmless, but the error differs from the child/parent case.

### src/processor/close_group.rs — 0/22 lines

Accounts: `[0] group (writable)`, `[1] payer (writable, signer, receives lamports)`, `[2] authority (optional signer)`. No system program, no CPI.

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| success → `close_program_account` | 24-45, 48-51 (all four `is_empty()` true), 57 | empty group, authority = UA | JS `closeGroup.test.ts` "it can close a group" (asserts `Key::Uninitialized`, 1-byte account) |
| group not writable | 35-37 | | none — new |
| `GroupV1::load` failure | 40 | non-group account / wrong owner | none — new |
| `InvalidAuthority` | 43-45 | signer ≠ `update_authority` | **none — new** (no JS test) |
| `GroupMustBeEmpty` — each vector | 48-53 | fabricate a group with a single random pubkey in `collections` / `groups` / `parent_groups` / `assets` respectively (4 fixtures; the `&&` chain short-circuits so each needs its own case for region coverage) | JS "it cannot close a group with child assets", "… child collections", "… child groups", "… parent groups" |
| payer not signer | 32 | | none — new (trivial) |

Notes:
- `close_program_account` (utils/account.rs:10-35) is otherwise dead in this program; this instruction is its only caller. It returns `rent(len) - rent(1)` to `payer` and leaves a 1-byte `Uninitialized` account holding one byte of rent, so the pubkey can never be re-used by `CreateGroupV1` (`create_account` rejects funded accounts).
- Lamports go to `payer`, not to the authority. Both must sign when they differ, so this is fine; noting for completeness.

### src/processor/update_group.rs — 0/40 lines

Accounts: `[0] group (writable)`, `[1] payer (writable, signer)`, `[2] authority (optional signer)`, `[3] new_update_authority (optional, **not** a signer)`, `[4] system_program`. Args: `new_name: Option<String>, new_uri: Option<String>`.

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| `InvalidSystemProgram` | 38-40 | | none — new |
| group not writable | 42-44 | | none — new |
| `InvalidAuthority` | 50-52 | old authority after transfer | JS `updateGroupAuthority.test.ts` "it can transfer a group's update authority" (step 3) |
| new update authority | 58-61 | account[3] present | same JS test (step 1); also `_setupRaw.createGroup` when `updateAuthority` is a pubkey |
| new name only | 64-67 | | same JS test (step 2) |
| name + uri, grow | 64-72, 75-81 → `save_flat_group` resize (utils 112-114) grow branch | longer strings | JS "it can updateGroup with both name and URI simultaneously" |
| shrink | 75-81 → `resize_or_reallocate_account` **return-lamports branch (utils/account.rs:63)** | shorter name/uri than existing | none — new (this is the only cheap way to hit account.rs:63) |
| no-op (`dirty == false`) | 75 false → 84 | all args `None`, no account[3] | none — new |

Note: `new_update_authority` is not required to sign and is not validated (can be any pubkey, including the group itself or a non-existent key). Same semantics as collection/asset update authority changes; flagging only because a mistyped key permanently orphans the group (no delegate mechanism exists for groups — `is_valid_group_authority` is exact-match only).

### src/processor/add_assets_to_group.rs — 0/61 lines

Accounts: `[0] group (writable)`, `[1] payer (writable, signer)`, `[2] authority (optional signer)`, `[3] system_program`, remaining = `AssetV1` accounts (writable) interleaved with optional read-only `CollectionV1` accounts (classified by discriminator byte). No args.

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| authority ≠ payer → redundant `assert_signer` | 46-48 | separate authority signer | JS `group.test.ts` "it allows collection update authority to add collection-managed assets to a group" |
| `InvalidSystemProgram` / group not writable | 50-56 | | none — new |
| group `InvalidAuthority` | 60-63 | | JS "it rejects addAssetsToGroup when signer is not group authority" |
| classification: `AssetV1` push, `CollectionV1` skip | 68-71 | asset + supplemental collection | JS collection-managed test above |
| other discriminator → `IncorrectAccount` | 72-75 | pass a `GroupV1` (or any non-asset/collection) as remaining account | none — new |
| only collections, no assets → `IncorrectAccount` | 78-81 | remaining = [collection] | none — new |
| zero remaining accounts → success no-op (group re-saved unchanged) | 78 false, 83 loop skipped, 111 | | none — new (documents the silent no-op) |
| asset not writable | 84-87 | | none — new |
| asset `InvalidAuthority` | 89-91 | asset UA ≠ authority; or `UpdateAuthority::Collection` with collection absent | none — new |
| asset via `UpdateDelegate` additional delegate | 89 true via utils 586-600 | | none — new |
| `DuplicateEntry` | 93-95 | already member, or same asset twice in remaining accounts (second iteration sees the first push) | JS "it rejects adding an already-member asset", "it rejects duplicate asset in remaining accounts for addAssetsToGroup" |
| `GroupVectorFull` | 97-99 | group fabricated with 256 `assets` | JS serial "…group asset vector is already at max size" (256 txs there; one fixture here) |
| success, plugin created on asset | 101-108, 111 | fresh asset (no plugins) → `create_meta_idempotent` create branch + `initialize_plugin::<AssetV1>` | JS `groupsPluginBlocking.test.ts` "it blocks generic asset plugin operations for Groups" (asserts `groups` plugin content), `closeGroup` "child assets", `burn.test.ts:313` |
| success, asset already has other plugins | same, `create_meta_idempotent` load branch | asset fabricated with e.g. `FreezeDelegate` (helper `build_asset_with_plugins` in `tests/account_ownership.rs:130`) | none — new |
| success, asset already in another group | → `groups_plugin_utils.rs:113-134` grow path | asset with `Groups{[g1]}` added to g2 | none — new |

### src/processor/remove_assets_from_group.rs — 0/91 lines

Args: `assets: Vec<Pubkey>`; remaining = matching `AssetV1` accounts in order, then optional read-only `CollectionV1` supplemental accounts (`len >=` args, not `==`).

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| `InvalidSystemProgram` / not writable | 45-51 | | none — new |
| `NotEnoughAccountKeys` | 53-56 | fewer remaining accounts than `args.assets` | none — new |
| supplemental non-collection → `IncorrectAccount` | 61-65 | extra remaining account that is an asset/group | none — new |
| supplemental collection accepted | 61-66 pass | | JS `group.test.ts` "it allows collection update authority to remove collection-managed assets from a group" |
| group `InvalidAuthority` | 69-71 | | JS "it rejects removeAssetsFromGroup when signer is not group authority" |
| key mismatch → `IncorrectAccount` | 76-78 | `args.assets[i] != remaining[i].key` | none — new |
| asset not writable | 79-81 | | none — new |
| asset `InvalidAuthority` | 83-85 | | none — new |
| not a member → `IncorrectAccount` | 90-93 | asset not in `group.assets` | none — new |
| success (group shrinks 32 B → `save_flat_group` resize shrink) | 88-89, 95-103 | | JS `burn.test.ts:343` "it allows burning an asset after removing it from all groups", collection-managed remove test |
| `process_asset_groups_plugin_remove` normal | 107-126, 134-143 → `save_updated_groups_plugin` shrink | asset with `Groups{[g]}`; plugin becomes `Groups{[]}` (plugin is **not** deleted) | same |
| plugin lacks this group → early `Ok` | 127-128 | fabricate: group lists asset, asset plugin `Groups{[other]}` | fabricated-state only |
| registry record type `Groups` but bytes deserialize to another variant → `InvalidPlugin` | 130-131 | fabricated corrupt registry | fabricated-state only |
| asset has no `Groups` plugin at all | 121 `None` → 145 | fabricate group listing an asset without the plugin; note `create_meta_idempotent` (113-114) will *create* an empty plugin header/registry on the asset (payer-funded realloc) before discovering nothing to remove | fabricated-state only |

### src/processor/add_collections_to_group.rs — 0/53 lines

Remaining = writable `CollectionV1` accounts. No args.

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| authority ≠ payer | 43-45 | | JS `group.test.ts` "it rejects addCollectionsToGroup when signer is not group authority" (attacker signer, then fails at 59-62) |
| `InvalidSystemProgram` / not writable | 47-53 | | none — new |
| group `InvalidAuthority` | 59-62 | | JS test above |
| collection not writable | 67-70 | | none — new |
| `CollectionV1::load` failure | 73 | pass an asset or group → `DeserializationError`; wrong owner → `InvalidAccountOwner` | none — new |
| collection `InvalidAuthority` | 76-79 | group UA == signer but collection UA differs, no delegate | none — new |
| via `UpdateDelegate` on collection | 76 true via utils 627-638 | | none — new |
| `DuplicateEntry` | 81-83 | | JS "it rejects adding an already-member collection", "…duplicate collection in remaining accounts" |
| `GroupVectorFull` | 85-87 | group fabricated with 256 `collections` | JS serial "…collection vector is already at max size" |
| success (plugin created / appended) | 89-102 | | JS `groupsPluginBlocking` "it blocks generic collection plugin operations for Groups", `closeGroup` "child collections", `burnCollection.test.ts:137` |
| collection already has plugins (e.g. Royalties) | `create_meta_idempotent` load branch | | none — new |
| collection in two groups | → `groups_plugin_utils.rs:53-75` | | none — new |

### src/processor/remove_collections_from_group.rs — 0/98 lines

Args: `collections: Vec<Pubkey>`; remaining count must `==` args length (no supplemental accounts, unlike assets).

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| `InvalidSystemProgram` / not writable | 51-57 | | none — new |
| `NotEnoughAccountKeys` | 59-66 | count mismatch either direction | none — new |
| group `InvalidAuthority` | 72-75 | | JS `group.test.ts` "it rejects removeCollectionsFromGroup when signer is not group authority" |
| key mismatch | 78-84 | | none — new |
| not writable | 86-89 | | none — new |
| load failure | 91 | | none — new |
| collection `InvalidAuthority` | 93-96 | | none — new |
| not a member → `IncorrectAccount` | 105-108 | | none — new |
| success | 99-104, 111-121 | | JS `burnCollection.test.ts:170` "it allows burning a collection after removing it from all groups" |
| `process_collection_groups_plugin_remove` normal | 124-143, 151-160 | | same |
| plugin lacks group / `InvalidPlugin` / no plugin | 144-145 / 147-148 / 138 `None` | fabricated-state only | — |

### src/processor/add_groups_to_group.rs — 0/75 lines

Accounts: `[0] parent_group (writable)`, `[1] payer`, `[2] authority (opt signer)`, `[3] system_program`; remaining = child `GroupV1` accounts matching `args.groups` (count must `==`).

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| authority ≠ payer | 45-47 | | JS `groupComplexRelations.test.ts` "it rejects addGroupsToGroup when signer is not parent group authority" |
| `InvalidSystemProgram` / not writable | 49-55 | | none — new |
| `NotEnoughAccountKeys` | 58-65 | | none — new |
| parent `InvalidAuthority` | 71-74 | | JS test above |
| key mismatch | 79-85 | | none — new |
| child not writable | 88-91 | | none — new |
| self as child → `IncorrectAccount` | 93-96 | | JS "it rejects adding a parent group as its own child group" |
| child load failure | 99 | | none — new |
| child `InvalidAuthority` | 102-105 | child UA ≠ parent UA | none — new |
| `DuplicateEntry` | 107-109 | | JS "it rejects duplicate child group in addGroupsToGroup" |
| parent `groups` full → `GroupVectorFull` | 111-113 | parent fabricated with 256 `groups` | JS serial "…child group vector exceeds max size" |
| child `parent_groups` full → `GroupNestingDepthExceeded` | 115-118 | child fabricated with 8 `parent_groups` | **none — new** (the JS depth test only exercises the `CreateGroupV1` variant) |
| success, both sides saved | 120-135 | | JS "it keeps parentGroups in sync with groups when adding and removing child groups", `closeGroup` "child groups"/"parent groups" |
| child already lists parent (skip child save) | 122 false | fabricated-state only (parent doesn't list child but child lists parent) | — |

### src/processor/remove_groups_from_group.rs — 0/74 lines

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| authority ≠ payer | 46-48 | | JS `groupComplexRelations.test.ts` "it rejects removeGroupsFromGroup when signer is not parent group authority" |
| `InvalidSystemProgram` / not writable / `NotEnoughAccountKeys` | 50-66 | | none — new |
| parent `InvalidAuthority` | 72-75 | | JS test above |
| key mismatch / not writable / load failure | 79-92 | | none — new |
| child `InvalidAuthority` | 95-98 | | none — new |
| child not linked → `IncorrectAccount` | 107-110 | | JS "it rejects removing a child group that is not linked" |
| success | 101-106, 112-118, 125-132 | | JS "it keeps parentGroups in sync…" |
| `InconsistentGroupRelationship` | 119-122 | parent lists child but child's `parent_groups` lacks parent | **unreachable via program-produced state** (every writer keeps both sides in sync; see bug notes); fabricated-state only |

### src/processor/groups_plugin_utils.rs — 0/166 lines

| Path | Lines | Trigger | Existing test |
|---|---|---|---|
| `process_collection_groups_plugin_add`, no plugin yet | 18-51 | first group for a collection | any collection-add success (JS `groupsPluginBlocking` collection test) |
| …plugin exists, append (grow) | 53-75 | second group for the same collection | none — new |
| …already contains group → early `Ok` | 57-58 | fabricated: group doesn't list collection but collection plugin lists group | fabricated-state only |
| …record type Groups but bytes another variant → `InvalidPlugin` | 61-62 | fabricated | fabricated-state only |
| `process_asset_groups_plugin_add` (mirror) | 82-137 | as above for assets | JS `groupsPluginBlocking` asset test (first), new (second group, early-return, InvalidPlugin) |
| `save_updated_groups_plugin`, grow, Groups is last plugin (`copy_len == 0`) | 143-189, 203, 219-223 | second group added to a member whose Groups plugin is the last plugin | none — new |
| grow with trailing plugin (`copy_len > 0`, memmove after realloc) | 206-216 | member fabricated with `[Groups{[g1]}, FreezeDelegate]` in that order, add g2; assert FreezeDelegate still deserializes and its registry offset moved +32 (`bump_offsets`) | none — new |
| shrink, Groups last | 161-189, 203 | remove the only group | any remove success |
| shrink with trailing plugin (memmove before realloc) | 191-201 | member fabricated with `[Groups{[g1]}, FreezeDelegate]`, remove g1 | none — new |
| `size_diff == 0` | 161 false | **unreachable**: add and remove always change the vector by exactly one pubkey (±32 B); the early returns above fire before this function on no-change | dead branch |
| `NumericalOverflow` arms | 158-159, 166-171, 175-184, 187-188 | not reachable with real account sizes (< 10 MB) | dead |

### src/state/group.rs — 35/64 lines

| Lines | Item | Coverage route |
|---|---|---|
| 51-70 | `GroupV1::new` | any `CreateGroupV1` success (the existing unit test builds the struct literally) |
| 86-88 | `SolanaAccount::key` | any `GroupV1::load` |
| 92-98 | `CoreAsset for GroupV1` (`update_authority()`, `owner()`) | **dead in program code**: no generic `CoreAsset` call site is ever instantiated with `GroupV1` (`assert_authority`, `approve_authority_on_plugin`, `process_approve_plugin_authority` only see Asset/Collection). Cover with a 3-line unit test in the existing `mod tests`, or delete the impl |
| 74-82 `len` | already covered by `test_group_len` | — |

### src/plugins/internal/authority_managed/groups.rs — 3/15 lines

`validate_burn` (30-54) is entirely uncovered. It runs from `BurnV1` (asset checks, `self_key = AssetV1`; and collection-inherited checks, `self_key = CollectionV1` with `asset_info = Some`) and from `BurnCollectionV1` (`self_key = CollectionV1`, `asset_info = None`, via `burn.rs:151-165` `validate_collection_permissions`). `PluginType::check_burn` returns `CanReject` for Groups (`lifecycle.rs:170`). These tests live in the burn processors' scope but are the only way to cover this file:

| Branch | Lines | Setup | Existing test |
|---|---|---|---|
| asset member, non-empty → reject (`InvalidAuthority`) | 40-42, 49-50 | asset with `Groups{[g]}`, `BurnV1` | JS `burn.test.ts:313` "it rejects burning an asset that belongs to a group" |
| asset member, empty vec → abstain | 42, 49 false, 52 | asset with `Groups{[]}` (state after `RemoveAssetsFromGroupV1`) | JS `burn.test.ts:343` "it allows burning an asset after removing it from all groups" |
| collection burn, non-empty → reject | 45 (`asset_info.is_none()` true), 49-50 | collection with `Groups{[g]}`, `BurnCollectionV1` | JS `burnCollection.test.ts:137` |
| collection burn after removal → abstain | 45, 49 false | | JS `burnCollection.test.ts:170` |
| asset in member collection → abstain | 45 (`is_none()` false), 52 | asset in collection that has `Groups{[g]}`, `BurnV1` with collection | JS `burn.test.ts:372`, `:395` (secondary owner) |
| `_ => false` | 46 | unreachable: `self_key` is always AssetV1/CollectionV1 (`lifecycle.rs:702-706` `unreachable!()`) | dead |

Note: `reject!()` surfaces to the caller as `MplCoreError::InvalidAuthority` (the JS tests assert that name), not a group-specific error.

### Bugs / suspicious logic / security notes

1. **Panic on empty-data remaining accounts** — `load_key` (`utils/mod.rs:29-33`) reads `data[0]` unguarded; `create_group.rs:173`, `add_assets_to_group.rs:69`, `remove_assets_from_group.rs:62` call it on caller-supplied remaining accounts. A 0-byte account (system wallet) causes an index-out-of-bounds panic → `ProgramFailedToComplete`. Only affects the caller's own transaction; a `data_len() == 0` guard would give a clean `IncorrectAccount`.
2. **No cycle prevention, and `MAX_GROUP_NESTING_DEPTH` is not a depth limit** — `AddGroupsToGroupV1` only rejects `child == parent`. A→B then B→A is accepted (both `groups`/`parent_groups` updated), and a linear chain of any length is accepted since the constant caps the *number of parents per group* (8), not chain depth. Off-chain traversal must handle cycles. `CloseGroupV1` still works after unlinking, so no lock-in.
3. **Dead defensive code** (only reachable by fabricating inconsistent state, since every writer updates both sides atomically): `remove_groups_from_group.rs:119-122` (`InconsistentGroupRelationship`), `groups_plugin_utils.rs:57-58,116-117` (already-contains early return), `remove_assets_from_group.rs:127-128` / `remove_collections_from_group.rs:144-145` (plugin lacks group), the `InvalidPlugin` arms (`groups_plugin_utils.rs:61-62,120-121`, `remove_*:130-131/147-148`), `add_groups_to_group.rs:122` false, `create_group.rs:235/274` false, `save_updated_groups_plugin` `size_diff == 0`. Mollusk can fabricate these states, so they are coverable, but the tests assert behaviour on state the program can't produce.
4. **`remove_*_plugin_remove` on a member without plugin metadata creates it** — `create_meta_idempotent` (`remove_assets_from_group.rs:113`, `remove_collections_from_group.rs:130`) will allocate an empty header/registry (payer-funded) before finding no Groups record. Only with inconsistent state; harmless.
5. **Silent no-op**: `AddAssetsToGroupV1` / `AddCollectionsToGroupV1` with zero remaining accounts succeed and rewrite the group unchanged (`add_assets_to_group.rs:78` only errors when remaining accounts are *non-empty* but contain no assets).
6. **Inconsistent count checks**: `RemoveAssetsFromGroupV1` allows extra remaining accounts (`<`, supplemental collections), `RemoveCollectionsFromGroupV1` / `Add|RemoveGroupsFromGroupV1` require exact equality. Intended (assets may be collection-managed) but note `AddAssetsToGroupV1` has no args at all and classifies purely by discriminator byte.
7. **`CreateGroupV1` creates the account before validating relationships** (`create_group.rs:131-151` precedes 156-319). All later failures revert the tx, so no state leak; only relevant for compute accounting.
8. **`is_valid_group_authority` is exact-match** (`utils/mod.rs:607-614`): groups have no delegate mechanism, while collection/asset links accept `UpdateDelegate.additional_delegates`. Consistent with the design, but means a group whose UA is set to a bad key via `UpdateGroupV1` (non-signing `new_update_authority`) is permanently frozen.
9. `Groups` plugin cannot be added/updated/removed/approved/revoked through the generic plugin instructions (`add_plugin.rs:52,145`, `update_plugin.rs:51,121`, `remove_plugin.rs:48,124`, `approve_plugin_authority.rs:51,120`, `revoke_plugin_authority.rs:54,134`, `create.rs:196`, `create_collection.rs:152`). JS `groupsPluginBlocking.test.ts` covers all ten blocks; those lines belong to the plugin-processor scope but the group fixtures below make them one-liners.

### Test plan

Naming: `tests/groups.rs` (or split `groups_create.rs`, `groups_membership.rs`, `groups_plugin.rs`). "Fab." = fabricated account bytes (no prior instruction needed).

| # | Test case | Sets up | Instruction(s) | Expected | Source paths covered | New / port | Size |
|---|---|---|---|---|---|---|---|
| 1 | `create_group_minimal` | payer, empty group signer | CreateGroupV1 (no rels) | Ok; GroupV1 bytes = expected, UA = payer | create_group.rs:43-151,156-160,321; state/group.rs:51-70,86-88; utils/mod.rs:105-120,532-542 | port JS createGroup "it can create a new group" | S |
| 2 | `create_group_with_update_authority_signer` | as 1 + account[1] signer | CreateGroupV1 | UA = account[1] | create_group.rs:54 (Some) ; utils 535-538 | port JS "collection authority… createGroupV1" (UA part) | S |
| 3 | `create_group_rejects_bad_system_program_and_nonwritable` | 2 variants | CreateGroupV1 | `InvalidSystemProgram`; `InvalidAccountData` | create_group.rs:57-63 | new | S |
| 4 | `create_group_rejects_missing_signers` | group or payer non-signer | CreateGroupV1 | `MissingRequiredSignature` | 52-53 | new | S |
| 5 | `create_group_rejects_duplicate_and_self_reference` | rels [X,X]; [ChildGroup self]; [ParentGroup self] | CreateGroupV1 | `DuplicateEntry`; `IncorrectAccount` ×2 | 84-96 | port JS ×2 + new | S |
| 6 | `create_group_rejects_vector_full_at_creation` | 257 Collection / ChildGroup / Asset entries (random keys) | CreateGroupV1 | `GroupVectorFull` ×3 | 106-111 | new (Mollusk-only) | S |
| 7 | `create_group_rejects_nesting_depth_at_creation` | 9 ParentGroup entries | CreateGroupV1 | `GroupNestingDepthExceeded` | 113-115 | port JS | S |
| 8 | `create_group_all_relationship_kinds` | fab. collection (no plugins), child group, parent group, asset (no plugins), all UA = payer | CreateGroupV1 with 4 rels + 4 writable remaining | Ok; group vectors; child.parent_groups=[g]; parent.groups=[g]; collection & asset have `Groups{[g]}` plugin with `Authority::UpdateAuthority` | 98-102,183-319; groups_plugin_utils.rs:18-51,82-111; plugins/utils.rs:26-59,265+ | port JS "all four relationship kinds" | M |
| 9 | `create_group_rejects_not_enough_remaining` | 1 rel, 0 remaining | CreateGroupV1 | `NotEnoughAccountKeys` | 156-167 | new | S |
| 10 | `create_group_supplemental_accounts` | asset with `UpdateAuthority::Collection(c)`, UA(c)=payer; supplemental c read-only; variant: supplemental is an asset | CreateGroupV1 | Ok / `IncorrectAccount` | 172-177, 293-318; utils 566-571 | port JS "collection authority… createGroupV1" + new | S |
| 11 | `create_group_collection_link_failures` | mismatch key; non-writable; foreign UA (no delegate); UA via `UpdateDelegate` delegate | CreateGroupV1 ×4 | `IncorrectAccount`; `InvalidAccountData`; `InvalidAuthority`; Ok | 187-201; utils 617-648 | new | M |
| 12 | `create_group_child_link_failures` | mismatch; non-writable; asset-as-child; foreign-owned group; foreign UA; child with 8 parents (fab.) | CreateGroupV1 ×6 | `IncorrectAccount`; `InvalidAccountData`; `DeserializationError`; `InvalidAccountOwner`; `InvalidAuthority`; `GroupNestingDepthExceeded` | 218-239 | new (+ port JS serial depth test) | M |
| 13 | `create_group_parent_link_failures` | as 12 but parent with 256 `groups` (fab.) | CreateGroupV1 ×6 | …; `GroupVectorFull` | 257-278 | new | M |
| 14 | `create_group_asset_link_failures` | mismatch; non-writable; foreign UA; Collection-UA without collection in tx; asset with `UpdateDelegate` delegate | CreateGroupV1 ×5 | `IncorrectAccount`; `InvalidAccountData`; `InvalidAuthority` ×2; Ok | 296-318; utils 553-604 | new | M |
| 15 | `close_group_success` | empty fab. group, payer lamports recorded | CloseGroupV1 | Ok; data=[0]; payer +rent(len)-rent(1) | close_group.rs all-success; utils/account.rs:10-35 | port JS "it can close a group" | S |
| 16 | `close_group_rejects_non_empty` | 4 fab. groups, one vector each non-empty | CloseGroupV1 ×4 | `GroupMustBeEmpty` | close_group.rs:48-53 (all 4 regions) | port JS ×4 | S |
| 17 | `close_group_rejects_auth_and_account_errors` | wrong authority; non-writable; asset passed as group; payer non-signer | CloseGroupV1 ×4 | `InvalidAuthority`; `InvalidAccountData`; `DeserializationError`; `MissingRequiredSignature` | 32-45 | new | S |
| 18 | `update_group_transfer_authority_then_rename` | fab. group | UpdateGroupV1 (new UA) → UpdateGroupV1 by new UA (name) → UpdateGroupV1 by old UA | Ok; Ok; `InvalidAuthority` | update_group.rs:47-67,75-81 | port JS "it can transfer a group's update authority" | S |
| 19 | `update_group_name_uri_grow_shrink_noop` | fab. group name "abc" | UpdateGroupV1 (longer both) → (shorter both) → (all None) | Ok ×3; account len tracks `GroupV1::len`; lamports returned on shrink | 64-84; utils/mod.rs:112-114; **utils/account.rs:63** | port JS "both name and URI" + new | S |
| 20 | `update_group_rejects_bad_system_program_and_nonwritable` | | UpdateGroupV1 ×2 | `InvalidSystemProgram`; `InvalidAccountData` | 38-44 | new | S |
| 21 | `add_assets_success_fresh_and_with_plugins` | fab. group; asset A (no plugins), asset B (fab. with FreezeDelegate via `build_asset_with_plugins`) | AddAssetsToGroupV1 [A,B] | Ok; group.assets=[A,B]; both have `Groups{[g]}`; B's FreezeDelegate intact | add_assets_to_group.rs:32-111; groups_plugin_utils.rs:82-111; plugins/utils.rs:26-66 | port JS groupsPluginBlocking (asset add) + new | M |
| 22 | `add_assets_collection_managed_with_separate_authority` | shared authority ≠ payer; asset UA=Collection(c); c UA=shared; remaining [asset, c(ro)] | AddAssetsToGroupV1 | Ok | 46-48,68-71,89; utils 566-571 | port JS "collection update authority to add collection-managed assets" | S |
| 23 | `add_assets_rejections` | wrong sys prog; non-writable group; attacker authority; remaining contains GroupV1; remaining only [collection]; non-writable asset; asset foreign UA; Collection-UA without c; asset via UpdateDelegate (Ok) | AddAssetsToGroupV1 ×9 | `InvalidSystemProgram`; `InvalidAccountData`; `InvalidAuthority`; `IncorrectAccount`; `IncorrectAccount`; `InvalidAccountData`; `InvalidAuthority`; `InvalidAuthority`; Ok | 50-63,72-91; utils 573-600 | port JS "not group authority" + new | M |
| 24 | `add_assets_duplicate_and_full` | member asset again; same asset twice; group fab. with 256 assets | AddAssetsToGroupV1 ×3 | `DuplicateEntry` ×2; `GroupVectorFull` | 93-99 | port JS ×3 (serial 256 → fixture) | S |
| 25 | `add_assets_zero_remaining_is_noop` | | AddAssetsToGroupV1 [] | Ok, group unchanged | 78 false, 111 | new | S |
| 26 | `remove_assets_success` | group.assets=[A]; A `Groups{[g]}` (last plugin) | RemoveAssetsFromGroupV1 [A] | Ok; group.assets=[]; A plugin `Groups{[]}`; sizes shrank | remove_assets_from_group.rs:30-105,107-126,134-143; groups_plugin_utils.rs:143-203,219-223 (shrink, copy_len 0); utils/account.rs:63 | port JS burn.test:343 (remove part) | S |
| 27 | `remove_assets_collection_managed_with_supplemental` | as 22, remaining [A, c(ro)] | RemoveAssetsFromGroupV1 [A] | Ok | 61-66 pass; utils 566-571 | port JS "…remove collection-managed assets" | S |
| 28 | `remove_assets_rejections` | wrong sys prog; non-writable group; fewer remaining than args; supplemental asset; attacker; key mismatch; non-writable asset; asset foreign UA; asset not member | ×9 | `InvalidSystemProgram`; `InvalidAccountData`; `NotEnoughAccountKeys`; `IncorrectAccount`; `InvalidAuthority`; `IncorrectAccount`; `InvalidAccountData`; `InvalidAuthority`; `IncorrectAccount` | 45-93 | port JS "not group authority" + new | M |
| 29 | `remove_assets_inconsistent_plugin_states` | (a) group lists A, A plugin `Groups{[other]}`; (b) A has no plugins at all; (c) registry record Groups pointing at FreezeDelegate bytes | ×3 | Ok (group updated, plugin untouched); Ok (empty meta created); `InvalidPlugin` | 121 None, 127-131 | new (fabricated state) | S |
| 30 | `add_collections_success_fresh_and_with_plugins` | collection C1 no plugins; C2 with Royalties | AddCollectionsToGroupV1 [C1,C2] | Ok; both `Groups{[g]}` | add_collections_to_group.rs:24-105; groups_plugin_utils.rs:18-51; plugins/utils.rs:26-66 | port JS groupsPluginBlocking (collection add) + new | S |
| 31 | `add_collections_rejections` | wrong sys prog; non-writable group; attacker (≠ payer); non-writable collection; asset as collection; collection foreign UA; via UpdateDelegate (Ok) | ×7 | `InvalidSystemProgram`; `InvalidAccountData`; `InvalidAuthority`; `InvalidAccountData`; `DeserializationError`; `InvalidAuthority`; Ok | 43-79; utils 617-648 | port JS "not group authority" + new | M |
| 32 | `add_collections_duplicate_and_full` | | ×3 | `DuplicateEntry` ×2; `GroupVectorFull` | 81-87 | port JS ×3 | S |
| 33 | `remove_collections_success` | | RemoveCollectionsFromGroupV1 | Ok; plugin `Groups{[]}` | remove_collections_from_group.rs:29-122,124-143,151-160 | port JS burnCollection:170 (remove part) | S |
| 34 | `remove_collections_rejections` | wrong sys prog; non-writable; count mismatch (both directions); attacker; key mismatch; non-writable coll; asset as coll; foreign UA; not member | ×10 | as listed in file table | 51-108 | port JS "not group authority" + new | M |
| 35 | `remove_collections_inconsistent_plugin_states` | as 29 for collections | ×3 | Ok; Ok; `InvalidPlugin` | 138 None, 144-148 | new (fabricated) | S |
| 36 | `member_in_two_groups_grow_and_shrink_with_trailing_plugin` | asset fab. `[Groups{[g1]}, FreezeDelegate]`; collection likewise | AddAssetsToGroupV1 (g2) → RemoveAssetsFromGroupV1 (g1) ; same for collections | Ok; plugin `[g1,g2]` then `[g2]`; FreezeDelegate intact and registry offsets bumped ±32 | groups_plugin_utils.rs:53-75,113-134,161-216 (both memmove branches); plugin_registry.rs:82-108 | new | M |
| 37 | `groups_plugin_add_already_contains_and_invalid_plugin` | group doesn't list A but A plugin `Groups{[g]}`; record Groups over FreezeDelegate bytes | AddAssetsToGroupV1 / AddCollectionsToGroupV1 | Ok (no plugin change); `InvalidPlugin` | groups_plugin_utils.rs:57-62,116-121 | new (fabricated) | S |
| 38 | `add_groups_success_bidirectional` | parent P, children C1 (no parents), C2 (fab. with 7 parents) | AddGroupsToGroupV1 [C1,C2] | Ok; P.groups=[C1,C2]; C1/C2.parent_groups gained P | add_groups_to_group.rs:25-136 | port JS "keeps parentGroups in sync" (add half) | S |
| 39 | `add_groups_rejections` | wrong sys prog; non-writable parent; count mismatch; attacker; key mismatch; non-writable child; self; asset as child; child foreign UA; duplicate; parent 256 groups (fab.); child 8 parents (fab.); child already lists parent (fab.) | ×13 | `InvalidSystemProgram`; `InvalidAccountData`; `NotEnoughAccountKeys`; `InvalidAuthority`; `IncorrectAccount`; `InvalidAccountData`; `IncorrectAccount`; `DeserializationError`; `InvalidAuthority`; `DuplicateEntry`; `GroupVectorFull`; `GroupNestingDepthExceeded`; Ok (child unsaved) | 49-125 | port JS ×4 + new | M |
| 40 | `remove_groups_success` | P.groups=[C], C.parent_groups=[P] | RemoveGroupsFromGroupV1 [C] | Ok; both empty; both accounts shrank | remove_groups_from_group.rs:25-133 | port JS "keeps parentGroups in sync" (remove half) | S |
| 41 | `remove_groups_rejections` | wrong sys prog; non-writable; count mismatch; attacker; key mismatch; non-writable child; asset as child; child foreign UA; not linked; inconsistent (P lists C, C lacks P) | ×10 | …; `IncorrectAccount`; `InconsistentGroupRelationship` | 50-122 | port JS ×2 + new | M |
| 42 | `groups_allow_cycle` (documentation test) | A child of B, then B child of A | AddGroupsToGroupV1 ×2 | Ok both — documents note 2 | add_groups_to_group.rs success | new | S |
| 43 | `group_state_core_asset_impl` | unit test in `state/group.rs` | `update_authority()`, `owner()` | values | state/group.rs:92-98 | new | S |
| 44 | `groups_plugin_validate_burn` (burn scope) | asset `Groups{[g]}`; asset `Groups{[]}`; collection `Groups{[g]}`; asset inside such a collection | BurnV1 / BurnCollectionV1 | `InvalidAuthority`; Ok; `InvalidAuthority`; Ok | plugins/…/groups.rs:30-54; lifecycle.rs:170 | port JS burn.test:313/343/372, burnCollection:137 | M |
| 45 | `groups_plugin_blocked_in_generic_plugin_ixs` (plugin-processor scope) | asset/collection with `Groups{[g]}` | CreateV1/CreateCollectionV1 with Groups plugin; Add/Update/Remove/Approve/RevokePlugin(Groups) on asset & collection | `InvalidPlugin` ×10 | add_plugin.rs:52,145; update_plugin.rs:51,121; remove_plugin.rs:48,124; approve/revoke:…; create.rs:196; create_collection.rs:152 | port JS groupsPluginBlocking ×4 | M |

### Harness prerequisites

All Mollusk-side; none need an external program stub (group instructions only CPI the system program, which `core_mollusk()` already provides and the existing `agent_identity.rs` tests already exercise for realloc).

1. **Group fixture builder** `group_account(update_authority, name, uri, collections, groups, parent_groups, assets) -> Account` — Borsh of `GroupV1` (it is `pub` in `mpl_core_program::state`), owner `MPL_CORE_ID`, rent-exempt lamports. Variants: `with_n_random_keys(vec, n)` for the 256/8 limits.
2. **Empty system account fixture** for `CreateGroupV1`: `(Pubkey::new_unique(), Account::default())` — must be 0 lamports for `create_account` CPI.
3. **Instruction builders** for all nine discriminators with Borsh args: `CreateGroupV1 {name, uri, Vec<(u8 kind, Pubkey)>}`, `RemoveAssetsFromGroupV1 {Vec<Pubkey>}`, `RemoveCollectionsFromGroupV1 {Vec<Pubkey>}`, `Add/RemoveGroupsFromGroupV1 {Vec<Pubkey>}`, `UpdateGroupV1 {Option<String>, Option<String>}`, `CloseGroupV1 {}`, `AddAssetsToGroupV1 {}`, `AddCollectionsToGroupV1 {}`. Builder takes `authority: Option<Pubkey>` and `remaining: Vec<AccountMeta>`; helpers for the "wrong system program" and "non-writable" variants.
4. **Asset/collection-with-plugins builders**: generalize `build_asset_with_plugins` from `tests/account_ownership.rs:130` to (a) accept `PluginType::from(&plugin)` instead of the hard-coded match (so `Groups`, `UpdateDelegate`, `Royalties` work), (b) accept an explicit `UpdateAuthority` (needed for `UpdateAuthority::Collection(c)`), and (c) a `CollectionV1` twin. Put these in `tests/common/mod.rs`.
5. **Assertion helpers**: `read_group(&result, pk) -> GroupV1`, `read_groups_plugin(&result, pk) -> (Groups, RegistryRecord)` (walk header/registry like the program does), `assert_error(&result, MplCoreError::X)` / `ProgramError::Y` (see `execution_delegate.rs:67 assert_failure`), and a rent/lamport delta check for `CloseGroupV1` and the shrink paths.
6. **Sequenced execution**: several cases chain 2-3 instructions (18, 19, 36, 38→40). Use `mollusk.process_and_validate_instruction_chain` or feed `result.resulting_accounts` back in.
7. For test 44/45 (burn and generic-plugin blocking) the fixtures above are sufficient; those tests belong with the burn / plugin-processor sections but should reuse the group fixtures.

### Estimated effort and ordering

About **45 test functions (~120 instruction executions)**. Ordered by coverage gained per unit of effort:

1. Fixtures + tests 1, 8, 15, 18, 21, 26, 30, 33, 38, 40 (the ten happy paths) — S/M each, together they take every processor from 0 % to roughly 55-65 % and light up `save_flat_group`, `resolve_authority`, `is_valid_*_authority` direct paths, `close_program_account`, `create_meta_idempotent`, `initialize_plugin` for both account types.
2. Tests 16, 17, 19, 24, 32, 39(limits), 6, 7, 12-13(limit rows) — the fabricated-state limit tests (256 / 8 / non-empty vectors); trivial in Mollusk, impossible-to-cheap in JS. Also 19 alone covers `utils/account.rs:63`.
3. Tests 23, 28, 31, 34, 41, 3, 9, 20 — the per-instruction rejection matrices (system program, writability, count mismatch, key mismatch, per-target authority). Mostly new; each is a parametrized loop over one fixture set, so M each but mechanical.
4. Tests 10, 11, 14, 22, 27 — collection-managed and `UpdateDelegate` authority paths (these also cover the otherwise-unreached `is_valid_asset_authority` / `is_valid_collection_authority` delegate branches in `utils/mod.rs`).
5. Test 36 — grow/shrink with trailing plugin (`sol_memmove` both directions, `bump_offsets`). One M test closes the last ~40 lines of `groups_plugin_utils.rs`.
6. Tests 29, 35, 37, 41(inconsistent row), 25, 42, 43 — defensive/dead-branch and documentation tests; low value but cheap, needed only for the last few percent.
7. Tests 44, 45 — belong to burn / plugin-processor scopes; schedule with those sections but use these fixtures.

After 1-5 the twelve in-scope files should be at ≥95 % lines; the remaining misses are the `NumericalOverflow` arms in `save_updated_groups_plugin`, the `size_diff == 0` branch, `validate_burn`'s `_ => false`, and (unless 43 is added) `CoreAsset for GroupV1`.


## 11. Plugin engine, lifecycle validation, and external plugin adapters

All paths are relative to `programs/mpl-core`. Line numbers come from the current tree and the
`cargo llvm-cov` data in `the per-file coverage data in `. Instruction discriminators cited below are the
`MplAssetInstruction` variant indices (`src/instruction.rs`): CreateV1=0, CreateCollectionV1=1,
AddPluginV1=2, AddCollectionPluginV1=3, RemovePluginV1=4, RemoveCollectionPluginV1=5,
UpdatePluginV1=6, UpdateCollectionPluginV1=7, ApprovePluginAuthorityV1=8,
ApproveCollectionPluginAuthorityV1=9, RevokePluginAuthorityV1=10, RevokeCollectionPluginAuthorityV1=11,
BurnV1=12, TransferV1=14, UpdateV1=15, UpdateCollectionV1=16, CompressV1=17, DecompressV1=18,
CreateV2=20, CreateCollectionV2=21, AddExternalPluginAdapterV1=22, AddCollectionExternalPluginAdapterV1=23,
RemoveExternalPluginAdapterV1=24, RemoveCollectionExternalPluginAdapterV1=25,
UpdateExternalPluginAdapterV1=26, UpdateCollectionExternalPluginAdapterV1=27,
WriteExternalPluginAdapterDataV1=28, WriteCollectionExternalPluginAdapterDataV1=29, UpdateV2=30,
ExecuteV1=31.

### Scope summary

| File | Lines covered | Missed | Notes |
|---|---|---|---|
| src/plugins/utils.rs | 214/633 (33.8%) | 419 | Every `CollectionV1` monomorphization is 0%; all internal-plugin mutators are 0% |
| src/plugins/lifecycle.rs | 137/473 (29.0%) | 336 | Only `Plugin::validate_transfer` + FreezeDelegate routing ever ran |
| src/plugins/external_plugin_adapters.rs | 141/470 (30.0%) | 329 | Only the AgentIdentity arms ran |
| src/plugins/mod.rs | 199/276 (72.1%) | 77 | Rest is `Plugin`/`PluginType` dispatch tables |
| src/plugins/plugin_registry.rs | 216/255 (84.7%) | 39 | `bump_offsets` never moved a record |
| src/plugins/plugin_header.rs | 14/14 (100%) | 0 | The 2 "uncovered functions" are duplicate monomorphs across the two test builds; nothing to do |
| src/plugins/external/oracle.rs | 0/92 (0%) | 92 | No Oracle test exists in Mollusk |
| src/plugins/external/lifecycle_hook.rs | 0/28 | 28 | Blocked on-chain (`NotAvailable`) |
| src/plugins/external/linked_lifecycle_hook.rs | 0/28 | 28 | Blocked on-chain (`NotAvailable`) |
| src/plugins/external/app_data.rs | 5/23 (21.7%) | 18 | Only `update()` (unit test) ran |
| src/plugins/external/linked_app_data.rs | 0/20 | 20 | |
| src/plugins/external/data_section.rs | 0/6 | 6 | Only created by a LinkedAppData write |
| src/plugins/external/agent_identity.rs | 60/88 (68.2%) | 28 | Missing Create/Transfer/Burn/Update hooks |

### Why the numbers are what they are

The three Mollusk suites (`tests/account_ownership.rs`, `tests/agent_identity.rs`,
`tests/execution_delegate.rs`) exercise exactly: TransferV1 / BurnV1 / UpdateV1 /
UpdateCollectionV1 (almost all negative, failing before lifecycle validation), ExecuteV1 with an
AgentIdentity adapter, and CreateV2 / CreateCollectionV2 / AddExternalPluginAdapterV1 /
UpdateExternalPluginAdapterV1 / RemoveExternalPluginAdapterV1 for the AgentIdentity adapter only.
Nothing exercises AddPluginV1, RemovePluginV1, UpdatePluginV1, Approve/RevokePluginAuthorityV1 or
their collection variants, CreateV1/V2 with internal plugins, any Oracle / AppData / LinkedAppData /
DataSection adapter, either Write*ExternalPluginAdapterDataV1 instruction, or any collection-level
external-adapter instruction. That is why `initialize_plugin`, `delete_plugin`,
`approve_authority_on_plugin`, `revoke_authority_on_plugin`, `update_external_plugin_adapter_data`,
`fetch_wrapped_plugin`, `fetch_plugin`, `list_plugins`, `create_plugin_meta` and every
`Plugin::validate_*` router except `validate_transfer` are at 0%.

Reachability classes used below:

- **A – pure logic, unit-testable in-crate** (`PluginValidationContext` and most `validate_*` are
  `pub(crate)`, so these tests must live in `#[cfg(test)] mod` blocks under `src/plugins/`):
  `ExternalCheckResult`/`ExternalCheckResultBits` conversions, all `PluginType::check_*` tables,
  `PluginType::manager`, `From<&Plugin> for PluginType`, `Plugin::inner`, `PluginRegistryV1::bump_offsets`,
  `ExternalRegistryRecord::update`, `ExternalPluginAdapter::update/check_create/check_execute`,
  every `From<&*InitInfo>`, `ExternalPluginAdapterKey::from(&init_info)`, `ExtraAccount::derive`
  and `transform_seeds` (need a fake `AccountInfo`, trivially built from a `Vec<u8>`),
  `Oracle::validate_helper`, `validate_lifecycle_checks`, the result-combination matrices in
  `Plugin::validate_update_plugin` / `ExternalPluginAdapter::validate_update_external_plugin_adapter`,
  and the aggregation loops `validate_plugin_checks` / `validate_external_plugin_adapter_checks`.
- **B – Mollusk, mpl-core accounts only**: every internal-plugin instruction, AppData, LinkedAppData,
  DataSection, and all collection-level flows.
- **C – Mollusk with a foreign account of a specific layout (no CPI needed)**: Oracle (any account
  whose data holds a Borsh `OracleValidation` at `results_offset`; owner is never checked),
  AgentIdentity create/add (PDA signer of `mpl_agent_identity`, already in `tests/agent_identity.rs:74`),
  AgentIdentity execute (`ExecutionDelegateRecordV1` owned by `mpl_agent_tools`, already stubbed in
  `tests/execution_delegate.rs:191`).
- **D – dead or blocked on-chain**: LifecycleHook and LinkedLifecycleHook are refused at
  `src/plugins/utils.rs:337-343` (`NotAvailable`); there is no CPI to a hooked program anywhere in
  the program (their `validate_*` are `abstain!()`), so **no hook-program stub is required**. Their
  remaining code is reachable only by unit tests or by hand-crafting an account that already contains
  such an adapter (the program reads it without complaint). CompressV1/DecompressV1 return
  `NotAvailable` (`src/processor/compress.rs:81`, `decompress.rs:85`), so `RegistryRecord::compare_offsets`,
  `PluginType::check_compress/decompress` and `Plugin::validate_compress/decompress` are dead.
  `assert_plugins_initialized`, `fetch_plugins` and `ExternalPluginAdapter::check_execute` have no
  callers at all.

### src/plugins/utils.rs (214/633)

| Path | Lines | Trigger | Existing test to port | Class |
|---|---|---|---|---|
| `create_meta_idempotent` "already exists" branch, `CollectionV1` instantiation | 60-66 (asset variant is covered), fn-level for `CollectionV1` | AddCollectionPluginV1 (3), AddCollectionExternalPluginAdapterV1 (23), CreateCollectionV2 (21) with `external_plugin_adapters`, group instructions | JS `addPlugin.test.ts` 'it can add a plugin to a collection'; Rust `create_collection_with_external_plugins.rs::test_create_oracle_on_collection` | B |
| `create_plugin_meta` (both instantiations) | 70-102 | CreateV1/V2 with non-empty `plugins` (`src/processor/create.rs:185`), CreateCollectionV1/V2 with non-empty `plugins` (`create_collection.rs:124`) | JS `create.test.ts` 'it can create a new asset in account state with plugins'; Rust `create.rs::create_asset_with_plugins`, `create_collection.rs::create_collection_with_plugins` | B |
| `assert_plugins_initialized` | 105-114 | no callers in the program | dead; unit test only (or delete) | D |
| `fetch_plugin::<CollectionV1, UpdateDelegate>` | 117-153 | UpdateV2 (30) moving an asset into a collection that carries an UpdateDelegate (`src/processor/update.rs:212`). 123-125 (`PluginNotFound`, no header) is unreachable from that caller (guarded by `plugin_set.contains`); 140-142 (type mismatch after registry lookup) is defensive, only reachable with a corrupted registry | JS `updateV2.test.ts` 'it can add asset to collection using additional update delegate on new collection', 'it can change an asset collection using delegate' | B |
| `fetch_wrapped_plugin` (both instantiations, `core: Some` and `None`) | 156-189 | RemovePluginV1 (4) / RevokePluginAuthorityV1 (10) pass `Some(core)`; UpdatePluginV1 (6) / ApprovePluginAuthorityV1 (8) pass `None`; `src/utils/mod.rs:586,627` look up UpdateDelegate for additional-delegate resolution; `update_delegate.rs:190`. 164-168: `None` core on a plugin-less asset -> `PluginNotFound` (ApprovePluginAuthorityV1 on a bare asset); 182: type not in registry -> `PluginNotFound` | JS `removePlugin.test.ts` 'it can remove a plugin from an asset'; `approveAuthority.test.ts` 'it can add an authority to a plugin'; `updateDelegate.test.ts` 'an updateDelegate additionalDelegate can update an asset' | B |
| `fetch_wrapped_external_plugin_adapter` error edges + `CollectionV1` instantiation | 203, 224; fn-level `CollectionV1` | 203: `core: None` on a plugin-less asset (UpdateExternalPluginAdapterV1 on a bare asset); 224: key not in registry (RemoveExternalPluginAdapterV1 with an Oracle key on an asset that only has AgentIdentity). Collection: UpdateCollectionExternalPluginAdapterV1 (27), RemoveCollectionExternalPluginAdapterV1 (25), WriteCollectionExternalPluginAdapterDataV1 (29) | Rust `update_external_plugins_on_collection.rs::test_update_oracle_on_collection`, `remove_external_plugins_on_collection.rs::test_remove_oracle_on_collection` | B |
| `fetch_plugins` | 229-241 | no callers in the program | dead; unit test only | D |
| `list_plugins::<CollectionV1>` | 244-261 | UpdateV2 (30) moving an asset into a collection that has any plugins (`update.rs:193`) | JS `updateV2.test.ts` 'it cannot add asset to collection if new collection contains permanent freeze delegate' (`PermanentDelegatesPreventMove`) | B |
| `initialize_plugin` (both instantiations) | 265-319 | CreateV1/V2 with plugins, CreateCollectionV1/V2 with plugins, AddPluginV1 (2), AddCollectionPluginV1 (3), `groups_plugin_utils.rs`. 279-285: `PluginAlreadyExists` on AddPluginV1 for a type already present | JS `addPlugin.test.ts` 'it can add a plugin to an asset', `freeze.test.ts` 'it cannot add multiple freeze plugins to an asset' | B |
| `initialize_external_plugin_adapter`: `NotAvailable` for hooks | 340 | AddExternalPluginAdapterV1 (22) with `LifecycleHook`; CreateCollectionV2 / AddCollectionExternalPluginAdapterV1 with `LifecycleHook` or `LinkedLifecycleHook` (on assets `LinkedLifecycleHook` is rejected earlier with `InvalidPluginAdapterTarget`, `add_external_plugin_adapter.rs:55`) | Rust `add_external_plugins.rs::test_temporarily_cannot_add_lifecycle_hook`, `..._on_collection`; JS `lifecycleHook.test.ts` (all `test.skip`) | B (blocked path itself is testable) |
| `initialize_external_plugin_adapter`: hook/Oracle/AppData/LinkedAppData/DataSection arms | 355-374, 377-385 (351 is a brace after the covered `return`) | Oracle: any Oracle add/create (369-374 also runs `validate_lifecycle_checks(.., true)`); AppData: AddExternalPluginAdapterV1 with AppData; LinkedAppData: AddCollectionExternalPluginAdapterV1; DataSection (385): first LinkedAppData write onto an asset (`write_external_plugin_adapter_data.rs:266`) | Rust `add_external_plugins.rs::test_add_oracle`, `test_add_app_data`; JS `linkedAppData.test.ts` | B |
| `initialize_external_plugin_adapter`: data offset/len bookkeeping | 414-417, 439 | Adapter with data: AppData or DataSection (LifecycleHook is blocked). `appended_data: Some` only from the DataSection path | JS `appData.test.ts`; `linkedAppDataMembership.test.ts` 'LinkedAppData write succeeds for a legitimate collection member' | B |
| `initialize_external_plugin_adapter`: `appended_data` memcpy | 462-468 | Same as above, DataSection init (data written in the same instruction) | JS `linkedAppData.test.ts` | B |
| `update_external_plugin_adapter_data` (entire fn) | 478-570 | WriteExternalPluginAdapterDataV1 (28) / WriteCollectionExternalPluginAdapterDataV1 (29) on an AppData adapter, or a second LinkedAppData write (DataSection exists). Branches: `size_diff > 0` (525-528, realloc-then-memmove), `< 0` (543-546, memmove-then-realloc), `== 0`; inline `data` vs `buffer` account (processor). 489-490 (`InvalidPlugin`, no data offset) and 559-563 (record not found) are unreachable from the processor: non-data adapters are rejected earlier with `UnsupportedOperation` and the record is loaded from the same registry | Rust `plugin_shrink_corruption.rs::test_write_external_plugin_adapter_data_shrink_preserves_second_plugin`, `..._single_plugin_shrink`; JS `appData.test.ts` 'Data offsets are correctly bumped when rewriting other external plugins to be larger' / 'smaller'; Rust `update_external_plugins.rs::test_update_app_data` | B |
| `validate_lifecycle_checks` errors | 577 (`RequiresLifecycleCheck`), 586 (`DuplicateLifecycleChecks`), 590-596 (`OracleCanRejectOnly`) | Oracle/AgentIdentity init or update with empty checks; duplicate events; Oracle with any flag other than `0x4` (e.g. `0x1` listen, `0x2` approve, `0x6`) | JS `oracle.test.ts` 'it cannot create asset with oracle that has no lifecycle checks', 'it cannot add oracle to asset that can approve', 'it cannot update oracle to listen'; Rust `add_external_plugins.rs::test_cannot_add_oracle_with_duplicate_lifecycle_checks` | A (direct) or B |
| `delete_plugin` (entire fn) | 603-681 | RemovePluginV1 (4) / RemoveCollectionPluginV1 (5). Covers memmove of trailing plugins and `bump_offsets` when the removed plugin is not the last. 610-612 and 677 (`PluginNotFound`) are unreachable from the processor (it already fetched the plugin) | JS `removePlugin.test.ts` 'it can remove a plugin from asset with existing plugins', 'it can remove authority managed plugin from collection'; `appData.test.ts` 'Data offsets are correctly bumped when removing other plugins' (internal plugin removed before an AppData with data) | B |
| `delete_external_plugin_adapter` error edges; data-bearing removal | 692, 760; 711 with `data_len: Some(n>0)` | 692/760 unreachable from the processor (already fetched). Data-bearing: RemoveExternalPluginAdapterV1 of an AppData that has data, with another adapter after it | JS `appData.test.ts` 'Data offsets are correctly bumped when removing other external plugins with data'; Rust `remove_external_plugins.rs::test_remove_app_data` | B |
| `approve_authority_on_plugin` | 768-804 | ApprovePluginAuthorityV1 (8) / ApproveCollectionPluginAuthorityV1 (9). `size_diff != 0` (791-799) whenever `Owner`/`UpdateAuthority` -> `Address` (+32 bytes); `size_diff == 0` for `Owner` -> `UpdateAuthority` | JS `approveAuthority.test.ts` 'it can add an authority to a plugin'; `updateDelegate.test.ts` 'it can approve/revoke the plugin authority of other plugins' | B |
| `revoke_authority_on_plugin` | 808-838 | RevokePluginAuthorityV1 (10) / RevokeCollectionPluginAuthorityV1 (11): authority reset to `plugin_type.manager()` (shrinks by 32 when revoking an `Address`) | JS `revokeAuthority.test.ts` 'it can remove an authority from a plugin', 'it can remove the default authority from a plugin to make it immutable' | B |
| `find_external_plugin_adapter[_mut]` non-match iterations; `check_plugin_key` non-AgentIdentity arms | 848, 851, 864, 867; 877-887 (pubkey keys), 890-901 (authority keys), 904-914 (DataSection), 923 (type mismatch) | Any Update/Remove/Write on an Oracle (pubkey key), AppData / LinkedAppData (authority key), or the DataSection lookup in `write_external_plugin_adapter_data.rs:228`. Type mismatch / not-found: registry holding two adapter types and looking up the second, or a key that is absent | JS `oracle.test.ts` 'it can add multiple oracles and internal plugins to asset' (+ update one of them); Rust `update_external_plugins.rs::test_update_oracle`, `test_update_app_data` | B |

### src/plugins/lifecycle.rs (137/473)

| Path | Lines | Trigger | Existing test | Class |
|---|---|---|---|---|
| `ExternalCheckResult::can_reject_only` | 52-54 | any Oracle init/update (via `validate_lifecycle_checks(.., true)`) | Rust `add_external_plugins.rs::test_add_oracle` | A/B |
| `ExternalCheckResultBits::set_*_checked` (modular_bitfield setters) | 61-64 | never called by the program; a unit test using `ExternalCheckResultBits::new().with_can_reject(true)` covers 61-63; 64 needs `with_empty_bits` | new unit | A |
| `From<ExternalCheckResultBits> for ExternalCheckResult` | 74-78 | no callers in the program | new unit (round-trip) | A / dead on-chain |
| `PluginType::check_add_plugin` | 83-96 | AddPluginV1 on an asset/collection that already has plugins (`check_registry`) | JS `addBlocker.test.ts` 'it cannot add UA-managed plugin if addBlocker had been added on creation' (CanReject arm), `updateDelegate.test.ts` 'an updateDelegate can add a plugin to an asset' (CanApprove) | A (table) + B |
| `check_remove_plugin` | 101-113 | RemovePluginV1 on an asset with >=2 plugins (every type is CanReject) | JS `removePlugin.test.ts` 'it cannot remove a plugin from a frozen asset' | A + B |
| `check_update_plugin`, `check_approve_plugin_authority`, `check_revoke_plugin_authority` | 116-139 | UpdatePluginV1 / ApprovePluginAuthorityV1 / RevokePluginAuthorityV1 on an asset with plugins | JS `updatePlugin.test.ts`, `approveAuthority.test.ts`, `revokeAuthority.test.ts` | A + B |
| `check_create` | 142-151 | CreateV1/V2 with internal plugins (`create.rs:200`), or create into a collection that has plugins (`check_registry` on the collection) | JS `create.test.ts` 'it can create a new asset in account state with plugins'; `royalties.test.ts` (Royalties -> CanReject) | A + B |
| `check_update`, `check_burn`, `check_transfer` (non-FreezeDelegate arms), `check_execute` | 154-161, 164-173, 178-185, 204-211 | UpdateV1/V2, BurnV1, TransferV1, ExecuteV1 on assets whose registry has the respective plugin types (Royalties, TransferDelegate, PermanentFreeze/Transfer/BurnDelegate, ImmutableMetadata, UpdateDelegate, Groups, FreezeExecute, PermanentFreezeExecute). Existing Mollusk tests only put FreezeDelegate on the asset and ExecuteV1 assets have no internal plugins | JS `plugins/asset/*.test.ts` (one per plugin), `freezeExecute.test.ts` | A + B |
| `check_compress` / `check_decompress` | 188-201 | CompressV1/DecompressV1 are `NotAvailable` before reaching validation | dead on-chain; unit only | D |
| `check_add/remove/update_external_plugin_adapter` | 214-236 | AddExternalPluginAdapterV1 / Remove.../Update... on an asset that also has internal plugins (existing AgentIdentity tests have none). BubblegumV2 -> CanReject on add | JS `plugins/collection/bubblegumV2.test.ts` (cannot add external adapter to BubblegumV2 collection); Rust `add_external_plugins.rs::test_add_oracle` (asset with plugins) | A + B |
| `Plugin::validate_add_plugin` .. `validate_remove_external_plugin_adapter` routers (all but `validate_transfer`) | 241-246, 249-261, 264-279, 282-315, 318-331, 335-374, 377-382, 393-406, 409-414, 417-430 | Reached from `validate_plugin_checks` once a check table returns CanApprove/CanReject. Specific branches: 253-257 `validate_remove_plugin` reject when self authority is `None` and self is the target (JS `removePlugin.test.ts` 'it cannot remove an authority managed plugin when the authority is None'); 270-276 `CannotRedelegate` when target already delegated (JS `approveAuthority.test.ts` 'it cannot reassign authority of a plugin while already delegated'), 275 `InvalidPlugin` when `target_plugin` is `None` (unreachable from the processor, unit only); 289-293 revoke rejected for `Authority::None`; 295-306 base Approved when signer resolves to self authority; 339-341 `InvalidAuthority` when `resolved_authorities` is `None` (unreachable from processors, unit only); 356-373 combination matrix (unit test: iterate all (base, inner) pairs with a stub plugin; on-chain only (Approved,Approved), (Approved,Pass), (Pass,*), (Approved,Rejected e.g. FreezeDelegate frozen update by owner) occur). `validate_compress/decompress` (393-406) dead on-chain | JS `updatePlugin.test.ts` (4 positive/negative pairs), `updateDelegateRevokeBug.test.ts`, `freeze.test.ts` 'owner cannot undelegate a freeze plugin with a delegate' | A (matrix) + B |
| `From<ExternalValidationResult> for ValidationResult` | 495-501 | Oracle validation | any Oracle deny/allow test | A/C |
| `PluginValidation` default bodies | 544-661 | Any plugin that does not override the method being routed. Broadest trigger: RemovePluginV1 (every type is CanReject) on an asset with e.g. Attributes + FreezeDelegate; AddExternalPluginAdapterV1 with an internal plugin present hits `validate_add_external_plugin_adapter` default; `validate_execute` default via ExecuteV1 with any non-execute plugin flagged CanReject (only FreezeExecute/PermanentFreezeExecute are) | JS `removePlugin.test.ts` 'it can remove a plugin from asset with existing plugins' | B |
| `validate_plugin_checks`: collection-keyed record, Rejected/Approved/ForceApproved results, final reject/approve | 703 (collection account), 705 (`unreachable!`, never), 728 (error edge), 730-731, 733, 739, 741 | 703: asset in a collection whose registry has plugins (Royalties/PermanentFreezeDelegate on collection, TransferV1 with collection passed); 730/739: TransferV1 on a frozen FreezeDelegate; 731/741: TransferV1 by a TransferDelegate `Address` authority; 733: BurnV1 by PermanentBurnDelegate authority / TransferV1 by PermanentTransferDelegate (`ForceApproved`) | JS `pluginValidationOverrides.test.ts` (all 5), `freeze.test.ts` 'it can freeze and unfreeze an asset', `delegateTransfer.test.ts`, `permanentBurn.test.ts` | B |
| `validate_external_plugin_adapter_checks`: collection-keyed, Rejected+can_reject | 777 (collection account), 779/816 (`unreachable!`), 802 (error edge), 805-807, 818 | 777: Oracle on the collection, UpdateV1/TransferV1/BurnV1 on a member asset with the collection passed; 805-807: any Oracle that denies (`OracleValidation::V1 { transfer: Rejected, .. }`). Approved path (809-812) is already covered by the AgentIdentity execute tests | JS `oracle.test.ts` 'it can use fixed address oracle to deny update via collection', 'it can use fixed address oracle to deny transfer' | C |

### src/plugins/external_plugin_adapters.rs (141/470)

| Path | Lines | Trigger | Existing test | Class |
|---|---|---|---|---|
| `From<&ExternalPluginAdapterKey> for ExternalPluginAdapterType`, non-AgentIdentity arms | 69-76 | Update/Remove/Write instructions keyed by `LifecycleHook`/`Oracle`/`AppData`/`LinkedLifecycleHook`/`LinkedAppData`/`DataSection` (via `check_plugin_key`). Hook keys only via crafted state | Rust `update_external_plugins.rs::test_update_oracle`, `test_update_app_data`; JS `linkedAppData.test.ts` | A + B |
| `From<&ExternalPluginAdapterInitInfo> for ExternalPluginAdapterType` / `for ExternalPluginAdapter` / `for ExternalPluginAdapterKey`, non-AgentIdentity arms | 86-96, 526-542, 917-933 | Any create/add of Oracle, AppData, LinkedAppData (collection), LifecycleHook/LinkedLifecycleHook (they reach `From` before `NotAvailable` in `initialize_external_plugin_adapter:406`; also at `add_external_plugin_adapter.rs:65` and `create.rs:274`), DataSection (only from the LinkedAppData write path) | Rust `create_with_external_plugins.rs::test_create_oracle`, `test_create_app_data`, `test_temporarily_cannot_create_lifecycle_hook` | A + B |
| `From<&ExternalPluginAdapter> for ExternalPluginAdapterType`, non-AgentIdentity arms | 107-114 | `validate_update_external_plugin_adapter:379` for Oracle/AppData/LinkedAppData updates | Rust `update_external_plugins.rs::test_update_oracle` | A + B |
| `ExternalPluginAdapter::update` arms: LifecycleHook, Oracle, LinkedLifecycleHook, LinkedAppData | 154-164, 172-182 | UpdateExternalPluginAdapterV1 with Oracle (`base_address_config`/`results_offset`), UpdateCollectionExternalPluginAdapterV1 with LinkedAppData (`schema`); hook arms via unit test or crafted account | JS `oracle.test.ts` 'it can update oracle to larger registry record'; Rust `update_external_plugins_on_collection.rs` | A + B |
| `check_create` arms other than AgentIdentity | 198-233 | CreateV1/V2 with Oracle/AppData (runs before the target check at `create.rs:253`, so LinkedAppData/DataSection/LinkedLifecycleHook arms (221-233) are also reached and then error `InvalidPluginAdapterTarget`/`CannotAddDataSection`); CreateCollectionV2 likewise. `checks.1` vs `none()` branch for each hookable adapter depends on whether `Create` is in `lifecycle_checks` | JS `dataSection.test.ts` 'it cannot create an asset with a DataSection', `linkedAppData.test.ts` 'it cannot create an asset with linked app data', `oracle.test.ts` 'it can use fixed address oracle to deny create' | A + B |
| `validate_create` (all arms incl. the `msg!`) | 249-270 | Only called when `check_create(..).can_reject()` (`create.rs:262`): CreateV2 with Oracle `(Create, 0x4)` or AgentIdentity `(Create, 0x4)`; existing AgentIdentity tests use `(Execute, 0x1)` so it never ran. 265 DataSection -> Rejected is unreachable from the processor (rejected earlier); 260-263 hook arms via unit test | JS `oracle.test.ts` 'it can use fixed address oracle to deny create', 'it can create an oracle on a collection with create set to reject' | A + C |
| `validate_update`, `validate_burn`, `validate_transfer` routers | 273-292, 295-314, 317-336 | UpdateV1/V2, BurnV1, TransferV1 on an asset (or member of a collection) whose adapter lists the event with any nonzero flag: AgentIdentity `(Update|Burn|Transfer, 0x1)` -> abstain arms; Oracle `(.., 0x4)` -> deny/allow. DataSection arms (287/309/331) need a DataSection with `lifecycle_checks: Some` which the program never writes (`utils.rs:385` stores `None`) -> unit only | JS `oracle.test.ts` deny update/burn/transfer tests | A + B/C |
| `validate_add_external_plugin_adapter` arms | 344-358, 360 | AddExternalPluginAdapterV1 Oracle/AppData; AddCollectionExternalPluginAdapterV1 LinkedAppData/LinkedLifecycleHook (Pass); 360 DataSection unreachable from processors (`CannotAddDataSection` first) | Rust `add_external_plugins.rs::test_add_oracle`, `test_add_app_data` | A + B |
| `validate_update_external_plugin_adapter`: base `Pass` (wrong authority), non-AgentIdentity arms, result matrix | 384, 388-401, 404, 408, 412, 415, 418, 421, 423-424, 426 | 384/423-424: signer not in `resolved_authorities` for the record's authority -> `Pass` -> processor `InvalidAuthority` (JS 'it cannot update oracle using update authority when different from external plugin authority', also doable with AgentIdentity today); 388-401: Oracle/AppData/LinkedAppData/hook updates. 411-422 (Approved/Rejected combos) are **unreachable on-chain**: no adapter overrides `validate_update_external_plugin_adapter`, so `result` is always `Pass`; 404 DataSection unreachable because `ExternalPluginAdapter::update` fails first (no DataSection update-info variant); 426 `unreachable!` | JS `oracle.test.ts` 'it cannot update oracle using update authority when different from external plugin authority'; Rust `update_external_plugins.rs::test_update_oracle` | A (matrix) + B |
| `check_execute` | 431-481 | no callers in the program (ExecuteV1 does not gate on it) | dead; unit only | D |
| `validate_execute` non-AgentIdentity arms | 489-498 | ExecuteV1 on an asset whose Oracle/AppData/... record lists `Execute` (Oracle accepts `Execute` in `lifecycle_checks`; AppData/LinkedAppData/DataSection have `lifecycle_checks: None` so they are never selected -> unit only for those) | new | B/C (Oracle), A (others) |
| `ExternalPluginAdapter::load` / `save` error branches | 509-511, 517-519 | 509-511: crafted account whose `ExternalRegistryRecord.offset` points at garbage -> `DeserializationError` on TransferV1 with the event registered; 517-519: unreachable in practice (account was just resized) | new (crafted) | B (crafted) |
| `ExtraAccount::derive` (all arms) and `transform_seeds` | 648-705, 709-759 | Only Oracle uses it (`oracle.rs:84`). Variants: PreconfiguredProgram / Collection (needs collection, else `MissingCollection`) / Owner (loads `AssetV1` from `asset_info`, `MissingAsset` on collection ops) / Recipient (`MissingNewOwner` unless TransferV1) / Asset / CustomPda with every `Seed` variant and `custom_program_id` / Address | JS `oracle.test.ts` 'it can use preconfigured program pda oracle to deny update', '... collection pda ...', '... owner pda ... burn', '... recipient pda ... transfer', '... asset pda ... update', 'custom pda (all seeds)', 'custom pda (typical)', 'custom pda (with custom program ID)' | A (fake ctx) + C |
| `ExternalPluginAdapterKey::from_record` non-AgentIdentity arms | 877-907 | `check_adapter_registry` (any lifecycle op on an asset with an Oracle/AppData/... record whose checks match) and duplicate detection in `initialize_external_plugin_adapter:347` | Rust `add_external_plugins.rs::test_cannot_add_duplicate_external_plugin_adapter` | B |

### src/plugins/mod.rs (199/276)

| Path | Lines | Trigger | Class |
|---|---|---|---|
| `Plugin::manager` | 74-76 | CreateV1/V2 with plugins (default authority), AddPluginV1, `validate_approve_plugin_authority:271` | B |
| `Plugin::load` error branch | 82-84 | Crafted account: registry offset pointing at invalid bytes, then TransferV1 with a FreezeDelegate record -> `DeserializationError` | B (crafted) |
| `Plugin::save` | 88-93 | `initialize_plugin`, UpdatePluginV1 | B |
| `Plugin::inner` arms other than FreezeDelegate | 98, 100-116 | Each plugin type routed through a `Plugin::validate_*`. A unit test iterating the fixture list already in `test_plugin_empty_size` and calling `inner().validate_transfer(&ctx)` covers all arms at once | A |
| `From<&Plugin> for PluginType` | 236-258 | Any add/remove/update-plugin instruction; unit test over the same fixture list | A |
| `PluginType::manager` arms | 265-267, 269-285 | Revoke (`revoke_authority_on_plugin:823`), create with plugins; unit test over `PluginType::iter()` | A |

### src/plugins/plugin_registry.rs (216/255)

| Path | Lines | Trigger | Class |
|---|---|---|---|
| `check_adapter_registry` loop closers | 73, 75 | Artifacts of the `?` at line 63; covered once a non-AgentIdentity record is iterated | — |
| `bump_offsets`: internal-registry shift | 84-90 | Any size change with an internal plugin located after the change point: UpdatePluginV1 growing/shrinking a plugin that precedes another, RemovePluginV1 of a non-last plugin, UpdateV1 name/uri size change with plugins (`update.rs:419`), AppData write with an internal plugin after it. Unit test: registry with 3 records, `bump_offsets(mid, +5)` and `(mid, -5)`; `NumericalOverflow` on negative result (87-89, 97-99, 106-108) only reachable in a unit test | A + B |
| `bump_offsets`: external-registry shift incl. `data_offset` | 95-111 | Asset with two external adapters where the first changes size (UpdateExternalPluginAdapterV1 on the leading Oracle; AppData data write with a second AppData/Oracle after it) | A + B (JS `oracle.test.ts` 'it can shrink/grow a leading oracle without corrupting trailing oracle metadata', Rust `plugin_shrink_corruption.rs`) |
| `RegistryRecord::compare_offsets` | 155-157 | Only `utils/compression.rs:82` (Compress = `NotAvailable`) | D, unit only |
| `ExternalRegistryRecord::update`: LifecycleHook, Oracle, `_` arms | 188-193, 195-200, 209 | Oracle: UpdateExternalPluginAdapterV1 with `lifecycle_checks: Some` (also exercises `OracleCanRejectOnly` on update); `_`: AppData/LinkedAppData/LinkedLifecycleHook update-info; LifecycleHook arm: unit test (blocked on-chain). Note the `LinkedLifecycleHook` update-info falls into `_` so its `lifecycle_checks` can never be updated (asymmetric with `LifecycleHook`; moot while blocked) | A + B |

### src/plugins/external/oracle.rs (0/92)

Entire file is Oracle-only; every path is class C on-chain (needs an account whose bytes hold
`OracleValidation`) or class A with a fake `AccountInfo`. No CPI, no owner check on the oracle account.

| Path | Lines | Trigger | JS test |
|---|---|---|---|
| `Oracle::update` | 29-36 | UpdateExternalPluginAdapterV1 with `base_address_config: Some` and/or `results_offset: Some` | 'it can update oracle to larger registry record' (adds a config) |
| `validate_add_external_plugin_adapter` | 40-45 | AddExternalPluginAdapterV1 / CreateV2 with Oracle | Rust `test_add_oracle` |
| `validate_create/transfer/burn/update` + `validate_helper` | 47-73, 77-123 | Lifecycle op with the matching event registered `(event, 0x4)`. Branches: 83 fixed address vs 84 derived; 91 `MissingExternalPluginAdapterAccount` (oracle account not passed); 97-98 `InvalidOracleAccountData` (offset past end); 100-102 (fewer than 5 bytes at offset); 104-105 (bad discriminant byte, e.g. `2`); 108 `UninitializedOracleAccount` (all-zero account); 115-118 per-event arms; 119 `Execute` arm is **dead** (`validate_execute` for Oracle is the trait default, never routed through the helper) | 'it can use fixed address oracle to deny create/update/transfer/burn', 'it transfer fails but does not panic when oracle account does not exist', '... is too small', 'it empty account does not default to valid oracle', 'it cannot use fixed address oracle to deny transfer if not registered for lifecycle event' |
| `From<&OracleInitInfo>` | 126-134 | any Oracle create/add; `results_offset: None` -> `NoOffset` | 'add oracle to asset with no offset' |
| `ValidationResultsOffset::to_offset_usize` | 184-190 | `NoOffset` / `Anchor` (8) / `Custom(n)` | 'it can use preconfigured asset pda custom offset oracle to deny update' |
| `OracleValidation::serialized_size` | 213-215 | with `validate_helper` | — |

### src/plugins/external/lifecycle_hook.rs, linked_lifecycle_hook.rs (0/28 each)

Class D. `initialize_external_plugin_adapter` refuses both (`utils.rs:337-343`), and there is no
hook CPI in the program; `validate_add_external_plugin_adapter` / `validate_transfer` are constant
`abstain!()`/`Pass`. Coverage options: (1) unit tests for `update()`, `From<&*InitInfo>` and the two
validators (10 lines each); (2) a "pre-existing state" Mollusk test: hand-build an asset whose
registry already holds a `LifecycleHook` record with `(Transfer, 0x1)` and run TransferV1 — this
additionally covers `from_record:877-880`, `check_plugin_key:877-887`, `validate_transfer` routing
at `external_plugin_adapters.rs:322-324`, and `ExternalRegistryRecord::update:188-193` if followed by
UpdateExternalPluginAdapterV1 with a `LifecycleHook` key. The `From<&LifecycleHookInitInfo>` arms
(58-65) are also reached on-chain by the blocked add path, since conversion happens before the
`NotAvailable` check (`add_external_plugin_adapter.rs:65`, `utils.rs:406` is after; `create.rs:274`).

### src/plugins/external/app_data.rs (5/23)

| Path | Lines | Trigger | Class |
|---|---|---|---|
| `validate_add_external_plugin_adapter` | 35-40 | AddExternalPluginAdapterV1 AppData (Rust `test_add_app_data`) | B |
| `validate_transfer` | 42-47 | AppData records are stored with `lifecycle_checks: None` (`utils.rs:376-383`), so `check_adapter_registry` never selects them: **dead on-chain**, unit only | A |
| `From<&AppDataInitInfo>` | 51-56 | any AppData create/add; `schema: None` -> `Binary` default | B |

### src/plugins/external/linked_app_data.rs (0/20)

| Path | Lines | Trigger | Class |
|---|---|---|---|
| `update` | 24-28 | UpdateCollectionExternalPluginAdapterV1 with `LinkedAppDataUpdateInfo { schema: Some }` (JS `linkedAppData.test.ts` 'it can update linked app data on collection with external plugin authority different than asset update authority') | B |
| `validate_create` | 32-42 | `check_create` returns `none()` for LinkedAppData (`external_plugin_adapters.rs:232`) so `can_reject()` is false and the processor never calls it; on assets `InvalidPluginAdapterTarget` fires first. **Dead on-chain**, unit only (both `asset_info` Some/None branches) | A |
| `From<&LinkedAppDataInitInfo>` | 46-51 | CreateCollectionV2 / AddCollectionExternalPluginAdapterV1 with LinkedAppData (JS `linkedAppData.test.ts`, `linkedAppDataMembership.test.ts`) | B |

### src/plugins/external/data_section.rs (0/6)

`From<&DataSectionInitInfo>` (18-23) is reached only from `write_external_plugin_adapter_data.rs:266-291`
(first LinkedAppData write onto a member asset). Port JS `linkedAppDataMembership.test.ts`
'LinkedAppData write succeeds for a legitimate collection member' (positive) plus 'LinkedAppData write
is rejected when the asset is not a member of the supplied collection' (`InvalidCollection`, processor
only). Class B.

### src/plugins/external/agent_identity.rs (60/88)

| Path | Lines | Trigger | Class |
|---|---|---|---|
| `validate_create` | 76-89 | CreateV2 with AgentIdentity whose checks include `(Create, 0x4)` (only then does `create.rs:262` call it); 81-85 PDA-signer branch (`AgentIdentityMustSign` when the PDA is missing/unsigned/wrong — mirror the three existing add-path negatives), 86-88 collection branch unreachable from CreateCollectionV2 (`InvalidPluginAdapterTarget` first) -> unit | C (existing PDA fixture) |
| `validate_add_external_plugin_adapter` collection reject | 102 | unreachable from AddCollectionExternalPluginAdapterV1 (rejected at `add_external_plugin_adapter.rs:172`) -> unit | A |
| `validate_transfer/burn/update` | 106-125 | TransferV1 / BurnV1 / UpdateV1 on an asset built with `(Transfer|Burn|Update, 0x1)` — the existing `build_asset_with_agent_identity` helper takes `lifecycle_checks`, so these are three small additions | B |

### Bugs, suspicious logic, security-relevant notes

1. `src/plugins/utils.rs:567` — `update_external_plugin_adapter_data` saves the plugin header at
   `core.map_or(0, ..)`. With `core: None` the 9-byte header would overwrite the start of the
   `AssetV1`/`CollectionV1` data. All current callers pass `Some(core)`; latent footgun.
2. `src/plugins/utils.rs:830-833` — `revoke_authority_on_plugin` casts `new_size as usize` without the
   `try_into` guard used everywhere else. Harmless today (shrink is at most 32 bytes) but inconsistent.
3. `src/plugins/external/oracle.rs:119-120` — an Oracle may register `HookableLifecycleEvent::Execute`
   with `can_reject` (nothing in `validate_lifecycle_checks` prevents it), but `ExternalPluginAdapter::validate_execute`
   routes Oracle to the trait default (`abstain!()`), never through `validate_helper`. A creator who
   configures an oracle to gate Execute gets no enforcement. The `Execute => Pass` arm is dead.
4. `src/plugins/external_plugin_adapters.rs:411-422` — the Rejected/Approved combinations in
   `validate_update_external_plugin_adapter` cannot occur because no adapter overrides the inner
   validator; the effective rule is "signer must resolve to the record's authority".
5. `src/plugins/plugin_registry.rs:209` — `ExternalRegistryRecord::update` ignores
   `LinkedLifecycleHookUpdateInfo.lifecycle_checks` while honouring the `LifecycleHook` one. Moot
   while both are blocked, but will bite when un-blocked.
6. `src/plugins/utils.rs:572-600` — `validate_lifecycle_checks` does not reject unknown flag bits
   (e.g. `0xFFFF_FFF8`) for non-Oracle adapters; `ExternalCheckResultBits::from` silently ignores them.
7. `src/plugins/external_plugin_adapters.rs:431-481` — `check_execute` is dead. Unlike Create, the
   Execute path never consults an adapter's declared `ExternalCheckResult` at add time; enforcement is
   purely in `validate_external_plugin_adapter_checks`, which is fine but the function is misleading.
8. `check_plugin_key` / `from_record` (`utils.rs:885`, `external_plugin_adapters.rs:878`) index
   `account.data.borrow()[offset..]` unchecked; a registry record with `offset >= data_len` panics
   (`ProgramFailedToComplete`) instead of returning `DeserializationError`. Only reachable with a
   corrupted program-owned account, so low impact, but a crafted-state test would document it.
9. `src/plugins/lifecycle.rs:271` — `validate_approve_plugin_authority` compares
   `plugin_to_approve == plugin` by full value, not by `PluginType`. Correct today because the target
   is the freshly fetched on-chain plugin, but fragile if a caller ever passes instruction-supplied data.

### Test plan

Sizes: S = one instruction + existing builders, M = new builder or multi-instruction sequence,
L = new foreign-account fixture or several assertions on post-state layout. "port-from" names the
documented behaviour to copy.

| # | Test case | Sets up | Instruction(s) | Expected | Source paths covered | Port-from | Size |
|---|---|---|---|---|---|---|---|
| U1 | `external_check_result_bits_roundtrip` | unit | — | flags <-> bits identity for 0x0..0x7, setters | lifecycle.rs:48-78 | new | S |
| U2 | `plugin_type_check_tables` | unit, `PluginType::iter()` | — | assert each `check_*` table matches the documented matrix | lifecycle.rs:83-236 | new | S |
| U3 | `plugin_dispatch_tables` | unit, fixture list from `mod.rs` test | — | `PluginType::from`, `manager`, `inner().validate_transfer` per variant | mod.rs:74-76, 96-118, 236-288 | new | S |
| U4 | `validate_update_plugin_matrix` | unit, `PluginValidationContext` with fake `AccountInfo`, stub plugin per (base, inner) | — | all 8 arms incl. `ForceApproved`; `InvalidAuthority` when `resolved_authorities` None | lifecycle.rs:335-374; external_plugin_adapters.rs:368-428 | new | M |
| U5 | `validate_plugin_checks_aggregation` | unit, serialized asset bytes with FreezeDelegate(frozen) + TransferDelegate + PermanentTransferDelegate | — | Rejected wins over Approved; `ForceApproved` short-circuits; `InvalidCollection`/`InvalidAsset` when key account missing | lifecycle.rs:676-745, 751-826 | new | M |
| U6 | `bump_offsets_internal_and_external` | unit registry with 3 internal + 2 external (data_offset) records | — | offsets after pivot shift by +/-n; `NumericalOverflow` on underflow | plugin_registry.rs:82-116 | new | S |
| U7 | `external_registry_record_update_variants` | unit | — | Oracle rejects `0x1`; LifecycleHook accepts; AppData/LinkedLifecycleHook no-op | plugin_registry.rs:186-213; utils.rs:572-600 | new | S |
| U8 | `extra_account_derive_all_variants` | unit ctx with asset/collection/new_owner fakes | — | each `ExtraAccount`/`Seed` variant, and `MissingCollection`/`MissingAsset`/`MissingNewOwner` | external_plugin_adapters.rs:648-759 | JS oracle pda tests | M |
| U9 | `oracle_validate_helper_errors` | unit ctx with oracle `AccountInfo` variants | — | missing account, short data, bad discriminant, Uninitialized, each event arm, `Anchor`/`Custom` offsets | oracle.rs:77-123, 184-215 | JS oracle negative tests | M |
| U10 | `external_adapter_conversions` | unit, one `InitInfo` per variant | — | `From<&InitInfo>` for Type/Adapter/Key; `check_create`/`check_execute` with and without the event | external_plugin_adapters.rs:66-118, 196-246, 431-481, 523-549, 914-940; lifecycle_hook.rs, linked_lifecycle_hook.rs, app_data.rs:51-56, linked_app_data.rs, data_section.rs, oracle.rs:126-134 | new | S |
| U11 | `dead_validators_unit` | unit | — | `AppData::validate_transfer`, `LinkedAppData::validate_create` (asset vs collection), `AgentIdentity::validate_create`/`add` collection branch, DataSection Rejected arms | app_data.rs:42-47, linked_app_data.rs:32-42, agent_identity.rs:86-88,102, external_plugin_adapters.rs:265,360,404 | new | S |
| M1 | `create_v2_with_internal_plugins` | bare payer | CreateV2 (20) with FreezeDelegate + Attributes | success; parse header/registry, 2 records | utils.rs:70-102, 265-319; mod.rs:88-93; lifecycle.rs:142-151 | JS create 'with plugins'; Rust create.rs::create_asset_with_plugins | M |
| M2 | `create_collection_v2_with_plugins_and_oracle` | bare payer | CreateCollectionV2 (21) with Royalties + Oracle `(Update,0x4)` | success | utils.rs CollectionV1 monomorphs, 369-374; lifecycle.rs:52-54; oracle.rs:40-45,126-134 | Rust create_collection.rs; create_collection_with_external_plugins.rs::test_create_oracle_on_collection | M |
| M3 | `add_plugin_to_asset_and_duplicate` | asset with FreezeDelegate | AddPluginV1 (2) Attributes; AddPluginV1 FreezeDelegate again | success then `PluginAlreadyExists` | utils.rs:265-319 (279-285), lifecycle.rs:83-98, 241-246 | JS addPlugin, freeze 'cannot add multiple freeze plugins' | S |
| M4 | `add_plugin_blocked_by_add_blocker` | asset with AddBlocker | AddPluginV1 Attributes as UA | `InvalidAuthority` | lifecycle.rs:85, 730/739 | JS addBlocker.test.ts | S |
| M5 | `remove_middle_plugin_shifts_trailing` | asset with FreezeDelegate, Attributes, TransferDelegate | RemovePluginV1 (4) Attributes | success; trailing plugin still parses at bumped offset | utils.rs:156-189, 603-681; plugin_registry.rs:84-90; lifecycle.rs:101-113, 249-261, 544-558 | JS removePlugin 'remove a plugin from asset with existing plugins' | M |
| M6 | `remove_plugin_authority_none_rejected` | asset with Attributes (authority None) | RemovePluginV1 | `InvalidAuthority` | lifecycle.rs:253-257 | JS removePlugin 'cannot remove ... when the authority is None' | S |
| M7 | `remove_collection_plugin` | collection with Royalties + Attributes | RemoveCollectionPluginV1 (5) | success | utils.rs `CollectionV1` delete/fetch monomorphs | JS removePlugin 'remove authority managed plugin from collection' | S |
| M8 | `update_plugin_grow_and_shrink` | asset with Attributes then FreezeDelegate | UpdatePluginV1 (6) Attributes larger, then smaller | trailing FreezeDelegate intact | lifecycle.rs:116-121, 335-374; plugin_registry.rs:84-90; mod.rs:88-93 | Rust plugin_shrink_corruption.rs::test_update_plugin_shrink_attributes_preserves_trailing_plugins | M |
| M9 | `update_plugin_owner_vs_update_authority` | asset with FreezeDelegate(Owner) + Attributes(UA), signer = owner | UpdatePluginV1 Attributes | `InvalidAuthority`; then FreezeDelegate as owner succeeds | lifecycle.rs:343-373 combos | JS updatePlugin 4 pairs | S |
| M10 | `approve_plugin_authority_grow` | asset with FreezeDelegate(Owner) | ApprovePluginAuthorityV1 (8) -> Address | success, registry grew by 32 | utils.rs:156-189 (`None` core), 768-804 (791-799); lifecycle.rs:124-129, 264-279 | JS approveAuthority 'it can add an authority to a plugin' | S |
| M11 | `approve_plugin_authority_cannot_redelegate` | asset with FreezeDelegate(Address X), signer X | ApprovePluginAuthorityV1 -> Address Y | `CannotRedelegate` | lifecycle.rs:270-273 | JS approveAuthority 'cannot reassign authority ... while already delegated' | S |
| M12 | `approve_on_bare_asset_plugin_not_found` | bare asset | ApprovePluginAuthorityV1 | `PluginNotFound` | utils.rs:164-168 | new | S |
| M13 | `revoke_plugin_authority_shrinks` | asset with FreezeDelegate(Address X) | RevokePluginAuthorityV1 (10) as owner | authority back to `Owner`, account shrank | utils.rs:808-838; lifecycle.rs:132-139, 282-315; mod.rs:263-288 | JS revokeAuthority 'it can remove an authority from a plugin' | S |
| M14 | `revoke_collection_plugin_authority` | collection with Royalties(Address X) | RevokeCollectionPluginAuthorityV1 (11) | success | utils.rs CollectionV1 monomorphs | JS revokeAuthority | S |
| M15 | `transfer_frozen_rejected_and_delegate_approved` | asset with FreezeDelegate(frozen) ; asset with TransferDelegate(Address D) | TransferV1 (14) x2 | `InvalidAuthority`; success as D | lifecycle.rs:730-731, 739, 741 | JS freeze, delegateTransfer | S |
| M16 | `permanent_burn_force_approves_frozen` | asset with FreezeDelegate(frozen) + PermanentBurnDelegate | BurnV1 (12) as UA | success | lifecycle.rs:733, 164-173; validate_burn 377-382 | JS pluginValidationOverrides 'it can burn a frozen asset using PermanentBurnDelegate' | S |
| M17 | `collection_plugins_checked_on_member_transfer` | collection with PermanentFreezeDelegate(frozen), member asset | TransferV1 with collection | `InvalidAuthority` | lifecycle.rs:703; plugin_registry.rs:35-47 (collection key) | JS pluginValidationOverrides asset/collection override tests | M |
| M18 | `execute_with_freeze_execute_plugin` | asset with FreezeExecute(frozen) | ExecuteV1 (31) | `InvalidAuthority` | lifecycle.rs:204-211, 409-414 | JS freezeExecute.test.ts | S |
| M19 | `update_v2_move_into_collection_with_update_delegate` | asset (UA=A), new collection with UpdateDelegate(additional [A]) | UpdateV2 (30) new_collection | success | utils.rs:117-153, 244-261 | JS updateV2 'add asset to collection using additional update delegate on new collection' | M |
| M20 | `update_v2_move_into_collection_with_permanent_delegate` | new collection with PermanentFreezeDelegate | UpdateV2 | `PermanentDelegatesPreventMove` | utils.rs:244-261 | JS updateV2 'cannot add asset to collection if new collection contains permanent freeze delegate' | S |
| M21 | `agent_identity_transfer_burn_update_hooks` | asset via `build_asset_with_agent_identity` with `(Transfer,0x1)`, `(Burn,0x1)`, `(Update,0x1)` | TransferV1, BurnV1, UpdateV1 | all succeed | agent_identity.rs:106-125; external_plugin_adapters.rs:273-336 (AgentIdentity arms) | new | S |
| M22 | `create_v2_agent_identity_create_reject_flag` | PDA signer fixture | CreateV2 with `(Create,0x4)`; again without PDA signer | success; then the same signer/derivation failure the existing add-path negatives assert (`tests/agent_identity.rs:774-873`) | agent_identity.rs:76-85; external_plugin_adapters.rs:249-270 (266-268) | new | S |
| M23 | `update_external_adapter_wrong_authority` | asset with AgentIdentity(authority Address X), signer = UA | UpdateExternalPluginAdapterV1 (26) | `InvalidAuthority` | external_plugin_adapters.rs:384, 423-424 | JS oracle 'cannot update oracle using update authority when different from external plugin authority' | S |
| M24 | `add_oracle_and_deny_transfer` | oracle account = Borsh `OracleValidation::V1{transfer: Rejected,..}` | AddExternalPluginAdapterV1 (22) Oracle `(Transfer,0x4)`; TransferV1 with oracle account in remaining accounts | add ok; transfer `InvalidAuthority` | utils.rs:369-374, 877-887; oracle.rs:54-59, 77-123; lifecycle.rs:495-501, 805-807; external_plugin_adapters.rs:325 | JS oracle 'it can use fixed address oracle to deny transfer'; Rust test_add_oracle | L |
| M25 | `oracle_can_reject_only_and_empty_checks` | — | AddExternalPluginAdapterV1 Oracle with `0x2`; with `[]`; with duplicate events | `OracleCanRejectOnly`; `RequiresLifecycleCheck`; `DuplicateLifecycleChecks` | utils.rs:577, 586, 590-596 | JS oracle 'cannot add oracle to asset that can approve/listen', 'no lifecycle checks'; Rust duplicate test | S |
| M26 | `oracle_deny_update_via_collection_pda` | collection with Oracle `PreconfiguredCollection` `(Update,0x4)`, member asset, oracle PDA account | UpdateV1 with collection | `InvalidAuthority` | lifecycle.rs:777; external_plugin_adapters.rs:659-667; oracle.rs:84 | JS oracle 'preconfigured collection pda oracle to deny update', 'deny update via collection' | L |
| M27 | `oracle_account_errors` | Oracle registered, oracle account: absent / 3 bytes / zeroed / bad discriminant / `Anchor` offset | TransferV1 | `MissingExternalPluginAdapterAccount`, `InvalidOracleAccountData` x3, `UninitializedOracleAccount`; Anchor variant allows | oracle.rs:91, 97-105, 108, 184-190 | JS oracle 'does not exist', 'too small', 'empty account does not default to valid oracle' | M |
| M28 | `update_oracle_config_and_lifecycle_checks` | asset with Oracle + trailing AgentIdentity | UpdateExternalPluginAdapterV1 Oracle `{base_address_config: Some(Address), results_offset: Some(Anchor), lifecycle_checks: Some([(Burn,0x4)])}`; then with `0x1` | success, trailing record offsets bumped; then `OracleCanRejectOnly` | oracle.rs:29-36; plugin_registry.rs:95-111, 195-200; external_plugin_adapters.rs:160-164, 107-114 | JS oracle 'update oracle to larger registry record', 'cannot update oracle to approve'; 'shrink/grow a leading oracle' | M |
| M29 | `remove_oracle_from_asset_and_collection` | asset with Oracle + AgentIdentity; collection with Oracle | RemoveExternalPluginAdapterV1 (24) Oracle key; RemoveCollectionExternalPluginAdapterV1 (25) | success both; wrong key -> `ExternalPluginAdapterNotFound` | utils.rs:224, 684-764 (data_len 0), 848-851, 923; CollectionV1 monomorph | Rust remove_external_plugins.rs::test_remove_oracle, ..._on_collection | S |
| M30 | `app_data_add_write_grow_shrink_remove` | asset with AppData(data_authority Address D) + trailing FreezeDelegate | AddExternalPluginAdapterV1 AppData; WriteExternalPluginAdapterDataV1 (28) 100 bytes; write 10 bytes; write via `buffer` account; RemoveExternalPluginAdapterV1 | data written; trailing plugin intact after each; remove shrinks by header+data | utils.rs:377-385, 414-417, 439, 478-570 (both branches), 890-901, 711; app_data.rs:35-40, 51-56; plugin_registry.rs:84-90, 209 | Rust plugin_shrink_corruption.rs (both write tests), update_external_plugins.rs::test_update_app_data, remove_external_plugins.rs::test_remove_app_data; JS appData offset tests | L |
| M31 | `app_data_write_wrong_data_authority` | AppData(data_authority Address D), signer UA | WriteExternalPluginAdapterDataV1 | `InvalidAuthority`; `TwoDataSources`/`NoDataSources` variants | processor only; utils.rs untouched | JS appData 'cannot update app data using update authority when different from external plugin authority' | S |
| M32 | `collection_app_data_write` | collection with AppData | WriteCollectionExternalPluginAdapterDataV1 (29) | success | utils.rs `update_external_plugin_adapter_data::<CollectionV1>` | JS appData 'it can update app data on collection ...' | S |
| M33 | `linked_app_data_creates_data_section` | collection with LinkedAppData(data_authority D), member asset | AddCollectionExternalPluginAdapterV1 LinkedAppData; WriteExternalPluginAdapterDataV1 with `LinkedAppData` key + collection; second write (grow); RemoveExternalPluginAdapterV1 `DataSection` key on the asset | DataSection created with data (`Authority::None`); second write updates; removal ok | utils.rs:385, 462-468, 904-914; data_section.rs:18-23; linked_app_data.rs:46-51; external_plugin_adapters.rs:903-907, 932-933 | JS linkedAppData.test.ts, linkedAppDataMembership 'write succeeds for a legitimate collection member', 'Data offsets are correctly bumped when removing Data Section with data' | L |
| M34 | `linked_app_data_non_member_rejected` | collection with LinkedAppData, asset with UA=Address | WriteExternalPluginAdapterDataV1 LinkedAppData key | `InvalidCollection` | processor (security fix path) | JS linkedAppDataMembership 'rejected when the asset is not a member' | S |
| M35 | `update_linked_app_data_schema_on_collection` | collection with LinkedAppData | UpdateCollectionExternalPluginAdapterV1 (27) `{schema: Some(Json)}` | success | linked_app_data.rs:24-28; external_plugin_adapters.rs:178-182; utils.rs CollectionV1 monomorph | JS linkedAppData 'can update linked app data on collection ...' | S |
| M36 | `lifecycle_hooks_not_available` | bare asset; bare collection | AddExternalPluginAdapterV1 LifecycleHook; AddCollectionExternalPluginAdapterV1 LinkedLifecycleHook; CreateV2 with LifecycleHook | `NotAvailable` x3 | utils.rs:340; lifecycle_hook.rs:58-65, linked_lifecycle_hook.rs:58-65; external_plugin_adapters.rs:85-87,90-92,526-528,535-537,917-919,926-928 | Rust add_external_plugins.rs::test_temporarily_cannot_add_lifecycle_hook(_on_collection), create_with_external_plugins.rs | S |
| M37 | `preexisting_lifecycle_hook_state` | hand-built asset with a `LifecycleHook` record `(Transfer,0x1)` and a LinkedLifecycleHook record on a collection | TransferV1; UpdateExternalPluginAdapterV1 LifecycleHook key `{schema: Some(Json)}` | transfer succeeds (abstain); update succeeds | lifecycle_hook.rs:31-38, 49-54; linked_lifecycle_hook.rs:49-54; plugin_registry.rs:188-193; external_plugin_adapters.rs:154-158, 322-324, 877-885, 69-72 | new (documents that the program will read blocked adapters) | M |
| M38 | `add_external_adapter_with_internal_plugins_present` | asset with Attributes + FreezeDelegate | AddExternalPluginAdapterV1 AgentIdentity (existing fixture) | success | lifecycle.rs:214-236, 417-430, 562-567 defaults | new | S |
| M39 | `corrupted_registry_offset_deserialization_error` | hand-built asset whose FreezeDelegate record offset points into the registry bytes | TransferV1 | `DeserializationError` (or documented panic for out-of-range offset) | mod.rs:79-85; external_plugin_adapters.rs:506-512 | new (security fork) | S |
| M40 | `create_v2_invalid_adapter_targets` | — | CreateV2 with LinkedAppData; with DataSection; CreateCollectionV2 with AgentIdentity | `InvalidPluginAdapterTarget`, `CannotAddDataSection`, `InvalidPluginAdapterTarget` | external_plugin_adapters.rs:232-233, 221-231 (`check_create` arms) | JS dataSection.test.ts, linkedAppData 'cannot create an asset with linked app data' | S |
| M41 | `oracle_execute_event_is_noop` | asset with Oracle `(Execute,0x4)` and oracle account | ExecuteV1 as owner | succeeds (documents finding 3) | external_plugin_adapters.rs:492; oracle.rs default | new | S |

### Harness prerequisites (tests/common)

1. **Generic account builder** replacing the per-file copies in `tests/account_ownership.rs:130-275`
   and `tests/agent_identity.rs:143-225`: `build_core_account<T: DataBlob + SolanaAccount>(core: T,
   internal: &[(Plugin, Authority)], external: &[ExternalFixture])` where `ExternalFixture` carries the
   `ExternalPluginAdapter`, record `authority`, `lifecycle_checks`, and optional appended data
   (`data_offset`/`data_len` computed). Use `PluginType::from(&plugin)` instead of the hard-coded
   4-arm match. Must produce byte-identical layout to `initialize_plugin` /
   `initialize_external_plugin_adapter` (header at `core.len()`, registry last).
2. **Instruction builders with hand-Borsh args** for discriminators 0-11, 14-15, 20-31. The args
   structs and `PluginAuthorityPair` are `pub(crate)`, so tests serialize `(Plugin, Option<Authority>)`
   and `(ExternalPluginAdapterKey, ExternalPluginAdapterUpdateInfo)` manually, as
   `tests/agent_identity.rs:227-276` already does for CreateV2. Optional-account slots use the
   program id sentinel (`core_program_account()` in `tests/common/mod.rs:56`).
3. **Post-state parser/invariant checker**: `parse_account(&Account) -> (core, header, registry,
   Vec<Plugin>, Vec<(ExternalPluginAdapter, Option<Vec<u8>>)>)` plus `assert_registry_consistent`
   (every record offset deserializes to a plugin of the recorded type, records are contiguous,
   `plugin_registry_offset == data_len - registry.len()`). Needed for every grow/shrink/remove test.
4. **Collection + member asset fixture**: `CollectionV1` with plugins/adapters and an `AssetV1` whose
   `update_authority = UpdateAuthority::Collection(collection)`; instruction builders that populate the
   optional `collection` slot.
5. **Oracle account fixture**: `oracle_account(validation: OracleValidation, offset: ValidationResultsOffset)
   -> Account` (any owner, rent-exempt lamports), plus PDA derivation helpers for the `Preconfigured*`
   variants (`Pubkey::find_program_address(&[b"mpl-core", key.as_ref()], &base_address)`).
6. **Buffer account fixture** for the `buffer` slot of Write*ExternalPluginAdapterDataV1.
7. **AgentIdentity fixtures** already exist (`agent_identity_pda`, `build_execution_delegate_record`);
   move them into `tests/common` so the new files can reuse them.
8. **Unit-test scaffolding in-crate** (`src/plugins/test_utils.rs` under `#[cfg(test)]`): a
   `fake_account_info(key, owner, data: &mut [u8], lamports)` helper and a `default_ctx()` producing a
   `PluginValidationContext` with all-`None` fields, for U4/U5/U8/U9.

### Effort and ordering (coverage gained per unit of effort)

Roughly 41 Mollusk tests + 11 in-crate unit tests. Suggested order:

1. **Prerequisite 1-3** (builders + parser), then **M1, M3, M5, M8, M10, M13** — six small tests that
   turn on `create_plugin_meta`, `initialize_plugin`, `fetch_wrapped_plugin`, `delete_plugin`,
   `approve/revoke_authority_on_plugin`, `bump_offsets` internal path, `Plugin::save`, and most of
   `PluginType::check_*` and `Plugin::validate_*` routers: ~250 lines in utils.rs/lifecycle.rs/mod.rs.
2. **U2, U3, U6, U7, U10** — trivial table/enumeration unit tests, ~150 lines across lifecycle.rs,
   mod.rs, external_plugin_adapters.rs, plugin_registry.rs and the small external files.
3. **M15-M18, M4, M6, M9, M11** — lifecycle result branches (Rejected/Approved/ForceApproved,
   `CannotRedelegate`, authority-None) — ~60 lines but these are the security-relevant paths.
4. **M30, M33** (AppData/DataSection) — unlock `update_external_plugin_adapter_data`, appended-data
   init and the authority/DataSection key branches: ~130 lines in utils.rs.
5. **Prerequisite 5 + M24, M25, M27, M28, M29** — Oracle: all of oracle.rs (92) plus
   `ExtraAccount::derive` fixed-address path, `check_plugin_key` pubkey branch, external reject path.
6. **U8, U9, M26** — remaining `ExtraAccount`/`transform_seeds` arms and collection-keyed external checks.
7. **M2, M7, M14, M32, M35** — `CollectionV1` monomorphizations (function-level coverage).
8. **M19-M23, M38, M40, M41** — UpdateV2 collection moves, AgentIdentity event hooks, dead-path documentation.
9. **U1, U4, U5, U11, M36, M37, M39** — matrices, bitfield setters, blocked-hook and crafted-state tests.
   These close the last ~80 lines; several are only reachable this way (class A/D).


## 12. Internal plugin validations

Scope: `src/plugins/internal/{authority_managed,owner_managed,permanent}/*.rs` except `groups.rs`.
All paths below are relative to `programs/mpl-core`. Coverage figures are from `uncovered/INDEX.txt`
(they include the `#[cfg(test)]` module lines, which is why the "len"-only test modules already
give 30-50% on tiny files).

### How the validate_* callbacks are reached (read this first)

Every `PluginValidation` method is invoked through `Plugin::validate_*` in `src/plugins/lifecycle.rs`
(lines 241-431) and only for plugin types whose `PluginType::check_*` table entry is not `None`
(`src/plugins/lifecycle.rs:83-236`). The `PluginValidationContext` the callback sees is built in
exactly three places:

| Call site | Lifecycle | `resolved_authorities` | `target_plugin` | `asset_info` / `collection_info` |
|---|---|---|---|---|
| `src/processor/create.rs:203`, `create_collection.rs:165` | `validate_create`, only for `check_create != None` (Royalties, UpdateDelegate, Autograph, VerifiedCreators) | `None` | `None` | asset: `Some/ctx.collection`; collection: `None/Some`. NB `create_collection.rs:171` passes **payer** as `authority_info` |
| `src/processor/add_plugin.rs:60`, `:148` ("self-validation": the *new* plugin validates itself) | `validate_add_plugin` | `None` | `Some(new plugin)` | asset path `Some/…`; collection path `None/Some` |
| `src/plugins/lifecycle.rs:708` `validate_plugin_checks` (via `utils::validate_asset_permissions` / `validate_collection_permissions`) | every other lifecycle: add/remove/update plugin, approve/revoke authority, update, burn, transfer, execute, add external adapter | `Some([Owner?, UpdateAuthority?, Address{signer}])` (`src/utils/mod.rs:476-507`) | add: new plugin; remove/approve/revoke: the stored plugin; update_plugin: `args.plugin` (new data); else `None` | `Some(asset)/collection` or `None/Some(collection)` |

Important mechanics for reachability:
- `checks` is a `BTreeMap<PluginType, …>` filled from the collection registry first, then the asset registry
  (`src/utils/mod.rs:198-219`, `plugin_registry.rs:35-47`) — an asset-level plugin **shadows** the same
  plugin type on the collection. A collection-level plugin is only evaluated for an asset lifecycle when the
  asset does not carry that type.
- `self_authority` is the *registry record authority* of the plugin being evaluated (default = `manager()`,
  or whatever it was delegated to). "Plugin authority signs" below means
  `resolved_authorities.contains(self_authority)`.
- The wrappers in `lifecycle.rs` short-circuit before the plugin code: remove/revoke with
  `self_authority == Authority::None` (`:257`, `:292`) → `Rejected`; approve when the target's authority ≠
  its manager (`:272`) → `CannotRedelegate`. Several JS tests that look like they hit a plugin branch
  actually stop there (called out per plugin).
- `check_remove_plugin` defaults to `CanReject` for **every** plugin type (`lifecycle.rs:111`), and
  `check_update_plugin`/`check_approve_plugin_authority`/`check_revoke_plugin_authority` default to
  `CanApprove`. So on those four instructions every plugin present on the asset (or inherited from the
  collection) has its callback executed — the "other target → abstain" arms are reached simply by having the
  plugin present while touching a different plugin.
- `create*` ignores `Approved` from `validate_create` (only `Rejected`/`ForceApproved` are consumed,
  `create.rs:220-222`).

Two strategies:
- **Unit tests in the module** (`#[cfg(test)]`): all callbacks are pure over `PluginValidationContext`
  (a `pub(crate)` struct with `pub` fields, `lifecycle.rs:506-538`) except `UpdateDelegate::validate_update`,
  which reads the asset account. A small shared `#[cfg(test)] pub(crate) mod test_ctx` in `src/plugins/`
  that builds an `AccountInfo` from local buffers and a `PluginValidationContext` with all-`None` defaults
  makes each branch a 5-10 line test. This is the cheapest route to 100% on these files and is exact about
  the branch being tested.
- **Mollusk instruction tests** (`tests/`): needed to prove the *wiring* (check tables, shadowing, wrapper
  short-circuits, error code surfaced) and they also lift the 0% processors (`add_plugin`, `remove_plugin`,
  `update_plugin`, `approve/revoke_plugin_authority`, `update`, `burn`). The existing
  `tests/account_ownership.rs:130-275` builders only serialize four plugin types and panic otherwise.

Note on the per-file "uncovered functions" lists: each function appears twice (crate hashes
`Cs30NUS1hy9Hm_` = unit-test build, `Cs9dbz0oj2f79_` = integration build). `DataBlob::len`, `new`,
`Default::default` show up only under the integration hash because the unit tests already execute them; they
need no action. Only the merged line ranges are authoritative.

---

### src/plugins/internal/authority_managed/attributes.rs — 42/42 lines (100%)
### src/plugins/internal/authority_managed/master_edition.rs — 19/19 lines (100%)
No validation logic (`impl PluginValidation for … {}`); nothing to do. `MasterEdition` cannot be added to
assets at all (`add_plugin.rs:52`, `create.rs:194`) — that rejection lives in the processors.

### src/plugins/internal/authority_managed/add_blocker.rs — 8/20 lines (40%)

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_add_plugin`: target is owner-managed → `abstain` | 30-35 | AddPluginV1 of FreezeDelegate/TransferDelegate/BurnDelegate/Autograph/FreezeExecute on an asset (or asset-in-collection) that has AddBlocker | JS `plugins/asset/addBlocker.test.ts` "it can add owner-managed plugins even if AddBlocker had been added" | unit + 1 Mollusk |
| target is `AddBlocker` itself → `abstain` | 32-35 | The self-validation in `add_plugin.rs:60-76` when AddBlocker is added post-creation | JS asset "it can add plugins unless AddBlocker is added"; collection "it can add addBlocker to collection" | Mollusk (same test as row 3) |
| any other UA-managed target → `reject` → `InvalidAuthority` | 38 | AddPluginV1 / AddCollectionPluginV1 of e.g. Attributes when AddBlocker exists on the asset **or** on its collection (Key::CollectionV1 check) | JS asset "it cannot add UA-managed plugin if addBlocker had been added on creation"; collection "…cannot add UA-managed plugin to an asset in a collection if addBlocker…", "…to a collection…" ×2 | unit + Mollusk |
| `target_plugin == None` → `reject` | 30, 38 | unreachable — every add path passes `Some` | — | unit only (for line coverage) |

### src/plugins/internal/authority_managed/immutable_metadata.rs — 8/14 lines (57%)

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_update` → unconditional `reject` → `InvalidAuthority` | 25-30 | UpdateV1/V2 on an asset that has ImmutableMetadata or whose collection has it; UpdateCollectionV1 on a collection with it. `check_update` = CanReject so it is evaluated even when the plugin authority is `None` (the normal "immutable" configuration) | JS `plugins/asset/immutableMetadata.test.ts` "it can prevent the asset from metadata updating"; `plugins/collection/immutableMetadata.test.ts` "it can prevent collection assets metadata from being updated", "it prevents both collection and asset…" | Mollusk (UpdateV1 is 0% — one test pays twice); trivial unit test too |

Note: the reject also blocks `new_update_authority` changes (any UpdateV1 args), not only name/uri.

### src/plugins/internal/authority_managed/royalties.rs — 87/152 lines (57%)

`validate_royalties` (79-105) is a pure function: unit test all four outcomes. The three callbacks that
call it are reached from create, add-plugin self-validation and update-plugin.

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_royalties`: `basis_points > 10000` → `InvalidPluginSetting` | 80-83 | Royalties with bp 10001 on CreateV1, AddPluginV1 (self-validation), UpdatePluginV1 by plugin authority | JS `plugins/asset/royalties.test.ts` "cannot create/add/update royalty basis points greater than 10000" | unit (pure) + 1 Mollusk per instruction |
| creator percentages don't sum to 100 → `InvalidPluginSetting` | 84-92 | same, percentages e.g. 50+40 | JS "…percentages that dont add up to 100" ×3 | unit |
| duplicate creator address → `InvalidPluginSetting` | 94-102 | same, two `Creator`s with the same pubkey | JS "…duplicate creators" ×3 | unit |
| valid → `abstain` | 104 | any valid Royalties | JS "it can transfer an asset with royalties"; Rust `clients/rust/tests/create_collection.rs::create_collection_with_plugins` (Royalties on collection) | unit + Mollusk create |
| `validate_create` | 108-113 | CreateV1/V2 or CreateCollectionV1/V2 with Royalties in `plugins` | Rust `create_collection_with_plugins`, `…_with_different_plugin_authority`; JS create tests | Mollusk port |
| `validate_transfer`: `new_owner == None` → `MissingNewOwner` | 119 | unreachable — `transfer.rs:84` always passes `Some(new_owner)` | — | unit only |
| `RuleSet::None` → `abstain` | 121 | TransferV1 with rule set None | JS "it can transfer an asset with royalties" (asset) / "…with collection royalties" (Key::CollectionV1 path) | Mollusk |
| `ProgramAllowList` both `authority_info.owner` and `new_owner.owner` listed → `abstain` | 122-126 | TransferV1 where the *program that owns* the signer account and the new-owner account are in the list (wallets: system program) | JS "…to an allowlisted program address" ×2 | Mollusk (set `Account.owner` of new_owner to the listed program) |
| allow list miss → `reject` → `InvalidAuthority` | 128 | new_owner owned by a program not in the list | JS "cannot transfer … not on the allowlist" ×2 | Mollusk |
| `ProgramDenyList` hit → `reject` | 131-135 | new_owner owned by a listed program | JS "cannot transfer … denylisted program" ×2 | Mollusk |
| deny list miss → `abstain` | 137 | e.g. `ProgramDenyList(vec![])` | Rust `clients/rust/tests/transfer.rs::transfer_asset_with_royalties` (DenyList([])) ; JS "…not on the denylist" ×2 | Mollusk port |
| `validate_add_plugin`: target Royalties → `validate_royalties(self)` | 147-148 | self-validation in `add_plugin.rs:60-76` (new plugin, `self` == target) | JS "cannot add royalty …" ×3 and any successful `addPlugin Royalties` | Mollusk |
| target other plugin → `abstain` | 149 | AddPluginV1 of any other plugin on an asset/collection that already has Royalties | no dedicated test found | Mollusk (new, S) |
| `validate_update_plugin`: `target None` / `resolved None` errors | 157-160 | unreachable from processors | — | unit only |
| target Royalties, signer is plugin authority → `validate_royalties(new)` | 163-165 | UpdatePluginV1 Royalties by UA | JS "cannot update royalty …" ×3 | Mollusk |
| target Royalties, signer not plugin authority → `abstain` (then `NoApprovals`) | 167 | UpdatePluginV1 Royalties signed by the owner | none found | Mollusk (new, S) |
| target other plugin → `abstain` | 170 | UpdatePluginV1 of e.g. Attributes while Royalties present | none dedicated | Mollusk (new, S; combine with row above) |

Observations:
- `validate_add_plugin` (148) validates `self` (the already-stored Royalties) rather than `ctx.target_plugin`
  when an existing Royalties plugin evaluates a *new* Royalties. Harmless today because the new plugin was
  already checked by the self-validation call and `initialize_plugin` then fails with `PluginAlreadyExists`
  (`src/plugins/utils.rs:284`), but the intent was clearly to validate the target.
- Allow/deny lists key on `AccountInfo::owner` of the signer and of the new owner, i.e. the owning
  *program* — test fixtures must set `Account.owner`, not the pubkey.

### src/plugins/internal/authority_managed/update_delegate.rs — 28/155 lines (18%)

The largest gap in scope. All seven callbacks are uncovered. "PA" below = the UpdateDelegate plugin's own
registry authority signs (default `UpdateAuthority`, i.e. the asset/collection UA, or the delegated
address); "AD" = signer ∈ `additional_delegates`.

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_create`: `resolved_authorities` branch | 53-57 | **dead**: both create call sites pass `resolved_authorities: None` | — | unit only |
| AD → `approve` | 59-61 | CreateV1 with UpdateDelegate whose `additional_delegates` contains the signer | JS `plugins/asset/updateDelegate.test.ts` "it can create an asset with updateDelegate with additional delegates" (signer is UA, so actually hits `abstain`) | unit |
| `abstain` | 63 | any other create with UpdateDelegate | JS "it can create an asset with updateDelegate" | Mollusk create |
| `validate_add_plugin`: (PA ∨ AD) ∧ new plugin UA-managed → `approve` | 70-79 | AddPluginV1 Attributes by delegate / additional delegate; also collection UD applied to asset add | JS "an updateDelegate can add a plugin to an asset", "…using delegated owner"; `addPlugin.test.ts` "it can add an authority-managed plugin to an asset via delegate authority" | Mollusk port |
| else → `abstain` (→ `NoApprovals`) | 81 | delegate adds an owner-managed plugin | JS `addPlugin.test.ts` "it cannot add a owner-managed plugin to an asset via delegate authority"; `updateDelegate.test.ts` "it cannot add updateDelegate plugin with additional delegate as additional delegate" | Mollusk port |
| `target None` → `InvalidPlugin` | 84 | unreachable | — | unit only |
| `validate_remove_plugin`: approve / abstain / err | 92-107 | RemovePluginV1 by delegate of UA-managed plugin (approve); of owner-managed plugin (abstain) | JS "an updateDelegate can remove a plugin from an asset", "…using delegated owner"; `removePlugin.test.ts` "it can remove authority managed plugin from collection using delegate auth", "it cannot remove owner managed plugin if the delegate authority is not owner" | Mollusk port |
| `validate_approve_plugin_authority`: (PA ∨ AD) ∧ UA-managed ∧ target ≠ UpdateDelegate → `Approved` | 118-131 | ApprovePluginAuthorityV1 on Attributes/Royalties by delegate | JS "it can approve/revoke the plugin authority of other plugins", "…as delegated owner", "…of non-updateDelegate plugins as additional delegate" | Mollusk port |
| else → `Pass` | 133 | AD tries to approve authority on the UpdateDelegate plugin itself | JS "it cannot approve the update delegate plugin authority as additional delegate" | Mollusk port |
| `validate_revoke_plugin_authority` (security-fixed precedence) → approve | 149-159 | RevokePluginAuthorityV1 on a UA-managed plugin by PA; or by AD when target ≠ UpdateDelegate | JS `updateDelegateRevokeBug.test.ts` "should allow update authority to revoke authority on UpdateAuthority-managed plugins via UpdateDelegate" | Mollusk port |
| → abstain | 161 | target owner-managed (FreezeDelegate/TransferDelegate) by PA or AD; or AD targeting UpdateDelegate | JS `updateDelegateRevokeBug.test.ts` "should NOT allow … owner-managed plugins via UpdateDelegate", "…TransferDelegate…", "…delegated update delegate…"; `updateDelegate.test.ts` "it cannot revoke the update delegate plugin authority as additional delegate" | Mollusk port (these are the regression tests for the fix — highest priority) |
| `validate_update`: (PA ∨ AD) ∧ asset ∧ collection ∧ `new_asset_authority ≠ Collection(current)` ∧ UpdateDelegate stored on the **asset** → `reject` | 182-198 | UpdateV2 by the asset-level update delegate/AD moving the asset out of its collection | JS `updateV2.test.ts` "it cannot remove an asset from collection using update delegate on the asset", "…using additional update delegate on the asset" | Mollusk (needs asset-in-collection fixture) |
| → `approve` | 201 | UpdateV1 by delegate/AD (name/uri change); collection-level delegate removing asset from collection (`fetch_wrapped_plugin` on the asset errs → falls to approve); UpdateCollectionV1 by AD | JS "an updateDelegate can update an asset", "…additionalDelegate can update an asset", `updateV2.test.ts` "it can remove an asset from collection using update delegate", "…additional update delegate"; collection "it can update collection details as an updateDelegate additional delegate" | Mollusk port |
| → `abstain` | 204 | signer is neither (e.g. delegate after revoke) | JS "an updateDelegate cannot update an asset after delegate authority revoked" | Mollusk port |
| `validate_update_plugin`: target UpdateDelegate and diff == {remove self} → `approve` | 222-229 | UpdatePluginV1 UpdateDelegate by an AD that only removes its own key | JS "it can remove additional delegate as additional delegate if self" | unit (set logic) + Mollusk |
| target UpdateDelegate, any other diff → fall through `abstain` | 222-231, 239 | AD adds a key / removes someone else; PA changing the list (PA is approved by the wrapper base result instead, `lifecycle.rs:348`) | JS "it cannot add additional delegate as additional delegate", "it cannot remove another additional delegate as additional delegate", "it can update updateDelegate on asset with additional delegates" | unit + Mollusk |
| other target with authority `UpdateAuthority` → `approve` | 233-234 | UpdatePluginV1 Attributes (authority UA) by AD; owner-managed plugin whose authority was delegated to UA | JS "it can update a non-updateDelegate plugin as additional delegate", "it can update an authority-managed plugin … as additional delegate", "it can update an owner-managed plugin … if the plugin authority is UpdateAuthority" (+ collection variants) | Mollusk port |
| other target with other authority → `abstain` | 239 | UpdatePluginV1 on a plugin delegated to Address{X} by AD | JS "…cannot update an authority-managed plugin … if the plugin authority is not UpdateAuthority", "…cannot update an owner-managed plugin … as collection update additional delegate" | Mollusk port |
| `target None` → `InvalidPlugin` | 212 | unreachable | — | unit only |

Observations:
- `validate_create` is inert: its only possible effect is `Approved`, which the create processors discard.
  The `resolved_authorities` arm can never execute. Candidate for removal or for making `additional_delegates`
  membership meaningful at create time.
- Shadowing (see mechanics): when an asset carries its own UpdateDelegate, the collection's UpdateDelegate is
  never consulted for that asset — a collection delegate/AD cannot update, add/remove plugins on, or
  re-parent such an asset (only the collection's real UA can, via `CollectionV1::validate_update`). JS
  `updateV2.test.ts:1051` documents the UA case only.
- `validate_update` decides "asset-level vs collection-level" by re-reading the asset account
  (`fetch_wrapped_plugin`, 190-195) instead of using `ctx.self_key`; `self_key` would be exact and cheaper.

### src/plugins/internal/authority_managed/verified_creators.rs — 35/160 lines (22%)

Three pure helpers carry all the logic; unit-test them exhaustively, then port ~5 JS cases to Mollusk for
the callback wiring. "PA" = plugin authority (UA by default) signs; "creator" = any other signer.

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `calculate_signature_changes`: duplicate address → `InvalidPluginSetting` | 77-81 | new list has the same pubkey twice | JS `plugins/asset/verifiedCreators.test.ts` "it cannot add duplicate verified creator signatures" | unit |
| added / changed / removed index computation | 83-110 | any diff; `verified_creators == None` on create/add | (all JS tests) | unit |
| `…_as_creator`: added or removed entries → `MissingSigner` | 121-125 | UpdatePluginV1 by non-PA that adds/removes an address | JS "it cannot remove verified creator plugin signture with unauthorized signature" | unit + Mollusk |
| changed entry not the signer → `MissingSigner` | 127-133 | non-PA flips someone else's `verified` | JS "it cannot verify a verified creator plugin with unauthorized signature" | unit + Mollusk |
| only own flag changed → `abstain` | 136 | creator verifies/unverifies self | JS "…unverified signatures and then verify", "it can unverify signature verified creator plugin" | unit + Mollusk |
| `…_as_plugin_authority`: removal of a verified entry that isn't the signer → `InvalidPluginOperation` | 150-156 | PA removes another verified creator | JS "it cannot remove verified creator plugin signature with update auth" | unit + Mollusk |
| flag change on another address → `InvalidPluginOperation` | 158-164 | PA unverifies someone else | JS "it cannot unverify verified creator plugin signature with update auth" | unit |
| added entry verified but not the signer → `MissingSigner` | 166-172 | create/add/update with `verified: true` for a third party | JS "it cannot create asset with verified creators plugin and unauthorized signature" | unit + Mollusk create |
| → `abstain` | 174 | PA adds/removes unverified entries, adds self verified | JS "it can create asset with verified creators plugin", "…with authorized signature", "it can remove and add unverified creator plugin signature with update auth" | unit + Mollusk |
| `validate_create` | 178-183 | CreateV1 with VerifiedCreators | JS create tests above | Mollusk |
| `validate_add_plugin`: target VerifiedCreators → validate as PA (existing == None) | 189-192 | self-validation on AddPluginV1 | JS "it cannot add verified creator plugin to asset by owner" (owner signs; rejected by `AssetV1::validate_add_plugin` unless a verified third party is present, in which case `MissingSigner` here first) | Mollusk |
| other target → `abstain` | 193 | adding any other plugin while VerifiedCreators is present | none | Mollusk (new, S) |
| `validate_update_plugin`: resolved None → `InvalidAuthority` | 201-203 | unreachable | — | unit only |
| target VerifiedCreators, PA → validate as PA → `Approved` | 205-212 | UpdatePluginV1 by UA | JS PA cases above | Mollusk |
| target VerifiedCreators, non-PA → validate as creator → `Approved` | 214-219 | UpdatePluginV1 by a listed creator | JS creator cases above | Mollusk |
| other target → `abstain` | 222 | UpdatePluginV1 of another plugin | none | Mollusk (new, S) |

Observations:
- The non-PA arm (214-219) returns `Approved` for *any* signer whose submitted list equals the stored one
  (empty diff), so an unrelated wallet can execute a no-op `UpdatePluginV1{VerifiedCreators}` on someone
  else's asset (state unchanged, seq bumped). Low impact, but it is an unauthenticated write path.
- `create_collection.rs:171` validates `validate_create` against the **payer** key; for a collection created
  with a separate update authority, a `verified: true` entry for the UA is rejected with `MissingSigner`
  unless the payer is that key.

### src/plugins/internal/owner_managed/autograph.rs — 35/111 lines (32%)

`validate_autograph` (45-92) is pure; `is_plugin_authority` is true for create/add and for update when
the plugin authority (owner by default, or delegate) signs.

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| existing signature with changed `message` → `InvalidPluginOperation` | 55-60 | UpdatePluginV1 editing a message (by anyone) | JS `plugins/asset/autograph.test.ts` "it cannot modify autograph message as signer", "…as owner" | unit + Mollusk |
| new signature whose address ≠ signer → `MissingSigner` | 62-66 | create/add/update adding someone else's signature | JS "it cannot create asset with autograph plugin and unauthorized signature", "it cannot add autograph to asset by unauthorized 3rd party" | unit + Mollusk |
| duplicate addresses → `InvalidPluginSetting` | 71-80 | list with same address twice | JS "it cannot add duplicate autographs" | unit |
| non-authority removed an existing signature → `MissingSigner` | 82-88 | UpdatePluginV1 by a 3rd party dropping an entry | JS "it cannot remove autograph if not owner" | unit + Mollusk |
| → `abstain` | 91 | valid change | JS "it can add additional autograph to asset via update by 3rd party", "it can remove autograph if owner", "it can remove and add autographs as owner/as delegate" | unit + Mollusk |
| `validate_create` | 102-107 | CreateV1 with Autograph | JS "it can create asset with autograph plugin", "…with authorized signature" | Mollusk |
| `validate_add_plugin`: target Autograph → validate, then `approve` | 114-116 | self-validation on AddPluginV1 Autograph by owner | JS "it can add autograph plugin to asset by owner", "it cannot add autograph plugin to asset by creator" (UA; rejected later by `AssetV1::validate_add_plugin`), "…by 3rd party" | Mollusk |
| other target → `abstain` | 118 | AddPluginV1 of another plugin with Autograph present | none | Mollusk (new, S) |
| `validate_update_plugin`: target Autograph → validate → `approve` | 130-137 | UpdatePluginV1 Autograph by owner / delegate / 3rd party | JS cases above | Mollusk |
| other target → `abstain`; resolved None → err | 139; 126-128 | update of another plugin; err unreachable | none | Mollusk (new, S) / unit |

Observation: `validate_add_plugin`/`validate_update_plugin` return `Approved` (not `Pass`) for
non-authority signers whose change is self-only — by design ("3rd party can autograph"), but it means
`validate_plugin_checks` treats a `CanReject`-registered plugin's `Approved` as an approval
(`lifecycle.rs:731`); nothing filters approvals by check kind.

### src/plugins/internal/owner_managed/burn_delegate.rs — 14/26 lines (54%)

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_burn`: plugin authority signs → `approve` | 36-42 | BurnV1 by the delegate (authority `Address{d}`), or by UA when authority is `UpdateAuthority` (asset or collection UA) | JS `plugins/asset/burnDelegate.test.ts` "a burnDelegate can burn an asset", "…using delegated update authority", "…from collection" | Mollusk (also lifts `processor/burn.rs` from 0%) |
| else → `abstain` (→ `NoApprovals` for a non-owner) | 44 | BurnV1 by revoked delegate / random signer with BurnDelegate present | JS "an burnDelegate cannot burn an asset after delegate authority revoked" | Mollusk |

### src/plugins/internal/owner_managed/freeze_delegate.rs — 21/66 lines (32%)

`validate_transfer` (54-58) is already covered by `tests/account_ownership.rs::transfer_rejects_valid_frozen_asset`
and `transfer_succeeds_valid_unfrozen_asset_with_freeze_plugin`; the flagged lines 50/52/59 are
signature/brace lines.

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_burn`: frozen → `reject`; else `abstain` | 43-48 | BurnV1 with FreezeDelegate{frozen} (→ `InvalidAuthority`) / {unfrozen} | JS `burn.test.ts` "it cannot burn an asset if it is frozen" | Mollusk (mirror of the existing transfer pair) |
| `validate_approve_plugin_authority`: target FreezeDelegate frozen → `reject` | 65-68 | ApprovePluginAuthorityV1 on FreezeDelegate while frozen **and** its authority is still `Owner` (otherwise the wrapper returns `CannotRedelegate` first, `lifecycle.rs:272`) — i.e. owner froze it themselves, then tries to delegate | none — JS `freeze.test.ts` "owner cannot approve to reassign authority back to owner if frozen" expects `CannotRedelegate`, so it never reaches this line | Mollusk (new, S) + unit |
| other target / unfrozen → `abstain` | 70 | ApprovePluginAuthorityV1 on another plugin while FreezeDelegate present; or unfrozen | JS "it can delegate then freeze an asset" | Mollusk |
| `validate_revoke_plugin_authority`: frozen → `reject` | 77-79 | RevokePluginAuthorityV1 by owner while delegate-frozen | JS `freeze.test.ts` "owner cannot undelegate a freeze plugin with a delegate" (`InvalidAuthority`) | Mollusk port |
| unfrozen and plugin authority signs → `approve` | 80-86 | the delegate revokes itself | JS `revokeAuthority.test.ts` "it can remove a pubkey authority from an owner-managed plugin if that pubkey is the signer authority", "…an update authority from an owner-managed plugin…" (both use FreezeDelegate) | Mollusk port |
| unfrozen, owner revokes → `abstain` (owner approved by `AssetV1`) | 90 | owner revokes unfrozen delegate | JS "it delegate cannot freeze after delegate has been revoked" | Mollusk port |
| `validate_remove_plugin`: frozen → `reject` (for **any** target) | 97-98 | RemovePluginV1 of FreezeDelegate or of any other plugin while frozen | JS `removePlugin.test.ts` "it cannot remove a plugin from a frozen asset" (removes TransferDelegate), `freeze.test.ts` "it cannot remove freeze plugin if update authority and frozen" | Mollusk port |
| unfrozen → `abstain` | 100 | RemovePluginV1 with unfrozen FreezeDelegate | JS `removePlugin.test.ts` "it can remove a plugin from asset with existing plugins" | Mollusk |

### src/plugins/internal/owner_managed/freeze_execute.rs — 14/61 lines (23%)

Same shape as FreezeDelegate but on the Execute lifecycle, and `validate_remove_plugin` only blocks removal
of **itself** (85-88), unlike FreezeDelegate/PermanentFreezeDelegate which block every removal.

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_execute`: frozen → `reject` / `abstain` | 45-50 | ExecuteV1 by owner with FreezeExecute{frozen} → `InvalidAuthority`; unfrozen → succeeds | JS `plugins/asset/freezeExecute.test.ts` "it covers the freeze execute backed NFT flow"; Rust `clients/rust/tests/freeze_execute.rs::test_freeze_execute_backed_nft_flow` | Mollusk (reuse `tests/execution_delegate.rs::execute_v1_instruction`) |
| `validate_approve_plugin_authority`: target FreezeExecute frozen → `reject` | 54-57 | owner-authority plugin frozen, owner approves a delegate | JS `freezeExecuteRemoval.test.ts` "it cannot approve a new authority for FreezeExecute as the owner while frozen" (`InvalidAuthority` — this one does reach the plugin) | Mollusk port |
| → `abstain` | 59 | unfrozen / other target | JS `freezeExecute.test.ts` flow (delegation step) | Mollusk |
| `validate_revoke_plugin_authority`: frozen → `reject` | 66-68 | owner revokes while frozen | JS "it cannot revoke FreezeExecute as the owner while frozen" | Mollusk port |
| unfrozen, plugin authority signs → `approve` | 69-76 | delegate revokes itself after unfreezing | JS "delegate can unfreeze FreezeExecute" + "owner can remove FreezeExecute after delegate unfreezes it" (revoke step) | Mollusk port |
| → `abstain` | 78 | owner revokes unfrozen | — | Mollusk |
| `validate_remove_plugin`: target FreezeExecute frozen → `reject` | 85-88 | RemovePluginV1 FreezeExecute while frozen | JS "it cannot remove FreezeExecute while frozen", "Protocol-delegated FreezeExecute cannot be removed by owner despite freeze" | Mollusk port |
| → `abstain` | 90 | unfrozen, or removing another plugin | JS "owner can remove FreezeExecute after delegate unfreezes it" | Mollusk port |

### src/plugins/internal/owner_managed/transfer_delegate.rs — 14/27 lines (52%)

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_transfer`: plugin authority signs → `approve` | 37-43 | TransferV1 by delegate / by UA when delegated to `UpdateAuthority` (asset or collection) | JS `plugins/asset/delegateTransfer.test.ts` "a delegate can transfer the asset", "it can transfer using delegated update authority", "…from collection" | Mollusk (extend existing transfer tests) |
| → `abstain` | 46 | owner transfers (approved by `AssetV1`), or revoked delegate (`NoApprovals`) | JS "owner can transfer asset with delegate transfer", "it cannot transfer after delegate authority has been revoked" | Mollusk |

### src/plugins/internal/permanent/bubblegum_v2.rs — 8/40 lines (20%)

Creating a collection with BubblegumV2 does not need the Bubblegum program (authority is forced to
`Address{mpl_bubblegum::ID}` in `create_collection.rs:189-197`); only actions *by* that authority would
need CPI, and none exist (the plugin rejects its own removal unconditionally). `ALLOW_LIST` is also consumed by
`create_collection.rs:134-160` (`BlockedByBubblegumV2`, other agent's scope).

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_add_plugin`: target BubblegumV2 → `reject` | 46-51 | self-validation on AddPluginV1/AddCollectionPluginV1 of BubblegumV2 (asset or collection) | JS `plugins/collection/bubblegumV2.test.ts` "it cannot add BubblegumV2 to collection after creation"; `plugins/asset/bubblegumV2.test.ts` "it cannot add BubblegumV2 to asset" | unit + Mollusk |
| `asset_info.is_some()` → `abstain` | 52-56 | AddPluginV1 of a non-allow-listed plugin on an **asset** in a BubblegumV2 collection (collection plugin evaluated with Key::CollectionV1) | JS collection "it can add non-allow-listed plugin to asset in BubblegumV2 collection" | Mollusk |
| allow-listed target on the collection → `abstain` | 57-58 | AddCollectionPluginV1 Attributes/Royalties/UpdateDelegate/… | JS "it can add allow-listed plugins to collection with BubblegumV2 plugin" | unit + Mollusk |
| other target on the collection → `reject` | 62 | AddCollectionPluginV1 ImmutableMetadata/AddBlocker/… | JS "it cannot add non-allow-listed plugins to collection with BubblegumV2 plugin" | unit + Mollusk |
| `target None` → `abstain` | 65 | unreachable | — | unit only |
| `validate_remove_plugin`: target BubblegumV2 → `reject` | 74-76 | RemoveCollectionPluginV1 BubblegumV2 by UA | JS "Update Authority cannot remove BubblegumV2 from collection" | unit + Mollusk |
| other target → `abstain` | 78 | RemoveCollectionPluginV1 of another plugin on a BubblegumV2 collection | none | Mollusk (new, S) |
| `validate_add_external_plugin_adapter`: asset → `abstain` | 89-91 | AddExternalPluginAdapterV1 on an asset in a BubblegumV2 collection | JS "it can add external plugin adapter to asset in BubblegumV2 collection" | Mollusk |
| collection → `reject` | 98 | AddCollectionExternalPluginAdapterV1 on a BubblegumV2 collection | JS "it cannot add external plugin to collection with BubblegumV2 plugin" | unit + Mollusk |

### src/plugins/internal/permanent/edition.rs — 8/26 lines (31%)

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_add_plugin`: target Edition → `reject` | 31-33 | self-validation on AddPluginV1 Edition | JS `plugins/asset/edition.test.ts` "it cannot add edition plugin after mint" (collection variant is rejected earlier by `create_collection.rs:152`) | unit + Mollusk |
| other target → `abstain` | 35 | AddPluginV1 of e.g. Attributes on an asset with Edition | none | Mollusk (new, S) |
| `validate_remove_plugin`: target Edition → `reject` | 45-47 | RemovePluginV1 Edition by UA | JS "it cannot remove edition plugin" | unit + Mollusk |
| other target → `abstain` | 49 | RemovePluginV1 of another plugin with Edition present | none | Mollusk (new, S) |

### src/plugins/internal/permanent/permanent_burn_delegate.rs — 8/28 lines (29%)

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_add_plugin`: target PermanentBurnDelegate → `reject` | 32-35 | self-validation on AddPluginV1/AddCollectionPluginV1 | JS `plugins/collection/permanentBurn.test.ts` "it cannot add permanentBurnDelegate to collection after creation" | unit + Mollusk |
| other target → `abstain` | 37 | any other add with the plugin present | JS `plugins/asset/permanentBurn.test.ts` "it can add another plugin on asset with permanent burn plugin" | Mollusk |
| `validate_burn`: plugin authority signs → `force_approve` (overrides frozen) | 45-48 | BurnV1 by UA / delegate (asset-level or collection-level plugin), even with FreezeDelegate{frozen} | JS `permanentBurn.test.ts` "it can burn an assets as a delegate", "…as a delegate for a collection"; `pluginValidationOverrides.test.ts` "it can burn a frozen asset using PermanentBurnDelegate (ForceApproved overrides freeze)", "…collection PermanentBurnDelegate…" | Mollusk port (ForceApproved short-circuit at `lifecycle.rs:733` is otherwise untested) |
| → `abstain` | 51 | owner burns (approved by `AssetV1`) | JS "it can burn an assets as an owner" | Mollusk |

### src/plugins/internal/permanent/permanent_freeze_delegate.rs — 14/47 lines (30%)

Note the existing Mollusk tests that mention this plugin (`account_ownership.rs:968-1190`) use *fake*
accounts that fail ownership checks before validation, so none of these lines execute.

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_burn`: frozen → `reject` / `abstain` | 46-50 | BurnV1 with asset or collection PermanentFreezeDelegate{frozen} | JS `burn.test.ts` "it cannot burn an asset if collection permanently frozen" | Mollusk |
| `validate_transfer`: frozen → `reject` / `abstain` | 57-61 | TransferV1 with frozen plugin on asset or collection; asset-level unfrozen shadows a frozen collection | JS `plugins/asset/permanentFreeze.test.ts` "it cannot be transferred while frozen", "it cannot move asset in a permanently frozen collection", "it can move asset with permanent freeze override in a frozen collection"; `pluginValidationOverrides.test.ts` two shadowing tests | Mollusk (valid-account variants of the existing fake-account tests) |
| `validate_add_plugin`: target PermanentFreezeDelegate → `reject` | 70-73 | self-validation on add | JS "it cannot add permanentFreeze after creation" (asset + collection) | unit + Mollusk |
| other target → `abstain` | 75 | | JS "it can add another plugin on asset with permanent freeze plugin" | Mollusk |
| `validate_remove_plugin`: frozen → `reject` (**any** target) | 84-85 | RemovePluginV1 of anything while frozen (asset-level or via frozen collection) | JS "it cannot remove permanent freeze plugin if update authority and frozen"; `removePlugin.test.ts` "it cannot remove a plugin from an asset with a frozen collection"; collection "…cannot remove permanentFreezeDelegate from collection when frozen" | Mollusk port |
| unfrozen → `abstain` | 87 | | JS "it can remove permanent freeze plugin if update authority and unfrozen" | Mollusk port |

### src/plugins/internal/permanent/permanent_freeze_execute.rs — 14/42 lines (33%)

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_execute`: frozen → `reject` / `abstain` | 48-52 | ExecuteV1 with asset or collection PermanentFreezeExecute{frozen}; asset-level unfrozen shadows collection | JS `plugins/asset/permanentFreezeExecute.test.ts` "it can freeze and unfreeze execute…", "PermanentFreezeExecute blocks execute but allows burn"; collection "assets inherit … execute is blocked when frozen", "asset-level … overrides collection-level plugin when unfrozen" | Mollusk |
| `validate_add_plugin`: target self → `reject`; other → `abstain` | 61-67 | self-validation on add; adding other plugins | JS "it cannot add PermanentFreezeExecute after creation" (asset + collection), "it can add other plugins alongside PermanentFreezeExecute" | unit + Mollusk |
| `validate_remove_plugin`: target PermanentFreezeExecute frozen → `reject` | 75-78 | RemovePluginV1 of itself while frozen | JS "it cannot remove PermanentFreezeExecute plugin if frozen" (asset + collection) | unit + Mollusk |
| → `abstain` | 81 | unfrozen removal or other target | JS "it can remove PermanentFreezeExecute plugin if unfrozen" | Mollusk |

### src/plugins/internal/permanent/permanent_transfer_delegate.rs — 8/28 lines (29%)

| Path | Lines | Trigger | Existing test | Strategy |
|---|---|---|---|---|
| `validate_add_plugin`: target self → `reject`; other → `abstain` | 31-37 | self-validation on add; other adds | JS `plugins/asset/permanentTransfer.test.ts` "it cannot add permanentTransfer after creation", "it can add another plugin on asset with permanent transfer plugin"; collection variant | unit + Mollusk |
| `validate_transfer`: plugin authority signs → `force_approve` | 44-47 | TransferV1 by UA/delegate (asset or collection plugin), including frozen assets and collections whose Royalties would reject | JS "it can transfer an asset as not the owner", "it can permanent transfer asset that is frozen as a delegate", "…collection permanent transfer asset that is frozen as a collection delegate/update auth"; `pluginValidationOverrides.test.ts` "it can transfer with PermanentTransferDelegate even when collection Royalties would reject" | Mollusk port |
| → `abstain` | 50 | owner transfers; owner of frozen asset (`InvalidAuthority`) | JS "it can transfer asset as the owner", "it cannot transfer asset that is frozen with permanent transfer by owner" | Mollusk |

---

### Dead / special-harness code in scope

| Location | Why |
|---|---|
| `royalties.rs:119` (`MissingNewOwner`) | `transfer.rs` always supplies `new_owner`; only reachable by a unit test |
| `update_delegate.rs:53-57` | `resolved_authorities` is always `None` for `validate_create` |
| `update_delegate.rs:84`, `:106`, `:115`, `:142`, `:212`; `royalties.rs:157-160`; `verified_creators.rs:201-203`; `autograph.rs:126-128`; `add_blocker.rs:30/38` (None arm); `bubblegum_v2.rs:65` | `target_plugin`/`resolved_authorities` are always `Some` on the corresponding lifecycle |
| `bubblegum_v2.rs` | No CPI needed: the Bubblegum program never has to sign for any of these branches |
| Nothing in scope needs lifecycle-hook or oracle programs | external adapters are out of scope; `validate_add_external_plugin_adapter` on BubblegumV2 only needs an `AddCollectionExternalPluginAdapterV1` instruction |

### Bugs / suspicious logic noticed (factual, see per-file notes)

1. `royalties.rs:148` validates `self` instead of `ctx.target_plugin` (masked by `PluginAlreadyExists`).
2. `verified_creators.rs:214-219` approves a no-op `UpdatePluginV1{VerifiedCreators}` from any signer.
3. `update_delegate.rs:49-64` (`validate_create`) can never influence the outcome; half of it is dead.
4. Asset-level plugins shadow collection-level plugins of the same type in `validate_plugin_checks`
   (`utils/mod.rs:198-219`): a collection UpdateDelegate cannot act on an asset that has its own
   UpdateDelegate; an unfrozen asset-level PermanentFreezeDelegate/PermanentFreezeExecute silently disables
   the frozen collection-level one (JS `pluginValidationOverrides.test.ts` treats this as intended).
5. `validate_plugin_checks` (`lifecycle.rs:731`) accepts `Approved` from plugins registered as
   `CanReject` (Autograph, VerifiedCreators return `Approved` from update/add paths).
6. `create_collection.rs:171` uses `payer` (not the update authority) as `authority_info` for
   `validate_create` — affects VerifiedCreators signature checks on collection creation.
7. Removal semantics are inconsistent: FreezeDelegate/PermanentFreezeDelegate block removal of *any*
   plugin while frozen (`ctx.target_plugin.is_some() && self.frozen`), FreezeExecute/PermanentFreezeExecute
   only block removal of themselves. Documented by JS tests, so probably intended, but worth a comment.
8. `freeze_delegate.rs:65-68` is only reachable while the plugin authority is still `Owner`; once delegated,
   the wrapper's `CannotRedelegate` fires first. The JS test named for this case tests the wrapper, not the
   plugin.

---

### Test plan

Naming: `U-` = unit test inside the plugin module, `M-` = Mollusk integration test in `tests/`.
"Covers" lists the plugin lines; Mollusk tests additionally cover the processor/lifecycle code they go
through (noted in parentheses).

| Test case | Sets up | Instruction(s) | Expected | Covers | New / port | Size |
|---|---|---|---|---|---|---|
| U-royalties-validate (4 cases) | `Royalties` structs: bp 10001; pct 50+40; dup creators; valid | direct `validate_royalties` | `InvalidPluginSetting` ×3, `Pass` | royalties.rs:79-105 | new | S |
| U-royalties-transfer (5 cases) | ctx with `authority_info.owner`/`new_owner.owner` = program ids; rule sets None/Allow hit/Allow miss/Deny hit/Deny miss; plus `new_owner=None` | direct `validate_transfer` | Pass/Pass/Rejected/Rejected/Pass, `MissingNewOwner` | royalties.rs:115-141 | new | S |
| U-royalties-add/update-plugin (4 cases) | target Royalties vs Attributes; resolved contains / doesn't contain self_authority | direct calls | per table | royalties.rs:143-172 | new | S |
| U-verified-creators (10 cases) | pairs of `VerifiedCreators` lists for: dup; added; removed; own flag; other flag; PA removing verified other; PA adding verified other; PA adding unverified; empty diff | direct helper calls | error variants per table | verified_creators.rs:59-175 | new | S |
| U-autograph (5 cases) | message change; foreign address; dup; non-authority removal; valid | direct `validate_autograph` | per table | autograph.rs:45-92 | new | S |
| U-update-delegate-update-plugin (4 cases) | self-removal diff; add key; remove other; other target with UA / Address authority | direct `validate_update_plugin` | Approved/Pass/Pass/Approved/Pass | update_delegate.rs:208-240 | new | S |
| U-update-delegate-misc (6 cases) | PA/AD/neither × UA-managed/owner-managed targets for add/remove/approve/revoke; `validate_create` with resolved Some/None | direct calls | per table; documents the precedence fix | update_delegate.rs:49-163 | new | S |
| U-stateless-reject-arms (12 cases) | AddBlocker, ImmutableMetadata, BubblegumV2, Edition, PermanentBurn/Transfer/Freeze/FreezeExecute, FreezeDelegate, FreezeExecute: each callback with target = self / other / None, frozen true/false | direct calls | per table | add_blocker.rs:26-39, immutable_metadata.rs:25-30, bubblegum_v2.rs:42-99, edition.rs:25-51, permanent_*.rs add/remove arms, freeze_*.rs approve/revoke/remove arms | new | S each |
| M-immutable-update | asset with ImmutableMetadata (authority None); variant: asset in collection with it | UpdateV1 name change by UA | `InvalidAuthority` | immutable_metadata.rs:25-30 (+ `processor/update.rs`) | port JS asset/collection immutableMetadata | S |
| M-addblocker-add | asset (and collection) with AddBlocker | AddPluginV1 Attributes; AddPluginV1 FreezeDelegate; AddCollectionPluginV1 AddBlocker post-creation | `InvalidAuthority`; ok; ok | add_blocker.rs:26-39 (+ `add_plugin.rs`) | port JS addBlocker ×4 | M |
| M-royalties-create | CreateV1 with invalid / valid Royalties; CreateCollectionV1 with Royalties | CreateV1, CreateCollectionV1 | `InvalidPluginSetting` / ok | royalties.rs:79-113 (+ `create.rs`, `create_collection.rs`) | port Rust `create_collection_with_plugins` + JS "cannot create royalty…" ×3 | M |
| M-royalties-transfer-rules | valid asset with Royalties: None, AllowList{P}, DenyList{P}; new_owner account owned by P or by system | TransferV1 | ok / `InvalidAuthority` per rule | royalties.rs:115-141 (+ `transfer.rs`) | port JS royalties transfer ×10 (asset + collection variants) | M |
| M-royalties-update-plugin | asset with Royalties | UpdatePluginV1 Royalties by UA (invalid, valid); by owner; UpdatePluginV1 Attributes | `InvalidPluginSetting`; ok; `NoApprovals`; ok | royalties.rs:143-172 (+ `update_plugin.rs`) | port JS "cannot update royalty…" ×3 + new | M |
| M-burn-delegates | asset with BurnDelegate (delegate / UA / collection-UA authority); asset with FreezeDelegate{frozen}; asset with PermanentBurnDelegate + FreezeDelegate{frozen}; collection PermanentFreezeDelegate{frozen} | BurnV1 by delegate / revoked delegate / owner | ok / `NoApprovals` / `InvalidAuthority` / ok (ForceApproved) | burn_delegate.rs:36-46, freeze_delegate.rs:43-48, permanent_burn_delegate.rs:41-52, permanent_freeze_delegate.rs:42-51 (+ `processor/burn.rs`) | port JS burnDelegate ×4, burn.test frozen ×2, pluginValidationOverrides burn ×2 | M |
| M-transfer-delegates | asset with TransferDelegate (delegate / UA authority); PermanentTransferDelegate on asset and on collection with frozen FreezeDelegate/Royalties deny; PermanentFreezeDelegate frozen on asset / collection / shadowed | TransferV1 by delegate / owner / UA | per table | transfer_delegate.rs:37-48, permanent_transfer_delegate.rs:40-51, permanent_freeze_delegate.rs:53-62 (+ `transfer.rs`, `validate_plugin_checks` ForceApproved) | port JS delegateTransfer ×4, permanentTransfer ×6, permanentFreeze ×3, pluginValidationOverrides ×3 | L |
| M-execute-freeze | asset with FreezeExecute{frozen/unfrozen} (owner/delegate authority); PermanentFreezeExecute on asset and on collection, incl. asset-level unfrozen override | ExecuteV1 (reuse `execution_delegate.rs::execute_v1_instruction`) | `InvalidAuthority` when frozen, ok otherwise | freeze_execute.rs:45-50, permanent_freeze_execute.rs:44-53 (+ `execute.rs`) | port Rust `freeze_execute.rs` flow + JS permanentFreezeExecute ×4 | M |
| M-freeze-authority-lifecycle | asset with FreezeDelegate: (a) authority Owner, frozen → ApprovePluginAuthorityV1; (b) delegate-frozen → RevokePluginAuthorityV1 by owner; (c) unfrozen delegate revokes self; (d) owner revokes unfrozen delegate; (e) frozen → RemovePluginV1 TransferDelegate; (f) unfrozen → RemovePluginV1 | Approve/Revoke/RemovePluginV1 | `InvalidAuthority`; `InvalidAuthority`; ok; ok; `InvalidAuthority`; ok | freeze_delegate.rs:65-104 (+ `approve_plugin_authority.rs`, `revoke_plugin_authority.rs`, `remove_plugin.rs`) | (a) new; rest port JS freeze.test / revokeAuthority.test / removePlugin.test | M |
| M-freeze-execute-authority-lifecycle | same six shapes with FreezeExecute, plus "protocol-delegated frozen, owner removes" | Approve/Revoke/RemovePluginV1 | per JS | freeze_execute.rs:54-91 | port JS freezeExecuteRemoval ×7 | M |
| M-permanent-add-after-create | asset/collection with each permanent plugin already present or not | AddPluginV1 / AddCollectionPluginV1 of PermanentFreeze/Transfer/Burn/FreezeExecute, Edition, BubblegumV2; and of Attributes alongside each | `InvalidAuthority` for the permanent ones; ok for Attributes | permanent_*.rs add arms, edition.rs:25-37, bubblegum_v2.rs:46-51 | port JS "cannot add … after creation" ×6 + "can add another plugin…" ×4 | M |
| M-permanent-remove | asset with Edition; collection with BubblegumV2; asset/collection with PermanentFreezeDelegate{frozen/unfrozen}, PermanentFreezeExecute{frozen/unfrozen}; plus removing Attributes next to each | RemovePluginV1 / RemoveCollectionPluginV1 by UA | reject when frozen / Edition / BubblegumV2; ok otherwise | edition.rs:39-51, bubblegum_v2.rs:69-80, permanent_freeze_delegate.rs:80-89, permanent_freeze_execute.rs:71-82 | port JS edition, bubblegumV2, permanentFreeze(Execute) removal tests + new "other target" cases | M |
| M-bubblegum-collection-adds | collection with BubblegumV2 | AddCollectionPluginV1 Attributes (allow-listed) / ImmutableMetadata (not); AddPluginV1 ImmutableMetadata on an asset in that collection; AddCollectionExternalPluginAdapterV1 AppData; AddExternalPluginAdapterV1 on the asset | ok / `InvalidAuthority` / ok / `InvalidAuthority` / ok | bubblegum_v2.rs:42-99 (+ `add_external_plugin_adapter.rs`) | port JS collection/bubblegumV2 ×6 | M |
| M-update-delegate-asset | asset with UpdateDelegate (authority UA or Address{d}, `additional_delegates=[ad]`) plus Attributes(UA) and FreezeDelegate(owner) | UpdateV1 by d/ad/revoked; AddPluginV1 Attributes / FreezeDelegate by d; RemovePluginV1 same; ApprovePluginAuthorityV1 Attributes by ad, UpdateDelegate by ad; RevokePluginAuthorityV1 Attributes / FreezeDelegate / TransferDelegate by d and by ad | per JS expectations (`InvalidAuthority`/`NoApprovals`/ok) | update_delegate.rs:66-163, 165-206 approve/abstain arms (+ all five plugin processors) | port JS asset/updateDelegate (subset ~14) + updateDelegateRevokeBug ×5 | L |
| M-update-delegate-update-plugin | as above | UpdatePluginV1 UpdateDelegate by ad removing self / adding key / removing other; by UA adding key; UpdatePluginV1 Attributes by ad; UpdatePluginV1 FreezeDelegate (authority Address{x}) by ad | ok / `NoApprovals` / `NoApprovals` / ok / ok / `NoApprovals` | update_delegate.rs:208-240 | port JS updateDelegate ×6 | M |
| M-update-delegate-collection | collection with UpdateDelegate(+ad); asset in collection with/without its own UpdateDelegate | UpdateV2 by collection delegate: rename asset; remove asset from collection (asset has no UD → ok; asset has UD → shadowed, `NoApprovals`); UpdateV2 by asset-level delegate removing from collection → `InvalidAuthority`; UpdateCollectionV1 by ad | per table | update_delegate.rs:182-204 (reject arm), collection-side approve (+ `update.rs` collection re-parenting) | port JS updateV2.test 1051-1388 subset + collection/updateDelegate ×4 | L |
| M-verified-creators | CreateV1 with VerifiedCreators (valid; foreign verified); asset with VerifiedCreators {UA unverified, C unverified} | CreateV1; UpdatePluginV1 by C verifying self; by C verifying UA; by UA removing verified C; by UA adding unverified; by UA removing unverified; AddPluginV1 Attributes | per table (`MissingSigner`, `InvalidPluginOperation`, ok) | verified_creators.rs:178-224 (+ helpers end-to-end) | port JS verifiedCreators ×8 | M |
| M-autograph | CreateV1 with Autograph (self / foreign); asset with Autograph{owner sig} | AddPluginV1 Autograph by owner / by UA; UpdatePluginV1 by 3rd party adding own sig / editing message / removing owner sig; by owner removing; by delegate; AddPluginV1 Attributes | per table | autograph.rs:102-141 | port JS autograph ×9 | M |

### Harness prerequisites

1. **Generic account builders in `tests/common/mod.rs`**: generalize `account_ownership.rs::build_asset_with_plugins`
   / `build_collection_with_plugins` to accept any `Plugin` (derive the registry type with
   `PluginType::from(&plugin)` instead of the 4-arm match), an explicit `owner`, `update_authority`
   (`Address` or `Collection(pubkey)`), and an `external_registry` (needed only for the BubblegumV2 external
   adapter case). This unlocks every M- test above without going through CreateV1.
2. **Instruction encoders** for `AddPluginV1`/`AddCollectionPluginV1`, `RemovePluginV1`/`RemoveCollectionPluginV1`,
   `UpdatePluginV1`/`UpdateCollectionPluginV1`, `ApprovePluginAuthorityV1`/`…Collection…`,
   `RevokePluginAuthorityV1`/`…Collection…`, `UpdateV1`/`UpdateV2`/`UpdateCollectionV1`, `BurnV1`, `CreateV1`,
   `CreateCollectionV1`, `AddExternalPluginAdapterV1`/`AddCollectionExternalPluginAdapterV1`. Either hand-encode
   discriminators as `account_ownership.rs:277-357` does, or add `mpl-core = { path = "../../clients/rust" }`
   as a dev-dependency and use the generated `*Builder`s (the client crate has no dependency on the program
   crate, so no cycle).
3. **Error assertion helper** taking an `MplCoreError` (generalize `execution_delegate.rs::assert_failure`),
   so tests assert the exact variant (`InvalidAuthority` vs `NoApprovals` vs `InvalidPluginSetting` …) — the
   distinction is what proves which arm fired.
4. **Program-owned "new owner" accounts** for Royalties rule sets: `Account { owner: <listed program>, .. }`.
5. **Asset-in-collection fixture**: asset with `update_authority = Collection(c)` plus the collection account in
   the same instruction (all asset lifecycles require the collection account when the asset is in one).
6. **Unit-test context helper** (`src/plugins/test_ctx.rs`, `#[cfg(test)]`): builds `AccountInfo`s from local
   buffers (key, owner program, optional serialized `AssetV1` data) and a `PluginValidationContext` with
   `accounts: &[]`, `self_key: Key::AssetV1`, everything else `None`, plus setters for `self_authority`,
   `resolved_authorities`, `target_plugin`, `target_plugin_authority`, `new_owner`, `new_asset_authority`.
   For `UpdateDelegate::validate_update`'s reject arm the asset `AccountInfo` needs real bytes
   (`AssetV1` + `PluginHeaderV1` + `UpdateDelegate` + `PluginRegistryV1`, same layout as prerequisite 1).

### Effort and ordering

Estimated tests: ~45 unit cases (all S, ~1 day total) and ~25 Mollusk test functions (each parameterized
over 2-6 scenarios; ~90 scenarios). Ordering by coverage gained per unit of effort:

1. Unit tests for the pure helpers and stateless arms (U-royalties-*, U-verified-creators, U-autograph,
   U-stateless-reject-arms): takes the 12 small files from ~30% to ~100% in isolation, no harness work.
2. Harness prerequisites 1-3 (one-off, unblocks everything else and every other processor section).
3. M-burn-delegates, M-transfer-delegates, M-execute-freeze: reuse existing burn/transfer/execute encoders,
   lift `processor/burn.rs` 0→~80% and cover all owner/permanent delegate `validate_burn/transfer/execute` arms.
4. M-immutable-update, M-permanent-add-after-create, M-permanent-remove, M-addblocker-add: first coverage of
   `processor/update.rs`, `add_plugin.rs`, `remove_plugin.rs`.
5. M-freeze-authority-lifecycle, M-freeze-execute-authority-lifecycle: first coverage of
   `approve/revoke_plugin_authority.rs` plus the wrapper short-circuits.
6. M-update-delegate-* (three tests, L): largest single-file gain (update_delegate.rs 18%→~95%) and the
   regression tests for the precedence security fix; needs the asset-in-collection fixture.
7. M-royalties-*, M-verified-creators, M-autograph, M-bubblegum-collection-adds: remaining arms and
   `update_plugin.rs` / external-adapter add paths.


## 13. State types and utilities

Scope: `src/state/{asset,collection,traits,mod,hashed_asset,hashable_plugin_schema,compression_proof,update_authority,collect}.rs`, `src/utils/{mod,account,compression}.rs`, `src/error.rs`, `src/entrypoint.rs`, `src/lib.rs`. (`src/state/group.rs` is covered in the groups section; the group-authority helpers that live in `src/utils/mod.rs` are covered here.)

| File | Lines covered | Notes |
|---|---|---|
| src/state/asset.rs | 125/291 (43.0%) | `len`/`new` covered; most `check_*`/`validate_*` untouched |
| src/state/collection.rs | 44/268 (16.4%) | only `new`, `len`, `key` covered; no collection-targeted instruction runs in Mollusk today |
| src/state/traits.rs | 13/34 (38.2%) | `load`/`save` happy paths covered; error arms, `hash`, `wrap` not |
| src/state/mod.rs | 27/28 (96.4%) | line 139 is a derive-expansion artefact; nothing to do |
| src/state/hashed_asset.rs | 14/17 (82.4%) | `SolanaAccount::key()` only reachable via `HashedAssetV1::load` (compression) |
| src/state/hashable_plugin_schema.rs | 0/3 | `compare_indeces` (compression) |
| src/state/compression_proof.rs | 0/10 | `CompressionProof::new` (compression) |
| src/state/update_authority.rs | 0/6 | `UpdateAuthority::key()` — only caller reached by an instruction is `AssetV1::validate_update` |
| src/state/collect.rs | 9/9 (100%) | function entries listed as uncovered only in the unit-test build hash; nothing to do |
| src/utils/mod.rs | 170/479 (35.5%) | collection branch of `validate_asset_permissions`, all of `validate_collection_permissions`, all `is_valid_*_authority`, `assert_*authority` uncovered |
| src/utils/account.rs | 28/49 (57.1%) | `close_program_account` never runs; realloc grow/shrink covered except the failed-transfer arm |
| src/utils/compression.rs | 0/92 | dormant (compression returns `NotAvailable`) but fully reachable before the error |
| src/error.rs | 3/3 (100%) | — |
| src/entrypoint.rs | 11/11 (100%) | — |
| src/lib.rs | n/a | only `declare_id!`/consts; no measurable regions |

Existing `#[cfg(test)]` modules already cover: `AssetV1::len` (three variants incl. `Collection`+`seq`), `CollectionV1::len`, `Authority::len` (all variants, `EnumCount`-guarded), `Key::len` (all variants via `EnumIter`), `HashedAssetV1::new`/`len`. Do not add more `len` round-trip tests.

### Cross-cutting findings (read first)

**1. Most of `tests/account_ownership.rs` never executes the program.** 21 of its 25 tests pass `(MPL_CORE_ID, Account::default())` as the optional-account sentinel (a non-executable, system-owned, zero-lamport account under the program id) and end with `assert_failure`, which accepts any error. Running `transfer_rejects_valid_frozen_asset` alone under `cargo llvm-cov` produces **0 hits in `src/entrypoint.rs`** — the runtime rejects the instruction before the program runs. The two tests that expect success use `core_program_account()` instead and do reach the program. Consequences for this scope: every `burn_*`, `update_*`, `transfer_rejects_*` and frozen/permanent-delegate test in that file contributes nothing, which is why `AssetV1::check_burn/validate_burn/check_update/validate_update`, `close_program_account`, the `Rejected`/`ForceApproved` arms of `validate_asset_permissions` (`src/utils/mod.rs:279-284`) and the `rejected` exit (`:314`) all read as uncovered. Fix before writing anything new: use `core_program_account()` for the sentinel in all 21 tests and replace `assert_failure` with an `assert_error(&result, MplCoreError::X)` helper that matches `ProgramResult::Failure(InstructionError::Custom(code))`. This single change is the cheapest coverage gain in this scope.

**2. Two crate hashes in the "uncovered functions" lists.** Entries tagged `Cs30NUS1hy9Hm_` come from the lib as linked into the integration-test binaries; `Cs9dbz0oj2f79_` is the lib compiled with `cfg(test)` for unit tests. A function listed under only one hash is covered overall (e.g. `load_key`, `resolve_authority`, `AssetV1::new`, `get_create_fee`, `entrypoint::process_instruction`). Only functions listed under both hashes are really uncovered; the analysis below uses the line data, which is merged.

**3. Compression code is dormant but reachable.** `CompressV1` runs `validate_asset_permissions` → `compress_into_account_space` → `CompressionProof::wrap()` (CPI to SPL noop) and *then* returns `NotAvailable` (`src/processor/compress.rs:41-82`). `DecompressV1`, hashed `BurnV1` and hashed `TransferV1` run `verify_proof` → `rebuild_account_state_from_proof_data` (and `wrap()` for burn) before `NotAvailable`. Mollusk rolls the state back but coverage still counts, and the distinction between `NotAvailable` (23) and `IncorrectAssetHash` (7) / `NoApprovals` (26) is a meaningful assertion. Requirements: a `HashedAssetV1` account crafted in-test (no on-chain path can create one), and the SPL noop program loaded into Mollusk (`wrap()` invokes `SPL_NOOP_ID` unconditionally; today's tests only register an empty loader-v3 account for it, which is enough only because no current test reaches `wrap()`).

**4. Dead or instruction-unreachable code in this scope** (unit-testable only, or candidates for removal):
- `assert_authority` (`src/utils/mod.rs:37-62`): no callers anywhere in `src`. The `CoreAsset` impl for `AssetV1` (`src/state/asset.rs:418-426`) and `CollectionV1::update_authority()` (`src/state/collection.rs:386-388`) are reachable only through it (`CollectionV1::owner()` is used by `resolve_pubkey_to_authorities_collection`).
- `AssetV1::check_update_external_plugin_adapter`/`validate_update_external_plugin_adapter` (`asset.rs:136-138, 371-378`) and the `CollectionV1` equivalents (`collection.rs:122-124, 333-340`): never referenced by any processor (`update_external_plugin_adapter.rs` uses `resolve_pubkey_to_authorities*` directly).
- Validators whose paired `check_*` returns `CheckResult::None` are never invoked by `validate_asset_permissions`/`validate_collection_permissions`: `AssetV1::validate_update_plugin` (`asset.rs:206-213`), `CollectionV1::validate_update_plugin`, `validate_transfer`, `validate_burn`, `validate_compress`, `validate_decompress` (`collection.rs:188-195, 240-257, 274-291`). The `check_*` functions themselves *are* reachable.
- `src/utils/mod.rs:174-178` (MissingCollection/InvalidCollection inside `validate_asset_permissions`): pre-empted by `resolve_pubkey_to_authorities` (`:488-500`) which is called first (`:170`) and returns the same errors for the same conditions. Line `:180` (collection supplied for an asset that is not in one) is reachable.
- `src/utils/mod.rs:165` and `:363` (`panic!` on mismatched fp/event parameters), `:309`, `:465` (`unreachable!()`), `:408` (`IncorrectAccount` when `core_check.0 != CollectionV1`, but it is always `CollectionV1`), `:240`/`:243` (Rejected/ForceApproved from `AssetV1` validators, which only approve or abstain), `:259`/`:415` (ForceApproved from `CollectionV1` validators, likewise).
- `src/processor/transfer.rs:126` call to `compress_into_account_space`: unreachable because the hashed branch returns `NotAvailable` at `:72`.
- The `None => Err(InvalidPlugin)` arms in `validate_add_plugin`/`validate_remove_plugin`/`validate_approve_plugin_authority`/`validate_revoke_plugin_authority` (`asset.rs:168, 192`; `collection.rs:154, 175, 206, 227`) and the `else { abstain!() }` for `plugin == None` (`asset.rs:232, 253`): every processor passes `Some(&plugin)`; unit-test only.

### src/state/asset.rs

Current: 125/291 lines. Uncovered paths, grouped by trigger:

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 63-64 | `increment_seq_and_save` with `seq == Some` | Only if an `AssetV1` account already has `seq: Some(_)` — impossible on-chain (only decompress sets it, and decompress is disabled). Craft an account with `seq: Some(3)` and run `UpdateV1`/`AddPluginV1`/`ExecuteV1`; expect `seq == Some(4)` afterwards. Also exercises `SolanaAccount::save` on a longer payload. | none (new) |
| 76-78, 160-181 | `check_add_plugin`, `validate_add_plugin` | `AddPluginV1` on an asset: owner adds owner-managed plugin (`FreezeDelegate`) → approve; UA adds authority-managed (`Attributes`) → approve; owner adds `Attributes` while UA is a different address → abstain → `NoApprovals`; asset in collection (`UpdateAuthority::Collection`) → abstain, collection decides | JS `addPlugin.test.ts` ('it can add a plugin to an asset', 'it cannot add an authority-managed plugin as owner', ':292', ':485') |
| 81-83, 184-203 | `check_remove_plugin`, `validate_remove_plugin` | `RemovePluginV1` with the same authority matrix | JS `removePlugin.test.ts` (':99', ':142') |
| 86-88 | `check_update_plugin` (returns `None`) | `UpdatePluginV1` on any asset (validator itself is dead, see above) | JS `updatePlugin.test.ts` |
| 91-98, 216-255 | approve/revoke plugin authority | `ApprovePluginAuthorityV1` / `RevokePluginAuthorityV1`: owner on owner-managed plugin → approve; UA on UA-managed → approve; cross → abstain | JS `approveAuthority.test.ts`, `revokeAuthority.test.ts` |
| 106-108, 272-283 | burn | `BurnV1` by owner → approve (`:279`); by any other signer → abstain (`:281`) then `NoApprovals` | JS `burn.test.ts` ':26', ':45'; existing Mollusk `burn_rejects_unauthorized_caller_on_valid_asset` once fixed (finding 1) |
| 111-113, 258-269 | update | `UpdateV1`/`UpdateV2` by UA (`Address`) → approve (`:265`, also covers `UpdateAuthority::key()`); other signer → abstain (`:267`) | JS `update.test.ts` ':160', ':264'; Mollusk `update_rejects_*` once fixed |
| 116-118, 300-311 | compress | `CompressV1` by owner → approve (`:307`) then `NotAvailable`; by other → abstain (`:309`) then `NoApprovals` | JS `compress.test.ts:90` ('not available'); skipped 'if not the owner' |
| 121-123, 314-325 | decompress | `DecompressV1` on a crafted hashed asset: owner → approve, other → abstain. Note `rebuild_account_state_from_proof_data` runs *before* validation. | JS `decompress.test.ts` (all skipped) → new |
| 154 | `validate_create` with `UpdateAuthority::Collection` → abstain | `CreateV1/V2` with a collection account | JS `createCollection.test.ts:62` |
| 295 | `validate_transfer` abstain | `TransferV1` by non-owner without a delegate → `NoApprovals` | JS `transfer.test.ts:37`; Mollusk `transfer_rejects_unauthorized_caller_on_valid_asset` once fixed |
| 352, 366 | add/remove external adapter abstain | `AddExternalPluginAdapterV1`/`RemoveExternalPluginAdapterV1` by a signer that is not the UA → `NoApprovals`; or asset in collection (collection approves) | JS `externalPlugins/appData.test.ts` (wrong-authority cases) |
| 406-415 | `From<CompressionProof>` | `verify_proof` (decompress / hashed burn / hashed transfer) | new; also unit-testable |
| 419-425 | `CoreAsset` impl | dead except via `assert_authority` | unit test only |
| 136-138, 206-213, 371-378 | dead validators (finding 4) | unit test only | — |

### src/state/collection.rs

Current: 44/268 lines. No Mollusk test executes a collection-targeted instruction or an asset-in-collection instruction, so everything except `new`/`len`/`key` is uncovered. Two distinct entry points feed this file: (a) `validate_asset_permissions` with a collection account (asset in collection), which calls `CollectionV1::check_*` and, when the check is `CanApprove`, `CollectionV1::validate_*` (`src/utils/mod.rs:191-195, 248-262`); (b) `validate_collection_permissions` for `*CollectionV1` instructions (`src/utils/mod.rs:376, 394-418`).

| Lines | Path | Trigger | Existing test to port |
|---|---|---|---|
| 57-59, 132-143 | create | `CreateV1/V2` into a collection: signer == `collection.update_authority` → approve (`:139`); other signer → abstain (`:141`) → `NoApprovals` unless an `UpdateDelegate` on the collection approves | JS `createCollection.test.ts:62, :97, :135` |
| 343-360 | `increment_minted`, `increment_size` | `CreateV1/V2` into a collection (`create.rs:310-311`); `UpdateV2` moving an asset into a collection (`update.rs:238`). Overflow arms (`:347`, `:357`) need `u32::MAX` counters → craft the collection account, or unit test. | JS `createCollection.test.ts:62`; `updateV2.test.ts:443` |
| 363-370 | `decrement_size` | `BurnV1` of an asset in a collection (`burn.rs:109`); `UpdateV2` removing from collection (`update.rs:158`). Underflow arm (`:367-368`): collection with `current_size == 0` while an asset still references it → `NumericalOverflowError` (crafted account). | JS `collectionSize.test.ts`, `burn.test.ts:141-206`; `updateV2.test.ts:281` |
| 62-64, 146-164 | add plugin | (a) `AddPluginV1` on asset in collection by collection UA → approve; (b) `AddCollectionPluginV1` by UA with UA-managed plugin → approve; owner-managed plugin → abstain → `InvalidAuthority` (`utils:469`) | JS `addPlugin.test.ts:174, :213, :292, :408, :485` |
| 67-69, 167-185 | remove plugin | `RemovePluginV1` (asset in collection) / `RemoveCollectionPluginV1` | JS `removePlugin.test.ts:78, :99, :184, :307` |
| 72-74 | `check_update_plugin` → `None` | `UpdatePluginV1` on asset in collection / `UpdateCollectionPluginV1` | JS `updatePlugin.test.ts:219, :260` |
| 77-84, 198-237 | approve/revoke plugin authority | `Approve/RevokePluginAuthorityV1` (asset in collection) and `Approve/RevokeCollectionPluginAuthorityV1` | JS `approveAuthority.test.ts`, `revokeAuthority.test.ts` (collection cases) |
| 87-89, 92-94, 102-109 | `check_transfer/burn/compress/decompress` → `None` | `TransferV1`/`BurnV1`/`CompressV1`/`DecompressV1` on an asset in a collection (validators are dead) | JS `transfer.test.ts:81`, `burn.test.ts:372` |
| 97-99, 260-271 | update | (a) `UpdateV1/V2` on asset in collection: signer == collection UA → approve; (b) `UpdateCollectionV1` by UA → approve, by other → abstain → `InvalidAuthority`; (c) `BurnCollectionV1` reuses `check_update`/`validate_update` (`burn.rs:160-162`) | JS `update.test.ts` collection cases, `burnCollection.test.ts:20, :39`; Mollusk `update_collection_rejects_*` once fixed |
| 112-119, 304-330 | add/remove external adapter | `AddExternalPluginAdapterV1` on asset in collection, `AddCollectionExternalPluginAdapterV1`, `RemoveCollectionExternalPluginAdapterV1`; wrong signer → abstain | JS `externalPlugins/oracle.test.ts:405` ('deny update via collection'), `appData.test.ts` collection cases; `tests/agent_identity.rs` has an `add_collection_agent_identity_instruction` builder that no test currently uses |
| 127-129, 294-301 | execute | `ExecuteV1` on asset in collection → collection abstains, owner approves | JS `execute.test.ts:312` |
| 386-392 | `CoreAsset` impl | `owner()` via `resolve_pubkey_to_authorities_collection` (any collection instruction); `update_authority()` dead | — |
| 122-124, 188-195, 240-257, 274-291, 333-340 | dead (finding 4) | unit test only | — |

### src/state/traits.rs

Current: 13/34 lines.

| Lines | Path | Trigger | Unit or Mollusk |
|---|---|---|---|
| 27 | `load`: discriminator mismatch → `DeserializationError` | Any `T::load` on an account carrying a different valid key, e.g. an asset whose `update_authority` is `Collection(X)` where `X` is an `AssetV1` account, then `TransferV1` with `X` as collection → `CollectionV1::load` fails. (Random/empty discriminators fail earlier in `load_key`, already covered.) | both |
| 36-38 | `load`: Borsh failure after a valid key byte → `DeserializationError` | Account = `[Key::AssetV1]` + truncated payload; `TransferV1` | both (unit test with a local `AccountInfo` is simplest) |
| 44-46 | `save`: `borsh::to_writer` into a too-small slice → `SerializationError` | Not reachable through instructions (every caller resizes first or writes an equal-length payload). Unit test with a 1-byte `AccountInfo` buffer. | unit |
| 53-56 | `Compressible::hash` | Compression paths; also trivially unit-testable (`keccak(borsh(x))`). | both |
| 62-72 | `Wrappable::wrap` (CPI to SPL noop) | `CompressV1` on an `AssetV1`; hashed `BurnV1`. Needs SPL noop in Mollusk. | Mollusk |

### Small state files

- `src/state/mod.rs:139`: inside `enum Authority`; a derive-expanded region (`PartialOrd`/`Ord`) for the `UpdateAuthority` variant. It will be hit incidentally by any `BTreeMap<Authority, _>`/`contains` use; not worth a targeted test.
- `src/state/hashed_asset.rs:35-37`: `SolanaAccount::key()` for `HashedAssetV1`, reached only by `HashedAssetV1::load` in `verify_proof` (`src/utils/compression.rs:146`). Mollusk decompress/hashed-burn test. (`new`/`len` already unit-tested.)
- `src/state/hashable_plugin_schema.rs:25-27`: `compare_indeces`, reached by the `sort_by` in `verify_proof` (`:132`). Unit test (sort two schemas with indices `[1, 0]`), and pass an unsorted `plugins` vector in the Mollusk decompress test so the sort is meaningful.
- `src/state/compression_proof.rs:27-36`: `CompressionProof::new`, reached by `compress_into_account_space` (`:75`). Unit-test together with `AssetV1::from(CompressionProof)` (round-trip: `AssetV1::from(CompressionProof::new(asset, seq, plugins))` equals the asset with `seq: Some(seq)`).
- `src/state/update_authority.rs:17-23`: `UpdateAuthority::key()`; `None → Pubkey::default()` (the system program id), `Address(a) → a`, `Collection(c) → c`. Instruction-reachable only via `AssetV1::validate_update` (`asset.rs:264`). Unit test all three arms; the `None` arm cannot be made to approve on-chain because the system program cannot sign.
- `src/state/collect.rs`: 100% lines; the two function entries are flagged only for the unit-test build hash. Nothing to do.

### src/utils/mod.rs

Current: 170/479 lines.

**`load_key` (29-34)**: covered.

**`assert_authority` (37-62)**: dead (finding 4). Unit test the four `Authority` arms against an `AssetV1` (Owner match/mismatch, UpdateAuthority match/mismatch, Address match/mismatch, None → `InvalidAuthority`) if kept; otherwise delete.

**`assert_collection_authority` (65-85)**: single caller `src/processor/update.rs:219`, inside `UpdateV2` when the *new* collection carries an `UpdateDelegate` plugin. Arms: plugin authority `UpdateAuthority` and signer == new collection UA → `Ok` (72-75); plugin authority `Address{x}` and signer `x` → `Ok` (77-80); `None`/`Owner` (71) or mismatch → `Err` (84), after which `update.rs` still accepts the signer if it is the collection UA or in `additional_delegates`. Port JS `updateV2.test.ts:857` ('change collection using delegate'), `:1388` ('additional update delegate on new collection'), `:583` ('not both asset and collection auth' → `InvalidAuthority`).

**`fetch_core_data` (88-101)**: lines covered via `AssetV1`; the `CollectionV1` instantiation needs any collection instruction (or `UpdateCollectionInfoV1`, which requires the Bubblegum signer PDA — special harness support, see groups/CPI notes elsewhere; Rust client `update_collection_info.rs` only tests the wrong-signer rejection).

**`save_flat_group` (105-120)**: group instructions only (`create_group.rs:146`, `add_*_to_group.rs`, `remove_*_from_group.rs`, `update_group.rs:76`). `:113-114` (resize when the serialized length changes) needs a relation to be added/removed; `:117` always. Owned by the groups section; listed here for completeness.

**`validate_asset_permissions` (124-320)** — uncovered arms:

| Lines | Condition | Trigger |
|---|---|---|
| 165 | fp/event mismatch panic | unreachable (all 16 call sites are consistent) |
| 174-178 | asset in collection, collection missing/wrong | unreachable — pre-empted by `resolve_pubkey_to_authorities` (`:499` → `MissingCollection`, `:492` → `InvalidCollection`) |
| 180 | collection account supplied for an asset whose UA is `Address`/`None` → `InvalidCollection` | `TransferV1` with a collection on a stand-alone asset. JS `addPlugin.test.ts:519` ('collection is wrong') |
| 192 | `collection_check_fp()` | any instruction with the collection account present |
| 199-212 | collection registry / adapter registry scan | asset in a collection that has internal plugins (`Royalties`, `PermanentFreezeDelegate`, `UpdateDelegate`) and/or external adapters (`Oracle`, `AppData`) |
| 225 | `?` on asset `check_adapter_registry` | only a corrupt external registry record (`ExternalPluginAdapterKey::from_record` failing); crafted account |
| 240, 243 | asset validator Rejected/ForceApproved | unreachable |
| 249-257, 259 | collection core validation | any `CanApprove` collection check with an asset in a collection: create, add/remove plugin, approve/revoke, update, execute, add/remove external adapter. `:259` unreachable |
| 279 | `?` on `validate_plugin_checks` | a plugin validator returning `Err` (e.g. `VerifiedCreators`/`Autograph` → `MissingSigner`; `Plugin::load` on a corrupt offset) |
| 280 | plugin Approved | `TransferDelegate` (authority `Address{d}`, signer `d`) on `TransferV1`; `BurnDelegate` on `BurnV1`; `UpdateDelegate` on `UpdateV1` |
| 281 | plugin Rejected | `FreezeDelegate{frozen:true}` on `TransferV1`/`BurnV1`; `ImmutableMetadata` on `UpdateV1`; `AddBlocker` on `AddPluginV1`. The existing `transfer_rejects_valid_frozen_asset` will hit this once finding 1 is fixed |
| 284 | plugin ForceApproved | `PermanentTransferDelegate`/`PermanentBurnDelegate` with the signer as plugin authority (even when a `FreezeDelegate` is frozen) |
| 304 | `?` on external adapter checks | `Oracle` adapter whose account is absent → `MissingExternalPluginAdapterAccount`, uninitialised → `UninitializedOracleAccount`, malformed → `InvalidOracleAccountData` |
| 306 | external adapter Rejected | `Oracle` with `Transfer: CanReject` and oracle account data = `Rejected` (JS `oracle.test.ts:502`, `:319`) |
| 314 | `rejected` → `InvalidAuthority` | any of `:281`/`:306`. Note `MplCoreError::AssetIsFrozen` is never emitted anywhere in `src`; frozen assets fail with `InvalidAuthority` |

**`validate_collection_permissions` (324-474)**: entirely uncovered. Callers: `UpdateCollectionV1`, `BurnCollectionV1` (with `check_update`/`validate_update` + `Plugin::validate_burn`), `AddCollectionPluginV1`, `RemoveCollectionPluginV1`, `UpdateCollectionPluginV1` (check `None` → core validation skipped), `ApproveCollectionPluginAuthorityV1`, `RevokeCollectionPluginAuthorityV1`, `AddCollectionExternalPluginAdapterV1`, `RemoveCollectionExternalPluginAdapterV1`. Arms: `:379-389` registry scan (collection with plugins/adapters); `:394-418` core validation (Approved via UA, Pass via other signer); `:420-442` plugin checks — Approved (`UpdateDelegate` with the signer in `additional_delegates` on `UpdateCollectionV1`), Rejected (`ImmutableMetadata` on `UpdateCollectionV1`, `AddBlocker` on `AddCollectionPluginV1`, `Groups` on `BurnCollectionV1`), ForceApproved (`PermanentBurnDelegate` with authority `UpdateAuthority` on `BurnCollectionV1`, since the UA must sign anyway); `:444-467` external adapter checks (`Oracle` on the collection rejecting `Update` → `:463`; `AppData` → Pass); `:469-471` `rejected || !approved` → `InvalidAuthority`. Note the asymmetry with the asset path, which returns `NoApprovals` for "nobody approved" (`:316`); tests should pin `InvalidAuthority` here. Port from JS `update.test.ts` (collection cases), `burnCollection.test.ts`, `addPlugin.test.ts:174/213/408`, `removePlugin.test.ts:78/184/307`, `plugins/collection/immutableMetadata.test.ts`, `plugins/collection/updateDelegate.test.ts`, `oracle.test.ts:405`; Rust `clients/rust/tests/create_collection.rs`, `remove_external_plugins_on_collection.rs`, `update_external_plugins_on_collection.rs`.

**`resolve_pubkey_to_authorities` (476-508)**: `:492` wrong collection → `InvalidCollection` (JS `transfer.test.ts:101`, `burn.test.ts:268`); `:495-497` signer == collection UA → `UpdateAuthority` pushed (any asset-in-collection instruction signed by the collection UA); `:499` collection missing → `MissingCollection` (JS `transfer.test.ts:60`, `burn.test.ts:141`, `execute.test.ts:292`).

**`resolve_pubkey_to_authorities_collection` (510-529)**: any collection instruction. `:516-518` pushes `Authority::Owner` when the signer is the collection UA because `CollectionV1::owner()` returns `update_authority`.

**`resolve_authority` (532-543)**: covered.

**`is_valid_asset_authority` (553-604)** — group instructions only (`CreateGroupV1` with assets, `AddAssetsToGroupV1`, `RemoveAssetsFromGroupV1`): `:561-565` UA `Address` == signer; `:566-572` UA `Collection` with the collection account in the transaction → `is_valid_collection_authority`; `:573-579` collection not in transaction → log, fall through; `:582` UA `None`; `:586-595` `UpdateDelegate` on the asset with the signer in `additional_delegates`; `:596` `Ok(_)` → `InvalidPlugin` only if a registry record typed `UpdateDelegate` points at a different plugin's bytes (crafted account); `:597-599` `PluginNotFound`/`PluginsNotInitialized` → `Ok(false)`; `:600` other errors propagate (corrupt header). Port JS `group.test.ts:48/67/96/123`, `createGroup.test.ts:152`.

**`is_valid_group_authority` (607-613)** and **`is_valid_collection_authority` (617-648)**: group instructions (`AddCollectionsToGroupV1`, `RemoveCollectionsFromGroupV1`, `CreateGroupV1` with collections, and via the `Collection` arm above). Same arm structure as `is_valid_asset_authority`.

### src/utils/account.rs

Current: 28/49 lines.

- `close_program_account` (10-35): `BurnV1` (`burn.rs:172`), `BurnCollectionV1`, `CloseGroupV1`. Refund = `minimum_balance(len) - minimum_balance(1)` credited to the payer/authority; the account is shrunk to one byte holding `Key::Uninitialized`. Assert after burn: `data == [0]`, `lamports == old - refund`, payer credited. The unchecked `-=` at `:29` (see notes) can be pinned with a crafted account whose lamports are below `minimum_balance(len)`.
- `resize_or_reallocate_account`: grow (49-63) and shrink (64-70) paths are covered; `:63` (system-transfer failure) needs a payer with insufficient lamports on a growing instruction (`AddPluginV1` with a 0-lamport payer) → system program `InsufficientFunds`.

### src/utils/compression.rs

Current: 0/92 lines. All three functions are reachable, all end in `NotAvailable` from their callers.

- `rebuild_account_state_from_proof_data` (21-61): `DecompressV1`, hashed `BurnV1`, hashed `TransferV1`. Branches: `:41` `plugins.is_empty()` (skip) vs non-empty (`create_meta_idempotent` + `initialize_plugin` per plugin, `:42-59`). Grows the account from 33 bytes to the asset length → system transfer from the payer (`account.rs:58-63`).
- `compress_into_account_space` (64-121): `CompressV1` on an `AssetV1` (the `transfer.rs:126` caller is dead). Branches: no registry (`:78` skipped) vs registry present (`:78-98` loop, `HashablePluginSchema` per record, `compare_offsets` sort). Shrinks the account to 33 bytes (refund to payer, `account.rs:64-70`).
- `verify_proof` (124-152): hash equal (`:151`) vs mismatch (`:147-149` → `IncorrectAssetHash`). The proof's `plugins` are sorted by `index` (`:132`), so pass them unsorted.

Hash construction for the crafted account (all types are `pub` in `mpl_core_program::state`): `asset = AssetV1::from(proof.clone())` (note `seq: Some(proof.seq)`), `asset_hash = asset.hash()`, `plugin_hashes = sorted_plugins.map(|p| p.hash())`, `account.hash = HashedAssetSchema { asset_hash, plugin_hashes }.hash()`; account data = `borsh(HashedAssetV1::new(hash))`, owner = program id. JS `decompress.test.ts` and `compress.test.ts` document the same computation but are `test.skip`ped except `compress.test.ts:90`, `:124`, `:160` and `decompress.test.ts:96`, `:128` (system-program / noop guards, which are processor-level and already covered).

### src/error.rs, src/entrypoint.rs, src/lib.rs

`error.rs`: the only region is `From<MplCoreError> for ProgramError` — covered. `entrypoint.rs`: 100% lines; `process_instruction` is flagged only in the unit-test build hash. `lib.rs`: no measurable regions. Nothing to do.

### Unit tests vs Mollusk

Best covered by pure unit tests inside the crate (fast, deterministic, no fixtures): `CompressionProof::new` + `From<CompressionProof> for AssetV1` round trip; `Compressible::hash` == `keccak(borsh)`; `HashablePluginSchema::compare_indeces`; `UpdateAuthority::key()` (three arms); `CollectionV1::increment_minted/increment_size/decrement_size` including the overflow/underflow arms; `SolanaAccount::load`/`save` error arms (build an `AccountInfo` over a local buffer); the `CoreAsset` impls; `assert_authority`/`assert_collection_authority` (dead/near-dead); the `check_*` matrices (documenting which validators are dead); the `validate_*` arms that no processor can reach (`None` plugin → `InvalidPlugin`/abstain, dead validators). Unit tests can also cover every branch of the reachable `validate_*` functions by calling them directly with a local `AccountInfo`, but that does not exercise the fp plumbing in `validate_asset_permissions`, so treat them as a complement, not a substitute.

Only reachable through Mollusk: everything that touches lamports or account size (`close_program_account`, `resize_or_reallocate_account` failure arm, `rebuild_account_state_from_proof_data`, `compress_into_account_space`), CPIs (`Wrappable::wrap`), the authority resolution and lifecycle-check plumbing (`validate_asset_permissions` collection branch, `validate_collection_permissions`, `resolve_pubkey_to_authorities*`, `is_valid_*_authority`, `save_flat_group`), and `increment_seq_and_save`.

### Notes: bugs, suspicious logic, security-relevant paths

1. **Harness (high impact):** 21/25 tests in `tests/account_ownership.rs` never execute the program (finding 1). Fixing the sentinel account and asserting error codes is a prerequisite for any of the coverage figures in that file to mean anything.
2. `MplCoreError::AssetIsFrozen` is defined (`src/error.rs:61`) but never emitted; frozen assets fail via the generic `rejected` exit (`src/utils/mod.rs:314` → `InvalidAuthority`). Tests should pin the actual code.
3. Error asymmetry: asset path returns `NoApprovals` when nothing approved (`src/utils/mod.rs:316`); collection path returns `InvalidAuthority` (`:469-471`).
4. `resolve_pubkey_to_authorities_collection` grants `Authority::Owner` to the collection update authority (`src/utils/mod.rs:516-518`, via `CollectionV1::owner()` returning `update_authority`, `src/state/collection.rs:390-392`). Owner-managed plugins are refused at add time for collections, but a crafted collection account with an `Authority::Owner` registry record would be satisfied by the UA.
5. Delegate semantics differ between paths: `is_valid_asset_authority`/`is_valid_collection_authority` ignore the `UpdateDelegate` record's own authority (`_plugin_authority`, `src/utils/mod.rs:588, 632`) and honour only `additional_delegates`, whereas `UpdateV2` (`src/processor/update.rs:212-229`, via `assert_collection_authority`) honours both. A delegate that can move an asset into a collection cannot add the same asset to a group.
6. `close_program_account` computes the refund from rent minimums, not the actual balance (`src/utils/account.rs:16-22`), and debits with an unchecked `-=` (`:29`). With `overflow-checks = true` (`Cargo.toml:6`) an account holding less than `minimum_balance(len)` would panic (`ProgramFailedToComplete`) rather than return an error; program-created accounts always hold at least that, and the surplus (the create fee) is intentionally left in the 1-byte `Uninitialized` account for `CollectV1` (`src/processor/collect.rs:44-52`). Pin both behaviours.
7. `CollectionV1::decrement_size` underflow (`src/state/collection.rs:363-370`) makes `BurnV1` and `UpdateV2`-remove fail with `NumericalOverflowError` for every asset of a collection whose `current_size` reached 0 while assets still reference it. `current_size` can be driven independently of real membership by `UpdateCollectionInfoV1` (Bubblegum signer, `saturating_sub`). Liveness footgun, not an exploit by arbitrary users.
8. `src/utils/mod.rs:174-178` is unreachable defence-in-depth (pre-empted by `resolve_pubkey_to_authorities`); harmless, but coverage will never reach it without reordering.
9. `UpdateAuthority::None.key()` returns `Pubkey::default()` (the system program id) and `validate_update` compares the signer to it (`src/state/asset.rs:264`). Harmless because the system program cannot sign; worth a unit assertion so a future refactor does not turn `None` into "anyone".
10. `DecompressV1` rebuilds and re-funds the account (`rebuild_account_state_from_proof_data`, payer-funded realloc) *before* `validate_asset_permissions` (`src/processor/decompress.rs:53-81`). Moot while the instruction always errors, but the order should be revisited if compression is ever enabled.

### Test plan

Sizes: S ≈ one instruction with existing-style fixtures, M ≈ new fixture or multi-step, L ≈ new harness support.

| Test case | Sets up | Instruction(s) | Expected | Source paths covered | New / port from | Size |
|---|---|---|---|---|---|---|
| unit_compression_proof_roundtrip | `AssetV1`, `CompressionProof::new(asset, 7, plugins)` | — | `AssetV1::from(proof)` == asset with `seq: Some(7)`; `hash()` == `keccak(borsh)` | asset.rs:406-415; compression_proof.rs:27-36; traits.rs:53-56 | new | S |
| unit_update_authority_key | three `UpdateAuthority` variants | — | `None` → `Pubkey::default()`, others → inner key | update_authority.rs:17-23 | new | S |
| unit_hashable_plugin_schema_sort | two schemas, indices `[1,0]` | — | sorted ascending | hashable_plugin_schema.rs:25-27 | new | S |
| unit_core_asset_impls | `AssetV1`, `CollectionV1` | — | `update_authority()`/`owner()` values | asset.rs:419-425; collection.rs:386-392 | new | S |
| unit_assert_authority_arms | local `AccountInfo`, asset + collection | — | Ok/`InvalidAuthority` per `Authority` arm | utils/mod.rs:37-85 | new | S |
| unit_collection_counters | `CollectionV1` with counters at 0 and `u32::MAX` | — | increments/decrement OK; overflow/underflow → `NumericalOverflowError` | collection.rs:343-370 | new | S |
| unit_solana_account_load_save_errors | local `AccountInfo` buffers | — | wrong key → `DeserializationError`; truncated → `DeserializationError`; wrong owner → `InvalidAccountOwner`; save into 1 byte → `SerializationError` | traits.rs:23-47 | new | S |
| unit_check_result_matrix | — | — | every `AssetV1::check_*`/`CollectionV1::check_*` value | asset.rs:71-143; collection.rs:57-129 | new | S |
| unit_validate_arms_direct | local `AccountInfo` for owner/UA/other; `FreezeDelegate` (owner-managed), `Attributes` (UA-managed); `None` plugin | — | approve/abstain/`InvalidPlugin` per arm incl. dead validators | asset.rs:146-378; collection.rs:132-340 | new | M |
| fix_account_ownership_sentinels | replace `Account::default()` under `MPL_CORE_ID` with `core_program_account()`; add `assert_error(code)` | existing Transfer/Burn/Update/UpdateCollection | specific error codes (`InvalidAccountOwner`, `DeserializationError`, `NoApprovals`, `InvalidAuthority`) | asset.rs:106-113, 272-283, 295; utils/mod.rs:281, 314; account.rs:10-35; traits.rs:27, 36-38 | port (existing Mollusk) | M |
| burn_asset_by_owner | valid asset, payer = owner | `BurnV1` | success; data `[0]`; refund to payer; surplus stays | asset.rs:106-108, 272-279; account.rs:10-35; utils/mod.rs:232-239 | JS burn.test.ts:26 | S |
| burn_asset_by_non_owner | valid asset, other signer | `BurnV1` | `NoApprovals` | asset.rs:281; utils/mod.rs:316 | JS burn.test.ts:45 | S |
| transfer_by_non_owner | valid asset, other signer | `TransferV1` | `NoApprovals` | asset.rs:295 | JS transfer.test.ts:37 | S |
| transfer_frozen_rejected | `FreezeDelegate{frozen:true}` | `TransferV1` by owner | `InvalidAuthority` | utils/mod.rs:281, 314 | JS plugins/asset/freeze.test.ts; existing Mollusk after fix | S |
| transfer_with_transfer_delegate | `TransferDelegate` authority `Address{d}`, signer `d` | `TransferV1` | success; owner-managed authorities reset | utils/mod.rs:280 | JS delegateTransfer.test.ts | S |
| transfer_permanent_delegate_force_approves | `PermanentTransferDelegate` (authority = UA) + frozen `FreezeDelegate`, signer UA | `TransferV1` | success | utils/mod.rs:284 | JS permanentTransfer.test.ts | S |
| update_asset_by_ua | valid asset | `UpdateV1` new name (longer, then shorter) | success; resized | asset.rs:111-113, 258-265; update_authority.rs:17-21; account.rs:45-70 | JS update.test.ts | S |
| update_asset_wrong_authority | valid asset, other signer | `UpdateV1` | `NoApprovals` | asset.rs:267 | JS update.test.ts:264 | S |
| update_asset_seq_some | crafted asset `seq: Some(3)` | `UpdateV1` | success; `seq == Some(4)` | asset.rs:63-64 | new | S |
| create_asset_in_collection | `CreateCollectionV2` then `CreateV2` with collection, signer = collection UA | `CreateV2` | success; `num_minted`/`current_size` = 1 | collection.rs:57-59, 132-139, 343-360; asset.rs:154; utils/mod.rs:192, 249-255, 495-497 | JS createCollection.test.ts:62 | M |
| create_asset_in_collection_wrong_ua | as above, other signer | `CreateV2` | `NoApprovals` | collection.rs:141 | JS createCollection.test.ts:135 | S |
| transfer_asset_in_collection_matrix | asset UA `Collection(c)`, collection fixture | `TransferV1` with c / without / with wrong / collection on stand-alone asset | success / `MissingCollection` / `InvalidCollection` / `InvalidCollection` | collection.rs:87-89; utils/mod.rs:180, 488-501 | JS transfer.test.ts:60/81/101 | M |
| burn_asset_in_collection | asset in collection, `current_size` 1; variant with 0 | `BurnV1` | success, `current_size` 0 / `NumericalOverflowError` | collection.rs:92-94, 363-370 | JS collectionSize.test.ts; new for underflow | S |
| add_plugin_asset_matrix | stand-alone asset, UA ≠ owner | `AddPluginV1` (owner+FreezeDelegate, UA+Attributes, owner+Attributes) | success / success / `NoApprovals` | asset.rs:76-78, 160-181 | JS addPlugin.test.ts | M |
| add_plugin_asset_in_collection | asset in collection, signer collection UA | `AddPluginV1` Attributes | success | collection.rs:62-64, 146-160; utils/mod.rs:249-255 | JS addPlugin.test.ts:292/485 | S |
| remove_plugin_matrix | asset with FreezeDelegate + Attributes; variant in collection | `RemovePluginV1` | success / `NoApprovals` | asset.rs:81-83, 184-203; collection.rs:67-69, 167-185 | JS removePlugin.test.ts:99/142 | M |
| approve_revoke_plugin_authority_matrix | asset with plugins; variant in collection | `ApprovePluginAuthorityV1`, `RevokePluginAuthorityV1` | success / `NoApprovals` | asset.rs:91-98, 216-255; collection.rs:77-84, 198-237 | JS approveAuthority/revokeAuthority | M |
| update_plugin_asset_and_collection | asset with Attributes; asset in collection | `UpdatePluginV1` | success | asset.rs:86-88; collection.rs:72-74 | JS updatePlugin.test.ts | S |
| execute_asset_in_collection | asset in collection, owner signs | `ExecuteV1` | success | collection.rs:127-129, 294-301 | JS execute.test.ts:312 | S |
| external_adapter_asset_in_collection | asset in collection with/without `AppData`; wrong signer variant | `AddExternalPluginAdapterV1`, `RemoveExternalPluginAdapterV1` | success / `NoApprovals` | asset.rs:352, 366; collection.rs:112-119, 304-330 | JS appData.test.ts | M |
| update_collection_matrix | collection fixture | `UpdateCollectionV1` by UA (name/uri/new UA); by other | success / `InvalidAuthority` | collection.rs:97-99, 260-271; utils/mod.rs:324-418, 469-474, 510-529 | JS update.test.ts collection cases | M |
| burn_collection_matrix | empty collection; non-empty; non-UA | `BurnCollectionV1` | success / `CollectionMustBeEmpty` / `InvalidAuthority` | account.rs:10-35; utils/mod.rs:324-474 | JS burnCollection.test.ts:20/39/64 | S |
| collection_plugin_ops | collection fixture | `AddCollectionPluginV1` (Attributes; then FreezeDelegate), `RemoveCollectionPluginV1`, `Approve/RevokeCollectionPluginAuthorityV1`, `UpdateCollectionPluginV1` | success / `InvalidAuthority` for owner-managed | collection.rs:62-84, 146-237; utils/mod.rs:379-389, 420-442 | JS addPlugin.test.ts:174/213, removePlugin.test.ts:78/184/307 | L |
| collection_plugin_reject_and_delegate | collection with `ImmutableMetadata`; collection with `UpdateDelegate{additional_delegates:[d]}` | `UpdateCollectionV1` | `InvalidAuthority` / success as `d` | utils/mod.rs:436-437, 469 | JS plugins/collection/immutableMetadata, updateDelegate | M |
| burn_collection_permanent_burn_force | empty collection with `PermanentBurnDelegate` (authority UA) | `BurnCollectionV1` | success | utils/mod.rs:439-441 | JS plugins/collection/permanentBurn | S |
| oracle_reject_matrix | `Oracle` adapter (Transfer/Update: CanReject) on asset, and on collection; oracle account = Rejected / missing / uninitialised | `TransferV1`, `UpdateV1`, `UpdateCollectionV1` | `InvalidAuthority` / `MissingExternalPluginAdapterAccount` / `UninitializedOracleAccount` | utils/mod.rs:204-212, 304, 306, 314, 444-467 | JS oracle.test.ts:319/405/502 | M |
| update_v2_move_to_collection_with_delegate | new collection with `UpdateDelegate` (authority `Address{d}`, additional `[e]`); signers d, e, UA, stranger | `UpdateV2` | success ×3 / `InvalidAuthority` | utils/mod.rs:65-85; collection.rs:353-357 | JS updateV2.test.ts:857/1388/583 | M |
| compress_runs_to_not_available | asset without plugins; asset with FreezeDelegate; SPL noop loaded | `CompressV1` by owner | `NotAvailable` (23) | asset.rs:116-118, 300-307; compression.rs:64-121; compression_proof.rs:27-36; hashed_asset.rs:20-27; hashable_plugin_schema.rs; traits.rs:53-56, 62-72 | JS compress.test.ts:90 | M |
| compress_by_non_owner | asset, other signer | `CompressV1` | `NoApprovals` | asset.rs:309 | JS compress.test.ts (skipped) | S |
| decompress_runs_to_not_available | crafted `HashedAssetV1` (hash computed in-test), proof with unsorted plugins `[Attributes idx1, FreezeDelegate idx0]`; variant with no plugins | `DecompressV1` by owner | `NotAvailable` | compression.rs:21-61, 124-152; asset.rs:121-123, 314-321, 406-415; hashed_asset.rs:35-37; hashable_plugin_schema.rs:25-27; account.rs:58-63 | JS decompress.test.ts (skipped) → new | L |
| decompress_wrong_proof | crafted hashed asset, proof with altered name | `DecompressV1` | `IncorrectAssetHash` | compression.rs:147-149 | new | S |
| decompress_by_non_owner | crafted hashed asset, other signer | `DecompressV1` | `NoApprovals` | asset.rs:323 | new | S |
| burn_hashed_asset_matrix | crafted hashed asset; SPL noop loaded | `BurnV1` with proof / without proof / without system program | `NotAvailable` / `MissingCompressionProof` / `MissingSystemProgram` | compression.rs:21-61, 124-152; traits.rs:62-72 | new | M |
| transfer_hashed_asset | crafted hashed asset | `TransferV1` with proof | `NotAvailable` | compression.rs (rebuild, verify) | new | S |
| realloc_insufficient_payer | asset, payer with 0 lamports | `AddPluginV1` | system program failure | account.rs:63 | new | S |
| load_key_mismatch_via_collection_slot | asset with UA `Collection(X)`, X is an `AssetV1` account | `TransferV1` with X | `DeserializationError` | traits.rs:27 | new | S |
| close_account_underfunded_crafted | asset account with lamports < `minimum_balance(len)` | `BurnV1` | `ProgramFailedToComplete` (pins note 6) | account.rs:20-29 | new | S |
| group_authority_helpers (owned by groups section) | group + assets (UA Address / Collection with & without collection in tx / None / UpdateDelegate additional delegate) and collections (UA / UpdateDelegate) | `CreateGroupV1`, `Add/RemoveAssetsToGroupV1`, `Add/RemoveCollectionsToGroupV1` | success / `InvalidAuthority` | utils/mod.rs:105-120, 553-648 | JS group.test.ts:48-179, createGroup.test.ts:152 | L |

### Harness prerequisites

- **Sentinel/assert fix** in `tests/account_ownership.rs` (finding 1); shared `assert_error(&InstructionResult, MplCoreError)` helper in `tests/common`.
- **Fixture module** (`tests/common/fixtures.rs`): `asset_account(owner, update_authority, seq, plugins: &[(Plugin, Authority)], adapters)` and `collection_account(update_authority, num_minted, current_size, plugins, adapters)` generalising `build_asset_with_plugins`/`build_collection_with_plugins` (which currently panic on any plugin other than the four freeze/permanent types); derive `PluginType` from the `Plugin` variant instead of a hard-coded match. Support `ExternalRegistryRecord`s for Oracle/AppData.
- **Instruction builders** for every instruction used above (hand-encoded discriminators as today, or add `clients/rust` as a path dev-dependency to use the generated builders — verify the `solana-program` version is compatible first).
- **SPL noop in Mollusk**: SBF mode — `spl_noop.so` fetched by `configs/scripts/program/dump.sh` into the program directory and `mollusk.add_program(&SPL_NOOP_ID, "spl_noop", &loader_v3)`; native mode — register a `Builtin` for `SPL_NOOP_ID` in `tests/common/mod.rs::native::mollusk()` whose entrypoint returns `Ok(())`. Both are needed because the tests run in both modes.
- **Hashed-asset helper**: `hashed_asset_fixture(asset: AssetV1 /*seq Some*/, plugins: Vec<HashablePluginSchema>) -> (Account, CompressionProof)` using `mpl_core_program::state::{Compressible, HashedAssetSchema, HashedAssetV1, CompressionProof}`.
- **Oracle account fixture**: raw account data laid out as `plugins/external/oracle.rs` expects (`OracleValidation::V1` with per-event `ExternalValidationResult`), owned by any key; plus the adapter init info with `base_address_config` = fixed address.
- **Payer/rent**: Mollusk's default rent sysvar; payer funded for reallocs; a 0-lamport payer variant for the failure arm.
- For `is_valid_*_authority` and `save_flat_group`: group account fixtures from the groups section.

### Estimate and ordering

About 9 unit tests (all S, ~150 lines of state/utils code) and ~35 Mollusk tests (≈18 S, ≈13 M, ≈4 L), many parameterised as small matrices. Suggested order by coverage gained per unit of effort:

1. Fix the 21 sentinel tests (M): immediately lights up burn/update/transfer-abstain, `close_program_account`, and the `Rejected`/`rejected` arms — with no new fixtures.
2. Unit tests (9 × S): `traits.rs` error arms, `compression_proof`/`hashable_plugin_schema`/`update_authority`, collection counters, `assert_*authority`, dead validators. Cheap and independent of the harness.
3. Collection fixture + asset-in-collection matrix (create, transfer, burn, add/remove plugin, execute) — unlocks the collection branch of `validate_asset_permissions` (~40 lines) and the reachable half of `collection.rs`.
4. Collection-targeted instructions (`UpdateCollectionV1`, `BurnCollectionV1`, `*CollectionPlugin*`) — all 150 lines of `validate_collection_permissions` plus `resolve_pubkey_to_authorities_collection` and the rest of `collection.rs`.
5. Asset plugin lifecycle matrix (add/remove/approve/revoke/update) — remaining `asset.rs` validators.
6. Compression (noop stub + hashed-asset helper): `compress`, `decompress`, hashed `burn`/`transfer` — 92 lines of `compression.rs` plus `traits.rs` `hash`/`wrap`, `hashed_asset.rs`, `From<CompressionProof>`.
7. Oracle reject matrix and `UpdateV2` move-with-delegate (`assert_collection_authority`).
8. Group authority helpers, coordinated with the groups section.


## 14. Test infrastructure, tooling, and process

Scope of this section: the harness, fixtures, dependency layout, CI gating and porting process needed to take `programs/mpl-core` from 29.43% lines / 38.11% functions / 27.68% regions to a defensible "full coverage". Per-area test lists are in the other sections; this one is about what they all need.

### 1. Current Mollusk harness and the three existing test files

**Harness (`tests/common/mod.rs`, 400 lines).** `core_mollusk()` returns a `Mollusk` with only mpl-core registered: in SBF mode via `Mollusk::new(&ID, "mpl_core_program")`, under `cfg(coverage)` or `MPL_CORE_NATIVE_PROGRAM=1` via `program_cache.add_builtin` around a `declare_process_instruction!` shim that serializes the instruction context with the SBF ABI, calls `mpl_core_program::entrypoint::process_instruction`, and commits the accounts back. Syscall stubs route logs, CPI (`sol_invoke_signed`, including owner/data/lamport write-back), all sysvar getters, return data and stack height into the live `InvokeContext`. Panics become `InstructionError::ProgramFailedToComplete`; native invocations are charged 1 CU. `core_program_account()` produces the right kind of executable account for the optional-account sentinel in each mode. This is solid and complete for single-program tests; nothing in it needs to change for the roadmap except adding extra builtins (section 5).

**How the three test files build state today.** All three (`account_ownership.rs` 25 tests, `agent_identity.rs` 14, `execution_delegate.rs` 14; 53 tests, 0.3 s total runtime, 1m11s instrumented build) work the same way:

- Account data is hand-laid out with borsh: `AssetV1::new(..)` -> `borsh::to_vec`, then `[AssetV1][PluginHeaderV1 (9 bytes)][plugin bytes...][PluginRegistryV1]` assembled by hand with manually computed offsets (`build_asset_with_plugins`, `build_collection_with_plugins`, `build_asset_with_agent_identity`). The plugin-type mapping inside `build_asset_with_plugins` is a 4-arm match that panics for any other plugin (`account_ownership.rs:150-156`), even though the program already provides `impl From<&Plugin> for PluginType` (`src/plugins/mod.rs:235`).
- Instructions are hand-encoded: literal discriminators (`vec![14u8, 0u8]` for TransferV1, `20u8` CreateV2, `21u8` CreateCollectionV2, `22u8` AddExternalPluginAdapterV1, `31u8` ExecuteV1) followed by field-by-field `BorshSerialize` of the args, and hand-written `AccountMeta` vectors with the optional-account sentinel (`MPL_CORE_ID`) filled in manually. Account order and discriminators are copied from the IDL by hand; a change to `src/instruction.rs` is only detected when a test fails at runtime.
- Signers: Mollusk performs no signature verification, so any `AccountMeta::new(k, true)` is a signer; `agent_identity.rs` uses this to make the agent-identity PDA "sign" without deploying mpl-agent-identity.
- Results are inspected via `result.program_result` only; nobody reads resulting accounts back (no test asserts on post-state data, lamports or plugin registry contents). `process_and_validate_instruction` / `Check` are unused.

**Duplicated helpers** (same or near-same code in 2-3 files):

| Helper | account_ownership | agent_identity | execution_delegate | Notes |
| --- | --- | --- | --- | --- |
| `ACCOUNT_LAMPORTS` const | x | x | x | 1 SOL |
| `payer_account()` | x | x | x | identical |
| `to_mollusk_accounts()` | x (adds system + noop loader-v3 account) | x (system only) | x | two variants; the noop variant adds an *account* but never registers noop as a program |
| `assert_failure()` | any-error | expects `ProgramError` | expects `ProgramError` | three different signatures |
| `assert_success()` | - | x | - | |
| `valid_asset_account()` | x | x | x | identical |
| `build_asset_with_agent_identity()` | - | x | x | identical 60-line copies |
| `empty_asset_account()` | - | x | - | |
| per-instruction builders | 5 | 5 | 1 | all hand-encoded |

**What the shared fixture library must provide** (proposed layout: keep `tests/common/mod.rs` as `harness`, add `tests/common/{accounts,ix,assert,read,programs,fixture}.rs`; everything `pub` and `#![allow(dead_code)]` as today):

1. `accounts.rs` — raw-layout account builders, generalized: `AssetSpec { owner, update_authority: UpdateAuthority, name, uri, seq, plugins: Vec<(Plugin, Authority)>, external: Vec<(ExternalPluginAdapter, ExternalRegistryRecordSpec)>, lamports, program_owner }` -> `Account`, and the same for `CollectionSpec` (with `num_minted`, `current_size`). Use `PluginType::from(&plugin)` / `ExternalPluginAdapterType::from(&adapter)` instead of the 4-arm match. Keep the raw builders even after the instruction path exists: they are the only way to produce corrupted registries, wrong discriminators, foreign-owned lookalikes, oversized offsets, and other adversarial layouts that the security fork specifically wants to exercise (the `MplCoreError::InvalidPluginRegistry`/`IncorrectAccount`/`DeserializationError` branches in `plugins/utils.rs`, `state/asset.rs`, `state/collection.rs`). Also: `payer_account(lamports)`, `empty_account()`, `oracle_account(offset, [create, transfer, burn, update])` (section 5), `execution_delegate_record(..)` built from `mpl_agent_tools::accounts::ExecutionDelegateRecordV1` rather than the hand-packed 104 bytes in `execution_delegate.rs:191-212`.
2. `fixture.rs` — the "create via instruction, then reuse" pattern on top of `MolluskContext<HashMap<Pubkey, Account>>` (`mollusk.with_context(HashMap::new())`). `MolluskContext` persists resulting accounts only when the instruction succeeded, auto-hydrates program and sysvar accounts, and defaults any account the test did not seed (`load_accounts_for_instructions`), so a test reads: `let f = Fixture::new(); let payer = f.fund(1 SOL); let coll = f.create_collection(CollectionArgs{..}); let asset = f.create_asset(AssetArgs{ collection: Some(coll), plugins: vec![..], ..}); let r = f.run(ix::transfer(..)); f.asset(&asset)`. This mirrors `_setupRaw.ts` (`createAsset`, `createCollection`, `createGroup`, `createAssetWithCollection`, `assertAsset`, `assertCollection`, `assertGroup`, `assertBurned`) one to one, which is exactly what makes porting the JS suite mechanical. Multi-step flows use `process_instruction_chain` (stops at first error) or `process_transaction_instructions` (single shared transaction context, atomic) when the test needs real transaction semantics.
3. `ix.rs` — thin wrappers over the generated Rust client builders (section 2): `ix::transfer(asset, collection, authority, payer, new_owner)`, `ix::add_plugin(..)`, etc., each returning `Instruction`. Keep a separate `ix::raw(discriminator, data, metas)` for deliberately malformed instructions (unknown discriminator, truncated args) that target `src/processor/mod.rs` decoding errors.
4. `assert.rs` — `assert_ok(&result)`, `assert_core_err(&result, MplCoreError::InvalidAuthority)` (compares against `ProgramError::Custom(e as u32)`, `src/error.rs:237`), `assert_program_err(&result, ProgramError::MissingRequiredSignature)`, `assert_instruction_err(.., InstructionError::ProgramFailedToComplete)` for panics/`unreachable!()`. Prefer Mollusk's `process_and_validate_instruction(&ix, &accounts, &[Check::err(MplCoreError::X.into()), Check::account(&k).lamports(n).build()])` for new tests; the helpers exist for readability in chains.
5. `read.rs` — `read_asset(&result_or_ctx, &key) -> AssetV1`, `read_collection`, `read_registry(&data) -> Option<(PluginHeaderV1, PluginRegistryV1)>` (header at `asset.len()`, registry at `header.plugin_registry_offset`), `read_plugin::<T>(&data, PluginType)` via the registry offsets, `read_external_adapter(..)`. Two options for implementation: program types (`AssetV1`, `PluginRegistryV1`, `Plugin` all implement `BorshDeserialize`) for exact on-chain layout checks, or the client's `mpl_core::Asset::from_bytes` (`clients/rust/src/hooked/asset.rs:56`) for a fully derived `PluginsList`. Provide both; use the program types in the default assert helpers so the tests do not depend on client deserialization correctness.
6. `assert_burned(&result, &key)`: after `close_program_account` (`src/utils/account.rs`) the account has 0 lamports, 1 byte of data equal to `Key::Uninitialized`, and is still owned by mpl-core. Mollusk's `Check::account(k).closed()` compares against `Account::default()` (empty data) and will **not** match; the helper must check `lamports == 0 && data == [0]`. Similarly `lamports_of(&result, &k)` and a `assert_lamport_flow(before, after, payer, asset, fee)` helper for `ExecuteV1`/`CollectV1`.
7. `programs.rs` — `core_program_account()` (existing), `noop_builtin()`, `recorder_builtin()` (section 5), `keyed_program_accounts()` that returns every registered program's keyed account so `to_mollusk_accounts` disappears.

### 2. Using the Rust client crate as a dev-dependency of the program tests

**Facts checked.**
- Workspace root `Cargo.toml` has members `clients/rust` (package `mpl-core` 0.12.0, lib `mpl_core`) and `programs/mpl-core` (package `mpl-core-program`, lib `mpl_core_program`); one shared `Cargo.lock`.
- `mpl-core` (client) does not depend on `mpl-core-program`, and none of the program's dependencies (`mpl-agent-tools 0.3.0`, `mpl-agent-identity 0.3.0`, `mpl-bubblegum 3.0.0`, `mpl-utils`) depend on a crates.io `mpl-core`, so a path dev-dependency creates no second copy of the client and no cycle. (Cargo would tolerate a dev-dependency cycle anyway, since dev-dependencies are not transitive.)
- Both crates use `solana-program 3.0.0` and borsh 1.x (`borsh 1.6.1` in the lock; the client's default `borsh-v1` feature). Generated client types are produced by Kinobi from the Shank IDL of this program, so `borsh::to_vec(&mpl_core::types::Plugin)` yields bytes that `mpl_core_program::plugins::Plugin::try_from_slice` accepts.
- Verified empirically: a scratch crate outside the workspace with `mpl-core-program = { path = ".../programs/mpl-core" }` and `mpl-core = { path = ".../clients/rust" }` runs `cargo check` cleanly, builds `TransferV1Builder::new()...instruction()`, asserts `ix.program_id == mpl_core_program::ID == mpl_core::ID`, and round-trips `Plugin::FreezeDelegate` client -> bytes -> program type. The client's own dependency set (`kaigan`, `rmp-serde`, `base64`, ...) is small and already in the lock file.
- The generated builders cover every instruction (54 variants in `src/instruction.rs`), fill the `MPL_CORE_ID` sentinel for optional accounts automatically (`transfer_v1.rs:40-60`), and support `add_remaining_account(s)` (needed for oracle/agent-identity/execute remaining accounts).

**Recommendation: adopt it.** Add to `programs/mpl-core/Cargo.toml`:

```toml
[dev-dependencies]
mpl-core = { path = "../../clients/rust" }   # generated instruction builders + hooked deserializers
```

and build every instruction in tests through `mpl_core::instructions::*Builder`, passing `mpl_core::types::*` args. Consequences to plan for:

- `configs/scripts/program/coverage.sh` must widen `IGNORE_REGEX` to `"programs/${p}/tests/|clients/rust/"`; otherwise the client crate (a workspace member compiled into the test binaries) appears in the report and dilutes the totals.
- `.github/file-filters.yml` `mpl_core_program` should gain `clients/rust/src/**` so a regenerated client that breaks the program tests triggers `test_programs`/`coverage`; and `generate_clients` in `main.yml` should keep running before program tests locally (`pnpm generate` after interface changes) — document in CLAUDE.md.
- Interface drift now fails at compile time (a renamed arg or reordered account changes the builder) instead of at runtime with a misleading error. That is the main maintenance win.
- Keep `ix::raw` (hand bytes) only for malformed-input tests.
- Alternative considered and rejected as the default: serializing `mpl_core_program::instruction::MplAssetInstruction::TransferV1(args)` directly gives correct discriminators without the client but still requires hand-written account metas (the Shank `#[account]` attributes are not queryable at runtime); acceptable as a fallback if the client dev-dependency is ever removed.

### 3. Running the existing `clients/rust/tests` under coverage

**What exists.** 14 files, **61** `#[tokio::test]` functions (not ~120; count from `grep -c tokio::test`), all gated `#![cfg(feature = "test-sbf")]` and run through `cargo test-sbf` (`configs/scripts/client/test-rust.sh`, `test-rust-client.yml`) against `solana-program-test 3.1.12` with `ProgramTest::new("mpl_core_program", mpl_core::ID, None)` loading the SBF ELF from `SBF_OUT_DIR`. They already use the generated builders and a `setup/mod.rs` helper module (`create_asset`, `assert_asset`, `airdrop`, ...). Areas: create (5), create_collection (5), *_with_external_plugins (14), add/update/remove external plugins on asset and collection (24), plugin_shrink_corruption (6, security regression), plugins (3), transfer (2), update_collection_info (1, fake Bubblegum signer), freeze_execute (1).

**Feasibility of a `cfg(coverage)` path.** `solana-program-test` supports native builtins through `processor!(process_instruction)` passed as the third argument of `ProgramTest::new`, and `.prefer_bpf(false)` forces the native processor even when `SBF_OUT_DIR` is set. Concretely: in `setup/mod.rs`

```rust
pub fn program_test() -> ProgramTest {
    #[cfg(coverage)]
    { let mut t = ProgramTest::new("mpl_core_program", mpl_core::ID,
          solana_program_test::processor!(mpl_core_program::entrypoint::process_instruction));
      t.prefer_bpf(false); return t; }
    #[cfg(not(coverage))]
    ProgramTest::new("mpl_core_program", mpl_core::ID, None)
}
```

which needs `mpl-core-program = { path = "../../programs/mpl-core" }` as a *dev*-dependency of the client (the mirror image of section 2; legal because dev-dependencies are not transitive, but both directions must stay dev-only) and `check-cfg` for `cfg(coverage)` in the client's `Cargo.toml`. `coverage.sh` then runs `cargo llvm-cov --no-report -p mpl-core --features test-sbf` after the program run and a single `cargo llvm-cov report` merges both profiles (llvm-cov merges `.profraw` across packages in the same target dir).

**Effort vs value.**

| | Plumb program-test under coverage | Port to Mollusk |
| --- | --- | --- |
| One-time work | ~1 day (dev-dep, cfg path, script, CI feature flag, ignore regex) | ~2-3 days for 61 tests once the fixture library exists (mechanical: same builders, replace `banks_client` with `Fixture`) |
| CI cost | compiles `solana-program-test` + tokio stack (several extra minutes, ~1 GB target); ~1-2 s per test | negligible (native, <10 ms per test) |
| Coverage gained | the same lines the ports would cover: mostly `write_external_plugin_adapter_data.rs` (0%), `add/update/remove_external_plugin_adapter.rs`, `create_collection.rs`, `plugins/utils.rs` resize paths — roughly +8-12 points of lines | same |
| Long-term | second harness to keep alive; async tests are slower to debug; program-test features not needed (no clock/warp use except none found) | one harness, deterministic, matches the rest of the roadmap |

**Recommendation.** Do the plumbing in the infrastructure milestone (it is cheap, immediately counts 61 behavioural tests, and keeps the security regression `plugin_shrink_corruption.rs` in the report), but freeze the suite: no new tests there, and each file is deleted from the coverage run once its Mollusk port lands (tracked in the porting table). If the extra CI minutes turn out to matter more than the temporary points, skip the plumbing and port directly; the ports are on the M1/M2 path anyway.

### 4. The JS AVA suite as the specification

**Inventory.** 79 files, 707 `test(...)` declarations (a few `test.skip`/`test.serial`; the brief's 682 counts runnable tests). Helpers in `clients/js/test/_setupRaw.ts` map directly onto the fixture library in section 1. The JS tests are the de-facto behavioural spec: every processor and plugin has positive and negative cases with named `MplCoreError`s (`t.throwsAsync(.., { name: 'InvalidAuthority' })`).

**Porting methodology.**

1. *One Rust module per JS file, same tree.* Do **not** create one Cargo integration-test file per JS file: each `tests/*.rs` is a separate crate that links the whole Solana runtime (tens of seconds of link time each; 79 of them would dominate CI). Use a single integration crate `tests/it/main.rs` with `mod transfer; mod plugins { pub mod asset { pub mod royalties; ... } }` mirroring `clients/js/test/**` exactly (`plugins/asset/royalties.test.ts` -> `tests/it/plugins/asset/royalties.rs`). Fold the three existing files in as `it/account_ownership.rs`, `it/agent_identity.rs`, `it/execution_delegate.rs` (or leave them; they are three crates, not seventy-nine).
2. *Same test names.* The Rust fn is the JS title in snake_case, and every test carries a marker doc comment `/// JS: transfer.test.ts :: it can transfer an asset as the owner`. Excluded tests get a marker in the mapping file instead.
3. *Mapping table* at `programs/mpl-core/tests/PORTING.md` (or `.csv`): `js_file | js_test | rust_test | status (ported|excluded|pending) | note`. A 30-line script (`configs/scripts/program/porting-status.sh`) extracts all `test(` titles from `clients/js/test`, all `/// JS:` markers from `programs/mpl-core/tests`, and the `excluded` rows, and prints the pending list and a percentage; run it in `test-programs.yml` as an informational step and fail only if a marker references a JS test that no longer exists (keeps the table honest when JS tests are renamed). No hand-maintained counts.
4. *Assertion translation rules* (documented once in `PORTING.md`): `t.throwsAsync({name})` -> `assert_core_err`; `fetchAsset` -> `f.asset(&k)`; `assertAsset({..plugins})` -> `read_plugin::<T>`; `assertBurned` -> `assert_burned`; `umi.rpc.getAccount(k).lamports` -> `lamports_of`; missing signer -> set `is_signer=false` on the meta (Mollusk skips signature verification, so a JS test that fails at the transaction level for a missing signature must be expressed as a program-level `MissingRequiredSignature`/`InvalidAuthority`); `generateSigner` -> `Pubkey::new_unique()`; airdrop -> `Account { lamports, .. }`.
5. *Priority order* (by uncovered lines in `INDEX.txt` weighted by security relevance):
   1. `update.test.ts`, `updateV2.test.ts` (48 tests) -> `processor/update.rs` 271 lines at 0%, `state/asset.rs`/`collection.rs` update paths.
   2. `addPlugin`, `removePlugin`, `updatePlugin`, `approveAuthority`, `revokeAuthority` (71) -> five processors totalling ~690 lines at 0%, plus most of `plugins/lifecycle.rs`.
   3. `burn`, `burnCollection`, `transfer`, `create`, `createCollection`, `signers/*` (73) -> `burn.rs` 0%, remaining `create*.rs`, `transfer.rs`.
   4. `plugins/asset/*`, `plugins/collection/*` (~230) -> `update_delegate.rs`, `verified_creators.rs`, `royalties.rs`, `autograph.rs`, `freeze_*`, `permanent_*`, `edition.rs`, `bubblegum_v2.rs`, and the per-plugin arms of `lifecycle.rs`.
   5. `externalPlugins/*` (75; oracle alone is 47) -> `external_plugin_adapters.rs` 329 missed, `write_external_plugin_adapter_data.rs` 194 at 0%, `oracle.rs` 92 at 0%, `app_data.rs`, `linked_app_data.rs`, `data_section.rs`.
   6. `group*`, `closeGroup`, `updateGroupAuthority`, `groupComplexRelations`, `groupsPluginBlocking` (38) -> the nine `*group*.rs` processors and `groups_plugin_utils.rs`, ~830 lines at 0%.
   7. `collect`, `compress`, `decompress` (22) -> `collect.rs`, `compress.rs`, `decompress.rs`, `utils/compression.rs` (needs the noop builtin, section 5).
   8. `execute.test.ts`, `instructions/*` legacy delegate/revoke/freeze (38) — `execute.rs` is already 87% covered; low marginal gain.

**Exclude from the porting target** (JS-only by nature; ~60 tests):

| JS file | Tests | Why |
| --- | --- | --- |
| `compute.test.ts` | 2 | measures CUs of group stress transactions via RPC. Under native coverage every invocation costs 1 CU (`NATIVE_COMPUTE_UNITS`), so a Mollusk port is only meaningful in SBF mode: optionally add as `#[cfg_attr(coverage, ignore)]` SBF-only tests with `Check::compute_units`, but do not count them toward the porting target. |
| `getProgram.test.ts`, `helps/authority|lifecycle|plugin|state|fetch.test.ts` | ~36 | Umi SDK helper logic (client-side state derivation, lifecycle simulation, plugin key conversions). No program code involved. |
| `asset.test.ts`, `collection.test.ts` | 4 | `getProgramAccounts` filters through RPC. |
| `info.test.ts` | 2 (1 skipped) | prints account size. |
| `sdkv1.test.ts` | 9 | SDK v1 wrapper; program behaviour duplicated elsewhere. Exception: port the two "create with all plugins" tests (lines 38, 270) as a single Mollusk test, since they are the only place every plugin is initialised at once. |
| `accountOwnership.test.ts` | 37 | already mirrored by `tests/account_ownership.rs`; verify 1:1 in the table rather than re-port. |

Also note validator-dependent tests that *are* portable but need harness support: `plugins/*/bubblegumV2.test.ts` and `update_collection_info` (the program only checks a fixed address `CbNY3...` or `mpl_bubblegum::ID` as signer, so mark that pubkey as signer in the meta), `externalPlugins/oracle.test.ts` (JS writes oracle accounts through the `mpl_core_oracle_example` program; in Mollusk write the `OracleValidation` bytes directly, section 5).

### 5. Special harness needs (Mollusk 0.11 capabilities checked in the vendored crate)

- **Registered programs.** `Mollusk::default()` registers only the system program and BPF loaders v2/v3 (`program.rs:239`, `all-builtins` feature adds compute-budget/loader-v4/vote/zk). Anything the program CPIs into must be added via `program_cache.add_builtin(Builtin{..})` (host function, works in both modes) or `add_program_with_loader(_and_elf)` (real ELF, SBF only). Unregistered program ids passed as instruction accounts get a stub executable account (`get_account_fallbacks`) but invoking them fails.
- **SPL Noop** (`Wrappable::wrap`, `src/state/traits.rs:60-72`, called from `compress.rs:77`, `burn.rs:75`, `transfer.rs:135` whenever a `compression_proof` is supplied): add a builtin `Builtin { program_id: SPL_NOOP_ID, name: "spl_noop", entrypoint: NoopEntrypoint::vm }` with `declare_process_instruction!(NoopEntrypoint, 0, |_| Ok(()))`. Deterministic, no ELF needed. The existing `to_mollusk_accounts` pushes a loader-v3 *account* for noop without registering it; replace with `create_keyed_account_for_builtin_program(&SPL_NOOP_ID, "spl_noop")`. (Loading `programs/.bin/spl_noop.so` via `dump.sh` is possible but adds a network-dependent build step.)
- **ExecuteV1 CPI target** (`execute.rs:97-160`): the system program already works (existing `execute_system_transfer_*` tests). Add a `recorder` builtin whose entrypoint copies the instruction it received (program id, account keys, signer/writable flags, data) into a `thread_local!` and returns a configurable result. It lets tests assert that `process_execute` rewrote the `asset_signer` meta to `is_signer=true`, that the `ExecutionDelegateRecordV1` was stripped before the CPI, that non-delegate remaining accounts are preserved, and that a failing target propagates its error. Enable the `inner-instructions` feature of `mollusk-svm` in `[dev-dependencies]` to also get `result.inner_instructions` and `Check::inner_instruction_count(n)` for CPI counting (noop wrap, execute fee transfer + target call).
- **Lifecycle hooks:** the program does **not** CPI into `hooked_program` (`LifecycleHook`/`LinkedLifecycleHook::validate_*` are `abstain!()`; there is no `invoke` under `src/plugins/`). No stub needed; tests only need the adapter add/update/remove paths and the `ExternalPluginAdapterKey::LifecycleHook(hooked_program)` matching.
- **Oracle:** read-only. `Oracle::validate_helper` (`oracle.rs:77-123`) locates the account by `base_address` or `ExtraAccount::derive`, slices at `results_offset`, and borsh-decodes `OracleValidation` (`Uninitialized` | `V1 { create, transfer, burn, update }` of `ExternalValidationResult`). Fixture: `oracle_account(offset: usize, v: Option<[ExternalValidationResult; 4]>) -> Account` (owner irrelevant to the program) plus `extra_account_address(&ExtraAccount, base, ctx)` helpers for the `Preconfigured{Program,Collection,Owner,Recipient,Asset}`, `CustomPda` and `Address` variants so the 47 oracle tests can pass the right remaining account. Error branches (`MissingExternalPluginAdapterAccount`, `InvalidOracleAccountData` for short data / bad offset, `UninitializedOracleAccount`) are trivial with raw bytes.
- **mpl-agent-tools / mpl-agent-identity:** no CPI from core. `ExecutionDelegateRecordV1` accounts: serialize `mpl_agent_tools::accounts::ExecutionDelegateRecordV1` (crate already a dependency) with owner `mpl_agent_tools::ID`. Agent-identity PDA: keep the "PDA marked as signer" trick; add a negative case with `is_signer=false`.
- **Bubblegum:** `mpl_bubblegum::ID` and the fixed `CbNY3...` signer are only compared as addresses/signers; mark them as signers in the metas. No Bubblegum ELF needed.
- **Sysvars:** `Mollusk::default().sysvars` has `Clock` (slot 0), default `Rent`, epoch schedule, etc.; the native stubs answer `Rent::get()`/`Clock::get()` from the sysvar cache. Tests that need time use `mollusk.sysvars.clock.unix_timestamp = ..` or `mollusk.warp_to_slot(n)`; instructions that take a sysvar *account* use `mollusk.sysvars.keyed_account_for_rent_sysvar()`. Rent-exemption checks in the program use the same default `Rent`, so `Check::all_rent_exempt()` is a cheap invariant to add to every positive test.
- **Signers:** any `AccountMeta` with `is_signer=true`, no keypairs; multi-signer flows are just several flagged metas. Tests for "missing signer" must flip the flag explicitly.
- **Lamports / closure / data checks:** `result.get_account(&k)`, `Check::account(&k).lamports(n) / .owner(&o) / .data(&bytes) / .data_slice(off, &bytes) / .space(n) / .rent_exempt() / .closed()` (`closed` == `Account::default()`; see section 1 item 6 for why burned mpl-core accounts need a custom check). Lamport movements (execute fee, collect fees, burn refunds, realloc top-ups) are asserted by diffing seed vs `resulting_accounts`.
- **Compute units:** native mode charges 1 CU per invocation; assert CUs only when `!common::native_program_enabled()`.
- **Panics / `unreachable!()`:** surface as `ProgramResult::UnknownError(InstructionError::ProgramFailedToComplete)` in both modes; use `Check::instruction_err(..)`.
- **Logs:** use `Mollusk::new_debuggable` / set `mollusk.logger` when debugging; native `sol_log` goes to the same `LogCollector`.
- **Stateful flows:** `MolluskContext` (`with_context(HashMap)`) for create-then-mutate sequences; failed instructions do not persist, which is what negative tests want.

### 6. CI and process

**Current state.** `main.yml` calls `coverage.yml` only when `programs/mpl-core/**` (or the workflow files) change; the job builds with `cargo llvm-cov --no-report -p mpl-core-program`, produces lcov/json/html/summary, posts a sticky PR comment (same-repo PRs only) and uploads the artifact. `COVERAGE_MIN_LINES` is supported by `coverage.sh` but never set, so nothing gates. Minor drift: `.github/.env` `RUST_VERSION=1.88.0` while `rust-toolchain.toml` pins 1.89.0 (rustup follows the toolchain file, so it works, but align them). Instrumented build here: 1m11s; tests: <1 s.

**Thresholds and ratcheting.**
- Replace the ad-hoc jq check with a committed `programs/mpl-core/coverage-thresholds.json` (`{"lines": 28, "functions": 37, "regions": 26, "file_lines": 0}`) that `coverage.sh` feeds to `cargo llvm-cov report --fail-under-lines/--fail-under-functions/--fail-under-regions/--fail-under-file-lines` (all exist in cargo-llvm-cov 0.9.1). Gate on regions as well as lines: regions are the stable-toolchain proxy for branch coverage.
- Ratchet rule (documented in CONTRIBUTING.md and enforced by the script printing a warning): if measured − threshold > 2 points for any metric, the PR must bump the file. Mollusk tests are deterministic, so the threshold can sit at `floor(measured) − 1` with no flakiness margin. Thresholds only go up; lowering requires a reviewer-approved note in the exclusions file (section 7).
- Per-file thresholds: `--fail-under-file-lines` is global (any file below it fails). Use it from M2 onwards (40 -> 60 -> 90). For files legitimately below the floor keep an allowlist in the thresholds JSON that the script checks with `jq '.data[0].files[]'` before invoking the global flag; every entry cites the exclusion record.
- Consider `--fail-uncovered-functions <N>` at M4: functions are the easiest metric to drive to ~100% and catch dead code.

**Branch coverage.** `cargo llvm-cov --branch` / `--mcdc` need a nightly toolchain (`-Z coverage-options=branch`). The repo pins stable 1.89 via `rust-toolchain.toml`; a nightly job would run `RUSTUP_TOOLCHAIN=nightly cargo llvm-cov --branch ...` and is exposed to nightly breakage of the Solana dependency tree. Recommendation: not a gate. Add an optional `workflow_dispatch`/weekly job that publishes the branch HTML as an artifact for reviewers of security-sensitive files (`lifecycle.rs`, `utils/mod.rs` validation, `plugins/utils.rs`), and gate on regions on stable. The same nightly job is where `#[cfg_attr(coverage_nightly, coverage(off))]` annotations (section 7) take effect.

**Sticky PR comment.** Keep the marker/update logic (it is correctly restricted to the bot's own comment). Make it useful for review:
1. Delta vs base: on pushes to `main`, upload `coverage.json` as an artifact (already done) and in PR runs download the latest `main` artifact (`gh run download` on the newest successful `Main` run; needs `actions: read`) to print `Lines 31.2% (+1.8)`, and the threshold headroom (`gate 28 -> ok`).
2. Changed-files table: `git diff --name-only origin/${base}...HEAD -- programs/mpl-core/src` joined with per-file numbers from `coverage.json`, plus uncovered ranges for those files from `cargo llvm-cov report --show-missing-lines` (write it to `missing.txt` next to `summary.txt`). Reviewers see immediately whether new code is tested.
3. Porting progress line from `porting-status.sh` (`ported 312/645 JS tests`).
4. Keep the full per-file table collapsed, as now.

**Other process items.**
- `test-programs.yml` runs `cargo clippy --all-targets`, so all new test code must be clippy-clean; keep the `#![allow(dead_code)]` on the fixture crate only.
- Add `pnpm programs:coverage -- --open`-style docs and the `MPL_CORE_NATIVE_PROGRAM=1 cargo test` fast loop to CONTRIBUTING (already in CLAUDE.md).
- Add `clients/rust/**` to the `mpl_core_program` path filter if section 2 is adopted, and run `pnpm generate` check-in drift detection before program tests (the `generate_clients` job already exists; make `test_programs` depend on it or on a lightweight "generated code up to date" check).

**Milestones and gates** (per-area detail is in the other sections; numbers are line coverage on the stable report):

| Milestone | Target | Contents (high level) | Gate added |
| --- | --- | --- | --- |
| M0 infrastructure | 29 -> ~30 | fixture library, client dev-dep, noop + recorder builtins, single `tests/it` crate, `PORTING.md` + status script, thresholds file, ignore-regex fix, optional program-test-under-coverage plumbing (+8-12 if done) | lines/functions/regions thresholds at current − 1; ratchet rule |
| M1 core instructions | 50 | ports of update/updateV2, add/remove/update plugin, approve/revoke authority, burn/burnCollection, transfer, create/createCollection, signers | lines ≥ 50; no `src/processor/*.rs` at 0% except groups/compress/collect |
| M2 plugins and external adapters | 70 | plugins/asset/*, plugins/collection/*, externalPlugins/* (oracle with raw accounts), write adapter data, `lifecycle.rs` arms | lines ≥ 70, regions ≥ 60, `--fail-under-file-lines 40` with allowlist |
| M3 groups, compression, collect, adversarial | 85 | group processors, collect, compress/decompress (noop), execute recorder tests, corrupted-account/defensive-branch tests from raw builders, delete program-test duplicates | lines ≥ 85, functions ≥ 90, file floor 60 |
| M4 closure | 95+ lines / 97+ functions / 90+ regions | unit tests for `plugins/utils.rs`, `state/*`, `utils/mod.rs` helpers; exclusion policy applied and reviewed; nightly branch report | file floor 90 with allowlist; `--fail-uncovered-functions` ≤ allowlist size |

### 7. What "full coverage" realistically means here, and an exclusion policy

`programs/mpl-core/src` has **no** `cfg(feature)`-gated code, no `#[deprecated]` items, and `entrypoint.rs`/`error.rs` are already fully covered, so the unreachable set is small and enumerable:

| Category | Locations | Reachable? | Disposition |
| --- | --- | --- | --- |
| `ForceApproved => unreachable!()` guards (external adapters never force-approve) | `utils/mod.rs:309,465`; `external_plugin_adapters.rs:424,426`; `lifecycle.rs:816` | no (would require a plugin implementation returning `ForceApproved` from an external check) | accepted exclusion |
| Exhaustiveness `_ => unreachable!()` after prior matching | `transfer.rs:142`; `lifecycle.rs:705,779`; `add_external_plugin_adapter.rs:76` (`DataSection` cannot be added directly), `:190` (adapters without a data-init step) | no by construction | accepted exclusion; consider making the matches exhaustive in a reviewed refactor, not as a coverage exercise |
| Defensive `panic!("Missing function parameters ...")` | `utils/mod.rs:165,363` | only by an internal caller bug | accepted exclusion |
| Compression-only types and paths | `state/compression_proof.rs`, `state/hashable_plugin_schema.rs`, `utils/compression.rs`, `hashed_asset.rs` | yes with the noop builtin (`CompressV1`/`DecompressV1`, transfer/burn with proof) | in scope (M3) |
| Corrupted-account / deserialization-failure branches | `plugins/utils.rs`, `plugin_registry.rs`, `state/asset.rs`, `state/collection.rs`, `utils/account.rs` | yes via raw-layout builders | in scope, and the highest-value part for a security fork |
| Trait impls used only by clients (`Display`, `From` conversions, `PrintProgramError`) | scattered; `state/update_authority.rs` (6 lines, 0%) | yes, via unit tests | in scope (M4 unit tests); do not exclude |
| Genuinely dead code found while porting | to be discovered | - | flag in the security review; remove or justify, never allowlist silently |

**Policy.**
1. Numeric definition of "full": ≥95% lines, ≥97% functions, ≥90% regions on the stable `cargo llvm-cov` report with `--ignore-filename-regex` limited to non-source paths (`programs/mpl-core/tests/`, `clients/rust/`). Source files are never hidden via the regex.
2. Every accepted exclusion is recorded in `programs/mpl-core/COVERAGE_EXCLUSIONS.md` as `file:line(s) | symbol | reason | reviewer | date`, and the sum of excluded lines is what separates the measured number from 100%. Reviewers can then answer "why is this line not covered?" from the file alone.
3. In-source marking: `#[cfg_attr(coverage_nightly, coverage(off))]` on whole functions that are accepted exclusions (the `coverage` attribute is nightly-only; the program's `Cargo.toml` already declares `cfg(coverage_nightly)` in `check-cfg`, so it is warning-free on stable and honoured by the optional nightly job). Branch-level arms (`unreachable!()`) cannot be annotated on any toolchain; they live in the exclusions file only.
4. No code changes purely to raise the number: replacing `unreachable!()` with exhaustive matches or deleting dead branches goes through the normal security-review path with its own justification.
5. Re-validate exclusions at each milestone: the nightly branch report plus `--show-missing-lines` output are diffed against the exclusions file; any uncovered line not in the file is either tested or added with a reason.
