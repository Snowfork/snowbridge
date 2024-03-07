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
    const { ethereum, ethereum: { contracts: { gateway } }, polkadot: { api: { assetHub, bridgeHub } } } = context

    const [assetHubHead, assetHubParaId, bridgeHubHead, bridgeHubParaId] = await Promise.all([
        assetHub.rpc.chain.getFinalizedHead(),
        assetHub.query.parachainInfo.parachainId(),
        bridgeHub.rpc.chain.getFinalizedHead(),
        bridgeHub.query.parachainInfo.parachainId(),
    ])

    const [bridgeStatus] = await Promise.all([
        bridgeStatusInfo(context),
        ethereum.api.getNetwork(),
        gateway.isTokenRegistered(tokenAddress)
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
        let account = (await assetHub.query.foreignAssets.account(assetInfo.multiLocation, source.address)).toPrimitive() as any
        if (account !== null) {
            assetBalance = BigInt(account.balance)
        }
    }
    const hasAsset = assetBalance >= amount
    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feeStorageKey = xxhashAsHex(':BridgeHubEthereumBaseFee:', 128, true)
    const [feeStorageItem, account] = await Promise.all([
        assetHub.rpc.state.getStorage(feeStorageKey),
        assetHub.query.system.account(source.address),
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
    const { polkadot: { api: { assetHub, bridgeHub } }, ethereum } = context
    if (plan.success) {
        const [bridgeHubHead, ethereumHead] = await Promise.all([
            bridgeHub.rpc.chain.getFinalizedHead(),
            ethereum.api.getBlock('finalized'),
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
            assetHub.tx.polkadotXcm.transferAssets(
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
                            if (assetHub.events.system.ExtrinsicFailed.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: false,
                                    dispatchError: (e.event.data.toHuman(true) as any)?.dispatchError
                                })
                            }

                            if (assetHub.events.polkadotXcm.Sent.is(e.event)) {
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

        const blockHash = u8aToHex(await assetHub.rpc.chain.getBlockHash(result.blockNumber))
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
    const { polkadot: { api: { relaychain, bridgeHub } }, ethereum, ethereum: { contracts: { beefyClient, gateway } } } = context
    const { success } = result

    if (result.failure || !success || !success.plan.success) {
        throw new Error('Send failed')
    }

    if (success.bridgeHub.events === undefined) {
        // Wait for nonce
        let nonce: bigint | undefined = undefined
        let extrinsicSuccess = false
        const receivedEvents = await firstValueFrom(
            bridgeHub.rx.query.system.events().pipe(
                take(options.scanBlocks),
                tap((events) => console.log(`Waiting for Bridge Hub xcm message block ${events.createdAtHash?.toHex()}.`)),
                filter(events => {
                    let events_iter: any = events
                    let foundMessageQueue = false
                    let foundMessageAccepted = false
                    for (const event of events_iter) {
                        let eventData = (event.event.toPrimitive() as any).data

                        if (bridgeHub.events.messageQueue.Processed.is(event.event)
                            && eventData[1]?.sibling === success?.plan.success?.assetHub.paraId) {

                            foundMessageQueue = true
                            extrinsicSuccess = eventData[3]
                        }
                        if (bridgeHub.events.ethereumOutboundQueue.MessageAccepted.is(event.event)
                            && eventData[0].toLowerCase() === success?.messageId?.toLowerCase()) {

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
        success.bridgeHub.events = receivedEvents
        success.bridgeHub.nonce = nonce
        success.bridgeHub.extrinsicSuccess = extrinsicSuccess
    }
    if (success.bridgeHub.extrinsicSuccess) {
        yield `Message delivered to Bridge Hub block ${success.bridgeHub.events?.createdAtHash?.toHex()}. Waiting for BEEFY client.`
    } else {
        throw new Error('Message processing failed on Bridge Hub.')
    }

    if (success.ethereum.beefyBlockNumber === undefined) {
        const polkadotBlock = (await relaychain.rpc.chain.getHeader()).number.toBigInt()
        const latestBeefyBlock = await beefyClient.latestBeefyBlock()
        console.log(`BEEFY client ${polkadotBlock - latestBeefyBlock} blocks behind.`)
        const NewMMRRootEvent = beefyClient.getEvent("NewMMRRoot")
        await new Promise<void>((resolve) => {
            const listener = (mmrRoot: string, beefyBlock: bigint) => {
                if (beefyBlock >= polkadotBlock) {
                    resolve()
                    beefyClient.removeListener(NewMMRRootEvent, listener)
                } else {
                    console.log(`BEEFY client ${polkadotBlock - beefyBlock} blocks behind.`)
                }
            }
            beefyClient.on(NewMMRRootEvent, listener);
        })
        success.ethereum.beefyBlockNumber = await ethereum.api.getBlockNumber()
    }

    yield `Included in BEEFY Light client block ${success.ethereum.beefyBlockNumber}. Waiting for message to be delivered.`
    {
        const InboundMessageDispatched = gateway.getEvent("InboundMessageDispatched")
        success.ethereum.messageDispatchSuccess = await new Promise<boolean>((resolve) => {
            const listener = (channelId: string, nonce: bigint, messageId: string, dispatchSuccess: boolean) => {
                if (messageId.toLowerCase() === success.messageId?.toLowerCase()
                    && nonce === success.bridgeHub.nonce
                    && channelId.toLowerCase() == paraIdToChannelId(success.plan.success?.assetHub.paraId ?? 1000).toLowerCase()) {

                    resolve(dispatchSuccess)
                    gateway.removeListener(InboundMessageDispatched, listener)
                }
            }
            gateway.on(InboundMessageDispatched, listener)
        })
        success.ethereum.transferBlockNumber = await ethereum.api.getBlockNumber()
    }
    if (success.ethereum.messageDispatchSuccess) {
        yield `Transfer complete in Ethereum block ${success.ethereum.transferBlockNumber}.`
    } else {
        throw Error("Message was not dispatched on successfully from Gateway.")
    }
}
