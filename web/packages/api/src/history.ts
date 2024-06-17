import { fetchBeaconSlot, paraIdToChannelId } from "./utils"
import { SubscanApi, fetchEvents, fetchExtrinsics } from "./subscan"
import { forwardedTopicId } from "./utils"
import { BeefyClient, IGateway } from "@snowbridge/contract-types"
import { AbstractProvider } from "ethers"

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

export const toPolkadotHistory = async (
    assetHubScan: SubscanApi,
    bridgeHubScan: SubscanApi,
    range: {
        assetHub: { fromBlock: number; toBlock: number }
        bridgeHub: { fromBlock: number; toBlock: number }
        ethereum: { fromBlock: number; toBlock: number }
    },
    skipLightClientUpdates: boolean,
    bridgeHubParaId: number,
    gateway: IGateway,
    provider: AbstractProvider,
    beacon_url: string
): Promise<ToPolkadotTransferResult[]> => {
    const [
        ethOutboundMessages,
        beaconClientUpdates,
        inboundMessagesReceived,
        assetHubMessageQueue,
    ] = [
        await getEthOutboundMessages(
            range.ethereum.fromBlock,
            range.ethereum.toBlock,
            skipLightClientUpdates,
            gateway,
            provider,
            beacon_url
        ),

        await (!skipLightClientUpdates
            ? getBeaconClientUpdates(
                  bridgeHubScan,
                  range.bridgeHub.fromBlock,
                  range.bridgeHub.toBlock
              )
            : Promise.resolve([])),

        await getBridgeHubInboundMessages(
            bridgeHubScan,
            range.bridgeHub.fromBlock,
            range.bridgeHub.toBlock
        ),

        await getAssetHubMessageQueueProccessed(
            assetHubScan,
            bridgeHubParaId,
            range.assetHub.fromBlock,
            range.assetHub.toBlock
        ),
    ]

    const results: ToPolkadotTransferResult[] = []
    for (const outboundMessage of ethOutboundMessages) {
        const result: ToPolkadotTransferResult = {
            id: outboundMessage.data.messageId,
            status: TransferStatus.Pending,
            info: {
                when: new Date(outboundMessage.data.timestamp * 1000),
                sourceAddress: outboundMessage.data.sourceAddress,
                beneficiaryAddress: outboundMessage.data.beneficiaryAddress,
                tokenAddress: outboundMessage.data.tokenAddress,
                destinationParachain: outboundMessage.data.destinationParachain,
                destinationFee: outboundMessage.data.destinationFee,
                amount: outboundMessage.data.amount,
            },
            submitted: {
                blockHash: outboundMessage.blockHash,
                blockNumber: outboundMessage.blockNumber,
                logIndex: outboundMessage.logIndex,
                transactionHash: outboundMessage.transactionHash,
                transactionIndex: outboundMessage.transactionIndex,
                channelId: outboundMessage.data.channelId,
                messageId: outboundMessage.data.messageId,
                nonce: outboundMessage.data.nonce,
                parentBeaconSlot: outboundMessage.data.parentBeaconSlot
                    ? Number(outboundMessage.data.parentBeaconSlot)
                    : undefined,
            },
        }
        results.push(result)

        if (result.submitted.parentBeaconSlot !== undefined) {
            const beaconClientIncluded = beaconClientUpdates.find(
                (ev) =>
                    ev.data.beaconSlot >
                    (result.submitted.parentBeaconSlot ?? Number.MAX_SAFE_INTEGER) + 1 // add one to parent to get current
            )
            if (beaconClientIncluded) {
                result.beaconClientIncluded = {
                    extrinsic_index: beaconClientIncluded.extrinsic_index,
                    extrinsic_hash: beaconClientIncluded.extrinsic_hash,
                    event_index: beaconClientIncluded.event_index,
                    block_timestamp: beaconClientIncluded.block_timestamp,
                    beaconSlot: beaconClientIncluded.data.beaconSlot,
                    beaconBlockHash: beaconClientIncluded.data.beaconBlockHash,
                }
            }
        }

        const inboundMessageReceived = inboundMessagesReceived.find(
            (ev) =>
                ev.data.messageId === result.submitted.messageId &&
                ev.data.channelId === result.submitted.channelId &&
                ev.data.nonce === result.submitted.nonce
        )
        if (inboundMessageReceived) {
            result.inboundMessageReceived = {
                extrinsic_index: inboundMessageReceived.extrinsic_index,
                extrinsic_hash: inboundMessageReceived.extrinsic_hash,
                event_index: inboundMessageReceived.event_index,
                block_timestamp: inboundMessageReceived.block_timestamp,
                messageId: inboundMessageReceived.data.messageId,
                channelId: inboundMessageReceived.data.channelId,
                nonce: inboundMessageReceived.data.nonce,
            }
        }

        const assetHubMessageProcessed = assetHubMessageQueue.find(
            (ev) =>
                ev.data.sibling === bridgeHubParaId &&
                ev.data.messageId == result.submitted.messageId
        )
        if (assetHubMessageProcessed) {
            result.assetHubMessageProcessed = {
                extrinsic_hash: assetHubMessageProcessed.extrinsic_hash,
                event_index: assetHubMessageProcessed.event_index,
                block_timestamp: assetHubMessageProcessed.block_timestamp,
                success: assetHubMessageProcessed.data.success,
                sibling: assetHubMessageProcessed.data.sibling,
            }
            if (!result.assetHubMessageProcessed.success) {
                result.status = TransferStatus.Failed
                continue
            }

            result.status = TransferStatus.Complete
        }
    }
    return results
}

export const toEthereumHistory = async (
    assetHubScan: SubscanApi,
    bridgeHubScan: SubscanApi,
    relaychainScan: SubscanApi,
    range: {
        assetHub: { fromBlock: number; toBlock: number }
        bridgeHub: { fromBlock: number; toBlock: number }
        ethereum: { fromBlock: number; toBlock: number }
    },
    skipLightClientUpdates: boolean,
    ethereumChainId: number,
    assetHubParaId: number,
    beefyClient: BeefyClient,
    gateway: IGateway
): Promise<ToEthereumTransferResult[]> => {
    const assetHubChannelId = paraIdToChannelId(assetHubParaId)

    const [
        allTransfers,
        allMessageQueues,
        allOutboundMessages,
        allBeefyClientUpdates,
        allInboundMessages,
    ] = [
        await getAssetHubTransfers(
            assetHubScan,
            relaychainScan,
            ethereumChainId,
            range.assetHub.fromBlock,
            range.assetHub.toBlock
        ),

        await getBridgeHubMessageQueueProccessed(
            bridgeHubScan,
            assetHubParaId,
            assetHubChannelId,
            range.bridgeHub.fromBlock,
            range.bridgeHub.toBlock
        ),

        await getBridgeHubOutboundMessages(
            bridgeHubScan,
            range.bridgeHub.fromBlock,
            range.bridgeHub.toBlock
        ),

        await (!skipLightClientUpdates
            ? getBeefyClientUpdates(range.ethereum.fromBlock, range.ethereum.toBlock, beefyClient)
            : Promise.resolve([])),

        await getEthInboundMessagesDispatched(
            range.ethereum.fromBlock,
            range.ethereum.toBlock,
            gateway
        ),
    ]

    const results: ToEthereumTransferResult[] = []
    for (const transfer of allTransfers) {
        const result: ToEthereumTransferResult = {
            id: transfer.data.messageId,
            status: TransferStatus.Pending,
            info: {
                when: new Date(transfer.block_timestamp * 1000),
                sourceAddress: transfer.data.account_id,
                tokenAddress: transfer.data.tokenAddress,
                beneficiaryAddress: transfer.data.beneficiaryAddress,
                amount: transfer.data.amount,
            },
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
                ev.data.sibling == assetHubParaId
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
        const { events, endOfPages: end } = await subFetchMessageQueueBySiblingOrChannel(
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

const getBeefyClientUpdates = async (
    fromBlock: number,
    toBlock: number,
    beefyClient: BeefyClient
) => {
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

const getEthInboundMessagesDispatched = async (
    fromBlock: number,
    toBlock: number,
    gateway: IGateway
) => {
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

const subFetchBridgeTransfers = async (
    assetHub: SubscanApi,
    relaychain: SubscanApi,
    ethChainId: number,
    fromBlock: number,
    toBlock: number,
    page: number,
    rows = 10
) => {
    return fetchExtrinsics(
        assetHub,
        "polkadotxcm",
        "transfer_assets",
        fromBlock,
        toBlock,
        page,
        rows,
        async (extrinsic, params) => {
            const dest = params.find((p: any) => p.name == "dest")
            const parents: number | null = dest.value.V3?.parents ?? dest.value.V4?.parents ?? null
            const chainId: number | null =
                dest.value.V3?.interior?.X1?.GlobalConsensus?.Ethereum ??
                (dest.value.V4?.interior?.X1 && dest.value.V4?.interior?.X1[0])?.GlobalConsensus
                    ?.Ethereum ??
                null

            if (!(parents === 2 && chainId === ethChainId)) {
                return null
            }

            const beneficiary = params.find((p: any) => p.name == "beneficiary")?.value
            const beneficiaryParents: number | null =
                beneficiary.V3?.parents ?? beneficiary.V4?.parents ?? null
            const beneficiaryAddress: string | null =
                beneficiary.V3?.interior?.X1?.AccountKey20?.key ??
                (beneficiary.V4?.interior?.X1 && beneficiary.V4?.interior?.X1[0])?.AccountKey20
                    ?.key ??
                null

            if (!(beneficiaryParents === 0 && beneficiaryAddress !== null)) {
                return null
            }

            const assets = params.find((p: any) => p.name == "assets")?.value
            let amount: string | null = null
            let tokenParents: number | null = null
            let tokenAddress: string | null = null
            let tokenChainId: number | null = null
            for (const asset of assets.V3 ?? assets.V4 ?? []) {
                amount = asset.fun?.Fungible ?? null
                if (amount === null) {
                    continue
                }

                tokenParents = asset.id?.parents ?? asset.id?.Concrete?.parents ?? null
                if (tokenParents === null) {
                    continue
                }

                const tokenX2 =
                    asset.id?.interior?.X2 ?? Object.values(asset.id?.Concrete?.interior?.X2 ?? {})
                if (tokenX2 === null || tokenX2.length !== 2) {
                    continue
                }

                tokenChainId = tokenX2[0].GlobalConsensus?.Ethereum ?? null
                if (tokenChainId === null) {
                    continue
                }

                tokenAddress = tokenX2[1].AccountKey20?.key ?? null
                if (tokenAddress === null) {
                    continue
                }

                // found first token
                break
            }

            if (
                !(
                    tokenParents === 2 &&
                    tokenChainId === ethChainId &&
                    tokenAddress !== null &&
                    amount !== null
                )
            ) {
                return null
            }

            const [
                {
                    json: { data: transfer },
                },
                {
                    json: { data: relayBlock },
                },
            ] = [
                await assetHub.post("api/scan/extrinsic", {
                    extrinsic_index: extrinsic.extrinsic_index,
                    only_extrinsic_event: true,
                }),
                await relaychain.post("api/scan/block", {
                    block_timestamp: extrinsic.block_timestamp,
                    only_head: true,
                }),
            ]
            const maybeEvent = transfer.event.find(
                (ev: any) => ev.module_id === "polkadotxcm" && ev.event_id === "Sent"
            )
            let messageId: string | null = null
            let bridgeHubMessageId: string | null = null

            if (transfer.success && maybeEvent) {
                const ev = JSON.parse(maybeEvent.params)
                messageId = ev.find((pa: any) => pa.name === "message_id")?.value ?? null
                if (messageId) {
                    bridgeHubMessageId = forwardedTopicId(messageId)
                }
            }

            const success =
                transfer.event.find(
                    (ev: any) => ev.module_id === "system" && ev.event_id === "ExtrinsicSuccess"
                ) !== undefined

            return {
                events: transfer.events,
                messageId,
                bridgeHubMessageId,
                success,
                block_hash: transfer.block_hash,
                account_id: transfer.account_id,
                relayChain: { block_num: relayBlock.block_num, block_hash: relayBlock.hash },
                tokenAddress,
                beneficiaryAddress,
                amount,
            }
        }
    )
}

const subFetchMessageQueueBySiblingOrChannel = async (
    api: SubscanApi,
    filterSibling: number,
    filterChannelId: string,
    fromBlock: number,
    toBlock: number,
    page: number,
    rows = 10
) => {
    return fetchEvents(
        api,
        "messagequeue",
        ["Processed", "ProcessingFailed", "OverweightEnqueued"],
        fromBlock,
        toBlock,
        page,
        rows,
        async (event, params) => {
            const messageId = params.find((e: any) => e.name === "id")?.value
            if (!messageId) {
                return null
            }

            const origin = params.find((e: any) => e.name === "origin")?.value
            const sibling = origin?.Sibling ?? null
            const channelId = origin?.Snowbridge ?? null

            if (sibling === null && channelId !== filterChannelId) {
                return null
            }
            if (channelId === null && sibling !== filterSibling) {
                return null
            }
            if (channelId === null && sibling === null) {
                return null
            }

            let success =
                event.event_id === "Processed" &&
                (params.find((e: any) => e.name === "success")?.value ?? false)

            return { messageId, sibling, channelId, success }
        }
    )
}

const subFetchMessageQueueBySibling = async (
    api: SubscanApi,
    filterSibling: number,
    fromBlock: number,
    toBlock: number,
    page: number,
    rows = 10
) => {
    return fetchEvents(
        api,
        "messagequeue",
        ["Processed", "ProcessingFailed", "OverweightEnqueued"],
        fromBlock,
        toBlock,
        page,
        rows,
        async (event, params) => {
            const messageId = params.find((e: any) => e.name === "id")?.value
            if (!messageId) {
                return null
            }

            const origin = params.find((e: any) => e.name === "origin")?.value
            const sibling = origin?.Sibling

            if (sibling !== filterSibling) {
                return null
            }

            let success =
                event.event_id === "Processed" &&
                (params.find((e: any) => e.name === "success")?.value ?? false)

            return { messageId, sibling, success }
        }
    )
}

const subFetchOutboundMessages = async (
    api: SubscanApi,
    fromBlock: number,
    toBlock: number,
    page: number,
    rows = 10
) => {
    return fetchEvents(
        api,
        "ethereumoutboundqueue",
        ["MessageAccepted", "MessageQueued"],
        fromBlock,
        toBlock,
        page,
        rows,
        async (_, params) => {
            const messageId = params.find((e: any) => e.name === "id")?.value
            // TODO: channelId
            const nonce = params.find((e: any) => e.name === "nonce")?.value ?? null
            return { messageId, nonce }
        }
    )
}

const getEthOutboundMessages = async (
    fromBlock: number,
    toBlock: number,
    skipLightClientUpdates: boolean,
    gateway: IGateway,
    provider: AbstractProvider,
    beacon_url: string
) => {
    const OutboundMessageAccepted = gateway.getEvent("OutboundMessageAccepted")
    const outboundMessages = await gateway.queryFilter(OutboundMessageAccepted, fromBlock, toBlock)
    const result = []
    for (const om of outboundMessages) {
        const [block, transaction] = await Promise.all([
            om.getBlock(),
            provider.getTransaction(om.transactionHash),
        ])
        if (transaction === null) {
            console.warn("Skipping message: Couldnt not find transaction", om.args.messageID)
            continue
        }

        try {
            const [
                tokenAddress,
                destinationParachain,
                [addressType, beneficiaryAddress],
                destinationFee,
                amount,
            ] = gateway.interface.decodeFunctionData("sendToken", transaction.data)
            let beneficiary = beneficiaryAddress as string
            switch (addressType) {
                case 0n:
                    {
                        // 4-byte index
                        const index = BigInt(beneficiary.substring(0, 6))
                        beneficiary = index.toString()
                    }
                    break
                case 2n:
                    {
                        // 20-byte address
                        beneficiary = beneficiary.substring(0, 42)
                    }
                    break
            }

            let beaconBlockRoot
            if (!skipLightClientUpdates) {
                try {
                    beaconBlockRoot = await fetchBeaconSlot(
                        beacon_url,
                        block.parentBeaconBlockRoot as any
                    )
                } catch (err) {
                    let message = "Unknown"
                    if (err instanceof Error) {
                        message = err.message
                    }
                    console.error(
                        `Error fetching beacon slot: ${message}. Skipping light client update.`,
                        err
                    )
                }
            }

            result.push({
                blockNumber: om.blockNumber,
                blockHash: om.blockHash,
                logIndex: om.index,
                transactionIndex: om.transactionIndex,
                transactionHash: om.transactionHash,
                data: {
                    sourceAddress: transaction.from,
                    timestamp: block.timestamp,
                    channelId: om.args.channelID,
                    nonce: Number(om.args.nonce),
                    messageId: om.args.messageID,
                    parentBeaconSlot: beaconBlockRoot
                        ? Number(beaconBlockRoot.data.message.slot)
                        : undefined,
                    tokenAddress: tokenAddress as string,
                    destinationParachain: Number(destinationParachain),
                    beneficiaryAddress: beneficiary,
                    destinationFee: destinationFee.toString() as string,
                    amount: amount.toString() as string,
                },
            })
        } catch (err) {
            let message = "Transaction decoding error"
            if (err instanceof Error) {
                message = `Transaction decoding error: ${err.message}`
            }
            console.warn("Skipping message:", message)
            continue
        }
    }
    return result
}

const getBeaconClientUpdates = async (
    bridgeHubScan: SubscanApi,
    fromBlock: number,
    toBlock: number
) => {
    const updates = []
    const rows = 100
    let page = 0
    let endOfPages = false
    while (!endOfPages) {
        const { events, endOfPages: end } = await subFetchBeaconHeaderImports(
            bridgeHubScan,
            fromBlock,
            toBlock,
            page,
            rows
        )
        endOfPages = end
        updates.push(...events)
        page++
    }
    updates.sort((a, b) => Number(a.data.beaconSlot - b.data.beaconSlot))
    return updates
}

const getBridgeHubInboundMessages = async (
    bridgeHubScan: SubscanApi,
    fromBlock: number,
    toBlock: number
) => {
    const updates = []
    const rows = 100
    let page = 0
    let endOfPages = false
    while (!endOfPages) {
        const { events, endOfPages: end } = await subFetchInboundMessageReceived(
            bridgeHubScan,
            fromBlock,
            toBlock,
            page,
            rows
        )
        endOfPages = end
        updates.push(...events)
        page++
    }
    return updates
}

const getAssetHubMessageQueueProccessed = async (
    bridgeHubScan: SubscanApi,
    bridgeHubParaId: number,
    fromBlock: number,
    toBlock: number
) => {
    const acc = []
    const rows = 100
    let page = 0
    let endOfPages = false
    while (!endOfPages) {
        const { events, endOfPages: end } = await subFetchMessageQueueBySibling(
            bridgeHubScan,
            bridgeHubParaId,
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

const subFetchBeaconHeaderImports = async (
    api: SubscanApi,
    fromBlock: number,
    toBlock: number,
    page: number,
    rows = 10
) => {
    return fetchEvents(
        api,
        "ethereumbeaconclient",
        ["BeaconHeaderImported"],
        fromBlock,
        toBlock,
        page,
        rows,
        async (_, params) => {
            const beaconBlockHash = params.find((e: any) => e.name === "block_hash")?.value
            const beaconSlot = params.find((e: any) => e.name === "slot")?.value
            return { beaconBlockHash, beaconSlot }
        }
    )
}

const subFetchInboundMessageReceived = async (
    api: SubscanApi,
    fromBlock: number,
    toBlock: number,
    page: number,
    rows = 10
) => {
    return fetchEvents(
        api,
        "ethereuminboundqueue",
        ["MessageReceived"],
        fromBlock,
        toBlock,
        page,
        rows,
        async (_, params) => {
            const channelId = params.find((e: any) => e.name === "channel_id")?.value
            const nonce = params.find((e: any) => e.name === "nonce")?.value
            const messageId = params.find((e: any) => e.name === "message_id")?.value
            return { channelId, nonce, messageId }
        }
    )
}
