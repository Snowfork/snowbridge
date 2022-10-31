import {} from "../src/hardhat"
import "@nomiclabs/hardhat-ethers"
import { ethers } from "hardhat"
import { expect } from "chai"
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers"

describe("BasicOutboundChannel", function () {
    let testPayload = ethers.utils.formatBytes32String("arbitrary-payload")

    async function fixture() {
        let [owner, app, user] = await ethers.getSigners()

        let BasicOutboundChannel = await ethers.getContractFactory("BasicOutboundChannel")
        let channel = await BasicOutboundChannel.deploy()
        await channel.deployed()

        await channel.initialize(owner.address, [app.address])

        return { channel, owner, app, user }
    }

    describe("send", function () {
        it("should send messages out with the correct event and fields", async function () {
            let { channel, app, user } = await loadFixture(fixture)

            await expect(channel.connect(app).submit(user.address, testPayload, 0))
                .to.emit(channel, "Message")
                .withArgs(app.address, user.address, 1, testPayload)
        })

        it("should increment nonces correctly", async function () {
            let { channel, app, user } = await loadFixture(fixture)

            await expect(channel.connect(app).submit(user.address, testPayload, 0))
                .to.emit(channel, "Message")
                .withArgs(app.address, user.address, 1, testPayload)

            await expect(channel.connect(app).submit(user.address, testPayload, 0))
                .to.emit(channel, "Message")
                .withArgs(app.address, user.address, 2, testPayload)

            await expect(channel.connect(app).submit(user.address, testPayload, 0))
                .to.emit(channel, "Message")
                .withArgs(app.address, user.address, 3, testPayload)
        })
    })
})
