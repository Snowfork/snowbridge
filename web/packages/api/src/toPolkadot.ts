import { MultiAddressStruct } from '@snowbridge/contract-types/src/IGateway'
import { decodeAddress } from '@polkadot/keyring'
import { filter, tap, take, takeWhile, lastValueFrom, map as rxmap, firstValueFrom } from 'rxjs'
import { Codec } from '@polkadot/types/types'
import { u8aToHex } from '@polkadot/util'
import { ContractTransactionReceipt, LogDescription, Signer, ethers } from 'ethers'
import { IGateway__factory } from '@snowbridge/contract-types'
import { Context } from './index'
import { channelStatusInfo, bridgeStatusInfo, assetStatusInfo } from './status'
import { paraIdToSovereignAccount, paraIdToChannelId, beneficiaryMultiAddress } from './utils'

export type SendValidationResult = {
    success?: {
        ethereumChainId: bigint,
        fee: bigint,
        sourceAddress: string,
        estimatedDeliverySeconds: number,
        estimatedDeliveryBlocks: number,
        destinationParaId: number,
        beneficiaryAddress: string,
        beneficiaryMultiAddress: MultiAddressStruct,
        destinationFee: bigint,
        token: string,
        amount: bigint,
        assetHub: {
            validatedAtHash: string,
            paraId: number,
        },
        bridgeHub: {
            validatedAtHash: string,
            paraId: number,
        },
    },
    failure?: {
        bridgeOperational: boolean,
        channelOperational: boolean,
        beneficiaryAccountExists: boolean,
        existentialDeposit: bigint,
        foreignAssetExists: boolean,
        tokenIsValidERC20: boolean,
        canPayFee: boolean,
        ethereumBalance: bigint,
        hasToken: boolean,
        tokenIsRegistered: boolean,
        tokenBalance: bigint,
        tokenSpendApproved: boolean,
        tokenSpendAllowance: bigint,
        lightClientLatencyIsAcceptable: boolean,
        lightClientLatencySeconds: number,
        destinationChainExists: boolean,
        hrmpChannelSetup: boolean,
    }
}

export const validateSend = async (context: Context, source: ethers.Addressable, beneficiary: string, tokenAddress: string, destinationParaId: number, amount: bigint, destinationFee: bigint, options = {
    acceptableLatencyInSeconds: 10800 /* 3 Hours */
}): Promise<SendValidationResult> => {
    const { ethereum, ethereum: { contracts: { gateway } }, polkadot: { api: { assetHub, bridgeHub, relaychain } } } = context

    const sourceAddress = await source.getAddress()

    // Asset checks
    const assetInfo = await assetStatusInfo(context, tokenAddress, sourceAddress)
    const tokenSpendApproved = assetInfo.tokenGatewayAllowance >= amount
    const hasToken = assetInfo.ownerBalance >= amount
    const tokenIsRegistered = assetInfo.isTokenRegistered
    const ethereumChainId = assetInfo.ethereumChainId
    const foreignAssetExists = assetInfo.foreignAsset !== null && assetInfo.foreignAsset.status === 'Live'

    let fee = 0n
    let ethereumBalance = 0n
    let canPayFee = false
    if (tokenIsRegistered) {
        ethereumBalance = await ethereum.api.getBalance(sourceAddress)
        fee = await gateway.quoteSendTokenFee(tokenAddress, destinationParaId, destinationFee)
        canPayFee = fee < ethereumBalance
    }

    const [assetHubHead, assetHubParaIdCodec, bridgeHubHead, bridgeHubParaIdCodec] = await Promise.all([
        assetHub.rpc.chain.getFinalizedHead(),
        assetHub.query.parachainInfo.parachainId(),
        bridgeHub.rpc.chain.getFinalizedHead(),
        bridgeHub.query.parachainInfo.parachainId(),
    ])

    const assetHubParaId = assetHubParaIdCodec.toPrimitive() as number;
    const assetHubChannelId = paraIdToChannelId(assetHubParaId)
    const [channelStatus, bridgeStatus] = await Promise.all([
        channelStatusInfo(context, assetHubChannelId),
        bridgeStatusInfo(context),
    ])

    let { address: beneficiaryAddress, hexAddress: beneficiaryHex } = beneficiaryMultiAddress(beneficiary)

    let beneficiaryAccountExists = true
    let destinationChainExists = true
    let hrmpChannelSetup = true
    const existentialDeposit = BigInt(assetHub.consts.balances.existentialDeposit.toPrimitive() as number)
    if (destinationParaId === assetHubParaId) {
        if (destinationFee !== 0n) throw new Error('Asset Hub does not require a destination fee.')
        if (beneficiaryAddress.kind !== 1) throw new Error('Asset Hub only supports 32 byte addresses.')
        const account = (await assetHub.query.system.account(beneficiaryHex)).toPrimitive() as { data: { free: string } }
        beneficiaryAccountExists = BigInt(account.data.free) > existentialDeposit
    } else {
        const [destinationHead, hrmpChannel] = await Promise.all([
            relaychain.query.paras.heads(destinationParaId),
            relaychain.query.hrmp.hrmpChannels({ sender: assetHubParaId, recipient: destinationParaId }),
        ])
        destinationChainExists = destinationHead.toPrimitive() !== null
        hrmpChannelSetup = hrmpChannel.toPrimitive() !== null
    }

    const lightClientLatencyIsAcceptable = bridgeStatus.toPolkadot.latencySeconds < options.acceptableLatencyInSeconds
    const bridgeOperational = bridgeStatus.toPolkadot.operatingMode.outbound === 'Normal' && bridgeStatus.toPolkadot.operatingMode.beacon === 'Normal'
    const channelOperational = channelStatus.toPolkadot.operatingMode.outbound === 'Normal'
    const canSend = bridgeOperational && channelOperational && canPayFee
        && beneficiaryAccountExists && foreignAssetExists && lightClientLatencyIsAcceptable
        && tokenSpendApproved && hasToken && tokenIsRegistered && destinationChainExists && hrmpChannelSetup

    if (canSend) {
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
            }
        }
    } else {
        return {
            failure: {
                // Bridge Status
                bridgeOperational: bridgeOperational,
                channelOperational: channelOperational,
                lightClientLatencyIsAcceptable: lightClientLatencyIsAcceptable,
                lightClientLatencySeconds: bridgeStatus.toPolkadot.latencySeconds,

                ethereumBalance,
                canPayFee,

                // Assets
                foreignAssetExists: foreignAssetExists,
                tokenIsRegistered: tokenIsRegistered,
                tokenIsValidERC20: assetInfo.tokenIsValidERC20,
                hasToken: hasToken,
                tokenBalance: assetInfo.ownerBalance,
                tokenSpendApproved: tokenSpendApproved,
                tokenSpendAllowance: assetInfo.tokenGatewayAllowance,

                // Beneficiary
                beneficiaryAccountExists: beneficiaryAccountExists,
                existentialDeposit: existentialDeposit,

                // Destination chain
                destinationChainExists: destinationChainExists,
                hrmpChannelSetup: hrmpChannelSetup,
            }
        }
    }
}

export type SendResult = {
    success?: {
        ethereum: {
            blockNumber: number,
            blockHash: string,
            transactionHash: string,
            events: LogDescription[],
        },
        bridgeHub: {
            submittedAtHash: string,
            beaconUpdate?: {
                createdAtHash?: `0x${string}`,
                blockNumber: number,
            },
            events?: Codec,
            extrinsicSuccess?: boolean
            extrinsicNumber?: number
        }
        assetHub: {
            submittedAtHash: string,
            events?: Codec,
            extrinsicSuccess?: boolean,
        },
        channelId: string,
        nonce: bigint,
        messageId: string,
        plan: SendValidationResult,
    }
    failure?: {
        receipt: ContractTransactionReceipt
    }
}

export const send = async (context: Context, signer: Signer, plan: SendValidationResult, confirmations = 1): Promise<SendResult> => {
    const { ethereum: { contracts: { gateway } }, polkadot: { api: { assetHub, bridgeHub } } } = context
    const { success } = plan

    if (plan.failure || !success) {
        throw new Error('Plan failed')
    }
    if (success.sourceAddress !== await signer.getAddress()) {
        throw new Error('Invalid signer')
    }

    // Get current heads to make tracking easier.
    const [assetHubHead, bridgeHubHead] = await Promise.all([
        assetHub.rpc.chain.getFinalizedHead(),
        bridgeHub.rpc.chain.getFinalizedHead(),
    ])

    const contract = IGateway__factory.connect(context.config.appContracts.gateway, signer)
    const response = await contract.sendToken(
        success.token,
        success.destinationParaId,
        success.beneficiaryMultiAddress,
        success.destinationFee,
        success.amount,
        {
            value: success.fee
        }
    )
    let receipt = await response.wait(confirmations)
    if (receipt === null) {
        throw new Error('Error waiting for transaction completion')
    }

    if (receipt?.status !== 1) {
        return {
            failure: {
                receipt: receipt,
            }
        }
    }
    const events: LogDescription[] = []
    receipt.logs.forEach(log => {
        let event = contract.interface.parseLog({
            topics: [...log.topics],
            data: log.data
        })
        if (event !== null) {
            events.push(event)
        }
    })
    const messageAccepted = events.find(log => log.name === 'OutboundMessageAccepted')

    return {
        success: {
            ethereum: {
                blockNumber: receipt.blockNumber,
                blockHash: receipt.blockHash,
                transactionHash: receipt.hash,
                events: events,
            },
            assetHub: {
                submittedAtHash: u8aToHex(assetHubHead)
            },
            bridgeHub: {
                submittedAtHash: u8aToHex(bridgeHubHead)
            },
            channelId: messageAccepted?.args[0],
            nonce: messageAccepted?.args[1],
            messageId: messageAccepted?.args[2],
            plan: plan,
        }
    }
}

export async function* trackSendProgress(context: Context, result: SendResult, beaconUpdateTimeout = 10, scanBlocks = 200): AsyncGenerator<string> {
    const { polkadot: { api: { assetHub, bridgeHub } } } = context
    const { success } = result

    if (result.failure || !success || !success.plan.success) {
        throw new Error('Send failed')
    }

    if (success.bridgeHub.beaconUpdate === undefined) {
        // Wait for light client
        const ethereumBlockNumber = success.ethereum.blockNumber
        const lastBeaconUpdate = await lastValueFrom(
            bridgeHub.rx.query.ethereumBeaconClient.latestExecutionState().pipe(
                rxmap(beaconUpdate => {
                    const update = beaconUpdate.toPrimitive() as { blockNumber: number }
                    return { createdAtHash: beaconUpdate.createdAtHash?.toHex(), blockNumber: update.blockNumber }
                }),
                take(beaconUpdateTimeout),
                takeWhile(({ blockNumber }) => ethereumBlockNumber > blockNumber),
                tap(({ createdAtHash, blockNumber }) => console.log(`Bridge Hub block ${createdAtHash}: Beacon client ${ethereumBlockNumber - blockNumber} blocks behind.`)),
            ),
            { defaultValue: undefined }
        )

        if (lastBeaconUpdate === undefined) {
            throw new Error('Timeout waiting for light client to include block')
        }
        success.bridgeHub.beaconUpdate = lastBeaconUpdate
    }
    yield `Included by light client in Bridge Hub block ${success.bridgeHub.beaconUpdate?.createdAtHash}. Waiting for message delivery to Bridge Hub.`

    if (success.bridgeHub.events === undefined) {
        // Wait for nonce
        let extrinsicNumber: number | undefined = undefined
        let extrinsicSuccess = false
        const receivedEvents = await firstValueFrom(
            bridgeHub.rx.query.system.events().pipe(
                take(scanBlocks),
                tap((events) => console.log(`Waiting for Bridge Hub inbound message block ${events.createdAtHash?.toHex()}.`)),
                filter(events => {
                    let events_iter: any = events
                    let messageReceivedFound = false
                    for (const event of events_iter) {
                        let eventData = (event.event.toHuman() as any).data
                        if (bridgeHub.events.ethereumInboundQueue.MessageReceived.is(event.event)
                            && eventData.nonce === success?.nonce.toString()
                            && eventData.messageId.toLowerCase() === success?.messageId.toLowerCase()
                            && eventData.channelId.toLowerCase() === success?.channelId.toLowerCase()) {

                            messageReceivedFound = true
                            extrinsicNumber = event.phase.toPrimitive().applyExtrinsic
                        }

                        if (bridgeHub.events.system.ExtrinsicSuccess.is(event.event) && event.phase.toPrimitive().applyExtrinsic == extrinsicNumber) {
                            extrinsicSuccess = true
                        }
                    }
                    return messageReceivedFound
                }),
            ),
            { defaultValue: undefined }
        )
        console.log(receivedEvents?.toHuman())
        if (receivedEvents === undefined) {
            throw Error('Timeout while waiting for Bridge Hub delivery.')
        }
        success.bridgeHub.extrinsicSuccess = extrinsicSuccess
        success.bridgeHub.extrinsicNumber = extrinsicNumber
        success.bridgeHub.events = receivedEvents
    }

    if (success.bridgeHub.extrinsicSuccess) {
        yield `Message delivered to Bridge Hub block ${success.bridgeHub.events?.createdAtHash?.toHex()}. Waiting for message delivery to Asset Hub.`
    } else {
        throw new Error('Message processing failed on Bridge Hub.')
    }

    if (success.assetHub.events === undefined) {
        let issuedTo = success.plan?.success.beneficiaryAddress
        if (success.plan.success.assetHub.paraId !== success.plan.success.destinationParaId) {
            issuedTo = paraIdToSovereignAccount("sibl", success.plan.success.destinationParaId)
        }
        let extrinsicSuccess = false
        let receivedEvents = await firstValueFrom(
            assetHub.rx.query.system.events().pipe(
                take(scanBlocks),
                tap((events) => console.log(`Waiting for Asset Hub inbound message block ${events.createdAtHash?.toHex()}.`)),
                filter(events => {
                    let foundMessageQueue = false
                    let foundAssetsIssued = false
                    let events_iter: any = events
                    for (const event of events_iter) {
                        let eventData = (event.event.toPrimitive() as any).data
                        if (assetHub.events.messageQueue.Processed.is(event.event)
                            && eventData[0].toLowerCase() === success?.messageId.toLowerCase()
                            && eventData[1]?.sibling === success?.plan.success?.bridgeHub.paraId) {

                            foundMessageQueue = true
                            extrinsicSuccess = eventData[3]
                        }
                        if (assetHub.events.foreignAssets.Issued.is(event.event)
                            && eventData[2].toString() === success?.plan.success?.amount.toString()
                            && u8aToHex(decodeAddress(eventData[1])).toLowerCase() === issuedTo.toLowerCase()
                            && eventData[0]?.parents === 2
                            && eventData[0]?.interior?.x2[0]?.globalConsensus?.ethereum?.chainId.toString() === success?.plan.success?.ethereumChainId.toString()
                            && eventData[0]?.interior?.x2[1]?.accountKey20?.key.toLowerCase() === success?.plan.success?.token.toLowerCase()) {

                            foundAssetsIssued = true
                        }
                    }
                    return foundMessageQueue && ((extrinsicSuccess && foundAssetsIssued) || !extrinsicSuccess)
                }),
            ),
            { defaultValue: undefined }
        )

        console.log(receivedEvents?.toHuman())
        if (receivedEvents === undefined) {
            throw Error('Timeout while waiting for Asset Hub delivery.')
        }
        success.assetHub.events = receivedEvents
        success.assetHub.extrinsicSuccess = extrinsicSuccess
    }
    if (success.assetHub.extrinsicSuccess) {
        yield `Message delivered to Asset Hub block ${success.assetHub.events?.createdAtHash?.toHex()}. Transfer complete.`
    } else {
        throw new Error('Message processing failed on Asset Hub.')
    }
}

// TODO: Register
//export type RegisterValidationResult = {
//    success?: {}
//    failure?: {}
//}
//
//export const validateRegister = async (context: Context, source: ethers.Addressable, beneficiary: string, tokenAddress: string, destinationParaId: number, amount: bigint, destinationFee: bigint, options={
//    acceptableLatencyInSeconds: 10800 /* 3 Hours */
//}): Promise<RegisterValidationResult> => {
//    return {}
//}
//
//export const register = async (context: Context, signer: Signer, plan: RegisterValidationResult, confirmations = 1): Promise<RegisterResult> => {
//    if (plan.failure || !plan.success) {
//        throw new Error('Plan failed')
//    }
//    if (plan.success.sourceAddress !== await signer.getAddress()) {
//        throw new Error('Invalid signer')
//    }
//}
//
//export async function* trackRegisterProgress(context: Context, result: RegisterResult, beaconUpdateTimeout = 10, scanBlocks = 200): AsyncGenerator<string> {
//}