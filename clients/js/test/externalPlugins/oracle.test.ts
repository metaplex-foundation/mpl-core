import test from 'ava';
import fs from 'fs';

import {
  mplCoreOracleExample,
  fixedAccountInit,
  fixedAccountSet,
  preconfiguredAssetPdaInit,
  MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
  preconfiguredAssetPdaSet,
} from '@metaplex-foundation/mpl-core-oracle-example';
import {
  createSignerFromKeypair,
  Context,
  generateSigner,
} from '@metaplex-foundation/umi';
import { ExternalValidationResult } from '@metaplex-foundation/mpl-core-oracle-example/dist/src/hooked';
import {
  assertAsset,
  assertBurned,
  createUmi as baseCreateUmi,
  DEFAULT_ASSET,
} from '../_setupRaw';
import { createAsset, createAssetWithCollection } from '../_setupSdk';
import {
  burn,
  CheckResult,
  create,
  findOracleAccount,
  OracleInitInfoArgs,
  transfer,
  update,
} from '../../src';

const createUmi = async () =>
  (await baseCreateUmi()).use(mplCoreOracleExample());
function loadSecretFromFile(filename: string) {
  const secret = JSON.parse(fs.readFileSync(filename).toString()) as number[];
  const secretKey = Uint8Array.from(secret);
  return secretKey;
}

const secret = loadSecretFromFile(
  '../../../mpl-core-oracle-example/aaa48hFxxsUJb2MUeUVe8ABH42F6nho69oXUkSgKeSM.json'
);
function getAuthoritySigner(umi: Context) {
  return createSignerFromKeypair(
    umi,
    umi.eddsa.createKeypairFromSecretKey(secret)
  );
}

test('it can use fixed address oracle to deny update', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  const signer = getAuthoritySigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer,
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

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          update: [CheckResult.CAN_DENY],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const result = update(umi, {
    asset,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer,
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

  await update(umi, {
    asset,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});

test('it can use fixed address oracle to deny update via collection', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  const signer = getAuthoritySigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer,
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

  // create asset referencing the oracle account
  const { asset } = await createAssetWithCollection(
    umi,
    {},
    {
      plugins: [
        {
          type: 'Oracle',
          resultsOffset: {
            type: 'Anchor',
          },
          lifecycleChecks: {
            update: [CheckResult.CAN_DENY],
          },
          baseAddress: account.publicKey,
        },
      ],
    }
  );

  const result = update(umi, {
    asset,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer,
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

  await update(umi, {
    asset,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});

test('it can use fixed address oracle to deny transfer', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  const signer = getAuthoritySigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer,
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

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_DENY],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const newOwner = generateSigner(umi);

  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer,
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

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it cannot use fixed address oracle to force approve transfer', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  const signer = getAuthoritySigner(umi);
  const owner = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Approved,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_APPROVE],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const newOwner = generateSigner(umi);

  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it cannot use fixed address oracle to deny transfer if not registered for lifecycle event but has same type', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  const signer = getAuthoritySigner(umi);
  const owner = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer,
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

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          transfer: [CheckResult.CAN_APPROVE],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const newOwner = generateSigner(umi);

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it cannot use fixed address oracle to deny transfer if not registered for lifecycle event', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  const signer = getAuthoritySigner(umi);
  const owner = generateSigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer,
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

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_DENY],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const newOwner = generateSigner(umi);

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  });
});

test('it can use fixed address oracle to deny create', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  const signer = getAuthoritySigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer,
    payer: umi.identity,
    args: {
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Rejected,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  // create asset referencing the oracle account
  const assetSigner = generateSigner(umi);
  const result = create(umi, {
    ...DEFAULT_ASSET,
    asset: assetSigner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_DENY],
        },
        baseAddress: account.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });
  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer,
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

  await create(umi, {
    ...DEFAULT_ASSET,
    asset: assetSigner,
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          create: [CheckResult.CAN_DENY],
        },
        baseAddress: account.publicKey,
      },
    ],
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: assetSigner.publicKey,
    owner: umi.identity.publicKey,
  });
});

test('it can use fixed address oracle to deny burn', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi);

  const signer = getAuthoritySigner(umi);

  // write to example program oracle account
  await fixedAccountInit(umi, {
    account,
    signer,
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

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Oracle',
        resultsOffset: {
          type: 'Anchor',
        },
        lifecycleChecks: {
          burn: [CheckResult.CAN_DENY],
        },
        baseAddress: account.publicKey,
      },
    ],
  });

  const result = burn(umi, {
    asset,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await fixedAccountSet(umi, {
    account: account.publicKey,
    signer,
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

  await burn(umi, {
    asset,
  }).sendAndConfirm(umi);

  await assertBurned(t, umi, asset.publicKey);
});

test('it can use asset pda oracle to deny update', async (t) => {
  const umi = await createUmi();
  const signer = getAuthoritySigner(umi);

  const oraclePlugin: OracleInitInfoArgs = {
    type: 'Oracle',
    resultsOffset: {
      type: 'Anchor',
    },
    lifecycleChecks: {
      update: [CheckResult.CAN_DENY],
    },
    baseAddress: MPL_CORE_ORACLE_EXAMPLE_PROGRAM_ID,
    pda: {
      type: 'PreconfiguredAsset',
    },
  };

  const asset = await createAsset(umi, {
    plugins: [oraclePlugin],
  });

  // Find the oracle PDA based on the asset we just created
  const account = findOracleAccount(umi, oraclePlugin, {
    asset: asset.publicKey,
  });

  // write to the PDA which corresponds to the asset
  await preconfiguredAssetPdaInit(umi, {
    account,
    signer,
    payer: umi.identity,
    args: {
      asset: asset.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Rejected,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  const result = update(umi, {
    asset,
    name: 'new name',
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, { name: 'InvalidAuthority' });

  await preconfiguredAssetPdaSet(umi, {
    account,
    signer,
    args: {
      asset: asset.publicKey,
      oracleData: {
        __kind: 'V1',
        create: ExternalValidationResult.Pass,
        update: ExternalValidationResult.Pass,
        transfer: ExternalValidationResult.Pass,
        burn: ExternalValidationResult.Pass,
      },
    },
  }).sendAndConfirm(umi);

  await update(umi, {
    asset,
    name: 'new name 2',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2',
  });
});
