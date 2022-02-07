const ethers = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .should();
const IncentivizedOutboundChannel = artifacts.require("IncentivizedOutboundChannel");
const MockFeeSource = artifacts.require("MockFeeSource");

const {
  printTxPromiseGas
} = require("./helpers");

describe("IncentivizedOutboundChannel", function () {
  let accounts;
  let owner;
  let appAddress;
  let origin;
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");
  const iface = new ethers.utils.Interface(IncentivizedOutboundChannel.abi);

  before(async function () {
    accounts = await web3.eth.getAccounts();
    owner = accounts[0];
    appAddress = accounts[1];
    origin = accounts[2];
  });

  describe("send", function () {
    beforeEach(async function () {
      this.channel = await IncentivizedOutboundChannel.new();
      const feeSource = await MockFeeSource.new();
      await this.channel.initialize(owner, feeSource.address, [appAddress]).should.be.fulfilled;
    });

    it("should send messages out with the correct event and fields", async function () {
      const txPromise = this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;
      printTxPromiseGas(txPromise)
      const tx = await txPromise;

      const log = tx.receipt.rawLogs[0];
      const event = iface.decodeEventLog('Message(address,uint64,uint256,bytes)', log.data, log.topics);

      log.address.should.be.equal(this.channel.address);
      event.source.should.be.equal(appAddress);
      event.nonce.eq(ethers.BigNumber.from(1)).should.be.true;
      event.payload.should.be.equal(testPayload)
    });

    it("should increment nonces correctly", async function () {
      await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      const { receipt } = await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      const log = receipt.rawLogs[0];
      const event = iface.decodeEventLog('Message(address,uint64,uint256,bytes)', log.data, log.topics);
      event.nonce.eq(ethers.BigNumber.from(3)).should.be.true;
    });

    it("should not send message if user cannot pay fee", async function () {

      // Trigger our mock fee source to revert in burnFee.
      await this.channel.setFee(1024).should.be.fulfilled;

      await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.not.be.fulfilled;

    });

  });

});
