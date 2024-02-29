import { UmiPlugin } from '@metaplex-foundation/umi';
import { createMplCoreProgram } from './generated';

export const mplCore = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplCoreProgram(), false);
  },
});
