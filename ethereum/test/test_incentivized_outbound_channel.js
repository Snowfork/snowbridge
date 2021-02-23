const IncentivizedOutboundChannel = artifacts.require("IncentivizedOutboundChannel");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = web3.BigNumber;

const { confirmChannelSend } = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("IncentivizedOutboundChannel", function (accounts) {
  // Accounts
  const userOne = accounts[1];
  const userTwo = accounts[2];
  const userThree = accounts[3];
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new(1000, userTwo);
    });
  });

  describe("send", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new(1000, userTwo);
    });

    it("should send messages out with the correct event and fields", async function () {
      const tx = await this.channel.submit(
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx.receipt.rawLogs[0];
      confirmChannelSend(rawLog, this.channel.address, userOne, 1, testPayload)
    });

    it("should increment nonces correctly", async function () {
      const tx = await this.channel.submit(
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const tx2 = await this.channel.submit(
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const tx3 = await this.channel.submit(
        testPayload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx3.receipt.rawLogs[0];
      confirmChannelSend(rawLog, this.channel.address, userOne, 3, testPayload)
    });

  });

  describe("relayFee", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new(1000, userTwo);
    });

    it("should let feeController set relayFee", async function () {
      await this.channel.setRelayFee(20000, {from: userTwo}).should.be.fulfilled;
    })

    it("should not allow non feeController caller to set relayFee", async function () {
      await this.channel.setRelayFee(20000, {from: userThree}).should.be.rejected;
    })
  })

  describe("feeController", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new(1000, userTwo);
    });

    it("should let feeController set itself", async function () {
      await this.channel.setFeeController(userThree, {from: userTwo}).should.be.fulfilled;
    })

    it("should not allow non feeController caller to change feeController", async function () {
      await this.channel.setFeeController(userThree, {from: userOne}).should.be.rejected;
    })
  })
});
