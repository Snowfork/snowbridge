import { Context, EthereumProviderTypes } from "../.."
import {
    Asset,
    AssetRegistry,
    ContractCall,
    ERC20Metadata,
    EthereumChain,
    Parachain,
} from "@snowbridge/base-types"
import { EventRecord } from "@polkadot/types/interfaces"
import { OperationStatus } from "../../status"
import { DeliveryFee, FeeInfo, ValidationLog } from "../../toEthereum_v2"

export type TransferEvm<T extends EthereumProviderTypes = EthereumProviderTypes> = {
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

export type ValidationResultEvm<T extends EthereumProviderTypes = EthereumProviderTypes> = {
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
    transfer: TransferEvm<T>
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

export interface TransferInterface<T extends EthereumProviderTypes = EthereumProviderTypes> {
    readonly context: Context<T>

    getDeliveryFee(
        tokenAddress: string,
        options?: {
            padPercentage?: bigint
            slippagePadPercentage?: bigint
            defaultFee?: bigint
            feeTokenLocation?: any
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<DeliveryFee>

    createTransfer(
        sourceAccount: string,
        beneficiaryAccount: string,
        tokenAddress: string,
        amount: bigint,
        fee: DeliveryFee,
        options?: {
            claimerLocation?: any
            contractCall?: ContractCall
        },
    ): Promise<TransferEvm<T>>

    validateTransfer(transfer: TransferEvm<T>): Promise<ValidationResultEvm<T>>

    getMessageReceipt(receipt: T["TransactionReceipt"]): Promise<MessageReceiptEvm>
}
