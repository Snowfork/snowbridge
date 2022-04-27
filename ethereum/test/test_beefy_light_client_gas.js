const {
  deployBeefyLightClient,
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

    const validatorFixture = await createValidatorFixture(
      fixture.finalSignatureCommitment.commitment.validatorSetId,
      totalNumberOfValidators
    )
    const beefyLightClient = await deployBeefyLightClient(
      validatorFixture.validatorSetId,
      validatorFixture.validatorSetRoot,
      validatorFixture.validatorSetLength,
    );

    const initialBitfieldPositions = await createRandomPositions(totalNumberOfSignatures, totalNumberOfValidators)

    const firstPosition = initialBitfieldPositions[0]

    const initialBitfield = await beefyLightClient.createInitialBitfield(
      initialBitfieldPositions, totalNumberOfValidators
    );

    const commitmentHash = fixture.commitmentHash

    const initialValidatorProofs = await createInitialValidatorProofs(commitmentHash, validatorFixture);

    const newSigTxPromise = beefyLightClient.newSignatureCommitment(
      commitmentHash,
      initialBitfield,
      initialValidatorProofs[firstPosition].signature,
      firstPosition,
      initialValidatorProofs[firstPosition].address,
      initialValidatorProofs[firstPosition].proof,
    )
    printTxPromiseGas(newSigTxPromise)
    await newSigTxPromise.should.be.fulfilled

    const lastId = (await beefyLightClient.nextID()).sub(new web3.utils.BN(1));

    await mine(45);

    const completeValidatorProofs = await createFinalValidatorProofs(lastId, beefyLightClient, initialValidatorProofs);

    const completeSigTxPromise = beefyLightClient.completeSignatureCommitment(
      fail ? 99 : lastId,
      fixture.finalSignatureCommitment.commitment,
      completeValidatorProofs,
      fixture.finalSignatureCommitment.leaf,
       {
          items: fixture.finalSignatureCommitment.proof.items,
          order: fixture.finalSignatureCommitment.proof.order
       }
    )
    printTxPromiseGas(completeSigTxPromise)
    if (fail) {
      await completeSigTxPromise.should.be.rejected
    } else {
      await completeSigTxPromise.should.be.fulfilled
      latestMMRRoot = await beefyLightClient.latestMMRRoot()
      expect(latestMMRRoot).to.eq(fixture.finalSignatureCommitment.commitment.payload.mmrRootHash)
    }
  }

});
