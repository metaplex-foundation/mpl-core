import { UmiPlugin } from '@metaplex-foundation/umi';
import { createMplCoreProgramProgram } from './generated';

export const mplCore = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplCoreProgramProgram(), false);
  },
});
