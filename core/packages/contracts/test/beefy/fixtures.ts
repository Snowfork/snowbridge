import { ethers, loadFixture } from "../setup"
import {
    MMRProofVerification__factory,
    MerkleProof__factory,
    BeefyClientPublic__factory
} from "../../src"

import { ValidatorSet } from "../helpers"

import fixtureData from "./data/beefy-commitment.json"

export { baseFixture, beefyClientFixture }

async function libsFixture() {
    let [owner] = await ethers.getSigners()

    let mmrProof = await new MMRProofVerification__factory(owner).deploy()
    let merkleProof = await new MerkleProof__factory(owner).deploy()

    return { mmrProof, merkleProof }
}

/**
 * beefy client base fixture with some internal methods made public
 */
async function baseFixture() {
    let [owner, user] = await ethers.getSigners()
    let { mmrProof, merkleProof } = await libsFixture()
    let beefyClient = await new BeefyClientPublic__factory(
        {
            "contracts/utils/MMRProofVerification.sol:MMRProofVerification": mmrProof.address,
            "contracts/utils/MerkleProof.sol:MerkleProof": merkleProof.address,
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

    let vset = new ValidatorSet(
        fixtureData.params.commitment.validatorSetID - 1,
        totalNumberOfValidators
    )

    await beefyClient.initialize(
        0,
        {
            id: vset.id,
            root: vset.root,
            length: vset.length
        },
        {
            id: vset.id + 1,
            root: vset.root,
            length: vset.length
        }
    )

    return { beefyClient, fixtureData, vset, owner, user }
}
