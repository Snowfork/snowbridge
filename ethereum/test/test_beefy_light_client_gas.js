const {
  deployBeefyLightClient,
  mine, printTxPromiseGas
} = require("./helpers");

const { createBeefyValidatorFixture, createRandomPositions,
  createAllValidatorProofs, createCompleteValidatorProofs } = require("./beefy-helpers");
const realWorldFixture = require('./fixtures/full-flow.json');

require("chai")
  .use(require("chai-as-promised"))
  .should();

const { expect } = require("chai");

describe("Beefy Light Client Gas Usage", function () {

  const testCases = [
    {
      totalNumberOfValidators: 10,
    },
    {
      totalNumberOfValidators: 200,
    },
    {
      totalNumberOfValidators: 255,
    },
    {
      totalNumberOfValidators: 257,
    },
    {
      totalNumberOfValidators: 1000,
      fail: true
    },
    {
      totalNumberOfValidators: 1000,
    },
  ]

  for (const testCase of testCases) {
    it(`runs full flow with ${testCase.totalNumberOfValidators} validators with the complete transaction ${testCase.fail ? 'failing' : 'succeeding'}`,
      async function () {
        this.timeout(10 * 4000);
        await runFlow(testCase.totalNumberOfValidators, testCase.fail)
      });
  }

  const runFlow = async function (totalNumberOfValidators, fail) {
    console.log(`Running flow with ${totalNumberOfValidators} validators with the complete transaction ${fail ? 'failing' : 'succeeding'}: `)

    const fixture = await createBeefyValidatorFixture(
      totalNumberOfValidators
    )
    const beefyLightClient = await deployBeefyLightClient(fixture.root,
      totalNumberOfValidators);

    const requiredNumberOfSignatures = (await beefyLightClient.requiredNumberOfSignatures()).toNumber()

    const initialBitfieldPositions = await createRandomPositions(requiredNumberOfSignatures, totalNumberOfValidators)

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
      realWorldFixture.completeSubmitInput.latestMMRLeafIndex,
      realWorldFixture.completeSubmitInput.latestMMRLeafCount,
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
