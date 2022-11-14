import { ethers, expect, loadFixture } from "../setup"
import { dotAppFixture } from "./fixtures"
import type { BigNumber } from "ethers"

let DOT_DECIMALS = 10
let ETHER_DECIMALS = 18
let GRANULARITY = Math.pow(10, ETHER_DECIMALS - DOT_DECIMALS)

// Convert native DOT to wrapped DOT
let wrapped = (amount: BigNumber) => {
    return {
        native: amount,
        wrapped: amount.mul(GRANULARITY)
    }
}

let POLKADOT_ACCOUNT = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

describe("DOTApp", function () {
    describe("minting", function () {
        it("should mint funds", async function () {
            let { app, token, user } = await loadFixture(dotAppFixture)

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
            let { app, user } = await loadFixture(dotAppFixture)
            await expect(
                app.connect(user).mint(POLKADOT_ACCOUNT, user.address, ethers.BigNumber.from(10))
            ).to.be.revertedWithCustomError(app, "Unauthorized")
        })
    })

    describe("wrapped token ownership", function () {
        it("should transfer ownership", async function () {
            let { app, token, user, owner } = await loadFixture(dotAppFixture)

            await expect(app.transferVaultOwnership(owner.address))
                .to.emit(token, "OwnershipTransferred")
                .withArgs(app.address, owner.address)

            let amount = wrapped(ethers.BigNumber.from("10000000000")) // 1 DOT
            await expect(app.mint(POLKADOT_ACCOUNT, user.address, amount.wrapped))
                .to.be.revertedWith("Ownable: caller is not the owner")
        })

        it("should not transfer ownership if unauthorized", async function () {
            let { app, user } = await loadFixture(dotAppFixture)

            await expect(app.connect(user).transferVaultOwnership(user.address))
                .to.be.revertedWithCustomError(app, "Unauthorized")
        })
    })

    describe("burning", function () {
        async function burningFixture() {
            let { app, token, owner, user, channelID } = await loadFixture(dotAppFixture)
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
