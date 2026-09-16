#!/bin/bash
#
# Generates a code coverage report for the Rust programs.
#
# Uses `cargo llvm-cov` (LLVM source-based coverage) to run the program's unit
# tests and Mollusk integration tests with instrumentation. `cargo llvm-cov`
# sets `cfg(coverage)`, which makes the Mollusk harness in
# `programs/<program>/tests/common` execute the host-compiled program instead
# of the SBF ELF, so the lines exercised by the Mollusk tests are attributed
# back to `programs/<program>/src` alongside the unit tests.
#
# Requirements:
#   rustup component add llvm-tools-preview
#   cargo install cargo-llvm-cov --locked
#   jq (already needed by the other program scripts)
#
#   A `file_lines` threshold additionally needs cargo-llvm-cov 0.8.6 or newer,
#   the release that added `--fail-under-file-lines`.
#
# Usage:
#   ./configs/scripts/program/coverage.sh [program] [-- extra cargo test args]
#
# Environment:
#   PROGRAM              Single program to cover (overrides PROGRAMS).
#   PROGRAMS             JSON array of programs (default: from .github/.env).
#   COVERAGE_OUTPUT_DIR  Where reports are written (default: ./coverage).
#   COVERAGE_MIN_LINES   Optional minimum line coverage percentage. Overrides
#                        the `lines` value of the thresholds file below.
#
# Thresholds (the coverage gate):
#   `programs/<program>/coverage-thresholds.json`, when present, holds the
#   minimum `lines`, `functions` and `regions` percentages (integers) and an
#   optional per-file `file_lines` floor. They are passed to the final
#   `cargo llvm-cov report` invocation as `--fail-under-lines`,
#   `--fail-under-functions`, `--fail-under-regions` and
#   `--fail-under-file-lines`, so the script exits non-zero when the program
#   is below any of them. The `allowlist` array lists source files (paths
#   relative to the repository root) that are exempt from the file floor;
#   when it is non-empty the per-file check is done here with jq instead of
#   the global `--fail-under-file-lines` flag, which has no exemptions.
#
#   Thresholds only go up (see CONTRIBUTING.md, "Code coverage"): when a
#   measured metric exceeds its threshold by more than 2 points the script
#   prints a ratchet warning asking for the file to be bumped.
#
# Outputs (per program, under COVERAGE_OUTPUT_DIR/<program>/):
#   lcov.info, coverage.json, html/, summary.txt (the per-file table),
#   missing.txt (the table plus uncovered line ranges, from
#   `--show-missing-lines`) and gate.json (thresholds in effect and result).

set -euo pipefail

# Number of points a metric may exceed its threshold by before the ratchet
# warning fires.
RATCHET_SLACK=2

# --- threshold helpers -------------------------------------------------------
#
# These only depend on jq and are kept free of side effects so they can be
# exercised on their own: `source coverage.sh` loads the functions and returns
# without running anything.

# load_thresholds <thresholds.json>
#
# Populates THRESHOLD_LINES, THRESHOLD_FUNCTIONS, THRESHOLD_REGIONS,
# THRESHOLD_FILE_LINES (empty when unset) and THRESHOLD_ALLOWLIST (array).
# A missing file leaves every threshold empty. COVERAGE_MIN_LINES, when set,
# overrides the `lines` value.
load_thresholds() {
    local file="$1"
    THRESHOLD_LINES=""
    THRESHOLD_FUNCTIONS=""
    THRESHOLD_REGIONS=""
    THRESHOLD_FILE_LINES=""
    THRESHOLD_ALLOWLIST=()

    if [ -f "${file}" ]; then
        if ! jq -e 'type == "object"' "${file}" >/dev/null 2>&1; then
            echo "Thresholds file ${file} is not a JSON object." >&2
            return 1
        fi
        # Only numeric values count; anything else is treated as unset.
        THRESHOLD_LINES="$(jq -r 'if (.lines | type) == "number" then .lines else empty end' "${file}")"
        THRESHOLD_FUNCTIONS="$(jq -r 'if (.functions | type) == "number" then .functions else empty end' "${file}")"
        THRESHOLD_REGIONS="$(jq -r 'if (.regions | type) == "number" then .regions else empty end' "${file}")"
        THRESHOLD_FILE_LINES="$(jq -r 'if (.file_lines | type) == "number" then .file_lines else empty end' "${file}")"
        while IFS= read -r entry; do
            [ -n "${entry}" ] && THRESHOLD_ALLOWLIST+=("${entry}")
        done < <(jq -r '(.allowlist // []) | .[] | strings' "${file}")
    fi

    if [ -n "${COVERAGE_MIN_LINES:-}" ]; then
        if ! [[ "${COVERAGE_MIN_LINES}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
            echo "COVERAGE_MIN_LINES must be a number, got '${COVERAGE_MIN_LINES}'." >&2
            return 1
        fi
        THRESHOLD_LINES="${COVERAGE_MIN_LINES}"
    fi
}

# FILE_LINES_MIN_VERSION
#
# `--fail-under-file-lines` landed in cargo-llvm-cov v0.8.6. Older versions
# reject it as an unknown argument, which is an opaque way to learn that the
# `file_lines` threshold needs a newer tool.
FILE_LINES_MIN_VERSION="0.8.6"

# require_file_lines_support
#
# Fails with an actionable message when the loaded thresholds ask for the
# `--fail-under-file-lines` flag but the installed cargo-llvm-cov predates it.
# Every other threshold works on any version, so this only runs for the flag
# that needs the floor.
require_file_lines_support() {
    [ -n "${THRESHOLD_FILE_LINES}" ] || return 0
    [ "${#THRESHOLD_ALLOWLIST[@]}" -eq 0 ] || return 0

    local version
    version="$(cargo llvm-cov --version 2>/dev/null | head -1 | awk '{print $2}')"
    if [ -z "${version}" ]; then
        echo "Could not read the cargo-llvm-cov version; ${FILE_LINES_MIN_VERSION} or newer is required for the file_lines threshold." >&2
        return 1
    fi
    # Sort the two versions and see whether the minimum comes first.
    if [ "$(printf '%s\n%s\n' "${FILE_LINES_MIN_VERSION}" "${version}" | sort -V | head -1)" != "${FILE_LINES_MIN_VERSION}" ]; then
        echo "The file_lines threshold needs --fail-under-file-lines, added in cargo-llvm-cov ${FILE_LINES_MIN_VERSION}; found ${version}." >&2
        echo "Upgrade with: cargo install cargo-llvm-cov --locked" >&2
        echo "Or drop file_lines from the thresholds file." >&2
        return 1
    fi
}

# threshold_args
#
# Prints, one per line, the `--fail-under-*` arguments for the thresholds
# loaded by load_thresholds. The file floor is only passed as a flag when the
# allowlist is empty (see the header).
threshold_args() {
    [ -n "${THRESHOLD_LINES}" ] && printf '%s\n' "--fail-under-lines" "${THRESHOLD_LINES}"
    [ -n "${THRESHOLD_FUNCTIONS}" ] && printf '%s\n' "--fail-under-functions" "${THRESHOLD_FUNCTIONS}"
    [ -n "${THRESHOLD_REGIONS}" ] && printf '%s\n' "--fail-under-regions" "${THRESHOLD_REGIONS}"
    if [ -n "${THRESHOLD_FILE_LINES}" ] && [ "${#THRESHOLD_ALLOWLIST[@]}" -eq 0 ]; then
        printf '%s\n' "--fail-under-file-lines" "${THRESHOLD_FILE_LINES}"
    fi
    return 0
}

# describe_thresholds
#
# One human-readable line naming the thresholds in effect.
describe_thresholds() {
    local parts=()
    [ -n "${THRESHOLD_LINES}" ] && parts+=("lines >= ${THRESHOLD_LINES}%")
    [ -n "${THRESHOLD_FUNCTIONS}" ] && parts+=("functions >= ${THRESHOLD_FUNCTIONS}%")
    [ -n "${THRESHOLD_REGIONS}" ] && parts+=("regions >= ${THRESHOLD_REGIONS}%")
    if [ -n "${THRESHOLD_FILE_LINES}" ]; then
        parts+=("every file >= ${THRESHOLD_FILE_LINES}% lines (${#THRESHOLD_ALLOWLIST[@]} allowlisted)")
    fi
    if [ "${#parts[@]}" -eq 0 ]; then
        echo "none (no thresholds file and COVERAGE_MIN_LINES unset)"
    else
        local out="${parts[0]}" i
        for ((i = 1; i < ${#parts[@]}; i++)); do
            out+=", ${parts[i]}"
        done
        echo "${out}"
    fi
}

# allowlist_json
#
# The allowlist as a JSON array (`[]` when empty).
allowlist_json() {
    if [ "${#THRESHOLD_ALLOWLIST[@]}" -eq 0 ]; then
        echo '[]'
    else
        printf '%s\n' "${THRESHOLD_ALLOWLIST[@]}" | jq -R . | jq -sc '.'
    fi
}

# files_below_floor <coverage.json>
#
# Prints `<file> <percent>` for every source file under the file floor that is
# not allowlisted. Prints nothing when no floor is set.
files_below_floor() {
    local json="$1"
    [ -n "${THRESHOLD_FILE_LINES}" ] || return 0
    jq -r --argjson floor "${THRESHOLD_FILE_LINES}" --argjson allow "$(allowlist_json)" '
        .data[0].files[]
        | select(.summary.lines.percent < $floor)
        | select(([.filename | endswith($allow[])] | any) | not)
        | "\(.filename) \(.summary.lines.percent | . * 100 | round / 100)%"' "${json}"
}

# ratchet_warnings <coverage.json>
#
# Prints a warning for every metric whose measured value exceeds its threshold
# by more than RATCHET_SLACK points.
ratchet_warnings() {
    local json="$1"
    local metric threshold measured
    for metric in lines functions regions; do
        case "${metric}" in
            lines) threshold="${THRESHOLD_LINES}" ;;
            functions) threshold="${THRESHOLD_FUNCTIONS}" ;;
            regions) threshold="${THRESHOLD_REGIONS}" ;;
        esac
        [ -n "${threshold}" ] || continue
        measured="$(jq -r ".data[0].totals.${metric}.percent | . * 100 | round / 100" "${json}")"
        if jq -e --argjson t "${threshold}" --argjson slack "${RATCHET_SLACK}" \
            ".data[0].totals.${metric}.percent - \$t > \$slack" "${json}" >/dev/null; then
            echo "ratchet: ${metric} coverage is ${measured}%, more than ${RATCHET_SLACK} points above the ${threshold}% threshold; bump coverage-thresholds.json"
        fi
    done
}

# write_gate_json <coverage.json> <passed 0|1> <output>
write_gate_json() {
    local json="$1" passed="$2" out="$3"
    jq -n \
        --arg lines "${THRESHOLD_LINES}" \
        --arg functions "${THRESHOLD_FUNCTIONS}" \
        --arg regions "${THRESHOLD_REGIONS}" \
        --arg file_lines "${THRESHOLD_FILE_LINES}" \
        --argjson allowlist "$(allowlist_json)" \
        --argjson passed "$([ "${passed}" = 1 ] && echo true || echo false)" \
        --slurpfile cov "${json}" '
        def num: if . == "" then null else tonumber end;
        {
          thresholds: {
            lines: ($lines | num),
            functions: ($functions | num),
            regions: ($regions | num),
            file_lines: ($file_lines | num),
            allowlist: $allowlist
          },
          measured: ($cov[0].data[0].totals | {
            lines: .lines.percent, functions: .functions.percent, regions: .regions.percent
          }),
          passed: $passed
        }' > "${out}"
}

# When sourced, stop here so the helpers above can be tested in isolation.
if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
    return 0
fi

# --- main --------------------------------------------------------------------

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)

# go to parent folder
cd "$(dirname "$(dirname "$(dirname "${SCRIPT_DIR}")")")"
WORKING_DIR=$(pwd)

if ! cargo llvm-cov --version >/dev/null 2>&1; then
    echo "cargo-llvm-cov is not installed." >&2
    echo "Install it with: rustup component add llvm-tools-preview && cargo install cargo-llvm-cov --locked" >&2
    exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "jq is required to select programs and read the coverage summary." >&2
    exit 1
fi

if [ -n "${PROGRAM:-}" ]; then
    PROGRAMS='["'"${PROGRAM}"'"]'
fi

if [ -z "${PROGRAMS:-}" ]; then
    PROGRAMS="$(grep "^PROGRAMS=" .github/.env | cut -d '=' -f 2)"
fi

# command-line arguments override env variable
ARGS=("$@")
if [ $# -gt 0 ] && [ "$1" != "--" ]; then
    PROGRAMS="[\"${1}\"]"
    shift
    ARGS=("$@")
fi

PROGRAMS=$(printf '%s\n' "${PROGRAMS}" | jq -c '.[]' | sed 's/"//g')

OUTPUT_DIR="${COVERAGE_OUTPUT_DIR:-${WORKING_DIR}/coverage}"
mkdir -p "${OUTPUT_DIR}"

FAILED=0

while IFS= read -r p; do
    PROGRAM_DIR="${WORKING_DIR}/programs/${p}"
    PACKAGE="$(grep -m1 '^name = ' "${PROGRAM_DIR}/Cargo.toml" | sed -E 's/name = "(.*)"/\1/')"
    PROGRAM_OUTPUT_DIR="${OUTPUT_DIR}/${p}"
    mkdir -p "${PROGRAM_OUTPUT_DIR}"

    # Only report on the program's own sources: not on the test harness, and
    # not on the Rust client crate, which the tests pull in as a path
    # dev-dependency for its generated instruction builders.
    IGNORE_REGEX="programs/${p}/tests/|clients/rust/"

    load_thresholds "${PROGRAM_DIR}/coverage-thresholds.json"
    require_file_lines_support
    GATE_ARGS=()
    while IFS= read -r arg; do
        [ -n "${arg}" ] && GATE_ARGS+=("${arg}")
    done < <(threshold_args)
    echo "==> Coverage thresholds in effect for ${p}: $(describe_thresholds)"

    echo "==> Collecting coverage for ${PACKAGE}"
    (
        cd "${PROGRAM_DIR}"
        cargo llvm-cov clean --workspace
        cargo llvm-cov --no-report --package "${PACKAGE}" ${ARGS[@]+"${ARGS[@]}"}

        cargo llvm-cov report --package "${PACKAGE}" \
            --ignore-filename-regex "${IGNORE_REGEX}" \
            --lcov --output-path "${PROGRAM_OUTPUT_DIR}/lcov.info"
        cargo llvm-cov report --package "${PACKAGE}" \
            --ignore-filename-regex "${IGNORE_REGEX}" \
            --json --output-path "${PROGRAM_OUTPUT_DIR}/coverage.json"
        cargo llvm-cov report --package "${PACKAGE}" \
            --ignore-filename-regex "${IGNORE_REGEX}" \
            --html --output-dir "${PROGRAM_OUTPUT_DIR}"
        cargo llvm-cov report --package "${PACKAGE}" \
            --ignore-filename-regex "${IGNORE_REGEX}" \
            --show-missing-lines > "${PROGRAM_OUTPUT_DIR}/missing.txt"
    )

    # The summary is the gated invocation: `cargo llvm-cov` prints the table
    # and then exits non-zero when a `--fail-under-*` threshold is not met.
    # Collect that status instead of letting `set -e` abort, so the remaining
    # checks and the other programs still run; the script fails at the end.
    GATE_STATUS=0
    (
        cd "${PROGRAM_DIR}"
        cargo llvm-cov report --package "${PACKAGE}" \
            --ignore-filename-regex "${IGNORE_REGEX}" \
            ${GATE_ARGS[@]+"${GATE_ARGS[@]}"} \
            | tee "${PROGRAM_OUTPUT_DIR}/summary.txt"
    ) || GATE_STATUS=$?

    PASSED=1
    if [ "${GATE_STATUS}" -ne 0 ]; then
        echo "Coverage gate FAILED for ${p}: below the thresholds in effect ($(describe_thresholds))" >&2
        PASSED=0
    fi

    if [ -n "${THRESHOLD_FILE_LINES}" ] && [ "${#THRESHOLD_ALLOWLIST[@]}" -gt 0 ]; then
        BELOW_FLOOR="$(files_below_floor "${PROGRAM_OUTPUT_DIR}/coverage.json")"
        if [ -n "${BELOW_FLOOR}" ]; then
            echo "Coverage gate FAILED for ${p}: files below the ${THRESHOLD_FILE_LINES}% line floor and not allowlisted:" >&2
            printf '  %s\n' "${BELOW_FLOOR}" >&2
            PASSED=0
        fi
    fi

    ratchet_warnings "${PROGRAM_OUTPUT_DIR}/coverage.json"
    write_gate_json "${PROGRAM_OUTPUT_DIR}/coverage.json" "${PASSED}" "${PROGRAM_OUTPUT_DIR}/gate.json"

    if [ "${PASSED}" -eq 1 ]; then
        echo "Coverage gate passed for ${p} ($(describe_thresholds))"
    else
        FAILED=1
    fi

    echo "Reports for ${p} written to ${PROGRAM_OUTPUT_DIR}"
done <<< "${PROGRAMS}"

exit "${FAILED}"
