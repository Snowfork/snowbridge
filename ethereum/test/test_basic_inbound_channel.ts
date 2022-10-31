import {} from "../src/hardhat"
import "@nomiclabs/hardhat-ethers"
import { ethers } from "hardhat"
import { expect } from "chai"
import { loadFixture } from "@nomicfoundation/hardhat-network-helpers"
import { anyValue } from "@nomicfoundation/hardhat-chai-matchers/withArgs"

import { deployMockContract } from "@ethereum-waffle/mock-contract"

import submitInput from "./fixtures/parachain-relay-basic.json"

describe("BasicInboundChannel", function () {
    async function fixture() {
        let [owner, user] = await ethers.getSigners()

        let MerkleProof = await ethers.getContractFactory("MerkleProof")
        let merkleProof = await MerkleProof.deploy()
        await merkleProof.deployed()

        // mock parachain client
        let iface = new ethers.utils.Interface([
            "function verifyCommitment(bytes32 commitment, bytes calldata opaqueProof) returns (bool)",
        ])
        let abi = JSON.parse(iface.format(ethers.utils.FormatTypes.json) as any)
        let mockParachainClient = await deployMockContract(owner as any, abi)

        // Make verifyCommitment() return true
        await mockParachainClient.mock.verifyCommitment.returns(true)

        let BasicInboundChannel = await ethers.getContractFactory("BasicInboundChannel", {
            libraries: {
                MerkleProof: merkleProof.address,
            },
        })
        let channel = await BasicInboundChannel.deploy(0, mockParachainClient.address)
        await channel.deployed()

        return { channel, user }
    }

    describe("submit", function () {
        it("should accept a valid commitment and dispatch messages", async function () {
            let { channel } = await loadFixture(fixture)

            let nonceBeforeSubmit = await channel.nonce(submitInput.params.bundle.account)

            await expect(
                channel.submit(
                    submitInput.params.bundle,
                    submitInput.params.leafProof,
                    submitInput.params.hashSides,
                    submitInput.params.proof
                )
            )
                .to.emit(channel, "MessageDispatched")
                .withArgs(ethers.BigNumber.from(0), anyValue)

            let nonceAfterSubmit = await channel.nonce(submitInput.params.bundle.account)
            expect(nonceAfterSubmit.sub(nonceBeforeSubmit)).to.be.equal(1)
        })

        it("should refuse to replay commitments", async function () {
            let { channel } = await loadFixture(fixture)

            // Submit messages
            await channel.submit(
                submitInput.params.bundle,
                submitInput.params.leafProof,
                submitInput.params.hashSides,
                submitInput.params.proof
            )

            // Submit messages again - should revert
            await expect(
                channel.submit(
                    submitInput.params.bundle,
                    submitInput.params.leafProof,
                    submitInput.params.hashSides,
                    submitInput.params.proof
                )
            ).to.be.reverted
        })
    })
})
