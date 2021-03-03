const IncentivizedOutboundChannel = artifacts.require("IncentivizedOutboundChannel");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = web3.BigNumber;

const { confirmIncentivizedChannelSend } = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("IncentivizedOutboundChannel", function (accounts) {
  // Accounts
  const appAddress = accounts[1];
  const origin = accounts[2];
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new();
    });
  });

  describe("send", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new();
    });

    it("should send messages out with the correct event and fields", async function () {
      const tx = await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx.receipt.rawLogs[0];
      confirmIncentivizedChannelSend(rawLog, this.channel.address, userOne, 1, testPayload)
    });

    it("should increment nonces correctly", async function () {
      const tx = await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      const tx2 = await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      const tx3 = await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx3.receipt.rawLogs[0];
      confirmIncentivizedChannelSend(rawLog, this.channel.address, appAddress, 3, testPayload)
    });

  });

});
