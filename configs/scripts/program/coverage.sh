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
# Usage:
#   ./configs/scripts/program/coverage.sh [program] [-- extra cargo test args]
#
# Environment:
#   PROGRAM              Single program to cover (overrides PROGRAMS).
#   PROGRAMS             JSON array of programs (default: from .github/.env).
#   COVERAGE_OUTPUT_DIR  Where reports are written (default: ./coverage).
#   COVERAGE_MIN_LINES   Optional minimum line coverage percentage; the script
#                        fails when a program is below it.

set -euo pipefail

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

    # Only report on the program's own sources, not on the test harness.
    IGNORE_REGEX="programs/${p}/tests/"

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
            | tee "${PROGRAM_OUTPUT_DIR}/summary.txt"
    )

    if [ -n "${COVERAGE_MIN_LINES:-}" ]; then
        LINES_PCT="$(jq -r '.data[0].totals.lines.percent' "${PROGRAM_OUTPUT_DIR}/coverage.json")"
        if ! jq -e --argjson min "${COVERAGE_MIN_LINES}" '.data[0].totals.lines.percent >= $min' \
            "${PROGRAM_OUTPUT_DIR}/coverage.json" >/dev/null; then
            echo "Line coverage for ${p} is ${LINES_PCT}%, below the required ${COVERAGE_MIN_LINES}%" >&2
            FAILED=1
        fi
    fi

    echo "Reports for ${p} written to ${PROGRAM_OUTPUT_DIR}"
done <<< "${PROGRAMS}"

exit "${FAILED}"
