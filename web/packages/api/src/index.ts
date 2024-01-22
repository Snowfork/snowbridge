// import '@polkadot/api-augment/polkadot'
import { ApiPromise, WsProvider } from '@polkadot/api'
import { ContractTransactionReceipt, LogDescription, Signer, ethers } from 'ethers'
import { BeefyClient, BeefyClient__factory, IGateway, IGateway__factory, IERC20__factory } from '@snowbridge/contract-types'
import { MultiAddressStruct } from '@snowbridge/contract-types/src/IGateway'
import { decodeAddress, } from '@polkadot/keyring'
import { hexToU8a, isHex, u8aToHex } from '@polkadot/util'
import { tap, take, takeWhile, lastValueFrom, map as rxmap } from 'rxjs'

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
                beacon: beaconOperatingMode as OperatingMode,
                inbound: inboundOperatingMode as OperatingMode,
                outbound: outboundOperatingMode as OperatingMode,
            },
            latestPolkadotBlockOnEthereum: latestBeefyBlock,
            latestPolkaotBlock: latestPolkadotBlock,
            blockLatency: beefyBlockLatency,
            latencySeconds: beefyLatencySeconds,
        },
        ethereumToPolkadot: {
            operatingMode: {
                outbound: ethereumOperatingMode == 0n ? 'Normal' : 'Halted' as OperatingMode,
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
                outbound: operatingMode == 0n ? 'Normal' : 'Halted' as OperatingMode
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
        existentialDeposit: number,
        foreignAssetExists: boolean,
        hasToken: boolean,
        tokenBalance: bigint,
        tokenSpendApproved: boolean,
        tokenSpendAllowance: bigint,
        lightClientLatencyTooHigh: boolean,
        lightClientLatencySeconds: number,
    }
}

export const planSendToken = async (context: Context, source: ethers.Addressable, destination: string, token: string, amount: bigint): Promise<SendTokenPlan> => {
    // TODO: Allow destinations that are not assethub and check existence of destination
    const assetHub = 1000
    // TODO: Allow destination fee
    const destinationFee = BigInt(0)

    const sourceAddress = await source.getAddress()
    const tokenContract = IERC20__factory.connect(token, context.ethereum.api)
    const tokenBalance = await tokenContract.balanceOf(sourceAddress)
    const hasToken = tokenBalance >= amount

    const allowance = await tokenContract.allowance(await source.getAddress(), await context.ethereum.contracts.gateway.getAddress())
    const tokenSpendApproved = allowance >= amount

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
                { AccountKey20: { key: token } },
            ]
        }
    })).toPrimitive() as { status: 'Live' }
    const foreignAssetExists = asset.status == 'Live'

    const fee = await context.ethereum.contracts.gateway.quoteSendTokenFee(token, assetHub, destinationFee)
    const abi = ethers.AbiCoder.defaultAbiCoder()

    const destinationBytes32 = u8aToHex(isHex(destination)
        ? hexToU8a(destination)
        : decodeAddress(destination))

    const destinationAddress: MultiAddressStruct = {
        kind: 1,
        data: abi.encode(['bytes32'], [destinationBytes32]),
    }
    // Destination account exists.
    const existentialDeposit = context.polkadot.api.assetHub.consts.balances.existentialDeposit.toPrimitive() as number
    const account = (await context.polkadot.api.assetHub.query.system.account(destinationBytes32)).toPrimitive() as { data: { free: string } }
    const destinationAccountExists = BigInt(account.data.free) > existentialDeposit

    const lightClientLatencyTooHigh = bridgeStatus.ethereumToPolkadot.latencySeconds > (60 * 60 * 3)
    const canSend = bridgeStatus.ethereumToPolkadot.operatingMode.outbound == 'Normal'
        && channelStatus.ethereumToPolkadot.operatingMode.outbound == 'Normal'
        && destinationAccountExists && foreignAssetExists && !lightClientLatencyTooHigh
        && tokenSpendApproved && hasToken

    if (canSend) {
        return {
            success: {
                fee: fee,
                sourceAddress: sourceAddress,
                token: token,
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
                bridgeOperational: bridgeStatus.ethereumToPolkadot.operatingMode.outbound === 'Normal',
                channelOperational: channelStatus.ethereumToPolkadot.operatingMode.outbound === 'Normal',
                destinationAccountExists: destinationAccountExists,
                existentialDeposit: existentialDeposit,
                foreignAssetExists: foreignAssetExists,
                hasToken: hasToken,
                tokenBalance: tokenBalance,
                tokenSpendApproved: tokenSpendApproved,
                tokenSpendAllowance: allowance,
                lightClientLatencyTooHigh: lightClientLatencyTooHigh,
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
        channelId: string,
        nonce: bigint,
        messageId: string,
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
    const messageAccepted = events.find(log => log.name == 'OutboundMessageAccepted')

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
            ...plan.success,
        }
    }
}

export const trackSendToken = async (context: Context, result: SendTokenResult, beaconUpdateTimeout = 10, nonceUpdateTimeout = 5) => {
    if (result.failure || !result.success) {
        throw new Error('Plan failed')
    }

    // Wait for light client
    let ethereumBlockNumber = result.success.ethereum.blockNumber
    let beaconUpdates = context.polkadot.api.bridgeHub.rx.query.ethereumBeaconClient.latestExecutionState()
    let lastBeaconUpdate = await lastValueFrom(
        beaconUpdates.pipe(
            rxmap(beaconUpdate => beaconUpdate.toPrimitive() as { blockNumber: number }),
            take(beaconUpdateTimeout),
            takeWhile(beaconUpdate => ethereumBlockNumber > beaconUpdate.blockNumber),
            tap(beaconUpdate => console.log('Beacon client %d blocks behind.', ethereumBlockNumber - beaconUpdate.blockNumber)),
        ),
        { defaultValue: { blockNumber: ethereumBlockNumber } }
    )
    if (ethereumBlockNumber < lastBeaconUpdate.blockNumber) {
        throw new Error('Timeout waiting for light client to include block')
    }
    console.log('Beacon client caught up.')

    // Wait for nonce
    let outboundNonce = Number(result.success.nonce)
    let nonceUpdates = context.polkadot.api.bridgeHub.rx.query.ethereumInboundQueue.nonce(result.success.channelId)
    let lastNonceUpdate = await lastValueFrom(
        nonceUpdates.pipe(
            rxmap(nonce => nonce.toPrimitive() as number),
            take(nonceUpdateTimeout),
            takeWhile(nonce => outboundNonce > nonce),
            tap(nonce => console.log('Inbound queue %d nonces behind.', outboundNonce - nonce)),
        ),
        { defaultValue: outboundNonce }
    )
    console.log((await context.polkadot.api.bridgeHub.query.system.number()).toPrimitive())
    if (outboundNonce < lastNonceUpdate) {
        throw new Error('Timeout waiting for message to be delivered to the inbound queue.')
    }
    console.log('Inbound queue caught up.')
}