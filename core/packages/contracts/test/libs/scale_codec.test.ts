import { ethers, expect, loadFixture } from "../setup"

describe("ScaleCodec", function () {
    async function fixture() {
        let codecFactory = await ethers.getContractFactory("$ScaleCodec")
        let codecLib = await codecFactory.deploy()
        return { codecLib }
    }

    describe("encoding unsigned integers", function () {
        it("should encode uint256", async function () {
            let { codecLib } = await loadFixture(fixture)
            let output = await codecLib.$encodeU256(
                "12063978950259949786323707366460749298097791896371638493358994162204017315152"
            )
            expect(output).to.be.equal(
                "0x504d8a21dd3868465c8c9f2898b7f014036935fa9a1488629b109d3d59f8ab1a"
            )
        })

        it("should encode uint128", async function () {
            let { codecLib } = await loadFixture(fixture)
            let output = await codecLib.$encodeU128("35452847761173902980759433963665451267")
            expect(output).to.be.equal("0x036935fa9a1488629b109d3d59f8ab1a")
        })

        it("should encode uint64", async function () {
            let { codecLib } = await loadFixture(fixture)
            let output = await codecLib.$encodeU64("1921902728173129883")
            expect(output).to.be.equal("0x9b109d3d59f8ab1a")
        })

        it("should encode uint32", async function () {
            let { codecLib } = await loadFixture(fixture)
            let output = await codecLib.$encodeU32("447477849")
            expect(output).to.be.equal("0x59f8ab1a")
        })

        it("should encode uint16", async function () {
            let { codecLib } = await loadFixture(fixture)
            let output = await codecLib.$encodeU16("6827")
            expect(output).to.be.equal("0xab1a")
        })
    })
})
