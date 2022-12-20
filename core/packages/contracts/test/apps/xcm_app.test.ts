import { defaultAbiCoder } from "@ethersproject/abi"
import { expect, loadFixture } from "../setup"
import { xcmAppFixture } from "./fixtures"

let POLKADOT_ORIGIN = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

describe("XCMApp", function () {
    describe("Proxies", function () {
        it("downstream sees proxy as msg.sender", async function () {
            let { app, executor, downstream, user } = await loadFixture(xcmAppFixture)
            let proxy = "0x04f9fa5a18b8A2E6486e76F66B9482DeBF012155"
            let abi = defaultAbiCoder

            let encodedFunc = downstream.interface.encodeFunctionData("doSomethingInteresting")

            // Xcm Transact
            let transact = abi.encode(
                ["tuple(address, bytes)"],
                [[downstream.address, encodedFunc]]
            )
            let payload = executor.interface.encodeFunctionData("execute", [
                [{ kind: 0, arguments: transact }],
            ])

            await expect(
                app.dispatchToProxy(POLKADOT_ORIGIN, executor.address, payload, {
                    gasLimit: 1_000_000,
                })
            )
                .to.emit(app, "XcmExecuted")
                .withArgs(POLKADOT_ORIGIN, proxy, executor.address, true)
                .to.emit(downstream, "RecordSender")
                .withArgs(proxy)
        })
    })
})
