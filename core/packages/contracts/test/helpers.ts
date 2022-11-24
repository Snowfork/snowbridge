import rlp from "rlp"

import { keccakFromHexString, keccak } from "ethereumjs-util"
import { MerkleTree } from "merkletreejs"
import { ethers, Event, Wallet } from "ethers"
import _ from "lodash"
import secp256k1 from "secp256k1"
import seedrandom from "seedrandom"
import { keccak256, entropyToMnemonic } from "ethers/lib/utils"

import type { BeefyClient } from "../src/contracts/BeefyClient"

let readSetBits = (bitfield: ethers.BigNumber[]): number[] => {
    let bits = bitfield
        .map((i) => {
            let bf = BigInt(i.toString()).toString(2).split("")
            while (bf.length < 256) {
                bf.unshift("0")
            }
            return bf.reverse().join("")
        })
        .join("")
        .replace(/0*$/g, "")
        .split("")
        .map((c) => parseInt(c))

    let indices: number[] = []
    for (let [index, value] of bits.entries()) {
        if (value === 1) {
            indices.push(index)
        }
    }

    return indices
}

let encodeLog = (log: Event) => {
    return rlp.encode([log.address, log.topics, log.data]).toString("hex")
}

class ValidatorSet {
    wallets: Wallet[]
    id: number
    root: string
    length: number
    proofs: string[][]

    constructor(id: number, length: number) {
        let wallets: Wallet[] = []
        for (let i = 0; i < length; i++) {
            let entropy = entropyToMnemonic(keccak256(Buffer.from(`${i}`)))
            let wallet = ethers.Wallet.fromMnemonic(entropy)
            wallets.push(wallet)
        }

        let leaves = wallets.map((w) => keccakFromHexString(w.address))
        let merkleTree = new MerkleTree(leaves, keccak, {
            sortLeaves: false,
            sortPairs: false
        })

        this.wallets = wallets
        this.id = id
        this.root = merkleTree.getHexRoot()
        this.length = length
        this.proofs = leaves.map((leaf, index) => merkleTree.getHexProof(leaf, index))
    }

    createSignatureProof(index: number, commitmentHash: string): BeefyClient.ValidatorProofStruct {
        let wallet = this.wallets[index]
        let signature = secp256k1.ecdsaSign(
            ethers.utils.arrayify(commitmentHash),
            ethers.utils.arrayify(wallet.privateKey)
        )

        let buf = new Uint8Array(signature.signature.buffer)
        let r = buf.slice(0, 32)
        let s = buf.slice(32)

        return {
            signature: {
                v: signature.recid + 27,
                r,
                s
            },
            index: index,
            addr: wallet.address,
            merkleProof: this.proofs[index]
        }
    }

    createSignatureMultiProof(
        indices: number[],
        commitmentHash: string
    ): BeefyClient.ValidatorMultiProofStruct {
        let multiProof: BeefyClient.ValidatorMultiProofStruct = {
            signatures: [],
            indices: [],
            addrs: [],
            merkleProofs: []
        }

        for (let i of indices) {
            let proof = this.createSignatureProof(i, commitmentHash)
            multiProof.signatures.push(proof.signature)
            multiProof.indices.push(proof.index)
            multiProof.addrs.push(proof.addr)
            multiProof.merkleProofs.push(proof.merkleProof)
        }

        return multiProof
    }
}

function createRandomSubset(population: number, size: number) {
    seedrandom("test", { global: true })
    return _.runInContext()
        .sampleSize(Array.from(Array(population).keys()), size)
        .sort((a, b) => a - b)
}

export { encodeLog, createRandomSubset, ValidatorSet, readSetBits }
