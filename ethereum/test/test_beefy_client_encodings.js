const BigNumber = web3.BigNumber;
const Bitfield = artifacts.require("Bitfield");
const ScaleCodec = artifacts.require("ScaleCodec");
const MerkleProof = artifacts.require("MerkleProof");
const MMRProofVerification = artifacts.require("MMRProofVerification");
const ExposedBeefyClient = artifacts.require("ExposedBeefyClient");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { expect } = require("chai");

describe("BeefyClient", function () {
  before(async function () {
    const bitfield = await Bitfield.new();
    const scaleCodec = await ScaleCodec.new();
    const merkleProof = await MerkleProof.new();
    const mmrProofVerification = await MMRProofVerification.new()
    await ExposedBeefyClient.link(bitfield);
    await ExposedBeefyClient.link(scaleCodec);
    await ExposedBeefyClient.link(merkleProof);
    await ExposedBeefyClient.link(mmrProofVerification);
    this.beefyClient = await ExposedBeefyClient.new();
  });

  it("encodes beefy commitment to SCALE-format", async function () {
    let commitment = {
      blockNumber: 5,
      validatorSetId: 7,
      payload: {
        mmrRootHash: "0x3ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c",
        prefix: "0x0861620c0001026d6880",
        suffix: "0x",
      },
    }

    let encoded = await this.beefyClient.encodeCommitmentExposed(commitment).should.be.fulfilled
    expect(encoded).to.eq("0x0861620c0001026d68803ac49cd24778522203e8bf40a4712ea3f07c3803bbd638cb53ebb3564ec13e8c050000000700000000000000");
  });
});
