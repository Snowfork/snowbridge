import {} from "../src/hardhat"
import "@nomiclabs/hardhat-ethers"
import { ethers } from "hardhat"
import { expect } from "chai"
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers"

let POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

describe("ERC20App", function () {
    async function baseFixture() {
        let [owner, user] = await ethers.getSigners()

        let ScaleCodec = await ethers.getContractFactory("ScaleCodec")
        let codec = await ScaleCodec.deploy()

        let MockOutboundChannel = await ethers.getContractFactory("MockOutboundChannel")
        let outboundChannel = await MockOutboundChannel.deploy()

        let Registry = await ethers.getContractFactory("ChannelRegistry")
        let registry = await Registry.deploy()

        await Promise.all([codec.deployed(), outboundChannel.deployed(), registry.deployed()])

        // Add mock inbound and outbound channels to registry
        await registry.updateChannel(0, owner.address, outboundChannel.address)

        let ERC20App = await ethers.getContractFactory("ERC20App", {
            signer: owner,
            libraries: {
                ScaleCodec: codec.address,
            },
        })

        let app = await ERC20App.deploy(registry.address)
        await app.deployed()

        let Token = await ethers.getContractFactory("TestToken")
        let token = await Token.deploy("Test Token", "TEST")
        await token.deployed()

        await token.mint(user.address, 100)
        await token.connect(user).approve(app.address, 100)

        return {
            app,
            token,
            owner,
            user,
            channelID: 0,
        }
    }

    describe("deposits", function () {
        async function depositsFixture() {
            return baseFixture()
        }

        it("should lock funds", async function () {
            let { app, token, owner, user, channelID } = await loadFixture(depositsFixture)

            let amount = ethers.BigNumber.from(10)
            let beforeVaultBalance = await app.balances(token.address)
            let beforeUserBalance = await token.balanceOf(user.address)

            await expect(
                app.connect(user).lock(token.address, POLKADOT_ADDRESS, amount, 0, 0, channelID)
            )
                .to.emit(app, "Locked")
                .withArgs(token.address, user.address, POLKADOT_ADDRESS, amount, 0, 0)

            let afterVaultBalance = await app.balances(token.address)
            let afterUserBalance = await token.balanceOf(user.address)

            expect(afterVaultBalance).to.be.equal(beforeVaultBalance.add(10))
            expect(afterUserBalance).to.be.equal(beforeUserBalance.sub(10))
        })

        it("should lock funds and forward to destination parachain", async function () {
            let { app, token, user, channelID } = await loadFixture(depositsFixture)

            let amount = ethers.BigNumber.from(10)
            let beforeVaultBalance = await app.balances(token.address)
            let beforeUserBalance = await token.balanceOf(user.address)

            await token.connect(user).approve(app.address, amount.mul(2))

            await expect(
                app.connect(user).lock(token.address, POLKADOT_ADDRESS, amount, 2048, 0, channelID)
            )
                .to.emit(app, "Locked")
                .withArgs(token.address, user.address, POLKADOT_ADDRESS, amount, 2048, 0)

            let afterVaultBalance = await app.balances(token.address)
            let afterUserBalance = await token.balanceOf(user.address)

            expect(afterVaultBalance).to.be.equal(beforeVaultBalance.add(10))
            expect(afterUserBalance).to.be.equal(beforeUserBalance.sub(10))
        })
    })

    describe("withdrawals", function () {
        async function withdrawalsFixture() {
            let { app, token, user, channelID } = await baseFixture()
            await expect(
                app.connect(user).lock(token.address, POLKADOT_ADDRESS, 10, 0, 0, channelID)
            ).to.emit(app, "Locked")
            return { app, token, user }
        }

        it("should unlock funds", async function () {
            let { app, token, user } = await loadFixture(withdrawalsFixture)

            let amount = ethers.BigNumber.from(10)

            await expect(app.unlock(token.address, POLKADOT_ADDRESS, user.address, amount)).to.emit(
                app,
                "Unlocked"
            )
        })
    })
})
