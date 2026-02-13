import { ChainKind, EthereumKind, ParachainKind } from "@snowbridge/base-types"
import { getEventIndex } from "./utils"

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
    amount: string
    fee?: string
}

export type ToPolkadotTransferResult = {
    sourceKind: ChainKind
    sourceId: number
    destinationKind: ParachainKind
    destinationId: number
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
    fee?: bigint
}

export type ToEthereumTransferResult = {
    sourceKind: ChainKind
    sourceId: number
    destinationKind: EthereumKind
    destinationId: number
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
    toEthereumL2?: {
        blockNumber: number
        depositId: string
        txHash: string
    }
    fee?: bigint
}

export type InterParachainTransfer = {
    sourceKind: ParachainKind
    sourceId: number
    destinationKind: ParachainKind
    destinationId: number
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
    destinationReceived?: {
        paraId: number
        success: boolean
        messageId: string
        event_index: string
        block_timestamp: string
        blockNumber: number
    }
}

export const buildToPolkadotTransferResult = (transfer: any): ToPolkadotTransferResult => {
    let result: ToPolkadotTransferResult = {
        sourceKind: transfer.sourceNetwork,
        sourceId: transfer.sourceParaId ?? transfer.l2ChainId,
        destinationKind: transfer.destinationNetwork,
        destinationId: transfer.destinationParaId,
        id: transfer.id,
        status: TransferStatus.Pending,
        info: {
            when: new Date(transfer.timestamp),
            sourceAddress: transfer.senderAddress,
            beneficiaryAddress: transfer.destinationAddress,
            tokenAddress: transfer.tokenAddress,
            amount: transfer.amount,
            fee: transfer.fee,
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

export const buildToEthereumTransferResult = (transfer: any): ToEthereumTransferResult => {
    let bridgeHubMessageId = transfer.id
    const result: ToEthereumTransferResult = {
        sourceKind: transfer.sourceNetwork,
        sourceId: transfer.sourceParaId,
        destinationKind: transfer.destinationNetwork,
        destinationId: transfer.l2ChainId,
        id: transfer.id,
        status: TransferStatus.Pending,
        info: {
            when: new Date(transfer.timestamp),
            sourceAddress: transfer.senderAddress,
            tokenAddress: transfer.tokenAddress,
            beneficiaryAddress: transfer.destinationAddress,
            amount: transfer.amount,
            fee: transfer.fee,
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
        if (!ethereumMessageDispatched.success) {
            result.status = TransferStatus.Failed
        } else if (transfer.transfer.destinationNetwork !== "ethereum_l2") {
            // if l2 leave pending
            result.status = TransferStatus.Complete
        }
    }
    let toEthereumL2Delivered = transfer.toEthereumL2
    if (toEthereumL2Delivered) {
        result.toEthereumL2 = {
            blockNumber: toEthereumL2Delivered.blockNumber,
            depositId: toEthereumL2Delivered.depositId,
            txHash: toEthereumL2Delivered.txHash,
        }
        result.status = TransferStatus.Complete
    }
    return result
}
