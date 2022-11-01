import { ethers, loadFixture } from "../setup"
import {
    ScaleCodec__factory,
    MMRProofVerification__factory,
    MerkleProof__factory,
    Bitfield__factory,
    BeefyClient__factory,
    BeefyClientPublic__factory,
} from "@src"

import fixtureData from "../data/beefy-relay-basic.json"

export { baseFixture, beefyClientPublicFixture, beefyClientFixture1, beefyClientFixture2 }

async function libsFixture() {
    let [owner] = await ethers.getSigners()

    let codec = await new ScaleCodec__factory(owner).deploy()
    let mmrProof = await new MMRProofVerification__factory(owner).deploy()
    let merkleProof = await new MerkleProof__factory(owner).deploy()
    let bitfield = await new Bitfield__factory(owner).deploy()

    return { codec, mmrProof, merkleProof, bitfield }
}

async function baseFixture() {
    let [owner, user] = await ethers.getSigners()
    let { codec, mmrProof, merkleProof, bitfield } = await libsFixture()
    let beefyClient = await new BeefyClient__factory(
        {
            "contracts/ScaleCodec.sol:ScaleCodec": codec.address,
            "contracts/utils/MMRProofVerification.sol:MMRProofVerification": mmrProof.address,
            "contracts/utils/MerkleProof.sol:MerkleProof": merkleProof.address,
            "contracts/utils/Bitfield.sol:Bitfield": bitfield.address,
        },
        owner
    ).deploy()
    await beefyClient.deployed()

    return { beefyClient, owner, user }
}

/**
 * beefy client base fixture with some internal methods made public
 */
async function beefyClientPublicFixture() {
    let [owner] = await ethers.getSigners()
    let { codec, mmrProof, merkleProof, bitfield } = await libsFixture()
    let beefyClient = await new BeefyClientPublic__factory(
        {
            "contracts/ScaleCodec.sol:ScaleCodec": codec.address,
            "contracts/utils/MMRProofVerification.sol:MMRProofVerification": mmrProof.address,
            "contracts/utils/MerkleProof.sol:MerkleProof": merkleProof.address,
            "contracts/utils/Bitfield.sol:Bitfield": bitfield.address,
        },
        owner
    ).deploy()

    return { beefyClient }
}

/**
 * beefy client fixture initialized with a current validator set
 * that is 1 session older than the validator set that signed the candidate BEEFY commitment
 */
async function beefyClientFixture1() {
    let { beefyClient, owner, user } = await loadFixture(baseFixture)

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
    let { beefyClient, owner, user } = await loadFixture(baseFixture)

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
