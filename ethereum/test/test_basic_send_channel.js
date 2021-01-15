const BasicSendChannel = artifacts.require("BasicSendChannel");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = web3.BigNumber;

const { confirmChannelSend } = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("BasicSendChannel", function (accounts) {
  // Accounts
  const userOne = accounts[1];
  const testAppId = "arbitrary-app-id";
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.basicSendChannel = await BasicSendChannel.new();
    });

    it("should deploy and initialize the ETHApp contract", async function () {
      this.basicSendChannel.should.exist;
    });
  });

  describe("send", function () {
    beforeEach(async function () {
      this.basicSendChannel = await BasicSendChannel.new();
    });

    it("should send messages out with the correct event and fields", async function () {
      const tx = await this.basicSendChannel.send(
        testAppId,
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx.receipt.rawLogs[0];
      confirmChannelSend(rawLog, this.basicSendChannel.address, userOne, testAppId, testPayload)
    });

  });

});