import rlp from "rlp"

import { keccakFromHexString, keccak } from "ethereumjs-util"
import { MerkleTree } from "merkletreejs"
import { ethers, Event, Wallet } from "ethers"
import _ from "lodash"
import secp256k1 from "secp256k1"

let makeBasicCommitment = (messages) => {
    let encoded = ethers.utils.defaultAbiCoder.encode(
        ["tuple(address target, uint64 nonce, bytes payload)[]"],
        [messages]
    )
    return ethers.utils.solidityKeccak256(["bytes"], [encoded])
}

let makeIncentivizedCommitment = (messages) => {
    let encoded = ethers.utils.defaultAbiCoder.encode(
        ["tuple(address target, uint64 nonce, uint256 fee, bytes payload)[]"],
        [messages]
    )
    return ethers.utils.solidityKeccak256(["bytes"], [encoded])
}

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

let hexPrefix = /^(0x)/i

let mergeKeccak256 = (left, right) =>
    "0x" +
    keccakFromHexString(
        "0x" + left.replace(hexPrefix, "") + right.replace(hexPrefix, ""),
        256
    ).toString("hex")

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

async function createValidatorFixture(validatorSetID: number, validatorSetLength: number) {
    let wallets: Wallet[] = []
    for (let i = 0; i < validatorSetLength; i++) {
        let wallet = ethers.Wallet.createRandom()
        wallets.push(wallet)
    }
    let walletsByLeaf = wallets.reduce((accum, wallet) => {
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

async function createInitialValidatorProofs(commitmentHash: string, validatorFixture) {
    let commitmentHashBytes = ethers.utils.arrayify(commitmentHash)
    let tree = validatorFixture.validatorMerkleTree
    let leaves = tree.getHexLeaves()

    return leaves.map((leaf, position) => {
        let wallet = validatorFixture.walletsByLeaf[leaf]
        let address = wallet.address
        let proof = tree.getHexProof(leaf, position)
        let privateKey = ethers.utils.arrayify(wallet.privateKey)
        let signatureECDSA = secp256k1.ecdsaSign(commitmentHashBytes, privateKey)
        let ethRecID = signatureECDSA.recid + 27
        let signature = Uint8Array.from(
            signatureECDSA.signature
                .join()
                .split(",")
                .concat(ethRecID as any) as any
        )
        return { signature: ethers.utils.hexlify(signature), position, address, proof }
    })
}

interface Proofs {
    signatures: string[]
    indices: number[]
    addrs: string[]
    merkleProofs: string[]
}

async function createFinalValidatorProofs(finalBitfield: ethers.BigNumber[], initialProofs) {
    let bitfieldString = printBitfield(finalBitfield)

    let proofs: Proofs = {
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
            proofs.indices.push(initialProofs[position].position)
            proofs.addrs.push(initialProofs[position].address)
            proofs.merkleProofs.push(initialProofs[position].proof)
        }
    }

    return proofs
}

export {
    createMerkleTree,
    encodeLog,
    mergeKeccak256,
    printBitfield,
    createValidatorFixture,
    createRandomPositions,
    createInitialValidatorProofs,
    createFinalValidatorProofs,
}
