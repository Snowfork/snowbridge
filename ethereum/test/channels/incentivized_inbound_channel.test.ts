import { ethers, expect, loadFixture, deployMockContract, anyValue } from "../setup"
import {
    IncentivizedInboundChannel__factory,
    ParachainClient__factory,
    RewardController__factory,
} from "@src"
import submitInput from "../data/parachain-relay-incentivized.json"

describe("IncentivizedInboundChannel", function () {
    async function fixture() {
        let [owner, user] = await ethers.getSigners()

        // mock parachain client
        let mockParachainClient = await deployMockContract(
            owner as any,
            ParachainClient__factory.abi
        )
        await mockParachainClient.mock.verifyCommitment.returns(true)

        // mock reward source
        let mockRewardSource = await deployMockContract(owner as any, RewardController__factory.abi)
        await mockRewardSource.mock.handleReward.returns()

        let channel = await new IncentivizedInboundChannel__factory(owner).deploy(
            1,
            mockParachainClient.address
        )
        await channel.deployed()
        await channel.initialize(owner.address, mockRewardSource.address)

        return { channel, user }
    }

    describe("submit", function () {
        it("should accept a valid commitment and dispatch messages", async function () {
            let { channel } = await loadFixture(fixture)

            let nonceBeforeSubmit = await channel.nonce()

            await expect(channel.submit(submitInput.params.bundle, submitInput.params.proof))
                .to.emit(channel, "MessageDispatched")
                .withArgs(ethers.BigNumber.from(0), anyValue)

            let nonceAfterSubmit = await channel.nonce()
            expect(nonceAfterSubmit.sub(nonceBeforeSubmit)).to.be.equal(1)
        })

        it("should refuse to replay commitments", async function () {
            let { channel } = await loadFixture(fixture)

            // Submit messages
            await channel.submit(submitInput.params.bundle, submitInput.params.proof)

            // Submit messages again - should revert
            await expect(channel.submit(submitInput.params.bundle, submitInput.params.proof)).to.be
                .reverted
        })
    })
})
