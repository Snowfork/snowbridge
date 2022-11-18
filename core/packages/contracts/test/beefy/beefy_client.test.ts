import { expect, loadFixture, mine } from "../setup"
import { beefyClientFixture } from "./fixtures"

import {
    createRandomPositions,
    createInitialValidatorProofs,
    createFinalValidatorProofs
} from "../helpers"

describe("BeefyClient", function () {
    it("runs commitment submission flow", async function () {
        let { beefyClient, fixtureData, validators, user } = await loadFixture(beefyClientFixture)

        // expecting 2/3+1 validator signatures
        let numSignatureClaims =
            validators.validatorSetLength - Math.floor((validators.validatorSetLength - 1) / 3)

        let initialBitfieldPositions = await createRandomPositions(
            numSignatureClaims,
            validators.validatorSetLength
        )
        let initialBitfield = await beefyClient.createInitialBitfield(
            initialBitfieldPositions,
            validators.validatorSetLength
        )

        let firstPosition = initialBitfieldPositions[0]

        let commitmentHash = fixtureData.commitmentHash

        let initialValidatorProofs = createInitialValidatorProofs(commitmentHash, validators)

        await expect(
            beefyClient
                .connect(user)
                .submitInitial(
                    commitmentHash,
                    fixtureData.params.commitment.validatorSetID,
                    initialBitfield,
                    {
                        signature: initialValidatorProofs[firstPosition].signature,
                        index: firstPosition,
                        addr: initialValidatorProofs[firstPosition].addr,
                        merkleProof: initialValidatorProofs[firstPosition].merkleProof
                    }
                )
        )
            .to.emit(beefyClient, "NewRequest")
            .withArgs(0, user.address)

        expect(await beefyClient.nextRequestID()).to.equal(1)

        await mine(45)

        let finalBitfield = await beefyClient.createFinalBitfield(0)
        let completeValidatorProofs = await createFinalValidatorProofs(
            finalBitfield,
            initialValidatorProofs
        )

        await expect(
            beefyClient
                .connect(user)
                .submitFinalWithLeaf(
                    0,
                    fixtureData.params.commitment,
                    completeValidatorProofs,
                    fixtureData.params.leaf,
                    fixtureData.params.leafProof
                )
        )
            .to.emit(beefyClient, "NewMMRRoot")
            .withArgs(
                fixtureData.params.commitment.payload.mmrRootHash,
                fixtureData.params.commitment.blockNumber
            )

        expect(await beefyClient.latestMMRRoot()).to.eq(
            fixtureData.params.commitment.payload.mmrRootHash
        )
        expect(await beefyClient.latestMMRRoot()).to.eq(
            fixtureData.params.commitment.payload.mmrRootHash
        )
    })

    it("encodes beefy commitment to SCALE-format", async function () {
        let { beefyClient } = await loadFixture(beefyClientFixture)
        let commitment = {
            blockNumber: 5,
            validatorSetID: 7,
            payload: {
                mmrRootHash: "0x3ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c",
                prefix: "0x0861620c0001026d6880",
                suffix: "0x"
            }
        }

        let encoded = await beefyClient.encodeCommitment_public(commitment)
        expect(encoded).to.eq(
            "0x0861620c0001026d68803ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c050000000700000000000000"
        )
    })
})
