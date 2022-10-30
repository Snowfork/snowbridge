const {
  deployBeefyClient,
  mine, printTxPromiseGas,
  createValidatorFixture, createRandomPositions,
  createInitialValidatorProofs, createFinalValidatorProofs
} = require("./helpers");

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
      totalNumberOfValidators: 20,
      totalNumberOfSignatures: 20,
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

    const validatorFixture = await createValidatorFixture(fixture.transactionParams.commitment.validatorSetID-1, totalNumberOfValidators)

    const beefyClient = await deployBeefyClient(
      validatorFixture.validatorSetID,
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

    const newSigTxPromise = beefyClient.submitInitial(
      commitmentHash,
      fixture.transactionParams.commitment.validatorSetId,
      initialBitfield,
      {
        signature: initialValidatorProofs[firstPosition].signature,
        index: firstPosition,
        addr: initialValidatorProofs[firstPosition].address,
        merkleProof: initialValidatorProofs[firstPosition].proof
      }
    )
    printTxPromiseGas(newSigTxPromise)
    await newSigTxPromise.should.be.fulfilled

    const lastId = (await beefyClient.nextRequestID()).sub(new web3.utils.BN(1));

    await mine(45);

    const completeValidatorProofs = await createFinalValidatorProofs(lastId, beefyClient, initialValidatorProofs);

    const completeSigTxPromise = beefyClient.submitFinal(
      fail ? 99 : lastId,
      fixture.params.commitment,
      completeValidatorProofs,
      fixture.params.leaf,
      fixture.params.leafProof
    )
    printTxPromiseGas(completeSigTxPromise)
    if (fail) {
      await completeSigTxPromise.should.be.rejected
    } else {
      await completeSigTxPromise.should.be.fulfilled
      latestMMRRoot = await beefyClient.latestMMRRoot()
      expect(latestMMRRoot).to.eq(fixture.params.commitment.payload.mmrRootHash)
    }
  }

});
