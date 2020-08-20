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

      it("case 0: [0, 63]", async function () {
        const tests = [
          {encoded: toHexBytes("00"), decoded: 0},
          {encoded: toHexBytes("fc"), decoded: 63},
        ];

        for(test of tests) {
          const output = Number(await this.scale.decodeUintCompact.call(test.encoded));
          output.should.be.bignumber.equal(test.decoded);
        }
      });

      it("case 1: [64, 16383]", async function () {
        const tests = [
          {encoded: toHexBytes("01 01"), decoded: 64},
          {encoded: toHexBytes("fd ff"), decoded: 16383},
        ];

        for(test of tests) {
          const output = Number(await this.scale.decodeUintCompact.call(test.encoded));
          output.should.be.bignumber.equal(test.decoded);
        }
      });

      it("case 2: [16384, 1073741823]", async function () {
        const tests = [
          {encoded: toHexBytes("02 00 01 00"), decoded: 16384},
          {encoded: toHexBytes("fe ff ff ff"), decoded: 1073741823},
        ];

        for(test of tests) {
          const output = Number(await this.scale.decodeUintCompact.call(test.encoded));
          output.should.be.bignumber.equal(test.decoded);
        }
      });

      it("case 3: [1073741823, 4503599627370496]", async function () {
        const tests = [
          {encoded: toHexBytes("03 00 00 00 40"), decoded: 1073741824},
          // {encoded: toHexBytes("03 ff ff ff ff"), decoded:  1<<32 - 1}, // TODO: slightly off
          // {encoded: toHexBytes("07 00 00 00 00 01"), decoded: 1 << 32},
          // {encoded: toHexBytes("0b 00 00 00 00 00 01"), decoded: 1 << 40},
          // {encoded: toHexBytes("0f ff ff ff ff ff ff ff"), decoded: 1 << 48},
          // {encoded: toHexBytes("0f 00 00 00 00 00 00 01"), decoded: 1<<56 - 1},
          // {encoded: toHexBytes("13 00 00 00 00 00 00 00 01"), decoded:  1 << 56},
        ];

        for(test of tests) {
          const output = Number(await this.scale.decodeUintCompact.call(test.encoded));
          output.should.be.bignumber.equal(test.decoded);
        }
      });
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

      it("compact uints: [1073741823, 4503599627370496]", async function () {
        const tests = [
          {encoded: toHexBytes("03 00 00 00 40"), decoded: 1073741824},
        ];

        let totalGas = 0;
        for(test of tests) {
          const gasCost = await this.scale.decodeUintCompact.estimateGas(test.encoded);
          totalGas += Number(gasCost);
        }
        totalGas = totalGas / tests.length;
        console.log('\tDecoding [1073741823, 4503599627370496] average gas: ' + totalGas);
      });
    });
});
