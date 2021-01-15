const IncentivizedSendChannel = artifacts.require("IncentivizedSendChannel");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = web3.BigNumber;

const { confirmChannelSend } = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("IncentivizedSendChannel", function (accounts) {
  // Accounts
  const userOne = accounts[1];
  const testAppId = "arbitrary-app-id";
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.incentivizedSendChannel = await IncentivizedSendChannel.new();
    });

    it("should deploy and initialize the ETHApp contract", async function () {
      this.incentivizedSendChannel.should.exist;
    });
  });

  describe("send", function () {
    beforeEach(async function () {
      this.incentivizedSendChannel = await IncentivizedSendChannel.new();
    });

    it("should send messages out with the correct event and fields", async function () {
      const tx = await this.incentivizedSendChannel.send(
        testAppId,
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx.receipt.rawLogs[0];
      confirmChannelSend(rawLog, this.incentivizedSendChannel.address, userOne, testAppId, testPayload)
    });

    it("should increment nonces correctly", async function () {
      const tx = await this.incentivizedSendChannel.send(
        testAppId,
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const tx2 = await this.incentivizedSendChannel.send(
        testAppId,
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const tx3 = await this.incentivizedSendChannel.send(
        testAppId,
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx3.receipt.rawLogs[0];
      confirmChannelSend(rawLog, this.incentivizedSendChannel.address, userOne, testAppId, testPayload, 2)
    });

  });

});