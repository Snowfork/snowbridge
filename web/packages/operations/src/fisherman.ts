import { Context, contextConfigFor, environment } from "@snowbridge/api"
import { BeefyClient } from "@snowbridge/contract-types"
import { AbstractProvider } from "ethers"
import { existsSync } from "fs"
import { readFile, writeFile } from "fs/promises"
import { ApiPromise } from "@polkadot/api"
import { sendForkVotingAlarm } from "./alarm"

const CheckpointFilepath = `checkpoint.json`
const CheckpointInterval = 1000 // blocks

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
            (await relaychain.rpc.beefy.getFinalizedHead()).toU8a()
        )
    ).number.toNumber()
    const startBlock = await loadCheckPoint()
    let endBlock = Math.min(latestFinalizedBeefyBlock, startBlock + CheckpointInterval)
    console.log(
        "Scaning NewTicket event from Beefy Client, blocks from %d to %d",
        startBlock,
        endBlock
    )
    await scanNewTicket(snowbridgeEnv.name, relaychain, ethereum, beefyClient, startBlock, endBlock)
    await scanNewMMRRoot(
        snowbridgeEnv.name,
        relaychain,
        ethereum,
        beefyClient,
        startBlock,
        endBlock
    )
    console.log("Saving checkpoint at block %d", endBlock)
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
        2
    )
    console.log("Save Checkpoint:", json)
    await writeFile(CheckpointFilepath, json)
}

const scanNewTicket = async (
    network: string,
    relaychain: ApiPromise,
    ethereum: AbstractProvider,
    beefyClient: BeefyClient,
    startBlock: number,
    endBlock: number
) => {
    const pastEvents = await beefyClient.queryFilter(
        beefyClient.filters.NewTicket(),
        startBlock,
        endBlock
    )
    for (let event of pastEvents) {
        console.log("Past NewTicket:", event.args.relayer, event.args.blockNumber)
        console.log("tx:", event.transactionHash)
        let tx = await ethereum.getTransaction(event.transactionHash)
        const parseTransaction = beefyClient.interface.parseTransaction({
            data: tx?.data || "",
        })
        const commitment = parseTransaction?.args[0]
        const beefyBlockNumber = commitment?.blockNumber
        const beefyMMRRoot = commitment?.payload[0].data
        console.log("Beefy Block Number:", beefyBlockNumber)
        console.log("Beefy MMR Root:", beefyMMRRoot)
        const beefyBlockHash = await relaychain.rpc.chain.getBlockHash(beefyBlockNumber)
        console.log("Beefy Block Hash:", beefyBlockHash.toHex())
        const canonicalMMRRoot = await relaychain.rpc.mmr.root(beefyBlockHash)
        console.log("Canonical MMR Root:", canonicalMMRRoot.toHex())
        if (canonicalMMRRoot.toHex() != beefyMMRRoot) {
            console.error("MMR Root mismatch!")
            await sendForkVotingAlarm(network, beefyBlockNumber)
        } else {
            console.log("MMR Root match.")
        }
    }
}

const scanNewMMRRoot = async (
    network: string,
    relaychain: ApiPromise,
    ethereum: AbstractProvider,
    beefyClient: BeefyClient,
    startBlock: number,
    endBlock: number
) => {
    const pastEvents = await beefyClient.queryFilter(
        beefyClient.filters.NewMMRRoot(),
        startBlock,
        endBlock
    )
    for (let event of pastEvents) {
        const beefyMMRRoot = event.args.mmrRoot
        const beefyBlockNumber = event.args.blockNumber
        console.log("Past NewMMRRoot:", beefyMMRRoot, beefyBlockNumber)
        const beefyBlockHash = await relaychain.rpc.chain.getBlockHash(beefyBlockNumber)
        console.log("Beefy Block Hash:", beefyBlockHash.toHex())
        const canonicalMMRRoot = await relaychain.rpc.mmr.root(beefyBlockHash)
        console.log("Canonical MMR Root:", canonicalMMRRoot.toHex())
        if (canonicalMMRRoot.toHex() != beefyMMRRoot) {
            console.error("MMR Root mismatch!")
            await sendForkVotingAlarm(network, Number(beefyBlockNumber))
        } else {
            console.log("MMR Root match.")
        }
    }
}
