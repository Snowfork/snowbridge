const {
  deployBeefyLightClient,
  mine, printBitfield, printTxPromiseGas
} = require("./helpers");
const { keccakFromHexString } = require("ethereumjs-util");
const secp256k1 = require('secp256k1')
const { ethers } = require("ethers");

const { createBeefyValidatorFixture, createRandomPositions } = require("./beefy-helpers");
const realWorldFixture = require('./fixtures/full-flow.json');

require("chai")
  .use(require("chai-as-promised"))
  .should();

const { expect } = require("chai");

describe("Beefy Light Client Gas Usage", function () {

  it("runs new signature commitment with 200 validators", async function () {
    this.timeout(10 * 2000)
    const numberOfValidators = 200;
    const numberOfSignatures = 140;

    const fixture = await createBeefyValidatorFixture(
      numberOfValidators
    )

    this.beefyLightClient = await deployBeefyLightClient(fixture.root,
      numberOfValidators);

    const initialBitfieldPositions = await createRandomPositions(numberOfSignatures, numberOfValidators)

    const initialBitfield = await this.beefyLightClient.createInitialBitfield(
      initialBitfieldPositions, numberOfValidators
    );

    const commitmentHash = await this.beefyLightClient.createCommitmentHash(realWorldFixture.completeSubmitInput.commitment);

    console.log({ initialBitfieldPositions })
    console.log(`Initial bitfield is: ${printBitfield(initialBitfield)}`)

    let commitmentHashBytes = ethers.utils.arrayify(commitmentHash)
    const tree = fixture.validatorsMerkleTree;
    const leaves = tree.getHexLeaves()

    const allValidatorProofs = initialBitfieldPositions.reduce((accum, position) => {
      const leaf = leaves[position]
      const wallet = fixture.walletsByLeaf[leaf]
      const address = wallet.address
      const proof = tree.getHexProof(leaf, position)
      const privateKey = ethers.utils.arrayify(wallet.privateKey)
      const signatureECDSA = secp256k1.ecdsaSign(commitmentHashBytes, privateKey)
      const ethRecID = signatureECDSA.recid + 27
      const signature = Uint8Array.from(
        signatureECDSA.signature.join().split(',').concat(ethRecID)
      )
      accum.positions.push(position)
      accum.publicKeys.push(address)
      accum.publicKeyMerkleProofs.push(proof)
      accum.signatures.push(ethers.utils.hexlify(signature))
      return accum
    }, {
      signatures: [],
      positions: [],
      publicKeys: [],
      publicKeyMerkleProofs: []
    });

    console.log("Sending new signature commitment tx")
    const newSigTxPromise = this.beefyLightClient.newSignatureCommitment(
      commitmentHash,
      initialBitfield,
      allValidatorProofs.signatures[0],
      allValidatorProofs.positions[0],
      allValidatorProofs.publicKeys[0],
      allValidatorProofs.publicKeyMerkleProofs[0],
    )
    printTxPromiseGas(newSigTxPromise)
    await newSigTxPromise.should.be.fulfilled

    const lastId = (await this.beefyLightClient.currentId()).sub(new web3.utils.BN(1));
    console.log("Onto the next one")

    await mine(45);

    const bitfield = await this.beefyLightClient.createRandomBitfield(lastId);
    console.log(`Random bitfield is: ${printBitfield(bitfield)}`)

    console.log("Sending complete signature commitment tx")
    const completeSigTxPromise = this.beefyLightClient.completeSignatureCommitment(
      lastId,
      realWorldFixture.completeSubmitInput.commitment,
      allValidatorProofs,
      realWorldFixture.completeSubmitInput.latestMMRLeaf,
      realWorldFixture.completeSubmitInput.mmrProofItems,
    )
    console.log("Sent it")
    // printTxPromiseGas(completeSigTxPromise)
    // completeSigTxPromise.catch((a, s, d) => console.log({ a, s, d }))
    await completeSigTxPromise.should.be.fulfilled

    latestMMRRoot = await this.beefyLightClient.latestMMRRoot()
    expect(latestMMRRoot).to.eq(fixture.commitment.payload)
  });


});
