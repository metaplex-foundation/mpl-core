import {
  generateSigner,
  createSignerFromKeypair,
  keypairIdentity,
  publicKey,
  Option,
  PublicKey,
} from '@metaplex-foundation/umi';
import { createAsset, createCollection, createUmiWithDas } from '../_setup';
import {
  pluginAuthorityPair,
  AssetV1,
  Key,
  ruleSet,
  addressPluginAuthority,
  CollectionV1,
} from '../../src';

const assetCommonData = {
  name: 'DAS Test Asset',
  uri: 'https://example.com/das-asset',
};

function getPluginsForCreation(
  transferDelegateAuthority: PublicKey,
  payer: PublicKey
) {
  return [
    pluginAuthorityPair({
      authority: addressPluginAuthority(transferDelegateAuthority),
      type: 'TransferDelegate',
    }),
    pluginAuthorityPair({
      type: 'BurnDelegate',
    }),
    pluginAuthorityPair({
      type: 'UpdateDelegate',
    }),
    pluginAuthorityPair({
      type: 'FreezeDelegate',
      data: { frozen: false },
    }),
    pluginAuthorityPair({
      type: 'Attributes',
      data: { attributeList: [{ key: 'some key', value: 'some value' }] },
    }),
    pluginAuthorityPair({
      type: 'Royalties',
      data: {
        basisPoints: 5,
        creators: [
          {
            address: payer,
            percentage: 100,
          },
        ],
        ruleSet: ruleSet('None'),
      },
    }),
  ];
}

// Run this function if assets for DAS tests need to be changed for some reason
// @ts-ignore
export async function createDasTestAssetOrCollection({
  payerSecretKey,
  rpcEndpoint,
  isUpdateAuthorityOwner = true,
  isCollection = false,
  collection,
}: {
  payerSecretKey: Uint8Array;
  rpcEndpoint: string;
  isUpdateAuthorityOwner?: boolean;
  isCollection?: boolean;
  collection?: PublicKey;
}) {
  const umi = createUmiWithDas(rpcEndpoint);
  const payerKeypair = umi.eddsa.createKeypairFromSecretKey(payerSecretKey);
  const payer = createSignerFromKeypair(umi, payerKeypair);
  const assetAddress = generateSigner(umi);
  const transferDelegateAuthority = generateSigner(umi);
  const updateAuthority = !isUpdateAuthorityOwner
    ? generateSigner(umi)
    : undefined;

  umi.use(keypairIdentity(payerKeypair));

  let asset: AssetV1 | CollectionV1;
  if (isCollection) {
    asset = await createCollection(umi, {
      ...assetCommonData,
      updateAuthority,
      collection: assetAddress,
      payer,
      plugins: getPluginsForCreation(
        transferDelegateAuthority.publicKey,
        payer.publicKey
      ),
    });
  } else {
    asset = await createAsset(umi, {
      ...assetCommonData,
      updateAuthority,
      asset: assetAddress,
      payer,
      collection,
      plugins: getPluginsForCreation(
        transferDelegateAuthority.publicKey,
        payer.publicKey
      ),
    });
  }

  const entity = isCollection ? 'collection' : 'asset';
  console.log(
    `A new ${entity} is created. Save the info below somewhere for further use in tests!`
  );
  console.log(`Created ${entity}:`, asset);
}

const testAssetCommonData = {
  key: Key.AssetV1,
  seq: { __option: 'None' } as Option<bigint>,
};

export const dasTestAsset1Owner = publicKey(
  'EBgC18R6zKNic1CLYKYEy3SMSz4zweymeqrMHkXeqpag'
);
export const dasTestAsset1PubKey = publicKey(
  '8gw7zp8cgo2wwp7wvykXaYvGUS6sHqYZ1PnzyQFW2ANG'
);
export const dasTestAsset1: AssetV1 = {
  ...assetCommonData,
  ...testAssetCommonData,
  owner: dasTestAsset1Owner,
  publicKey: dasTestAsset1PubKey,
  updateAuthority: {
    type: 'Address',
    address: dasTestAsset1Owner,
  },
  header: {
    executable: false,
    owner: publicKey('CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'),
    lamports: { basisPoints: 4541520n, identifier: 'SOL', decimals: 9 },
    rentEpoch: 18446744073709552000,
    // exists: true,
  },
  // pluginHeader: { key: 3, pluginRegistryOffset: 208n },
  transferDelegate: {
    authority: {
      type: 'Address',
      address: publicKey('3XSp2DSVM99ETD8YrxrytkGVAAu4rE47TJLoD7m3Ww1J'),
    },
    offset: 127n,
  },
  burnDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 128n,
  },
  updateDelegate: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 129n,
    additionalDelegates: [],
  },
  freezeDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 134n,
    frozen: false,
  },
  attributes: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 136n,
    attributeList: [{ key: 'some key', value: 'some value' }],
  },
  royalties: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 167n,
    basisPoints: 5,
    creators: [
      {
        address: dasTestAsset1Owner,
        percentage: 100,
      },
    ],
    ruleSet: { __kind: 'None' },
  },
};

export const dasTestAsset2PubKey = publicKey(
  '6wTjNnv5a4QFZDSyVMYaVPSGyAgk9Tppr8Cb3yBjd5th'
);
export const dasTestAsset2Owner = dasTestAsset1Owner;
export const dasTestAsset2UpdateAuthority = publicKey(
  '7Vrco3r1u1wiBwTm7GZtBmFfVKqVR1tyLn8adUuorQ63'
);
export const dasTestAsset2: AssetV1 = {
  ...assetCommonData,
  ...testAssetCommonData,
  owner: dasTestAsset2Owner,
  publicKey: dasTestAsset2PubKey,
  updateAuthority: {
    type: 'Address',
    address: dasTestAsset2UpdateAuthority,
  },
  header: {
    executable: false,
    owner: publicKey('CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'),
    lamports: { basisPoints: 4541520n, identifier: 'SOL', decimals: 9 },
    rentEpoch: 18446744073709552000,
    // exists: true,
  },
  // pluginHeader: { key: 3, pluginRegistryOffset: 208n },
  transferDelegate: {
    authority: {
      type: 'Address',
      address: publicKey('7f7w6BVuqHPb1hQgGUfxQP13NjZzF45YUHioUZkdP4Vd'),
    },
    offset: 127n,
  },
  burnDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 128n,
  },
  updateDelegate: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 129n,
    additionalDelegates: [],
  },
  freezeDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 134n,
    frozen: false,
  },
  attributes: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 136n,
    attributeList: [{ key: 'some key', value: 'some value' }],
  },
  royalties: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 167n,
    basisPoints: 5,
    creators: [
      {
        address: dasTestAsset2Owner,
        percentage: 100,
      },
    ],
    ruleSet: { __kind: 'None' },
  },
};

const testCollectionCommonData = {
  key: Key.CollectionV1,
  numMinted: 0,
  currentSize: 0,
};

export const dasTestCollection1PubKey = publicKey(
  '6yNsdZyWiVYzFd56jZe7SjqBwf8UCbv1gbZb3eqyfVSL'
);
export const dasTestCollection1: CollectionV1 = {
  ...assetCommonData,
  ...testCollectionCommonData,
  numMinted: 1,
  currentSize: 1,
  publicKey: dasTestCollection1PubKey,
  header: {
    executable: false,
    owner: publicKey('CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'),
    lamports: { basisPoints: 2860560n, identifier: 'SOL', decimals: 9 },
    rentEpoch: 18446744073709552000,
    // exists: true,
  },
  // pluginHeader: { key: 3, pluginRegistryOffset: 182n },
  transferDelegate: {
    authority: {
      type: 'Address',
      address: publicKey('A8SQ9q2aRg2SAxJtBQxj8XwWMU74H1R7DYVCyNeYnQFx'),
    },
    offset: 101n,
  },
  burnDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 102n,
  },
  updateDelegate: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 103n,
    additionalDelegates: [],
  },
  freezeDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 108n,
    frozen: false,
  },
  attributes: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 110n,
    attributeList: [{ key: 'some key', value: 'some value' }],
  },
  royalties: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 141n,
    basisPoints: 5,
    creators: [
      {
        address: dasTestAsset1Owner,
        percentage: 100,
      },
    ],
    ruleSet: { __kind: 'None' },
  },
  updateAuthority: dasTestAsset1Owner,
};

export const dasTestCollection2PubKey = publicKey(
  '9g6VQeAn2XUTPyuGXgqZc3YnGJjMxXonhACUy8pCZXLS'
);
export const dasTestCollection2UpdateAuthority = publicKey(
  'UbmPXW4h1P1oNHegWY81a1Q9p7o2Jj5TC4LsUHSUGfm'
);
export const dasTestCollection2: CollectionV1 = {
  ...assetCommonData,
  ...testCollectionCommonData,
  publicKey: dasTestCollection2PubKey,
  header: {
    executable: false,
    owner: publicKey('CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'),
    lamports: { basisPoints: 2860560n, identifier: 'SOL', decimals: 9 },
    rentEpoch: 18446744073709552000,
    // exists: true,
  },
  // pluginHeader: { key: 3, pluginRegistryOffset: 182n },
  transferDelegate: {
    authority: {
      type: 'Address',
      address: publicKey('G8xVUAnuYbMFq8pxtWFhUvHC8TVoAkqpWgahizugWFfF'),
    },
    offset: 101n,
  },
  burnDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 102n,
  },
  updateDelegate: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 103n,
    additionalDelegates: [],
  },
  freezeDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 108n,
    frozen: false,
  },
  attributes: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 110n,
    attributeList: [{ key: 'some key', value: 'some value' }],
  },
  royalties: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 141n,
    basisPoints: 5,
    creators: [
      {
        address: dasTestAsset1Owner,
        percentage: 100,
      },
    ],
    ruleSet: { __kind: 'None' },
  },
  updateAuthority: dasTestCollection2UpdateAuthority,
};

export const dasTestAssetInCollectionPubKey = publicKey(
  'CLvNjH92gVevfwBwdabV2WeiqvkWnNBvgy1of64Xvnr3'
);
export const dasTestAssetInCollection: AssetV1 = {
  ...assetCommonData,
  ...testAssetCommonData,
  publicKey: dasTestAssetInCollectionPubKey,
  header: {
    executable: false,
    owner: publicKey('CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'),
    lamports: { basisPoints: 4541520n, identifier: 'SOL', decimals: 9 },
    rentEpoch: 18446744073709552000,
    // exists: true,
  },
  // pluginHeader: { key: 3, pluginRegistryOffset: 208n },
  transferDelegate: {
    authority: {
      type: 'Address',
      address: publicKey('GWxshtvVv5UP3KrRKLVEmfh393gpobKiS3qJHoEt8qf4'),
    },
    offset: 127n,
  },
  burnDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 128n,
  },
  updateDelegate: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 129n,
    additionalDelegates: [],
  },
  freezeDelegate: {
    authority: { type: 'Owner', address: undefined },
    offset: 134n,
    frozen: false,
  },
  attributes: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 136n,
    attributeList: [{ key: 'some key', value: 'some value' }],
  },
  royalties: {
    authority: { type: 'UpdateAuthority', address: undefined },
    offset: 167n,
    basisPoints: 5,
    creators: [
      {
        address: dasTestAsset1Owner,
        percentage: 100,
      },
    ],
    ruleSet: { __kind: 'None' },
  },
  owner: dasTestAsset1Owner,
  updateAuthority: {
    type: 'Collection',
    address: dasTestCollection1PubKey,
  },
};
