import type {
  OverrideBundleType,
  OverrideBundleDefinition,
} from "@polkadot/types/types";

import { alias, types as typesV1 } from "./v1";

export const definition: OverrideBundleDefinition = {
  alias: alias,
  types: [
    {
      minmax: [0, undefined],
      types: typesV1,
    },
  ],
};

export const bundle: OverrideBundleType = {
  spec: {
    snowbridge: definition,
  },
};
