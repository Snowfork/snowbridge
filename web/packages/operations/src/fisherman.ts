import { createApi } from "@snowbridge/api"
import { EthersEthereumProvider } from "@snowbridge/provider-ethers"
import { BeefyClient, BeefyClient__factory } from "@snowbridge/contract-types"
import { AbstractProvider } from "ethers"
import { existsSync } from "fs"
import { readFile, rename, writeFile } from "fs/promises"
import { ApiPromise } from "@polkadot/api"
import { sendForkVotingAlarm, sendFutureBlockVotingAlarm } from "./alarm"
import { pino, type Logger } from "pino"
import { bridgeInfoFor } from "@snowbridge/registry"

const CheckpointFilepath = process.env["FISHERMAN_CHECKPOINT_PATH"] || `checkpoint.json`
const CheckpointInterval = process.env["FISHERMAN_CHECKPOINT_INTERVAL"] || "5000" // blocks

export type Equivocations = { ForkVoting: number; FutureBlockVoting: number }

export type Checkpoint = {
    lastProcessedBlock: number
    equivocations: Equivocations
}

export type FishermanScanResult = {
    lastScannedBlock: number
    latestEthereumBlock: number
    detected: Equivocations
}

const getLogger = (): Logger => {
    return pino({
        transport: {
            target: "pino-pretty",
            options: {
                colorize: true,
            },
        },
        level: process.env.PINO_LOG_LEVEL || "info",

        redact: [], // prevent logging of sensitive data
    })
}

let logger = getLogger()

export const run = async (sendCloudWatchAlarms = true): Promise<FishermanScanResult> => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const info = bridgeInfoFor(env)
    const { environment: snowbridgeEnv } = info
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const ctx = createApi({ info, ethereumProvider: new EthersEthereumProvider() }).context

    try {
        const relaychain = await ctx.relaychain()
        await relaychain.isReady
        const ethereum = ctx.ethereum()
        const beefyClient = BeefyClient__factory.connect(ctx.environment.beefyContract)

        const latestFinalizedBeefyBlock = (
            await relaychain.rpc.chain.getHeader(
                (await relaychain.rpc.beefy.getFinalizedHead()).toU8a(),
            )
        ).number.toNumber()
        const latestEthereumBlock = await ethereum.getBlockNumber()
        const checkpoint = await loadCheckPoint()
        const startBlock = checkpoint.lastProcessedBlock
        let endBlock = Math.min(latestEthereumBlock, startBlock + parseInt(CheckpointInterval))
        logger.info(
            "Scanning NewTicket event from Beefy Client, blocks from %d to %d",
            startBlock,
            endBlock,
        )
        const fromTickets = await scanNewTicket(
            snowbridgeEnv.name,
            relaychain,
            ethereum,
            beefyClient,
            startBlock,
            endBlock,
            latestFinalizedBeefyBlock,
            sendCloudWatchAlarms,
        )
        const fromMMRRoots = await scanNewMMRRoot(
            snowbridgeEnv.name,
            relaychain,
            ethereum,
            beefyClient,
            startBlock,
            endBlock,
            latestFinalizedBeefyBlock,
            sendCloudWatchAlarms,
        )

        const detected: Equivocations = {
            ForkVoting: fromTickets.ForkVoting + fromMMRRoots.ForkVoting,
            FutureBlockVoting: fromTickets.FutureBlockVoting + fromMMRRoots.FutureBlockVoting,
        }
        logger.info("Saving checkpoint at block %d", endBlock)
        await saveCheckPoint(endBlock, {
            ForkVoting: checkpoint.equivocations.ForkVoting + detected.ForkVoting,
            FutureBlockVoting:
                checkpoint.equivocations.FutureBlockVoting + detected.FutureBlockVoting,
        })

        return { lastScannedBlock: endBlock, latestEthereumBlock, detected }
    } finally {
        await ctx.destroyContext()
    }
}

export const loadCheckPoint = async (): Promise<Checkpoint> => {
    if (!existsSync(CheckpointFilepath)) {
        return {
            lastProcessedBlock: process.env["FISHERMAN_START_BLOCK"]
                ? parseInt(process.env["FISHERMAN_START_BLOCK"])
                : 23423100,
            equivocations: { ForkVoting: 0, FutureBlockVoting: 0 },
        }
    }
    const json = await readFile(CheckpointFilepath, "utf-8")
    let obj
    try {
        obj = JSON.parse(json)
    } catch (e) {
        // Never fall back to the default start block here: that silently rewinds the
        // scanner by hundreds of thousands of blocks.
        throw Error(`Corrupt fisherman checkpoint at ${CheckpointFilepath}: ${e}`)
    }
    return {
        lastProcessedBlock: obj.lastProcessedBlock,
        equivocations: {
            ForkVoting: obj.equivocations?.ForkVoting ?? 0,
            FutureBlockVoting: obj.equivocations?.FutureBlockVoting ?? 0,
        },
    }
}

const saveCheckPoint = async (blockNumber: number, equivocations: Equivocations) => {
    const json = JSON.stringify(
        {
            lastProcessedBlock: blockNumber,
            equivocations,
        },
        null,
        2,
    )
    // Block and counts must land together, so a kill mid-scan either re-scans the range
    // or skips it, never one without the other.
    await writeFile(`${CheckpointFilepath}.tmp`, json)
    await rename(`${CheckpointFilepath}.tmp`, CheckpointFilepath)
}

const scanNewTicket = async (
    network: string,
    relaychain: ApiPromise,
    ethereum: AbstractProvider,
    beefyClient: BeefyClient,
    startBlock: number,
    endBlock: number,
    latestBlock: number,
    sendCloudWatchAlarms: boolean,
): Promise<Equivocations> => {
    const detected: Equivocations = { ForkVoting: 0, FutureBlockVoting: 0 }
    const pastEvents = await beefyClient.queryFilter(
        beefyClient.filters.NewTicket(),
        startBlock,
        endBlock,
    )
    for (let event of pastEvents) {
        const blockNumber = event.blockNumber
        logger.info("Past NewTicket: %d", blockNumber)
        let tx = await ethereum.getTransaction(event.transactionHash)
        const parseTransaction = beefyClient.interface.parseTransaction({
            data: tx?.data || "",
        })
        const commitment = parseTransaction?.args[0]
        const beefyBlockNumber = commitment?.blockNumber
        const beefyMMRRoot = commitment?.payload[0].data
        logger.info("Beefy Commitment: %o", commitment)
        const beefyBlockHash = await relaychain.rpc.chain.getBlockHash(beefyBlockNumber)
        const canonicalMMRRoot = await relaychain.rpc.mmr.root(beefyBlockHash)
        logger.info("Canonical MMR Root: %s", canonicalMMRRoot.toHex())
        if (canonicalMMRRoot.toHex() != beefyMMRRoot) {
            logger.fatal("MMR Root mismatch!")
            detected.ForkVoting++
            if (sendCloudWatchAlarms) {
                await sendForkVotingAlarm(network, blockNumber)
            }
        }
        if (beefyBlockNumber > latestBlock) {
            logger.fatal("Voting on a future block!")
            detected.FutureBlockVoting++
            if (sendCloudWatchAlarms) {
                await sendFutureBlockVotingAlarm(network, blockNumber)
            }
        }
    }
    return detected
}

const scanNewMMRRoot = async (
    network: string,
    relaychain: ApiPromise,
    ethereum: AbstractProvider,
    beefyClient: BeefyClient,
    startBlock: number,
    endBlock: number,
    latestBlock: number,
    sendCloudWatchAlarms: boolean,
): Promise<Equivocations> => {
    const detected: Equivocations = { ForkVoting: 0, FutureBlockVoting: 0 }
    const pastEvents = await beefyClient.queryFilter(
        beefyClient.filters.NewMMRRoot(),
        startBlock,
        endBlock,
    )
    for (let event of pastEvents) {
        const blockNumber = event.blockNumber
        logger.info("Past NewMMRRoot: %d", blockNumber)
        const beefyMMRRoot = event.args.mmrRoot
        const beefyBlockNumber = event.args.blockNumber
        logger.info("Past NewMMRRoot: %o", event.args)
        const beefyBlockHash = await relaychain.rpc.chain.getBlockHash(beefyBlockNumber)
        const canonicalMMRRoot = await relaychain.rpc.mmr.root(beefyBlockHash)
        logger.info("Canonical MMR Root: %s", canonicalMMRRoot.toHex())
        if (canonicalMMRRoot.toHex() != beefyMMRRoot) {
            logger.fatal("MMR Root mismatch!")
            detected.ForkVoting++
            if (sendCloudWatchAlarms) {
                await sendForkVotingAlarm(network, blockNumber)
            }
        }
        if (beefyBlockNumber > latestBlock) {
            logger.fatal("Voting on a future block!")
            detected.FutureBlockVoting++
            if (sendCloudWatchAlarms) {
                await sendFutureBlockVotingAlarm(network, blockNumber)
            }
        }
    }
    return detected
}
