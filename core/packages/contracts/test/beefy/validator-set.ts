#!/usr/bin/env ts-node

import { ValidatorSet, createRandomSubset, readSetBits } from "../helpers"
import { ethers } from "ethers"
import type { BeefyClient } from "../../src/contracts/BeefyClient"

const encoder = new ethers.utils.AbiCoder()
const command = process.argv[2]
const setId = parseInt(process.argv[3])
const setSize = parseInt(process.argv[4])
const setIndex = parseInt(process.argv[5])
const commitHash = process.argv[6]
const subsetSize = setSize - Math.floor((setSize - 1) / 3)
const subset = createRandomSubset(setSize, subsetSize)
if (command == "RandomSubset") {
    process.stdout.write(`${encoder.encode(["uint256[]"], [subset])}`)
} else if (command == "FinalProof") {
    const validatorSet = new ValidatorSet(setId, setSize)
    const validatorProof = validatorSet.createSignatureProof(subset[setIndex], commitHash)
    const finalBitfieldLength = parseInt(process.argv[7])
    let finalBitfield: any = []
    for (let i = 0; i < finalBitfieldLength; i++) {
        finalBitfield.push(ethers.BigNumber.from(process.argv[8 + i]))
    }
    const validatorFinalProofs: BeefyClient.ValidatorProofStruct[] = readSetBits(finalBitfield).map(
        (i) => validatorSet.createSignatureProof(i, commitHash)
    )
    process.stdout.write(
        `${encoder.encode(
            [
                "bytes32",
                "tuple(uint8 v, bytes32 r, bytes32 s, uint256 index,address account,bytes32[] proof)",
                "tuple(uint8 v, bytes32 r, bytes32 s, uint256 index,address account,bytes32[] proof)[]",
            ],
            [validatorSet.root, validatorProof, validatorFinalProofs]
        )}`
    )
}
