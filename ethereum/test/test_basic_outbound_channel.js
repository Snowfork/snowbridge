const BasicOutboundChannel = artifacts.require("BasicOutboundChannel");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = web3.BigNumber;

const { confirmChannelSend } = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("BasicOutboundChannel", function (accounts) {
  // Accounts
  const userOne = accounts[1];
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.basicSendChannel = await BasicOutboundChannel.new();
    });

    it("should deploy and initialize the ETHApp contract", async function () {
      this.basicSendChannel.should.exist;
    });
  });

  describe("send", function () {
    beforeEach(async function () {
      this.basicSendChannel = await BasicOutboundChannel.new();
    });

    it("should send messages out with the correct event and fields", async function () {
      const tx = await this.basicSendChannel.submit(
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx.receipt.rawLogs[0];
      confirmChannelSend(rawLog, this.basicSendChannel.address, userOne, testPayload)
    });

  });

});