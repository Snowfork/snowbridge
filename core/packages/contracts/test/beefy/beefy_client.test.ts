import { expect, loadFixture, mine } from "../setup"
import { beefyClientFixture } from "./fixtures"

import { createRandomSubset, readSetBits } from "../helpers"

describe("BeefyClient", function () {
    it("runs commitment submission flow", async function () {
        let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture)

        let commitmentHash = fixtureData.commitmentHash

        // create initial bitfield with signature claims (with 2/3+1 claimed validator signatures)
        let claims = createRandomSubset(
            vset.length,
            vset.length - Math.floor((vset.length - 1) / 3)
        )
        let bitfield = await beefyClient.createInitialBitfield(claims, vset.length)

        // Submit initial signature proof
        await expect(
            beefyClient
                .connect(user)
                .submitInitial(
                    commitmentHash,
                    fixtureData.params.commitment.validatorSetID,
                    bitfield,
                    vset.createSignatureProof(claims[0], commitmentHash)
                )
        )
            .to.emit(beefyClient, "NewRequest")
            .withArgs(0, user.address)

        expect(await beefyClient.nextRequestID()).to.equal(1)

        // wait 3+ blocks and then create the final bitfield
        await mine(3)
        let finalBitfield = await beefyClient.createFinalBitfield(0)

        // Submit final signature proof
        await expect(
            beefyClient
                .connect(user)
                .submitFinalWithLeaf(
                    0,
                    fixtureData.params.commitment,
                    vset.createSignatureMultiProof(readSetBits(finalBitfield), commitmentHash),
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
        expect(await beefyClient.latestBeefyBlock()).to.eq(
            fixtureData.params.commitment.blockNumber
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
                suffix: "0x",
            },
        }

        let encoded = await beefyClient.encodeCommitment_public(commitment)
        expect(encoded).to.eq(
            "0x0861620c0001026d68803ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c050000000700000000000000"
        )
    })
})
