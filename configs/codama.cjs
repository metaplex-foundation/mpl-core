const { rootNodeFromAnchor } = require("@codama/nodes-from-anchor");
const {
  renderJavaScriptUmiVisitor,
  renderJavaScriptVisitor,
  renderRustVisitor,
} = require("@codama/renderers");
const {
  accountValueNode,
  arrayValueNode,
  createFromRoot,
  enumValueNode,
  noneValueNode,
  pdaSeedValueNode,
  publicKeyValueNode,
  setAccountDiscriminatorFromFieldVisitor,
  updateAccountsVisitor,
  updateDefinedTypesVisitor,
  updateInstructionsVisitor,
  updateProgramsVisitor,
} = require("codama");
const path = require("node:path");
const CoreIdl = require("../idls/mpl_core.json");

// Paths.
const clientDir = path.join(__dirname, "..", "clients");

// Instanciate codama.
const codama = createFromRoot(rootNodeFromAnchor(CoreIdl));

// Update programs.
codama.update(
  updateProgramsVisitor({
    mplCoreProgram: { name: "mplCore" },
  })
);

codama.update(
  updateAccountsVisitor({
    assetV1: {
      name: "baseAssetV1",
    },
    collectionV1: {
      name: "baseCollectionV1",
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

pdaSeedValueNode("authority", accountValueNode("authority"));

// Update instructions with default values
codama.update(
  updateInstructionsVisitor({
    // create: {
    //   bytesCreatedOnChain:  bytesFromAccount("assetAccount"),
    // },
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
  })
);

// Set ShankAccount discriminator.
const key = (name) => ({
  field: "key",
  value: enumValueNode("Key", name),
});
codama.update(
  setAccountDiscriminatorFromFieldVisitor({
    assetV1: key("AssetV1"),
    collectionV1: key("CollectionV1"),
  })
);

// Render Rust.
// const crateDir = path.join(clientDir, "rust");
// const rustDir = path.join(clientDir, "rust", "src", "generated");

// renderRustVisitor(rustDir, {
//   formatCode: true,
//   crateFolder: crateDir,
// });

// rewrite the account names for custom account data

updateAccountsVisitor({
  baseAssetV1: {
    name: "assetV1",
  },
  baseCollectionV1: {
    name: "collectionV1",
  },
});

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
});

// Render JavaScript.
console.log("clientDir", clientDir);
const jsDir = path.join(clientDir, "js");
const prettier = require(path.join(clientDir, "js", "web3js2", ".prettierrc.json"));

const renderOptions = {
  prettierOptions: prettier,
  internalNodes: [],
  customAccountData: [
    {
      name: "assetV1",
      extract: true,
    },
    {
      name: "collectionV1",
      extract: true,
    },
    {
      name: "pluginRegistryV1",
      extract: true,
    },
  ],
};

// render the web3js2 client
codama.accept(
  renderJavaScriptVisitor(jsDir + "/web3js2/src/generated", renderOptions)
);

// render the umi client

// codama.accept(
//   renderJavaScriptUmiVisitor(jsDir + "/umi/src/generated", renderOptions)
// );
