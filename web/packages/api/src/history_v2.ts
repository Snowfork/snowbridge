import { ToPolkadotTransferResult, ToEthereumTransferResult, TransferStatus } from "./history"
import {
    fetchToPolkadotTransfers as fetchToPolkadotTransfersV2,
    fetchToEthereumTransfers as fetchToEthereumTransfersV2,
    fetchToPolkadotTransferById as fetchToPolkadotTransferByIdV2,
    fetchToEthereumTransferById as fetchToEthereumTransferByIdV2,
    fetchToEthereumTransfersBySenders,
    fetchToPolkadotTransfersBySenders,
    fetchToPolkadotPendingTransfers,
    fetchToEthereumPendingTransfers,
    fetchBridgeHubInboundMessageReceivedById,
    fetchPolkadotMessageProcessedById,
    fetchInboundMessageDispatchedOnEthereumById,
} from "./subsquid_v2"
import { getEventIndex } from "./utils"
export {
    TransferStatus,
    TransferInfo,
    ToPolkadotTransferResult,
    ToEthereumTransferResult,
    InterParachainTransfer,
} from "./history"

export const buildToPolkadotTransferResult = async (
    graphqlApiUrl: string,
    transfer: any,
): Promise<ToPolkadotTransferResult> => {
    let result: ToPolkadotTransferResult = {
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
            fee: transfer.fee,
            sourceNetwork: transfer.sourceNetwork,
            destinationNetwork: transfer.destinationNetwork,
            sourceParachain: transfer.sourceParaId,
        },
        submitted: {
            blockNumber: transfer.blockNumber,
            transactionHash: transfer.txHash,
            channelId: transfer.channelId,
            messageId: transfer.messageId,
            nonce: transfer.nonce,
        },
    }
    if (transfer.sourceNetwork == "kusama" || transfer.destinationNetwork == "kusama") {
        result.sourceType = "kusama"
    }
    let inboundMessageReceived
    if (transfer.toBridgeHubInboundQueueId) {
        inboundMessageReceived = await fetchBridgeHubInboundMessageReceivedById(
            graphqlApiUrl,
            transfer.toBridgeHubInboundQueueId,
        )
    }
    if (inboundMessageReceived) {
        result.inboundMessageReceived = {
            event_index: getEventIndex(inboundMessageReceived.id),
            block_timestamp: inboundMessageReceived.timestamp,
            messageId: inboundMessageReceived.messageId,
            channelId: inboundMessageReceived.channelId,
            nonce: inboundMessageReceived.nonce,
        }
    }

    let assetHubMessageProcessed
    if (transfer.toAssetHubMessageQueueId) {
        assetHubMessageProcessed = await fetchPolkadotMessageProcessedById(
            graphqlApiUrl,
            transfer.toAssetHubMessageQueueId,
        )
    }
    if (assetHubMessageProcessed) {
        result.assetHubMessageProcessed = {
            event_index: getEventIndex(assetHubMessageProcessed.id),
            block_timestamp: assetHubMessageProcessed.timestamp,
            success: assetHubMessageProcessed.success,
        }
        result.status = TransferStatus.Complete
        if (!assetHubMessageProcessed.success) {
            result.status = TransferStatus.Failed
        }
    }

    let destinationReceived
    if (transfer.toDestinationId) {
        destinationReceived = await fetchPolkadotMessageProcessedById(
            graphqlApiUrl,
            transfer.toDestinationId,
        )
    }
    if (destinationReceived) {
        result.destinationReceived = {
            event_index: getEventIndex(destinationReceived.id),
            block_timestamp: destinationReceived.timestamp,
            blockNumber: destinationReceived.blockNumber,
            paraId: destinationReceived.paraId,
            messageId: destinationReceived.messageId,
            success: destinationReceived.success,
        }
        result.status = TransferStatus.Complete
        if (!destinationReceived.success) {
            result.status = TransferStatus.Failed
        }
    }
    return result
}

export const buildToEthereumTransferResult = async (
    graphqlApiUrl: string,
    transfer: any,
): Promise<ToEthereumTransferResult> => {
    let bridgeHubMessageId = transfer.messageId
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
    let bridgeHubXcmDelivered
    if (transfer.toBridgeHubMessageQueueId) {
        bridgeHubXcmDelivered = await fetchPolkadotMessageProcessedById(
            graphqlApiUrl,
            transfer.toBridgeHubMessageQueueId,
        )
    }
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

    let ethereumMessageDispatched
    if (transfer.toDestinationId) {
        ethereumMessageDispatched = await fetchInboundMessageDispatchedOnEthereumById(
            graphqlApiUrl,
            transfer.toDestinationId,
        )
    }
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

export const toPolkadotHistory = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
): Promise<ToPolkadotTransferResult[]> => {
    const allTransfers = await fetchToPolkadotTransfersV2(graphqlApiUrl, graphqlQuerySize)
    const results: ToPolkadotTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = await buildToPolkadotTransferResult(graphqlApiUrl, transfer)
        results.push(result)
    }
    return results
}

export const toEthereumHistory = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
): Promise<ToEthereumTransferResult[]> => {
    const allTransfers = await fetchToEthereumTransfersV2(graphqlApiUrl, graphqlQuerySize)
    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = await buildToEthereumTransferResult(graphqlApiUrl, transfer)
        results.push(result)
    }
    return results
}

export const toPolkadotTransferById = async (
    graphqlApiUrl: string,
    id: string,
): Promise<ToPolkadotTransferResult | undefined> => {
    const transfers = await fetchToPolkadotTransferByIdV2(graphqlApiUrl, id)
    if (transfers?.length > 0) {
        let result = await buildToPolkadotTransferResult(graphqlApiUrl, transfers[0])
        return result
    }
    return
}

export const toEthereumTransferById = async (
    graphqlApiUrl: string,
    id: string,
): Promise<ToEthereumTransferResult | undefined> => {
    const transfers = await fetchToEthereumTransferByIdV2(graphqlApiUrl, id)
    if (transfers?.length > 0) {
        let result = await buildToEthereumTransferResult(graphqlApiUrl, transfers[0])
        return result
    }
    return
}

export const toPolkadotTransfersBySenders = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
    senders: string[],
): Promise<ToPolkadotTransferResult[]> => {
    const allTransfers = await fetchToPolkadotTransfersBySenders(
        graphqlApiUrl,
        graphqlQuerySize,
        senders,
    )
    const results: ToPolkadotTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = await buildToPolkadotTransferResult(graphqlApiUrl, transfer)
        results.push(result)
    }
    return results
}

export const toEthereumTransfersBySenders = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
    senders: string[],
): Promise<ToEthereumTransferResult[]> => {
    const allTransfers = await fetchToEthereumTransfersBySenders(
        graphqlApiUrl,
        graphqlQuerySize,
        senders,
    )
    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = await buildToEthereumTransferResult(graphqlApiUrl, transfer)
        results.push(result)
    }
    return results
}

export const toPolkadotPendingTransfers = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
): Promise<ToPolkadotTransferResult[]> => {
    const allTransfers = await fetchToPolkadotPendingTransfers(graphqlApiUrl, graphqlQuerySize)
    const results: ToPolkadotTransferResult[] = []
    for (const transfer of allTransfers) {
        let result = await buildToPolkadotTransferResult(graphqlApiUrl, transfer)
        results.push(result)
    }
    return results
}

export const toEthereumPendingTransfers = async (
    graphqlApiUrl: string,
    graphqlQuerySize: number = 100,
): Promise<ToEthereumTransferResult[]> => {
    const allTransfers = await fetchToEthereumPendingTransfers(graphqlApiUrl, graphqlQuerySize)
    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        if (transfer.status === 0) {
            let result = await buildToEthereumTransferResult(graphqlApiUrl, transfer)
            results.push(result)
        }
    }
    return results
}
