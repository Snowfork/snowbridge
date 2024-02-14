// import '@polkadot/api-augment/polkadot'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { ContractTransactionReceipt, LogDescription, Signer, ethers } from 'ethers'
import { BeefyClient, BeefyClient__factory, IGateway, IGateway__factory, IERC20__factory } from '@snowbridge/contract-types'
import { MultiAddressStruct } from '@snowbridge/contract-types/src/IGateway'
import { decodeAddress } from '@polkadot/keyring'
import { hexToU8a, isHex, u8aToHex } from '@polkadot/util'
import { filter, tap, take, takeWhile, lastValueFrom, map as rxmap, firstValueFrom } from 'rxjs'
import { Codec } from '@polkadot/types/types'

interface Config {
    ethereum: {
        url: string
    }
    polkadot: {
        url: {
            bridgeHub: string
            assetHub: string
            relaychain: string
        }
    }
    appContracts: {
        gateway: string
        beefy: string
    }
}

interface AppContracts {
    gateway: IGateway
    beefyClient: BeefyClient
}

export class Context {
    config: Config
    ethereum: EthereumContext
    polkadot: PolkadotContext

    constructor(config: Config, ethereum: EthereumContext, polkadot: PolkadotContext) {
        this.config = config
        this.ethereum = ethereum
        this.polkadot = polkadot
    }
}

class EthereumContext {
    api: ethers.WebSocketProvider
    contracts: AppContracts

    constructor(api: ethers.WebSocketProvider, contracts: AppContracts) {
        this.api = api
        this.contracts = contracts
    }
}

class PolkadotContext {
    api: {
        relaychain: ApiPromise
        assetHub: ApiPromise
        bridgeHub: ApiPromise
    }
    constructor(relaychain: ApiPromise, assetHub: ApiPromise, bridgeHub: ApiPromise) {
        this.api = {
            relaychain: relaychain,
            assetHub: assetHub,
            bridgeHub: bridgeHub,
        }
    }
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export const contextFactory = async (config: Config): Promise<Context> => {
    let ethApi = new ethers.WebSocketProvider(config.ethereum.url)
    let relaychainApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.relaychain),
    })
    let assetHubApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.assetHub),
    })
    let bridgeHubApi = await ApiPromise.create({
        provider: new WsProvider(config.polkadot.url.bridgeHub),
    })

    let gatewayAddr = config.appContracts.gateway
    let beefyAddr = config.appContracts.beefy

    let appContracts: AppContracts = {
        //TODO: Get gateway address from bridgehub
        gateway: IGateway__factory.connect(gatewayAddr, ethApi),
        //TODO: Get beefy client from gateway
        beefyClient: BeefyClient__factory.connect(beefyAddr, ethApi),
    }

    let ethCtx = new EthereumContext(ethApi, appContracts)
    let polCtx = new PolkadotContext(relaychainApi, assetHubApi, bridgeHubApi)

    return new Context(config, ethCtx, polCtx)
}

export type OperatingMode = 'Normal' | 'Halted'

export const bridgeStatusInfo = async (context: Context) => {
    const latestBeefyBlock = Number(await context.ethereum.contracts.beefyClient.latestBeefyBlock())
    const latestPolkadotBlock = (await context.polkadot.api.relaychain.query.system.number()).toPrimitive() as number

    const latestBeaconState = (await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.latestExecutionState()).toPrimitive() as { blockNumber: number }
    const latestEthereumBlock = await context.ethereum.api.getBlockNumber()

    const polkadotBlockTimeInSeconds = 6
    const beefyBlockLatency = latestPolkadotBlock - latestBeefyBlock
    const beefyLatencySeconds = beefyBlockLatency * polkadotBlockTimeInSeconds

    const ethereumBlockTimeInSeconds = 12
    const beaconBlockLatency = latestEthereumBlock - latestBeaconState.blockNumber
    const beaconLatencySeconds = beaconBlockLatency * ethereumBlockTimeInSeconds

    const ethereumOperatingMode = await context.ethereum.contracts.gateway.operatingMode()
    const beaconOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumBeaconClient.operatingMode()).toPrimitive()
    const inboundOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumInboundQueue.operatingMode()).toPrimitive()
    const outboundOperatingMode = (await context.polkadot.api.bridgeHub.query.ethereumOutboundQueue.operatingMode()).toPrimitive()

    return {
        polkadotToEthereum: {
            operatingMode: {
                outbound: outboundOperatingMode as OperatingMode,
            },
            latestPolkadotBlockOnEthereum: latestBeefyBlock,
            latestPolkaotBlock: latestPolkadotBlock,
            blockLatency: beefyBlockLatency,
            latencySeconds: beefyLatencySeconds,
        },
        ethereumToPolkadot: {
            operatingMode: {
                beacon: beaconOperatingMode as OperatingMode,
                inbound: inboundOperatingMode as OperatingMode,
                outbound: ethereumOperatingMode === 0n ? 'Normal' : 'Halted' as OperatingMode,
            },
            latestEthereumBlockOnPolkadot: latestBeaconState.blockNumber,
            latestEthereumBlock: latestEthereumBlock,
            blockLatency: beaconBlockLatency,
            latencySeconds: beaconLatencySeconds,
        },
    }
}

export const channelStatusInfo = async (context: Context, channelId: string) => {
    const [inbound_nonce_eth, outbound_nonce_eth] = await context.ethereum.contracts.gateway.channelNoncesOf(channelId)
    const operatingMode = await context.ethereum.contracts.gateway.channelOperatingModeOf(channelId)
    const inbound_nonce_sub = (await context.polkadot.api.bridgeHub.query.ethereumInboundQueue.nonce(channelId)).toPrimitive() as number
    const outbound_nonce_sub = (await context.polkadot.api.bridgeHub.query.ethereumOutboundQueue.nonce(channelId)).toPrimitive() as number
    return {
        ethereumToPolkadot: {
            operatingMode: {
                outbound: operatingMode === 0n ? 'Normal' : 'Halted' as OperatingMode
            },
            outbound: Number(outbound_nonce_eth),
            inbound: inbound_nonce_sub,
        },
        polkadotToEthereum: {
            outbound: outbound_nonce_sub,
            inbound: Number(inbound_nonce_eth),
        },
    }
}

export type SendTokenPlan = {
    success?: {
        ethereumChainId: bigint,
        fee: bigint,
        sourceAddress: string,
        estimatedDeliverySeconds: number,
        estimatedDeliveryBlocks: number,
        destinationChain: number,
        destinationAddress: string,
        destinationMultiAddress: MultiAddressStruct,
        destinationFee: bigint,
        token: string,
        amount: bigint,
    },
    failure?: {
        bridgeOperational: boolean,
        channelOperational: boolean,
        destinationAccountExists: boolean,
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
    }
}

export const planSendToken = async (context: Context, source: ethers.Addressable, destination: string, tokenAddress: string, amount: bigint): Promise<SendTokenPlan> => {
    // TODO: Allow destinations that are not assethub and check existence of destination
    const assetHub = 1000
    // TODO: Allow destination fee
    const destinationFee = BigInt(0)

    const sourceAddress = await source.getAddress()
    const tokenContract = IERC20__factory.connect(tokenAddress, context.ethereum.api)
    let tokenBalance = BigInt(0)
    let allowance = BigInt(0)
    let tokenSpendApproved = false
    let tokenIsValidERC20 = true
    try {
        tokenBalance = await tokenContract.balanceOf(sourceAddress)
        allowance = await tokenContract.allowance(await source.getAddress(), await context.ethereum.contracts.gateway.getAddress())
        tokenSpendApproved = allowance >= amount
    } catch {
        tokenIsValidERC20 = false
    }
    const hasToken = tokenBalance >= amount

    // Check bridge status.
    const bridgeStatus = await bridgeStatusInfo(context)

    //TODO: Convert parachain to channel id
    const ASSET_HUB_CHANNEL_ID = '0xc173fac324158e77fb5840738a1a541f633cbec8884c6a601c567d2b376a0539'
    const channelStatus = await channelStatusInfo(context, ASSET_HUB_CHANNEL_ID)

    // Asset exists
    const ethereumChainId = (await context.ethereum.api.getNetwork()).chainId
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

    const tokenIsRegistered = await context.ethereum.contracts.gateway.isTokenRegistered(tokenAddress)
    let fee = BigInt(0)
    if (tokenIsRegistered) {
        fee = await context.ethereum.contracts.gateway.quoteSendTokenFee(tokenAddress, assetHub, destinationFee)
    }
    const abi = ethers.AbiCoder.defaultAbiCoder()

    const destinationBytes32 = u8aToHex(isHex(destination)
        ? hexToU8a(destination)
        : decodeAddress(destination))

    const destinationAddress: MultiAddressStruct = {
        kind: 1,
        data: abi.encode(['bytes32'], [destinationBytes32]),
    }
    // Destination account exists.
    const existentialDeposit = BigInt(context.polkadot.api.assetHub.consts.balances.existentialDeposit.toPrimitive() as number)
    const account = (await context.polkadot.api.assetHub.query.system.account(destinationBytes32)).toPrimitive() as { data: { free: string } }
    const destinationAccountExists = BigInt(account.data.free) > existentialDeposit

    const lightClientLatencyIsAcceptable = bridgeStatus.ethereumToPolkadot.latencySeconds < (60 * 60 * 3) // 3 Hours
    const canSend = bridgeStatus.ethereumToPolkadot.operatingMode.outbound === 'Normal'
        && channelStatus.ethereumToPolkadot.operatingMode.outbound === 'Normal'
        && destinationAccountExists && foreignAssetExists && lightClientLatencyIsAcceptable
        && tokenSpendApproved && hasToken && tokenIsRegistered

    if (canSend) {
        return {
            success: {
                fee: fee,
                sourceAddress: sourceAddress,
                token: tokenAddress,
                destinationChain: assetHub,
                destinationAddress: destinationBytes32,
                destinationMultiAddress: destinationAddress,
                destinationFee: destinationFee,
                ethereumChainId: ethereumChainId,
                amount: amount,
                estimatedDeliverySeconds: bridgeStatus.ethereumToPolkadot.latencySeconds * 2,
                estimatedDeliveryBlocks: bridgeStatus.ethereumToPolkadot.blockLatency * 2,
            }
        }
    } else {
        return {
            failure: {
                bridgeOperational: bridgeStatus.ethereumToPolkadot.operatingMode.outbound === 'Normal' && bridgeStatus.ethereumToPolkadot.operatingMode.beacon === 'Normal',
                channelOperational: channelStatus.ethereumToPolkadot.operatingMode.outbound === 'Normal',
                destinationAccountExists: destinationAccountExists,
                existentialDeposit: existentialDeposit,
                foreignAssetExists: foreignAssetExists,
                tokenIsRegistered: tokenIsRegistered,
                tokenIsValidERC20: tokenIsValidERC20,
                hasToken: hasToken,
                tokenBalance: tokenBalance,
                tokenSpendApproved: tokenSpendApproved,
                tokenSpendAllowance: allowance,
                lightClientLatencyIsAcceptable: lightClientLatencyIsAcceptable,
                lightClientLatencySeconds: bridgeStatus.ethereumToPolkadot.latencySeconds,
            }
        }
    }
}

export type SendTokenResult = {
    success?: {
        ethereum: {
            blockNumber: number,
            blockHash: string,
            transactionHash: string,
            events: LogDescription[],
        },
        bridgeHubEvents?: Codec,
        assetHubEvents?: Codec,
        lastBeaconUpdate?: {
            createdAtHash?: `0x${string}`;
            blockNumber: number;
        },
        channelId: string,
        nonce: bigint,
        messageId: string,
        plan: SendTokenPlan,
    }
    failure?: {
        receipt: ContractTransactionReceipt
    }
}

export const doSendToken = async (context: Context, signer: Signer, plan: SendTokenPlan, confirmations = 1): Promise<SendTokenResult> => {
    if (plan.failure || !plan.success) {
        throw new Error('Plan failed')
    }
    if (plan.success.sourceAddress !== await signer.getAddress()) {
        throw new Error('Invalid signer')
    }
    const contract = IGateway__factory.connect(context.config.appContracts.gateway, signer)
    const response = await contract.sendToken(
        plan.success.token,
        plan.success.destinationChain,
        plan.success.destinationMultiAddress,
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
            channelId: messageAccepted?.args[0],
            nonce: messageAccepted?.args[1],
            messageId: messageAccepted?.args[2],
            plan: plan,
        }
    }
}

export async function* trackSendToken(context: Context, result: SendTokenResult, beaconUpdateTimeout = 10, scanBlocks = 100): AsyncGenerator<string> {
    if (result.failure || !result.success) {
        throw new Error('Plan failed')
    }

    // Wait for light client
    let ethereumBlockNumber = result.success.ethereum.blockNumber
    let beaconUpdates = context.polkadot.api.bridgeHub.rx.query.ethereumBeaconClient.latestExecutionState()
    let lastBeaconUpdate = await lastValueFrom(
        beaconUpdates.pipe(
            rxmap(beaconUpdate => {
                const update = beaconUpdate.toPrimitive() as { blockNumber: number }
                return { createdAtHash: beaconUpdate.createdAtHash?.toHex(), blockNumber: update.blockNumber}
            }),
            take(beaconUpdateTimeout),
            takeWhile(({blockNumber}) => ethereumBlockNumber > blockNumber),
            tap(({createdAtHash, blockNumber}) => console.log(`Bridge Hub block ${createdAtHash}: Beacon client ${ethereumBlockNumber - blockNumber} blocks behind.`)),
        ),
        { defaultValue: undefined }
    )

    if (lastBeaconUpdate === undefined) {
        throw new Error('Timeout waiting for light client to include block')
    }
    result.success.lastBeaconUpdate = lastBeaconUpdate
    yield `Included by light client in Bridge Hub block ${lastBeaconUpdate.createdAtHash}. Waiting for message delivery to Bridge Hub.`

    // Wait for nonce
    let bridgeHubEvents = await firstValueFrom(
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
    console.log(bridgeHubEvents?.toHuman())
    if(bridgeHubEvents === undefined) {
        throw Error('Timeout while waiting for Bridge Hub delivery.')
    }
    result.success.bridgeHubEvents = bridgeHubEvents

    // TODO: Expect extrinsic success 
    yield `Message delivered to Bridge Hub block ${bridgeHubEvents?.createdAtHash?.toHex()}. Waiting for message delivery to Asset Hub.`

    let assetHubEvents = await firstValueFrom(
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
                        // TODO: Check bridgehub para id.
                        && eventData[1]?.sibling === 1013) {

                        foundMessageQueue = true
                    }
                    if (context.polkadot.api.assetHub.events.foreignAssets.Issued.is(event.event)
                        && eventData[2].toString() === result.success?.plan.success?.amount.toString()
                        && u8aToHex(decodeAddress(eventData[1])).toLowerCase() === result.success?.plan.success?.destinationAddress.toLowerCase()
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

    console.log(assetHubEvents?.toHuman())
    if(assetHubEvents === undefined) {
        throw Error('Timeout while waiting for Asset Hub delivery.')
    }
    // TODO: Expect extrinsic success
    result.success.assetHubEvents = assetHubEvents
    yield 'Transfer complete.'
}