/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/metaplex-foundation/kinobi
 */

import {
  Account,
  Context,
  Pda,
  PublicKey,
  RpcAccount,
  RpcGetAccountOptions,
  RpcGetAccountsOptions,
  assertAccountExists,
  deserializeAccount,
  gpaBuilder,
  publicKey as toPublicKey,
} from '@metaplex-foundation/umi';
import {
  Serializer,
  bytes,
  publicKey as publicKeySerializer,
  string,
  struct,
} from '@metaplex-foundation/umi/serializers';

export type AssetSigner = Account<AssetSignerAccountData>;

export type AssetSignerAccountData = { data: Uint8Array };

export type AssetSignerAccountDataArgs = AssetSignerAccountData;

export function getAssetSignerAccountDataSerializer(): Serializer<
  AssetSignerAccountDataArgs,
  AssetSignerAccountData
> {
  return struct<AssetSignerAccountData>([['data', bytes()]], {
    description: 'AssetSignerAccountData',
  }) as Serializer<AssetSignerAccountDataArgs, AssetSignerAccountData>;
}

export function deserializeAssetSigner(rawAccount: RpcAccount): AssetSigner {
  return deserializeAccount(rawAccount, getAssetSignerAccountDataSerializer());
}

export async function fetchAssetSigner(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<AssetSigner> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  assertAccountExists(maybeAccount, 'AssetSigner');
  return deserializeAssetSigner(maybeAccount);
}

export async function safeFetchAssetSigner(
  context: Pick<Context, 'rpc'>,
  publicKey: PublicKey | Pda,
  options?: RpcGetAccountOptions
): Promise<AssetSigner | null> {
  const maybeAccount = await context.rpc.getAccount(
    toPublicKey(publicKey, false),
    options
  );
  return maybeAccount.exists ? deserializeAssetSigner(maybeAccount) : null;
}

export async function fetchAllAssetSigner(
  context: Pick<Context, 'rpc'>,
  publicKeys: Array<PublicKey | Pda>,
  options?: RpcGetAccountsOptions
): Promise<AssetSigner[]> {
  const maybeAccounts = await context.rpc.getAccounts(
    publicKeys.map((key) => toPublicKey(key, false)),
    options
  );
  return maybeAccounts.map((maybeAccount) => {
    assertAccountExists(maybeAccount, 'AssetSigner');
    return deserializeAssetSigner(maybeAccount);
  });
}

export async function safeFetchAllAssetSigner(
  context: Pick<Context, 'rpc'>,
  publicKeys: Array<PublicKey | Pda>,
  options?: RpcGetAccountsOptions
): Promise<AssetSigner[]> {
  const maybeAccounts = await context.rpc.getAccounts(
    publicKeys.map((key) => toPublicKey(key, false)),
    options
  );
  return maybeAccounts
    .filter((maybeAccount) => maybeAccount.exists)
    .map((maybeAccount) => deserializeAssetSigner(maybeAccount as RpcAccount));
}

export function getAssetSignerGpaBuilder(
  context: Pick<Context, 'rpc' | 'programs'>
) {
  const programId = context.programs.getPublicKey(
    'mplCore',
    'CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'
  );
  return gpaBuilder(context, programId)
    .registerFields<{ data: Uint8Array }>({ data: [0, bytes()] })
    .deserializeUsing<AssetSigner>((account) =>
      deserializeAssetSigner(account)
    );
}

export function getAssetSignerSize(): number {
  return 0;
}

export function findAssetSignerPda(
  context: Pick<Context, 'eddsa' | 'programs'>,
  seeds: {
    /** The address of the asset account */
    asset: PublicKey;
  }
): Pda {
  const programId = context.programs.getPublicKey(
    'mplCore',
    'CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d'
  );
  return context.eddsa.findPda(programId, [
    string({ size: 'variable' }).serialize('mpl-core-execute'),
    publicKeySerializer().serialize(seeds.asset),
  ]);
}

export async function fetchAssetSignerFromSeeds(
  context: Pick<Context, 'eddsa' | 'programs' | 'rpc'>,
  seeds: Parameters<typeof findAssetSignerPda>[1],
  options?: RpcGetAccountOptions
): Promise<AssetSigner> {
  return fetchAssetSigner(context, findAssetSignerPda(context, seeds), options);
}

export async function safeFetchAssetSignerFromSeeds(
  context: Pick<Context, 'eddsa' | 'programs' | 'rpc'>,
  seeds: Parameters<typeof findAssetSignerPda>[1],
  options?: RpcGetAccountOptions
): Promise<AssetSigner | null> {
  return safeFetchAssetSigner(
    context,
    findAssetSignerPda(context, seeds),
    options
  );
}