import { AccountMeta, Context, PublicKey } from '@metaplex-foundation/umi';
import { ExternalPluginAdaptersList } from '../generated/plugins/registry';
import { LifecycleEvent } from './lifecycleChecks';
import { extraAccountToAccountMeta } from './extraAccount';

export * from '../generated/plugins/registry';

export const findExtraAccounts = (
  context: Pick<Context, 'eddsa'>,
  lifecycle: LifecycleEvent,
  externalPluginAdapters: ExternalPluginAdaptersList,
  inputs: {
    asset: PublicKey;
    collection?: PublicKey;
    owner: PublicKey;
    recipient?: PublicKey;
  }
): AccountMeta[] => {
  const accounts: AccountMeta[] = [];

  externalPluginAdapters.oracles?.forEach((oracle) => {
    if (oracle.lifecycleChecks?.[lifecycle]) {
      if (oracle.baseAddressConfig) {
        accounts.push(
          extraAccountToAccountMeta(context, oracle.baseAddressConfig, {
            ...inputs,
            program: oracle.baseAddress,
          })
        );
      } else {
        accounts.push({
          pubkey: oracle.baseAddress,
          isSigner: false,
          isWritable: false,
        });
      }
    }
  });

  externalPluginAdapters.lifecycleHooks?.forEach((hook) => {
    if (hook.lifecycleChecks?.[lifecycle]) {
      accounts.push({
        pubkey: hook.hookedProgram,
        isSigner: false,
        isWritable: false,
      });

      hook.extraAccounts?.forEach((extra) => {
        accounts.push(
          extraAccountToAccountMeta(context, extra, {
            ...inputs,
            program: hook.hookedProgram,
          })
        );
      });
    }
  });

  externalPluginAdapters.linkedLifecycleHooks?.forEach((hook) => {
    if (hook.lifecycleChecks?.[lifecycle]) {
      accounts.push({
        pubkey: hook.hookedProgram,
        isSigner: false,
        isWritable: false,
      });

      hook.extraAccounts?.forEach((extra) => {
        accounts.push(
          extraAccountToAccountMeta(context, extra, {
            ...inputs,
            program: hook.hookedProgram,
          })
        );
      });
    }
  });

  return accounts;
};
