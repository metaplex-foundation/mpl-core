import { ExternalRegistryRecord } from '../generated';
import { ExternalPluginAdapterTypeString } from './externalPluginAdapters';

export type ExternalPluginAdapterManifest<
  T extends Object,
  Base extends Object,
  Init extends Object,
  InitBase extends Object,
  Update extends Object,
  UpdateBase extends Object,
> = {
  type: ExternalPluginAdapterTypeString;
  fromBase: (
    input: Base,
    record: ExternalRegistryRecord,
    account: Uint8Array
  ) => T;
  initToBase: (input: Init) => InitBase;
  updateToBase: (input: Update) => UpdateBase;
};
