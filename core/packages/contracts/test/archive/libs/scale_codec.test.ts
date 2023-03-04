import { ethers, expect, loadFixture } from "../setup"
import { ScaleCodecWrapper__factory } from "../../src"

describe("ScaleCodec", function () {
    async function fixture() {
        let [owner] = await ethers.getSigners()
        let codec = await new ScaleCodecWrapper__factory(owner).deploy()
        await codec.deployed()
        return { codec }
    }

    describe("encoding unsigned integers", function () {
        it("should encode uint256", async function () {
            let { codec } = await loadFixture(fixture)
            let output = await codec.encodeU256(
                "12063978950259949786323707366460749298097791896371638493358994162204017315152"
            )
            expect(output).to.be.equal(
                "0x504d8a21dd3868465c8c9f2898b7f014036935fa9a1488629b109d3d59f8ab1a"
            )
        })

        it("should encode uint128", async function () {
            let { codec } = await loadFixture(fixture)
            let output = await codec.encodeU128("35452847761173902980759433963665451267")
            expect(output).to.be.equal("0x036935fa9a1488629b109d3d59f8ab1a")
        })

        it("should encode uint64", async function () {
            let { codec } = await loadFixture(fixture)
            let output = await codec.encodeU64("1921902728173129883")
            expect(output).to.be.equal("0x9b109d3d59f8ab1a")
        })

        it("should encode uint32", async function () {
            let { codec } = await loadFixture(fixture)
            let output = await codec.encodeU32("447477849")
            expect(output).to.be.equal("0x59f8ab1a")
        })

        it("should encode uint16", async function () {
            let { codec } = await loadFixture(fixture)
            let output = await codec.encodeU16("6827")
            expect(output).to.be.equal("0xab1a")
        })
    })
})
