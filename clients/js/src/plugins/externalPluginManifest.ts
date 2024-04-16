import { BaseExternalPluginKeyArgs } from "../generated"
import { ExternalPluginType } from "./externalPlugins"

export type ExternalPluginManifest<
  Init extends Object,
  InitBase extends Object,
  Update extends Object,
  UpdateBase extends Object
> = {
  type: ExternalPluginType,
  initToBase: (input: Init) => InitBase,
  updateToBase: (input: Update) => UpdateBase
}

