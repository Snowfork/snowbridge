import { xxhashAsHex } from "@polkadot/util-crypto"
import { Context } from "./index"
import { assetStatusInfo, bridgeStatusInfo } from "./status"
import { paraIdToChannelId } from "./utils"
import { BN, u8aToHex } from "@polkadot/util"
import { Codec, IKeyringPair } from "@polkadot/types/types"
import { EventRecord } from "@polkadot/types/interfaces"
import { filter, firstValueFrom, take, tap } from "rxjs"

export type SendValidationResult = {
    success?: {
        ethereumChainId: bigint
        assetHub: {
            validatedAt: string
            paraId: number
        }
        bridgeHub: {
            validatedAt: string
            paraId: number
        }
        sourceAddress: string
        beneficiary: string
        feeInDOT: bigint
        amount: bigint
        multiLocation: object,
        tokenAddress: string
    }
    failure?: {
        bridgeOperational: boolean
        lightClientLatencyIsAcceptable: boolean
        lightClientLatencySeconds: number
        lightClientLatencyBlocks: number
        tokenIsValidERC20: boolean
        tokenIsRegistered: boolean
        foreignAssetExists: boolean
        hasAsset: boolean
        assetBalance: bigint
        canPayFee: boolean
        dotBalance: bigint
    }
}

export const validateSend = async (context: Context, source: IKeyringPair, beneficiary: string, tokenAddress: string, amount: bigint, options = {
    defaultFee: 2_750_872_500_000n,
    acceptableLatencyInSeconds: 10800 /* 3 Hours */
}): Promise<SendValidationResult> => {
    const [assetHubHead, assetHubParaId, bridgeHubHead, bridgeHubParaId] = await Promise.all([
        context.polkadot.api.assetHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.assetHub.query.parachainInfo.parachainId(),
        context.polkadot.api.bridgeHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.bridgeHub.query.parachainInfo.parachainId(),
    ])

    const [bridgeStatus] = await Promise.all([
        bridgeStatusInfo(context),
        context.ethereum.api.getNetwork(),
        context.ethereum.contracts.gateway.isTokenRegistered(tokenAddress)
    ])
    const bridgeOperational = bridgeStatus.toEthereum.operatingMode.outbound === 'Normal'
    const lightClientLatencyIsAcceptable = bridgeStatus.toEthereum.latencySeconds < options.acceptableLatencyInSeconds

    // Asset checks
    const assetInfo = await assetStatusInfo(context, tokenAddress)
    const tokenIsRegistered = assetInfo.isTokenRegistered
    const tokenIsValidERC20 = assetInfo.isTokenRegistered
    const foreignAssetExists = assetInfo.foreignAsset !== null && assetInfo.foreignAsset.status === 'Live'

    let assetBalance = 0n
    if (foreignAssetExists) {
        let account = (await context.polkadot.api.assetHub.query.foreignAssets.account(assetInfo.multiLocation, source.address)).toPrimitive() as any
        if (account !== null) {
            assetBalance = BigInt(account.balance)
        }
    }
    const hasAsset = assetBalance >= amount
    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feeStorageKey = xxhashAsHex(':BridgeHubEthereumBaseFee:', 128, true)
    const [feeStorageItem, account] = await Promise.all([
        context.polkadot.api.assetHub.rpc.state.getStorage(feeStorageKey),
        context.polkadot.api.assetHub.query.system.account(source.address),
    ])
    let leFee = new BN((feeStorageItem as Codec).toHex().replace('0x', ''), "hex", "le");
    const fee = leFee.eqn(0) ? options.defaultFee : BigInt(leFee.toString())
    const dotBalance = BigInt((account.toPrimitive() as any).data.free)
    const canPayFee = fee < dotBalance

    const canSend = bridgeOperational && lightClientLatencyIsAcceptable
        && tokenIsRegistered && foreignAssetExists && tokenIsValidERC20 && hasAsset && canPayFee

    if (canSend) {
        return {
            success: {
                ethereumChainId: assetInfo.ethereumChainId,
                assetHub: {
                    paraId: assetHubParaId.toPrimitive() as number,
                    validatedAt: u8aToHex(assetHubHead),
                },
                bridgeHub: {
                    paraId: bridgeHubParaId.toPrimitive() as number,
                    validatedAt: u8aToHex(bridgeHubHead),
                },
                feeInDOT: fee,
                sourceAddress: source.address,
                beneficiary,
                amount,
                multiLocation: assetInfo.multiLocation,
                tokenAddress
            }
        }
    } else {
        return {
            failure: {
                bridgeOperational,
                lightClientLatencyIsAcceptable,
                lightClientLatencySeconds: bridgeStatus.toEthereum.latencySeconds,
                lightClientLatencyBlocks: bridgeStatus.toEthereum.blockLatency,
                tokenIsValidERC20,
                tokenIsRegistered,
                foreignAssetExists,
                hasAsset,
                assetBalance,
                canPayFee,
                dotBalance,
            }
        }
    }
}

export type SendResult = {
    success?: {
        plan: SendValidationResult
        messageId?: string
        assetHub: {
            events: EventRecord[]
            txHash: string
            txIndex: number
            blockHash: string
            blockNumber: number
        }
        bridgeHub: {
            submittedAtHash: string
            events?: Codec
            extrinsicSuccess?: boolean
            nonce?: bigint
        }
        ethereum: {
            submittedAtHash: string
            beefyBlockNumber?: number
            transferBlockNumber?: number
            messageDispatchSuccess?: boolean
        },
    },
    failure?: {
        txHash: string
        txIndex: number
        blockHash: string
        blockNumber: number
        plan: SendValidationResult
        dispatchError?: any
    }
}

export const send = async (context: Context, signer: IKeyringPair, plan: SendValidationResult, options = {
    xcm_version: 3
}): Promise<SendResult> => {
    if (plan.success) {
        const [bridgeHubHead, ethereumHead] = await Promise.all([
            context.polkadot.api.bridgeHub.rpc.chain.getFinalizedHead(),
            context.ethereum.api.getBlock('finalized'),
        ])

        const assets: { [key: string]: any } = {}
        const versionKey = `V${options.xcm_version}`
        assets[versionKey] = [{
            id: { Concrete: plan.success.multiLocation },
            fun: { Fungible: plan.success.amount }
        }]
        const destination: { [key: string]: any } = {}
        destination[versionKey] = {
            parents: 2,
            interior: { X1: { GlobalConsensus: { Ethereum: { chain_id: plan.success.ethereumChainId } } } }
        }
        const beneficiary: { [key: string]: any } = {}
        beneficiary[versionKey] = {
            parents: 0,
            interior: { X1: { AccountKey20: { key: plan.success.beneficiary } } }
        }
        const fee_asset = 0
        const weight = "Unlimited"

        let result = await new Promise<{
            blockNumber: number
            txIndex: number
            txHash: string
            success: boolean
            events: EventRecord[]
            dispatchError?: any
            messageId?: string
        }>((resolve, reject) => {
            context.polkadot.api.assetHub.tx.polkadotXcm.transferAssets(
                destination,
                beneficiary,
                assets,
                fee_asset,
                weight
            )
                .signAndSend(signer, (c) => {
                    if (c.isError) {
                        reject(c.internalError || c.dispatchError)
                    }
                    if (c.isCompleted) {
                        const result = {
                            txHash: u8aToHex(c.txHash),
                            txIndex: c.txIndex || 0,
                            blockNumber: Number((c as any).blockNumber),
                            events: c.events
                        }
                        for (const e of c.events) {
                            if (context.polkadot.api.assetHub.events.system.ExtrinsicFailed.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: false,
                                    dispatchError: (e.event.data.toHuman(true) as any)?.dispatchError
                                })
                            }

                            if (context.polkadot.api.assetHub.events.polkadotXcm.Sent.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: true,
                                    messageId: (e.event.data.toHuman(true) as any)?.messageId
                                })
                            }
                        }
                        resolve({
                            ...result,
                            success: false,
                        })
                    }
                })
        });

        const blockHash = u8aToHex(await context.polkadot.api.assetHub.rpc.chain.getBlockHash(result.blockNumber))
        if (result.success) {
            return {
                success: {
                    messageId: result.messageId,
                    plan,
                    assetHub: {
                        txHash: result.txHash,
                        txIndex: result.txIndex,
                        blockNumber: result.blockNumber,
                        blockHash,
                        events: result.events,
                    },
                    bridgeHub: {
                        submittedAtHash: u8aToHex(bridgeHubHead)
                    },
                    ethereum: {
                        submittedAtHash: ethereumHead?.hash || `block:${ethereumHead?.number}`
                    }
                }
            };
        } else {
            return {
                failure: {
                    txHash: result.txHash,
                    txIndex: result.txIndex,
                    blockHash,
                    blockNumber: result.blockNumber,
                    dispatchError: result.dispatchError,
                    plan,
                }
            };
        }
    }
    else {
        throw Error("plan failed")
    }
}

export async function* trackSendProgress(context: Context, result: SendResult, options = {
    beaconUpdateTimeout: 10,
    scanBlocks: 200
}): AsyncGenerator<string> {
    if (result.failure || !result.success || !result.success.plan.success) {
        throw new Error('Send failed')
    }

    if (result.success.bridgeHub.events === undefined) {
        // Wait for nonce
        let nonce: bigint | undefined = undefined
        let extrinsicSuccess = false
        const receivedEvents = await firstValueFrom(
            context.polkadot.api.bridgeHub.rx.query.system.events().pipe(
                take(options.scanBlocks),
                tap((events) => console.log(`Waiting for Bridge Hub xcm message block ${events.createdAtHash?.toHex()}.`)),
                filter(events => {
                    let events_iter: any = events
                    let foundMessageQueue = false
                    let foundMessageAccepted = false
                    for (const event of events_iter) {
                        let eventData = (event.event.toPrimitive() as any).data

                        if (context.polkadot.api.bridgeHub.events.messageQueue.Processed.is(event.event)
                            && eventData[1]?.sibling === result.success?.plan.success?.assetHub.paraId) {

                            foundMessageQueue = true
                            extrinsicSuccess = eventData[3]
                        }
                        if (context.polkadot.api.bridgeHub.events.ethereumOutboundQueue.MessageAccepted.is(event.event)
                            && eventData[0].toLowerCase() === result.success?.messageId?.toLowerCase()) {

                            foundMessageAccepted = true
                            nonce = BigInt(eventData[1])
                        }
                    }
                    return foundMessageQueue && ((foundMessageAccepted && extrinsicSuccess) || !extrinsicSuccess)
                }),
            ),
            { defaultValue: undefined }
        )
        console.log(receivedEvents?.toHuman())
        if (receivedEvents === undefined) {
            throw Error('Timeout while waiting for Bridge Hub delivery.')
        }
        result.success.bridgeHub.events = receivedEvents
        result.success.bridgeHub.nonce = nonce
        result.success.bridgeHub.extrinsicSuccess = extrinsicSuccess
    }
    if (result.success.bridgeHub.extrinsicSuccess) {
        yield `Message delivered to Bridge Hub block ${result.success.bridgeHub.events?.createdAtHash?.toHex()}. Waiting for BEEFY client.`
    } else {
        throw new Error('Message processing failed on Bridge Hub.')
    }

    if (result.success.ethereum.beefyBlockNumber === undefined) {
        const polkadotBlock = (await context.polkadot.api.relaychain.rpc.chain.getHeader()).number.toBigInt()
        const latestBeefyBlock = await context.ethereum.contracts.beefyClient.latestBeefyBlock()
        console.log(`BEEFY client ${polkadotBlock - latestBeefyBlock} blocks behind.`)
        const NewMMRRootEvent = context.ethereum.contracts.beefyClient.getEvent("NewMMRRoot")
        await new Promise<void>((resolve) => {
            const listener = (mmrRoot: string, beefyBlock: bigint) => {
                if (beefyBlock >= polkadotBlock) {
                    resolve()
                    context.ethereum.contracts.beefyClient.removeListener(NewMMRRootEvent, listener)
                } else {
                    console.log(`BEEFY client ${polkadotBlock - beefyBlock} blocks behind.`)
                }
            }
            context.ethereum.contracts.beefyClient.on(NewMMRRootEvent, listener);
        })
        result.success.ethereum.beefyBlockNumber = await context.ethereum.api.getBlockNumber()
    }

    yield `Included in BEEFY Light client block ${result.success.ethereum.beefyBlockNumber}. Waiting for message to be delivered.`
    {
        const InboundMessageDispatched = context.ethereum.contracts.gateway.getEvent("InboundMessageDispatched")
        result.success.ethereum.messageDispatchSuccess = await new Promise<boolean>((resolve) => {
            const listener = (channelId: string, nonce: bigint, messageId: string, success: boolean) => {
                if (messageId.toLowerCase() === result.success?.messageId?.toLowerCase()
                    && nonce === result.success?.bridgeHub.nonce
                    && channelId.toLowerCase() == paraIdToChannelId(result.success.plan.success?.assetHub.paraId ?? 1000).toLowerCase()) {

                    resolve(success)
                    context.ethereum.contracts.gateway.removeListener(InboundMessageDispatched, listener)
                }
            }
            context.ethereum.contracts.gateway.on(InboundMessageDispatched, listener)
        })
        result.success.ethereum.transferBlockNumber = await context.ethereum.api.getBlockNumber()
    }
    if (result.success.ethereum.messageDispatchSuccess) {
        yield `Transfer complete in Ethereum block ${result.success.ethereum.transferBlockNumber}.`
    } else {
        throw Error("Message was not dispatched on successfully from Gateway.")
    }
}
