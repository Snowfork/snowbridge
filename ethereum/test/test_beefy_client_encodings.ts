import { expect, loadFixture } from "./setup"
import { exposedBeefyClientFixture } from "./fixtures/beefy"

describe("BeefyClient", function () {
    it("encodes beefy commitment to SCALE-format", async function () {
        let { beefyClient } = await loadFixture(exposedBeefyClientFixture)
        let commitment = {
            blockNumber: 5,
            validatorSetID: 7,
            payload: {
                mmrRootHash: "0x3ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c",
                prefix: "0x0861620c0001026d6880",
                suffix: "0x",
            },
        }

        let encoded = await beefyClient.encodeCommitmentExposed(commitment)
        expect(encoded).to.eq(
            "0x0861620c0001026d68803ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c050000000700000000000000"
        )
    })
})
