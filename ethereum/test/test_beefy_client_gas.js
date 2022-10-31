const { ethers, contract } = require("hardhat");
const { expect } = require("chai");
const { loadFixture, mine } = require("@nomicfoundation/hardhat-network-helpers");
const { baseFixture } = require("./fixtures");

const {
  createRandomPositions, createValidatorFixture,
  createInitialValidatorProofs, createFinalValidatorProofs
} = require("./helpers");

const fixtureData = require('./fixtures/beefy-relay-basic.json');

let SUBMIT_FINAL_2 = "submitFinal(uint256,(uint32,uint64,(bytes32,bytes,bytes)),(bytes[],uint256[],address[],bytes32[][]),(uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32),(bytes32[],uint64))";

const runFlow = async function (totalNumberOfValidators, totalNumberOfSignatures) {
  let { beefyClient, user } = await loadFixture(async function foo() { return baseFixture("BeefyClient") } );

  const validators = await createValidatorFixture(fixtureData.params.commitment.validatorSetID-1, totalNumberOfValidators)

  await beefyClient.initialize(0,
    { id: validators.validatorSetID, root: validators.validatorSetRoot, length: validators.validatorSetLength },
    { id: validators.validatorSetID + 1, root: validators.validatorSetRoot, length: validators.validatorSetLength }
  );

  const initialBitfieldPositions = await createRandomPositions(totalNumberOfValidators, totalNumberOfValidators)

  const firstPosition = initialBitfieldPositions[0]

  const initialBitfield = await beefyClient.createInitialBitfield(
    initialBitfieldPositions, totalNumberOfValidators
  );

  const commitmentHash = fixtureData.commitmentHash

  const initialValidatorProofs = await createInitialValidatorProofs(commitmentHash, validators);


  await beefyClient.connect(user).submitInitial(
    commitmentHash,
    fixtureData.params.commitment.validatorSetID,
    initialBitfield,
    {
      signature: initialValidatorProofs[firstPosition].signature,
      index: firstPosition,
      addr: initialValidatorProofs[firstPosition].address,
      merkleProof: initialValidatorProofs[firstPosition].proof
    }
  );

  const lastId = (await beefyClient.nextRequestID()).sub(ethers.BigNumber.from(1));

  await mine(45);

  const completeValidatorProofs = await createFinalValidatorProofs(lastId, beefyClient, initialValidatorProofs);

  await expect(beefyClient.connect(user)[SUBMIT_FINAL_2](
    lastId,
    fixtureData.params.commitment,
    completeValidatorProofs,
    fixtureData.params.leaf,
    fixtureData.params.leafProof
  )).to.emit(beefyClient, "NewMMRRoot")
}


describe.skip("Beefy Client Gas Usage", function () {

  const testCases = [
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 10,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 20,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 30,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 40,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 50,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 60,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 70,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 80,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 90,
    },
    {
      totalNumberOfValidators: 600,
      totalNumberOfSignatures: 100,
    },
  ]

  for (const testCase of testCases) {
    it(`runs full flow with ${testCase.totalNumberOfValidators} validators and ${testCase.totalNumberOfSignatures} signers`,
      async function () {
        this.timeout(1000 * 65);
        await runFlow(testCase.totalNumberOfValidators, testCase.totalNumberOfSignatures)
      });
  }



});
