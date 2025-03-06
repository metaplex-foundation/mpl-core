import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  customPdaAllSeedsInit,
  fixedAccountInit,
  MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
} from '@metaplex-foundation/mpl-core-oracle-example';
import {
  canBurn,
  canTransfer,
  CheckResult,
  findOracleAccount,
  LifecycleValidationError,
  OracleInitInfoArgs,
  validateBurn,
  validateTransfer,
  validateUpdate,
  ExternalValidationResult,
} from '../../src';
import { createUmi } from '../_setupRaw';
import { createAsset, createAssetWithCollection } from '../_setupSdk';

test('it can detect transferrable on basic asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  t.assert(canTransfer(owner.publicKey, asset));
  t.is(
    await validateTransfer(umi, { authority: owner.publicKey, asset }),
    null
  );
});

test('it can detect non transferrable from frozen asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'FreezeDelegate',
        frozen: true,
      },
    ],
  });

  t.assert(!canTransfer(owner.publicKey, asset));
  t.is(
    await validateTransfer(umi, { authority: owner.publicKey, asset }),
    LifecycleValidationError.AssetFrozen
  );
});

test('it can detect transferrable on asset with transfer delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'TransferDelegate',
        authority: {
          type: 'Address',
          address: delegate.publicKey,
        },
      },
    ],
  });

  t.assert(canTransfer(delegate.publicKey, asset));
  t.is(
    await validateTransfer(umi, { authority: delegate.publicKey, asset }),
    null
  );
});

test('it can detect transferrable from permanent transfer', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'PermanentTransferDelegate',
        authority: {
          type: 'Address',
          address: delegate.publicKey,
        },
      },
    ],
  });

  t.assert(canTransfer(delegate.publicKey, asset));
  t.is(
    await validateTransfer(umi, { authority: delegate.publicKey, asset }),
    null
  );
});

test('it can detect transferrable when frozen with permanent transfer', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'PermanentTransferDelegate',
        authority: {
          type: 'Address',
          address: delegate.publicKey,
        },
      },
      {
        type: 'FreezeDelegate',
        frozen: true,
      },
    ],
  });

  t.assert(!canTransfer(owner.publicKey, asset));
  t.assert(canTransfer(delegate.publicKey, asset));
  t.is(
    await validateTransfer(umi, { authority: owner.publicKey, asset }),
    LifecycleValidationError.AssetFrozen
  );
  t.is(
    await validateTransfer(umi, { authority: delegate.publicKey, asset }),
    null
  );
});

test('it can detect transferrable when frozen with permanent collection transfer delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      plugins: [
        {
          type: 'FreezeDelegate',
          frozen: true,
        },
      ],
    },
    {
      plugins: [
        {
          type: 'PermanentTransferDelegate',
          authority: {
            type: 'Address',
            address: delegate.publicKey,
          },
        },
      ],
    }
  );

  t.assert(!canTransfer(owner.publicKey, asset, collection));
  t.assert(canTransfer(delegate.publicKey, asset, collection));
  t.is(
    await validateTransfer(umi, {
      authority: owner.publicKey,
      asset,
      collection,
    }),
    LifecycleValidationError.AssetFrozen
  );
  t.is(
    await validateTransfer(umi, {
      authority: delegate.publicKey,
      asset,
      collection,
    }),
    null
  );
});

test('it can detect burnable on basic asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
  });

  t.assert(canBurn(owner.publicKey, asset));
  t.is(await validateBurn(umi, { authority: owner.publicKey, asset }), null);
});

test('it can detect non burnable from frozen asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'FreezeDelegate',
        frozen: true,
      },
    ],
  });

  t.assert(!canBurn(owner.publicKey, asset));
  t.is(
    await validateBurn(umi, { authority: owner.publicKey, asset }),
    LifecycleValidationError.AssetFrozen
  );
});

test('it can detect burnable on asset with burn delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'BurnDelegate',
        authority: {
          type: 'Address',
          address: delegate.publicKey,
        },
      },
    ],
  });

  t.assert(canBurn(delegate.publicKey, asset));
  t.is(await validateBurn(umi, { authority: delegate.publicKey, asset }), null);
});

test('it can detect burnable from permanent burn', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'PermanentBurnDelegate',
        authority: {
          type: 'Address',
          address: delegate.publicKey,
        },
      },
    ],
  });

  t.assert(canBurn(delegate.publicKey, asset));
  t.is(await validateBurn(umi, { authority: delegate.publicKey, asset }), null);
});

test('it can detect burnable when frozen with permanent burn', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'PermanentBurnDelegate',
        authority: {
          type: 'Address',
          address: delegate.publicKey,
        },
      },
      {
        type: 'FreezeDelegate',
        frozen: true,
      },
    ],
  });

  t.assert(!canBurn(owner.publicKey, asset));
  t.assert(canBurn(delegate.publicKey, asset));
  t.is(
    await validateBurn(umi, { authority: owner.publicKey, asset }),
    LifecycleValidationError.AssetFrozen
  );
  t.is(await validateBurn(umi, { authority: delegate.publicKey, asset }), null);
});

test('it can detect burnable when frozen with permanent collection burn delegate', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const delegate = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      plugins: [
        {
          type: 'FreezeDelegate',
          frozen: true,
        },
      ],
    },
    {
      plugins: [
        {
          type: 'PermanentBurnDelegate',
          authority: {
            type: 'Address',
            address: delegate.publicKey,
          },
        },
      ],
    }
  );

  t.assert(!canBurn(owner.publicKey, asset, collection));
  t.assert(canBurn(delegate.publicKey, asset, collection));
  t.is(
    await validateBurn(umi, { authority: owner.publicKey, asset, collection }),
    LifecycleValidationError.AssetFrozen
  );
  t.is(
    await validateBurn(umi, {
      authority: delegate.publicKey,
      asset,
      collection,
    }),
    null
  );
});

test('it can validate non-transferrable asset with oracle', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const oracle = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: oracle,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Rejected,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        baseAddress: oracle.publicKey,
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
      },
    ],
  });

  t.is(
    await validateTransfer(umi, {
      authority: owner.publicKey,
      asset,
    }),
    LifecycleValidationError.OracleValidationFailed
  );
});

test('it can validate transferrable asset with oracle', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const oracle = generateSigner(umi);
  const oracle2 = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: oracle,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        baseAddress: oracle.publicKey,
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
      },
      {
        type: 'Oracle',
        baseAddress: oracle2.publicKey,
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
      },
    ],
  });

  t.is(
    await validateTransfer(umi, {
      authority: owner.publicKey,
      asset,
    }),
    null
  );
});

test('it can validate non-transferrable asset with oracle with recipient seed', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const seedPubkey = generateSigner(umi).publicKey;
  const newOwner = generateSigner(umi);

  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      transfer: [CheckResult.CAN_REJECT],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    baseAddressConfig: {
      type: 'CustomPda',
      seeds: [
        { type: 'Collection' },
        { type: 'Owner' },
        { type: 'Recipient' },
        { type: 'Asset' },
        { type: 'Address', pubkey: seedPubkey },
        {
          type: 'Bytes',
          bytes: Buffer.from('example-seed-bytes', 'utf8'),
        },
      ],
    },
  };

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      plugins: [oraclePlugin],
    },
    {}
  );

  const account = findOracleAccount(umi, oraclePlugin, {
    collection: collection.publicKey,
    owner: owner.publicKey,
    recipient: newOwner.publicKey,
    asset: asset.publicKey,
  });

  // write to example program oracle account
  await customPdaAllSeedsInit(umi, {
    account,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      collection: collection.publicKey,
      owner: owner.publicKey,
      recipient: newOwner.publicKey,
      asset: asset.publicKey,
      address: seedPubkey,
      bytes: Buffer.from('example-seed-bytes', 'utf8'),
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Rejected,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  t.is(
    await validateTransfer(umi, {
      authority: owner.publicKey,
      asset,
      recipient: newOwner.publicKey,
      collection,
    }),
    LifecycleValidationError.OracleValidationFailed
  );
});

test('it can validate and skip transferrable asset with oracle with recipient seed if missing recipient', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const { asset, collection } = await createAssetWithCollection(
    umi,
    {
      owner,
      plugins: [
        {
          type: 'Oracle',
          resultsOffset: {
            type: 'Anchor',
          },
          lifecycleChecks: {
            transfer: [CheckResult.CAN_REJECT],
          },
          baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
          baseAddressConfig: {
            type: 'CustomPda',
            seeds: [
              {
                type: 'Recipient',
              },
            ],
          },
        },
      ],
    },
    {}
  );

  t.is(
    await validateTransfer(umi, {
      authority: owner.publicKey,
      collection,
      asset,
    }),
    null
  );
});

test('it can validate non-burnable asset with oracle', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const oracle = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: oracle,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Rejected,
      },
    },
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        baseAddress: oracle.publicKey,
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          burn: [CheckResult.CAN_REJECT],
        },
      },
    ],
  });

  t.is(
    await validateBurn(umi, {
      authority: owner.publicKey,
      asset,
    }),
    LifecycleValidationError.OracleValidationFailed
  );
});

test('it can validate burnable asset with oracle', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const oracle = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: oracle,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        baseAddress: oracle.publicKey,
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
      },
    ],
  });

  t.is(
    await validateBurn(umi, {
      authority: owner.publicKey,
      asset,
    }),
    null
  );
});

test('it can validate non-updatable asset with oracle', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const oracle = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: oracle,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Rejected,
      },
    },
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        baseAddress: oracle.publicKey,
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          update: [CheckResult.CAN_REJECT],
        },
      },
    ],
  });

  t.is(
    await validateUpdate(umi, {
      authority: owner.publicKey,
      asset,
    }),
    LifecycleValidationError.OracleValidationFailed
  );
});

test('it can validate updatable asset with oracle', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const oracle = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account: oracle,
    signer: umi.identity,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        baseAddress: oracle.publicKey,
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_REJECT],
        },
      },
    ],
  });

  t.is(
    await validateUpdate(umi, {
      authority: umi.identity.publicKey,
      asset,
    }),
    null
  );
});
