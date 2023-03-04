import { ethers, expect, loadFixture } from "../setup"
import { MMRProofWrapper__factory } from "../../src"
import fixture7leaves from "./data/mmr-fixture-data-7-leaves.json"
import fixture15leaves from "./data/mmr-fixture-data-15-leaves.json"

describe("MMR Proof Verification", function () {
    async function fixture() {
        let [owner] = await ethers.getSigners()

        let verifier = await new MMRProofWrapper__factory(owner).deploy()
        await verifier.deployed()

        return { verifier }
    }

    describe("7-leaf, 11-node MMR", function () {
        before(function () {
            console.log("                 7-leaf MMR:           ")
            console.log("                                       ")
            console.log("    Height 3 |      7")
            console.log("    Height 2 |   3      6     10")
            console.log("    Height 1 | 1  2   4  5   8  9    11")
            console.log("             | |--|---|--|---|--|-----|-")
            console.log("Leaf indexes | 0  1   2  3   4  5     6")
        })

        fixture7leaves.proofs.forEach((proof, i) => {
            it(`should verify valid proof for leaf index ${i}`, async () => {
                let { verifier } = await loadFixture(fixture)

                expect(
                    await verifier.verifyLeafProof(
                        fixture7leaves.rootHash,
                        fixture7leaves.leaves[i],
                        fixture7leaves.proofs[i].items,
                        fixture7leaves.proofs[i].order
                    )
                ).to.be.true
            })

            it(`should reject invalid proof for leaf index ${i}`, async () => {
                let { verifier } = await loadFixture(fixture)

                let j = i + 1
                if (j >= fixture7leaves.proofs.length) {
                    j = 0
                }
                expect(
                    await verifier.verifyLeafProof(
                        fixture7leaves.rootHash,
                        fixture7leaves.leaves[i],
                        fixture7leaves.proofs[j].items,
                        fixture7leaves.proofs[j].order
                    )
                ).to.be.false
            })
        })
    })

    describe("15-leaf, 26-node MMR", function () {
        before(function () {
            console.log(
                "                                    15-leaf MMR:                            "
            )
            console.log(
                "                                                                            "
            )
            console.log(
                "    Height 4 |             15                                               "
            )
            console.log(
                "    Height 3 |      7             14                22                      "
            )
            console.log(
                "    Height 2 |   3      6     10      13       18        21       25        "
            )
            console.log(
                "    Height 1 | 1  2   4  5   8  9   11  12   16  17   19   20   23  24  26  "
            )
            console.log(
                "             | |--|---|--|---|--|-----|---|---|---|----|---|----|---|---|---"
            )
            console.log(
                "Leaf indexes | 0  1   2  3   4  5     6   7   8   9   10   11   12  13  14  "
            )
        })

        fixture15leaves.proofs.forEach((proof, i) => {
            it(`should verify valid proof for leaf index ${i}`, async () => {
                let { verifier } = await loadFixture(fixture)

                expect(
                    await verifier.verifyLeafProof(
                        fixture15leaves.rootHash,
                        fixture15leaves.leaves[i],
                        fixture15leaves.proofs[i].items,
                        fixture15leaves.proofs[i].order
                    )
                ).to.be.true
            })

            it(`should reject invalid proof for leaf index ${i}`, async () => {
                let { verifier } = await loadFixture(fixture)

                let j = i + 1
                if (j >= fixture15leaves.proofs.length) {
                    j = 0
                }
                expect(
                    await verifier.verifyLeafProof(
                        fixture15leaves.rootHash,
                        fixture15leaves.leaves[i],
                        fixture15leaves.proofs[j].items,
                        fixture15leaves.proofs[j].order
                    )
                ).to.be.false
            })
        })
    })

    it("ge report", async function () {
        let { verifier } = await loadFixture(fixture)
        let gasuse = await verifier.estimateGas.verifyLeafProof(
            fixture15leaves.rootHash,
            fixture15leaves.leaves[7],
            fixture15leaves.proofs[7].items,
            fixture15leaves.proofs[7].order
        )
        console.log(gasuse)
    })
})
