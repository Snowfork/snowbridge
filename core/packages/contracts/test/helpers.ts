import rlp from "rlp"

import { keccakFromHexString, keccak } from "ethereumjs-util"
import { MerkleTree } from "merkletreejs"
import { ethers, Event, Wallet } from "ethers"
import _ from "lodash"
import secp256k1 from "secp256k1"

function createMerkleTree(values: string[]) {
    let leaves = values.map((value) => keccakFromHexString(value))
    let merkleTree = new MerkleTree(leaves, keccak, {
        sortLeaves: false,
        sortPairs: false,
    })
    return merkleTree
}

let encodeLog = (log: Event) => {
    return rlp.encode([log.address, log.topics, log.data]).toString("hex")
}

function printBitfield(bitfield: ethers.BigNumber[]) {
    return bitfield
        .map((i) => {
            let bf = BigInt(i.toString()).toString(2).split("")
            while (bf.length < 256) {
                bf.unshift("0")
            }
            return bf.join("")
        })
        .reverse()
        .join("")
        .replace(/^0*/g, "")
}

type WalletsByLeaf = {
    [key: string]: Wallet
}

interface ValidatorFixture {
    wallets: Wallet[]
    walletsByLeaf: WalletsByLeaf
    validatorAddresses: string[]
    validatorAddressesHashed: string[]
    validatorSetID: number
    validatorSetRoot: string
    validatorSetLength: number
    validatorAddressProofs: string[][]
    validatorMerkleTree: MerkleTree
}

function createValidatorFixture(
    validatorSetID: number,
    validatorSetLength: number
): ValidatorFixture {
    let wallets: Wallet[] = []
    for (let i = 0; i < validatorSetLength; i++) {
        let wallet = ethers.Wallet.createRandom()
        wallets.push(wallet)
    }

    let walletsByLeaf = wallets.reduce((accum: WalletsByLeaf, wallet) => {
        let leaf = "0x" + keccakFromHexString(wallet.address).toString("hex")
        accum[leaf] = wallet
        return accum
    }, {})

    wallets = wallets.sort((a, b) => {
        if (a.address < b.address) {
            return -1
        } else if (a.address > b.address) {
            return 1
        } else {
            return 0
        }
    })

    let validatorAddresses = wallets.map((wallet) => {
        return wallet.address
    })

    let validatorAddressesHashed = validatorAddresses.map((address) => {
        return "0x" + keccakFromHexString(address).toString("hex")
    })

    let validatorMerkleTree = createMerkleTree(validatorAddresses)
    let validatorAddressProofs = validatorAddresses.map((address, index) => {
        return validatorMerkleTree.getHexProof(address, index)
    })

    return {
        wallets,
        walletsByLeaf,
        validatorAddresses,
        validatorAddressesHashed,
        validatorSetID,
        validatorSetRoot: validatorMerkleTree.getHexRoot(),
        validatorSetLength,
        validatorAddressProofs,
        validatorMerkleTree,
    }
}

async function createRandomPositions(numberOfPositions: number, numberOfValidators: number) {
    let positions: number[] = []
    for (let i = 0; i < numberOfValidators; i++) {
        positions.push(i)
    }

    let shuffled = _.shuffle(positions)

    return shuffled.slice(0, numberOfPositions)
}

interface ValidatorSignature {
    v: number,
    r: Uint8Array,
    s: Uint8Array,
}

interface ValidatorProof {
    signature: ValidatorSignature
    index: number
    addr: string
    merkleProof: string[]
}

interface ValidatorMultiProof {
    signatures: ValidatorSignature[]
    indices: number[]
    addrs: string[]
    merkleProofs: string[][]
}

function createInitialValidatorProofs(
    commitmentHash: string,
    validatorFixture: ValidatorFixture
): ValidatorProof[] {
    let commitmentHashBytes = ethers.utils.arrayify(commitmentHash)
    let tree = validatorFixture.validatorMerkleTree
    let leaves = tree.getHexLeaves()

    return leaves.map((leaf, position) => {
        let wallet = validatorFixture.walletsByLeaf[leaf]
        let address = wallet.address
        let proof = tree.getHexProof(leaf, position)
        let privateKey = ethers.utils.arrayify(wallet.privateKey)
        let signature = secp256k1.ecdsaSign(commitmentHashBytes, privateKey)
        let recid = signature.recid + 27

        let buf = new Uint8Array(signature.signature.buffer)
        let r = buf.slice(0, 32)
        let s = buf.slice(32)

        return {
            signature: {
                v: recid,
                r, s
            },
            index: position,
            addr: address,
            merkleProof: proof,
        }
    })
}

async function createFinalValidatorProofs(
    finalBitfield: ethers.BigNumber[],
    initialProofs: ValidatorProof[]
) {
    let bitfieldString = printBitfield(finalBitfield)

    let proofs: ValidatorMultiProof = {
        signatures: [],
        indices: [],
        addrs: [],
        merkleProofs: [],
    }

    let ascendingBitfield = bitfieldString.split("").reverse().join("")
    for (let position = 0; position < ascendingBitfield.length; position++) {
        let bit = ascendingBitfield[position]
        if (bit === "1") {
            proofs.signatures.push(initialProofs[position].signature)
            proofs.indices.push(initialProofs[position].index)
            proofs.addrs.push(initialProofs[position].addr)
            proofs.merkleProofs.push(initialProofs[position].merkleProof)
        }
    }

    return proofs
}

export {
    createMerkleTree,
    encodeLog,
    printBitfield,
    createValidatorFixture,
    createRandomPositions,
    createInitialValidatorProofs,
    createFinalValidatorProofs,
}
