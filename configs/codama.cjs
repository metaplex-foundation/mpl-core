// @ts-check
const { rootNodeFromAnchor } = require("@codama/nodes-from-anchor");
const { renderJavaScriptVisitor } = require("@codama/renderers");
const {
  accountNode,
  arrayValueNode,
  bottomUpTransformerVisitor,
  bytesTypeNode,
  constantPdaSeedNodeFromString,
  createFromRoot,
  enumValueNode,
  noneValueNode,
  pdaValueNode,
  programNode,
  publicKeyTypeNode,
  publicKeyValueNode,
  setAccountDiscriminatorFromFieldVisitor,
  structFieldTypeNode,
  structTypeNode,
  updateAccountsVisitor,
  updateDefinedTypesVisitor,
  updateInstructionsVisitor,
  updateProgramsVisitor,
  variablePdaSeedNode,
} = require("codama");
const { writeFileSync } = require("node:fs");
const path = require("node:path");

// Paths.
const idlDir = path.join(__dirname, "..", "idls");
const anchorPath = path.join(idlDir, "mpl_core.json");
const anchorIdl = require(anchorPath);
const codamaTreePath = path.join(__dirname, "codama-tree.json");
const clientDir = path.join(__dirname, "..", "clients");

// Instantiate Codama.
const codama = createFromRoot(rootNodeFromAnchor(anchorIdl));

// Update programs.
codama.update(
  updateProgramsVisitor({
    mplCoreProgram: { name: "mplCore" },
  })
);

// Append empty signer accounts.
codama.update(
  bottomUpTransformerVisitor([
    {
      select: (nodePath) => {
        const node = nodePath[nodePath.length - 1];
        return (
          node.kind === "programNode" && node.name === "mplCore"
        );
      },
      transform: (node) => {
        if (node.kind !== "programNode") return node;
        return programNode({
          ...node,
          accounts: [
            ...node.accounts,
            accountNode({
              name: "assetSigner",
              size: 0,
              data: structTypeNode([
                structFieldTypeNode({
                  name: "data",
                  type: bytesTypeNode(),
                }),
              ]),
            }),
          ],
        });
      },
    },
  ])
);

codama.update(
  updateAccountsVisitor({
    assetV1: {
      name: "baseAssetV1",
    },
    collectionV1: {
      name: "baseCollectionV1",
    },
    assetSigner: {
      size: 0,
      seeds: [
        constantPdaSeedNodeFromString("utf8", "mpl-core-execute"),
        variablePdaSeedNode(
          "asset",
          publicKeyTypeNode(),
          "The address of the asset account"
        ),
      ],
    },
  })
);

codama.update(
  updateDefinedTypesVisitor({
    authority: {
      name: "pluginAuthority",
    },
  })
);

// Update instructions with default values
codama.update(
  updateInstructionsVisitor({
    transferV1: {
      arguments: {
        compressionProof: {
          defaultValue: noneValueNode(),
        },
      },
    },
    addPluginV1: {
      arguments: {
        initAuthority: {
          defaultValue: noneValueNode(),
        },
      },
    },
    addCollectionPluginV1: {
      arguments: {
        initAuthority: {
          defaultValue: noneValueNode(),
        },
      },
    },
    burnV1: {
      arguments: {
        compressionProof: {
          defaultValue: noneValueNode(),
        },
      },
    },
    createV1: {
      arguments: {
        plugins: {
          defaultValue: arrayValueNode([]),
        },
        dataState: {
          defaultValue: enumValueNode("DataState", "AccountState"),
        },
      },
    },
    createV2: {
      arguments: {
        plugins: {
          defaultValue: arrayValueNode([]),
        },
        externalPluginAdapters: {
          defaultValue: arrayValueNode([]),
        },
        dataState: {
          defaultValue: enumValueNode("DataState", "AccountState"),
        },
      },
    },
    createCollectionV1: {
      arguments: {
        plugins: {
          defaultValue: noneValueNode(),
        },
      },
    },
    createCollectionV2: {
      arguments: {
        plugins: {
          defaultValue: noneValueNode(),
        },
        externalPluginAdapters: {
          defaultValue: arrayValueNode([]),
        },
      },
    },
    collect: {
      accounts: {
        recipient1: {
          defaultValue: publicKeyValueNode(
            "8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v"
          ),
        },
        recipient2: {
          defaultValue: publicKeyValueNode(
            "MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt"
          ),
        },
      },
    },
    updateV1: {
      arguments: {
        newUpdateAuthority: {
          defaultValue: noneValueNode(),
        },
        newName: {
          defaultValue: noneValueNode(),
        },
        newUri: {
          defaultValue: noneValueNode(),
        },
      },
    },
    updateV2: {
      arguments: {
        newUpdateAuthority: {
          defaultValue: noneValueNode(),
        },
        newName: {
          defaultValue: noneValueNode(),
        },
        newUri: {
          defaultValue: noneValueNode(),
        },
      },
    },
    updateCollectionV1: {
      arguments: {
        newName: {
          defaultValue: noneValueNode(),
        },
        newUri: {
          defaultValue: noneValueNode(),
        },
      },
    },
    executeV1: {
      accounts: {
        assetSigner: {
          defaultValue: pdaValueNode("assetSigner"),
        },
      },
    },
  })
);

// Set ShankAccount discriminator.
function key(name) {
  return {
    field: "key",
    value: enumValueNode("Key", name),
  };
}
codama.update(
  setAccountDiscriminatorFromFieldVisitor({
    assetV1: key("AssetV1"),
    collectionV1: key("CollectionV1"),
  })
);

// Render tree.
writeFileSync(
  codamaTreePath,
  JSON.stringify(JSON.parse(codama.getJson()), null, 2)
);

// Rewrite the account names for custom account data
codama.update(
  updateAccountsVisitor({
    baseAssetV1: {
      name: "assetV1",
    },
    baseCollectionV1: {
      name: "collectionV1",
    },
  })
);

codama.update(
  updateDefinedTypesVisitor({
    ruleSet: {
      name: "baseRuleSet",
    },
    royalties: {
      name: "baseRoyalties",
    },
    pluginAuthority: {
      name: "basePluginAuthority",
    },
    updateAuthority: {
      name: "baseUpdateAuthority",
    },
    seed: {
      name: "baseSeed",
    },
    extraAccount: {
      name: "baseExtraAccount",
    },
    externalPluginAdapterKey: {
      name: "baseExternalPluginAdapterKey",
    },
    linkedDataKey: {
      name: "baseLinkedDataKey",
    },
    externalPluginAdapterInitInfo: {
      name: "baseExternalPluginAdapterInitInfo",
    },
    externalPluginAdapterUpdateInfo: {
      name: "baseExternalPluginAdapterUpdateInfo",
    },
    oracle: {
      name: "baseOracle",
    },
    oracleInitInfo: {
      name: "baseOracleInitInfo",
    },
    oracleUpdateInfo: {
      name: "baseOracleUpdateInfo",
    },
    lifecycleHook: {
      name: "baseLifecycleHook",
    },
    lifecycleHookInitInfo: {
      name: "baseLifecycleHookInitInfo",
    },
    lifecycleHookUpdateInfo: {
      name: "baseLifecycleHookUpdateInfo",
    },
    linkedLifecycleHook: {
      name: "baseLinkedLifecycleHook",
    },
    linkedLifecycleHookInitInfo: {
      name: "baseLinkedLifecycleHookInitInfo",
    },
    linkedLifecycleHookUpdateInfo: {
      name: "baseLinkedLifecycleHookUpdateInfo",
    },
    appData: {
      name: "baseAppData",
    },
    appDataInitInfo: {
      name: "baseAppDataInitInfo",
    },
    appDataUpdateInfo: {
      name: "baseAppDataUpdateInfo",
    },
    linkedAppData: {
      name: "baseLinkedAppData",
    },
    linkedAppDataInitInfo: {
      name: "baseLinkedAppDataInitInfo",
    },
    linkedAppDataUpdateInfo: {
      name: "baseLinkedAppDataUpdateInfo",
    },
    dataSection: {
      name: "baseDataSection",
    },
    dataSectionInitInfo: {
      name: "baseDataSectionInitInfo",
    },
    dataSectionUpdateInfo: {
      name: "baseDataSectionUpdateInfo",
    },
    validationResultsOffset: {
      name: "baseValidationResultsOffset",
    },
    masterEdition: {
      name: "baseMasterEdition",
    },
  })
);

// Render JavaScript (to js-kit folder for Solana Kit-based client).
const jsKitDir = path.join(clientDir, "js-kit", "src", "generated");
const prettierConfig = require(path.join(clientDir, "js", ".prettierrc.json"));
codama.accept(
  renderJavaScriptVisitor(jsKitDir, {
    prettierOptions: prettierConfig,
  })
);
