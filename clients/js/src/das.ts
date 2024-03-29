import {
  AccountHeader,
  BigIntInput,
  lamports,
  none,
  PublicKey,
  publicKey,
  some,
  Umi,
} from '@metaplex-foundation/umi';
import {
  DasApiAsset,
  DasApiAssetAuthority,
  DasApiAssetGrouping,
  DasApiAssetInterface,
  SearchAssetsRpcInput,
} from '@metaplex-foundation/digital-asset-standard-api';
import {
  AssetV1,
  Attributes,
  BurnDelegate,
  CollectionV1,
  FreezeDelegate,
  Key,
  MPL_CORE_PROGRAM_ID,
  PermanentBurnDelegate,
  PermanentFreezeDelegate,
  PermanentTransferDelegate,
  Royalties,
  RuleSet,
  ruleSet,
  TransferDelegate,
  UpdateDelegate,
} from './generated';
import { BaseUpdateAuthority, PluginsList } from './types';

type Pagination = Pick<
  SearchAssetsRpcInput,
  'sortBy' | 'limit' | 'page' | 'before' | 'after'
>;

const MPL_CORE_ASSET = 'MplCoreAsset';
const MPL_CORE_COLLECTION = 'MplCoreCollection';

function convertSnakeCase(str: string, toCase: 'pascal' | 'camel' = 'camel') {
  return str
    .toLowerCase()
    .split('_')
    .map((word, index) =>
      toCase === 'camel' && index === 0
        ? word
        : word.charAt(0).toUpperCase() + word.slice(1)
    )
    .join('');
}

function getUpdateAuthority(
  groupingItem: DasApiAssetGrouping | undefined,
  authority: DasApiAssetAuthority
): Record<'updateAuthority', BaseUpdateAuthority> {
  const result: { updateAuthority: BaseUpdateAuthority } = {
    updateAuthority: { type: 'None' },
  };

  if (groupingItem && groupingItem.group_key === 'collection') {
    result.updateAuthority = {
      type: 'Collection',
      address: publicKey(groupingItem.group_value),
    };
  } else {
    result.updateAuthority = {
      type: 'Address',
      address: authority.address,
    };
  }

  return result;
}

function getAccountHeader(
  executable?: boolean,
  lamps?: BigIntInput,
  rentEpoch?: number
): Record<'header', AccountHeader> {
  return {
    header: {
      executable: executable ?? false,
      owner: MPL_CORE_PROGRAM_ID,
      lamports: lamports(lamps ?? -1),
      ...(rentEpoch !== undefined ? { rentEpoch } : {}),
    },
  };
}

function getRuleSet(dasRuleSet: string | Record<string, any>): RuleSet {
  const isRuleSetString = typeof dasRuleSet === 'string';
  const ruleSetKind = isRuleSetString ? dasRuleSet : Object.keys(dasRuleSet)[0];
  const ruleSetData: PublicKey[] = !isRuleSetString
    ? dasRuleSet[ruleSetKind].map((bytes: Uint8Array) =>
        publicKey(new Uint8Array(bytes))
      )
    : [];

  if (ruleSetKind === 'program_allow_list') {
    return ruleSet('ProgramAllowList', [ruleSetData]);
  }

  if (ruleSetKind === 'program_deny_list') {
    return ruleSet('ProgramDenyList', [ruleSetData]);
  }

  return ruleSet('None');
}

function dasPluginDataToCorePluginData(
  dasPluginData: Record<string, any>
):
  | Royalties
  | FreezeDelegate
  | BurnDelegate
  | TransferDelegate
  | UpdateDelegate
  | PermanentFreezeDelegate
  | Attributes
  | PermanentTransferDelegate
  | PermanentBurnDelegate {
  return (({
    basis_points,
    creators,
    rule_set,
    attribute_list,
    frozen,
    additional_delegates,
  }) => ({
    ...(basis_points !== undefined ? { basisPoints: basis_points } : {}),
    ...(creators !== undefined
      ? {
          creators: creators.map((creator: any) => ({
            ...creator,
            address: publicKey(creator.address),
          })),
        }
      : {}),
    ...(rule_set !== undefined ? { ruleSet: getRuleSet(rule_set) } : {}),
    ...(attribute_list !== undefined ? { attributeList: attribute_list } : {}),
    ...(frozen !== undefined ? { frozen } : {}),
    ...(additional_delegates !== undefined
      ? { additionalDelegates: additional_delegates }
      : {}),
  }))(dasPluginData);
}

export function dasPluginsToCorePlugins(dasPlugins: Record<string, any>) {
  return Object.keys(dasPlugins).reduce((acc: PluginsList, dasPluginKey) => {
    const dasPlugin = dasPlugins[dasPluginKey];
    const { authority, data, offset } = dasPlugin;
    const authorityAddress = authority?.address;

    acc = {
      ...acc,
      [convertSnakeCase(dasPluginKey)]: {
        authority: {
          type: authority.type,
          ...(authorityAddress ? { address: publicKey(authorityAddress) } : {}),
        },
        ...dasPluginDataToCorePluginData(data),
        offset: BigInt(offset),
      },
    };

    return acc;
  }, {});
}

function dasAssetToCoreAssetOrCollection(
  dasAsset: DasApiAsset
): AssetV1 | CollectionV1 {
  // TODO: Define types in Umi DAS client.
  const {
    interface: assetInterface,
    id,
    ownership: { owner },
    content: {
      metadata: { name },
      json_uri: uri,
    },
    compression: { seq },
    grouping,
    authorities,
    plugins,
    executable,
    lamports: lamps,
    rent_epoch: rentEpoch,
    mpl_core_info: mplCoreInfo,
  } = dasAsset as DasApiAsset & {
    plugins: Record<string, any>;
    executable?: boolean;
    lamports?: number;
    rent_epoch?: number;
    mpl_core_info?: {
      num_minted?: number;
      current_size?: number;
      plugins_json_version: number;
    };
  };
  const { num_minted: numMinted = 0, current_size: currentSize = 0 } =
    mplCoreInfo ?? {};
  const commonFields = {
    publicKey: id,
    uri,
    name,
    ...getAccountHeader(executable, lamps, rentEpoch),
    ...dasPluginsToCorePlugins(plugins),
    // pluginHeader: // TODO: Reconstruct
  };

  const isCollection =
    assetInterface === (MPL_CORE_COLLECTION as DasApiAssetInterface);
  if (isCollection) {
    return {
      ...commonFields,
      key: Key.CollectionV1,
      updateAuthority: authorities[0].address,
      numMinted,
      currentSize,
    };
  }

  return {
    ...commonFields,
    key: Key.AssetV1,
    owner,
    seq: seq ? some(BigInt(seq)) : none(),
    ...getUpdateAuthority(grouping[0], authorities[0]),
  };
}

async function searchAssets(
  context: Umi,
  input: Omit<SearchAssetsRpcInput, 'interface' | 'burnt'> & {
    interface?: typeof MPL_CORE_ASSET;
  }
): Promise<AssetV1[]>;
async function searchAssets(
  context: Umi,
  input: Omit<SearchAssetsRpcInput, 'interface' | 'burnt'> & {
    interface?: typeof MPL_CORE_COLLECTION;
  }
): Promise<CollectionV1[]>;
async function searchAssets(
  context: Umi,
  input: Omit<SearchAssetsRpcInput, 'interface' | 'burnt'> & {
    interface?: typeof MPL_CORE_ASSET | typeof MPL_CORE_COLLECTION;
  }
) {
  const dasAssets = await context.rpc.searchAssets({
    ...input,
    interface: (input.interface ?? MPL_CORE_ASSET) as DasApiAssetInterface,
    burnt: false,
  });

  return dasAssets.items.map((dasAsset) =>
    dasAssetToCoreAssetOrCollection(dasAsset)
  );
}

function searchCollections(
  context: Umi,
  input: Omit<SearchAssetsRpcInput, 'interface' | 'burnt'>
) {
  return searchAssets(context, { ...input, interface: MPL_CORE_COLLECTION });
}

function fetchAssetsByOwner(
  context: Umi,
  input: {
    owner: PublicKey;
  } & Pagination
) {
  return searchAssets(context, {
    owner: input.owner,
  });
}

function fetchAssetsByAuthority(
  context: Umi,
  input: {
    authority: PublicKey;
  } & Pagination
) {
  return searchAssets(context, {
    ...input,
    authority: input.authority,
  });
}

function fetchAssetsByCollection(
  context: Umi,
  input: {
    collection: PublicKey;
  } & Pagination
) {
  return searchAssets(context, {
    ...input,
    grouping: ['collection', input.collection],
  });
}

function fetchCollectionsByUpdateAuthority(
  context: Umi,
  input: {
    updateAuthority: PublicKey;
  } & Pagination
) {
  return searchCollections(context, {
    ...input,
    authority: input.updateAuthority,
  });
}

export const das = {
  searchAssets,
  searchCollections,
  fetchAssetsByOwner,
  fetchAssetsByAuthority,
  fetchAssetsByCollection,
  fetchCollectionsByUpdateAuthority,
} as const;
