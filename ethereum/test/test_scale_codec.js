const ScaleCodec = artifacts.require("ScaleCodec");

const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

function toHexBytes(uint) {
  return "0x" + uint.split(" ").join("");
}

describe("ScaleCodec", function () {

  let codec;

  before(async function() {
    codec = await ScaleCodec.new();
  });

  describe("encoding unsigned integers", async function () {
    it("should encode uint256", async function () {
      const output = await codec.methods["encode256(uint256)"].call("12063978950259949786323707366460749298097791896371638493358994162204017315152");
      output.should.be.equal("0x504d8a21dd3868465c8c9f2898b7f014036935fa9a1488629b109d3d59f8ab1a");
    });

    it("should encode uint128", async function () {
      const output = await codec.methods["encode128(uint128)"].call("35452847761173902980759433963665451267");
      output.should.be.equal("0x036935fa9a1488629b109d3d59f8ab1a");
    });

    it("should encode uint64", async function () {
      const output = await codec.methods["encode64(uint64)"].call("1921902728173129883");
      output.should.be.equal("0x9b109d3d59f8ab1a");
    });

    it("should encode uint32", async function () {
      const output = await codec.methods["encode32(uint32)"].call("447477849");
      output.should.be.equal("0x59f8ab1a");
    });

    it("should encode uint16", async function () {
      const output = await codec.methods["encode16(uint16)"].call("6827");
      output.should.be.equal("0xab1a");
    });
  });


  describe("Gas costs (encoding)", async function () {
    it("uint256", async function () {
      const gasUsed = await codec.encode256.estimateGas("12063978950259949786323707366460749298097791896371638493358994162204017315152");
      console.log('\tEncoding uint256 average gas: ' + gasUsed);
    });
  });
});
