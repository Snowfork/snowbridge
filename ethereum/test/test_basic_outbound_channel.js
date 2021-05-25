const BasicOutboundChannel = artifacts.require("BasicOutboundChannel");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

describe("BasicOutboundChannel", function () {
  let appAddress;
  let origin;
  const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");
  const iface = new ethers.utils.Interface(BasicOutboundChannel.abi);

  before(async function() {
    accounts = await web3.eth.getAccounts();
    appAddress = accounts[1];
    origin = accounts[2];
  });

  describe("send", function () {
    beforeEach(async function () {
      this.channel = await BasicOutboundChannel.new();
    });

    it("should send messages out with the correct event and fields", async function () {
      const tx = await this.channel.submit(
        origin,
        testPayload,
        { from: appAddress, value: 0 }
      ).should.be.fulfilled;

      const log = tx.receipt.rawLogs[0];
      const event = iface.decodeEventLog('Message(address,uint64,bytes)', log.data, log.topics);

      log.address.should.be.equal(this.channel.address);
      event.source.should.be.equal(appAddress);
      event.nonce.eq(ethers.BigNumber.from(1)).should.be.true;
      event.payload.should.be.equal(testPayload)
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

      const log = tx3.receipt.rawLogs[0];
      const event = iface.decodeEventLog('Message(address,uint64,bytes)', log.data, log.topics);
      event.nonce.eq(ethers.BigNumber.from(3)).should.be.true;
    });

  });

});
