import { ethers, expect, loadFixture } from "../setup"
import { ethAppFixture } from "./fixtures"

let POLKADOT_ACCOUNT = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

describe("ETHApp", function () {
    describe("deposits", function () {
        it("should lock funds", async function () {
            let { app, vault, user, channelID } = await loadFixture(ethAppFixture)

            let beforeBalance = await ethers.provider.getBalance(vault.address)
            let amount = ethers.utils.parseEther("0.25")

            await expect(
                app.connect(user).lock(POLKADOT_ACCOUNT, 0, 0, channelID, {
                    value: amount,
                })
            )
                .to.emit(app, "Locked")
                .withArgs(user.address, POLKADOT_ACCOUNT, amount, 0, 0)
                .to.emit(vault, "Deposit")
                .withArgs(app.address, user.address, amount)

            // Confirm contract's balance has increased
            let afterBalance = await ethers.provider.getBalance(vault.address)
            expect(afterBalance).to.equal(beforeBalance.add(amount))
        })

        it("should lock funds and forward to destination parachain", async function () {
            let { app, vault, user, channelID } = await loadFixture(ethAppFixture)

            let beforeBalance = await ethers.provider.getBalance(vault.address)
            let amount = ethers.utils.parseEther("0.25")

            await expect(
                app.connect(user).lock(POLKADOT_ACCOUNT, 2048, 0, channelID, {
                    value: amount,
                })
            )
                .to.emit(app, "Locked")
                .withArgs(user.address, POLKADOT_ACCOUNT, amount, 2048, 0)
                .to.emit(vault, "Deposit")
                .withArgs(app.address, user.address, amount)

            // Confirm contract's balance has increased
            let afterBalance = await ethers.provider.getBalance(vault.address)
            expect(afterBalance).to.equal(beforeBalance.add(amount))
        })

        it("should not lock funds if amount is zero", async function () {
            let { app, user, channelID } = await loadFixture(ethAppFixture)
            await expect(
                app.connect(user).lock(POLKADOT_ACCOUNT, 0, 0, channelID, {
                    value: 0,
                })
            ).to.be.revertedWithCustomError(app, "MinimumAmount")
        })

        it("should not lock funds if amount is greater than 128-bits", async function () {
            let { app, user, channelID } = await loadFixture(ethAppFixture)
            await expect(
                app.connect(user).lock(POLKADOT_ACCOUNT, 0, 0, channelID, {
                    value: ethers.BigNumber.from("340282366920938463463374607431768211457"),
                })
            ).to.be.revertedWithCustomError(app, "MaximumAmount")
        })
    })

    describe("withdrawals", function () {
        async function withdrawalsFixture() {
            let { app, vault, owner, user, channelID } = await loadFixture(ethAppFixture)
            await app.connect(user).lock(POLKADOT_ACCOUNT, 0, 0, channelID, {
                value: ethers.utils.parseEther("2"),
            })
            return { app, vault, owner, user }
        }

        it("should unlock", async function () {
            let { app, vault, user } = await loadFixture(withdrawalsFixture)

            let amount = ethers.utils.parseEther("1")
            let beforeBalance = await ethers.provider.getBalance(vault.address)
            let beforeRecipientBalance = await ethers.provider.getBalance(user.address)

            await expect(app.unlock(POLKADOT_ACCOUNT, user.address, amount))
                .to.emit(app, "Unlocked")
                .withArgs(POLKADOT_ACCOUNT, user.address, amount)
                .to.emit(vault, "Withdraw")
                .withArgs(app.address, user.address, amount)

            let afterBalance = await ethers.provider.getBalance(vault.address)
            let afterRecipientBalance = await ethers.provider.getBalance(user.address)

            expect(afterBalance).to.be.equal(beforeBalance.sub(amount))
            expect(afterRecipientBalance.sub(beforeRecipientBalance)).to.be.equal(amount)
        })

        it("should not unlock amounts greater than locked balance", async function () {
            let { app, vault, user } = await loadFixture(withdrawalsFixture)

            let unlockAmount = ethers.utils.parseEther("2").add(1)

            await expect(
                app.unlock(POLKADOT_ACCOUNT, user.address, unlockAmount)
            ).to.be.revertedWithCustomError(vault, "InsufficientBalance")
        })
    })
})
