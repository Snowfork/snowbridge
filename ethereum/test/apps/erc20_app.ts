import { ethers, expect, loadFixture } from "../setup"
import { erc20AppFixture } from "./fixtures"

let POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

describe("ERC20App", function () {
    describe("deposits", function () {
        it("should lock funds", async function () {
            let { app, token, owner, user, channelID } = await loadFixture(erc20AppFixture)

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
            let { app, token, user, channelID } = await loadFixture(erc20AppFixture)

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
            let { app, token, user, channelID } = await loadFixture(erc20AppFixture)
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
