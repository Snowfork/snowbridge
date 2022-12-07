import { ethers } from "../setup"
import { MerkleProof__factory, Bitfield__factory, BeefyClientMock__factory } from "../../src"

import { ValidatorSet } from "../helpers"

import fixtureData from "./data/beefy-commitment.json"

export { baseFixture, beefyClientFixture, beefyClientFixture2 }

async function libsFixture() {
    let [owner] = await ethers.getSigners()

    let merkleProof = await new MerkleProof__factory(owner).deploy()
    let bitfield = await new Bitfield__factory(owner).deploy()

    return { merkleProof, bitfield }
}

/**
 * beefy client base fixture with some internal methods made public
 */
async function baseFixture() {
    let [owner, user] = await ethers.getSigners()
    let { merkleProof, bitfield } = await libsFixture()
    let beefyClient = await new BeefyClientMock__factory(
        {
            "contracts/utils/MerkleProof.sol:MerkleProof": merkleProof.address,
            "contracts/utils/Bitfield.sol:Bitfield": bitfield.address
        },
        owner
    ).deploy(3, 8)

    return { beefyClient, owner, user }
}

const totalNumberOfValidators = 300

/**
 * beefy client fixture initialized with a current validator set
 * that is 1 session older than the validator set that signed the candidate BEEFY commitment
 */
async function beefyClientFixture() {
    let { beefyClient, owner, user } = await baseFixture()

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

/**
 * beefy client fixture initialized with a current validator set
 * that is the same set that signed the candidate BEEFY commitment
 */
async function beefyClientFixture2() {
    let { beefyClient, owner, user } = await baseFixture()

    let vset = new ValidatorSet(
        fixtureData.params.commitment.validatorSetID,
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
