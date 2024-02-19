import { MultiAddressStruct } from '@snowbridge/contract-types/src/IGateway'
import { decodeAddress } from '@polkadot/keyring'
import { filter, tap, take, takeWhile, lastValueFrom, map as rxmap, firstValueFrom } from 'rxjs'
import { Codec } from '@polkadot/types/types'
import { BlockHash } from '@polkadot/types/interfaces'
import { isHex, u8aToHex } from '@polkadot/util'
import { ContractTransactionReceipt, LogDescription, Signer, ethers } from 'ethers'
import { IGateway__factory, IERC20__factory } from '@snowbridge/contract-types'
import { Context } from './index'
import { channelStatusInfo, bridgeStatusInfo } from './status'
import { paraIdToSovereignAccount, paraIdToChannelId } from './utils'

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
            plannedAtHash: BlockHash,
            paraId: number,
        },
        bridgeHub: {
            plannedAtHash: BlockHash,
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

export const validateSend = async (context: Context, source: ethers.Addressable, beneficiary: string, tokenAddress: string, destinationParaId: number, amount: bigint, destinationFee: bigint): Promise<SendValidationResult> => {
    const [sourceAddress, gatewayAddress] = await Promise.all([
        source.getAddress(),
        context.ethereum.contracts.gateway.getAddress()
    ])
    const tokenContract = IERC20__factory.connect(tokenAddress, context.ethereum.api)
    let tokenBalance = BigInt(0)
    let allowance = BigInt(0)
    let tokenSpendApproved = false
    let tokenIsValidERC20 = true
    try {
        const [tokenBalance_, allowance_] = await Promise.all([
            tokenContract.balanceOf(sourceAddress),
            tokenContract.allowance(sourceAddress, gatewayAddress),
        ])
        allowance = allowance_;
        tokenBalance = tokenBalance_;
        tokenSpendApproved = allowance >= amount
    } catch {
        tokenIsValidERC20 = false
    }
    const hasToken = tokenBalance >= amount

    const [assetHubHead, assetHubParaId, bridgeHubHead, bridgeHubParaId] = await Promise.all([
        context.polkadot.api.assetHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.assetHub.query.parachainInfo.parachainId(),
        context.polkadot.api.bridgeHub.rpc.chain.getFinalizedHead(),
        context.polkadot.api.bridgeHub.query.parachainInfo.parachainId(),
    ])

    // Destination account exists.
    const assetHub = assetHubParaId.toPrimitive() as number;
    const assetHubChannelId = paraIdToChannelId(assetHub)
    console.log('AAAAAAA', assetHubChannelId)

    const [channelStatus, bridgeStatus, ethereumNetwork, tokenIsRegistered] = await Promise.all([
        channelStatusInfo(context, assetHubChannelId),
        bridgeStatusInfo(context),
        context.ethereum.api.getNetwork(),
        context.ethereum.contracts.gateway.isTokenRegistered(tokenAddress)
    ])
    let fee = BigInt(0)
    if (tokenIsRegistered) {
        fee = await context.ethereum.contracts.gateway.quoteSendTokenFee(tokenAddress, destinationParaId, destinationFee)
    }

    // Asset exists
    const ethereumChainId = ethereumNetwork.chainId
    const asset = (await context.polkadot.api.assetHub.query.foreignAssets.asset({
        parents: 2,
        interior: {
            X2: [
                { GlobalConsensus: { Ethereum: { chain_id: ethereumChainId } } },
                { AccountKey20: { key: tokenAddress } },
            ]
        }
    })).toPrimitive() as { status: 'Live' }
    const foreignAssetExists = asset !== null && asset.status === 'Live'

    const abi = ethers.AbiCoder.defaultAbiCoder()

    let beneficiaryAddress: MultiAddressStruct;
    let beneficiaryHex: string;
    if (isHex(beneficiary)) {
        beneficiaryHex = beneficiary
        if (beneficiary.length === 42) {
            // 20 byte address
            beneficiaryAddress = {
                kind: 2,
                data: abi.encode(['bytes20'], [beneficiaryHex]),
            }
        } else if (beneficiary.length === 66) {
            // 32 byte address
            beneficiaryAddress = {
                kind: 1,
                data: abi.encode(['bytes32'], [beneficiaryHex]),
            }
        } else {
            throw new Error('Unknown Beneficiary address format.')
        }
    } else {
        // SS58 address
        beneficiaryHex = u8aToHex(decodeAddress(beneficiary))
        beneficiaryAddress = {
            kind: 1,
            data: abi.encode(['bytes32'], [beneficiaryHex]),
        }
    }

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
    const lightClientLatencyIsAcceptable = bridgeStatus.toPolkadot.latencySeconds < (60 * 60 * 3) // 3 Hours
    const canSend = bridgeStatus.toPolkadot.operatingMode.outbound === 'Normal'
        && channelStatus.toPolkadot.operatingMode.outbound === 'Normal'
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
                    plannedAtHash: assetHubHead,
                    paraId: assetHub,
                },
                bridgeHub: {
                    plannedAtHash: bridgeHubHead,
                    paraId: bridgeHubParaId.toPrimitive() as number,
                },
            }
        }
    } else {
        return {
            failure: {
                bridgeOperational: bridgeStatus.toPolkadot.operatingMode.outbound === 'Normal' && bridgeStatus.toPolkadot.operatingMode.beacon === 'Normal',
                channelOperational: channelStatus.toPolkadot.operatingMode.outbound === 'Normal',
                beneficiaryAccountExists: beneficiaryAccountExists,
                existentialDeposit: existentialDeposit,
                foreignAssetExists: foreignAssetExists,
                tokenIsRegistered: tokenIsRegistered,
                tokenIsValidERC20: tokenIsValidERC20,
                hasToken: hasToken,
                tokenBalance: tokenBalance,
                tokenSpendApproved: tokenSpendApproved,
                tokenSpendAllowance: allowance,
                lightClientLatencyIsAcceptable: lightClientLatencyIsAcceptable,
                lightClientLatencySeconds: bridgeStatus.toPolkadot.latencySeconds,
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
            submittedAtHash: BlockHash,
            beaconUpdate?: {
                createdAtHash?: `0x${string}`,
                blockNumber: number,
            },
            events?: Codec,
        }
        assetHub: {
            submittedAtHash: BlockHash,
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
                submittedAtHash: assetHubHead
            },
            bridgeHub: {
                submittedAtHash: bridgeHubHead
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
