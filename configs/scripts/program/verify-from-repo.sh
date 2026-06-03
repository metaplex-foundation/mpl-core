#!/bin/bash

# Wraps `solana-verify verify-from-repo` for this workspace.
#
# Usage:
#   ./verify-from-repo.sh --program-id <PUBKEY> [--url <RPC_URL>] [--keypair <PATH>]
#                         [--commit-hash <SHA>] [--library-name <LIB>]
#                         [--mount-path <DIR>] [--workspace-path <DIR>]
#                         [--repo-url <URL>] [--skip-prompt] [--] [extra args]
#
# Defaults:
#   --library-name : mpl_core_program (matches programs/mpl-core/Cargo.toml)
#   --mount-path   : . (repo root; solana-verify 0.4.15 expects the mounted
#                    workspace to contain Cargo.lock)
#   --workspace-path : omitted by default, so solana-verify uses --mount-path
#   --repo-url     : derived from `git remote get-url origin`, normalized to https://
#   --commit-hash  : current HEAD if inside a clean repo, otherwise required
#
# The on-chain hash is fetched from the cluster targeted by --url and compared
# against a deterministic docker build of the resolved commit.

set -euo pipefail

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
ROOT_DIR=$(dirname "$(dirname "$(dirname "${SCRIPT_DIR}")")")
cd "${ROOT_DIR}"

PROGRAM_ID=""
RPC_URL=""
KEYPAIR=""
COMMIT_HASH=""
LIBRARY_NAME="mpl_core_program"
MOUNT_PATH="."
WORKSPACE_PATH=""
REPO_URL=""
SKIP_PROMPT=()
PASSTHROUGH=()

while [ $# -gt 0 ]; do
    case "$1" in
        --program-id)
            PROGRAM_ID="$2"
            shift 2
            ;;
        --url|-u)
            RPC_URL="$2"
            shift 2
            ;;
        --keypair|-k)
            KEYPAIR="$2"
            shift 2
            ;;
        --commit-hash)
            COMMIT_HASH="$2"
            shift 2
            ;;
        --library-name)
            LIBRARY_NAME="$2"
            shift 2
            ;;
        --mount-path)
            MOUNT_PATH="$2"
            shift 2
            ;;
        --workspace-path)
            WORKSPACE_PATH="$2"
            shift 2
            ;;
        --repo-url)
            REPO_URL="$2"
            shift 2
            ;;
        --skip-prompt|-y)
            SKIP_PROMPT=(--skip-prompt)
            shift
            ;;
        --)
            shift
            PASSTHROUGH=("$@")
            break
            ;;
        -h|--help)
            sed -n '3,20p' "$0"
            exit 0
            ;;
        *)
            PASSTHROUGH+=("$1")
            shift
            ;;
    esac
done

if [ -z "${PROGRAM_ID}" ]; then
    echo "error: --program-id is required" >&2
    exit 1
fi

# Resolve repo URL from git remote when the caller did not supply one. We
# normalize SSH-style remotes to https://, since solana-verify expects an
# https URL it can hand to the OtterSec API for remote verification.
if [ -z "${REPO_URL}" ]; then
    if ! origin=$(git remote get-url origin 2>/dev/null); then
        echo "error: --repo-url not provided and no git origin found" >&2
        exit 1
    fi

    if [[ "${origin}" =~ ^git@github\.com:(.+)$ ]]; then
        REPO_URL="https://github.com/${BASH_REMATCH[1]}"
    elif [[ "${origin}" =~ ^ssh://git@github\.com[:/](.+)$ ]]; then
        REPO_URL="https://github.com/${BASH_REMATCH[1]}"
    else
        REPO_URL="${origin}"
    fi

    if [[ "${REPO_URL}" =~ ^https://github\.com/.+ ]] && [[ ! "${REPO_URL}" =~ \.git$ ]]; then
        REPO_URL="${REPO_URL}.git"
    fi
fi

if [ -z "${COMMIT_HASH}" ]; then
    if ! COMMIT_HASH=$(git rev-parse HEAD 2>/dev/null); then
        echo "error: --commit-hash not provided and HEAD could not be resolved" >&2
        exit 1
    fi
fi

CMD=(solana-verify verify-from-repo)
if [ -n "${RPC_URL}" ]; then
    CMD+=(--url "${RPC_URL}")
fi
if [ -n "${KEYPAIR}" ]; then
    CMD+=(--keypair "${KEYPAIR}")
fi
if [ -n "${WORKSPACE_PATH}" ]; then
    CMD+=(--workspace-path "${WORKSPACE_PATH}")
fi
CMD+=(
    --program-id "${PROGRAM_ID}"
    --library-name "${LIBRARY_NAME}"
    --mount-path "${MOUNT_PATH}"
    --commit-hash "${COMMIT_HASH}"
    ${SKIP_PROMPT[@]+"${SKIP_PROMPT[@]}"}
    "${REPO_URL}"
)

if [ ${#PASSTHROUGH[@]} -gt 0 ]; then
    CMD+=("${PASSTHROUGH[@]}")
fi

echo "+ ${CMD[*]}"
exec "${CMD[@]}"
