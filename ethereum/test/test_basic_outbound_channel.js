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
  const payload = ethers.utils.formatBytes32String("arbitrary-payload");

  describe("submit messages", function () {
    beforeEach(async function () {
      this.channel = await BasicOutboundChannel.new();
    });

    it("should send messages out with the correct event and fields", async function () {
      const tx = await this.channel.submit(
        payload,
        { from: userOne, value: 0 }
      ).should.be.fulfilled;

      const rawLog = tx.receipt.rawLogs[0];
      confirmChannelSend(rawLog, this.channel.address, userOne, 1, payload)
    });

  });

});
