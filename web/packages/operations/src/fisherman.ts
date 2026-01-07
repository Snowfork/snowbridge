import { Context, contextConfigFor, environment } from "@snowbridge/api"
import { BeefyClient } from "@snowbridge/contract-types"
import { AbstractProvider } from "ethers"
import { existsSync } from "fs"
import { readFile, writeFile } from "fs/promises"
import { ApiPromise } from "@polkadot/api"
import { sendForkVotingAlarm, sendFutureBlockVotingAlarm } from "./alarm"
import { pino, type Logger } from "pino"

const CheckpointFilepath = `checkpoint.json`
const CheckpointInterval = process.env["FISHERMAN_CHECKPOINT_INTERVAL"] || "5000" // blocks

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

export const run = async (): Promise<void> => {
    let env = "local_e2e"
    if (process.env.NODE_ENV !== undefined) {
        env = process.env.NODE_ENV
    }
    const snowbridgeEnv = environment.SNOWBRIDGE_ENV[env]
    if (snowbridgeEnv === undefined) {
        throw Error(`Unknown environment '${env}'`)
    }

    const ctx = new Context(contextConfigFor(env))

    const relaychain = await ctx.relaychain()
    await relaychain.isReady
    const ethereum = await ctx.ethereum()
    const beefyClient = await ctx.beefyClient()

    const latestFinalizedBeefyBlock = (
        await relaychain.rpc.chain.getHeader(
            (await relaychain.rpc.beefy.getFinalizedHead()).toU8a(),
        )
    ).number.toNumber()
    const latestEthereumBlock = await ethereum.getBlockNumber()
    const startBlock = await loadCheckPoint()
    let endBlock = Math.min(latestEthereumBlock, startBlock + parseInt(CheckpointInterval))
    logger.info(
        "Scanning NewTicket event from Beefy Client, blocks from %d to %d",
        startBlock,
        endBlock,
    )
    await scanNewTicket(
        snowbridgeEnv.name,
        relaychain,
        ethereum,
        beefyClient,
        startBlock,
        endBlock,
        latestFinalizedBeefyBlock,
    )
    await scanNewMMRRoot(
        snowbridgeEnv.name,
        relaychain,
        ethereum,
        beefyClient,
        startBlock,
        endBlock,
        latestFinalizedBeefyBlock,
    )
    logger.info("Saving checkpoint at block %d", endBlock)
    await saveCheckPoint(endBlock)
}

const loadCheckPoint = async () => {
    let checkpointBlock
    if (existsSync(CheckpointFilepath)) {
        const json = await readFile(CheckpointFilepath, "utf-8")
        const obj = JSON.parse(json)
        checkpointBlock = obj.lastProcessedBlock
    } else {
        checkpointBlock = process.env["FISHERMAN_START_BLOCK"]
            ? parseInt(process.env["FISHERMAN_START_BLOCK"])
            : 23423100
    }
    return checkpointBlock
}

const saveCheckPoint = async (blockNumber: number) => {
    const json = JSON.stringify(
        {
            lastProcessedBlock: blockNumber,
        },
        null,
        2,
    )
    await writeFile(CheckpointFilepath, json)
}

const scanNewTicket = async (
    network: string,
    relaychain: ApiPromise,
    ethereum: AbstractProvider,
    beefyClient: BeefyClient,
    startBlock: number,
    endBlock: number,
    latestBlock: number,
) => {
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
            await sendForkVotingAlarm(network, blockNumber)
        }
        if (beefyBlockNumber > latestBlock) {
            logger.fatal("Voting on a future block!")
            await sendFutureBlockVotingAlarm(network, blockNumber)
        }
    }
}

const scanNewMMRRoot = async (
    network: string,
    relaychain: ApiPromise,
    ethereum: AbstractProvider,
    beefyClient: BeefyClient,
    startBlock: number,
    endBlock: number,
    latestBlock: number,
) => {
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
            await sendForkVotingAlarm(network, blockNumber)
        }
        if (beefyBlockNumber > latestBlock) {
            logger.fatal("Voting on a future block!")
            await sendFutureBlockVotingAlarm(network, blockNumber)
        }
    }
}
