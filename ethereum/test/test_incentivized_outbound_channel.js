const IncentivizedOutboundChannel = artifacts.require("IncentivizedOutboundChannel");
const MockContract = artifacts.require("./test/MockContract.sol")

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
  const owner = accounts[0];
  const userOne = accounts[1];
  const userTwo = accounts[2];
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");
  const relayFee = process.env.RELAY_FEE;
  const feeController = process.env.FEE_CONTROLLER

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new(relayFee, feeController);
    });
  });

  describe("send", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new(relayFee, feeController);
      const mock = await MockContract.new()
      await mock.givenAnyReturnBool(true)
      this.channel.setDOTApp(mock.address, { from: owner })
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
      this.channel = await IncentivizedOutboundChannel.new(relayFee, feeController);
    });

    it("should let feeController set relayFee", async function () {
      await this.channel.setRelayFee(20000, {from: feeController}).should.be.fulfilled;
    })

    it("should not allow non feeController caller to set relayFee", async function () {
      await this.channel.setRelayFee(20000, {from: userTwo}).should.be.rejected;
    })
  })

  describe("feeController", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new(relayFee, feeController);
    });

    it("should let feeController set itself", async function () {
      await this.channel.setFeeController(userOne, {from: feeController}).should.be.fulfilled;
    })

    it("should not allow non feeController caller to change feeController", async function () {
      await this.channel.setFeeController(userOne, {from: userTwo}).should.be.rejected;
    })
  })
});
