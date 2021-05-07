const IncentivizedOutboundChannel = artifacts.require("IncentivizedOutboundChannel");
const MockFeeSource = artifacts.require("MockFeeSource");

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
  const owner = accounts[0];
  const appAddress = accounts[1];
  const origin = accounts[2];
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");

  describe("send", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new();

      const feeSource = await MockFeeSource.new();
      await this.channel.initialize(owner, feeSource.address, [appAddress]).should.be.fulfilled;
    });

    it("should send messages out with the correct event and fields", async function () {
      const tx = await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx.receipt.rawLogs[0];
      confirmIncentivizedChannelSend(rawLog, this.channel.address, appAddress, 1, testPayload)
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

    it("should not send message if user cannot pay fee", async function () {

      // Trigger our mock fee source to revert in burnFee.
      await this.channel.setFee(1024).should.be.fulfilled;

      const tx = await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.not.be.fulfilled;

    });

  });

});
