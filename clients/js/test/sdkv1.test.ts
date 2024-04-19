import { generateSigner } from '@metaplex-foundation/umi';
import test from 'ava';
import { createAsset, createCollection } from './_setupSdk';
import { assertAsset, assertCollection, createUmi, DEFAULT_COLLECTION } from './_setupRaw';

test('it can create asset and collection with all update auth managed party plugins', async (t) => {
  // Given a Umi instance and a new signer.
  const umi = await createUmi();
  const owner = generateSigner(umi);

  const collection = await createCollection(umi, {
    plugins: [{
      type: 'Royalties',
      basisPoints: 500,
      creators: [{
        address: owner.publicKey,
        percentage: 100,
      }],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [owner.publicKey],
      },
      authority: {
        type: 'Address',
        address: owner.publicKey,
      }
    }, {
      type: 'PermanentBurnDelegate',
      authority: {
        type: 'None'
      }
    }, {
      type: 'PermanentFreezeDelegate',
      frozen: false,
    }, {
      type: 'PermanentTransferDelegate',
      authority: {
        type: 'UpdateAuthority'
      }
    }, {
      type: 'Attributes',
      attributeList: [{
        key: '123',
        value: '456',
      }]
    }]
  });

  await assertCollection(t, umi, {
    ...DEFAULT_COLLECTION,
    collection: collection.publicKey,
    royalties: {
      basisPoints: 500,
      creators: [{
        address: owner.publicKey,
        percentage: 100,
      }],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [owner.publicKey],
      },
      authority: {
        type: 'Address',
        address: owner.publicKey,
      }
    },
    permanentBurnDelegate: {
      authority: {
        type: 'None'
      }
    },
    permanentFreezeDelegate: {
      frozen: false,
      authority: {
        type: 'UpdateAuthority'
      }
    },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority'
      }
    },
    attributes: {
      attributeList: [{
        key: '123',
        value: '456',
      }],
      authority: {
        type: 'UpdateAuthority'
      }
    }
  })

  const asset = await createAsset(umi, {
    owner,
    collection,
    plugins: [{
      type: 'Edition',
      authority: {
        type: 'UpdateAuthority',
      },
      number: 1,
    }, {
      type: 'Royalties',
      basisPoints: 500,
      creators: [{
        address: owner.publicKey,
        percentage: 100,
      }],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [owner.publicKey],
      },
      authority: {
        type: 'Address',
        address: owner.publicKey,
      }
    }, {
      type: 'PermanentBurnDelegate',
      authority: {
        type: 'None'
      }
    }, {
      type: 'PermanentFreezeDelegate',
      frozen: false,
    }, {
      type: 'PermanentTransferDelegate',
      authority: {
        type: 'UpdateAuthority'
      }
    }, {
      type: 'Attributes',
      attributeList: [{
        key: '123',
        value: '456',
      }]
    }]
  })

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
        type: 'UpdateAuthority'
      }
    },
    royalties: {
      basisPoints: 500,
      creators: [{
        address: owner.publicKey,
        percentage: 100,
      }],
      ruleSet: {
        type: 'ProgramDenyList',
        addresses: [owner.publicKey],
      },
      authority: {
        type: 'Address',
        address: owner.publicKey,
      }
    },
    permanentBurnDelegate: {
      authority: {
        type: 'None'
      }
    },
    permanentFreezeDelegate: {
      frozen: false,
      authority: {
        type: 'UpdateAuthority'
      }
    },
    permanentTransferDelegate: {
      authority: {
        type: 'UpdateAuthority'
      }
    },
    attributes: {
      attributeList: [{
        key: '123',
        value: '456',
      }],
      authority: {
        type: 'UpdateAuthority'
      }
    }
  })

});

/*

{
      type: 'FreezeDelegate',
      authority: {
        type: 'Owner',
      },
    },

    {
      type: 'BurnDelegate',
      authority: {
        type: 'None',
      }
    }, 
*/
