import { ApiPromise } from "@polkadot/api"
import { BlockHash } from "@polkadot/types/interfaces"
import { Codec } from "@polkadot/types/types"
import { filter, firstValueFrom, take } from "rxjs"

export const scanSubstrateEvents = async (
    parachain: ApiPromise,
    start: bigint,
    scanBlocks: bigint,
    filter: (blockNumber: bigint, blockHash: BlockHash, event: Codec) => Promise<boolean>
): Promise<{
    found: boolean
    lastScannedBlock: bigint
    events?: Codec
}> => {
    const finalized = (
        await parachain.rpc.chain.getHeader(await parachain.rpc.chain.getFinalizedHead())
    ).number.toBigInt()
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
        scanBlocks: 10,
    }
): Promise<{ foundEvent?: Codec; allEvents: Codec; extrinsicSuccess: boolean }> => {
    let extrinsicSuccess = false
    let returnEvent = undefined
    let receivedEvents = await firstValueFrom(
        parachain.rx.query.system.events().pipe(
            take(options.scanBlocks),
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
