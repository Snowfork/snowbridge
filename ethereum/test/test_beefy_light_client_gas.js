const BigNumber = web3.BigNumber;
const {
  deployBeefyLightClient,
  createMerkleTree, mine, printBitfield, printTxPromiseGas
} = require("./helpers");
const fixture = require('./fixtures/beefy-fixture-data.json');
const fixture200 = require('./fixtures/beefy-fixture-data-200.js');

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { expect } = require("chai");

describe("Beefy Light Client", function () {

  beforeEach(async function () {
    this.validatorsMerkleTree = createMerkleTree(fixture200.validatorPublicKeys);
    this.beefyLightClient = await deployBeefyLightClient(this.validatorsMerkleTree.getHexRoot(),
      fixture200.validatorPublicKeys.length);
  });

  it("runs new signature commitment with 200 validators", async function () {
    this.timeout(20 * 1000)

    const initialBitfield = await this.beefyLightClient.createInitialBitfield(fixture200.positions, 200);
    console.log(`Initial bitfield is: ${printBitfield(initialBitfield)}`)

    const commitmentHash = await this.beefyLightClient.createCommitmentHash(fixture.commitment);

    console.log("Sending new signature commitment tx")
    const newSigTxPromise = this.beefyLightClient.newSignatureCommitment(
      commitmentHash,
      initialBitfield,
      fixture200.signatures[0],
      0,
      fixture200.validatorPublicKeys[0],
      fixture200.validatorPublicKeyProofs[0]
    )
    printTxPromiseGas(newSigTxPromise)
    await newSigTxPromise.should.be.fulfilled

    const lastId = (await this.beefyLightClient.currentId()).sub(new web3.utils.BN(1));

    await mine(45);

    const bitfield = await this.beefyLightClient.createRandomBitfield(lastId);
    console.log(`Random bitfield is: ${printBitfield(bitfield)}`)

    const validatorProof = {
      signatures: fixture200.signatures,
      positions: fixture200.positions,
      publicKeys: fixture200.validatorPublicKeys,
      publicKeyMerkleProofs: fixture200.validatorPublicKeyProofs
    }

    console.log("Sending complete signature commitment tx")
    const completeSigTxPromise = this.beefyLightClient.completeSignatureCommitment(
      lastId,
      fixture.commitment,
      validatorProof,
      fixture.beefyMMRLeaf,
      fixture.leafProof,
    )
    printTxPromiseGas(completeSigTxPromise)
    await completeSigTxPromise.should.be.fulfilled

    latestMMRRoot = await this.beefyLightClient.latestMMRRoot()
    expect(latestMMRRoot).to.eq(fixture.commitment.payload)
  });


});
