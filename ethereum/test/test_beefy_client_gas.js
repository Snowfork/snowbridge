const {
  deployBeefyClient,
  mine, printTxPromiseGas
} = require("./helpers");

const {
  createValidatorFixture, createRandomPositions,
  createInitialValidatorProofs, createFinalValidatorProofs
} = require("./beefy-helpers");

const fixture = require('./fixtures/beefy-relay-basic.json')

require("chai")
  .use(require("chai-as-promised"))
  .should();

const { expect } = require("chai");

describe.skip("Beefy Client Gas Usage", function () {

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

    const validatorFixture = await createValidatorFixture(fixture.transactionParams.commitment.validatorSetId, totalNumberOfValidators)

    const beefyClient = await deployBeefyClient(
      validatorFixture.validatorSetId,
      validatorFixture.validatorSetRoot,
      validatorFixture.validatorSetLength,
    );

    const initialBitfieldPositions = await createRandomPositions(totalNumberOfSignatures, totalNumberOfValidators)

    const firstPosition = initialBitfieldPositions[0]

    const initialBitfield = await beefyClient.createInitialBitfield(
      initialBitfieldPositions, totalNumberOfValidators
    );

    const commitmentHash = fixture.commitmentHash

    const initialValidatorProofs = await createInitialValidatorProofs(commitmentHash, validatorFixture);

    const newSigTxPromise = beefyClient.newSignatureCommitment(
      commitmentHash,
      fixture.transactionParams.commitment.validatorSetId,
      initialBitfield,
      initialValidatorProofs[firstPosition].signature,
      firstPosition,
      initialValidatorProofs[firstPosition].address,
      initialValidatorProofs[firstPosition].proof,
    )
    printTxPromiseGas(newSigTxPromise)
    await newSigTxPromise.should.be.fulfilled

    const lastId = (await beefyClient.nextID()).sub(new web3.utils.BN(1));

    await mine(45);

    const completeValidatorProofs = await createFinalValidatorProofs(lastId, beefyClient, initialValidatorProofs);

    const completeSigTxPromise = beefyClient.completeSignatureCommitment(
      fail ? 99 : lastId,
      fixture.transactionParams.commitment,
      completeValidatorProofs,
    )
    printTxPromiseGas(completeSigTxPromise)
    if (fail) {
      await completeSigTxPromise.should.be.rejected
    } else {
      await completeSigTxPromise.should.be.fulfilled
      latestMMRRoot = await beefyClient.latestMMRRoot()
      expect(latestMMRRoot).to.eq(fixture.transactionParams.commitment.payload.mmrRootHash)
    }
  }

});
