import {} from "../src/hardhat"
import "@nomiclabs/hardhat-ethers"
import { ethers } from "hardhat"
import { expect } from "chai"
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers"

let DOT_DECIMALS = 10
let ETHER_DECIMALS = 18
let GRANULARITY = Math.pow(10, ETHER_DECIMALS - DOT_DECIMALS)

// Convert native DOT to wrapped DOT
let wrapped = (amount) => {
    return {
        native: amount,
        wrapped: amount.mul(GRANULARITY),
    }
}

let POLKADOT_ACCOUNT = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

describe("DOTApp", function () {
    async function baseFixture() {
        let [owner, user] = await ethers.getSigners()

        let ScaleCodec = await ethers.getContractFactory("ScaleCodec")
        let codec = await ScaleCodec.deploy()

        let WrappedToken = await ethers.getContractFactory("WrappedToken")
        let token = await WrappedToken.deploy("Wrapped DOT", "WDOT")

        let MockOutboundChannel = await ethers.getContractFactory("MockOutboundChannel")
        let outboundChannel = await MockOutboundChannel.deploy()

        let Registry = await ethers.getContractFactory("ChannelRegistry")
        let registry = await Registry.deploy()

        await Promise.all([
            codec.deployed(),
            token.deployed(),
            outboundChannel.deployed(),
            registry.deployed(),
        ])

        // Add mock inbound and outbound channels to registry
        await registry.updateChannel(0, owner.address, outboundChannel.address)

        let DOTApp = await ethers.getContractFactory("DOTApp", {
            signer: owner,
            libraries: {
                ScaleCodec: codec.address,
            },
        })

        let app = await DOTApp.deploy(token.address, outboundChannel.address, registry.address)
        await app.deployed()

        await token.transferOwnership(app.address)

        return {
            app,
            token,
            owner,
            user,
            channelID: 0,
        }
    }

    describe("minting", function () {
        async function mintingFixture() {
            return baseFixture()
        }

        it("should mint funds", async function () {
            let { app, token, user } = await loadFixture(mintingFixture)

            let beforeTotalSupply = await token.totalSupply()
            let beforeUserBalance = await token.balanceOf(user.address)
            let amount = wrapped(ethers.BigNumber.from("10000000000")) // 1 DOT

            await expect(app.mint(POLKADOT_ACCOUNT, user.address, amount.wrapped))
                .to.emit(token, "Transfer")
                .withArgs(ethers.constants.AddressZero, user.address, amount.wrapped)

            let afterTotalSupply = await token.totalSupply()
            let afterUserBalance = await token.balanceOf(user.address)

            expect(afterTotalSupply.sub(beforeTotalSupply)).to.be.equal(amount.wrapped)
            expect(afterUserBalance.sub(beforeUserBalance)).to.be.equal(amount.wrapped)
        })

        it("should reject mint messages from unauthorised accounts", async function () {
            let { app, user } = await loadFixture(mintingFixture)
            await expect(
                app.connect(user).mint(POLKADOT_ACCOUNT, user.address, ethers.BigNumber.from(10))
            ).to.be.revertedWithCustomError(app, "Unauthorized")
        })
    })

    describe("burning", function () {
        async function burningFixture() {
            let { app, token, owner, user, channelID } = await baseFixture()
            let amount = wrapped(ethers.BigNumber.from("20000000000")) // 2 DOT
            await app.mint(POLKADOT_ACCOUNT, user.address, amount.wrapped)
            return { app, token, owner, user, channelID }
        }

        it("should burn funds", async function () {
            let { app, token, user, channelID } = await loadFixture(burningFixture)

            let beforeTotalSupply = await token.totalSupply()
            let beforeUserBalance = await token.balanceOf(user.address)
            let amount = wrapped(ethers.BigNumber.from("10000000000")) // 1 DOT

            await expect(app.connect(user).burn(POLKADOT_ACCOUNT, amount.wrapped, channelID))
                .to.emit(token, "Transfer")
                .withArgs(user.address, ethers.constants.AddressZero, amount.wrapped)

            let afterTotalSupply = await token.totalSupply()
            let afterUserBalance = await token.balanceOf(user.address)

            expect(beforeTotalSupply.sub(afterTotalSupply)).to.be.equal(amount.wrapped)
            expect(beforeUserBalance.sub(afterUserBalance)).to.be.equal(amount.wrapped)
        })

        it("should revert on unknown outbound channel", async function () {
            let { app, user } = await loadFixture(burningFixture)
            await expect(
                app.connect(user).burn(POLKADOT_ACCOUNT, ethers.BigNumber.from(10), 77)
            ).to.be.revertedWithCustomError(app, "UnknownChannel")
        })
    })
})
