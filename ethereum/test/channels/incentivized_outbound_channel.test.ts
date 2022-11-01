import { ethers, expect, loadFixture, deployMockContract } from "../setup"
import { IncentivizedOutboundChannel__factory, FeeController__factory } from "@src"

let testPayload = ethers.utils.formatBytes32String("arbitrary-payload")

describe("IncentivizedOutboundChannel", function () {
    async function fixture() {
        let [owner, app, user] = await ethers.getSigners()

        // mock fee controller
        let mockFeeController = await deployMockContract(owner as any, FeeController__factory.abi)
        await mockFeeController.mock.handleFee.returns()

        let channel = await new IncentivizedOutboundChannel__factory(owner).deploy()
        await channel.deployed()
        await channel.initialize(owner.address, mockFeeController.address, [app.address])
        await channel.setFee(10)

        return { channel, app, user, mockFeeController }
    }

    describe("send", function () {
        it("should send messages out with the correct event and fields", async function () {
            let { channel, app, user } = await loadFixture(fixture)

            await expect(channel.connect(app).submit(user.address, testPayload, 0))
                .to.emit(channel, "Message")
                .withArgs(app.address, 1, 10, testPayload)
        })

        it("should increment nonces correctly", async function () {
            let { channel, app, user } = await loadFixture(fixture)

            await expect(channel.connect(app).submit(user.address, testPayload, 0))
                .to.emit(channel, "Message")
                .withArgs(app.address, 1, 10, testPayload)

            await expect(channel.connect(app).submit(user.address, testPayload, 0))
                .to.emit(channel, "Message")
                .withArgs(app.address, 2, 10, testPayload)

            await expect(channel.connect(app).submit(user.address, testPayload, 0))
                .to.emit(channel, "Message")
                .withArgs(app.address, 3, 10, testPayload)
        })

        it("should not send message if user cannot pay fee", async function () {
            let { channel, app, user, mockFeeController } = await loadFixture(fixture)

            await mockFeeController.mock.handleFee.reverts()

            await expect(channel.connect(app).submit(user.address, testPayload, 0)).to.be.reverted
        })
    })
})
