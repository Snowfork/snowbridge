export type FeeAsset = { amount: bigint; symbol: string }
export type FeeItem = { description: string } & FeeAsset

export type DeliveryFee<K extends string> = {
    breakdown: { [P in K]?: FeeAsset[] }
    summary: FeeItem[]
    totals: FeeAsset[]
}

// Volume fee deposited to `recipient` on Asset Hub instead of added to the
// relayerFee. `amount` uses the collection asset of the route, which is ether on
// Asset Hub for Ethereum<->Polkadot routes.
export type ServiceFee = {
    recipient: string
    amount: bigint
}

// ethereum → polkadot (V2)
export type ToPolkadotFeeKey =
    | "assetHubDelivery"
    | "assetHubExecution"
    | "destinationDelivery"
    | "destinationExecution"
    | "relayer"
    | "extrinsic"
    | "serviceFee"

// ethereum_l2 → polkadot
export type L2ToPolkadotFeeKey = ToPolkadotFeeKey | "l2Bridge" | "l1Swap"

// polkadot → ethereum (and polkadot → ethereum_l2)
export type ToEthereumFeeKey =
    | "localExecution"
    | "localDelivery"
    | "assetHubExecution"
    | "bridgeHubDelivery"
    | "snowbridgeDelivery"
    | "returnToSenderDelivery"
    | "returnToSenderExecution"
    | "ethereumExecution"
    | "l2Bridge"
    | "serviceFee"

// polkadot → polkadot (inter-parachain)
export type InterParachainFeeKey = "assetHubDelivery" | "destinationExecution"

// kusama ↔ polkadot
export type KusamaFeeKey = "xcmBridge" | "bridgeHubDelivery" | "destinationExecution" | "serviceFee"

// v1 ethereum → polkadot (legacy adapter)
export type V1ToPolkadotFeeKey = "destinationDelivery" | "destinationExecution"
