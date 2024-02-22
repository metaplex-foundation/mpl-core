import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
// import { base58 } from '@metaplex-foundation/umi/serializers';
import { Asset, AssetWithPlugins, DataState, PluginType, addAuthority, addPlugin, create, fetchAsset, fetchAssetWithPlugins } from '../src';
import { createUmi } from './_setup';

test('it can add an authority to a plugin', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const assetAddress = generateSigner(umi);
  const delegateAddress = generateSigner(umi);

  // When we create a new account.
  await create(umi, {
    dataState: DataState.AccountState,
    assetAddress,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  }).sendAndConfirm(umi);

  // Then an account was created with the correct data.
  const asset = await fetchAsset(umi, assetAddress.publicKey);
  // console.log("Account State:", asset);
  t.like(asset, <Asset>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
  });

  await addPlugin(umi, {
    assetAddress: assetAddress.publicKey,
    plugin: {
      __kind: 'Freeze',
      fields: [{ frozen: false }],
    }
  }).append(
    addAuthority(umi, {
      assetAddress: assetAddress.publicKey,
      pluginType: PluginType.Freeze,
      newAuthority: {
        __kind: 'Pubkey',
        address: delegateAddress.publicKey,
      }
    })
  ).sendAndConfirm(umi);

  const asset1 = await fetchAssetWithPlugins(umi, assetAddress.publicKey);
  // console.log(JSON.stringify(asset1, (_, v) => typeof v === 'bigint' ? v.toString() : v, 2));
  t.like(asset1, <AssetWithPlugins>{
    publicKey: assetAddress.publicKey,
    updateAuthority: umi.identity.publicKey,
    owner: umi.identity.publicKey,
    name: 'Test Bread',
    uri: 'https://example.com/bread',
    pluginHeader: {
      key: 3,
      pluginRegistryOffset: BigInt(119),
    },
    pluginRegistry: {
      key: 4,
      registry: [{
        pluginType: 2,
        data: {
          offset: BigInt(117),
          authorities: [
            { __kind: "Owner" },
            { __kind: "Pubkey", address: delegateAddress.publicKey }
          ]
        }
      }],
    },
    plugins: [{
      authorities: [
        { __kind: "Owner" },
        { __kind: "Pubkey", address: delegateAddress.publicKey }
      ],
      plugin: {
        __kind: 'Freeze',
        fields: [{ frozen: false }],
      },
    }],
  });
});
