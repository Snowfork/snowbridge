const {
  deployBeefyFullClient,
  printTxPromiseGas
} = require("./helpers");

const { createBeefyValidatorFixture,
  createAllValidatorProofs } = require("./beefy-helpers");
const realWorldFixture = require('./fixtures/full-flow.json');

require("chai")
  .use(require("chai-as-promised"))
  .should();

const { expect } = require("chai");

describe("Beefy Full Client Gas Usage", function () {

  const testCases = [
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
    }
  ]

  for (const testCase of testCases) {
    it(`runs full flow with ${testCase.totalNumberOfValidators} validators and ${testCase.totalNumberOfSignatures} signers with the complete transaction ${testCase.fail ? 'failing' : 'succeeding'}`,
      async function () {
        this.timeout(10 * 4000);
        await runFlow(testCase.totalNumberOfValidators, testCase.totalNumberOfSignatures, testCase.fail)
      });
  }

  const runFlow = async function (totalNumberOfValidators, totalNumberOfSignatures, fail) {
    console.log(`Running flow with ${totalNumberOfValidators} validators and ${totalNumberOfSignatures} signatures with the complete transaction ${fail ? 'failing' : 'succeeding'}: `)

    const fixture = await createBeefyValidatorFixture(
      totalNumberOfValidators
    )
    const beefyFullClient = await deployBeefyFullClient(fixture.validatorAddresses);

    const commitmentHash = await beefyFullClient.createCommitmentHash(realWorldFixture.completeSubmitInput.commitment);

    const allValidatorProofs = await createAllValidatorProofs(commitmentHash, fixture);

    const newBeefyBlockPromise = beefyFullClient.newBeefyBlock(
      realWorldFixture.completeSubmitInput.commitment,
      allValidatorProofs.map(p => p.signature),
      realWorldFixture.completeSubmitInput.latestMMRLeaf,
      realWorldFixture.completeSubmitInput.mmrProofItems,
    )
    printTxPromiseGas(newBeefyBlockPromise)
    if (fail) {
      await newBeefyBlockPromise.should.be.rejected
    } else {
      await newBeefyBlockPromise.should.be.fulfilled
      latestMMRRoot = await beefyFullClient.latestMMRRoot()
      expect(latestMMRRoot).to.eq(realWorldFixture.completeSubmitInput.commitment.payload)
    }
  }

});
