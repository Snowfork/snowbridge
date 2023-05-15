import { ApiPromise, WsProvider } from "@polkadot/api"
import { MerkleTree } from "merkletreejs"
import createKeccakHash from "keccak"
import { publicKeyConvert } from "secp256k1"
import type {
    ValidatorSetId,
    BeefyNextAuthoritySet,
    BeefyId,
} from "@polkadot/types/interfaces/beefy/types"
import fs from "fs"
import path from "path"

let endpoint = process.env.RELAYCHAIN_ENDPOINT || "ws://127.0.0.1:9944"
const beefyStartBlock = process.env.BEEFY_START_BLOCK ? parseInt(process.env.BEEFY_START_BLOCK) : 12
const BeefyStateFile =
    process.env.BEEFY_STATE_FILE || path.join(process.env.output_dir!, "beefy-state.json")

async function configureBeefy() {
    let api1 = await ApiPromise.create({
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

    let blockHash = await api1.rpc.chain.getBlockHash(beefyStartBlock)

    let api = await api1.at(blockHash)

    let validatorSetId = await api.query.beefy.validatorSetId<ValidatorSetId>()
    let authorities = await api.query.beefy.authorities<BeefyId[]>()

    let addrs = []
    for (let i = 0; i < authorities.length; i++) {
        let publicKey = publicKeyConvert(authorities[i], false).slice(1)
        let publicKeyHashed = createKeccakHash("keccak256").update(Buffer.from(publicKey)).digest()
        addrs.push(publicKeyHashed.slice(12))
    }

    let tree = createMerkleTree(addrs)

    let nextAuthorities = await api.query.mmrLeaf.beefyNextAuthorities<BeefyNextAuthoritySet>()

    let validatorSets = {
        current: {
            id: validatorSetId.toNumber(),
            root: tree.getHexRoot(),
            length: addrs.length,
        },
        next: {
            id: nextAuthorities.id.toNumber(),
            root: nextAuthorities.root.toHex(),
            length: nextAuthorities.len.toNumber(),
        },
    }

    console.log("Configuring BeefyClient with initial BEEFY state")
    console.log("Validator sets: ", validatorSets)

    fs.writeFileSync(BeefyStateFile, JSON.stringify({ validatorSets }, null, 2), "utf8")
    console.log("Beefy state writing to dest file: " + BeefyStateFile)

    return
}

function hasher(data: Buffer): Buffer {
    return createKeccakHash("keccak256").update(data).digest()
}

function createMerkleTree(leaves: Buffer[]) {
    const leafHashes = leaves.map((value) => hasher(value))
    const tree = new MerkleTree(leafHashes, hasher, {
        sortLeaves: false,
        sortPairs: true,
    })
    return tree
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
configureBeefy()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
