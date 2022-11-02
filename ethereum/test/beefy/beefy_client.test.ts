import { expect, loadFixture, mine } from "../setup"
import { beefyClientFixture1, beefyClientPublicFixture } from "./fixtures"

describe("BeefyClient", function () {
    it("runs commitment submission flow", async function () {
        let { beefyClient, fixtureData, user } = await loadFixture(beefyClientFixture1)

        let bitfield = await beefyClient.createInitialBitfield(fixtureData.params.proof.indices, 3)

        await expect(
            beefyClient
                .connect(user)
                .submitInitial(
                    fixtureData.commitmentHash,
                    fixtureData.params.commitment.validatorSetID,
                    bitfield,
                    {
                        signature: fixtureData.params.proof.signatures[0],
                        index: fixtureData.params.proof.indices[0],
                        addr: fixtureData.params.proof.addrs[0],
                        merkleProof: fixtureData.params.proof.merkleProofs[0],
                    }
                )
        )
            .to.emit(beefyClient, "NewRequest")
            .withArgs(0, user.address)

        await mine(3)

        await expect(
            beefyClient
                .connect(user)
                [
                    "submitFinal(uint256,(uint32,uint64,(bytes32,bytes,bytes)),(bytes[],uint256[],address[],bytes32[][]),(uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32),(bytes32[],uint64))"
                ](
                    0,
                    fixtureData.params.commitment,
                    fixtureData.params.proof,
                    fixtureData.params.leaf,
                    fixtureData.params.leafProof
                )
        )
            .to.emit(beefyClient, "NewMMRRoot")
            .withArgs(
                fixtureData.params.commitment.payload.mmrRootHash,
                fixtureData.params.commitment.blockNumber
            )

        let root = await beefyClient.latestMMRRoot()
        expect(root).to.eq(fixtureData.params.commitment.payload.mmrRootHash)
    })
    it("encodes beefy commitment to SCALE-format", async function () {
        let { beefyClient } = await loadFixture(beefyClientPublicFixture)
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
