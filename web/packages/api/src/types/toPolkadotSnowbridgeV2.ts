import type {
    Asset,
    AssetRegistry,
    ERC20Metadata,
    EthereumProviderTypes,
    MultiAddressStruct,
    Parachain,
} from "@snowbridge/base-types"
import type { OperationStatus } from "../status"
import type { FeeInfo, ValidationLog } from "./toPolkadot"

export type DeliveryFee = {
    kind: "ethereum->polkadot" | "ethereum_l2->polkadot"
    feeAsset: any
    assetHubDeliveryFeeEther: bigint
    assetHubExecutionFeeEther: bigint
    destinationDeliveryFeeEther: bigint
    destinationExecutionFeeEther?: bigint
    destinationExecutionFeeDOT?: bigint
    relayerFee: bigint
    extrinsicFeeDot: bigint
    extrinsicFeeEther: bigint
    totalFeeInWei: bigint
    bridgeFeeInL2Token?: bigint
    swapFeeInL1Token?: bigint
    volumeTip?: bigint
}

export type Transfer<T extends EthereumProviderTypes> = {
    kind: "ethereum->polkadot" | "ethereum_l2->polkadot"
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: string
        tokenAddress: string
        destinationParaId: number
        amount: bigint
        fee: DeliveryFee
        customXcm?: any[]
        l2TokenAddress?: string
        sourceChainId?: number
    }
    computed: {
        gatewayAddress: string
        beneficiaryAddressHex: string
        beneficiaryMultiAddress: MultiAddressStruct
        totalValue: bigint
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        destAssetMetadata: Asset
        destParachain: Parachain
        minimalBalance: bigint
        claimer: any
        topic: string
        l2AdapterAddress?: string
        totalInputAmount: bigint
    }
    tx: T["ContractTransaction"]
}

export type ValidatedTransfer<T extends EthereumProviderTypes> = Transfer<T> & {
    logs: ValidationLog[]
    success: boolean
    data: {
        etherBalance: bigint
        totalInputAmount?: bigint
        tokenBalance: {
            balance: bigint
            gatewayAllowance: bigint
        }
        feeInfo?: FeeInfo
        bridgeStatus: OperationStatus
        assetHubDryRunError?: string
        destinationParachainDryRunError?: string
        l2BridgeDryRunError?: string
    }
}

export type MessageReceipt = {
    nonce: bigint
    payload: any
    blockNumber: number
    blockHash: string
    txHash: string
    txIndex: number
}
