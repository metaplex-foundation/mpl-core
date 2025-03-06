#!/bin/bash

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
# go to parent folder
cd $(dirname $(dirname $(dirname $SCRIPT_DIR)))
WORKING_DIR=$(pwd)

# command-line input
ARGS=$*

# js umi client tests folder
cd ${WORKING_DIR}/clients/js

pnpm install && pnpm format:fix

# js web3js2 client tests folder
cd ${WORKING_DIR}/clients/js/web3js2

pnpm install && pnpm format:fix