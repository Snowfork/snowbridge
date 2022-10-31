import {} from "../src/hardhat"
import "@nomiclabs/hardhat-ethers"
import { ethers } from "hardhat"

import fixtureData from "./fixtures/beefy-relay-basic.json"

export { baseFixture, exposedBeefyClientFixture, beefyClientFixture1, beefyClientFixture2 }

async function libsFixture() {
    let codecFactory = await ethers.getContractFactory("ScaleCodec")
    let codec = await codecFactory.deploy()

    let mmrProofFactory = await ethers.getContractFactory("MMRProofVerification")
    let mmrProof = await mmrProofFactory.deploy()

    let merkleProofFactory = await ethers.getContractFactory("MerkleProof")
    let merkleProof = await merkleProofFactory.deploy()

    let bitfieldFactory = await ethers.getContractFactory("Bitfield")
    let bitfield = await bitfieldFactory.deploy()

    await Promise.all([
        codec.deployed(),
        mmrProof.deployed(),
        merkleProof.deployed(),
        bitfield.deployed(),
    ])

    return { codec, mmrProof, merkleProof, bitfield }
}

async function baseFixture() {
    let [owner, user] = await ethers.getSigners()

    let { codec, mmrProof, merkleProof, bitfield } = await libsFixture()

    let BeefyClient = await ethers.getContractFactory("BeefyClient", {
        libraries: {
            ScaleCodec: codec.address,
            MMRProofVerification: mmrProof.address,
            MerkleProof: merkleProof.address,
            Bitfield: bitfield.address,
        },
    })
    let beefyClient = await BeefyClient.deploy()
    await beefyClient.deployed()

    return { beefyClient, owner, user }
}

/**
 * beefy client base fixture with some internal methods exposed
 */
async function exposedBeefyClientFixture() {
    let { codec, mmrProof, merkleProof, bitfield } = await libsFixture()

    let BeefyClient = await ethers.getContractFactory("ExposedBeefyClient", {
        libraries: {
            ScaleCodec: codec.address,
            MMRProofVerification: mmrProof.address,
            MerkleProof: merkleProof.address,
            Bitfield: bitfield.address,
        },
    })
    let beefyClient = await BeefyClient.deploy()
    await beefyClient.deployed()

    return { beefyClient }
}

/**
 * beefy client fixture initialized with a current validator set
 * that is 1 session older than the validator set that signed the candidate BEEFY commitment
 */
async function beefyClientFixture1() {
    let { beefyClient, owner, user } = await baseFixture()

    let validatorSetID = fixtureData.params.commitment.validatorSetID - 1
    let validatorSetRoot = fixtureData.params.leaf.nextAuthoritySetRoot
    let validatorSetLength = fixtureData.params.leaf.nextAuthoritySetLen

    await beefyClient.initialize(
        0,
        { id: validatorSetID, root: validatorSetRoot, length: validatorSetLength },
        { id: validatorSetID + 1, root: validatorSetRoot, length: validatorSetLength }
    )

    return { beefyClient, fixtureData, owner, user }
}

/**
 * beefy client fixture initialized with a current validator set
 * that is the same set that signed the candidate BEEFY commitment
 */
async function beefyClientFixture2() {
    let { beefyClient, owner, user } = await baseFixture()

    let validatorSetID = fixtureData.params.commitment.validatorSetID
    let validatorSetRoot = fixtureData.params.leaf.nextAuthoritySetRoot
    let validatorSetLength = fixtureData.params.leaf.nextAuthoritySetLen

    await beefyClient.initialize(
        0,
        { id: validatorSetID, root: validatorSetRoot, length: validatorSetLength },
        { id: validatorSetID + 1, root: validatorSetRoot, length: validatorSetLength }
    )

    return { beefyClient, fixtureData, owner, user }
}
