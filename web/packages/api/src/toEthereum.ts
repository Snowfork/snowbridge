import { EventRecord } from "@polkadot/types/interfaces"
import { Codec, IKeyringPair, Signer } from "@polkadot/types/types"
import { BN, u8aToHex } from "@polkadot/util"
import { decodeAddress, xxhashAsHex } from "@polkadot/util-crypto"
import { Context } from "./index"
import { scanSubstrateEvents, waitForMessageQueuePallet } from "./query"
import { assetStatusInfo, bridgeStatusInfo } from "./status"
import { paraIdToChannelId } from "./utils"

export interface WalletSigner {
    address: string
    signer: Signer
}

export type WalletOrKeypair = WalletSigner | IKeyringPair

function isWallet(walletOrKeypair: WalletSigner | IKeyringPair): walletOrKeypair is WalletSigner {
    return (walletOrKeypair as WalletSigner).signer !== undefined
}

export enum SendValidationCode {
    BridgeNotOperational,
    ForeignAssetMissing,
    ERC20InvalidToken,
    ERC20NotRegistered,
    InsufficientFee,
    InsufficientAsset,
    LightClientLatencyTooHigh,
    ParachainContextMissing,
    PalletXcmMissing,
    NoHRMPChannelToAssetHub,
}

export type SendValidationError = {
    code: SendValidationCode
    message: string
}

export type SendValidationResult = {
    success?: {
        ethereumChainId: bigint
        assetHub: {
            validatedAtHash: `0x${string}`
            paraId: number
        }
        bridgeHub: {
            validatedAtHash: `0x${string}`
            paraId: number
        },
        sourceParachain?: {
            validatedAtHash: `0x${string}`
            paraId: number
        }
        relayChain: {
            validatedAtHash: `0x${string}`
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
        errors: SendValidationError[]
        lightClientLatencySeconds: number
        lightClientLatencyBlocks: number
        assetBalance: bigint
        dotBalance: bigint
    }
}

export const getSendFee = async (context: Context, options = {
    defaultFee: 2_750_872_500_000n,
}) => {
    const { polkadot: { api: { assetHub } } } = context
    // Fees stored in 0x5fbc5c7ba58845ad1f1a9a7c5bc12fad
    const feeStorageKey = xxhashAsHex(':BridgeHubEthereumBaseFee:', 128, true)
    const feeStorageItem = await assetHub.rpc.state.getStorage(feeStorageKey)
    let leFee = new BN((feeStorageItem as Codec).toHex().replace('0x', ''), "hex", "le")
    return leFee.eqn(0) ? options.defaultFee : BigInt(leFee.toString())
}

export const validateSend = async (context: Context, signer: WalletOrKeypair, sourceParachainId: number, beneficiary: string, tokenAddress: string, amount: bigint, options = {
    defaultFee: 2_750_872_500_000n,
    acceptableLatencyInSeconds: 28800 /* 8 Hours */
}): Promise<SendValidationResult> => {
    const { ethereum, ethereum: { contracts: { gateway } }, polkadot: { api: { assetHub, bridgeHub, relaychain, parachains } } } = context

    const errors: SendValidationError[] = []

    const [assetHubHead, assetHubParaId, bridgeHubHead, bridgeHubParaId, relaychainHead] = await Promise.all([
        assetHub.rpc.chain.getFinalizedHead(),
        assetHub.query.parachainInfo.parachainId(),
        bridgeHub.rpc.chain.getFinalizedHead(),
        bridgeHub.query.parachainInfo.parachainId(),
        relaychain.rpc.chain.getFinalizedHead(),
    ])
    let assetHubParaIdDecoded = assetHubParaId.toPrimitive() as number

    // Asset checks
    const assetInfo = await assetStatusInfo(context, tokenAddress)
    const foreignAssetExists = assetInfo.foreignAsset !== null && assetInfo.foreignAsset.status === 'Live'

    if (!foreignAssetExists) errors.push({ code: SendValidationCode.ForeignAssetMissing, message: "Foreign asset is not registered on Asset Hub."})
    if (!assetInfo.isTokenRegistered) errors.push({ code: SendValidationCode.ERC20NotRegistered, message: "ERC20 token is not registered with the Snowbridge Gateway."})
    if (!assetInfo.isValidERC20) errors.push({ code: SendValidationCode.ERC20InvalidToken, message: "Token address is not a valid ERC20 token."})

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
            validatedAtHash: u8aToHex(sourceParachainHead),
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
    if(!parachainKnownToContext) errors.push({ code: SendValidationCode.ParachainContextMissing, message: "The source parachain is missing from context configuration."})
    if(!parachainHasPalletXcm) errors.push({ code: SendValidationCode.PalletXcmMissing, message: "The source parachain does not have pallet-xcm."})
    if(!hrmpChannelSetup) errors.push({ code: SendValidationCode.NoHRMPChannelToAssetHub, message: "The source parachain does have an open HRMP channel to Asset Hub."})
    if (!hasAsset) errors.push({ code: SendValidationCode.InsufficientAsset, message: "Asset balance insufficient for transfer."})

    const [bridgeStatus] = await Promise.all([
        bridgeStatusInfo(context),
        ethereum.api.getNetwork(),
        gateway.isTokenRegistered(tokenAddress)
    ])
    const bridgeOperational = bridgeStatus.toEthereum.operatingMode.outbound === 'Normal'
    const lightClientLatencyIsAcceptable = bridgeStatus.toEthereum.latencySeconds < options.acceptableLatencyInSeconds

    if (!bridgeOperational) errors.push({ code: SendValidationCode.BridgeNotOperational, message: "Bridge status is not operational."})
    if (!lightClientLatencyIsAcceptable) errors.push({ code: SendValidationCode.LightClientLatencyTooHigh, message: "Light client is too far behind."})

    const [account, fee] = await Promise.all([
        assetHub.query.system.account(signer.address),
        getSendFee(context, options)
    ])
    const dotBalance = BigInt((account.toPrimitive() as any).data.free)
    const canPayFee = fee < dotBalance
    if (!canPayFee) errors.push({ code: SendValidationCode.InsufficientFee, message: "Insufficient DOT balance to pay fees."})

    if (errors.length === 0) {
        return {
            success: {
                ethereumChainId: assetInfo.ethereumChainId,
                assetHub: {
                    paraId: assetHubParaIdDecoded,
                    validatedAtHash: u8aToHex(assetHubHead),
                },
                bridgeHub: {
                    paraId: bridgeHubParaId.toPrimitive() as number,
                    validatedAtHash: u8aToHex(bridgeHubHead),
                },
                relayChain: {
                    validatedAtHash: u8aToHex(relaychainHead)
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
                errors: errors,
                lightClientLatencySeconds: bridgeStatus.toEthereum.latencySeconds,
                lightClientLatencyBlocks: bridgeStatus.toEthereum.blockLatency,
                assetBalance,
                dotBalance,
            }
        }
    }
}

export type SendResult = {
    success?: {
        plan: SendValidationResult
        messageId?: string
        sourceParachain?: {
            events: EventRecord[]
            txHash: string
            txIndex: number
            blockHash: string
            blockNumber: number
        }
        relayChain: {
            submittedAtHash: `0x${string}`
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
            messageAcceptedAtHash?: `0x${string}`
        }
        ethereum: {
            submittedAtHash: string
            beefyBlockNumber?: number
            beefyBlockHash?: string
            transferBlockNumber?: number
            transferBlockHash?: string
            messageDispatchSuccess?: boolean
        },
        polling?: {
            bridgeHubMessageQueueProcessed: bigint
            ethereumBeefyClient: bigint
            ethereumMessageDispatched: bigint
        }
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
    const { polkadot: { api: { assetHub, bridgeHub, parachains, relaychain } }, ethereum } = context
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
                    if (c.isError) {
                        reject(c.internalError || c.dispatchError)
                    }
                    if (c.isFinalized) {
                        const result = {
                            txHash: u8aToHex(c.txHash),
                            txIndex: c.txIndex || 0,
                            blockNumber: Number((c as any).blockNumber),
                            blockHash: "",
                            events: c.events
                        }
                        for (const e of c.events) {
                            if (parachainApi.events.system.ExtrinsicFailed.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: false,
                                    dispatchError: (e.event.data.toHuman(true) as any)?.dispatchError
                                })
                            }

                            if (parachainApi.events.polkadotXcm.Sent.is(e.event)) {
                                resolve({
                                    ...result,
                                    success: true,
                                    messageId: (e.event.data.toPrimitive() as any)[3]
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

        pResult.blockHash = u8aToHex(await parachainApi.rpc.chain.getBlockHash(pResult.blockNumber))
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

    const [bridgeHubHead, ethereumHead, relaychainHead] = await Promise.all([
        bridgeHub.rpc.chain.getFinalizedHead(),
        ethereum.api.getBlock('finalized'),
        relaychain.rpc.chain.getFinalizedHead(),
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
                if (c.isError) {
                    reject(c.internalError || c.dispatchError)
                }
                if (c.isFinalized) {
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
                                messageId: (e.event.data.toPrimitive() as any)[3]
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
                relayChain: {
                    submittedAtHash: u8aToHex(relaychainHead)
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

export const trackSendProgressPolling = async (context: Context, result: SendResult, options = {
    beaconUpdateTimeout: 10, scanBlocks: 600n
}): Promise<{ status: "success" | "pending", result: SendResult }> => {
    const { polkadot: { api: { relaychain, bridgeHub } }, ethereum, ethereum: { contracts: { beefyClient, gateway } } } = context
    const { success } = result

    if (result.failure || !success || !success.plan.success) {
        throw new Error('Send failed')
    }

    if (success.polling === undefined) {
        let a =
            success.polling = {
                bridgeHubMessageQueueProcessed: (await bridgeHub.rpc.chain.getHeader(success.bridgeHub.submittedAtHash)).number.toBigInt() + 1n,
                ethereumBeefyClient: BigInt((await ethereum.api.getBlock(success.ethereum.submittedAtHash))?.number ?? 0n) + 1n,
                ethereumMessageDispatched: BigInt((await ethereum.api.getBlock(success.ethereum.submittedAtHash))?.number ?? 0n) + 1n,
            }
    }

    if (success.bridgeHub.events === undefined) {
        console.log("Waiting for message to be accepted by Bridge Hub.")
        let { found, lastScannedBlock, events } = await scanSubstrateEvents(bridgeHub, success.polling.bridgeHubMessageQueueProcessed, options.scanBlocks, async (n, blockHash, ev) => {
            const event = ev as any
            let eventData = event.event.toPrimitive().data
            if (bridgeHub.events.ethereumOutboundQueue.MessageAccepted.is(event.event)
                && eventData[0].toLowerCase() === success?.messageId?.toLowerCase()) {

                success.bridgeHub.nonce = BigInt(eventData[1])
                success.bridgeHub.extrinsicSuccess = true
                success.bridgeHub.messageAcceptedAtHash = blockHash.toHex()
                return true
            }
            return false
        })
        success.polling.bridgeHubMessageQueueProcessed = lastScannedBlock + 1n
        if (!found) { return { status: "pending", result } }
        console.log(`Message accepted on Bridge Hub block ${success.bridgeHub.messageAcceptedAtHash}.`)
        success.bridgeHub.events = events
    }

    if (success.ethereum.beefyBlockNumber === undefined && success.bridgeHub.extrinsicSuccess === true) {
        // Estimate the relaychain block
        const blockGap = (success.polling?.bridgeHubMessageQueueProcessed ?? 0n) - ((await bridgeHub.rpc.chain.getHeader(success.bridgeHub.submittedAtHash)).number.toBigInt() + 1n)
        const relaychainSubmittedBlock = (await relaychain.rpc.chain.getHeader(success.relayChain.submittedAtHash)).number.toBigInt() + blockGap
        console.log("Waiting for message to be included by BEEFY light client.")
        const NewMMRRootEvent = beefyClient.getEvent("NewMMRRoot")

        const from = success.polling.ethereumBeefyClient
        let to = BigInt(await ethereum.api.getBlockNumber() ?? 0)
        if (from - to > options.scanBlocks) { to = from + options.scanBlocks }
        if (from > to) { return { status: "pending", result } }

        const events = await beefyClient.queryFilter(NewMMRRootEvent, Number(from.toString()), Number(to.toString()))
        for (const { blockHash, blockNumber, args } of events) {
            const relayChainBlock = args.blockNumber
            if (relayChainBlock >= relaychainSubmittedBlock) {
                success.ethereum.beefyBlockNumber = blockNumber
                success.ethereum.beefyBlockHash = blockHash
                console.log(`Included in BEEFY Light client block ${success.ethereum.beefyBlockHash}. Waiting for message to be delivered.`)
                break;
            } else {
                console.log(`BEEFY client ${relaychainSubmittedBlock - relayChainBlock} blocks behind.`)
            }
        }
        success.polling.ethereumBeefyClient = to + 1n
        if (success.ethereum.beefyBlockNumber === undefined) { return { status: "pending", result } }
    }

    if (success.ethereum.transferBlockNumber === undefined && success.bridgeHub.extrinsicSuccess === true) {
        console.log("Waiting for message to be dispatched to Gateway.")
        const InboundMessageDispatched = gateway.getEvent("InboundMessageDispatched")

        const from = success.polling.ethereumMessageDispatched
        let to = BigInt(await ethereum.api.getBlockNumber() ?? 0)
        if (from - to > options.scanBlocks) { to = from + options.scanBlocks }
        if (from > to) { return { status: "pending", result } }

        const events = await gateway.queryFilter(InboundMessageDispatched, Number(from.toString()), Number(to.toString()))
        for (const { blockHash, blockNumber, args } of events) {
            let { messageID, nonce, channelID, success: dispatchSuccess } = args
            if (messageID.toLowerCase() === success.messageId?.toLowerCase()
                && nonce === success.bridgeHub.nonce
                && channelID.toLowerCase() == paraIdToChannelId(success.plan.success?.assetHub.paraId ?? 1000).toLowerCase()) {

                success.ethereum.transferBlockNumber = blockNumber
                success.ethereum.transferBlockHash = blockHash
                success.ethereum.messageDispatchSuccess = dispatchSuccess
                break
            }
        }
        success.polling.ethereumMessageDispatched = to + 1n
        if (success.ethereum.transferBlockNumber === undefined) { return { status: "pending", result } }
    }

    return { status: "success", result }
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
