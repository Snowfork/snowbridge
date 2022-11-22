import { ethers, expect, loadFixture } from "../setup"
import { readSetBits, createRandomSubset } from "../helpers"

describe("Bitfield", function () {
    async function fixture() {
        let bitfieldFactory = await ethers.getContractFactory("$Bitfield");
        let bitfieldLib = await bitfieldFactory.deploy();
        return { bitfieldLib }
    }

    it("creates initial bitfield correctly in simple case", async function () {
        let { bitfieldLib } = await loadFixture(fixture)

        let positions = [0, 5, 8]
        let bitfield = await bitfieldLib.$createBitfield(positions, 9)

        expect(readSetBits(bitfield)).to.eql(positions)
    })

    it("creates initial bitfield correctly with bigger example", async function () {
        let { bitfieldLib } = await loadFixture(fixture)

        let positions = createRandomSubset(200, 140)
        let bitfield = await bitfieldLib.$createBitfield(positions, 200)

        expect(readSetBits(bitfield)).to.eql(positions)
    })
})
