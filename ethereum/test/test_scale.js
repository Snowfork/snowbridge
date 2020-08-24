const Scale = artifacts.require("Scale");

const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

function toHexBytes(uint) {
  return "0x" + uint.split(" ").join("");
}

contract("Scale", function () {

  describe("Scale contract deployment", function () {
    beforeEach(async function () {
      this.scale = await Scale.new();
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
      this.scale = await Scale.new();
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

  describe("Gas costs", async function () {

    beforeEach(async function () {
      this.scale = await Scale.new();
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
