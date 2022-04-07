const BigNumber = web3.BigNumber;
const {
  deployBeefyLightClient,
  mine, catchRevert, printBitfield
} = require("./helpers");

const fixture = require('./fixtures/beefy-relay-basic.json')

const SimpleMMRVerification = artifacts.require("SimplifiedMMRVerification");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { expect } = require("chai");

describe("Beefy Light Client", function () {

  before(async function () {
    this.timeout(10 * 1000)

    this.beefyLightClient = await deployBeefyLightClient();
  });

  it("encodes, hashes and verifies beefy mmr leaves correctly", async function () {
    await this.beefyLightClient.verifyNewestMMRLeaf(
      fixture.finalSignatureCommitment.leaf,
      fixture.finalSignatureCommitment.commitment.payload.mmrRootHash,
      {
        merkleProofItems: fixture.finalSignatureCommitment.proof.merkleProofItems,
        merkleProofOrderBitField: fixture.finalSignatureCommitment.proof.merkleProofOrderBitField
      }
    ).should.be.fulfilled
  });

  it("runs new signature commitment and complete signature commitment correctly", async function () {

    const bitfield = await this.beefyLightClient.createInitialBitfield(
      fixture.finalSignatureCommitment.validatorProof.positions,
      2
    );
    expect(printBitfield(bitfield)).to.eq('100')

    const tx = this.beefyLightClient.newSignatureCommitment(
      fixture.commitmentHash,
      bitfield,
      fixture.finalSignatureCommitment.validatorProof.signatures[0],
      fixture.finalSignatureCommitment.validatorProof.positions[0],
      fixture.finalSignatureCommitment.validatorProof.publicKeys[0],
      fixture.finalSignatureCommitment.validatorProof.publicKeyMerkleProofs[0],
    )

    await tx.should.be.fulfilled

    const lastId = (await this.beefyLightClient.currentId()).sub(new web3.utils.BN(1));

    await mine(3);

    await this.beefyLightClient.completeSignatureCommitment(
      lastId,
      fixture.finalSignatureCommitment.commitment,
      fixture.finalSignatureCommitment.validatorProof,
      fixture.finalSignatureCommitment.leaf,
        {
          merkleProofItems: fixture.finalSignatureCommitment.proof.merkleProofItems,
          merkleProofOrderBitField: fixture.finalSignatureCommitment.proof.merkleProofOrderBitField
        }
    ).should.be.fulfilled

    root = await this.beefyLightClient.latestMMRRoot()
    expect(root).to.eq(fixture.finalSignatureCommitment.commitment.payload.mmrRootHash)
  });


});
