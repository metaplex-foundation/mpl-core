import { type ExternalRegistryRecord } from '../generated';
import { type ExternalPluginAdapterTypeString } from './externalPluginAdapters';

export type ExternalPluginAdapterManifest<
  T extends object,
  Base extends object,
  Init extends object,
  InitBase extends object,
  Update extends object,
  UpdateBase extends object,
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
