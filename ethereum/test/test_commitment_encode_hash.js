const ScaleCodec = artifacts.require("ScaleCodec");
const Blake2b = artifacts.require("Blake2b");
const BigNumber = require("bignumber.js");
const { expect } = require("chai");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("Commitment encode and hash", function () {
  let scale;
  let blake2b;
  beforeEach(async function () {
    scale = await ScaleCodec.new();
    blake2b = await Blake2b.new();
  });

  it("should hash correctly", async function () {
    const commitment = {
      payload: "0xf1993fbbd4a16e2bfde57f431495e06ab72c784e52e3b78c987c2c8d07aed87b", // types.H256
      block_number: "4", // types.U32
      validator_set_id: "0", // types.U64
    }

    expect(await scale.encode32(commitment.block_number)).to.equal('0x04000000')
    expect(await scale.encode64(commitment.validator_set_id)).to.equal('0x0000000000000000')

    const commitmentBytes = "0xf1993fbbd4a16e2bfde57f431495e06ab72c784e52e3b78c987c2c8d07aed87b040000000000000000000000"
    const hashedCommitment = "0xd4fd3b5591f7ff85b071cbfd869218b391bad8c5f6cb469ce413d72c785d966e"

    const out = await blake2b.blake2b(commitmentBytes, "0x", 32)
    const formatted = await blake2b.formatOutput(out)

    expect(formatted[0]).to.equal(hashedCommitment);
  });
});