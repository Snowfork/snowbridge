import { expect, loadFixture, mine, setPrevRandao } from "../setup"
import { beefyClientFixture, beefyClientFixture2 } from "./fixtures"

import { createRandomSubset, readSetBits } from "../helpers"

describe("BeefyClient", function () {
    it("submit signature proof", async function () {
        let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

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
                    bitfield,
                    vset.createSignatureProof(claims[0], commitmentHash)
                )
        ).not.to.be.reverted

        // wait RANDAO_COMMIT_DELAY number of blocks, commit to a PREVRANDAO, create a final bitfield
        let delay = await beefyClient.connect(user).RANDAO_COMMIT_DELAY()
        await mine(delay)
        setPrevRandao(377)
        await expect(beefyClient.connect(user).commitPrevRandao(commitmentHash)).not.to.be.reverted
        let finalBitfield = await beefyClient
            .connect(user)
            .createFinalBitfield(commitmentHash, bitfield)

        // Submit final signature proof
        await expect(
            beefyClient.connect(user).submitFinal(
                fixtureData.params.commitment,
                bitfield,
                readSetBits(finalBitfield).map((i) => vset.createSignatureProof(i, commitmentHash))
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

    it("submit signature proof should fail if PREVRANDAO not captured", async function () {
        let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

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
                    bitfield,
                    vset.createSignatureProof(claims[0], commitmentHash)
                )
        ).not.to.be.reverted

        // Submit final signature proof
        await expect(
            beefyClient.connect(user).submitFinal(fixtureData.params.commitment, bitfield, [])
        ).to.be.revertedWithCustomError(beefyClient, "PrevRandaoNotCaptured")
    })

    it("submit signature proof with MMR leaf", async function () {
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
                .submitInitialWithHandover(
                    commitmentHash,
                    bitfield,
                    vset.createSignatureProof(claims[0], commitmentHash)
                )
        ).not.to.be.reverted

        // wait RANDAO_COMMIT_DELAY number of blocks, commit to a PREVRANDAO, and then create
        // a final bitfield
        let delay = await beefyClient.connect(user).RANDAO_COMMIT_DELAY()
        await mine(delay)
        await expect(beefyClient.connect(user).commitPrevRandao(commitmentHash)).not.to.be.reverted
        let finalBitfield = await beefyClient
            .connect(user)
            .createFinalBitfield(commitmentHash, bitfield)

        // Submit final signature proof
        await expect(
            beefyClient.connect(user).submitFinalWithHandover(
                fixtureData.params.commitment,
                bitfield,
                readSetBits(finalBitfield).map((i) => vset.createSignatureProof(i, commitmentHash)),
                fixtureData.params.leaf,
                fixtureData.params.leafProof,
                fixtureData.params.leafProofOrder
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

    it("submit signature proof with MMR leaf should fail if PREVRANDAO not captured", async function () {
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
                .submitInitialWithHandover(
                    commitmentHash,
                    bitfield,
                    vset.createSignatureProof(claims[0], commitmentHash)
                )
        ).not.to.be.reverted

        // Submit final signature proof
        await expect(
            beefyClient
                .connect(user)
                .submitFinalWithHandover(
                    fixtureData.params.commitment,
                    bitfield,
                    [],
                    fixtureData.params.leaf,
                    fixtureData.params.leafProof,
                    fixtureData.params.leafProofOrder
                )
        ).to.be.revertedWithCustomError(beefyClient, "PrevRandaoNotCaptured")
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
