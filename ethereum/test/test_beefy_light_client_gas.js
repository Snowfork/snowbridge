const {
  deployBeefyLightClient,
  mine, printBitfield, printTxPromiseGas
} = require("./helpers");
const { keccakFromHexString } = require("ethereumjs-util");
const secp256k1 = require('secp256k1')
const { ethers } = require("ethers");

const { createBeefyValidatorFixture, createRandomPositions, createFullPositions } = require("./beefy-helpers");
const realWorldFixture = require('./fixtures/full-flow.json');

require("chai")
  .use(require("chai-as-promised"))
  .should();

const { expect } = require("chai");


describe("Beefy Light Client Gas Usage", function () {

  const testCases = [
    {
      totalNumberOfValidators: 200,
      totalNumberOfSignatures: 200,
    },
    {
      totalNumberOfValidators: 200,
      totalNumberOfSignatures: 134,
    },
    // {
    //   totalNumberOfValidators: 265,
    //   totalNumberOfSignatures: 265,
    // },
    // {
    //   totalNumberOfValidators: 266,
    //   totalNumberOfSignatures: 266,
    // },
    {
      totalNumberOfValidators: 1000,
      totalNumberOfSignatures: 1000,
      fail: true
    },
    // {
    //   totalNumberOfValidators: 1000,
    //   totalNumberOfSignatures: 1000,
    // },
    // {
    //   totalNumberOfValidators: 1000,
    //   totalNumberOfSignatures: 667,
    // }
  ]

  for (const testCase of testCases) {
    it(`runs full flow with ${testCase.totalNumberOfValidators} validators and ${testCase.totalNumberOfSignatures} signers with the complete transaction ${testCase.fail ? 'failing' : 'succeeding'}`,
      async function () {
        this.timeout(10 * 2000);
        await runFlow(testCase.totalNumberOfValidators, testCase.totalNumberOfSignatures, testCase.fail)
      });
  }

  const runFlow = async function (totalNumberOfValidators, totalNumberOfSignatures, fail) {
    console.log(`Running flow with ${totalNumberOfValidators} validators and ${totalNumberOfSignatures} signatures: `)

    const fixture = await createBeefyValidatorFixture(
      totalNumberOfValidators
    )
    const beefyLightClient = await deployBeefyLightClient(fixture.root,
      totalNumberOfValidators);

    const initialBitfieldPositions = await createRandomPositions(totalNumberOfSignatures, totalNumberOfValidators)

    const initialBitfield = await beefyLightClient.createInitialBitfield(
      initialBitfieldPositions, totalNumberOfValidators
    );

    const commitmentHash = await beefyLightClient.createCommitmentHash(realWorldFixture.completeSubmitInput.commitment);

    let commitmentHashBytes = ethers.utils.arrayify(commitmentHash)
    const tree = fixture.validatorsMerkleTree;
    const leaves = tree.getHexLeaves()

    const allValidatorProofs = leaves.map((leaf, position) => {
      const wallet = fixture.walletsByLeaf[leaf]
      const address = wallet.address
      const proof = tree.getHexProof(leaf, position)
      const privateKey = ethers.utils.arrayify(wallet.privateKey)
      const signatureECDSA = secp256k1.ecdsaSign(commitmentHashBytes, privateKey)
      const ethRecID = signatureECDSA.recid + 27
      const signature = Uint8Array.from(
        signatureECDSA.signature.join().split(',').concat(ethRecID)
      )
      return { signature: ethers.utils.hexlify(signature), position, address, proof };
    });

    const newSigTxPromise = beefyLightClient.newSignatureCommitment(
      commitmentHash,
      initialBitfield,
      allValidatorProofs[0].signature,
      allValidatorProofs[0].position,
      allValidatorProofs[0].address,
      allValidatorProofs[0].proof,
    )
    printTxPromiseGas(newSigTxPromise)
    await newSigTxPromise.should.be.fulfilled

    const lastId = (await beefyLightClient.currentId()).sub(new web3.utils.BN(1));

    await mine(45);

    const bitfieldInts = await beefyLightClient.createRandomBitfield(lastId);
    const bitfieldString = printBitfield(bitfieldInts);

    const validatorProofs = {
      signatures: [],
      positions: [],
      publicKeys: [],
      publicKeyMerkleProofs: [],
    }

    ascendingBitfield = bitfieldString.split('').reverse().join('');
    for (let position = 0; position < ascendingBitfield.length; position++) {
      const bit = ascendingBitfield[position]
      if (bit === '1') {
        validatorProofs.signatures.push(allValidatorProofs[position].signature)
        validatorProofs.positions.push(allValidatorProofs[position].position)
        validatorProofs.publicKeys.push(allValidatorProofs[position].address)
        validatorProofs.publicKeyMerkleProofs.push(allValidatorProofs[position].proof)
      }
    }

    const completeSigTxPromise = beefyLightClient.completeSignatureCommitment(
      fail ? 99 : lastId,
      realWorldFixture.completeSubmitInput.commitment,
      validatorProofs,
      realWorldFixture.completeSubmitInput.latestMMRLeaf,
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
