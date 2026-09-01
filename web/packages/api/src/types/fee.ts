export type FeeAsset = { amount: bigint; symbol: string }
export type FeeItem = { description: string } & FeeAsset

export type DeliveryFee<K extends string> = {
    breakdown: { [P in K]?: FeeAsset[] }
    summary: FeeItem[]
    totals: FeeAsset[]
}

// A fee the message deposits to `recipient` on Asset Hub. `amount` has two sources:
// Ethereum<->Polkadot derives it from the transfer's USD value through FEE_SCHEDULE,
// in bridged ether; Polkadot<->Kusama takes a flat amount from the caller, in the
// source chain's own token.
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
