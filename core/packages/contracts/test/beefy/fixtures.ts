import { ethers, loadFixture } from "../setup"
import {
    ScaleCodec__factory,
    MMRProofVerification__factory,
    MerkleProof__factory,
    Bitfield__factory,
    BeefyClientPublic__factory
} from "@src"

import { createValidatorFixture } from "../helpers"

import fixtureData from "./data/beefy-commitment.json"

export { baseFixture, beefyClientFixture }

async function libsFixture() {
    let [owner] = await ethers.getSigners()

    let codec = await new ScaleCodec__factory(owner).deploy()
    let mmrProof = await new MMRProofVerification__factory(owner).deploy()
    let merkleProof = await new MerkleProof__factory(owner).deploy()
    let bitfield = await new Bitfield__factory(owner).deploy()

    return { codec, mmrProof, merkleProof, bitfield }
}

/**
 * beefy client base fixture with some internal methods made public
 */
async function baseFixture() {
    let [owner, user] = await ethers.getSigners()
    let { codec, mmrProof, merkleProof, bitfield } = await libsFixture()
    let beefyClient = await new BeefyClientPublic__factory(
        {
            "contracts/ScaleCodec.sol:ScaleCodec": codec.address,
            "contracts/utils/MMRProofVerification.sol:MMRProofVerification": mmrProof.address,
            "contracts/utils/MerkleProof.sol:MerkleProof": merkleProof.address,
            "contracts/utils/Bitfield.sol:Bitfield": bitfield.address
        },
        owner
    ).deploy()

    return { beefyClient, owner, user }
}

const totalNumberOfValidators = 300

/**
 * beefy client fixture initialized with a current validator set
 * that is 1 session older than the validator set that signed the candidate BEEFY commitment
 */
async function beefyClientFixture() {
    let { beefyClient, owner, user } = await loadFixture(baseFixture)

    let validators = createValidatorFixture(
        fixtureData.params.commitment.validatorSetID - 1,
        totalNumberOfValidators
    )

    await beefyClient.initialize(
        0,
        {
            id: validators.validatorSetID,
            root: validators.validatorSetRoot,
            length: validators.validatorSetLength
        },
        {
            id: validators.validatorSetID + 1,
            root: validators.validatorSetRoot,
            length: validators.validatorSetLength
        }
    )

    return { beefyClient, fixtureData, validators, owner, user }
}
