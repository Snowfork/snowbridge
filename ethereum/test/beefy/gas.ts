import { ethers, expect, loadFixture, mine } from "../setup"
import { baseFixture } from "./fixtures"
import {
    createRandomPositions,
    createValidatorFixture,
    createInitialValidatorProofs,
    createFinalValidatorProofs,
} from "../helpers"

import fixtureData from "../data/beefy-relay-basic.json"

let SUBMIT_FINAL_2 =
    "submitFinal(uint256,(uint32,uint64,(bytes32,bytes,bytes)),(bytes[],uint256[],address[],bytes32[][]),(uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32),(bytes32[],uint64))"

let runFlow = async function (totalNumberOfValidators, totalNumberOfSignatures) {
    let { beefyClient, user } = await loadFixture(async function foo() {
        return baseFixture()
    })

    let validators = await createValidatorFixture(
        fixtureData.params.commitment.validatorSetID - 1,
        totalNumberOfValidators
    )

    await beefyClient.initialize(
        0,
        {
            id: validators.validatorSetID,
            root: validators.validatorSetRoot,
            length: validators.validatorSetLength,
        },
        {
            id: validators.validatorSetID + 1,
            root: validators.validatorSetRoot,
            length: validators.validatorSetLength,
        }
    )

    let initialBitfieldPositions = await createRandomPositions(
        totalNumberOfValidators,
        totalNumberOfValidators
    )

    let firstPosition = initialBitfieldPositions[0]

    let initialBitfield = await beefyClient.createInitialBitfield(
        initialBitfieldPositions,
        totalNumberOfValidators
    )

    let commitmentHash = fixtureData.commitmentHash

    let initialValidatorProofs = await createInitialValidatorProofs(commitmentHash, validators)

    await beefyClient
        .connect(user)
        .submitInitial(
            commitmentHash,
            fixtureData.params.commitment.validatorSetID,
            initialBitfield,
            {
                signature: initialValidatorProofs[firstPosition].signature,
                index: firstPosition,
                addr: initialValidatorProofs[firstPosition].address,
                merkleProof: initialValidatorProofs[firstPosition].proof,
            }
        )

    let lastId = (await beefyClient.nextRequestID()).sub(ethers.BigNumber.from(1))

    await mine(45)

    let finalBitfield = await beefyClient.createFinalBitfield(lastId)
    let completeValidatorProofs = await createFinalValidatorProofs(
        finalBitfield,
        initialValidatorProofs
    )

    await expect(
        beefyClient
            .connect(user)
            [SUBMIT_FINAL_2](
                lastId,
                fixtureData.params.commitment,
                completeValidatorProofs,
                fixtureData.params.leaf,
                fixtureData.params.leafProof
            )
    ).to.emit(beefyClient, "NewMMRRoot")
}

describe.skip("Beefy Client Gas Usage", function () {
    let testCases = [
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 10,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 20,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 30,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 40,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 50,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 60,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 70,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 80,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 90,
        },
        {
            totalNumberOfValidators: 600,
            totalNumberOfSignatures: 100,
        },
    ]

    for (let testCase of testCases) {
        it(`runs full flow with ${testCase.totalNumberOfValidators} validators and ${testCase.totalNumberOfSignatures} signers`, async function () {
            this.timeout(1000 * 65)
            await runFlow(testCase.totalNumberOfValidators, testCase.totalNumberOfSignatures)
        })
    }
})
