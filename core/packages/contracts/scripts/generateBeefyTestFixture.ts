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
    const BeefyValidatorSetFile = path.join(
        process.env.contract_dir!,
        "test/data/beefy-validator-set.json"
    )
    const BeefyValidatorProofFile = path.join(
        process.env.contract_dir!,
        "test/data/beefy-validator-proof.json"
    )
    const command = process.argv[2]
    const validatorSetID = fixtureData.params.id
    const validatorSetSize =
        process.env["FixedSet"] == "true"
            ? accounts.length
            : process.env["ValidatorSetSize"]
            ? parseInt(process.env["ValidatorSetSize"])
            : 300
    const commitHash = fixtureData.commitmentHash
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

    if (command == "GenerateInitialSet") {
        const mmrLeaf: BeefyClient.MMRLeafStruct = fixtureData.params.leaf

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
            validatorSetSize,
            participants,
            absentees,
            validatorRoot: validatorSet.root,
            mmrLeafRaw,
        }
        fs.writeFileSync(BeefyValidatorSetFile, JSON.stringify(testFixture, null, 2), "utf8")
        console.log("Beefy fixture writing to dest file: " + BeefyValidatorSetFile)
    } else if (command == "GenerateProofs") {
        const testFixture = JSON.parse(fs.readFileSync(BeefyValidatorProofFile, "utf8"))
        const bitField = encoder.decode(["uint256[]"], testFixture.finalBitFieldRaw)[0]
        console.log(bitField)
        let finalBitfield: BigNumber[] = []
        for (let i = 0; i < bitField.length; i++) {
            finalBitfield.push(bitField[i])
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
        testFixture.finalValidatorsProof = finalValidatorsProof
        testFixture.finalValidatorsProofRaw = finalValidatorsProofRaw
        fs.writeFileSync(BeefyValidatorProofFile, JSON.stringify(testFixture, null, 2), "utf8")
        console.log("Beefy fixture writing to dest file: " + BeefyValidatorProofFile)
    }
}

run()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
