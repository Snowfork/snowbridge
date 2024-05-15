import { ApiPromise, WsProvider } from "@polkadot/api"
import { MerkleTree } from "merkletreejs"
import createKeccakHash from "keccak"
import { publicKeyConvert } from "secp256k1"
import type {
    ValidatorSetId,
    BeefyId,
} from "@polkadot/types/interfaces/beefy/types"
import fs from "fs"
import path from "path"
import { u32, u64 } from "@polkadot/types-codec";
import { H256 } from "@polkadot/types/interfaces";
import { Struct } from "@polkadot/types";

interface AuthoritySet extends Struct {
    id: u64;
    len: u32;
    keysetCommitment: H256;
}

async function generateBeefyCheckpoint() {
    let beefyBlock = parseInt(process.env.BEEFY_BLOCK!, 10)
    if (!Number.isInteger(beefyBlock) || beefyBlock == 0) {
        throw new Error("InvalidBeefyBlock")
    }

    let _api = await ApiPromise.create({
        provider: new WsProvider("wss://polkadot-rpc.dwellir.com"),
    })

    let api = await _api.at(await _api.rpc.chain.getBlockHash(beefyBlock))

    let authorities = await api.query.beefyMmrLeaf.beefyAuthorities<AuthoritySet>()
    let nextAuthorities = await api.query.beefyMmrLeaf.beefyNextAuthorities<AuthoritySet>()

    let beefyCheckpoint = {
        startBlock: beefyBlock,
        current: {
            id: authorities.id.toNumber(),
            root: authorities.keysetCommitment.toHex(),
            length: authorities.len.toNumber(),
        },
        next: {
            id: nextAuthorities.id.toNumber(),
            root: nextAuthorities.keysetCommitment.toHex(),
            length: nextAuthorities.len.toNumber(),
        },
    }

    console.log(JSON.stringify(beefyCheckpoint, null, 2))
}

generateBeefyCheckpoint()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
