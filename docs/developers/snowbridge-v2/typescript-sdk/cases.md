---
description: Route and example coverage for the Snowbridge V2 TypeScript SDK.
---

# SDK Cases

This page lists the route cases currently covered by the operations testcase matrix in [web/packages/operations/src/testcases/testAll.ts](../../../../web/packages/operations/src/testcases/testAll.ts).

## Ethereum -> Polkadot

Examples covered:

* `ethereum:1 -> polkadot:1000` with `DOT`
* `ethereum:1 -> polkadot:2000` with `ETH`
* `ethereum:1 -> polkadot:2004` with `WETH`
* `ethereum:1 -> polkadot:2030` with `ETH`
* `ethereum:1 -> polkadot:2034` with `USDC`
* `ethereum:1 -> polkadot:2043` with `TRAC`
* `ethereum:1 -> polkadot:3369` with `MYTH`

## Polkadot -> Ethereum

Examples covered:

* `polkadot:1000 -> ethereum:1` with `DOT`
* `polkadot:2000 -> ethereum:1` with `ETH`
* `polkadot:2004 -> ethereum:1` with `WETH`
* `polkadot:2030 -> ethereum:1` with `ETH`
* `polkadot:2034 -> ethereum:1` with `USDC`
* `polkadot:2043 -> ethereum:1` with `TRAC`
* `polkadot:3369 -> ethereum:1` with `MYTH`
* `ethereum:1284 -> ethereum:1` with `WETH`

## L2 -> Polkadot

Examples covered:

* `ethereum_l2:10 -> polkadot:1000` with `ETH`
* `ethereum_l2:42161 -> polkadot:1000` with `WETH`
* `ethereum_l2:8453 -> polkadot:1000` with `USDC`

## Polkadot -> L2

Examples covered:

* `polkadot:1000 -> ethereum_l2:10` with `ETH`
* `polkadot:1000 -> ethereum_l2:42161` with `WETH`
* `polkadot:1000 -> ethereum_l2:8453` with `USDC`

## Inter-Parachain

Examples covered:

* `polkadot:1000 -> polkadot:2034` with `USDC`
* `polkadot:2034 -> polkadot:1000` with `USDC`

## Registration

Examples covered:

* Agent creation with `api.createAgent()`
* Token registration with `api.registerToken()`

## Common SDK Pattern

Most route cases follow the same pattern:

```typescript
const sender = api.sender(from, to)
const transfer = await sender.build(
    sourceAccount,
    beneficiaryAccount,
    tokenAddress,
    amount,
    options,
)
```

The returned `transfer.tx` can then be submitted to the wallet by your application.
