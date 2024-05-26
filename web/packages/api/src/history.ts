import { Context } from "./index"
import {
    subFetchBridgeTransfers,
    subFetchMessageQueueProcessed,
    subFetchOutboundMessages,
} from "./query"
import { SubscanApi } from "./subscan"
import { paraIdToChannelId } from "./utils"

export enum TransferStatus {
    Pending,
    Complete,
    Failed,
}

export type ToEthereumTransferResult = {
    status: TransferStatus
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
        relayChain: {
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

export const toEthereumHistory = async (
    context: Context,
    assetHubScan: SubscanApi,
    bridgeHubScan: SubscanApi,
    relaychainScan: SubscanApi,
    range: {
        assetHub: { fromBlock: number; toBlock: number }
        bridgeHub: { fromBlock: number; toBlock: number }
        ethereum: { fromBlock: number; toBlock: number }
    }
): Promise<ToEthereumTransferResult[]> => {
    const [ethNetwork, assetHubParaId] = await Promise.all([
        context.ethereum.api.getNetwork(),
        context.polkadot.api.assetHub.query.parachainInfo.parachainId(),
    ])
    const assetHubParaIdDecoded = assetHubParaId.toPrimitive() as number
    const assetHubChannelId = paraIdToChannelId(assetHubParaIdDecoded)

    console.log("Fetching history To Ethereum")
    console.log(
        `eth from ${range.ethereum.fromBlock} to ${range.ethereum.toBlock} (${
            range.ethereum.toBlock - range.ethereum.fromBlock
        } blocks)`
    )
    console.log(
        `assethub from ${range.assetHub.fromBlock} to ${range.assetHub.toBlock} (${
            range.assetHub.toBlock - range.assetHub.fromBlock
        } blocks)`
    )
    console.log(
        `bridgehub from ${range.bridgeHub.fromBlock} to ${range.bridgeHub.toBlock} (${
            range.bridgeHub.toBlock - range.bridgeHub.fromBlock
        } blocks)`
    )

    const [
        allTransfers,
        allMessageQueues,
        allOutboundMessages,
        allBeefyClientUpdates,
        allInboundMessages,
    ] = await Promise.all([
        await getAssetHubTransfers(
            assetHubScan,
            relaychainScan,
            Number(ethNetwork.chainId),
            range.assetHub.fromBlock,
            range.assetHub.toBlock
        ),

        await getBridgeHubMessageQueueProccessed(
            bridgeHubScan,
            assetHubParaIdDecoded,
            assetHubChannelId,
            range.bridgeHub.fromBlock,
            range.bridgeHub.toBlock
        ),

        await getBridgeHubOutboundMessages(
            bridgeHubScan,
            range.bridgeHub.fromBlock,
            range.bridgeHub.toBlock
        ),

        await getBeefyClientUpdates(context, range.ethereum.fromBlock, range.ethereum.toBlock),

        await getInboundMessagesDispatched(
            context,
            range.ethereum.fromBlock,
            range.ethereum.toBlock
        ),
    ])

    console.log("number of transfers", allTransfers.length)
    console.log("message queues", allMessageQueues.length)
    console.log("outbound messages", allOutboundMessages.length)
    console.log("beefy updates", allBeefyClientUpdates.length)
    console.log("inbound messages", allInboundMessages.length)

    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        const result: ToEthereumTransferResult = {
            status: TransferStatus.Pending,
            submitted: {
                extrinsic_index: transfer.extrinsic_index,
                extrinsic_hash: transfer.extrinsic_hash,
                block_hash: transfer.data.block_hash,
                account_id: transfer.data.account_id,
                block_num: transfer.block_num,
                block_timestamp: transfer.block_timestamp,
                messageId: transfer.data.messageId,
                bridgeHubMessageId: transfer.data.bridgeHubMessageId,
                success: transfer.data.success,
                relayChain: {
                    block_num: transfer.data.relayChain.block_num,
                    block_hash: transfer.data.relayChain.block_hash,
                },
            },
        }
        results.push(result)
        if (!result.submitted.success) {
            result.status = TransferStatus.Failed
            continue
        }

        const bridgeHubXcmDelivered = allMessageQueues.find(
            (ev: any) =>
                ev.data.messageId === result.submitted.bridgeHubMessageId &&
                ev.data.sibling == assetHubParaIdDecoded
        )
        if (bridgeHubXcmDelivered) {
            result.bridgeHubXcmDelivered = {
                block_timestamp: bridgeHubXcmDelivered.block_timestamp,
                event_index: bridgeHubXcmDelivered.event_index,
                extrinsic_hash: bridgeHubXcmDelivered.extrinsic_hash,
                siblingParachain: bridgeHubXcmDelivered.data.sibling,
                success: bridgeHubXcmDelivered.data.success,
            }
            if (!result.bridgeHubXcmDelivered.success) {
                result.status = TransferStatus.Failed
                continue
            }
        }
        const bridgeHubChannelDelivered = allMessageQueues.find(
            (ev: any) =>
                ev.extrinsic_hash === result.bridgeHubXcmDelivered?.extrinsic_hash &&
                ev.data.channelId === assetHubChannelId &&
                ev.block_timestamp === result.bridgeHubXcmDelivered?.block_timestamp
        )
        if (bridgeHubChannelDelivered) {
            result.bridgeHubChannelDelivered = {
                block_timestamp: bridgeHubChannelDelivered.block_timestamp,
                event_index: bridgeHubChannelDelivered.event_index,
                extrinsic_hash: bridgeHubChannelDelivered.extrinsic_hash,
                channelId: bridgeHubChannelDelivered.data.channelId,
                success: bridgeHubChannelDelivered.data.success,
            }
            if (!result.bridgeHubChannelDelivered.success) {
                result.status = TransferStatus.Failed
                continue
            }
        }

        const bridgeHubMessageQueued = allOutboundMessages.find(
            (ev: any) =>
                ev.data.messageId === result.submitted.messageId &&
                ev.event_id === "MessageQueued" /* TODO: ChannelId */
        )
        if (bridgeHubMessageQueued) {
            result.bridgeHubMessageQueued = {
                block_timestamp: bridgeHubMessageQueued.block_timestamp,
                event_index: bridgeHubMessageQueued.event_index,
                extrinsic_hash: bridgeHubMessageQueued.extrinsic_hash,
            }
        }
        const bridgeHubMessageAccepted = allOutboundMessages.find(
            (ev: any) =>
                ev.data.messageId === result.submitted.messageId &&
                ev.event_id === "MessageAccepted" /* TODO: ChannelId */
        )
        if (bridgeHubMessageAccepted) {
            result.bridgeHubMessageAccepted = {
                block_timestamp: bridgeHubMessageAccepted.block_timestamp,
                event_index: bridgeHubMessageAccepted.event_index,
                extrinsic_hash: bridgeHubMessageAccepted.extrinsic_hash,
                nonce: bridgeHubMessageAccepted.data.nonce,
            }
        }

        const secondsTillAcceptedByRelayChain = 6 /* 6 secs per block */ * 10 /* blocks */
        const ethereumBeefyIncluded = allBeefyClientUpdates.find(
            (ev) =>
                ev.data.blockNumber >
                result.submitted.relayChain.block_num + secondsTillAcceptedByRelayChain
        )
        if (ethereumBeefyIncluded) {
            result.ethereumBeefyIncluded = {
                blockNumber: ethereumBeefyIncluded.blockNumber,
                blockHash: ethereumBeefyIncluded.blockHash,
                transactionHash: ethereumBeefyIncluded.transactionHash,
                transactionIndex: ethereumBeefyIncluded.transactionIndex,
                logIndex: ethereumBeefyIncluded.logIndex,
                relayChainblockNumber: ethereumBeefyIncluded.data.blockNumber,
                mmrRoot: ethereumBeefyIncluded.data.mmrRoot,
            }
        }

        const ethereumMessageDispatched = allInboundMessages.find(
            (ev) =>
                ev.data.channelId === result.bridgeHubChannelDelivered?.channelId &&
                ev.data.messageId === result.submitted.messageId &&
                ev.data.nonce === result.bridgeHubMessageAccepted?.nonce
        )

        if (ethereumMessageDispatched) {
            result.ethereumMessageDispatched = {
                blockNumber: ethereumMessageDispatched.blockNumber,
                blockHash: ethereumMessageDispatched.blockHash,
                transactionHash: ethereumMessageDispatched.transactionHash,
                transactionIndex: ethereumMessageDispatched.transactionIndex,
                logIndex: ethereumMessageDispatched.logIndex,
                messageId: ethereumMessageDispatched.data.messageId,
                channelId: ethereumMessageDispatched.data.channelId,
                nonce: ethereumMessageDispatched.data.nonce,
                success: ethereumMessageDispatched.data.success,
            }
            if (!result.ethereumMessageDispatched.success) {
                result.status = TransferStatus.Failed
                continue
            }

            result.status = TransferStatus.Complete
        }
    }
    return results
}

const getAssetHubTransfers = async (
    assetHubScan: SubscanApi,
    relaychainScan: SubscanApi,
    ethChainId: number,
    fromBlock: number,
    toBlock: number
) => {
    const acc = []
    const rows = 100
    let page = 0

    let endOfPages = false
    while (!endOfPages) {
        const { extrinsics: transfers, endOfPages: end } = await subFetchBridgeTransfers(
            assetHubScan,
            relaychainScan,
            ethChainId,
            fromBlock,
            toBlock,
            page,
            rows
        )
        endOfPages = end
        acc.push(...transfers)
        page++
    }
    return acc
}

const getBridgeHubMessageQueueProccessed = async (
    bridgeHubScan: SubscanApi,
    assetHubParaId: number,
    assetHubChannelId: string,
    fromBlock: number,
    toBlock: number
) => {
    const acc = []
    const rows = 100
    let page = 0
    let endOfPages = false
    while (!endOfPages) {
        const { events, endOfPages: end } = await subFetchMessageQueueProcessed(
            bridgeHubScan,
            assetHubParaId,
            assetHubChannelId,
            fromBlock,
            toBlock,
            page,
            rows
        )
        endOfPages = end
        acc.push(...events)
        page++
    }
    return acc
}

const getBridgeHubOutboundMessages = async (
    bridgeHubScan: SubscanApi,
    fromBlock: number,
    toBlock: number
) => {
    const acc = []
    const rows = 100
    let page = 0
    let endOfPages = false
    while (!endOfPages) {
        const { events, endOfPages: end } = await subFetchOutboundMessages(
            bridgeHubScan,
            fromBlock,
            toBlock,
            page,
            rows
        )
        endOfPages = end
        acc.push(...events)
        page++
    }
    return acc
}

const getBeefyClientUpdates = async (context: Context, fromBlock: number, toBlock: number) => {
    const { beefyClient } = context.ethereum.contracts
    const NewMMRRoot = beefyClient.getEvent("NewMMRRoot")
    const roots = await beefyClient.queryFilter(NewMMRRoot, fromBlock, toBlock)
    const updates = roots.map((r) => {
        return {
            blockNumber: r.blockNumber,
            blockHash: r.blockHash,
            logIndex: r.index,
            transactionIndex: r.transactionIndex,
            transactionHash: r.transactionHash,
            data: {
                blockNumber: Number(r.args.blockNumber),
                mmrRoot: r.args.mmrRoot,
            },
        }
    })
    updates.sort((a, b) => Number(a.data.blockNumber - b.data.blockNumber))
    return updates
}

const getInboundMessagesDispatched = async (
    context: Context,
    fromBlock: number,
    toBlock: number
) => {
    const { gateway } = context.ethereum.contracts
    const InboundMessageDispatched = gateway.getEvent("InboundMessageDispatched")
    const inboundMessages = await gateway.queryFilter(InboundMessageDispatched, fromBlock, toBlock)
    return inboundMessages.map((im) => {
        return {
            blockNumber: im.blockNumber,
            blockHash: im.blockHash,
            logIndex: im.index,
            transactionIndex: im.transactionIndex,
            transactionHash: im.transactionHash,
            data: {
                channelId: im.args.channelID,
                nonce: Number(im.args.nonce),
                messageId: im.args.messageID,
                success: im.args.success,
            },
        }
    })
}
