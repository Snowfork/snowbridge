const Scale = artifacts.require("Scale");

const Web3Utils = require("web3-utils");
const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("Scale", function (accounts) {
  const char = "a";
  const charEncoded = [0x04, 0x61];
  // SCALE PREFIX: unsigned integer 1: 0x04
  // UTF ENCODING FOR "a": \x61

  const recipientAddress = "c230f38ff05860753840e0d7cbc66128ad308b67";
  const scaleEncodedRecipientAddress = Buffer.from("50c230f38ff05860753840e0d7cbc66128ad308b67", "hex");
  console.log("scaleEncodedRecipientAddress:", scaleEncodedRecipientAddress)

  const uint64Amount = 1000000000000000000000;
  const uint64ScaleEncodedAmount = "0x0000a0dec5adc9353600000000000000"

  describe("Scale contract deployment", function () {
    beforeEach(async function () {
      this.scale = await Scale.new();
    });

    it("should deploy and initialize the contract", async function () {
      this.scale.should.exist;

    });

    // it("should decode a scale-encoded byte array", async function () {
    //     const char = "a";
    //     const charEncoded = [0x04, 0x61];

    //     const { logs } = await this.scale.decodeBytes(charEncoded);
    //     const logData = logs.find(
    //         e => e.event === "LogData"
    //     );
    //     console.log("LogData data:", logData.args._data)

    //     console.log("decodedBytes:", decodedBytes);

    //     char.should.be.equal(decodedBytes);
    //   });


    it("should decode a scale-encoded Ethereum address", async function () {
        const decodedBytes = await this.scale.decodeBytes(scaleEncodedRecipientAddress);

        console.log("decodedBytes:", decodedBytes);

        char.should.be.equal(decodedBytes);
      });

  });
});

// 1. let { ApiPromise } = require('@polkadot/api');
// 2. var api = async function() { return await ApiPromise.create(); }
