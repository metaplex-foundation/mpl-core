import test from "ava";
import { generateSigner } from "@metaplex-foundation/umi";
import { DEFAULT_ASSET, assertAsset, createAsset, createUmi } from "../../_setup";
import { PluginType, addPluginV1, createPlugin, pluginAuthorityPair, removePluginV1, transferV1, updateV1 } from "../../../src";

test('it cannot update an immutable asset', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const asset = await createAsset(umi, {
        plugins: [
            pluginAuthorityPair({
                type: 'Immutable',
            }),
        ],
    });

    const result = updateV1(umi, {
        asset: asset.publicKey,
        newName: 'Test Bread 2',
        newUri: 'https://example.com/bread2',
    }).sendAndConfirm(umi);

    await t.throwsAsync(result, { name: 'ImmutableAsset' });
});

test('it can make an existing asset immutable', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const asset = await createAsset(umi);

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: "Immutable"
        })
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });
});

test('it can make an existing asset immutable and then cannot update it', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const asset = await createAsset(umi);

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: "Immutable"
        })
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });

    const result = updateV1(umi, {
        asset: asset.publicKey,
        newName: 'Test Bread 2',
        newUri: 'https://example.com/bread2',
    }).sendAndConfirm(umi);

    await t.throwsAsync(result, { name: 'ImmutableAsset' });
});

test('it cannot add an authority-managed plugin to an immutable asset', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();

    const asset = await createAsset(umi, {
        plugins: [
            pluginAuthorityPair({
                type: 'Immutable',
            }),
        ],
    });

    // Then an account was created with the correct data.
    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    });

    const result = addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: 'Attributes', data: {
                attributeList: []
            }
        }),
    }).sendAndConfirm(umi);

    await t.throwsAsync(result, { name: 'ImmutableAsset' });
});

test('it can make an existing asset immutable and then cannot add an authority-managed plugin', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const asset = await createAsset(umi);

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: "Immutable"
        })
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });

    const result = addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: 'Attributes', data: {
                attributeList: []
            }
        }),
    }).sendAndConfirm(umi);

    await t.throwsAsync(result, { name: 'ImmutableAsset' });
});

test('it can add an owner-managed plugin to an immutable asset', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();

    const asset = await createAsset(umi, {
        plugins: [
            pluginAuthorityPair({
                type: 'Immutable',
            }),
        ],
    });

    // Then an account was created with the correct data.
    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    });

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: 'TransferDelegate'
        }),
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        transferDelegate: {
            authority: {
                type: 'Owner',
            },
        },
    });
});

test('it can make an existing asset immutable bt can still add an owner-managed plugin', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const asset = await createAsset(umi);

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: "Immutable"
        })
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: 'TransferDelegate'
        }),
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        transferDelegate: {
            authority: {
                type: 'Owner',
            },
        },
    });
});

test('it cannot remove an authority-managed plugin from an immutable asset', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();

    const asset = await createAsset(umi, {
        plugins: [
            pluginAuthorityPair({
                type: 'Immutable',
            }),
            pluginAuthorityPair({
                type: 'Attributes',
                data: { attributeList: [] }
            }),
        ],
    });

    // Then an account was created with the correct data.
    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } },
        attributes: {
            attributeList: [], authority: { type: 'UpdateAuthority' }
        }
    });

    const result = removePluginV1(umi, {
        asset: asset.publicKey,
        pluginType: PluginType.Attributes,
    }).sendAndConfirm(umi);

    await t.throwsAsync(result, { name: 'ImmutableAsset' });
});

test('it can make an existing asset immutable and then cannot remove an authority-managed plugin', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const asset = await createAsset(umi, {
        plugins: [
            pluginAuthorityPair({
                type: 'Attributes',
                data: { attributeList: [] }
            })]
    });

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: "Immutable"
        })
    }).sendAndConfirm(umi);

    // Then an account was created with the correct data.
    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } },
        attributes: {
            attributeList: [], authority: { type: 'UpdateAuthority' }
        }
    });

    const result = removePluginV1(umi, {
        asset: asset.publicKey,
        pluginType: PluginType.Attributes,
    }).sendAndConfirm(umi);

    await t.throwsAsync(result, { name: 'ImmutableAsset' });
});

test('it can remove an owner-managed plugin to an immutable asset', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();

    const asset = await createAsset(umi, {
        plugins: [
            pluginAuthorityPair({
                type: 'Immutable',
            }),
            pluginAuthorityPair({
                type: 'TransferDelegate',
            }),
        ],
    });

    // Then an account was created with the correct data.
    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
    });

    await removePluginV1(umi, {
        asset: asset.publicKey,
        pluginType: PluginType.TransferDelegate,
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        transferDelegate: undefined,
        immutable: { authority: { type: 'UpdateAuthority' } }
    });
});

test('it can make an existing asset immutable but can still remove an owner-managed plugin', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const asset = await createAsset(umi, {
        plugins: [
            pluginAuthorityPair({
                type: 'TransferDelegate',
            }),]
    });

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: "Immutable"
        })
    }).sendAndConfirm(umi);

    // Then an account was created with the correct data.
    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });

    const tx = await removePluginV1(umi, {
        asset: asset.publicKey,
        pluginType: PluginType.TransferDelegate,
    }).sendAndConfirm(umi);

    console.log(await umi.rpc.getTransaction(tx.signature));

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        transferDelegate: undefined,
        immutable: { authority: { type: 'UpdateAuthority' } }
    });
});

// test('it cannot update an authority-managed plugin on an immutable asset', async (t) => {
//     // Given a Umi instance and a new signer.
//     const umi = await createUmi();

//     const asset = await createAsset(umi, {
//         plugins: [
//             pluginAuthorityPair({
//                 type: 'Immutable',
//             }),
//             pluginAuthorityPair({
//                 type: 'Attributes',
//                 data: { attributeList: [] }
//             }),
//         ],
//     });

//     // Then an account was created with the correct data.
//     await assertAsset(t, umi, {
//         ...DEFAULT_ASSET,
//         asset: asset.publicKey,
//         owner: umi.identity.publicKey,
//         updateAuthority: { type: 'Address', address: umi.identity.publicKey },
//         immutable: { authority: { type: 'UpdateAuthority' } },
//         attributes: {
//             attributeList: [], authority: { type: 'UpdateAuthority' }
//         }
//     });

//     const result = updatePluginV1(umi, {
//         asset: asset.publicKey,
//         plugin: createPlugin({
//             type: "Attributes",
//             data: {
//                 attributeList: [{
//                     key: "key",
//                     value: "value"
//                 }]
//             }
//         })
//     }).sendAndConfirm(umi);

//     await t.throwsAsync(result, { name: 'ImmutableAsset' });
// });

// test('it can make an existing asset immutable and cannot update an authority-managed plugin', async (t) => {
//     // Given a Umi instance and a new signer.
//     const umi = await createUmi();
//     const asset = await createAsset(umi);

//     await addPluginV1(umi, {
//         asset: asset.publicKey,
//         plugin: createPlugin({
//             type: "Immutable"
//         })
//     }).sendAndConfirm(umi);

//     // Then an account was created with the correct data.
//     await assertAsset(t, umi, {
//         ...DEFAULT_ASSET,
//         asset: asset.publicKey,
//         owner: umi.identity.publicKey,
//         updateAuthority: { type: 'Address', address: umi.identity.publicKey },
//         immutable: { authority: { type: 'UpdateAuthority' } },
//         attributes: {
//             attributeList: [], authority: { type: 'UpdateAuthority' }
//         }
//     });

//     const result = updatePluginV1(umi, {
//         asset: asset.publicKey,
//         plugin: createPlugin({
//             type: "Attributes",
//             data: {
//                 attributeList: [{
//                     key: "key",
//                     value: "value"
//                 }]
//             }
//         })
//     }).sendAndConfirm(umi);

//     await t.throwsAsync(result, { name: 'ImmutableAsset' });
// });

// test('it can update an owner-managed plugin on an immutable asset', async (t) => {
//     // Given a Umi instance and a new signer.
//     const umi = await createUmi();
//     const delegate = await generateSigner(umi);

//     const asset = await createAsset(umi, {
//         plugins: [
//             pluginAuthorityPair({
//                 type: 'Immutable',
//             }),
//             pluginAuthorityPair({
//                 type: 'FreezeDelegate',
//                 authority: addressPluginAuthority(delegate.publicKey),
//                 data: {
//                     frozen: false
//                 }
//             }),
//         ],
//     });

//     // Then an account was created with the correct data.
//     await assertAsset(t, umi, {
//         ...DEFAULT_ASSET,
//         asset: asset.publicKey,
//         owner: umi.identity.publicKey,
//         updateAuthority: { type: 'Address', address: umi.identity.publicKey },
//         immutable: { authority: { type: 'UpdateAuthority' } },
//         freezeDelegate: {
//             authority: {
//                 type: 'Address',
//                 address: delegate.publicKey
//             },
//             frozen: false
//         }
//     });

//     await updatePluginV1(umi, {
//         asset: asset.publicKey,
//         authority: delegate,
//         plugin: createPlugin({
//             type: "FreezeDelegate",
//             data: {
//                 frozen: true
//             }
//         }),
//     }).sendAndConfirm(umi);

//     await assertAsset(t, umi, {
//         ...DEFAULT_ASSET,
//         asset: asset.publicKey,
//         owner: umi.identity.publicKey,
//         updateAuthority: { type: 'Address', address: umi.identity.publicKey },
//         freezeDelegate: {
//             frozen: true,
//             authority: {
//                 type: "Address",
//                 address: delegate.publicKey
//             }
//         },
//         immutable: { authority: { type: 'UpdateAuthority' } }
//     });
// });

// test('it can make an existing asset immutable but can still update an owner-managed plugin', async (t) => {
//     // Given a Umi instance and a new signer.
//     const umi = await createUmi();
//     const delegate = await generateSigner(umi);
//     const asset = await createAsset(umi, {
//         plugins: [
//             pluginAuthorityPair({
//                 type: 'FreezeDelegate',
//                 authority: addressPluginAuthority(delegate.publicKey),
//                 data: {
//                     frozen: false
//                 }
//             })
//         ]
//     });

//     await addPluginV1(umi, {
//         asset: asset.publicKey,
//         plugin: createPlugin({
//             type: "Immutable"
//         })
//     }).sendAndConfirm(umi);

//     await assertAsset(t, umi, {
//         ...DEFAULT_ASSET,
//         asset: asset.publicKey,
//         owner: umi.identity.publicKey,
//         updateAuthority: { type: 'Address', address: umi.identity.publicKey },
//         immutable: { authority: { type: 'UpdateAuthority' } },
//         freezeDelegate: {
//             authority: {
//                 type: 'Address',
//                 address: delegate.publicKey
//             },
//             frozen: false
//         }
//     });

//     await updatePluginV1(umi, {
//         asset: asset.publicKey,
//         authority: delegate,
//         plugin: createPlugin({
//             type: "FreezeDelegate",
//             data: {
//                 frozen: true
//             }
//         }),
//     }).sendAndConfirm(umi);

//     await assertAsset(t, umi, {
//         ...DEFAULT_ASSET,
//         asset: asset.publicKey,
//         owner: umi.identity.publicKey,
//         updateAuthority: { type: 'Address', address: umi.identity.publicKey },
//         freezeDelegate: {
//             frozen: true,
//             authority: {
//                 type: "Address",
//                 address: delegate.publicKey
//             }
//         },
//         immutable: { authority: { type: 'UpdateAuthority' } }
//     });
// });

test('it can still transfer an immutable asset', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const newOwner = generateSigner(umi);

    const asset = await createAsset(umi, {
        plugins: [
            pluginAuthorityPair({
                type: 'Immutable',
            }),
        ],
    });
    await assertAsset(t, umi, {
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });

    await transferV1(umi, {
        asset: asset.publicKey,
        newOwner: newOwner.publicKey,
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        asset: asset.publicKey,
        owner: newOwner.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });
});

test('it can make an existing asset immutable but can still transfer the asset', async (t) => {
    // Given a Umi instance and a new signer.
    const umi = await createUmi();
    const asset = await createAsset(umi);
    const newOwner = generateSigner(umi);

    await addPluginV1(umi, {
        asset: asset.publicKey,
        plugin: createPlugin({
            type: "Immutable"
        })
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        ...DEFAULT_ASSET,
        asset: asset.publicKey,
        owner: umi.identity.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });

    await transferV1(umi, {
        asset: asset.publicKey,
        newOwner: newOwner.publicKey,
    }).sendAndConfirm(umi);

    await assertAsset(t, umi, {
        asset: asset.publicKey,
        owner: newOwner.publicKey,
        updateAuthority: { type: 'Address', address: umi.identity.publicKey },
        immutable: { authority: { type: 'UpdateAuthority' } }
    });
});