import { ApiPromise, WsProvider } from "@polkadot/api"
import createKeccakHash from "keccak"
import { publicKeyConvert } from "secp256k1"
import type { ValidatorSetId, BeefyId } from "@polkadot/types/interfaces/beefy/types"
import fs from "fs"
import path from "path"
import { u32, u64 } from "@polkadot/types-codec"
import { H256 } from "@polkadot/types/interfaces"
import { Struct } from "@polkadot/types"

interface AuthoritySet extends Struct {
    id: u64
    len: u32
    keysetCommitment: H256
}

async function generateBeefyCheckpoint() {
    const endpoint = process.env.RELAYCHAIN_ENDPOINT || "ws://127.0.0.1:9944"
    const beefyStartBlock = process.env.BEEFY_START_BLOCK
        ? parseInt(process.env.BEEFY_START_BLOCK)
        : 1
    const basedir = process.env.contract_dir || "../../../contracts"
    const BeefyStateFile = path.join(basedir, "beefy-state.json")

    const api1 = await ApiPromise.create({
        provider: new WsProvider(endpoint),
    })

    console.log(`waiting for header ${beefyStartBlock}...`)

    // eslint-disable-next-line no-async-promise-executor
    await new Promise<void>(async (resolve) => {
        const unsub = await api1.rpc.chain.subscribeFinalizedHeads((header) => {
            console.log(`Header #${header.number}`)
            if (header.number.toNumber() > beefyStartBlock) {
                unsub()
                resolve()
            }
        })
    })

    const blockHash = await api1.rpc.chain.getBlockHash(beefyStartBlock)

    const api = await api1.at(blockHash)

    const validatorSetId = await api.query.beefy.validatorSetId<ValidatorSetId>()
    const authorities = await api.query.beefy.authorities<BeefyId[]>()

    console.log("validatorSetId:", validatorSetId)
    console.log("authority length is:" + authorities.length)

    let addrs = []

    for (let i = 0; i < authorities.length; i++) {
        console.log("index is:" + i + ",authority is:" + authorities[i])
        try {
            let publicKey = publicKeyConvert(authorities[i], false).slice(1)
            let publicKeyHashed = createKeccakHash("keccak256")
                .update(Buffer.from(publicKey))
                .digest()
            addrs.push(publicKeyHashed.slice(12))
        } catch (err) {
            console.log(err)
        }
    }

    console.log("valid authority length is:" + addrs.length)

    let currentAuthorities, nextAuthorities

    currentAuthorities = await api.query.beefyMmrLeaf.beefyAuthorities<AuthoritySet>()
    nextAuthorities = await api.query.beefyMmrLeaf.beefyNextAuthorities<AuthoritySet>()

    const beefyCheckpoint = {
        startBlock: beefyStartBlock,
        current: {
            id: currentAuthorities.id.toNumber(),
            root: currentAuthorities.keysetCommitment.toHex(),
            length: currentAuthorities.len.toNumber(),
        },
        next: {
            id: nextAuthorities.id.toNumber(),
            root: nextAuthorities.keysetCommitment.toHex(),
            length: nextAuthorities.len.toNumber(),
        },
    }

    console.log("Configuring BeefyClient with initial BEEFY state", beefyCheckpoint)

    fs.writeFileSync(BeefyStateFile, JSON.stringify(beefyCheckpoint, null, 2), "utf8")
    console.log("Beefy state writing to dest file: " + BeefyStateFile)
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
generateBeefyCheckpoint()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
