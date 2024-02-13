import { UmiPlugin } from '@metaplex-foundation/umi';
import { createMplAssetProgram } from './generated';

export const mplAsset = (): UmiPlugin => ({
  install(umi) {
    umi.programs.add(createMplAssetProgram(), false);
  },
});
