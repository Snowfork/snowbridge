const IncentivizedReceiveChannel = artifacts.require("IncentivizedReceiveChannel");
const ETHApp = artifacts.require("ETHApp");

const Web3Utils = require("web3-utils");
const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const ethers = require("ethers");

contract("IncentivizedReceiveChannel", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.incentivizedReceiveChannel = await IncentivizedReceiveChannel.new();
    });

    it("should deploy and initialize the IncentivizedReceiveChannel contract", async function () {
      this.incentivizedReceiveChannel.should.exist;
    });
  });

  describe("newParachainCommitment", function () {
    beforeEach(async function () {
      this.incentivizedReceiveChannel = await IncentivizedReceiveChannel.new();
      this.ethApp = await ETHApp.new();
    });

    it("should accept a new valid commitment and dispatch the contained messages to their respective destinations", async function () {

      const testMessage = {
        nonce: 0,
        senderApplicationId: 'eth-app',
        targetApplicationAddress: this.ethApp.address,
        payload: ethers.utils.formatBytes32String("fake-payload")
      }

      const { logs } = await this.incentivizedReceiveChannel.newParachainCommitment(
        { commitmentHash: ethers.utils.formatBytes32String("fake-hash") },
        { messages: [testMessage] },
        5,
        ethers.utils.formatBytes32String("fake-proof1"),
        ethers.utils.formatBytes32String("fake-proof2"),
        { from: userOne }
      ).should.be.fulfilled;

    });
  });

});
