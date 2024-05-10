import { AdapterRegistryRecord } from 'src/generated';
import { PluginAdapterTypeString } from './pluginAdapters';

export type PluginAdapterManifest<
  T extends Object,
  Base extends Object,
  Init extends Object,
  InitBase extends Object,
  Update extends Object,
  UpdateBase extends Object,
> = {
  type: PluginAdapterTypeString;
  fromBase: (
    input: Base,
    record: AdapterRegistryRecord,
    account: Uint8Array
  ) => T;
  initToBase: (input: Init) => InitBase;
  updateToBase: (input: Update) => UpdateBase;
};
