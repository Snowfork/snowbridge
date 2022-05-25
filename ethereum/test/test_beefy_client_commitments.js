const BigNumber = web3.BigNumber;
const {
  deployBeefyClient, mine
} = require("./helpers");

const fixture = require('./fixtures/beefy-relay-basic.json')


require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { expect } = require("chai");


let METHOD = "submitFinal(uint256,(uint32,uint64,(bytes32,bytes,bytes)),(bytes[],uint256[],address[],bytes32[][]),(uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32),(bytes32[],uint64))";

describe("BeefyClient", function () {

  before(async function () {
    this.beefyClient = await deployBeefyClient(
      fixture.params.commitment.validatorSetID-1,
      fixture.params.leaf.nextAuthoritySetRoot,
      fixture.params.leaf.nextAuthoritySetLen
    );
  });

  it("runs new signature commitment and complete signature commitment correctly", async function () {

    const bitfield = await this.beefyClient.createInitialBitfield(
      fixture.params.proof.indices,
      3
    );

    const tx = this.beefyClient.submitInitial(
      fixture.commitmentHash,
      fixture.params.commitment.validatorSetID,
      bitfield,
      {
        signature: fixture.params.proof.signatures[0],
        index: fixture.params.proof.indices[0],
        addr: fixture.params.proof.addrs[0],
        merkleProof: fixture.params.proof.merkleProofs[0]
      }
    )

    await tx.should.be.fulfilled

    const lastId = (await this.beefyClient.nextRequestID()).sub(new web3.utils.BN(1));

    await mine(3);

    await this.beefyClient.methods[METHOD](
      lastId,
      fixture.params.commitment,
      fixture.params.proof,
      fixture.params.leaf,
      fixture.params.leafProof
    ).should.be.fulfilled;

    root = await this.beefyClient.latestMMRRoot()
    expect(root).to.eq(fixture.params.commitment.payload.mmrRootHash)
  });


});
