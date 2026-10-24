# Metaplex Core for Solana Kit

A native Solana Kit 8 client for the Metaplex Core program. It provides generated
instruction builders, codecs, errors, PDA derivation, and account fetch functions.

## Install

```sh
pnpm add @metaplex-foundation/mpl-core-kit @solana/kit
```

Requires Node.js 20.18 or newer and Kit 8.5 or newer within major version 8.
TypeScript declarations are tested with TypeScript 5.9. The package uses ESM.

## Create and fetch an asset

Supply Kit `TransactionSigner` values for `asset` and `payer`, then add the
instruction to your Kit transaction message and sign/send it normally.

```ts
import {
  fetchAssetV1,
  getCreateV2Instruction,
} from '@metaplex-foundation/mpl-core-kit';

const instruction = getCreateV2Instruction({
  asset,
  name: 'My asset',
  payer,
  plugins: [
    {
      authority: null,
      plugin: { __kind: 'FreezeDelegate', fields: [{ frozen: false }] },
    },
  ],
  uri: 'https://example.com/asset.json',
});

// After confirming the transaction:
const account = await fetchAssetV1(rpc, asset.address);
const freeze = account.data.plugins.find(
  ({ plugin }) => plugin.__kind === 'FreezeDelegate'
);
console.log(freeze?.plugin, freeze?.authority);
```

`fetchAssetV1`, `fetchCollectionV1`, and `fetchGroupV1`, including their batch and
optional-account variants, use the complete account decoder. The generated Kit
program plugin, `mplCoreProgram()`, uses the same codecs.

Decoded accounts expose:

- `externalPluginAdapters`: registry records with the decoded `adapter`, parsed
  `data`, and original `rawData` when the record has a data region.
- `pluginHeader`: the on-chain plugin registry pointer, when present.
- `plugins`: registry records with the decoded `plugin`, its authority, and offset.

Addresses, bigint values, enums, options, and discriminated unions retain Kit's
native representations. External binary data remains bytes; JSON and MessagePack
data are parsed. Empty JSON data is `undefined`; empty MessagePack data is `null`.
Invalid JSON, UTF-8, or MessagePack payloads preserve `rawData` and expose the parse
error as `dataError` on the adapter, with `data` set to `undefined`. Other account
data and batch results remain available. Malformed account layouts and unsupported
plugin discriminants still throw.

An account's plugin list contains its own stored plugins. Fetch its collection to
inspect collection plugins and apply the program's inheritance rules.

## Account codecs

`getAssetV1Codec`, `getCollectionV1Codec`, and `getGroupV1Codec` decode complete
accounts and encode account data with recalculated registry offsets. When encoding
external adapter data, supply `rawData`; the parsed `data` property is a decoded
view. Instruction builders use their own generated argument codecs.

## Development

From this directory:

```sh
pnpm install --frozen-lockfile
pnpm build
pnpm format
pnpm lint
pnpm test
pnpm test:package
```

From the repository root, build and start the existing local validator:

```sh
pnpm install --frozen-lockfile
pnpm programs:build
pnpm validator
```

Then run `pnpm test:integration` from this directory. Tests default to local RPC
`http://127.0.0.1:8899` and subscriptions `ws://127.0.0.1:8900`; use `RPC_URL` and
`WS_URL` to select another local validator.

Decoder fixtures cover every current internal plugin and external adapter variant,
all three account types, external data schemas, and malformed account boundaries.
Integration tests exercise asset/collection creation, plugin authority changes,
plugin updates/removal, metadata updates, transfers, burns, external data writes,
and data shrinking. They use the current locally built Core program. The package
test installs the tarball into a separate consumer and checks declarations and
runtime imports. Advanced instructions are generated from the same IDL; see the
test files for the operations exercised against the validator.

## Generation

Run `pnpm generate:kit` from the repository root after changing the IDL or Kit
visitors. `pnpm generate:clients` also runs the existing Umi/Rust generation.
Generated files must be reproduced by `configs/codama.cjs`; edit that configuration
or the handwritten hooks instead of editing generated files.

The generator follows the existing Kinobi instruction defaults. Its account
export bridge supplies the custom codecs omitted by Codama 2.8's account barrel.

## Attribution

This client builds on the Codama integration started by Tony Boyle in
[PR #250](https://github.com/metaplex-foundation/mpl-core/pull/250). The
[beeman](https://github.com/beeman/mpl-core-kit) and
[Obrera](https://github.com/obrera/mpl-core-kit-lib) implementations informed the
integration work. Generated code comes from this repository's IDL; account plugin
layout follows the existing Core clients and on-chain program.
