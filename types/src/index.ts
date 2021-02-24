import type { OverrideBundleType, OverrideBundleDefinition } from '@polkadot/types/types';

import { types as v1 } from "./v1";

export const definition: OverrideBundleDefinition = {
  types: [
    {
      minmax: [0, undefined],
      types: v1
    }
  ]
}

export const bundle: OverrideBundleType = {
  spec: {
    snowbridge: definition
  }
}
