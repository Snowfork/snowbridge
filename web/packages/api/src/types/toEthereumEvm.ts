import type {
    Asset,
    AssetRegistry,
    ERC20Metadata,
    EthereumChain,
    EthereumProviderTypes,
    Parachain,
} from "@snowbridge/base-types"
import type { EventRecord } from "@polkadot/types/interfaces"
import type { OperationStatus } from "../status"
import type { DeliveryFee, FeeInfo, ValidationLog } from "./toEthereum"

export type TransferEvm<T extends EthereumProviderTypes> = {
    kind: "ethereum->ethereum"
    input: {
        registry: AssetRegistry
        sourceAccount: string
        beneficiaryAccount: any
        tokenAddress: string
        amount: bigint
        fee: DeliveryFee
    }
    computed: {
        sourceParaId: number
        sourceAccountHex: string
        tokenErcMetadata: ERC20Metadata
        ahAssetMetadata: Asset
        sourceAssetMetadata: Asset
        sourceParachain: Parachain
        messageId: string
        ethChain?: EthereumChain
        customXcmHex?: string
        xcTokenAddress?: string
    }
    tx: T["ContractTransaction"]
}

export type ValidatedTransferEvm<T extends EthereumProviderTypes> = TransferEvm<T> & {
    logs: ValidationLog[]
    success: boolean
    data: {
        bridgeStatus: OperationStatus
        nativeBalance: bigint
        dotBalance?: bigint
        tokenBalance: bigint
        feeInfo?: FeeInfo
        sourceDryRunError: any
        assetHubDryRunError: any
    }
}

export type MessageReceiptEvm = {
    blockNumber: number
    blockHash: string
    substrateBlockHash: string
    txIndex: number
    txHash: string
    success: boolean
    events: EventRecord[]
    dispatchError?: any
    messageId?: string
}
