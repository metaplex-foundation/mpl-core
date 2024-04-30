import { ExternalRegistryRecord } from 'src/generated';
import { ExternalPluginTypeString } from './externalPlugins';

export type ExternalPluginManifest<
  T extends Object,
  Base extends Object,
  Init extends Object,
  InitBase extends Object,
  Update extends Object,
  UpdateBase extends Object
> = {
  type: ExternalPluginTypeString;
  fromBase: (
    input: Base,
    record: ExternalRegistryRecord,
    account: Uint8Array
  ) => T;
  initToBase: (input: Init) => InitBase;
  updateToBase: (input: Update) => UpdateBase;
};
