const {
  deployBeefyLightClient,
  mine, printTxPromiseGas
} = require("./helpers");

const { createBeefyValidatorFixture, createRandomPositions,
  createAllValidatorProofs, createCompleteValidatorProofs } = require("./beefy-helpers");
const realWorldFixture = require('./fixtures/full-flow-basic.json');

require("chai")
  .use(require("chai-as-promised"))
  .should();

const { expect } = require("chai");

describe("Beefy Light Client Gas Usage", function () {

  const testCases = [
    {
      totalNumberOfValidators: 10,
      totalNumberOfSignatures: 10,
    },
    {
      totalNumberOfValidators: 200,
      totalNumberOfSignatures: 200,
    },
    {
      totalNumberOfValidators: 200,
      totalNumberOfSignatures: 134,
    },
    {
      totalNumberOfValidators: 255,
      totalNumberOfSignatures: 255,
    },
    {
      totalNumberOfValidators: 257,
      totalNumberOfSignatures: 257,
    },
    {
      totalNumberOfValidators: 1000,
      totalNumberOfSignatures: 1000,
      fail: true
    },
    {
      totalNumberOfValidators: 1000,
      totalNumberOfSignatures: 1000,
    },
    {
      totalNumberOfValidators: 1000,
      totalNumberOfSignatures: 667,
    },
  ]

  for (const testCase of testCases) {
    it(`runs full flow with ${testCase.totalNumberOfValidators} validators and ${testCase.totalNumberOfSignatures} signers with the complete transaction ${testCase.fail ? 'failing' : 'succeeding'}`,
      async function () {
        this.timeout(1000 * 65);
        await runFlow(testCase.totalNumberOfValidators, testCase.totalNumberOfSignatures, testCase.fail)
      });
  }

  const runFlow = async function (totalNumberOfValidators, totalNumberOfSignatures, fail) {
    console.log(`Running flow with ${totalNumberOfValidators} validators and ${totalNumberOfSignatures} signatures with the complete transaction ${fail ? 'failing' : 'succeeding'}: `)

    const fixture = await createBeefyValidatorFixture(
      totalNumberOfValidators
    )
    const beefyLightClient = await deployBeefyLightClient(fixture.root,
      totalNumberOfValidators);

    const initialBitfieldPositions = await createRandomPositions(totalNumberOfSignatures, totalNumberOfValidators)

    const firstPosition = initialBitfieldPositions[0]

    const initialBitfield = await beefyLightClient.createInitialBitfield(
      initialBitfieldPositions, totalNumberOfValidators
    );

    const commitmentHash = await beefyLightClient.createCommitmentHash(realWorldFixture.completeSubmitInput.commitment);

    const allValidatorProofs = await createAllValidatorProofs(commitmentHash, fixture);

    const newSigTxPromise = beefyLightClient.newSignatureCommitment(
      commitmentHash,
      initialBitfield,
      allValidatorProofs[firstPosition].signature,
      firstPosition,
      allValidatorProofs[firstPosition].address,
      allValidatorProofs[firstPosition].proof,
    )
    printTxPromiseGas(newSigTxPromise)
    await newSigTxPromise.should.be.fulfilled

    const lastId = (await beefyLightClient.currentId()).sub(new web3.utils.BN(1));

    await mine(45);

    const completeValidatorProofs = await createCompleteValidatorProofs(lastId, beefyLightClient, allValidatorProofs);

    const completeSigTxPromise = beefyLightClient.completeSignatureCommitment(
      fail ? 99 : lastId,
      realWorldFixture.completeSubmitInput.commitment,
      completeValidatorProofs,
      realWorldFixture.completeSubmitInput.latestMMRLeaf,
      realWorldFixture.completeSubmitInput.mmrLeafIndex,
      realWorldFixture.completeSubmitInput.mmrLeafCount,
      realWorldFixture.completeSubmitInput.mmrProofItems,
    )
    printTxPromiseGas(completeSigTxPromise)
    if (fail) {
      await completeSigTxPromise.should.be.rejected
    } else {
      await completeSigTxPromise.should.be.fulfilled
      latestMMRRoot = await beefyLightClient.latestMMRRoot()
      expect(latestMMRRoot).to.eq(realWorldFixture.completeSubmitInput.commitment.payload)
    }
  }

});
