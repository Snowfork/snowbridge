import { ethers, expect, loadFixture, anyValue, deployMockContract } from "../setup"
import {
    MerkleProof__factory,
    BasicInboundChannel__factory,
    ParachainClient__factory,
} from "../../src"
import submitInput from "./data/basic-commitment.json"

describe("BasicInboundChannel", function () {
    async function fixture() {
        let [owner, user] = await ethers.getSigners()

        let merkleProof = await new MerkleProof__factory(owner).deploy()
        await merkleProof.deployed()

        // mock parachain client
        let mockParachainClient = await deployMockContract(owner, ParachainClient__factory.abi)
        await mockParachainClient.mock.verifyCommitment.returns(true)

        let channel = await new BasicInboundChannel__factory(
            {
                "contracts/utils/MerkleProof.sol:MerkleProof": merkleProof.address,
            },
            owner
        ).deploy(mockParachainClient.address)
        await channel.deployed()

        return { channel, user }
    }

    describe("submit", function () {
        it("should accept a valid commitment and dispatch messages", async function () {
            let { channel } = await loadFixture(fixture)

            let nonceBeforeSubmit = await channel.nonce(submitInput.message.sourceID)

            await channel.submit(
                submitInput.message,
                submitInput.leafProof,
                submitInput.hashSides,
                "0xdeadbeef"
            )

            let nonceAfterSubmit = await channel.nonce(submitInput.message.sourceID)
            expect(nonceAfterSubmit.sub(nonceBeforeSubmit)).to.be.equal(1)
        })

        it("should refuse to replay commitments", async function () {
            let { channel } = await loadFixture(fixture)

            // Submit messages
            await channel.submit(
                submitInput.message,
                submitInput.leafProof,
                submitInput.hashSides,
                "0xdeadbeef"
            )

            // Submit messages again - should revert
            await expect(
                channel.submit(
                    submitInput.message,
                    submitInput.leafProof,
                    submitInput.hashSides,
                    "0xdeadbeef"
                )
            ).to.be.reverted
        })
    })
})
