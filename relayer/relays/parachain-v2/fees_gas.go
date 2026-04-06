package parachain

// Relayer-side gas units for fee / profitability estimation (not enforced on-chain).
// When gateway or beefy costs change meaningfully, update these and release.
//
// BaseMessageVerificationGas is the v2_submit verification leg (MMR/header checks before dispatch). It is used for both
// MessageMinFeeWei and minGasForV2SubmitProof (one number for fee and multicall gas floor). BaseUnlockGas, BaseMintGas pair
// with command kinds in contracts/src/Gateway.sol.
const (
	BaseBeefyFiatShamirGas     uint64 = 2_000_000
	BaseMessageVerificationGas uint64 = 100_000
	BaseUnlockGas              uint64 = 80_000
	BaseMintGas                uint64 = 60_000
)

// Multicall3 batch tx gas heuristics (nested v2_submit must satisfy Gateway v2_dispatch checks).
// gatewayDispatchOverheadBuffer must stay in sync with DISPATCH_OVERHEAD_GAS_V2 in contracts/src/Gateway.sol.
const (
	gatewayDispatchOverheadBuffer     uint64 = 24_000 // contracts/src/Gateway.sol — paired with command MaxDispatchGas in v2_dispatch
	multicall3AggregateOverheadBuffer uint64 = 120_000
)
