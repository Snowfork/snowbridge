import { ApiPromise } from "@polkadot/api"
import { BlockHash } from "@polkadot/types/interfaces"
import { Codec } from "@polkadot/types/types"
import { concatMap, filter, firstValueFrom, take } from "rxjs"
import { SubscanApi, fetchEvents, fetchExtrinsics } from "./subscan"
import { forwardedTopicId } from "./utils"

export const scanSubstrateEvents = async (
    parachain: ApiPromise,
    start: number,
    scanBlocks: number,
    filter: (blockNumber: number, blockHash: BlockHash, event: Codec) => Promise<boolean>
): Promise<{
    found: boolean
    lastScannedBlock: number
    events?: Codec
}> => {
    const finalized = (
        await parachain.rpc.chain.getHeader(await parachain.rpc.chain.getFinalizedHead())
    ).number.toNumber()
    const stopScan = start + scanBlocks
    const end = finalized < stopScan ? finalized : stopScan

    for (let blockNumber = start; blockNumber <= end; ++blockNumber) {
        const blockHash = await parachain.rpc.chain.getBlockHash(blockNumber)
        const events = await (await parachain.at(blockHash)).query.system.events()
        for (const event of events as any) {
            if (await filter(blockNumber, blockHash, event)) {
                return { found: true, lastScannedBlock: blockNumber, events: events }
            }
        }
    }

    return { found: false, lastScannedBlock: end }
}

export const waitForMessageQueuePallet = async (
    parachain: ApiPromise,
    messageId: string,
    siblingParachain: number,
    eventFilter: (event: Codec) => boolean,
    options = {
        scanBlocks: 40,
    }
): Promise<{ foundEvent?: Codec; allEvents: Codec; extrinsicSuccess: boolean }> => {
    let extrinsicSuccess = false
    let returnEvent = undefined

    parachain.rpc.chain.subscribeFinalizedHeads
    let receivedEvents = await firstValueFrom(
        parachain.rx.rpc.chain.subscribeFinalizedHeads().pipe(
            take(options.scanBlocks),
            concatMap(async (header) => {
                const api1 = await parachain.at(header.hash)
                return await api1.query.system.events()
            }),
            filter((events) => {
                let foundMessageQueue = false
                let foundEvent = false
                for (const event of events as any) {
                    let eventData = event.event.toPrimitive().data
                    if (
                        parachain.events.messageQueue.Processed.is(event.event) &&
                        eventData[0].toLowerCase() === messageId.toLowerCase() &&
                        eventData[1]?.sibling === siblingParachain
                    ) {
                        foundMessageQueue = true
                        extrinsicSuccess = eventData[3]
                    }

                    if (eventFilter(event)) {
                        foundEvent = true
                        returnEvent = event
                    }
                }
                return foundMessageQueue && ((extrinsicSuccess && foundEvent) || !extrinsicSuccess)
            })
        ),
        { defaultValue: undefined }
    )

    if (receivedEvents === undefined) {
        throw Error("Timeout while waiting for event.")
    }
    return {
        foundEvent: returnEvent,
        allEvents: receivedEvents,
        extrinsicSuccess: extrinsicSuccess,
    }
}

export const subFetchBridgeTransfers = async (assetHub: SubscanApi, relaychain: SubscanApi, ethChainId: number, fromBlock: number, toBlock: number, page: number, rows = 10) => {
    return fetchExtrinsics(assetHub, "polkadotxcm", "transfer_assets", fromBlock, toBlock, page, rows, async (extrinsic, params) => {
        const dest = params.find((p: any) => p.name == 'dest')
        const parents: number | null = dest.value.V3?.parents ?? dest.value.V4?.parents ?? null
        const chainId: number | null = dest.value.V3?.interior?.X1?.GlobalConsensus?.Ethereum ?? (dest.value.V4?.interior?.X1 && dest.value.V4?.interior?.X1[0])?.GlobalConsensus?.Ethereum ?? null

        if (!(parents === 2 && chainId === ethChainId)) { return null }

        const [
            { json: { data: transfer } },
            { json: { data: relayBlock } },
        ] = await Promise.all([
            assetHub.post("scan/extrinsic", { extrinsic_index: extrinsic.extrinsic_index, only_extrinsic_event: true }),
            relaychain.post("scan/block", { block_timestamp: extrinsic.block_timestamp, only_head: true })
        ])
        const maybeEvent = transfer.event.find((ev: any) => ev.module_id === 'polkadotxcm' && ev.event_id === 'Sent')
        let messageId: string | null = null
        let bridgeHubMessageId: string | null = null

        if (transfer.success && maybeEvent) {
            const ev = JSON.parse(maybeEvent.params)
            messageId = ev.find((pa: any) => pa.name === 'message_id')?.value ?? null
            if (messageId) { bridgeHubMessageId = forwardedTopicId(messageId) }
        }

        const success = transfer.event.find((ev: any) => ev.module_id === 'system' && ev.event_id === 'ExtrinsicSuccess') !== undefined

        return {
            events: transfer.events,
            messageId, bridgeHubMessageId,
            success, block_hash: transfer.block_hash,
            account_id: transfer.account_id,
            relayChain: { block_num: relayBlock.block_num, block_hash: relayBlock.hash },
        }
    })
}

export const subFetchMessageQueueProcessed = async (api: SubscanApi, filterSibling: number, filterChannelId: string, fromBlock: number, toBlock: number, page: number, rows = 10) => {
    return fetchEvents(api, "messagequeue", ['Processed', 'ProcessingFailed', 'OverweightEnqueued'], fromBlock, toBlock, page, rows,
        async (event, params) => {
            const messageId = params.find((e: any) => e.name === 'id')?.value
            if (!messageId) { return null }

            const origin = params.find((e: any) => e.name === 'origin')?.value
            const sibling = origin?.Sibling ?? null
            const channelId = origin?.Snowbridge ?? null

            if (sibling === null && channelId !== filterChannelId) { return null; }
            if (channelId === null && sibling !== filterSibling) { return null; }
            if (channelId === null && sibling === null) { return null; }

            let success = (event.event_id === 'Processed') && (params.find((e: any) => e.name === 'success')?.value ?? false)

            return { messageId, sibling, channelId, success }
        })
}

export const subFetchOutboundMessages = async (api: SubscanApi, fromBlock: number, toBlock: number, page: number, rows = 10) => {
    return fetchEvents(api, "ethereumoutboundqueue", ['MessageAccepted', 'MessageQueued'], fromBlock, toBlock, page, rows,
        async (_, params) => {
            const messageId = params.find((e: any) => e.name === 'id')?.value
            // TODO: channelId
            const nonce = params.find((e: any) => e.name === 'nonce')?.value ?? null
            return { messageId, nonce }
        })
}
