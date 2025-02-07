import {
    fetchToPolkadotTransfers,
    fetchToEthereumTransfers,
    fetchToPolkadotTransferById,
    fetchToEthereumTransferById,
} from "./subsquid"
import { forwardedTopicId, getEventIndex } from "./utils"

export enum TransferStatus {
    Pending,
    Complete,
    Failed,
}

export type TransferInfo = {
    when: Date
    sourceAddress: string
    beneficiaryAddress: string
    tokenAddress: string
    destinationParachain?: number
    destinationFee?: string
    amount: string
}

export type ToPolkadotTransferResult = {
    id: string
    status: TransferStatus
    info: TransferInfo
    submitted: {
        blockHash: string
        blockNumber: number
        logIndex: number
        transactionHash: string
        transactionIndex: number
        channelId: string
        messageId: string
        nonce: number
        parentBeaconSlot?: number
    }
    beaconClientIncluded?: {
        extrinsic_index: string
        extrinsic_hash: string
        event_index: string
        block_timestamp: number
        beaconSlot: number
        beaconBlockHash: string
    }
    inboundMessageReceived?: {
        extrinsic_index: string
        extrinsic_hash: string
        event_index: string
        block_timestamp: number
        messageId: string
        channelId: string
        nonce: number
    }
    assetHubMessageProcessed?: {
        extrinsic_hash: string
        event_index: string
        block_timestamp: number
        success: boolean
        sibling: number
    }
}

export type ToEthereumTransferResult = {
    id: string
    status: TransferStatus
    info: TransferInfo
    submitted: {
        extrinsic_index: string
        extrinsic_hash: string
        block_hash: string
        account_id: string
        block_num: number
        block_timestamp: number
        messageId: string
        bridgeHubMessageId: string
        success: boolean
        relayChain?: {
            block_hash: string
            block_num: number
        }
    }
    bridgeHubXcmDelivered?: {
        extrinsic_hash: string
        event_index: string
        block_timestamp: number
        siblingParachain: number
        success: boolean
    }
    bridgeHubChannelDelivered?: {
        extrinsic_hash: string
        event_index: string
        block_timestamp: number
        channelId: string
        success: boolean
    }
    bridgeHubMessageQueued?: {
        extrinsic_hash: string
        event_index: string
        block_timestamp: number
    }
    bridgeHubMessageAccepted?: {
        extrinsic_hash: string
        event_index: string
        block_timestamp: number
        nonce: number
    }
    ethereumBeefyIncluded?: {
        blockNumber: number
        blockHash: string
        transactionHash: string
        transactionIndex: number
        logIndex: number
        relayChainblockNumber: number
        mmrRoot: string
    }
    ethereumMessageDispatched?: {
        blockNumber: number
        blockHash: string
        transactionHash: string
        transactionIndex: number
        logIndex: number
        messageId: string
        channelId: string
        nonce: number
        success: boolean
    }
}

const buildToPolkadotTransferResult = (transfer: any): ToPolkadotTransferResult => {
    const result: ToPolkadotTransferResult = {
        id: transfer.id,
        status: TransferStatus.Pending,
        info: {
            when: new Date(transfer.timestamp),
            sourceAddress: transfer.senderAddress,
            beneficiaryAddress: transfer.destinationAddress,
            tokenAddress: transfer.tokenAddress,
            destinationParachain: transfer.destinationParaId,
            destinationFee: "",
            amount: transfer.amount,
        },
        submitted: {
            blockHash: "",
            blockNumber: transfer.blockNumber,
            logIndex: 0,
            transactionHash: transfer.txHash,
            transactionIndex: 0,
            channelId: transfer.channelId,
            messageId: transfer.messageId,
            nonce: transfer.nonce,
        },
    }
    let inboundMessageReceived = transfer.toBridgeHubInboundQueue
    if (inboundMessageReceived) {
        result.inboundMessageReceived = {
            extrinsic_index: "",
            extrinsic_hash: "",
            event_index: getEventIndex(inboundMessageReceived.id),
            block_timestamp: inboundMessageReceived.timestamp,
            messageId: inboundMessageReceived.messageId,
            channelId: inboundMessageReceived.channelId,
            nonce: inboundMessageReceived.nonce,
        }
    }

    const assetHubMessageProcessed = transfer.toDestination || transfer.toAssetHubMessageQueue
    if (assetHubMessageProcessed) {
        result.assetHubMessageProcessed = {
            extrinsic_hash: "",
            event_index: getEventIndex(assetHubMessageProcessed.id),
            block_timestamp: assetHubMessageProcessed.timestamp,
            success: assetHubMessageProcessed.success,
            sibling: 0,
        }
        result.status = TransferStatus.Complete
        if (!assetHubMessageProcessed.success) {
            result.status = TransferStatus.Failed
        }
    }
    return result
}

const buildToEthereumTransferResult = (transfer: any): ToEthereumTransferResult => {
    let bridgeHubMessageId = forwardedTopicId(transfer.id)
    const result: ToEthereumTransferResult = {
        id: transfer.id,
        status: TransferStatus.Pending,
        info: {
            when: new Date(transfer.timestamp),
            sourceAddress: transfer.senderAddress,
            tokenAddress: transfer.tokenAddress,
            beneficiaryAddress: transfer.destinationAddress,
            amount: transfer.amount,
        },
        submitted: {
            extrinsic_index: "",
            extrinsic_hash: transfer.txHash,
            block_hash: "",
            account_id: transfer.senderAddress,
            block_num: transfer.blockNumber,
            block_timestamp: transfer.timestamp,
            messageId: transfer.id,
            bridgeHubMessageId,
            success: true,
        },
    }
    let bridgeHubXcmDelivered = transfer.toBridgeHubMessageQueue
    if (bridgeHubXcmDelivered) {
        result.bridgeHubXcmDelivered = {
            block_timestamp: bridgeHubXcmDelivered.timestamp,
            event_index: getEventIndex(bridgeHubXcmDelivered.id),
            extrinsic_hash: "",
            siblingParachain: 1000,
            success: bridgeHubXcmDelivered.success,
        }
        if (!bridgeHubXcmDelivered.success) {
            result.status = TransferStatus.Failed
            return result
        }
    }

    let outboundQueueAccepted = transfer.toBridgeHubOutboundQueue
    if (outboundQueueAccepted) {
        result.bridgeHubMessageQueued = {
            block_timestamp: outboundQueueAccepted.timestamp,
            event_index: getEventIndex(outboundQueueAccepted.id),
            extrinsic_hash: "",
        }
    }

    let ethereumMessageDispatched = transfer.toDestination
    if (ethereumMessageDispatched) {
        result.ethereumMessageDispatched = {
            blockNumber: ethereumMessageDispatched.blockNumber,
            blockHash: "",
            transactionHash: ethereumMessageDispatched.txHash,
            transactionIndex: 0,
            logIndex: 0,
            messageId: ethereumMessageDispatched.messageId,
            channelId: ethereumMessageDispatched.channelId,
            nonce: ethereumMessageDispatched.nonce,
            success: ethereumMessageDispatched.success,
        }
        result.status = TransferStatus.Complete
        if (!ethereumMessageDispatched.success) {
            result.status = TransferStatus.Failed
        }
    }
    return result
}

export const toPolkadotHistory = async (): Promise<ToPolkadotTransferResult[]> => {
    const allTransfers = await fetchToPolkadotTransfers()
    const results: ToPolkadotTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = buildToPolkadotTransferResult(transfer)
        results.push(result)
    }
    return results
}

export const toEthereumHistory = async (): Promise<ToEthereumTransferResult[]> => {
    const allTransfers = await fetchToEthereumTransfers()
    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = buildToEthereumTransferResult(transfer)
        results.push(result)
    }
    return results
}

export const toPolkadotTransferById = async (
    id: string
): Promise<ToPolkadotTransferResult | undefined> => {
    const transfers = await fetchToPolkadotTransferById(id)
    if (transfers?.length > 0) {
        let result = buildToPolkadotTransferResult(transfers[0])
        return result
    }
    return
}

export const toEthereumTransferById = async (
    id: string
): Promise<ToEthereumTransferResult | undefined> => {
    const transfers = await fetchToEthereumTransferById(id)
    if (transfers?.length > 0) {
        let result = buildToEthereumTransferResult(transfers[0])
        return result
    }
    return
}
