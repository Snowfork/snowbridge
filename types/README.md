# Snowbridge Types

Type definitions for the Snowbridge parachain

## Usage

Import the types in your JS or TS app:

```ts
import { ApiPromise } from '@polkadot/api';
import { bundle } from "@snowfork/snowbridge-types";

const makeAPI = async (provider) =>
    ApiPromise.create({ provider, typesBundle: bundle });
```
