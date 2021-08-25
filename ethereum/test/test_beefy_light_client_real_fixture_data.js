const BigNumber = web3.BigNumber;
const {
  deployBeefyLightClient,
  mine, catchRevert, printBitfield
} = require("./helpers");
const fixture = require('./fixtures/full-flow-basic.json');

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
      fixture.completeSubmitInput.mmrLeafIndex,
      fixture.completeSubmitInput.mmrLeafCount,
    ).should.be.fulfilled
  });

  it("runs new signature commitment and complete signature commitment correctly", async function () {

    const initialBitfield = await this.beefyLightClient.createInitialBitfield(
      fixture.completeSubmitInput.validatorProof.positions,
      2
    );
    expect(printBitfield(initialBitfield)).to.eq('1')

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
    const bitfieldString = printBitfield(bitfield);
    const bitFieldHasOneBit = bitfieldString === '1' || bitfieldString === '10' // (trailing 0's are removed)
    expect(bitFieldHasOneBit).to.be.true

    const validatorProofs = {
      signatures: [],
      positions: [],
      publicKeys: [],
      publicKeyMerkleProofs: [],
    }

    const ascendingBitfield = bitfieldString.split('').reverse().join('');
    for (let position = 0; position < ascendingBitfield.length; position++) {
      const bit = ascendingBitfield[position]
      if (bit === '1') {
        validatorProofs.signatures.push(fixture.completeSubmitInput.validatorProof.signatures[position])
        validatorProofs.positions.push(fixture.completeSubmitInput.validatorProof.positions[position])
        validatorProofs.publicKeys.push(fixture.completeSubmitInput.validatorProof.publicKeys[position])
        validatorProofs.publicKeyMerkleProofs.push(fixture.completeSubmitInput.validatorProof.publicKeyMerkleProofs[position])
      }
    }

    await this.beefyLightClient.completeSignatureCommitment(
      lastId,
      fixture.completeSubmitInput.commitment,
      validatorProofs,
      fixture.completeSubmitInput.latestMMRLeaf,
      fixture.completeSubmitInput.mmrLeafIndex,
      fixture.completeSubmitInput.mmrLeafCount,
      fixture.completeSubmitInput.mmrProofItems,
    ).should.be.fulfilled

    latestMMRRoot = await this.beefyLightClient.latestMMRRoot()
    expect(latestMMRRoot).to.eq(fixture.completeSubmitInput.commitment.payload)
  });


});
