const BigNumber = web3.BigNumber;
const {
  deployBeefyClient,
  mine, catchRevert, printBitfield
} = require("./helpers");

const fixture = require('./fixtures/beefy-relay-basic.json')

const leafFixture = require('./fixtures/beefy-relay-validators-basic.json')


require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { expect } = require("chai");

describe("BeefyClient", function () {

  before(async function () {
    this.beefyClient = await deployBeefyClient(
      leafFixture.updateLeaf.leaf.nextAuthoritySetId,
      leafFixture.updateLeaf.leaf.nextAuthoritySetRoot,
      leafFixture.updateLeaf.leaf.nextAuthoritySetLen,
    );
  });

  it("runs new signature commitment and complete signature commitment correctly", async function () {

    const bitfield = await this.beefyClient.createInitialBitfield(
      fixture.transactionParams.proof.positions,
      3
    );

    const tx = this.beefyClient.newSignatureCommitment(
      fixture.commitmentHash,
      fixture.transactionParams.commitment.validatorSetId + 1,
      bitfield,
      fixture.transactionParams.proof.signatures[0],
      fixture.transactionParams.proof.positions[0],
      fixture.transactionParams.proof.publicKeys[0],
      fixture.transactionParams.proof.publicKeyMerkleProofs[0],
    )

    await tx.should.be.fulfilled

    const lastId = (await this.beefyClient.nextID()).sub(new web3.utils.BN(1));

    await mine(3);

    await this.beefyClient.completeSignatureCommitment(
      lastId,
      fixture.transactionParams.commitment,
      fixture.transactionParams.proof,
    ).should.be.fulfilled

    root = await this.beefyClient.latestMMRRoot()
    expect(root).to.eq(fixture.transactionParams.commitment.payload.mmrRootHash)
  });


});
