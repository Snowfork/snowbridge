import hre from "hardhat"

import { ApiPromise, WsProvider } from "@polkadot/api"
import { MerkleTree } from "merkletreejs"
import createKeccakHash from "keccak"
import { publicKeyConvert } from "secp256k1"
import type {
    ValidatorSetId,
    BeefyNextAuthoritySet,
    BeefyId,
} from "@polkadot/types/interfaces/beefy/types"

let endpoint = process.env.RELAYCHAIN_ENDPOINT

async function configureBeefy() {
    let [signer] = await hre.ethers.getSigners()

    let beefyDeployment = await hre.deployments.get("BeefyClient")
    let beefyClientContract = new hre.ethers.Contract(beefyDeployment.address, beefyDeployment.abi)
    let beefyClient = beefyClientContract.connect(signer)

    let api1 = await ApiPromise.create({
        provider: new WsProvider(endpoint),
    })

    console.log("waiting for header 29...")

    // eslint-disable-next-line no-async-promise-executor
    await new Promise<void>(async (resolve) => {
        const unsub = await api1.rpc.chain.subscribeFinalizedHeads((header) => {
            console.log(`Header #${header.number}`)
            if (header.number.toNumber() > 29) {
                unsub()
                resolve()
            }
        })
    })

    let blockHash = await api1.rpc.chain.getBlockHash(29)

    let api = await api1.at(blockHash)

    let validatorSetId = await api.query.beefy.validatorSetId<ValidatorSetId>()
    let authorities = await api.query.beefy.authorities<BeefyId[]>()

    let addrs = []
    for (let i = 0; i < authorities.length; ++i) {
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

    let tx = await beefyClient.initialize(29, validatorSets.current, validatorSets.next)

    await tx.wait()

    return
}

function hasher(data: Buffer): Buffer {
    return createKeccakHash("keccak256").update(data).digest()
}

function createMerkleTree(leaves: Buffer[]) {
    const leafHashes = leaves.map((value) => hasher(value))
    const tree = new MerkleTree(leafHashes, hasher, {
        sortLeaves: false,
        sortPairs: false,
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
