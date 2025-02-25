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
    amount: string
}

export type ToPolkadotTransferResult = {
    sourceType: "ethereum"
    id: string
    status: TransferStatus
    info: TransferInfo
    submitted: {
        blockNumber: number
        transactionHash: string
        channelId: string
        messageId: string
        nonce: number
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
        event_index: string
        block_timestamp: number
        messageId: string
        channelId: string
        nonce: number
    }
    assetHubMessageProcessed?: {
        event_index: string
        block_timestamp: string
        success: boolean
    }
    destinationReceived?: {
        paraId: number
        success: boolean
        messageId: string
        event_index: string
        block_timestamp: string
        blockNumber: number
    }
}

export type ToEthereumTransferResult = {
    sourceType: "substrate"
    id: string
    status: TransferStatus
    info: TransferInfo
    submitted: {
        sourceParachainId: number
        extrinsic_hash: string
        account_id: string
        block_num: number
        block_timestamp: number
        messageId: string
        bridgeHubMessageId: string
        success: boolean
    }
    bridgeHubXcmDelivered?: {
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
        transactionHash: string
        messageId: string
        channelId: string
        nonce: number
        success: boolean
    }
}

const buildToPolkadotTransferResult = (transfer: any): ToPolkadotTransferResult => {
    const result: ToPolkadotTransferResult = {
        sourceType: "ethereum",
        id: transfer.id,
        status: TransferStatus.Pending,
        info: {
            when: new Date(transfer.timestamp),
            sourceAddress: transfer.senderAddress,
            beneficiaryAddress: transfer.destinationAddress,
            tokenAddress: transfer.tokenAddress,
            destinationParachain: transfer.destinationParaId,
            amount: transfer.amount,
        },
        submitted: {
            blockNumber: transfer.blockNumber,
            transactionHash: transfer.txHash,
            channelId: transfer.channelId,
            messageId: transfer.messageId,
            nonce: transfer.nonce,
        },
    }
    let inboundMessageReceived = transfer.toBridgeHubInboundQueue
    if (inboundMessageReceived) {
        result.inboundMessageReceived = {
            event_index: getEventIndex(inboundMessageReceived.id),
            block_timestamp: inboundMessageReceived.timestamp,
            messageId: inboundMessageReceived.messageId,
            channelId: inboundMessageReceived.channelId,
            nonce: inboundMessageReceived.nonce,
        }
    }

    if (transfer.toAssetHubMessageQueue) {
        result.assetHubMessageProcessed = {
            event_index: getEventIndex(transfer.toAssetHubMessageQueue.id),
            block_timestamp: transfer.toAssetHubMessageQueue.timestamp,
            success: transfer.toAssetHubMessageQueue.success,
        }
        result.status = TransferStatus.Complete
        if (!transfer.toAssetHubMessageQueue.success) {
            result.status = TransferStatus.Failed
        }
    }
    
    if (transfer.toDestination) {
        result.destinationReceived = {
            event_index: getEventIndex(transfer.toDestination.id),
            block_timestamp: transfer.toDestination.timestamp,
            blockNumber: transfer.toDestination.blockNumber,
            paraId: transfer.toDestination.paraId,
            messageId: transfer.toDestination.messageId,
            success: transfer.toDestination.success,
        }
        result.status = TransferStatus.Complete
        if (!transfer.toDestination.success) {
            result.status = TransferStatus.Failed
        }
    }
    return result
}

const buildToEthereumTransferResult = (transfer: any): ToEthereumTransferResult => {
    let bridgeHubMessageId = forwardedTopicId(transfer.id)
    const result: ToEthereumTransferResult = {
        sourceType: "substrate",
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
            sourceParachainId: transfer.sourceParaId,
            extrinsic_hash: transfer.txHash,
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
        }
    }

    let ethereumMessageDispatched = transfer.toDestination
    if (ethereumMessageDispatched) {
        result.ethereumMessageDispatched = {
            blockNumber: ethereumMessageDispatched.blockNumber,
            transactionHash: ethereumMessageDispatched.txHash,
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
