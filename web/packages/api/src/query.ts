import { ApiPromise } from '@polkadot/api'
import { Codec } from '@polkadot/types/types'
import { filter, firstValueFrom, take } from 'rxjs'

export const waitForMessageQueuePallet = async (
    parachain: ApiPromise,
    messageId: string | undefined,
    siblingParachain: number,
    eventFilter: (event: Codec) => boolean,
    options = {
        scanBlocks: 10,
    }
): Promise<{ foundEvent?: Codec, allEvents: Codec, extrinsicSuccess: boolean }> => {
    let extrinsicSuccess = false
    let returnEvent = undefined
    let receivedEvents = await firstValueFrom(
        parachain.rx.query.system.events().pipe(
            take(options.scanBlocks),
            filter(events => {
                let foundMessageQueue = false
                let foundEvent = false
                for (const event of (events as any)) {
                    let eventData = event.event.toPrimitive().data
                    if (parachain.events.messageQueue.Processed.is(event.event)
                        // TODO: Use SetTopic to forward the message id to the destination chain and then remove undefined check.
                        && (messageId === undefined || eventData[0].toLowerCase() === messageId.toLowerCase())
                        && eventData[1]?.sibling === siblingParachain) {

                        foundMessageQueue = true
                        extrinsicSuccess = eventData[3]
                    }

                    if (eventFilter(event)) {
                        foundEvent = true
                        returnEvent = event
                    }
                }
                return foundMessageQueue && ((extrinsicSuccess && foundEvent) || !extrinsicSuccess)
            }),
        ),
        { defaultValue: undefined }
    )

    if (receivedEvents === undefined) {
        throw Error('Timeout while waiting for event.')
    }
    return {
        foundEvent: returnEvent,
        allEvents: receivedEvents,
        extrinsicSuccess: extrinsicSuccess
    }
}
