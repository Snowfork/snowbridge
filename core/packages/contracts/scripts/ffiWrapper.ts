#!/usr/bin/env node
import { ValidatorSet, createRandomSubset, readSetBits } from "./helpers"
import { BigNumber, ethers } from "ethers"
import fs from "fs"
import type { BeefyClient } from "../types/BeefyClient"
import { accounts } from "./wallets"

const encoder = new ethers.utils.AbiCoder()
const command = process.argv[2]
const fixtureData = JSON.parse(fs.readFileSync("test/data/beefy-commitment.json", "utf8"))
const validatorSetID = fixtureData.params.id
const validatorSetSize = process.env["FixedSet"] == "true" ? accounts.length : 300
const commitHash = fixtureData.commitmentHash
const blockNumber = fixtureData.params.commitment.blockNumber
const mmrLeafProofs = fixtureData.params.leafProof
const mmrRoot = fixtureData.params.commitment.payload[0].data
const mmrLeaf: BeefyClient.MMRLeafStruct = fixtureData.params.leaf
const leafProofOrder = fixtureData.params.leafProofOrder

const absentSubsetSize = Math.floor((validatorSetSize - 1) / 3)
const subsetSize = validatorSetSize - absentSubsetSize
const subset = createRandomSubset(validatorSetSize, subsetSize)
const absentSubset = createRandomSubset(validatorSetSize, absentSubsetSize)
let validatorSet: ValidatorSet

if (command == "GenerateInitialSet") {
    process.stdout.write(
        `${encoder.encode(
            ["uint32", "uint32", "uint32", "uint256[]", "uint256[]", "bytes32", "bytes32"],
            [blockNumber, validatorSetID, validatorSetSize, subset, absentSubset, commitHash, mmrRoot]
        )}`
    )
} else if (command == "GenerateProofs") {
    if (process.env["FixedSet"] == "true") {
        validatorSet = new ValidatorSet(
            validatorSetID,
            validatorSetSize,
            accounts.map((account) => account.privateKey)
        )
    } else {
        validatorSet = new ValidatorSet(validatorSetID, validatorSetSize)
    }
    const finalBitfieldLength = parseInt(process.argv[3])
    let finalBitfield: BigNumber[] = []
    for (let i = 0; i < finalBitfieldLength; i++) {
        finalBitfield.push(ethers.BigNumber.from(process.argv[4 + i]))
    }
    const validatorFinalProofs: BeefyClient.ValidatorProofStruct[] = readSetBits(finalBitfield).map(
        (i) => validatorSet.createSignatureProof(i, commitHash)
    )
    process.stdout.write(
        `${encoder.encode(
            [
                "bytes32",
                "tuple(uint8 v, bytes32 r, bytes32 s, uint256 index,address account,bytes32[] proof)[]",
                "bytes32[]",
                "tuple(uint8 version,uint32 parentNumber,bytes32 parentHash,uint64 nextAuthoritySetID,uint32 nextAuthoritySetLen,bytes32 nextAuthoritySetRoot,bytes32 parachainHeadsRoot)",
                "uint256",
            ],
            [validatorSet.root, validatorFinalProofs, mmrLeafProofs, mmrLeaf, leafProofOrder]
        )}`
    )
}
