import { defaultAbiCoder } from "@ethersproject/abi"
import { expect, loadFixture } from "../setup"
import { xcmAppFixture } from "./fixtures"
import { XcmFungibleAsset__factory } from "../../src"

let PARA_2001_ORIGIN = "0x2001000000000000000000000000000000000000000000000000000000000000"
let PARA_2002_ORIGIN = "0x2002000000000000000000000000000000000000000000000000000000000000"

describe("XCMApp", function () {
    describe("proxies", function () {
        it("xcm transact presents msg.sender as the proxy account", async function () {
            let { app, executor, assetManager, downstream, user } = await loadFixture(xcmAppFixture)
            let proxy = "0xe1d2a389cd3e9694D374507E00C49d643605a2fb"
            let abi = defaultAbiCoder

            let encodedFunc = downstream.interface.encodeFunctionData("recordMsgSender")

            // Xcm Transact
            let transact = abi.encode(
                ["tuple(address, bytes)"],
                [[downstream.address, encodedFunc]]
            )

            let instructions = [{ kind: 0, arguments: transact }]

            let expectedEncodedCall = executor.interface.encodeFunctionData("execute", [
                assetManager.address,
                instructions,
            ])

            let payload = abi.encode(["tuple(uint8 kind,bytes arguments)[]"], [instructions])

            // HACK: This fixes the encoding.
            payload = payload.substring(0, 64) + "4" + payload.substring(65)

            await expect(
                app.dispatchToProxy(PARA_2001_ORIGIN, executor.address, payload, {
                    gasLimit: 1_000_000,
                })
            )
                .to.emit(app, "XcmExecuted")
                .withArgs(PARA_2001_ORIGIN, proxy, executor.address, true, expectedEncodedCall)
                .to.emit(downstream, "RecordSender")
                .withArgs(proxy)
        })
    })
    describe("substrate native assets", function () {
        it("the owning proxy can mint new tokens", async function () {
            let { app, executor, assetManager, owner } = await loadFixture(xcmAppFixture)
            let abi = defaultAbiCoder

            let proxy = "0xe1d2a389cd3e9694D374507E00C49d643605a2fb"
            let assetHash = "0x0001000000000000000000000000000000000000000000000000000000000000"

            let mintAmount = 1000

            let reserveAssetDeposited = abi.encode(
                ["tuple(bytes32, uint256)"],
                [[assetHash, mintAmount]]
            )

            let instructions = [{ kind: 1, arguments: reserveAssetDeposited }]

            let expectedEncodedCall = executor.interface.encodeFunctionData("execute", [
                assetManager.address,
                instructions,
            ])

            let payload = abi.encode(["tuple(uint8 kind,bytes arguments)[]"], [instructions])

            // HACK: This fixes the encoding.
            payload = payload.substring(0, 64) + "4" + payload.substring(65)

            await expect(
                app.dispatchToProxy(PARA_2001_ORIGIN, executor.address, payload, {
                    gasLimit: 30_000_000,
                })
            )
                .to.emit(app, "XcmExecuted")
                .withArgs(PARA_2001_ORIGIN, proxy, executor.address, true, expectedEncodedCall)

            let assetTokenAddr = await assetManager.lookup(assetHash)
            let asset = new XcmFungibleAsset__factory(owner).attach(assetTokenAddr)
            expect(await asset.totalSupply()).to.be.equal(mintAmount)
            expect(await asset.balanceOf(proxy)).to.be.equal(mintAmount)
        })
        it("non owning proxy can not mint new tokens", async function () {
            let { app, executor, assetManager, owner } = await loadFixture(xcmAppFixture)
            let abi = defaultAbiCoder

            let owningProxy = "0xe1d2a389cd3e9694D374507E00C49d643605a2fb"
            let nonOwningProxy = "0x3f3137e0885A7347E47018071a86ddee4c6Ad7F0"
            let assetHash = "0x0001000000000000000000000000000000000000000000000000000000000000"

            let mintAmount = 1000

            let reserveAssetDeposited = abi.encode(
                ["tuple(bytes32, uint256)"],
                [[assetHash, mintAmount]]
            )

            let instructions = [{ kind: 1, arguments: reserveAssetDeposited }]

            let expectedEncodedCall = executor.interface.encodeFunctionData("execute", [
                assetManager.address,
                instructions,
            ])

            let payload = abi.encode(["tuple(uint8 kind,bytes arguments)[]"], [instructions])

            // HACK: This fixes the encoding.
            payload = payload.substring(0, 64) + "4" + payload.substring(65)

            await expect(
                app.dispatchToProxy(PARA_2001_ORIGIN, executor.address, payload, {
                    gasLimit: 30_000_000,
                })
            )
                .to.emit(app, "XcmExecuted")
                .withArgs(PARA_2001_ORIGIN, owningProxy, executor.address, true, expectedEncodedCall)

            let assetTokenAddr = await assetManager.lookup(assetHash)
            let asset = new XcmFungibleAsset__factory(owner).attach(assetTokenAddr)
            expect(await asset.totalSupply()).to.be.equal(mintAmount)

            await expect(
                app.dispatchToProxy(PARA_2002_ORIGIN, executor.address, payload, {
                    gasLimit: 30_000_000,
                })
            )
                .to.emit(app, "XcmExecuted")
                .withArgs(PARA_2002_ORIGIN, nonOwningProxy, executor.address, false, expectedEncodedCall)

            expect(await asset.totalSupply()).to.be.equal(mintAmount)
            expect(await asset.balanceOf(owningProxy)).to.be.equal(mintAmount)
        })
        it("minted tokens can be transfered", async function () {
            let { app, executor, assetManager, owner, user } = await loadFixture(xcmAppFixture)
            let abi = defaultAbiCoder

            let owningProxy = "0xe1d2a389cd3e9694D374507E00C49d643605a2fb"
            let assetHash = "0x0001000000000000000000000000000000000000000000000000000000000000"

            let mintAmount = 1000
            let depositAmount = 500

            let reserveAssetDeposited = abi.encode(
                ["tuple(bytes32, uint256)"],
                [[assetHash, mintAmount]]
            )
            
            let depositAsset = abi.encode(
                ["tuple(bytes32,uint256,address)"],
                [[assetHash, depositAmount, user.address]]
            )

            let instructions = [{ kind: 1, arguments: reserveAssetDeposited },
            {kind: 2, arguments: depositAsset}]

            let expectedEncodedCall = executor.interface.encodeFunctionData("execute", [
                assetManager.address,
                instructions,
            ])

            let payload = abi.encode(["tuple(uint8 kind,bytes arguments)[]"], [instructions])

            // HACK: This fixes the encoding.
            payload = payload.substring(0, 64) + "4" + payload.substring(65)

            await expect(
                app.dispatchToProxy(PARA_2001_ORIGIN, executor.address, payload, {
                    gasLimit: 30_000_000,
                })
            )
                .to.emit(app, "XcmExecuted")
                .withArgs(PARA_2001_ORIGIN, owningProxy, executor.address, true, expectedEncodedCall)

            let assetTokenAddr = await assetManager.lookup(assetHash)
            let asset = new XcmFungibleAsset__factory(owner).attach(assetTokenAddr)
            expect(await asset.totalSupply()).to.be.equal(mintAmount)

            expect(await asset.balanceOf(owningProxy)).to.be.equal(mintAmount - depositAmount);
            expect(await asset.balanceOf(user.address)).to.be.equal(depositAmount);
        })
    })
})
