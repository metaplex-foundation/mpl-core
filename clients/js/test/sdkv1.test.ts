import { assertAccountExists, generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import {
  AddablePluginAuthorityPairArgsV2,
  addPlugin,
  burn,
  Key,
  PluginArgsV2,
  removePlugin,
  transfer,
  update,
  updateCollection,
  updateCollectionPlugin,
  updatePlugin,
} from '../src';
import {
  createAsset,
  createAssetWithCollection,
  createCollection,
} from './_setupSdk';
import {
  assertAsset,
  assertCollection,
  createUmi,
  DEFAULT_ASSET,
  DEFAULT_COLLECTION,
} from './_setupRaw';

test('it can create asset and collection with all update auth managed party plugins', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'Royalties',
        basisPoints: 500,
        creators: [
          {
            address: owner.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: {
          type: 'ProgramDenyList',
          addresses: [owner.publicKey],
        },
        authority: {
          type: 'Address',
          address: owner.publicKey,
        },
      },
      {
        type: 'PermanentBurnDelegate',
        authority: {
          type: 'None',
        },
      },
      {
        type: 'PermanentFreezeDelegate',
        frozen: false,
      },
      {
        type: 'PermanentTransferDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      },
      {
        type: 'Attributes',
        attributeList: [
          {
            key: '123',
            value: '456',
          },
        ],
      },
    ],
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      basisPoints: 500,
      creators: [
        {
          address: owner.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [owner.publicKey],
      },
      authority: {
        type: 'Address',
        address: owner.publicKey,
      },
    },
    permanentBurnDelegate: {
      authority: {
        type: 'None',
      },
    },
    permanentFreezeDelegate: {
      frozen: false,
      authority: {
        type: 'UpdateAuthority',
      },
    },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
    attributes: {
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  const asset = await createAsset(umi, {
    owner,
    collection,
    plugins: [
      {
        type: 'Edition',
        authority: {
          type: 'UpdateAuthority',
        },
        number: 1,
      },
      {
        type: 'Royalties',
        basisPoints: 500,
        creators: [
          {
            address: owner.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: {
          type: 'ProgramDenyList',
          addresses: [owner.publicKey],
        },
        authority: {
          type: 'Address',
          address: owner.publicKey,
        },
      },
      {
        type: 'PermanentBurnDelegate',
        authority: {
          type: 'None',
        },
      },
      {
        type: 'PermanentFreezeDelegate',
        frozen: false,
      },
      {
        type: 'PermanentTransferDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      },
      {
        type: 'Attributes',
        attributeList: [
          {
            key: '123',
            value: '456',
          },
        ],
      },
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: owner.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: collection.publicKey,
    },
    edition: {
      number: 1,
      authority: {
        type: 'UpdateAuthority',
      },
    },
    royalties: {
      basisPoints: 500,
      creators: [
        {
          address: owner.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [owner.publicKey],
      },
      authority: {
        type: 'Address',
        address: owner.publicKey,
      },
    },
    permanentBurnDelegate: {
      authority: {
        type: 'None',
      },
    },
    permanentFreezeDelegate: {
      frozen: false,
      authority: {
        type: 'UpdateAuthority',
      },
    },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
    attributes: {
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can create all owner and update auth managed party plugins to asset', async (t) => {
  const umi = await createUmi();

  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Royalties',
        basisPoints: 500,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: {
          type: 'ProgramDenyList',
          addresses: [umi.identity.publicKey],
        },
        authority: {
          type: 'Address',
          address: umi.identity.publicKey,
        },
      },
      {
        type: 'PermanentBurnDelegate',
        authority: {
          type: 'None',
        },
      },
      {
        type: 'PermanentFreezeDelegate',
        frozen: false,
      },
      {
        type: 'PermanentTransferDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      },
      {
        type: 'Attributes',
        attributeList: [
          {
            key: '123',
            value: '456',
          },
        ],
      },
      {
        type: 'Edition',
        number: 1,
        authority: {
          type: 'UpdateAuthority',
        },
      },
      {
        type: 'FreezeDelegate',
        frozen: false,
      },
      {
        type: 'BurnDelegate',
        authority: {
          type: 'None',
        },
      },
      {
        type: 'TransferDelegate',
        authority: {
          type: 'UpdateAuthority',
        },
      },
    ],
  });

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    royalties: {
      basisPoints: 500,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [umi.identity.publicKey],
      },
      authority: {
        type: 'Address',
        address: umi.identity.publicKey,
      },
    },
    permanentBurnDelegate: {
      authority: {
        type: 'None',
      },
    },
    permanentFreezeDelegate: {
      frozen: false,
      authority: {
        type: 'UpdateAuthority',
      },
    },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
    attributes: {
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
      authority: {
        type: 'UpdateAuthority',
      },
    },
    edition: {
      number: 1,
      authority: {
        type: 'UpdateAuthority',
      },
    },
    freezeDelegate: {
      frozen: false,
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'None',
      },
    },
    transferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can add and remove all owner and update auth managed party plugins to asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi);

  const plugins: AddablePluginAuthorityPairArgsV2[] = [
    {
      type: 'Royalties',
      basisPoints: 500,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [umi.identity.publicKey],
      },
      authority: {
        type: 'Address',
        address: umi.identity.publicKey,
      },
    },
    {
      type: 'Attributes',
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
    },
    {
      type: 'FreezeDelegate',
      frozen: false,
    },
    {
      type: 'BurnDelegate',
      authority: {
        type: 'None',
      },
    },
    {
      type: 'TransferDelegate',
      authority: {
        type: 'UpdateAuthority',
      },
    },
  ];

  await Promise.all(
    plugins.map(async (plugin) =>
      addPlugin(umi, {
        asset: asset.publicKey,
        plugin,
      }).sendAndConfirm(umi)
    )
  );

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    royalties: {
      basisPoints: 500,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [umi.identity.publicKey],
      },
      authority: {
        type: 'Address',
        address: umi.identity.publicKey,
      },
    },
    attributes: {
      attributeList: [
        {
          key: '123',
          value: '456',
        },
      ],
      authority: {
        type: 'UpdateAuthority',
      },
    },
    freezeDelegate: {
      frozen: false,
      authority: {
        type: 'Owner',
      },
    },
    burnDelegate: {
      authority: {
        type: 'None',
      },
    },
    transferDelegate: {
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });

  await Promise.all(
    plugins.map(async (plugin) =>
      removePlugin(umi, {
        asset: asset.publicKey,
        plugin,
      }).sendAndConfirm(umi)
    )
  );

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    attributes: undefined,
    royalties: undefined,
    freezeDelegate: undefined,
    burnDelegate: undefined,
    transferDelegate: undefined,
  });
});

test('it can update all updatable plugins on asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {
    plugins: [
      {
        type: 'Royalties',
        basisPoints: 500,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: {
          type: 'ProgramDenyList',
          addresses: [umi.identity.publicKey],
        },
      },
      {
        type: 'Attributes',
        attributeList: [
          {
            key: '123',
            value: '456',
          },
        ],
      },
      {
        type: 'FreezeDelegate',
        frozen: false,
      },
      {
        type: 'PermanentFreezeDelegate',
        frozen: false,
      },
      {
        type: 'Edition',
        number: 1,
      },
    ],
  });

  const updates: PluginArgsV2[] = [
    {
      type: 'Royalties',
      basisPoints: 1000,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramAllowList',
        addresses: [umi.identity.publicKey],
      },
    },
    {
      type: 'Attributes',
      attributeList: [
        {
          key: 'abc',
          value: 'xyz',
        },
      ],
    },
    {
      type: 'FreezeDelegate',
      frozen: true,
    },
    {
      type: 'PermanentFreezeDelegate',
      frozen: true,
    },
    {
      type: 'Edition',
      number: 2,
    },
  ];

  await Promise.all(
    updates.map(async (plugin) =>
      updatePlugin(umi, {
        asset: asset.publicKey,
        plugin,
      }).sendAndConfirm(umi)
    )
  );

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    royalties: {
      basisPoints: 1000,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramAllowList',
        addresses: [umi.identity.publicKey],
      },
      authority: {
        type: 'UpdateAuthority',
      },
    },
    attributes: {
      attributeList: [
        {
          key: 'abc',
          value: 'xyz',
        },
      ],
      authority: {
        type: 'UpdateAuthority',
      },
    },
    freezeDelegate: {
      frozen: true,
      authority: {
        type: 'Owner',
      },
    },
    permanentFreezeDelegate: {
      frozen: true,
      authority: {
        type: 'UpdateAuthority',
      },
    },
    edition: {
      number: 2,
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can update all updatable plugins on collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {
    plugins: [
      {
        type: 'Royalties',
        basisPoints: 500,
        creators: [
          {
            address: umi.identity.publicKey,
            percentage: 100,
          },
        ],
        ruleSet: {
          type: 'ProgramDenyList',
          addresses: [umi.identity.publicKey],
        },
      },
      {
        type: 'Attributes',
        attributeList: [
          {
            key: '123',
            value: '456',
          },
        ],
      },
      {
        type: 'PermanentFreezeDelegate',
        frozen: false,
      },
    ],
  });

  const updates: PluginArgsV2[] = [
    {
      type: 'Royalties',
      basisPoints: 1000,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramAllowList',
        addresses: [umi.identity.publicKey],
      },
    },
    {
      type: 'Attributes',
      attributeList: [
        {
          key: 'abc',
          value: 'xyz',
        },
      ],
    },
    {
      type: 'PermanentFreezeDelegate',
      frozen: true,
    },
  ];

  await Promise.all(
    updates.map(async (plugin) =>
      updateCollectionPlugin(umi, {
        collection: collection.publicKey,
        plugin,
      }).sendAndConfirm(umi)
    )
  );

  await assertCollection(t, umi, {
    collection: collection.publicKey,
    royalties: {
      basisPoints: 1000,
      creators: [
        {
          address: umi.identity.publicKey,
          percentage: 100,
        },
      ],
      ruleSet: {
        type: 'ProgramAllowList',
        addresses: [umi.identity.publicKey],
      },
      authority: {
        type: 'UpdateAuthority',
      },
    },
    attributes: {
      attributeList: [
        {
          key: 'abc',
          value: 'xyz',
        },
      ],
      authority: {
        type: 'UpdateAuthority',
      },
    },
    permanentFreezeDelegate: {
      frozen: true,
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer asset', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const asset = await createAsset(umi, {
    owner,
    plugins: [
      {
        type: 'Edition',
        number: 1,
      },
    ],
  });

  await transfer(umi, {
    asset,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    edition: {
      number: 1,
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can transfer asset in collection', async (t) => {
  const umi = await createUmi();
  const owner = generateSigner(umi);
  const newOwner = generateSigner(umi);
  const { asset, collection } = await createAssetWithCollection(umi, {
    owner,
    plugins: [
      {
        type: 'Edition',
        number: 1,
      },
    ],
  });

  await transfer(umi, {
    asset,
    collection,
    newOwner: newOwner.publicKey,
    authority: owner,
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    asset: asset.publicKey,
    owner: newOwner.publicKey,
    updateAuthority: {
      type: 'Collection',
      address: collection.publicKey,
    },
    edition: {
      number: 1,
      authority: {
        type: 'UpdateAuthority',
      },
    },
  });
});

test('it can update asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {});

  await update(umi, {
    asset,
    name: 'bleh',
  }).sendAndConfirm(umi);

  await assertAsset(t, umi, {
    ...DEFAULT_ASSET,
    asset: asset.publicKey,
    owner: umi.identity.publicKey,
    name: 'bleh',
  });
});

test('it can update collection', async (t) => {
  const umi = await createUmi();
  const collection = await createCollection(umi, {});

  await updateCollection(umi, {
    collection: collection.publicKey,
    name: 'bleh',
  }).sendAndConfirm(umi);

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    name: 'bleh',
  });
});

test('it can burn asset', async (t) => {
  const umi = await createUmi();
  const asset = await createAsset(umi, {});

  await burn(umi, {
    asset,
  }).sendAndConfirm(umi);

  const afterAsset = await umi.rpc.getAccount(asset.publicKey);
  t.true(afterAsset.exists);
  assertAccountExists(afterAsset);
  t.is(afterAsset.data.length, 1);
  t.is(afterAsset.data[0], Key.Uninitialized);
});

test('it can burn asset in collection', async (t) => {
  const umi = await createUmi();
  const { asset, collection } = await createAssetWithCollection(umi, {});

  await burn(umi, {
    asset,
    collection,
  }).sendAndConfirm(umi);

  const afterAsset = await umi.rpc.getAccount(asset.publicKey);
  t.true(afterAsset.exists);
  assertAccountExists(afterAsset);
  t.is(afterAsset.data.length, 1);
  t.is(afterAsset.data[0], Key.Uninitialized);
});
