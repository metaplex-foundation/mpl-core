// Kit generation is additive: the existing Kinobi/Umi/Rust pipeline stays separate.
// Instruction defaults follow configs/kinobi.cjs; see also PR #250 by Tony Boyle (@tonyboylehub).
const { appendFileSync } = require("node:fs");
const path = require("node:path");
const k = require("codama");
const { rootNodeFromAnchor } = require("@codama/nodes-from-anchor");
const { renderVisitor } = require("@codama/renderers-js");
const clientDir = path.join(__dirname, "..", "clients", "js-kit");
const codama = k.createFromRoot(
    rootNodeFromAnchor(require("../idls/mpl_core.json")),
);
codama.update(k.updateProgramsVisitor({ mplCoreProgram: { name: "mplCore" } }));
codama.update(
    k.bottomUpTransformerVisitor([
        {
            select: "[programNode]",
            transform: (node) =>
                k.programNode({
                    ...node,
                    accounts: [
                        ...node.accounts,
                        k.accountNode({
                            data: k.structTypeNode([]),
                            name: "assetSigner",
                            pda: k.pdaLinkNode("assetSigner"),
                            size: 0,
                        }),
                    ],
                    pdas: [
                        ...(node.pdas ?? []),
                        k.pdaNode({
                            name: "assetSigner",
                            seeds: [
                                k.constantPdaSeedNodeFromString(
                                    "utf8",
                                    "mpl-core-execute",
                                ),
                                k.variablePdaSeedNode(
                                    "asset",
                                    k.publicKeyTypeNode(),
                                    "The address of the asset account",
                                ),
                            ],
                        }),
                    ],
                }),
        },
    ]),
);
codama.update(
    k.updateDefinedTypesVisitor({
        authority: { name: "pluginAuthority" },
        crate: { name: "relationshipEntry" },
    }),
);
// Update instructions with default values
codama.update(
    k.updateInstructionsVisitor({
        addCollectionPluginV1: {
            arguments: {
                initAuthority: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        addPluginV1: {
            arguments: {
                initAuthority: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        burnV1: {
            arguments: {
                compressionProof: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        collect: {
            accounts: {
                recipient1: {
                    defaultValue: k.publicKeyValueNode(
                        "8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v",
                    ),
                },
                recipient2: {
                    defaultValue: k.publicKeyValueNode(
                        "MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt",
                    ),
                },
            },
        },
        createCollectionV1: {
            arguments: {
                plugins: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        createCollectionV2: {
            arguments: {
                externalPluginAdapters: {
                    defaultValue: k.arrayValueNode([]),
                },
                plugins: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        createV1: {
            arguments: {
                dataState: {
                    defaultValue: k.enumValueNode("DataState", "AccountState"),
                },
                plugins: {
                    defaultValue: k.arrayValueNode([]),
                },
            },
        },
        createV2: {
            arguments: {
                dataState: {
                    defaultValue: k.enumValueNode("DataState", "AccountState"),
                },
                externalPluginAdapters: {
                    defaultValue: k.arrayValueNode([]),
                },
                plugins: {
                    defaultValue: k.arrayValueNode([]),
                },
            },
        },
        executeV1: {
            accounts: {
                assetSigner: {
                    defaultValue: k.pdaValueNode("assetSigner"),
                },
            },
        },
        transferV1: {
            arguments: {
                compressionProof: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        updateCollectionV1: {
            arguments: {
                newName: {
                    defaultValue: k.noneValueNode(),
                },
                newUri: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        updateGroupV1: {
            arguments: {
                newName: {
                    defaultValue: k.noneValueNode(),
                },
                newUri: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        updateV1: {
            arguments: {
                newName: {
                    defaultValue: k.noneValueNode(),
                },
                newUpdateAuthority: {
                    defaultValue: k.noneValueNode(),
                },
                newUri: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
        updateV2: {
            arguments: {
                newName: {
                    defaultValue: k.noneValueNode(),
                },
                newUpdateAuthority: {
                    defaultValue: k.noneValueNode(),
                },
                newUri: {
                    defaultValue: k.noneValueNode(),
                },
            },
        },
    }),
);
const key = (name) => ({ field: "key", value: k.enumValueNode("Key", name) });
codama.update(
    k.setAccountDiscriminatorFromFieldVisitor({
        assetV1: key("AssetV1"),
        collectionV1: key("CollectionV1"),
        groupV1: key("GroupV1"),
        hashedAssetV1: key("HashedAssetV1"),
        pluginHeaderV1: key("PluginHeaderV1"),
        pluginRegistryV1: key("PluginRegistryV1"),
    }),
);
codama
    .accept(
        renderVisitor(clientDir, {
            customAccountData: ["assetV1", "collectionV1", "groupV1"].map(
                (name) => ({
                    extract: true,
                    extractAs: `base${name[0].toUpperCase()}${name.slice(1)}AccountData`,
                    name,
                }),
            ),
            importExtension: "js",
        }),
    )
    .then(() => {
        // Codama 2.8 omits custom account codecs from the account barrel, while its
        // generated program client imports them. Keep this bridge in generation.
        for (const name of ["Asset", "Collection", "Group"]) {
            appendFileSync(
                path.join(clientDir, "src/generated/accounts/index.ts"),
                `export {\n  get${name}V1AccountDataCodec as get${name}V1Codec,\n  get${name}V1AccountDataDecoder as get${name}V1Decoder,\n  get${name}V1AccountDataEncoder as get${name}V1Encoder,\n  type ${name}V1AccountData as ${name}V1,\n  type ${name}V1AccountDataArgs as ${name}V1Args,\n} from '../../hooked/index.js';\n`,
            );
        }
    })
    .catch((error) => {
        console.error(error);
        process.exitCode = 1;
    });
