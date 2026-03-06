import {
    Asset,
    AssetRegistry,
    ContractCall,
    ERC20Metadata,
    EthereumChain,
    Parachain,
} from "@snowbridge/base-types"
import { EventRecord } from "@polkadot/types/interfaces"
import { ContractTransaction, TransactionReceipt } from "ethers"
import { OperationStatus } from "../../status"
import { DeliveryFee, FeeInfo, ValidationLog } from "../../toEthereum_v2"

export type TransferEvm = {
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
    tx: ContractTransaction
}

export type ValidationResultEvm = {
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
    transfer: TransferEvm
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

export interface TransferInterface {
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
    ): Promise<TransferEvm>

    validateTransfer(transfer: TransferEvm): Promise<ValidationResultEvm>

    getMessageReceipt(receipt: TransactionReceipt): Promise<MessageReceiptEvm>
}
