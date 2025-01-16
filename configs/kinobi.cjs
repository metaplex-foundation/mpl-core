const path = require("path");
const k = require("@metaplex-foundation/kinobi");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");
const idlDir = path.join(__dirname, "..", "idls");

// Instantiate Kinobi.
const kinobi = k.createFromIdls([path.join(idlDir, "mpl_core.json")]);

// Update programs.
kinobi.update(
  k.updateProgramsVisitor({
    mplCoreProgram: { name: "mplCore" },
  })
);

// Append empty signer accounts.
kinobi.update(
  new k.bottomUpTransformerVisitor([{
    select: ['[programNode]', node => 'name' in node && node.name === "mplCore"],
    transform: (node) => {
      return k.programNode({
        ...node,
        accounts: [
          ...node.accounts,
          k.accountNode({
            name: "assetSigner",
            size: 0,
            data: k.structTypeNode([
              k.structFieldTypeNode({
                name: "data",
                type: k.bytesTypeNode(k.remainderSizeNode()),
              })
            ]),
          }),
        ],
      });
    },
  }])
);


kinobi.update(
  new k.updateAccountsVisitor({
    assetV1: {
      name: "baseAssetV1",
    },
    collectionV1: {
      name: "baseCollectionV1",
    },
    assetSigner: {
      size: 0,
      seeds: [
        k.constantPdaSeedNodeFromString("mpl-core-execute"),
        k.variablePdaSeedNode(
          "asset",
          k.publicKeyTypeNode(),
          "The address of the asset account"
        ),
      ],
    },
  })
);

kinobi.update(new k.updateDefinedTypesVisitor({
  authority: {
    name: "pluginAuthority"
  }
}))

// Update instructions with default values
kinobi.update(
  k.updateInstructionsVisitor({
    // create: {
    //   bytesCreatedOnChain: k.bytesFromAccount("assetAccount"),
    // },
    transferV1: {
      arguments: {
        compressionProof: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    addPluginV1: {
      arguments: {
        initAuthority: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    addCollectionPluginV1: {
      arguments: {
        initAuthority: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    burnV1: {
      arguments: {
        compressionProof: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    createV1: {
      arguments: {
        plugins: {
          defaultValue: k.arrayValueNode([])
        },
        dataState: {
          defaultValue: k.enumValueNode('DataState', 'AccountState')
        }
      }
    },
    createV2: {
      arguments: {
        plugins: {
          defaultValue: k.arrayValueNode([])
        },
        externalPluginAdapters: {
          defaultValue: k.arrayValueNode([])
        },
        dataState: {
          defaultValue: k.enumValueNode('DataState', 'AccountState')
        }
      }
    },
    createCollectionV1: {
      arguments: {
        plugins: {
          defaultValue: k.noneValueNode()
        }
      }
    },
    createCollectionV2: {
      arguments: {
        plugins: {
          defaultValue: k.noneValueNode()
        },
        externalPluginAdapters: {
          defaultValue: k.arrayValueNode([])
        },
      }
    },
    collect: {
      accounts: {
        recipient1: {
          defaultValue: k.publicKeyValueNode("8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v")
        },
        recipient2: {
          defaultValue: k.publicKeyValueNode("MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt")
        }
      }
    },
    updateV1: {
      arguments: {
        newUpdateAuthority: {
          defaultValue: k.noneValueNode()
        },
        newName: {
          defaultValue: k.noneValueNode()
        },
        newUri: {
          defaultValue: k.noneValueNode()
        },
      }
    },
    updateV2: {
      arguments: {
        newUpdateAuthority: {
          defaultValue: k.noneValueNode()
        },
        newName: {
          defaultValue: k.noneValueNode()
        },
        newUri: {
          defaultValue: k.noneValueNode()
        },
      }
    },
    updateCollectionV1: {
      arguments: {
        newName: {
          defaultValue: k.noneValueNode()
        },
        newUri: {
          defaultValue: k.noneValueNode()
        },
      }
    },
    executeV1: {
      accounts: {
        assetSigner: {
          defaultValue: k.pdaValueNode("assetSigner")
        }
      }
    },
  })
);

// Set ShankAccount discriminator.
const key = (name) => ({ field: "key", value: k.enumValueNode("Key", name) });
kinobi.update(
  k.setAccountDiscriminatorFromFieldVisitor({
    assetV1: key("AssetV1"),
    collectionV1: key("CollectionV1"),
  })
);

// Render Rust.
const crateDir = path.join(clientDir, "rust");
const rustDir = path.join(clientDir, "rust", "src", "generated");
kinobi.accept(
  k.renderRustVisitor(rustDir, {
    formatCode: true,
    crateFolder: crateDir,
  })
);


// rewrite the account names for custom account data
kinobi.update(
  new k.updateAccountsVisitor({
    baseAssetV1: {
      name: "assetV1",
    },
    baseCollectionV1: {
      name: "collectionV1",
    }
  })
);

kinobi.update(
  new k.updateDefinedTypesVisitor({
    ruleSet: {
      name: "baseRuleSet"
    },
    royalties: {
      name: "baseRoyalties"
    },
    pluginAuthority: {
      name: "basePluginAuthority"
    },
    updateAuthority: {
      name: "baseUpdateAuthority"
    },
    seed: {
      name: "baseSeed"
    },
    extraAccount: {
      name: "baseExtraAccount"
    },
    externalPluginAdapterKey: {
      name: "baseExternalPluginAdapterKey"
    },
    linkedDataKey: {
      name: 'baseLinkedDataKey'
    },
    externalPluginAdapterInitInfo: {
      name: "baseExternalPluginAdapterInitInfo"
    },
    externalPluginAdapterUpdateInfo: {
      name: "baseExternalPluginAdapterUpdateInfo"
    },
    oracle: {
      name: "baseOracle"
    },
    oracleInitInfo: {
      name: "baseOracleInitInfo"
    },
    oracleUpdateInfo: {
      name: "baseOracleUpdateInfo"
    },
    lifecycleHook: {
      name: "baseLifecycleHook"
    },
    lifecycleHookInitInfo: {
      name: "baseLifecycleHookInitInfo"
    },
    lifecycleHookUpdateInfo: {
      name: "baseLifecycleHookUpdateInfo"
    },
    linkedLifecycleHook: {
      name: "baseLinkedLifecycleHook"
    },
    linkedLifecycleHookInitInfo: {
      name: "baseLinkedLifecycleHookInitInfo"
    },
    linkedLifecycleHookUpdateInfo: {
      name: "baseLinkedLifecycleHookUpdateInfo"
    },
    appData: {
      name: "baseAppData"
    },
    appDataInitInfo: {
      name: "baseAppDataInitInfo"
    },
    appDataUpdateInfo: {
      name: "baseAppDataUpdateInfo"
    },
    linkedAppData: {
      name: "baseLinkedAppData"
    },
    linkedAppDataInitInfo: {
      name: "baseLinkedAppDataInitInfo"
    },
    linkedAppDataUpdateInfo: {
      name: "baseLinkedAppDataUpdateInfo"
    },
    dataSection: {
      name: "baseDataSection"
    },
    dataSectionInitInfo: {
      name: "baseDataSectionInitInfo"
    },
    dataSectionUpdateInfo: {
      name: "baseDataSectionUpdateInfo"
    },
    validationResultsOffset: {
      name: "baseValidationResultsOffset"
    },
    masterEdition: {
      name: "baseMasterEdition"
    }
  })
)

// Render JavaScript.
const jsDir = path.join(clientDir, "js", "src", "generated");
const prettier = require(path.join(clientDir, "js", ".prettierrc.json"));
kinobi.accept(k.renderJavaScriptVisitor(jsDir, {
  prettier,
  internalNodes: [],
  customAccountData: [{
    name: "assetV1",
    extract: true,
  }, {
    name: "collectionV1",
    extract: true,
  }, {
    name: "pluginRegistryV1",
    extract: true,
  }],
}));