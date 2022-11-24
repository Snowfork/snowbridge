import { ethers, expect, loadFixture, mine, setPrevRandao } from "../setup"
import { ValidatorSet } from "../helpers"
import { beefyClientFixture, beefyClientFixture2 } from "./fixtures"
import { BeefyClientMock } from "../../src"

import { createRandomSubset, readSetBits } from "../helpers"

// create initial bitfield with signature claims (with 2/3+1 claimed validator signatures)
let createInitialBitfield = async (beefyClient: BeefyClientMock, vset: ValidatorSet) => {
    let claims = createRandomSubset(vset.length, vset.length - Math.floor((vset.length - 1) / 3))
    let bitfield = await beefyClient.createInitialBitfield(claims, vset.length)
    return { claims, bitfield }
}

describe("BeefyClient", function () {
    describe("submit signature proof", function () {
        it("should succeed", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            // wait RANDAO_COMMIT_DELAY number of blocks, commit to a PREVRANDAO, create a final bitfield
            let delay = await beefyClient.connect(user).randaoCommitDelay()
            await mine(delay)
            setPrevRandao(377)
            await expect(beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)).not
                .to.be.reverted
            let finalBitfield = await beefyClient
                .connect(user)
                .createFinalBitfield(fixtureData.commitmentHash, bitfield)

            // Submit final signature proof
            await expect(
                beefyClient.connect(user).submitFinal(
                    fixtureData.params.commitment,
                    bitfield,
                    readSetBits(finalBitfield).map((i) =>
                        vset.createSignatureProof(i, fixtureData.commitmentHash)
                    )
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

        it("should fail when commitment is stale", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            // wait RANDAO_COMMIT_DELAY number of blocks, commit to a PREVRANDAO, create a final bitfield
            let delay = await beefyClient.connect(user).randaoCommitDelay()
            await mine(delay)
            setPrevRandao(377)
            await expect(beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)).not
                .to.be.reverted
            let finalBitfield = await beefyClient
                .connect(user)
                .createFinalBitfield(fixtureData.commitmentHash, bitfield)

            // Submit final signature proof
            await expect(
                beefyClient.connect(user).submitFinal(
                    fixtureData.params.commitment,
                    bitfield,
                    readSetBits(finalBitfield).map((i) =>
                        vset.createSignatureProof(i, fixtureData.commitmentHash)
                    )
                )
            ).to.not.be.reverted

            // Submit again, should fail
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted
            await mine(delay)
            setPrevRandao(377)
            await expect(beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)).not
                .to.be.reverted
            await expect(
                beefyClient.connect(user).submitFinal(
                    fixtureData.params.commitment,
                    bitfield,
                    readSetBits(finalBitfield).map((i) =>
                        vset.createSignatureProof(i, fixtureData.commitmentHash)
                    )
                )
            ).to.be.revertedWithCustomError(beefyClient, "StaleCommitment")
        })

        it("should fail when submission concluded using submitFinalWithHandover", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            // wait RANDAO_COMMIT_DELAY number of blocks, commit to a PREVRANDAO, create a final bitfield
            let delay = await beefyClient.connect(user).randaoCommitDelay()
            await mine(delay)
            setPrevRandao(377)
            await expect(beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)).not
                .to.be.reverted
            let finalBitfield = await beefyClient
                .connect(user)
                .createFinalBitfield(fixtureData.commitmentHash, bitfield)

            // Submit final signature proof
            await expect(
                beefyClient.connect(user).submitFinalWithHandover(
                    fixtureData.params.commitment,
                    bitfield,
                    readSetBits(finalBitfield).map((i) =>
                        vset.createSignatureProof(i, fixtureData.commitmentHash)
                    ),
                    fixtureData.params.leaf,
                    fixtureData.params.leafProof,
                    fixtureData.params.leafProofOrder
                )
            ).to.be.revertedWithCustomError(beefyClient, "InvalidCommitment")
        })

        it("should fail when submission concluded using different bitfield", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            // wait RANDAO_COMMIT_DELAY number of blocks, commit to a PREVRANDAO, create a final bitfield
            let delay = await beefyClient.connect(user).randaoCommitDelay()
            await mine(delay)
            setPrevRandao(377)
            await expect(beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)).not
                .to.be.reverted
            let finalBitfield = await beefyClient
                .connect(user)
                .createFinalBitfield(fixtureData.commitmentHash, bitfield)

            // Submit final signature proof
            await expect(
                beefyClient.connect(user).submitFinal(
                    fixtureData.params.commitment,
                    [ethers.BigNumber.from(0)],
                    readSetBits(finalBitfield).map((i) =>
                        vset.createSignatureProof(i, fixtureData.commitmentHash)
                    )
                )
            ).to.be.revertedWithCustomError(beefyClient, "InvalidBitfield")
        })

        it("should fail when PREVRANDAO is not captured", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            // Submit final signature proof
            await expect(
                beefyClient.connect(user).submitFinal(fixtureData.params.commitment, bitfield, [])
            ).to.be.revertedWithCustomError(beefyClient, "PrevRandaoNotCaptured")
        })

        it("should fail when PREVRANDAO capture attempted more than once", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            let delay = await beefyClient.connect(user).randaoCommitDelay()
            await mine(delay)

            await expect(
                beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)
            ).to.not.be.reverted

            await expect(
                beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)
            ).to.be.revertedWithCustomError(beefyClient, "PrevRandaoAlreadyCaptured")
        })

        it("should fail when PREVRANDAO capture attempted too early", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            await expect(
                beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)
            ).to.be.revertedWithCustomError(beefyClient, "WaitPeriodNotOver")
        })

        it("should fail when PREVRANDAO capture window expired", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitial(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            let delay = await beefyClient.connect(user).randaoCommitDelay()
            let expiration = await beefyClient.connect(user).randaoCommitExpiration()
            await mine(delay.add(expiration).add(1))
            await expect(
                beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)
            ).to.be.revertedWithCustomError(beefyClient, "TaskExpired")
        })

        it("should fail when initial signature proof has invalid signature", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            let proof = vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
            proof.s[0] = 7

            // Submit initial signature proof
            await expect(
                beefyClient.connect(user).submitInitial(fixtureData.commitmentHash, bitfield, proof)
            ).to.be.revertedWithCustomError(beefyClient, "InvalidSignature")
        })

        it("should fail when initial signature proof has invalid validator proof (1)", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            let proof = vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
            proof.account = user.address

            // Submit initial signature proof
            await expect(
                beefyClient.connect(user).submitInitial(fixtureData.commitmentHash, bitfield, proof)
            ).to.be.revertedWithCustomError(beefyClient, "InvalidValidatorProof")
        })

        it("should fail when initial signature proof has invalid validator proof (2)", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture2)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            let proof = vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
            proof.proof.reverse()

            // Submit initial signature proof
            await expect(
                beefyClient.connect(user).submitInitial(fixtureData.commitmentHash, bitfield, proof)
            ).to.be.revertedWithCustomError(beefyClient, "InvalidValidatorProof")
        })
    })

    describe("submit signature proof with handover", function () {
        it("should succeed", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitialWithHandover(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
                    )
            ).not.to.be.reverted

            // wait RANDAO_COMMIT_DELAY number of blocks, commit to a PREVRANDAO, and then create
            // a final bitfield
            let delay = await beefyClient.connect(user).randaoCommitDelay()
            await mine(delay)
            await expect(beefyClient.connect(user).commitPrevRandao(fixtureData.commitmentHash)).not
                .to.be.reverted
            let finalBitfield = await beefyClient
                .connect(user)
                .createFinalBitfield(fixtureData.commitmentHash, bitfield)

            // Submit final signature proof
            await expect(
                beefyClient.connect(user).submitFinalWithHandover(
                    fixtureData.params.commitment,
                    bitfield,
                    readSetBits(finalBitfield).map((i) =>
                        vset.createSignatureProof(i, fixtureData.commitmentHash)
                    ),
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

        it("should fail when PREVRANDAO not captured", async function () {
            let { beefyClient, fixtureData, vset, user } = await loadFixture(beefyClientFixture)

            let { claims, bitfield } = await createInitialBitfield(beefyClient, vset)

            // Submit initial signature proof
            await expect(
                beefyClient
                    .connect(user)
                    .submitInitialWithHandover(
                        fixtureData.commitmentHash,
                        bitfield,
                        vset.createSignatureProof(claims[0], fixtureData.commitmentHash)
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
