import test from 'ava';
import { generateSigner } from '@metaplex-foundation/umi';
import {
  getPubkeyAuthority,
  addPlugin,
  // getPubkeyAuthority,
    // plugin,
  plugin,
  pluginAuthorityPair,
  transfer,
  updateAuthority,
  // updatePlugin,
  // approvePluginAuthority,
  // PluginType,
  // authority,
} from '../../../src';
import {
//   DEFAULT_ASSET,
  assertAsset,
  createAsset,
//   createAssetWithCollection,
  createUmi,
} from '../../_setup';

test('it cannot add permanentTransfer after creation', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const asset = await createAsset(umi, { owner });

  const result = addPlugin(umi, {
    asset: asset.publicKey,
    plugin: plugin('PermanentTransfer', [{authority: getPubkeyAuthority(owner.publicKey)}]),
    authority: owner,
  }).sendAndConfirm(umi);

    await t.throwsAsync(result, {
      name: 'InvalidAuthority',
    });

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    permanentTransfer: undefined
  });
});

test('it cannot transfer an asset as the owner and not the delegate', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const brandNewOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({ type: 'PermanentTransfer', authority: getPubkeyAuthority(owner.publicKey) }),
    ],
  });

  await transfer(umi, {
    authority: owner,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    permanentTransfer: {
      authority: {
        type: 'Pubkey',
        address: owner.publicKey,
      }
    },
  });

  const result = transfer(umi, {
    authority: newOwner,
    asset: asset.publicKey,
    newOwner: brandNewOwner.publicKey,
  }).sendAndConfirm(umi);

  await t.throwsAsync(result, {
    name: 'InvalidAuthority',
  });

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    permanentTransfer: {
      authority: {
        type: 'Pubkey',
        address: owner.publicKey,
      }
    },
  });
});

test('it can transfer an asset as the delegate and the owner', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);

  const asset = await createAsset(umi, {
    owner,
    plugins: [
      pluginAuthorityPair({ type: 'PermanentTransfer', authority: getPubkeyAuthority(owner.publicKey) }),
    ],
  });

  await transfer(umi, {
    authority: owner,
    asset: asset.publicKey,
    newOwner: newOwner.publicKey,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...asset,
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
    permanentTransfer: {
      authority: {
        type: 'Pubkey',
        address: owner.publicKey,
      }
    },
  });
});

test('it can transfer an asset as not the owner', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const owner = generateSigner(umi);
    const newOwner = generateSigner(umi);
    const brandNewOwner = generateSigner(umi);

    const asset = await createAsset(umi, {
        owner,
        plugins: [
          pluginAuthorityPair({ type: 'PermanentTransfer', authority: getPubkeyAuthority(owner.publicKey) }),
        ],
      });

    await transfer(umi, {
      asset: asset.publicKey,
      newOwner: newOwner.publicKey,
      authority: owner,
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...asset,
        asset: asset.publicKey,
        owner: newOwner.publicKey,
        updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
        permanentTransfer: {
          authority: {
            type: 'Pubkey',
            address: owner.publicKey,
          }
        },
      });

    await transfer(umi, {
        asset: asset.publicKey,
        newOwner: brandNewOwner.publicKey,
        authority: owner,
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...asset,
        asset: asset.publicKey,
        owner: brandNewOwner.publicKey,
        updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
        permanentTransfer: {
          authority: {
            type: 'Pubkey',
            address: owner.publicKey,
          }
        },
      });
  });

  // test('it cannot delegate its authority', async (t) => {
  //   // Given a Umi instance and a new signer.
  //   const umi = await createUmi();
  //   const owner = generateSigner(umi);
  //   const newOwner = generateSigner(umi);
  
  //   const asset = await createAsset(umi, {
  //     owner,
  //     plugins: [
  //       pluginAuthorityPair({ type: 'PermanentTransfer', authority: getPubkeyAuthority(owner.publicKey) }),
  //     ],
  //   });
  
  //   await transfer(umi, {
  //     authority: owner,
  //     asset: asset.publicKey,
  //     newOwner: newOwner.publicKey,
  //   }).sendAndConfirm(umi);
  
  //   await assertAsset(t, umi, {
  //     ...asset,
  //     asset: asset.publicKey,
  //     owner: newOwner.publicKey,
  //     updateAuthority: updateAuthority('Address', [umi.identity.publicKey]),
  //     permanentTransfer: {
  //       authority: {
  //         type: 'Pubkey',
  //         address: owner.publicKey,
  //       }
  //     },
  //   });
  // });

// test('it cannot transfer asset in collection if no collection', async (t) => {
//   // Given a Umi instance and a new signer.
//   const umi = await createUmi();
//   const owner = generateSigner(umi);
//   const newOwner = generateSigner(umi);

//   const { asset, collection } = await createAssetWithCollection(umi, {
//     owner,
//     plugins: [{ plugin: plugin('PermanentTransfer', [{}]), authority: null }],
//   });

//   const result = transfer(umi, {
//     asset: asset.publicKey,
//     newOwner: newOwner.publicKey,
//   }).sendAndConfirm(umi);

//   await t.throwsAsync(result, { name: 'MissingCollection' });

//   await assertAsset(t, umi, {
//     asset: asset.publicKey,
//     owner: umi.identity.publicKey,
//     updateAuthority: updateAuthority('Collection', [collection.publicKey]),
//   });
// });
