const ScaleCodec = artifacts.require("ScaleCodec");

const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

function toHexBytes(uint) {
  return "0x" + uint.split(" ").join("");
}

contract("ScaleCodec", function () {

  describe("Scale contract deployment", function () {
    beforeEach(async function () {
      this.scale = await ScaleCodec.new();
    });

    it("should deploy and initialize the contract", async function () {
      this.scale.should.exist;
    });

    describe("decoding compact uints", async function () {

      it("should decode case 0: [0, 63]", async function () {
        const tests = [
          {encoded: toHexBytes("00"), decoded: 0},
          {encoded: toHexBytes("fc"), decoded: 63},
        ];

        for(test of tests) {
          const output = Number(await this.scale.decodeUintCompact.call(test.encoded));
          output.should.be.bignumber.equal(test.decoded);
        }
      });

      it("should decode case 1: [64, 16383]", async function () {
        const tests = [
          {encoded: toHexBytes("01 01"), decoded: 64},
          {encoded: toHexBytes("fd ff"), decoded: 16383},
        ];

        for(test of tests) {
          const output = Number(await this.scale.decodeUintCompact.call(test.encoded));
          output.should.be.bignumber.equal(test.decoded);
        }
      });

      it("should decode case 2: [16384, 1073741823]", async function () {
        const tests = [
          {encoded: toHexBytes("02 00 01 00"), decoded: 16384},
          {encoded: toHexBytes("fe ff ff ff"), decoded: 1073741823},
        ];

        for(test of tests) {
          const output = Number(await this.scale.decodeUintCompact.call(test.encoded));
          output.should.be.bignumber.equal(test.decoded);
        }
      });

      it("should reject case 3: [1073741824, 4503599627370496]", async function () {
        const tests = [
          {encoded: toHexBytes("03 00 00 00 40"), decoded: 1073741824},
          {encoded: toHexBytes("07 00 00 00 00 01"), decoded: 1 << 32},
          {encoded: toHexBytes("0f ff ff ff ff ff ff ff"), decoded: 1 << 48},
          {encoded: toHexBytes("13 00 00 00 00 00 00 00 01"), decoded:  1 << 56},
        ];

        for(test of tests) {
          await this.scale.decodeUintCompact.call(test.encoded).should.not.be.fulfilled;
        }
      });
    });
  });

  describe("decoding uint256s", async function () {

    beforeEach(async function () {
      this.scale = await ScaleCodec.new();
    });

    it("should decode uint256", async function () {
      const tests = [
        {encoded: toHexBytes("1d 00 00"), decoded: 29},
        {encoded: "0x1d000000000000000000000000000000", decoded: 29},
        {encoded: "0x3412", decoded: 4660},
        {encoded: "0x201f1e1d1c1b1a1817161514131211100f0e0d0c0b0a09080706050403020100", decoded: 1780731860627700044960722568376592200742329637303199754547880948779589408},
      ];

      for(test of tests) {
        const output = Number(await this.scale.decodeUint256.call(test.encoded));
        output.should.be.bignumber.equal(test.decoded);
      }
    });
  });

  describe("encoding unsigned integers", async function () {
    beforeEach(async function () {
      this.scale = await ScaleCodec.new();
    });

    it("should encode uint256", async function () {
      const output = await this.scale.methods["encode256(uint256)"].call("12063978950259949786323707366460749298097791896371638493358994162204017315152");
      output.should.be.equal("0x504d8a21dd3868465c8c9f2898b7f014036935fa9a1488629b109d3d59f8ab1a");
    });

    it("should encode uint128", async function () {
      const output = await this.scale.methods["encode128(uint128)"].call("35452847761173902980759433963665451267");
      output.should.be.equal("0x036935fa9a1488629b109d3d59f8ab1a");
    });

    it("should encode uint64", async function () {
      const output = await this.scale.methods["encode64(uint64)"].call("1921902728173129883");
      output.should.be.equal("0x9b109d3d59f8ab1a");
    });

    it("should encode uint32", async function () {
      const output = await this.scale.methods["encode32(uint32)"].call("447477849");
      output.should.be.equal("0x59f8ab1a");
    });

    it("should encode uint16", async function () {
      const output = await this.scale.methods["encode16(uint16)"].call("6827");
      output.should.be.equal("0xab1a");
    });
  });


  describe("Gas costs (encoding)", async function () {

    beforeEach(async function () {
      this.scale = await ScaleCodec.new();
    });

    it("uint256", async function () {
      const gasUsed = await this.scale.encode256.estimateGas("12063978950259949786323707366460749298097791896371638493358994162204017315152");
      console.log('\tEncoding uint256 average gas: ' + gasUsed);
    });

  });

  describe("Gas costs (decoding)", async function () {

    beforeEach(async function () {
      this.scale = await ScaleCodec.new();
    });

    it("compact uints: [0, 63]", async function () {
      const tests = [
        {encoded: toHexBytes("00"), decoded: 0},
        {encoded: toHexBytes("fc"), decoded: 63},
      ];

      let totalGas = 0;
      for(test of tests) {
        const gasCost = await this.scale.decodeUintCompact.estimateGas(test.encoded);
        totalGas += Number(gasCost);
      }
      totalGas = totalGas / tests.length;
      console.log('\tDecoding [0, 63] average gas: ' + totalGas);
    });

    it("compact uints: [64, 16383]", async function () {
      const tests = [
        {encoded: toHexBytes("01 01"), decoded: 64},
        {encoded: toHexBytes("fd ff"), decoded: 16383},
      ];

      let totalGas = 0;
      for(test of tests) {
        const gasCost = await this.scale.decodeUintCompact.estimateGas(test.encoded);
        totalGas += Number(gasCost);
      }
      totalGas = totalGas / tests.length;
      console.log('\tDecoding [64, 16383] average gas: ' + totalGas);
    });

    it("compact uints: [16384, 1073741823]", async function () {
      const tests = [
        {encoded: toHexBytes("02 00 01 00"), decoded: 16384},
        {encoded: toHexBytes("fe ff ff ff"), decoded: 1073741823},
      ];

      let totalGas = 0;
      for(test of tests) {
        const gasCost = await this.scale.decodeUintCompact.estimateGas(test.encoded);
        totalGas += Number(gasCost);
      }
      totalGas = totalGas / tests.length;
      console.log('\tDecoding [16384, 1073741823] average gas: ' + totalGas);
    });

    it("uint256", async function () {
      const tests = [
        {encoded: "0x1d000000000000000000000000000000", decoded: 29},
        {encoded: "0x3412", decoded: 4660},
        {encoded: "0x201f1e1d1c1b1a1817161514131211100f0e0d0c0b0a09080706050403020100", decoded: 1780731860627700044960722568376592200742329637303199754547880948779589408},
      ];

      let totalGas = 0;
      for(test of tests) {
        const gasCost = await this.scale.decodeUint256.estimateGas(test.encoded);
        totalGas += Number(gasCost);
      }
      totalGas = totalGas / tests.length;
      console.log('\tDecoding uint256 average gas: ' + totalGas);
    });
  });
});
