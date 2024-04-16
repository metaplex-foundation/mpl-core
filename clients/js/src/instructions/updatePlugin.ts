import { Context } from "@metaplex-foundation/umi";
import { updatePluginV1, updateExternalPluginV1, PluginType } from "../generated";
import { createExternalPluginUpdateInfo, CreatePluginArgsV2, createPluginV2, externalPluginKeyToBase, isExternalPluginType } from "../plugins";
import { ExternalPluginUpdateInfoArgs } from "../plugins/externalPlugins";

export type UpdatePluginArgs = Omit<Parameters<typeof updatePluginV1>[1], 'plugin'> & {
  plugin: CreatePluginArgsV2 | ExternalPluginUpdateInfoArgs
}

export const updatePlugin = (
  context: Pick<Context, 'payer' | 'programs' | 'identity'>, {
    plugin,
    ...args
  }: UpdatePluginArgs) => {
    if (isExternalPluginType(plugin)) {
      const plug = plugin as ExternalPluginUpdateInfoArgs
      return updateExternalPluginV1(context, {
        ...args,
        updateInfo: createExternalPluginUpdateInfo(plug),
        key: externalPluginKeyToBase(plug.key),
      })
    }

    return updatePluginV1(context, {
      ...args,
      plugin: createPluginV2(plugin as CreatePluginArgsV2),
    })
  }