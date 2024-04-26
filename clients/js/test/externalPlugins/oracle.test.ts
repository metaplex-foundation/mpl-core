import test from 'ava';
import fs from "fs";

import { mplCoreOracleExample, fixedAccountInit, fixedAccountSet }  from '@metaplex-foundation/mpl-core-oracle-example'
import { createSignerFromKeypair, Context, generateSigner } from '@metaplex-foundation/umi';
import { ExternalValidationResult } from '@metaplex-foundation/mpl-core-oracle-example/dist/src/hooked';
import { assertAsset, createUmi as baseCreateUmi, DEFAULT_ASSET } from '../_setupRaw';
import { createAsset } from '../_setupSdk';
import { CheckResult, transfer, update } from '../../src';


const createUmi = async () => (await baseCreateUmi()).use(mplCoreOracleExample());
function loadSecretFromFile(filename: string) {
  const secret = JSON.parse(fs.readFileSync(filename).toString()) as number[];
  const secretKey = Uint8Array.from(secret);
  return secretKey
}

const secret = loadSecretFromFile('../../../mpl-core-oracle-example/aaa48hFxxsUJb2MUeUVe8ABH42F6nho69oXUkSgKeSM.json')
function getAuthoritySigner(umi: Context) {
  return createSignerFromKeypair(umi, umi.eddsa.createKeypairFromSecretKey(secret))
}

test('it can use fixed address oracle to control update', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi)

  const signer = getAuthoritySigner(umi)
  
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
      }
    }
  }).sendAndConfirm(umi)

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [{
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor'
      },
      lifecycleChecks: {
        update: [CheckResult.CAN_DENY]
      },
      baseAddress: account.publicKey,
    }]
  })

  const result = update(umi, {
    asset,
    name: 'new name'
  }).sendAndConfirm(umi)

  await t.throwsAsync(result, {name: 'InvalidAuthority'})

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
      }
    }
  }).sendAndConfirm(umi)

  await update(umi, {
    asset,
    name: 'new name 2'
  }).sendAndConfirm(umi)

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'new name 2'
  })

})

test('it can use fixed address oracle to control transfer', async (t) => {
  const umi = await createUmi();
  const account = generateSigner(umi)

  const signer = getAuthoritySigner(umi)
  
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
      }
    }
  }).sendAndConfirm(umi)

  // create asset referencing the oracle account
  const asset = await createAsset(umi, {
    plugins: [{
      type: 'Oracle',
      resultsOffset: {
        type: 'Anchor'
      },
      lifecycleChecks: {
        transfer: [CheckResult.CAN_DENY]
      },
      baseAddress: account.publicKey,
    }]
  })

  const newOwner = generateSigner(umi)

  const result = transfer(umi, {
    asset,
    newOwner: newOwner.publicKey
  }).sendAndConfirm(umi)

  await t.throwsAsync(result, {name: 'InvalidAuthority'})

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
      }
    }
  }).sendAndConfirm(umi)

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey
  }).sendAndConfirm(umi)

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
  })

})