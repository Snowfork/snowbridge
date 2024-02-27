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

export const validateSend = async (context: Context, source: ethers.Addressable, beneficiary: string, tokenAddress: string, destinationParaId: number, amount: bigint, destinationFee: bigint, options={
    acceptableLatencyInSeconds: 10800 /* 3 Hours */
}): Promise<SendValidationResult> => {
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
        ethereumBalance = await context.ethereum.api.getBalance(sourceAddress)
        fee = await context.ethereum.contracts.gateway.quoteSendTokenFee(tokenAddress, destinationParaId, destinationFee)
        canPayFee = fee < ethereumBalance
    }

    const [assetHubHead, assetHubParaId, bridgeHubHead, bridgeHubParaId] = await Promise.all([
        context.polkadot.api.assetHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.assetHub.query.parachainInfo.parachainId(),
        context.polkadot.api.bridgeHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.bridgeHub.query.parachainInfo.parachainId(),
    ])

    const assetHub = assetHubParaId.toPrimitive() as number;
    const assetHubChannelId = paraIdToChannelId(assetHub)
    const [channelStatus, bridgeStatus] = await Promise.all([
        channelStatusInfo(context, assetHubChannelId),
        bridgeStatusInfo(context),
    ])

    let { address: beneficiaryAddress, hexAddress: beneficiaryHex } = beneficiaryMultiAddress(beneficiary)

    let beneficiaryAccountExists = true
    let destinationChainExists = true
    let hrmpChannelSetup = true
    const existentialDeposit = BigInt(context.polkadot.api.assetHub.consts.balances.existentialDeposit.toPrimitive() as number)
    if (destinationParaId === assetHub) {
        if (destinationFee !== 0n) throw new Error('Asset Hub does not require a destination fee.')
        if (beneficiaryAddress.kind !== 1) throw new Error('Asset Hub only supports 32 byte addresses.')
        const account = (await context.polkadot.api.assetHub.query.system.account(beneficiaryHex)).toPrimitive() as { data: { free: string } }
        beneficiaryAccountExists = BigInt(account.data.free) > existentialDeposit
    } else {
        const [destinationHead, hrmpChannel] = await Promise.all([
            context.polkadot.api.relaychain.query.paras.heads(destinationParaId),
            context.polkadot.api.relaychain.query.hrmp.hrmpChannels({ sender: assetHub, recipient: destinationParaId }),
        ])
        destinationChainExists = destinationHead.toPrimitive() !== null
        hrmpChannelSetup = hrmpChannel.toPrimitive() !== null
    }

    // TODO: Make acceptable latency configurable.
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
                    paraId: assetHub,
                    validatedAtHash: u8aToHex(assetHubHead),
                },
                bridgeHub: {
                    paraId: bridgeHubParaId.toPrimitive() as number,
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
        }
        assetHub: {
            submittedAtHash: string,
            events?: Codec,
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
    if (plan.failure || !plan.success) {
        throw new Error('Plan failed')
    }
    if (plan.success.sourceAddress !== await signer.getAddress()) {
        throw new Error('Invalid signer')
    }

    // Get current heads to make tracking easier.
    const [assetHubHead, bridgeHubHead] = await Promise.all([
        context.polkadot.api.assetHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.bridgeHub.rpc.chain.getFinalizedHead(),
    ])

    const contract = IGateway__factory.connect(context.config.appContracts.gateway, signer)
    const response = await contract.sendToken(
        plan.success.token,
        plan.success.destinationParaId,
        plan.success.beneficiaryMultiAddress,
        plan.success.destinationFee,
        plan.success.amount,
        {
            value: plan.success.fee
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
    if (result.failure || !result.success || !result.success.plan.success) {
        throw new Error('Send failed')
    }

    if (result.success.bridgeHub.beaconUpdate === undefined) {
        // Wait for light client
        const ethereumBlockNumber = result.success.ethereum.blockNumber
        const lastBeaconUpdate = await lastValueFrom(
            context.polkadot.api.bridgeHub.rx.query.ethereumBeaconClient.latestExecutionState().pipe(
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
        result.success.bridgeHub.beaconUpdate = lastBeaconUpdate
    }
    yield `Included by light client in Bridge Hub block ${result.success.bridgeHub.beaconUpdate?.createdAtHash}. Waiting for message delivery to Bridge Hub.`

    if (result.success.bridgeHub.events === undefined) {
        // Wait for nonce
        const receivedEvents = await firstValueFrom(
            context.polkadot.api.bridgeHub.rx.query.system.events().pipe(
                take(scanBlocks),
                tap((events) => console.log(`Waiting for Bridge Hub inbound message block ${events.createdAtHash?.toHex()}.`)),
                filter(events => {
                    let events_iter: any = events
                    for (const event of events_iter) {
                        let eventData = (event.event.toHuman() as any).data
                        if (context.polkadot.api.bridgeHub.events.ethereumInboundQueue.MessageReceived.is(event.event)
                            && eventData.nonce === result.success?.nonce.toString()
                            && eventData.messageId.toLowerCase() === result.success?.messageId.toLowerCase()
                            && eventData.channelId.toLowerCase() === result.success?.channelId.toLowerCase()) {

                            return true
                        }
                    }
                    return false
                }),
            ),
            { defaultValue: undefined }
        )
        console.log(receivedEvents?.toHuman())
        if (receivedEvents === undefined) {
            throw Error('Timeout while waiting for Bridge Hub delivery.')
        }
        result.success.bridgeHub.events = receivedEvents
    }

    // TODO: Expect extrinsic success 
    yield `Message delivered to Bridge Hub block ${result.success.bridgeHub.events?.createdAtHash?.toHex()}. Waiting for message delivery to Asset Hub.`

    if (result.success.assetHub.events === undefined) {
        let issuedTo = result.success.plan.success.beneficiaryAddress
        if (result.success.plan.success.assetHub.paraId !== result.success.plan.success.destinationParaId) {
            issuedTo = paraIdToSovereignAccount("sibl", result.success.plan.success.destinationParaId)
        }
        let receivedEvents = await firstValueFrom(
            context.polkadot.api.assetHub.rx.query.system.events().pipe(
                take(scanBlocks),
                tap((events) => console.log(`Waiting for Asset Hub inbound message block ${events.createdAtHash?.toHex()}.`)),
                filter(events => {
                    let foundMessageQueue = false
                    let foundAssetsIssued = false
                    let events_iter: any = events
                    for (const event of events_iter) {
                        let eventData = (event.event.toPrimitive() as any).data
                        if (context.polkadot.api.assetHub.events.messageQueue.Processed.is(event.event)
                            && eventData[0].toLowerCase() === result.success?.messageId.toLowerCase()
                            && eventData[1]?.sibling === result.success?.plan.success?.bridgeHub.paraId) {

                            foundMessageQueue = true
                        }
                        if (context.polkadot.api.assetHub.events.foreignAssets.Issued.is(event.event)
                            && eventData[2].toString() === result.success?.plan.success?.amount.toString()
                            && u8aToHex(decodeAddress(eventData[1])).toLowerCase() === issuedTo.toLowerCase()
                            && eventData[0]?.parents === 2
                            && eventData[0]?.interior?.x2[0]?.globalConsensus?.ethereum?.chainId.toString() === result.success?.plan.success?.ethereumChainId.toString()
                            && eventData[0]?.interior?.x2[1]?.accountKey20?.key.toLowerCase() === result.success?.plan.success?.token.toLowerCase()) {

                            foundAssetsIssued = true
                        }
                    }
                    return foundMessageQueue && foundAssetsIssued
                }),
            ),
            { defaultValue: undefined }
        )

        console.log(receivedEvents?.toHuman())
        if (receivedEvents === undefined) {
            throw Error('Timeout while waiting for Asset Hub delivery.')
        }
        // TODO: Expect extrinsic success
        result.success.assetHub.events = receivedEvents
    }
    yield `Message delivered to Asset Hub block ${result.success.assetHub.events?.createdAtHash?.toHex()}. Transfer complete.`
}
