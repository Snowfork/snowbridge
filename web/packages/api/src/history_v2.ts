import {
    fetchToPolkadotTransfers,
    fetchToEthereumTransfers,
    fetchBridgeHubOutboundMessageAccepted,
    fetchEthereumInboundMessageDispatched,
    fetchBridgeHubInboundMessageReceived,
    fetchMessageProcessedOnPolkadot,
} from "./subsquid"
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

export const toPolkadotHistory = async (): Promise<ToPolkadotTransferResult[]> => {
    const ethOutboundMessages = await fetchToPolkadotTransfers()
    const results: ToPolkadotTransferResult[] = []
    for (const outboundMessage of ethOutboundMessages) {
        const result: ToPolkadotTransferResult = {
            id: outboundMessage.id,
            status: TransferStatus.Pending,
            info: {
                when: new Date(outboundMessage.timestamp),
                sourceAddress: outboundMessage.senderAddress,
                beneficiaryAddress: outboundMessage.destinationAddress,
                tokenAddress: outboundMessage.tokenAddress,
                destinationParachain: outboundMessage.destinationParaId,
                destinationFee: "",
                amount: outboundMessage.amount,
            },
            submitted: {
                blockHash: "",
                blockNumber: outboundMessage.blockNumber,
                logIndex: 0,
                transactionHash: outboundMessage.txHash,
                transactionIndex: 0,
                channelId: outboundMessage.channelId,
                messageId: outboundMessage.messageId,
                nonce: outboundMessage.nonce,
            },
        }
        let inboundMessageReceived = await fetchBridgeHubInboundMessageReceived(result.id)
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

        const assetHubMessageProcessed = await fetchMessageProcessedOnPolkadot(result.id)
        if (assetHubMessageProcessed) {
            result.assetHubMessageProcessed = {
                extrinsic_hash: "",
                event_index: getEventIndex(assetHubMessageProcessed.id),
                block_timestamp: assetHubMessageProcessed.timestamp,
                success: assetHubMessageProcessed.success,
                sibling: 0,
            }
            if (!result.assetHubMessageProcessed.success) {
                result.status = TransferStatus.Failed
                continue
            }

            result.status = TransferStatus.Complete
        }

        results.push(result)
    }
    return results
}

export const toEthereumHistory = async (): Promise<ToEthereumTransferResult[]> => {
    const allTransfers = await fetchToEthereumTransfers()
    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
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
                bridgeHubMessageId: "",
                success: true,
            },
        }

        let outboundQueueAccepted = await fetchBridgeHubOutboundMessageAccepted(transfer.id)
        if (outboundQueueAccepted) {
            result.bridgeHubMessageQueued = {
                block_timestamp: outboundQueueAccepted.timestamp,
                event_index: getEventIndex(outboundQueueAccepted.id),
                extrinsic_hash: "",
            }
        }

        let ethereumMessageDispatched = await fetchEthereumInboundMessageDispatched(transfer.id)
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
            if (!result.ethereumMessageDispatched.success) {
                result.status = TransferStatus.Failed
                continue
            }
            result.status = TransferStatus.Complete
        }
        results.push(result)
    }
    return results
}
