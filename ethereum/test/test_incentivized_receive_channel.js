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
  const userTwo = accounts[2];

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
      this.ethApp = await ETHApp.new(this.incentivizedReceiveChannel.address, this.incentivizedReceiveChannel.address);
      await this.ethApp.register(this.incentivizedReceiveChannel.address);
    });

    it("should accept a new valid commitment and dispatch the contained messages to their respective destinations", async function () {

      const recipient = userTwo;
      const amount = 1;
      const testPayload = this.ethApp.contract.methods.unlockETH(recipient, amount).encodeABI();

      const testMessage = {
        nonce: 1,
        senderApplicationId: 'eth-app',
        targetApplicationAddress: this.ethApp.address,
        payload: testPayload
      }

      // Send commitment including one payload for the ETHApp
      const tx = await this.incentivizedReceiveChannel.newParachainCommitment(
        { commitmentHash: ethers.utils.formatBytes32String("fake-hash") },
        { messages: [testMessage] },
        5,
        ethers.utils.formatBytes32String("fake-proof1"),
        ethers.utils.formatBytes32String("fake-proof2"),
        { from: userOne }
      ).should.be.fulfilled;

      console.log(tx.logs);
      console.log(tx.logs);

      // Confirm Message delivered correctly
      const deliveryEvent = tx.logs.find(
        e => e.event === "MessageDelivered"
      );

      expect(deliveryEvent).to.not.be.equal(undefined);
      deliveryEvent.args.nonce.toNumber().should.be.equal(testMessage.nonce);
      deliveryEvent.args.result.should.be.equal(true);

      // Confirm ETHApp processed event correctly
      const appEvent = tx.logs.find(
        e => e.event === "Unlock"
      );

      expect(appEvent).to.not.be.equal(undefined);
      appEvent.args._recipient.should.be.equal(recipient);
      Number(appEvent.args._amount).should.be.bignumber.equal(amount);
    });
  });

});
