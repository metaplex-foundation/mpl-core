#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
OUTPUT="./programs/.bin"
# saves external programs binaries to the output directory
"${SCRIPT_DIR}/dump.sh" "${OUTPUT}"

# Oracle program used for tests and is only deployed to devnet
"${SCRIPT_DIR}/dump_oracle_example.sh" "${OUTPUT}"

# go to parent folder
cd "$(dirname "$(dirname "$(dirname "${SCRIPT_DIR}")")")"

if [ -z "${PROGRAMS+x}" ]; then
    PROGRAMS="$(grep "^PROGRAMS=" .github/.env | cut -d '=' -f 2)"
fi

# default to input from the command-line
ARGS=("$@")

# command-line arguments override env variable
if [ ${#ARGS[@]} -gt 0 ]; then
    PROGRAMS="[\"${1}\"]"
    shift
    ARGS=("$@")
fi

PROGRAM_LIST=()
while IFS= read -r program; do
    PROGRAM_LIST+=("${program}")
done < <(echo "${PROGRAMS}" | jq -r '.[]')

# creates the output directory if it doesn't exist
mkdir -p "${OUTPUT}"

WORKING_DIR=$(pwd)
export SBF_OUT_DIR="${WORKING_DIR}/${OUTPUT}"

# Resolve a program directory's `[lib].name` from its Cargo.toml. solana-verify
# builds workspace members by their library name, not the package or directory
# name. We fall back to the conventional `<dir-with-_>_program` form used in
# this template so newly-added programs work without extra wiring.
resolve_library_name() {
    local program_dir="$1"
    local cargo_toml="${WORKING_DIR}/programs/${program_dir}/Cargo.toml"
    local lib_name=""

    if [ -f "${cargo_toml}" ]; then
        lib_name=$(awk '
            /^\[lib\]/ { in_lib = 1; next }
            /^\[/      { in_lib = 0 }
            in_lib && /^[[:space:]]*name[[:space:]]*=/ {
                gsub(/[" ]/, "", $0)
                split($0, parts, "=")
                print parts[2]
                exit
            }
        ' "${cargo_toml}")
    fi

    if [ -z "${lib_name}" ]; then
        lib_name="${program_dir//-/_}_program"
    fi

    echo "${lib_name}"
}

for p in "${PROGRAM_LIST[@]}"; do
    LIB_NAME=$(resolve_library_name "${p}")
    echo "Building verified program: ${p} (library: ${LIB_NAME})"

    # `solana-verify build` runs the build inside a deterministic docker image
    # so the resulting .so hash matches a remote verification of the same
    # source. The output lands at target/deploy/<lib>.so under the workspace
    # root regardless of which program was selected.
    # ${ARGS[@]+"${ARGS[@]}"} guards against empty-array expansion under
    # `set -u` on Bash < 4.4 (see PR #27 review).
    solana-verify build --library-name "${LIB_NAME}" ${ARGS[@]+"${ARGS[@]}"}

    cp "${WORKING_DIR}/target/deploy/${LIB_NAME}.so" "${WORKING_DIR}/${OUTPUT}/${LIB_NAME}.so"
done
