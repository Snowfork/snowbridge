#!/usr/bin/env ts-node
import { ValidatorSet, createRandomSubset, readSetBits } from "./helpers"
import { BigNumber, ethers } from "ethers"
import fs from "fs"
import type { BeefyClient } from "../types/BeefyClient"
import { accounts } from "./wallets"
import path from "path"
const encoder = new ethers.utils.AbiCoder()

const run = async () => {
    const fixtureData = JSON.parse(
        fs.readFileSync(
            path.join(process.env.contract_dir!, "test/data/beefy-commitment.json"),
            "utf8"
        )
    )
    const TestFixtureFile = path.join(process.env.contract_dir!, "beefy-test-fixture.json")
    const command = process.argv[2]
    const validatorSetID = fixtureData.params.id
    const validatorSetSize =
        process.env["FixedSet"] == "true"
            ? accounts.length
            : process.env["ValidatorSetSize"]
            ? parseInt(process.env["ValidatorSetSize"])
            : 300
    const commitHash = fixtureData.commitmentHash
    if (command == "GenerateInitialSet") {
        const blockNumber = fixtureData.params.commitment.blockNumber
        const mmrLeafProofs = fixtureData.params.leafProof
        const mmrRoot = fixtureData.params.commitment.payload[0].data
        const mmrLeaf: BeefyClient.MMRLeafStruct = fixtureData.params.leaf
        const leafProofOrder = fixtureData.params.leafProofOrder

        const absentSubsetSize = Math.floor((validatorSetSize - 1) / 3)
        const subsetSize = validatorSetSize - absentSubsetSize
        const randomSet = createRandomSubset(validatorSetSize, subsetSize)
        const participants = randomSet.participants
        const absentees = randomSet.absentees

        const mmrLeafRaw = encoder.encode(
            [
                "tuple(uint8 version,uint32 parentNumber,bytes32 parentHash,uint64 nextAuthoritySetID,uint32 nextAuthoritySetLen,bytes32 nextAuthoritySetRoot,bytes32 parachainHeadsRoot)",
            ],
            [mmrLeaf]
        )

        const testFixture = {
            blockNumber,
            validatorSetID,
            validatorSetSize,
            participants,
            absentees,
            commitHash,
            mmrRoot,
            mmrLeaf,
            mmrLeafProofs,
            leafProofOrder,
            mmrLeafRaw,
        }
        fs.writeFileSync(TestFixtureFile, JSON.stringify(testFixture, null, 2), "utf8")
        console.log("Beefy fixture writing to dest file: " + TestFixtureFile)
    } else if (command == "GenerateProofs") {
        const prevRandao = process.env["PREV_RANDAO"] ? parseInt(process.env["PREV_RANDAO"]): 377;
        const testFixture = JSON.parse(fs.readFileSync(TestFixtureFile, "utf8"))
        let validatorSet: ValidatorSet
        if (process.env["FixedSet"] == "true") {
            validatorSet = new ValidatorSet(
                validatorSetID,
                validatorSetSize,
                accounts.map((account) => account.privateKey)
            )
        } else {
            validatorSet = new ValidatorSet(validatorSetID, validatorSetSize)
        }
        let finalBitfield: BigNumber[] = []
        for (let i = 0; i < testFixture.finalBitField.length; i++) {
            finalBitfield.push(ethers.BigNumber.from(testFixture.finalBitField[i]))
        }
        const finalValidatorsProof: BeefyClient.ValidatorProofStruct[] = readSetBits(
            finalBitfield
        ).map((i) => validatorSet.createSignatureProof(i, commitHash))
        console.log("Final Validator proofs:", finalValidatorsProof)
        const finalValidatorsProofRaw = encoder.encode(
            [
                "tuple(uint8 v, bytes32 r, bytes32 s, uint256 index,address account,bytes32[] proof)[]",
            ],
            [finalValidatorsProof]
        )
        testFixture.prevRandao = prevRandao
        testFixture.finalValidatorsProof = finalValidatorsProof
        testFixture.finalValidatorsProofRaw = finalValidatorsProofRaw
        testFixture.validatorRoot = validatorSet.root
        fs.writeFileSync(TestFixtureFile, JSON.stringify(testFixture, null, 2), "utf8")
        console.log("Beefy fixture writing to dest file: " + TestFixtureFile)
    }
}

run()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
