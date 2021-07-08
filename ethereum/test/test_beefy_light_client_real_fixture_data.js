const BigNumber = web3.BigNumber;
const {
  deployBeefyLightClient,
  mine, catchRevert, printBitfield
} = require("./helpers");
const fixture = require('./fixtures/full-flow.json');

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
      fixture.completeSubmitInput.latestMMRLeaf,
      fixture.completeSubmitInput.mmrProofItems,
      fixture.completeSubmitInput.commitment.payload,
      fixture.completeSubmitInput.commitment.blockNumber,
    ).should.be.fulfilled
  });

  it("runs new signature commitment and complete signature commitment correctly", async function () {

    const initialBitfield = await this.beefyLightClient.createInitialBitfield(
      fixture.completeSubmitInput.validatorProof.positions,
      2
    );
    expect(printBitfield(initialBitfield)).to.eq('11')

    const commitmentHash = await this.beefyLightClient.createCommitmentHash(
      fixture.completeSubmitInput.commitment
    );

    const tx = this.beefyLightClient.newSignatureCommitment(
      commitmentHash,
      initialBitfield,
      fixture.completeSubmitInput.validatorProof.signatures[0],
      fixture.completeSubmitInput.validatorProof.positions[0],
      fixture.completeSubmitInput.validatorProof.publicKeys[0],
      fixture.completeSubmitInput.validatorProof.publicKeyMerkleProofs[0],
    )

    await tx.should.be.fulfilled

    const lastId = (await this.beefyLightClient.currentId()).sub(new web3.utils.BN(1));

    await catchRevert(this.beefyLightClient.createRandomBitfield(lastId), 'Error: Block wait period not over');

    await mine(45);

    const bitfield = await this.beefyLightClient.createRandomBitfield(lastId);
    const bitFieldHasOneBit = bitfield.toString() === '2' || bitfield.toString() === '1'
    expect(bitFieldHasOneBit).to.be.true

    await this.beefyLightClient.completeSignatureCommitment(
      lastId,
      fixture.completeSubmitInput.commitment,
      fixture.completeSubmitInput.validatorProof,
      fixture.completeSubmitInput.latestMMRLeaf,
      fixture.completeSubmitInput.mmrProofItems,
    ).should.be.fulfilled

    latestMMRRoot = await this.beefyLightClient.latestMMRRoot()
    expect(latestMMRRoot).to.eq(fixture.completeSubmitInput.commitment.payload)
  });


});
