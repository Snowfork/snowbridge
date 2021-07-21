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
    },
    {
      totalNumberOfValidators: 400,
    },
    {
      totalNumberOfValidators: 667,
    },
    {
      totalNumberOfValidators: 1000,
    },
  ]

  for (const testCase of testCases) {
    it(`runs full flow with ${testCase.totalNumberOfValidators} validators`,
      async function () {
        this.timeout(10 * 4000);
        await runFlow(testCase.totalNumberOfValidators)
      });
  }

  const runFlow = async function (totalNumberOfValidators) {
    console.log(`Running flow with ${totalNumberOfValidators} validators`)

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
    await newBeefyBlockPromise.should.be.fulfilled
    latestMMRRoot = await beefyFullClient.latestMMRRoot()
    expect(latestMMRRoot).to.eq(realWorldFixture.completeSubmitInput.commitment.payload)
  }

});
