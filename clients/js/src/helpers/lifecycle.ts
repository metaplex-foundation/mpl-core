import { Context, PublicKey } from '@metaplex-foundation/umi';
import {
  AssetV1,
  CollectionV1,
  ExternalValidationResult,
  PluginType,
} from '../generated';
import { deriveAssetPlugins, isFrozen } from './state';
import { checkPluginAuthorities } from './plugin';
import { hasAssetUpdateAuthority } from './authority';
import {
  CheckResult,
  deserializeOracleValidation,
  findOracleAccount,
  getExtraAccountRequiredInputs,
} from '../plugins';

export enum LifecycleValidationError {
  OracleValidationFailed = 'Oracle validation failed.',
  NoAuthority = 'No authority to perform this action.',
  AssetFrozen = 'Asset is frozen.',
}

/**
 * Check if the given authority is eligible to transfer the asset.
 * This does NOT check the asset's royalty rule sets or external plugin adapters. Use `validateTransfer` for more comprehensive checks.
 * @deprecated since v1.0.0. Use `validateTransfer` instead.
 * @param {PublicKey | string} authority Pubkey
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the pubkey has the authority
 */
export function canTransfer(
  authority: PublicKey | string,
  asset: AssetV1,
  collection?: CollectionV1
): boolean {
  const dAsset = deriveAssetPlugins(asset, collection);

  // Permanent plugins have force approve powers
  const permaTransferDelegate = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.PermanentTransferDelegate],
    asset: dAsset,
    collection,
  });
  if (permaTransferDelegate.some((d) => d)) {
    return true;
  }

  if (isFrozen(asset, collection)) {
    return false;
  }

  if (dAsset.owner === authority) {
    return true;
  }
  const transferDelegates = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.TransferDelegate],
    asset: dAsset,
    collection,
  });
  return transferDelegates.some((d) => d);
}

export type ValidateTransferInput = {
  authority: PublicKey | string;
  asset: AssetV1;
  collection?: CollectionV1;
  recipient?: PublicKey;
};

/**
 * Check if the given authority is eligible to transfer the asset and receive an error message if not.
 *
 * @param {Context} context Umi context
 * @param {ValidateTransferInput} inputs Inputs to validate transfer
 * @returns {null | LifecycleValidationError} null if success or error message
 */
export async function validateTransfer(
  context: Pick<Context, 'eddsa' | 'rpc'>,
  { authority, asset, collection, recipient }: ValidateTransferInput
): Promise<null | LifecycleValidationError> {
  const dAsset = deriveAssetPlugins(asset, collection);

  // Permanent plugins have force approve powers
  const permaTransferDelegate = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.PermanentTransferDelegate],
    asset: dAsset,
    collection,
  });
  if (permaTransferDelegate.some((d) => d)) {
    return null;
  }

  if (isFrozen(asset, collection)) {
    return LifecycleValidationError.AssetFrozen;
  }

  if (dAsset.oracles?.length) {
    const eligibleOracles = dAsset.oracles
      .filter((o) =>
        o.lifecycleChecks?.transfer?.includes(CheckResult.CAN_REJECT)
      )
      .filter((o) => {
        // there's no PDA to derive, we can check the oracle account
        if (!o.baseAddressConfig) {
          return true;
        }
        // If there's a recipient in the inputs, we can try to check the oracle account
        if (recipient) {
          return true;
        }

        if (
          !getExtraAccountRequiredInputs(o.baseAddressConfig).includes(
            'recipient'
          )
        ) {
          return true;
        }
        // we skip the check if there's a recipient required but no recipient provided
        // this is due how UIs generally show the availability of the transfer button before requiring the recipient address
        return false;
      });
    if (eligibleOracles.length) {
      const accountsWithOffset = eligibleOracles.map((o) => {
        const account = findOracleAccount(context, o, {
          asset: asset.publicKey,
          collection: collection?.publicKey,
          owner: asset.owner,
          recipient,
        });

        return {
          pubkey: account,
          offset: o.resultsOffset,
        };
      });

      const oracleValidations = (
        await context.rpc.getAccounts(accountsWithOffset.map((a) => a.pubkey))
      ).map((a, index) => {
        if (a.exists) {
          return deserializeOracleValidation(
            a.data,
            accountsWithOffset[index].offset
          );
        }
        return null;
      });

      const oraclePass = oracleValidations.every((v) => {
        if (v?.__kind === 'Uninitialized') {
          return false;
        }
        return v?.transfer === ExternalValidationResult.Pass;
      });
      if (!oraclePass) {
        return LifecycleValidationError.OracleValidationFailed;
      }
    }
  }

  if (dAsset.owner === authority) {
    return null;
  }
  const transferDelegates = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.TransferDelegate],
    asset: dAsset,
    collection,
  });
  if (transferDelegates.some((d) => d)) {
    return null;
  }

  return LifecycleValidationError.NoAuthority;
}

/**
 * Check if the given pubkey is eligible to burn the asset.
 * This does NOT check external plugin adapters, use `validateBurn` for more comprehensive checks.
 * @deprecated since v1.0.0. Use `validateBurn` instead.
 * @param {PublicKey | string} authority Pubkey
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the pubkey has the authority
 */
export function canBurn(
  authority: PublicKey | string,
  asset: AssetV1,
  collection?: CollectionV1
): boolean {
  const dAsset = deriveAssetPlugins(asset, collection);
  const permaBurnDelegate = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.PermanentBurnDelegate],
    asset: dAsset,
    collection,
  });
  if (permaBurnDelegate.some((d) => d)) {
    return true;
  }

  if (isFrozen(asset, collection)) {
    return false;
  }

  if (dAsset.owner === authority) {
    return true;
  }
  const burnDelegates = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.BurnDelegate],
    asset,
    collection,
  });
  return burnDelegates.some((d) => d);
}

export type ValidateBurnInput = {
  authority: PublicKey | string;
  asset: AssetV1;
  collection?: CollectionV1;
};

/**
 * Check if the given authority is eligible to burn the asset and receive an error message if not.
 *
 * @param {Context} context Umi context
 * @param {ValidateBurnInput} inputs Inputs to validate burn
 * @returns {null | LifecycleValidationError} null if success or error message
 */
export async function validateBurn(
  context: Pick<Context, 'eddsa' | 'rpc'>,
  {
    authority,
    asset,
    collection,
  }: {
    authority: PublicKey | string;
    asset: AssetV1;
    collection?: CollectionV1;
  }
): Promise<null | LifecycleValidationError> {
  const dAsset = deriveAssetPlugins(asset, collection);
  const permaBurnDelegate = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.PermanentBurnDelegate],
    asset: dAsset,
    collection,
  });
  if (permaBurnDelegate.some((d) => d)) {
    return null;
  }

  if (isFrozen(asset, collection)) {
    return LifecycleValidationError.AssetFrozen;
  }

  if (dAsset.oracles?.length) {
    const eligibleOracles = dAsset.oracles.filter((o) =>
      o.lifecycleChecks?.burn?.includes(CheckResult.CAN_REJECT)
    );
    if (eligibleOracles.length) {
      const accountsWithOffset = eligibleOracles.map((o) => {
        const account = findOracleAccount(context, o, {
          asset: asset.publicKey,
          collection: collection?.publicKey,
          owner: asset.owner,
        });

        return {
          pubkey: account,
          offset: o.resultsOffset,
        };
      });

      const oracleValidations = (
        await context.rpc.getAccounts(accountsWithOffset.map((a) => a.pubkey))
      ).map((a, index) => {
        if (a.exists) {
          return deserializeOracleValidation(
            a.data,
            accountsWithOffset[index].offset
          );
        }
        return null;
      });

      const oraclePass = oracleValidations.every((v) => {
        if (v?.__kind === 'Uninitialized') {
          return false;
        }
        return v?.burn === ExternalValidationResult.Pass;
      });
      if (!oraclePass) {
        return LifecycleValidationError.OracleValidationFailed;
      }
    }
  }

  if (dAsset.owner === authority) {
    return null;
  }
  const burnDelegates = checkPluginAuthorities({
    authority,
    pluginTypes: [PluginType.BurnDelegate],
    asset,
    collection,
  });
  if (burnDelegates.some((d) => d)) {
    return null;
  }

  return LifecycleValidationError.NoAuthority;
}

/**
 * Check if the given pubkey is eligible to update the asset.
 * This does NOT check external plugin adapters. Use `validateUpdate` for more comprehensive checks.
 * @deprecated since v1.0.0. Use `validateTransfer` instead.
 * @param {PublicKey | string} authority Pubkey
 * @param {AssetV1} asset Asset
 * @param {CollectionV1 | undefined} collection Collection
 * @returns {boolean} True if the pubkey has the authority
 */
export function canUpdate(
  authority: PublicKey | string,
  asset: AssetV1,
  collection?: CollectionV1
): boolean {
  return hasAssetUpdateAuthority(authority, asset, collection);
}

export type ValidateUpdateInput = {
  authority: PublicKey | string;
  asset: AssetV1;
  collection?: CollectionV1;
};

/**
 * Check if the given authority is eligible to update the asset and receive an error message if not.
 *
 * @param {Context} context Umi context
 * @param {ValidateUpdateInput} inputs Inputs to validate update
 * @returns {null | LifecycleValidationError} null if success or error message
 */
export async function validateUpdate(
  context: Pick<Context, 'eddsa' | 'rpc'>,
  { authority, asset, collection }: ValidateUpdateInput
): Promise<null | LifecycleValidationError> {
  if (asset.oracles?.length) {
    const eligibleOracles = asset.oracles.filter((o) =>
      o.lifecycleChecks?.update?.includes(CheckResult.CAN_REJECT)
    );
    if (eligibleOracles.length) {
      const accountsWithOffset = eligibleOracles.map((o) => {
        const account = findOracleAccount(context, o, {
          asset: asset.publicKey,
          collection: collection?.publicKey,
          owner: asset.owner,
        });

        return {
          pubkey: account,
          offset: o.resultsOffset,
        };
      });

      const oracleValidations = (
        await context.rpc.getAccounts(accountsWithOffset.map((a) => a.pubkey))
      ).map((a, index) => {
        if (a.exists) {
          return deserializeOracleValidation(
            a.data,
            accountsWithOffset[index].offset
          );
        }
        return null;
      });

      const oraclePass = oracleValidations.every((v) => {
        if (v?.__kind === 'Uninitialized') {
          return false;
        }
        return v?.update === ExternalValidationResult.Pass;
      });

      if (!oraclePass) {
        return LifecycleValidationError.OracleValidationFailed;
      }
    }
  }

  if (!hasAssetUpdateAuthority(authority, asset, collection)) {
    return LifecycleValidationError.NoAuthority;
  }

  return null;
}
