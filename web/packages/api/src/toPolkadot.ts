import { decodeAddress } from "@polkadot/keyring"
import { Codec } from "@polkadot/types/types"
import { u8aToHex } from "@polkadot/util"
import { IERC20__factory, IGateway__factory, WETH9__factory } from "@snowbridge/contract-types"
import { MultiAddressStruct } from "@snowbridge/contract-types/src/IGateway"
import { Contract, ContractTransaction, LogDescription, Signer, TransactionReceipt, ethers } from "ethers"
import { concatMap, filter, firstValueFrom, lastValueFrom, take, takeWhile, tap } from "rxjs"
import { assetStatusInfo } from "./assets"
import { Context } from "./index"
import { scanSubstrateEvents, waitForMessageQueuePallet } from "./query"
import { bridgeStatusInfo, channelStatusInfo } from "./status"
import {
    beneficiaryMultiAddress,
    fetchBeaconSlot,
    paraIdToChannelId,
    paraIdToSovereignAccount,
} from "./utils"
import { ApiPromise } from "@polkadot/api"

export enum SendValidationCode {
    BridgeNotOperational,
    ChannelNotOperational,
    BeneficiaryAccountMissing,
    BeneficiaryHasHitMaxConsumers,
    ForeignAssetMissing,
    ERC20InvalidToken,
    ERC20NotRegistered,
    InsufficientFee,
    InsufficientToken,
    ERC20SpendNotApproved,
    LightClientLatencyTooHigh,
    DestinationChainMissing,
    NoHRMPChannelToDestination,
}

export type SendValidationError = {
    code: SendValidationCode
    message: string
}

export type SendValidationResult = {
    success?: {
        ethereumChainId: bigint
        fee: bigint
        sourceAddress: string
        estimatedDeliverySeconds: number
        estimatedDeliveryBlocks: number
        destinationParaId: number
        beneficiaryAddress: string
        beneficiaryMultiAddress: MultiAddressStruct
        destinationFee: bigint
        token: string
        amount: bigint
        assetHub: {
            validatedAtHash: string
            paraId: number
        }
        bridgeHub: {
            validatedAtHash: string
            paraId: number
        }
        destinationParachain?: {
            validatedAtHash: string
        }
    }
    failure?: {
        errors: SendValidationError[]
        existentialDeposit: bigint
        ethereumBalance: bigint
        tokenBalance: bigint
        tokenSpendAllowance: bigint
        lightClientLatencySeconds: number
        accountConsumers: number | null
    }
}

export interface IValidateOptions {
    acceptableLatencyInSeconds: number /* 3 Hours */
    maxConsumers: number
    ignoreExistentialDeposit: boolean
}

const ValidateOptionDefaults: IValidateOptions = {
    acceptableLatencyInSeconds: 28800 /* 3 Hours */,
    maxConsumers: 16,
    ignoreExistentialDeposit: false,
}

export const approveTokenSpend = async (
    context: Context,
    signer: Signer,
    tokenAddress: string,
    amount: bigint
): Promise<ethers.ContractTransactionResponse> => {
    const token = IERC20__factory.connect(tokenAddress, signer)
    return token.approve(context.config.appContracts.gateway, amount)
}

export const depositWeth = async (
    context: Context,
    signer: Signer,
    tokenAddress: string,
    amount: bigint
): Promise<ethers.ContractTransactionResponse> => {
    const token = WETH9__factory.connect(tokenAddress, signer)
    return token.deposit({ value: amount })
}

export const getSendFee = async (
    context: Context,
    tokenAddress: string,
    destinationParaId: number,
    destinationFee: bigint
): Promise<bigint> => {
    return await context.gateway().quoteSendTokenFee(tokenAddress, destinationParaId, destinationFee)
}

export const getSubstrateAccount = async (parachain: ApiPromise, beneficiaryHex: string) => {
    const account = (await parachain.query.system.account(beneficiaryHex)).toPrimitive() as {
        data: { free: string }
        consumers: number
    }
    return { balance: account.data.free, consumers: account.consumers }
}


export type SendTokenTx = {
    input: {
        gatewayAddress: string,
        sourceAddress: string;
        beneficiaryAddress: string;
        tokenAddress: string;
        destinationParaId: number;
        amount: bigint;
        totalFeeInWei: bigint;
        destinationFeeInDOT: bigint;
    },
    computed: {
        beneficiaryAddressHex: string;
        beneficiaryMultiAddress: MultiAddressStruct;
        totalValue: bigint;
    },
    tx: ContractTransaction
}

export async function createTx(
    gatewayAddress: string,
    sourceAddress: string,
    beneficiaryAddress: string,
    tokenAddress: string,
    destinationParaId: number,
    amount: bigint,
    totalFeeInWei: bigint,
    destinationFeeInDOT: bigint,
): Promise<SendTokenTx> {
    let { address: beneficiary, hexAddress: beneficiaryAddressHex } = beneficiaryMultiAddress(beneficiaryAddress)
    const value = totalFeeInWei
    const ifce = IGateway__factory.createInterface()
    const con = new Contract(gatewayAddress, ifce);
    const tx = await con.getFunction("sendToken").populateTransaction(
        tokenAddress,
        destinationParaId,
        beneficiary,
        destinationFeeInDOT,
        amount,
        {
            value,
            from: sourceAddress
        }
    )

    return {
        input: {
            gatewayAddress,
            sourceAddress,
            beneficiaryAddress,
            tokenAddress,
            destinationParaId,
            amount,
            totalFeeInWei,
            destinationFeeInDOT,
        }, computed: {
            beneficiaryAddressHex,
            beneficiaryMultiAddress: beneficiary,
            totalValue: value,
        },
        tx,
    }
}


export const validateSend = async (
    context: Context,
    source: ethers.Addressable,
    beneficiary: string,
    tokenAddress: string,
    destinationParaId: number,
    amount: bigint,
    destinationFee: bigint,
    validateOptions: Partial<IValidateOptions> = {}
): Promise<SendValidationResult> => {
    const options = { ...ValidateOptionDefaults, ...validateOptions }
    const [assetHub, bridgeHub, ethereum, relaychain] = await Promise.all([
        context.assetHub(),
        context.bridgeHub(),
        context.ethereum(),
        context.relaychain(),
    ])

    const sourceAddress = await source.getAddress()

    const errors: SendValidationError[] = []

    // Asset checks
    const assetInfo = await assetStatusInfo(context, tokenAddress, sourceAddress)
    const tokenSpendApproved = assetInfo.tokenGatewayAllowance >= amount
    const hasToken = assetInfo.ownerBalance >= amount
    const ethereumChainId = assetInfo.ethereumChainId
    const foreignAssetExists =
        assetInfo.foreignAsset !== null && assetInfo.foreignAsset.status === "Live"

    if (!foreignAssetExists)
        errors.push({
            code: SendValidationCode.ForeignAssetMissing,
            message: "Foreign asset is not registered on Asset Hub.",
        })
    if (!assetInfo.isTokenRegistered)
        errors.push({
            code: SendValidationCode.ERC20NotRegistered,
            message: "ERC20 token is not registered with the Snowbridge Gateway.",
        })
    if (!assetInfo.isValidERC20)
        errors.push({
            code: SendValidationCode.ERC20InvalidToken,
            message: "Token address is not a valid ERC20 token.",
        })
    if (!hasToken)
        errors.push({
            code: SendValidationCode.InsufficientToken,
            message: "ERC20 token balance insufficient for transfer.",
        })
    if (!tokenSpendApproved)
        errors.push({
            code: SendValidationCode.ERC20SpendNotApproved,
            message: "ERC20 token spend insufficient for transfer.",
        })

    let fee = 0n
    let ethereumBalance = 0n
    let canPayFee = false
    if (assetInfo.isTokenRegistered) {
        ethereumBalance = await ethereum.getBalance(sourceAddress)
        fee = await getSendFee(context, tokenAddress, destinationParaId, destinationFee)
        canPayFee = fee < ethereumBalance
    }
    if (!canPayFee)
        errors.push({
            code: SendValidationCode.InsufficientFee,
            message: "Insufficient ETH balance to pay fees.",
        })

    const [assetHubHead, assetHubParaIdCodec, bridgeHubHead, bridgeHubParaIdCodec] =
        await Promise.all([
            assetHub.rpc.chain.getFinalizedHead(),
            assetHub.query.parachainInfo.parachainId(),
            bridgeHub.rpc.chain.getFinalizedHead(),
            bridgeHub.query.parachainInfo.parachainId(),
        ])

    const assetHubParaId = assetHubParaIdCodec.toPrimitive() as number
    const assetHubChannelId = paraIdToChannelId(assetHubParaId)
    const [channelStatus, bridgeStatus] = await Promise.all([
        channelStatusInfo(context, assetHubChannelId),
        bridgeStatusInfo(context),
    ])

    let { address: beneficiaryAddress, hexAddress: beneficiaryHex } =
        beneficiaryMultiAddress(beneficiary)

    let beneficiaryAccountExists = false
    let hasConsumers = false
    let destinationChainExists = true
    let hrmpChannelSetup = true
    let accountConsumers: number | null = null
    let existentialDeposit = BigInt(
        assetHub.consts.balances.existentialDeposit.toPrimitive() as number
    )
    if (destinationParaId === assetHubParaId) {
        if (destinationFee !== 0n) throw new Error("Asset Hub does not require a destination fee.")
        if (beneficiaryAddress.kind !== 1)
            throw new Error("Asset Hub only supports 32 byte addresses.")
        const { balance, consumers } = await getSubstrateAccount(assetHub, beneficiaryHex)
        beneficiaryAccountExists =
            options.ignoreExistentialDeposit || BigInt(balance) > existentialDeposit
        hasConsumers = consumers + 2 <= options.maxConsumers
        accountConsumers = consumers
    } else {
        const [destinationHead, hrmpChannel] = await Promise.all([
            relaychain.query.paras.heads(destinationParaId),
            relaychain.query.hrmp.hrmpChannels({
                sender: assetHubParaId,
                recipient: destinationParaId,
            }),
        ])
        destinationChainExists = destinationHead.toPrimitive() !== null
        hrmpChannelSetup = hrmpChannel.toPrimitive() !== null

        if (context.hasParachain(destinationParaId)) {
            const destinationParachainApi = await context.parachain(destinationParaId)
            existentialDeposit = BigInt(
                destinationParachainApi.consts.balances.existentialDeposit.toPrimitive() as number
            )
            const { balance, consumers } = await getSubstrateAccount(
                destinationParachainApi,
                beneficiaryHex
            )
            beneficiaryAccountExists =
                options.ignoreExistentialDeposit || BigInt(balance) > existentialDeposit

            hasConsumers = consumers + 2 <= options.maxConsumers
            accountConsumers = consumers
        } else {
            // We cannot check this as we do not know the destination.
            beneficiaryAccountExists = true
            hasConsumers = true
        }
    }
    if (!destinationChainExists)
        errors.push({
            code: SendValidationCode.DestinationChainMissing,
            message: "Cannot find a parachain matching the destination parachain id.",
        })
    if (!beneficiaryAccountExists)
        errors.push({
            code: SendValidationCode.BeneficiaryAccountMissing,
            message: "Beneficiary does not hold existential deposit on destination.",
        })
    if (!hasConsumers)
        errors.push({
            code: SendValidationCode.BeneficiaryHasHitMaxConsumers,
            message: "Benificiary is approaching the asset consumer limit. Transfer may fail.",
        })
    if (!hrmpChannelSetup)
        errors.push({
            code: SendValidationCode.NoHRMPChannelToDestination,
            message: "No HRMP channel set up from Asset Hub to destination.",
        })

    let destinationParachain = undefined
    if (context.hasParachain(destinationParaId)) {
        const destParaApi = await context.parachain(destinationParaId)
        destinationParachain = {
            validatedAtHash: u8aToHex(await destParaApi.rpc.chain.getFinalizedHead()),
        }
    }

    const lightClientLatencyIsAcceptable =
        bridgeStatus.toPolkadot.latencySeconds < options.acceptableLatencyInSeconds
    const bridgeOperational =
        bridgeStatus.toPolkadot.operatingMode.outbound === "Normal" &&
        bridgeStatus.toPolkadot.operatingMode.beacon === "Normal"
    const channelOperational = channelStatus.toPolkadot.operatingMode.outbound === "Normal"

    if (!bridgeOperational)
        errors.push({
            code: SendValidationCode.BridgeNotOperational,
            message: "Bridge status is not operational.",
        })
    if (!channelOperational)
        errors.push({
            code: SendValidationCode.ChannelNotOperational,
            message: "Channel to destination is not operational.",
        })
    if (!lightClientLatencyIsAcceptable)
        errors.push({
            code: SendValidationCode.LightClientLatencyTooHigh,
            message: "Light client is too far behind.",
        })

    if (errors.length === 0) {
        return {
            success: {
                fee: fee,
                sourceAddress: sourceAddress,
                token: tokenAddress,
                destinationParaId: destinationParaId,
                beneficiaryAddress: beneficiaryHex,
                beneficiaryMultiAddress: beneficiaryAddress,
                destinationFee: destinationFee,
                ethereumChainId: ethereumChainId,
                amount: amount,
                estimatedDeliverySeconds: bridgeStatus.toPolkadot.latencySeconds * 2,
                estimatedDeliveryBlocks: bridgeStatus.toPolkadot.blockLatency * 2,
                assetHub: {
                    paraId: assetHubParaId,
                    validatedAtHash: u8aToHex(assetHubHead),
                },
                bridgeHub: {
                    paraId: bridgeHubParaIdCodec.toPrimitive() as number,
                    validatedAtHash: u8aToHex(bridgeHubHead),
                },
                destinationParachain: destinationParachain,
            },
        }
    } else {
        return {
            failure: {
                errors: errors,
                lightClientLatencySeconds: bridgeStatus.toPolkadot.latencySeconds,
                ethereumBalance,
                tokenBalance: assetInfo.ownerBalance,
                tokenSpendAllowance: assetInfo.tokenGatewayAllowance,
                existentialDeposit: existentialDeposit,
                accountConsumers: accountConsumers,
            },
        }
    }
}

export type SendResult = {
    success?: {
        ethereum: {
            blockNumber: number
            blockHash: string
            transactionHash: string
            events: LogDescription[]
        }
        bridgeHub: {
            submittedAtHash: string
            beaconUpdate?: {
                createdAtHash?: `0x${string}`
                beaconBlockRoot: `0x${string}`
                blockNumber: `${number}` | undefined
                blockHash: `0x${string}` | undefined
            }
            events?: Codec
            extrinsicSuccess?: boolean
            extrinsicNumber?: number
            messageReceivedBlockHash?: `0x${string}`
        }
        assetHub: {
            submittedAtHash: string
            events?: Codec
            extrinsicSuccess?: boolean
            messageQueueProcessedAt?: `0x${string}`
        }
        destinationParachain?: {
            submittedAtHash: string
            events?: Codec
            extrinsicSuccess?: boolean
            messageQueueProcessedAt?: `0x${string}`
        }
        channelId: string
        nonce: bigint
        messageId: string
        plan: SendValidationResult
        polling?: {
            bridgeHubBeaconBlock: number
            bridgeHubMessageReceived: number
            assetHubMessageProcessed: number
            destinationMessageProcessed: number | undefined
        }
    }
    failure?: {
        receipt: TransactionReceipt
    }
}

export const send = async (
    context: Context,
    signer: Signer,
    plan: SendValidationResult,
    confirmations = 1
): Promise<SendResult> => {
    const [assetHub, bridgeHub, gateway] = await Promise.all([
        context.assetHub(),
        context.bridgeHub(),
        context.gateway(),
    ])

    if (plan.failure || !plan.success) {
        throw new Error("Plan failed")
    }

    const { success } = plan
    if (success.sourceAddress !== (await signer.getAddress())) {
        throw new Error("Invalid signer")
    }

    // Get current heads to make tracking easier.
    const [assetHubHead, bridgeHubHead] = await Promise.all([
        assetHub.rpc.chain.getFinalizedHead(),
        bridgeHub.rpc.chain.getFinalizedHead(),
    ])

    let { tx } = await createTx(
        await gateway.getAddress(),
        plan.success.sourceAddress,
        plan.success.beneficiaryAddress,
        plan.success.token,
        plan.success.destinationParaId,
        plan.success.amount,
        plan.success.fee,
        plan.success.destinationFee
    )
    const response = await signer.sendTransaction(tx)
    let receipt = await response.wait(confirmations)

    if (receipt === null) {
        throw new Error("Error waiting for transaction completion")
    }

    if (receipt?.status !== 1) {
        return {
            failure: {
                receipt: receipt,
            },
        }
    }
    const events: LogDescription[] = []
    receipt.logs.forEach((log) => {
        let event = gateway.interface.parseLog({
            topics: [...log.topics],
            data: log.data,
        })
        if (event !== null) {
            events.push(event)
        }
    })
    const messageAccepted = events.find((log) => log.name === "OutboundMessageAccepted")

    let destinationParachain = undefined
    if (context.hasParachain(success.destinationParaId)) {
        const destParaApi = await context.parachain(success.destinationParaId)
        destinationParachain = {
            submittedAtHash: u8aToHex(await destParaApi.rpc.chain.getFinalizedHead()),
        }
    }

    return {
        success: {
            ethereum: {
                blockNumber: receipt.blockNumber,
                blockHash: receipt.blockHash,
                transactionHash: receipt.hash,
                events: events,
            },
            assetHub: {
                submittedAtHash: u8aToHex(assetHubHead),
            },
            bridgeHub: {
                submittedAtHash: u8aToHex(bridgeHubHead),
            },
            destinationParachain: destinationParachain,
            channelId: messageAccepted?.args[0],
            nonce: messageAccepted?.args[1],
            messageId: messageAccepted?.args[2],
            plan: plan,
        },
    }
}

export const trackSendProgressPolling = async (
    context: Context,
    result: SendResult,
    options = {
        beaconUpdateTimeout: 10,
        scanBlocks: 600,
    }
): Promise<{ status: "success" | "pending"; result: SendResult }> => {
    const [assetHub, bridgeHub] = await Promise.all([
        context.assetHub(),
        context.bridgeHub(),
    ])
    const { success } = result

    if (result.failure || !success || !success.plan.success) {
        throw new Error("Send failed")
    }

    if (success.polling === undefined) {
        let destinationMessageProcessed: number | undefined = undefined
        if (
            success.destinationParachain !== undefined &&
            context.hasParachain(success.plan.success.destinationParaId)
        ) {
            const destinationParaApi = await context.parachain(success.plan.success.destinationParaId)
            destinationMessageProcessed =
                (
                    await destinationParaApi.rpc.chain.getHeader(
                        success.destinationParachain.submittedAtHash
                    )
                ).number.toNumber() + 1
        }
        success.polling = {
            bridgeHubBeaconBlock:
                (
                    await bridgeHub.rpc.chain.getHeader(success.bridgeHub.submittedAtHash)
                ).number.toNumber() + 1,
            bridgeHubMessageReceived:
                (
                    await bridgeHub.rpc.chain.getHeader(success.bridgeHub.submittedAtHash)
                ).number.toNumber() + 1,
            assetHubMessageProcessed:
                (
                    await assetHub.rpc.chain.getHeader(success.assetHub.submittedAtHash)
                ).number.toNumber() + 1,
            destinationMessageProcessed: destinationMessageProcessed,
        }
    }

    let beaconUpdate = success.bridgeHub.beaconUpdate
    if (success.bridgeHub.beaconUpdate === undefined) {
        const ethereumBlockNumber = success.ethereum.blockNumber
        console.log(
            `Waiting for ethereum block ${ethereumBlockNumber} to be included by light client.`
        )
        let { found, lastScannedBlock } = await scanSubstrateEvents(
            bridgeHub,
            success.polling.bridgeHubBeaconBlock,
            options.scanBlocks,
            async (n, blockHash, ev) => {
                const event = ev as any
                if (bridgeHub.events.ethereumBeaconClient.BeaconHeaderImported.is(event.event)) {
                    const [beaconBlockRoot] = (event.event.toPrimitive() as any).data
                    const slot = await fetchBeaconSlot(
                        context.config.ethereum.beacon_url,
                        beaconBlockRoot
                    )
                    const ethBlockNumber = slot.data.message.body.execution_payload?.block_number
                    const ethBlockHash = slot.data.message.body.execution_payload?.block_hash
                    if (
                        ethBlockNumber !== undefined &&
                        ethBlockHash !== undefined &&
                        ethereumBlockNumber <= Number(ethBlockNumber)
                    ) {
                        beaconUpdate = {
                            createdAtHash: blockHash.toHex(),
                            blockNumber: ethBlockNumber,
                            blockHash: ethBlockHash,
                            beaconBlockRoot,
                        }
                        return true
                    }

                    console.log(
                        `Bridge Hub block ${blockHash.toHex()}: Beacon client ${ethereumBlockNumber - Number(ethBlockNumber)
                        } blocks behind.`
                    )
                }
                return false
            }
        )
        success.polling.bridgeHubBeaconBlock = lastScannedBlock + 1
        if (!found) {
            return { status: "pending", result }
        }
        success.bridgeHub.beaconUpdate = beaconUpdate
        console.log(`Included by light client in Bridge Hub block ${beaconUpdate?.createdAtHash}.`)
    }

    if (success.bridgeHub.events === undefined) {
        console.log(`Waiting for messageId ${success.messageId} to be recieved on Bridge Hub.`)
        let messageQueueFound = false
        let { found, lastScannedBlock, events } = await scanSubstrateEvents(
            bridgeHub,
            success.polling.bridgeHubMessageReceived,
            options.scanBlocks,
            async (n, blockHash, ev) => {
                const event = ev as any
                const data = event.event.toPrimitive().data

                if (
                    bridgeHub.events.ethereumInboundQueue.MessageReceived.is(event.event) &&
                    data[1].toString() === success?.nonce.toString() &&
                    data[2].toLowerCase() === success?.messageId.toLowerCase() &&
                    data[0].toLowerCase() === success?.channelId.toLowerCase()
                ) {
                    messageQueueFound = true
                    success.bridgeHub.messageReceivedBlockHash = blockHash.toHex()
                    success.bridgeHub.extrinsicNumber = event.phase.toPrimitive().applyExtrinsic
                }

                if (
                    messageQueueFound &&
                    bridgeHub.events.system.ExtrinsicSuccess.is(event.event) &&
                    event.phase.toPrimitive().applyExtrinsic == success.bridgeHub.extrinsicNumber &&
                    success.bridgeHub.messageReceivedBlockHash === blockHash.toHex()
                ) {
                    success.bridgeHub.extrinsicSuccess = true
                    return true
                }
                if (
                    messageQueueFound &&
                    bridgeHub.events.system.ExtrinsicFailed.is(event.event) &&
                    event.phase.toPrimitive().applyExtrinsic == success.bridgeHub.extrinsicNumber &&
                    success.bridgeHub.messageReceivedBlockHash === blockHash.toHex()
                ) {
                    success.bridgeHub.extrinsicSuccess = false
                    return true
                }
                return false
            }
        )
        success.polling.bridgeHubMessageReceived = lastScannedBlock + 1
        if (!found) {
            return { status: "pending", result }
        }
        console.log(
            `Message received on Bridge Hub block ${success.bridgeHub.messageReceivedBlockHash}.`
        )
        success.bridgeHub.events = events
    }

    if (success.assetHub.events === undefined && success.bridgeHub.extrinsicSuccess === true) {
        const issuedTo =
            success.plan.success.assetHub.paraId !== success.plan.success.destinationParaId
                ? paraIdToSovereignAccount("sibl", success.plan.success.destinationParaId)
                : success.plan?.success.beneficiaryAddress
        console.log(`Waiting for messageId ${success.messageId} to be recieved on Asset Hub.`)
        let transferBlockHash = ""
        let { found, lastScannedBlock, events } = await scanSubstrateEvents(
            assetHub,
            success.polling.assetHubMessageProcessed,
            options.scanBlocks,
            async (n, blockHash, ev) => {
                const event = ev as any
                let eventData = event.event.toPrimitive().data

                if (
                    assetHub.events.foreignAssets.Issued.is(event.event) &&
                    eventData[2].toString() === success?.plan.success?.amount.toString() &&
                    u8aToHex(decodeAddress(eventData[1])).toLowerCase() ===
                    issuedTo.toLowerCase() &&
                    eventData[0]?.parents === 2 &&
                    eventData[0]?.interior?.x2[0]?.globalConsensus?.ethereum?.chainId.toString() ===
                    success?.plan.success?.ethereumChainId.toString() &&
                    eventData[0]?.interior?.x2[1]?.accountKey20?.key.toLowerCase() ===
                    success?.plan.success?.token.toLowerCase()
                ) {
                    transferBlockHash = blockHash.toHex()
                }

                if (
                    assetHub.events.messageQueue.Processed.is(event.event) &&
                    eventData[0].toLowerCase() === success.messageId.toLowerCase() &&
                    eventData[1]?.sibling === success.plan.success?.bridgeHub.paraId
                ) {
                    success.assetHub.extrinsicSuccess = eventData[3]
                    success.assetHub.messageQueueProcessedAt = blockHash.toHex()
                    return transferBlockHash === success.assetHub.messageQueueProcessedAt
                }
                return false
            }
        )
        success.polling.assetHubMessageProcessed = lastScannedBlock + 1
        if (!found) {
            return { status: "pending", result }
        }
        console.log(
            `Message received on Asset Hub block ${success.assetHub.messageQueueProcessedAt}.`
        )
        success.assetHub.events = events
    }

    if (
        success.destinationParachain !== undefined &&
        success.plan.success.assetHub.paraId !== success.plan.success.destinationParaId &&
        context.hasParachain(success.plan.success.destinationParaId) &&
        success.polling.destinationMessageProcessed !== undefined &&
        success.destinationParachain.events === undefined &&
        success.assetHub.extrinsicSuccess === true
    ) {
        const destParaApi = await context.parachain(success.plan.success.destinationParaId)
        let extrinsicSuccess = false
        let messageQueueProcessedAt
        console.log(
            `Waiting for messageId ${success.messageId} to be recieved on Parachain ${success.plan.success.destinationParaId}.`
        )
        let { found, lastScannedBlock, events } = await scanSubstrateEvents(
            destParaApi,
            success.polling.destinationMessageProcessed,
            options.scanBlocks,
            async (n, blockHash, ev) => {
                const event = ev as any
                let eventData = event.event.toPrimitive().data
                if (
                    destParaApi.events.messageQueue.Processed.is(event.event) &&
                    eventData[0].toLowerCase() === success.messageId.toLowerCase() &&
                    eventData[1]?.sibling === success.plan.success?.assetHub.paraId
                ) {
                    extrinsicSuccess = eventData[3]
                    messageQueueProcessedAt = blockHash.toHex()
                    return true
                }
                return false
            }
        )
        success.polling.destinationMessageProcessed = lastScannedBlock + 1
        if (!found) {
            return { status: "pending", result }
        }
        success.destinationParachain.extrinsicSuccess = extrinsicSuccess
        success.destinationParachain.messageQueueProcessedAt = messageQueueProcessedAt
        console.log(
            `Message received on Destination Parachain block ${success.assetHub.messageQueueProcessedAt}.`
        )
        success.destinationParachain.events = events
    }

    return { status: "success", result }
}

export async function* trackSendProgress(
    context: Context,
    result: SendResult,
    options = {
        beaconUpdateTimeout: 10,
        scanBlocks: 200,
    }
): AsyncGenerator<string> {
    const [assetHub, bridgeHub] = await Promise.all([
        context.assetHub(),
        context.bridgeHub(),
    ])
    const { success } = result

    if (result.failure || !success || !success.plan.success) {
        throw new Error("Send failed")
    }

    if (success.bridgeHub.beaconUpdate === undefined) {
        yield "Waiting for inclusion by light client."
        // Wait for light client
        const ethereumBlockNumber = success.ethereum.blockNumber
        const lastBeaconUpdate = await lastValueFrom(
            bridgeHub.rx.query.ethereumBeaconClient.latestFinalizedBlockRoot().pipe(
                concatMap(async (finalizedBlockRoot) => {
                    const beaconBlockRoot = finalizedBlockRoot.toHex()
                    const slot = await fetchBeaconSlot(
                        context.config.ethereum.beacon_url,
                        beaconBlockRoot
                    )
                    const blockNumber = slot.data.message.body.execution_payload?.block_number
                    const blockHash = slot.data.message.body.execution_payload?.block_hash

                    return {
                        createdAtHash: finalizedBlockRoot.createdAtHash?.toHex(),
                        blockNumber,
                        blockHash,
                        beaconBlockRoot,
                    }
                }),
                filter(({ blockHash }) => blockHash !== undefined),
                take(options.beaconUpdateTimeout),
                takeWhile(({ blockNumber }) => ethereumBlockNumber > Number(blockNumber)),
                tap(({ createdAtHash, blockNumber }) =>
                    console.log(
                        `Bridge Hub block ${createdAtHash}: Beacon client ${ethereumBlockNumber - Number(blockNumber)
                        } blocks behind.`
                    )
                )
            ),
            { defaultValue: undefined }
        )

        if (lastBeaconUpdate === undefined) {
            throw new Error("Timeout waiting for light client to include block.")
        }
        success.bridgeHub.beaconUpdate = lastBeaconUpdate
    }
    yield `Included by light client in Bridge Hub block ${success.bridgeHub.beaconUpdate?.createdAtHash}.`

    if (success.bridgeHub.events === undefined) {
        yield "Waiting for message delivery to Bridge Hub."

        // Wait for nonce
        let extrinsicNumber: number | undefined = undefined
        let extrinsicSuccess = false
        const receivedEvents = await firstValueFrom(
            bridgeHub.rx.query.system.events().pipe(
                take(options.scanBlocks),
                tap((events) =>
                    console.log(
                        `Waiting for Bridge Hub inbound message block ${events.createdAtHash?.toHex()}.`
                    )
                ),
                filter((events) => {
                    let events_iter: any = events
                    let messageReceivedFound = false
                    for (const event of events_iter) {
                        let eventData = (event.event.toHuman() as any).data
                        if (
                            bridgeHub.events.ethereumInboundQueue.MessageReceived.is(event.event) &&
                            eventData.nonce === success?.nonce.toString() &&
                            eventData.messageId.toLowerCase() ===
                            success?.messageId.toLowerCase() &&
                            eventData.channelId.toLowerCase() === success?.channelId.toLowerCase()
                        ) {
                            messageReceivedFound = true
                            extrinsicNumber = event.phase.toPrimitive().applyExtrinsic
                        }

                        if (
                            bridgeHub.events.system.ExtrinsicSuccess.is(event.event) &&
                            event.phase.toPrimitive().applyExtrinsic == extrinsicNumber
                        ) {
                            extrinsicSuccess = true
                        }
                    }
                    return messageReceivedFound
                })
            ),
            { defaultValue: undefined }
        )
        if (receivedEvents === undefined) {
            throw Error("Timeout while waiting for Bridge Hub delivery.")
        }
        success.bridgeHub.extrinsicSuccess = extrinsicSuccess
        success.bridgeHub.extrinsicNumber = extrinsicNumber
        success.bridgeHub.events = receivedEvents
        if (!success.bridgeHub.extrinsicSuccess) {
            throw new Error("Message processing failed on Bridge Hub.")
        }
    }

    yield `Message delivered to Bridge Hub block ${success.bridgeHub.events?.createdAtHash?.toHex()}.`

    if (success.assetHub.events === undefined) {
        yield "Waiting for message delivery to Asset Hub."

        const issuedTo =
            success.plan.success.assetHub.paraId !== success.plan.success.destinationParaId
                ? paraIdToSovereignAccount("sibl", success.plan.success.destinationParaId)
                : success.plan?.success.beneficiaryAddress

        const { allEvents: receivedEvents, extrinsicSuccess } = await waitForMessageQueuePallet(
            assetHub,
            success.messageId,
            success.plan.success.bridgeHub.paraId,
            (eventRow) => {
                let event = eventRow as any
                let eventData = (event.event.toPrimitive() as any).data
                return (
                    assetHub.events.foreignAssets.Issued.is(event.event) &&
                    eventData[2].toString() === success?.plan.success?.amount.toString() &&
                    u8aToHex(decodeAddress(eventData[1])).toLowerCase() ===
                    issuedTo.toLowerCase() &&
                    eventData[0]?.parents === 2 &&
                    eventData[0]?.interior?.x2[0]?.globalConsensus?.ethereum?.chainId.toString() ===
                    success?.plan.success?.ethereumChainId.toString() &&
                    eventData[0]?.interior?.x2[1]?.accountKey20?.key.toLowerCase() ===
                    success?.plan.success?.token.toLowerCase()
                )
            },
            {
                scanBlocks: options.scanBlocks,
            }
        )

        success.assetHub.events = receivedEvents
        success.assetHub.extrinsicSuccess = extrinsicSuccess
        if (!success.assetHub.extrinsicSuccess) {
            throw new Error("Message processing failed on Asset Hub.")
        }
    }
    yield `Message delivered to Asset Hub block ${success.assetHub.events?.createdAtHash?.toHex()}.`

    if (success.destinationParachain !== undefined) {
        if (
            success.plan.success.assetHub.paraId !== success.plan.success.destinationParaId &&
            context.hasParachain(success.plan.success.destinationParaId) &&
            success.destinationParachain.events === undefined
        ) {
            yield `Waiting for delivery to destination parachain ${success.plan.success.destinationParaId}`

            const destParaApi = await context.parachain(success.plan.success.destinationParaId)
            const issuedTo = success.plan?.success.beneficiaryAddress

            const { allEvents: receivedEvents, extrinsicSuccess } = await waitForMessageQueuePallet(
                destParaApi,
                success.messageId,
                success.plan.success.assetHub.paraId,
                (eventRow) => {
                    let event = eventRow as any
                    let eventData = (event.event.toPrimitive() as any).data
                    return (
                        destParaApi.events.foreignAssets.Issued.is(event.event) &&
                        eventData[2].toString() === success?.plan.success?.amount.toString() &&
                        u8aToHex(decodeAddress(eventData[1])).toLowerCase() ===
                        issuedTo.toLowerCase() &&
                        eventData[0]?.parents === 2 &&
                        eventData[0]?.interior?.x2[0]?.globalConsensus?.ethereum?.chainId.toString() ===
                        success?.plan.success?.ethereumChainId.toString() &&
                        eventData[0]?.interior?.x2[1]?.accountKey20?.key.toLowerCase() ===
                        success?.plan.success?.token.toLowerCase()
                    )
                },
                {
                    scanBlocks: options.scanBlocks,
                }
            )
            success.destinationParachain.events = receivedEvents
            success.destinationParachain.extrinsicSuccess = extrinsicSuccess
            if (!success.destinationParachain?.extrinsicSuccess) {
                throw new Error(
                    `Message delivered failed on parachain ${success.plan.success.destinationParaId}.`
                )
            }
        }
        yield `Message delivered to parachain ${success.plan.success.destinationParaId
            } at block ${success.destinationParachain?.events?.createdAtHash?.toHex()}.`
    }

    yield "Transfer complete."
}
