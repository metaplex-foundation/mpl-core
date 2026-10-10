# Coverage exclusions

Lines of `programs/mpl-core/src` that are accepted as not coverable because no
test can reach them. This file is not an inventory of every uncovered line: the
rest of the shortfall is ordinary coverage debt, which the thresholds and the
ratchet rule exist to drive down. Reviewers should be able to answer "why can
this line never be covered?" from this file alone.

## Policy

1. "Full coverage" means at least 95% lines, 97% functions and 90% regions on
   the stable `cargo llvm-cov` report, with `--ignore-filename-regex` limited to
   test code: the paths outside `src` (`programs/mpl-core/tests/`,
   `clients/rust/`) and, should the in-crate unit tests ever be split out into
   dedicated `src/**/tests.rs` modules, those files too. Program code is never
   hidden through the regex.
2. Every accepted exclusion is recorded below as
   `file:line(s) | symbol | reason | reviewer | date`. These lines are the part
   of the shortfall that is permanent: no test can reach them, so they would
   still be uncovered at the end of the roadmap. Any other uncovered line is
   coverage debt and belongs in a future milestone, not in this table.
3. In-source marking: whole functions that are accepted exclusions carry
   `#[cfg_attr(coverage_nightly, coverage(off))]` (the `coverage` attribute is
   nightly-only; the crate's `check-cfg` already declares `cfg(coverage_nightly)`,
   so it is warning-free on stable and honoured by the optional nightly job).
   Branch-level arms such as `unreachable!()` cannot be annotated on any
   toolchain and live in this file only.
4. No code changes purely to raise the number: replacing `unreachable!()` with
   exhaustive matches or deleting dead branches goes through the normal
   security-review path with its own justification.
5. Exclusions are re-validated at each milestone: the `--show-missing-lines`
   output (`coverage/mpl-core/missing.txt`) and the nightly branch report are
   diffed against this file, and any uncovered line not listed here is either
   tested or added with a reason.
6. Lowering a value in `coverage-thresholds.json` is only allowed with a
   reviewer-approved note in this file explaining why (see "Code coverage" in
   the root CONTRIBUTING.md). Thresholds otherwise only go up.

Genuinely dead code found while porting is not an exclusion: it is flagged in
the security review and removed or justified, never allowlisted silently.

## Accepted exclusions

| file:line(s) | symbol | reason | reviewer | date |
| --- | --- | --- | --- | --- |
| `src/utils/mod.rs:165` | `validate_asset_permissions` | Defensive `panic!` when `external_plugin_adapter_validate_fp` and `hookable_lifecycle_event` disagree about being `Some`. Every call site in `src/processor` passes a consistent pair, so no instruction can produce the mismatch; the arm exists to catch a future caller that gets it wrong. | Claude (pending review) | 2026-09-14 |
| `src/utils/mod.rs:309` | `validate_asset_permissions` | `ValidationResult::ForceApproved => unreachable!()` on the result of `validate_external_plugin_adapter_checks`. `force_approve!` is used only by `PermanentBurnDelegate` and `PermanentTransferDelegate`, which are internal plugins evaluated on the `validate_plugin_checks` path; no external plugin adapter can return `ForceApproved`. | Claude (pending review) | 2026-09-14 |
| `src/utils/mod.rs:363` | `validate_collection_permissions` | Same defensive `panic!` as `:165`, on the collection path. | Claude (pending review) | 2026-09-14 |
| `src/utils/mod.rs:465` | `validate_collection_permissions` | Same external-adapter `ForceApproved => unreachable!()` as `:309`, on the collection path. | Claude (pending review) | 2026-09-14 |
| `src/plugins/external_plugin_adapters.rs:424` | `ExternalPluginAdapter::validate_update_external_plugin_adapter` | `(ForceApproved, _)` arm of the `(base_result, result)` match. `base_result` is assigned only `Approved` or `Pass` a few lines above, so this pair cannot occur. | Claude (pending review) | 2026-09-14 |
| `src/plugins/external_plugin_adapters.rs:426` | `ExternalPluginAdapter::validate_update_external_plugin_adapter` | `(_, ForceApproved)` arm of the same match. No external plugin adapter returns `ForceApproved`, and the remaining `base_result` value for this arm (`Rejected`) is itself never produced. | Claude (pending review) | 2026-09-14 |
| `src/plugins/lifecycle.rs:816` | `validate_external_plugin_adapter_checks` | `ValidationResult::ForceApproved => unreachable!()` over adapter validation results; see `src/utils/mod.rs:309`. | Claude (pending review) | 2026-09-14 |
| `src/plugins/lifecycle.rs:705` | `validate_plugin_checks` | Exhaustiveness `_ => unreachable!()` after matching `Key::CollectionV1` and `Key::AssetV1`. The `checks` map is populated only by `PluginRegistryV1::check_registry`, which is called with those two keys and no other. Making the match exhaustive is a reviewed refactor, not a coverage exercise. | Claude (pending review) | 2026-09-14 |
| `src/plugins/lifecycle.rs:779` | `validate_external_plugin_adapter_checks` | Same exhaustiveness arm for `external_checks`, which is populated only by `PluginRegistryV1::check_adapter_registry` with `Key::CollectionV1` / `Key::AssetV1`. | Claude (pending review) | 2026-09-14 |
| `src/processor/transfer.rs:142` | `transfer` | Exhaustiveness `_ => unreachable!()` on the account key when reserializing. The earlier `match key` returns `IncorrectAccount` for every key other than `HashedAssetV1` / `AssetV1`, and the `HashedAssetV1` branch returns `NotAvailable` before reaching this point. | Claude (pending review) | 2026-09-14 |
| `src/processor/add_external_plugin_adapter.rs:76` | `add_external_plugin_adapter` | `ExternalPluginAdapterInitInfo::DataSection(_) => unreachable!()` while resolving the adapter authority. The guard above the match returns `CannotAddDataSection` for that variant first. | Claude (pending review) | 2026-09-14 |
| `src/processor/add_external_plugin_adapter.rs:190` | `add_collection_external_plugin_adapter` | `DataSection` / `AgentIdentity` arms of the same authority match. The guard above returns `CannotAddDataSection` and `InvalidPluginAdapterTarget` respectively before the match runs. | Claude (pending review) | 2026-09-14 |

All twelve lines above were re-read against the source at the time of the entry
and the enclosing function and pre-empting guard verified by hand. None of them
is a whole-function exclusion, so no `#[cfg_attr(coverage_nightly, coverage(off))]`
attribute is currently needed in `src`; every entry is a single branch arm inside
an otherwise covered function.

## Threshold reductions

| date | metric | from | to | reason | reviewer |
| --- | --- | --- | --- | --- | --- |
