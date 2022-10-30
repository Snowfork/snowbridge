const { ethers } = require("hardhat");
const { expect } = require("chai");

const fixtureData = require('./fixtures/beefy-relay-basic.json');

module.exports = {
  exposedBeefyClientFixture,
  beefyClientFixture1,
  beefyClientFixture2,
}

async function baseFixture(contractName) {
  let [owner, user] = await ethers.getSigners();

  let ScaleCodec = await ethers.getContractFactory("ScaleCodec");
  let codec = await ScaleCodec.deploy();

  let MMRProofVerification = await ethers.getContractFactory("MMRProofVerification");
  let mmrProof = await MMRProofVerification.deploy();

  let MerkleProof = await ethers.getContractFactory("MerkleProof");
  let merkleProof = await MerkleProof.deploy();

  let Bitfield = await ethers.getContractFactory("Bitfield");
  let bitfield = await Bitfield.deploy();

  await Promise.all([
    codec.deployed(),
    mmrProof.deployed(),
    merkleProof.deployed(),
    bitfield.deployed()
  ]);

  let BeefyClient = await ethers.getContractFactory(contractName, {
    libraries: {
      ScaleCodec: codec.address,
      MMRProofVerification: mmrProof.address,
      MerkleProof: merkleProof.address,
      Bitfield: bitfield.address
    }
  });
  let beefyClient = await BeefyClient.deploy();
  await beefyClient.deployed();

  return { beefyClient, owner, user};
}

/**
 * beefy client fixture with some internal methods exposed
 */
async function exposedBeefyClientFixture() {
  return await baseFixture("ExposedBeefyClient");
}

/**
 * beefy client fixture initialized with a current validator set
 * that is 1 session older than the validator set that signed the candidate BEEFY commitment
 */
async function beefyClientFixture1() {
  let { beefyClient, owner, user } = await baseFixture("BeefyClient");

  let validatorSetID = fixtureData.params.commitment.validatorSetID-1;
  let validatorSetRoot = fixtureData.params.leaf.nextAuthoritySetRoot;
  let validatorSetLength = fixtureData.params.leaf.nextAuthoritySetLen;

  await beefyClient.initialize(0,
    { id: validatorSetID, root: validatorSetRoot, length: validatorSetLength },
    { id: validatorSetID + 1, root: validatorSetRoot, length: validatorSetLength }
  );

  return { beefyClient, fixtureData, owner, user };
}

/**
 * beefy client fixture initialized with a current validator set
 * that is the same set that signed the candidate BEEFY commitment
 */
async function beefyClientFixture2() {
  let { beefyClient, owner, user } = await baseFixture("BeefyClient");

  let validatorSetID = fixtureData.params.commitment.validatorSetID;
  let validatorSetRoot = fixtureData.params.leaf.nextAuthoritySetRoot;
  let validatorSetLength = fixtureData.params.leaf.nextAuthoritySetLen;

  await beefyClient.initialize(0,
    { id: validatorSetID, root: validatorSetRoot, length: validatorSetLength },
    { id: validatorSetID + 1, root: validatorSetRoot, length: validatorSetLength }
  );

  return { beefyClient, fixtureData, owner, user };
}
