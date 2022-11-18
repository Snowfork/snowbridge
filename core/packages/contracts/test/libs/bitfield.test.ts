import { ethers, expect, loadFixture } from "../setup"
import { printBitfield, createRandomPositions } from "../helpers"
import { Bitfield__factory } from "../../src"

describe("Bitfield", function () {
    async function fixture() {
        let [owner] = await ethers.getSigners()
        let bitfieldLib = await new Bitfield__factory(owner).deploy()
        return { bitfieldLib }
    }

    it("creates initial bitfield correctly in simple case", async function () {
        let { bitfieldLib } = await loadFixture(fixture)

        let positions = [0, 5, 8]
        let expected = "100100001"
        let n = 9
        let bitfield = await bitfieldLib.createBitfield(positions, n)

        expect(printBitfield(bitfield)).to.equal(expected)
    })

    it("creates initial bitfield correctly with bigger example", async function () {
        let { bitfieldLib } = await loadFixture(fixture)

        let positions = await createRandomPositions(140, 200)
        let bitfield = await bitfieldLib.createBitfield(positions, 200)

        expect(printBitfield(bitfield)).to.equal(positionsToBitfield(positions))
    })
})

let positionsToBitfield = (positions: number[]) => {
    let bitfield: string[] = []
    for (let i = 0; i < positions.length; i++) {
        let position = positions[i]
        bitfield[position] = "1"
    }
    for (let i = 0; i < bitfield.length; i++) {
        if (bitfield[i] !== "1") {
            bitfield[i] = "0"
        }
    }
    return bitfield.reverse().join("")
}
