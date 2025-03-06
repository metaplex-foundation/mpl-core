#!/bin/bash

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
# go to parent folder
cd $(dirname $(dirname $(dirname $SCRIPT_DIR)))
WORKING_DIR=$(pwd)

# command-line input
ARGS=$*

# js umi client tests folder
cd ${WORKING_DIR}/clients/js/umi

pnpm install && pnpm format

# js web3js2 client tests folder
cd ${WORKING_DIR}/clients/js/web3js2

pnpm install && pnpm format