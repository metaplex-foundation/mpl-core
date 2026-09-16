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

## Threshold reductions

| date | metric | from | to | reason | reviewer |
| --- | --- | --- | --- | --- | --- |
