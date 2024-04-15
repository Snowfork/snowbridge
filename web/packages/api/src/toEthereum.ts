import { decodeAddress, xxhashAsHex } from "@polkadot/util-crypto"
import { Context } from "./index"
import { assetStatusInfo, bridgeStatusInfo } from "./status"
import { paraIdToChannelId } from "./utils"
import { BN, u8aToHex } from "@polkadot/util"
import { Codec, IKeyringPair, Signer } from "@polkadot/types/types"
import { EventRecord } from "@polkadot/types/interfaces"
import { waitForMessageQueuePallet } from "./query"

export interface WalletSigner {
    address: string
    signer: Signer
}
export type WalletOrKeypair = WalletSigner | IKeyringPair
function isWallet(walletOrKeypair: WalletSigner | IKeyringPair): walletOrKeypair is WalletSigner {
    return (walletOrKeypair as WalletSigner).signer !== undefined
}

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
        },
        sourceParachain?: {
            validatedAt: string
            paraId: number
        }
        sourceAddress: string
        sourceAddressRaw: string
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
        parachainKnownToContext: boolean
        parachainHasPalletXcm: boolean
        hrmpChannelSetup: boolean
    }
}

export const validateSend = async (context: Context, signer: WalletOrKeypair, sourceParachainId: number, beneficiary: string, tokenAddress: string, amount: bigint, options = {
    defaultFee: 2_750_872_500_000n,
    acceptableLatencyInSeconds: 28800 /* 8 Hours */
}): Promise<SendValidationResult> => {
    const { ethereum, ethereum: { contracts: { gateway } }, polkadot: { api: { assetHub, bridgeHub, relaychain, parachains } } } = context

    const [assetHubHead, assetHubParaId, bridgeHubHead, bridgeHubParaId] = await Promise.all([
        assetHub.rpc.chain.getFinalizedHead(),
        assetHub.query.parachainInfo.parachainId(),
        bridgeHub.rpc.chain.getFinalizedHead(),
        bridgeHub.query.parachainInfo.parachainId(),
    ])
    let assetHubParaIdDecoded = assetHubParaId.toPrimitive() as number

    // Asset checks
    const assetInfo = await assetStatusInfo(context, tokenAddress)
    const tokenIsRegistered = assetInfo.isTokenRegistered
    const tokenIsValidERC20 = assetInfo.isTokenRegistered
    const foreignAssetExists = assetInfo.foreignAsset !== null && assetInfo.foreignAsset.status === 'Live'

    let parachainHasPalletXcm = true
    let hrmpChannelSetup = true
    let sourceParachain = undefined
    let parachainKnownToContext = true
    let assetBalance = 0n
    let hasAsset = false
    if (parachainKnownToContext && sourceParachainId != assetHubParaIdDecoded) {
        parachainKnownToContext = sourceParachainId in parachains
        parachainHasPalletXcm = parachains[sourceParachainId].tx.polkadotXcm.transferAssets !== undefined
        let [hrmpChannel, sourceParachainHead] = await Promise.all([
            relaychain.query.hrmp.hrmpChannels({ sender: sourceParachainId, recipient: assetHubParaIdDecoded }),
            relaychain.rpc.chain.getFinalizedHead(),
        ])
        hrmpChannelSetup = hrmpChannel.toPrimitive() !== null
        sourceParachain = {
            paraId: sourceParachainId,
            validatedAt: u8aToHex(sourceParachainHead),
        }
        if (foreignAssetExists) {
            let account = (await parachains[sourceParachainId].query.foreignAssets.account(assetInfo.multiLocation, signer.address)).toPrimitive() as any
            if (account !== null) {
                assetBalance = BigInt(account.balance)
            }
            hasAsset = assetBalance >= amount
        }
    }
    else {
        if (foreignAssetExists) {
            let account = (await assetHub.query.foreignAssets.account(assetInfo.multiLocation, signer.address)).toPrimitive() as any
            if (account !== null) {
                assetBalance = BigInt(account.balance)
            }
            hasAsset = assetBalance >= amount
        }
    }

    const [bridgeStatus] = await Promise.all([
        bridgeStatusInfo(context),
        ethereum.api.getNetwork(),
        gateway.isTokenRegistered(tokenAddress)
    ])
    const bridgeOperational = bridgeStatus.toEthereum.operatingMode.outbound === 'Normal'
    const lightClientLatencyIsAcceptable = bridgeStatus.toEthereum.latencySeconds < options.acceptableLatencyInSeconds

    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feeStorageKey = xxhashAsHex(':BridgeHubEthereumBaseFee:', 128, true)
    const [feeStorageItem, account] = await Promise.all([
        assetHub.rpc.state.getStorage(feeStorageKey),
        assetHub.query.system.account(signer.address),
    ])
    let leFee = new BN((feeStorageItem as Codec).toHex().replace('0x', ''), "hex", "le")
    const fee = leFee.eqn(0) ? options.defaultFee : BigInt(leFee.toString())
    const dotBalance = BigInt((account.toPrimitive() as any).data.free)
    const canPayFee = fee < dotBalance

    const canSend = bridgeOperational && lightClientLatencyIsAcceptable && tokenIsRegistered
        && foreignAssetExists && tokenIsValidERC20 && hasAsset && canPayFee && parachainKnownToContext
        && parachainHasPalletXcm && hrmpChannelSetup

    if (canSend) {
        return {
            success: {
                ethereumChainId: assetInfo.ethereumChainId,
                assetHub: {
                    paraId: assetHubParaIdDecoded,
                    validatedAt: u8aToHex(assetHubHead),
                },
                bridgeHub: {
                    paraId: bridgeHubParaId.toPrimitive() as number,
                    validatedAt: u8aToHex(bridgeHubHead),
                },
                sourceParachain,
                feeInDOT: fee,
                sourceAddress: signer.address,
                sourceAddressRaw: u8aToHex(decodeAddress(signer.address)),
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
                parachainKnownToContext,
                parachainHasPalletXcm,
                hrmpChannelSetup
            }
        }
    }
}

export type SendResult = {
    success?: {
        plan: SendValidationResult
        messageId?: string
        sourceParachain?: {
            txHash: string
            txIndex: number
            blockHash: string
            blockNumber: number
        }
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
        sourceParachain?: {
            blockNumber: number
            txIndex: number
            txHash: string
            success: boolean
            events: EventRecord[]
            dispatchError?: any
            messageId?: string
        }
        assetHubReceiveError?: { foundEvent?: Codec, allEvents: Codec }
        assetHub?: {
            blockHash: string
            blockNumber: number
            txIndex: number
            txHash: string
            success: boolean
            events: EventRecord[]
            dispatchError?: any
            messageId?: string
        }
        plan: SendValidationResult
        dispatchError?: any
    }
}

export const send = async (context: Context, signer: WalletOrKeypair, plan: SendValidationResult, options = {
    xcmVersion: 3,
    sourceParachainFee: 10_000_000_000n,
    scanBlocks: 100,
}): Promise<SendResult> => {
    const { polkadot: { api: { assetHub, bridgeHub, parachains } }, ethereum } = context
    if (!plan.success) {
        throw Error("plan failed")
    }
    if (plan.success.sourceAddress !== signer.address) {
        throw Error("Signers do not match.")
    }

    let addressOrPair: string | IKeyringPair
    let walletSigner: Signer | undefined = undefined
    if (isWallet(signer)) {
        addressOrPair = signer.address
        walletSigner = signer.signer
    } else {
        addressOrPair = signer
    }

    const versionKey = `V${options.xcmVersion}`
    const fee_asset = 0
    const weight = "Unlimited"
    const transferAsset = {
        id: { Concrete: plan.success.multiLocation },
        fun: { Fungible: plan.success.amount }
    }

    let pResult = undefined
    if (plan.success.sourceParachain) {
        // TODO: Support orml xtokens
        let parachainApi = parachains[plan.success.sourceParachain.paraId]
        const dotLocation = parachainApi.createType('StagingXcmV3MultiLocation', {
            parents: 1,
            interior: "Here",
        })
        const pDestination: { [key: string]: any } = {}
        pDestination[versionKey] = parachainApi.createType('StagingXcmV3MultiLocation', {
            parents: 1,
            interior: { X1: { Parachain: plan.success.assetHub.paraId } },
        })
        const pAssets: { [key: string]: any } = {}
        pAssets[versionKey] = [
            {
                id: { Concrete: dotLocation },
                fun: { Fungible: options.sourceParachainFee },
            },
            transferAsset,
        ]
        const pBeneficiary: { [key: string]: any } = {}
        pBeneficiary[versionKey] = {
            parents: 0,
            interior: { X1: { AccountId32: { id: plan.success.sourceAddressRaw } } }
        }
        pResult = await new Promise<{
            blockNumber: number
            blockHash: string
            txIndex: number
            txHash: string
            success: boolean
            events: EventRecord[]
            dispatchError?: any
            messageId?: string
        }>((resolve, reject) => {
            try {
                parachainApi.tx.polkadotXcm.transferAssets(
                    pDestination,
                    pBeneficiary,
                    pAssets,
                    fee_asset,
                    weight
                ).signAndSend(addressOrPair, { signer: walletSigner }, (c) => {
                    console.log('BBB', c)
                    if (c.isError) {
                        reject(c.internalError || c.dispatchError)
                    }
                    if (c.isCompleted) {
                        const result = {
                            txHash: u8aToHex(c.txHash),
                            txIndex: c.txIndex || 0,
                            blockNumber: Number((c as any).blockNumber),
                            blockHash: u8aToHex((c as any).blockHash),
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
            }
            catch (e) {
                reject(e)
            }
        })

        if (!pResult.success) {
            return {
                failure: {
                    sourceParachain: pResult,
                    plan,
                }
            }
        }

        const { extrinsicSuccess, allEvents, foundEvent } = await waitForMessageQueuePallet(assetHub, pResult.messageId, plan.success.sourceParachain.paraId,
            _ => true,
            options,
        )
        if (!extrinsicSuccess) {
            return {
                failure: {
                    sourceParachain: pResult,
                    assetHubReceiveError: { foundEvent, allEvents },
                    plan,
                }
            }
        }
    }

    const [bridgeHubHead, ethereumHead] = await Promise.all([
        bridgeHub.rpc.chain.getFinalizedHead(),
        ethereum.api.getBlock('finalized'),
    ])

    const assets: { [key: string]: any } = {}
    assets[versionKey] = [transferAsset]
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

    let result = await new Promise<{
        blockNumber: number
        blockHash: string
        txIndex: number
        txHash: string
        success: boolean
        events: EventRecord[]
        dispatchError?: any
        messageId?: string
    }>((resolve, reject) => {
        try {
            assetHub.tx.polkadotXcm.transferAssets(
                destination,
                beneficiary,
                assets,
                fee_asset,
                weight
            ).signAndSend(addressOrPair, { signer: walletSigner }, (c) => {
                console.log('AAA', c)
                if (c.isError) {
                    reject(c.internalError || c.dispatchError)
                }
                if (c.isCompleted) {
                    const result = {
                        txHash: u8aToHex(c.txHash),
                        txIndex: c.txIndex || 0,
                        blockNumber: Number((c as any).blockNumber),
                        blockHash: u8aToHex((c as any).blockHash),
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

        }
        catch (e) {
            reject(e)
        }
    })

    const blockHash = u8aToHex(await assetHub.rpc.chain.getBlockHash(result.blockNumber))
    if (result.success) {
        return {
            success: {
                messageId: result.messageId,
                plan,
                sourceParachain: pResult,
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
        }
    } else {
        return {
            failure: {
                sourceParachain: pResult,
                assetHub: { ...result, blockHash },
                plan,
            }
        }
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
        let { extrinsicSuccess, allEvents: receivedEvents } = await waitForMessageQueuePallet(
            bridgeHub,
            undefined,
            success.plan.success.assetHub.paraId,
            eventRow => {
                let event = eventRow as any
                let eventData = (event.event.toPrimitive() as any).data
                if (bridgeHub.events.ethereumOutboundQueue.MessageAccepted.is(event.event)
                    && eventData[0].toLowerCase() === success?.messageId?.toLowerCase()) {

                    nonce = BigInt(eventData[1])
                    return true
                }
                return false
            },
            options
        )

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
            beefyClient.on(NewMMRRootEvent, listener)
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
