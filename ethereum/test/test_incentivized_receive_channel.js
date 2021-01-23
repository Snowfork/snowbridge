const IncentivizedReceiveChannel = artifacts.require("IncentivizedReceiveChannel");
const ETHApp = artifacts.require("ETHApp");
const IncentivizedSendChannel = artifacts.require("IncentivizedSendChannel");

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

   xit("should deploy and initialize the IncentivizedReceiveChannel contract", async function () {
      this.incentivizedReceiveChannel.should.exist;
    });
  });

  describe("newParachainCommitment", function () {
    beforeEach(async function () {
      const incentivizedSendChannel = await IncentivizedSendChannel.new();
      this.ethApp = await ETHApp.new(incentivizedSendChannel.address, incentivizedSendChannel.address);

      this.incentivizedReceiveChannel = await IncentivizedReceiveChannel.new();
      await this.ethApp.register(this.incentivizedReceiveChannel.address);

      // Prepare ETHApp with some liquidity for testing
      const lockAmountWei = 5000;
      const POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"
      const substrateRecipient = Buffer.from(POLKADOT_ADDRESS, "hex");

      // Send to a substrate recipient to load contract with unlockable ETH
      await this.ethApp.sendETH(
        substrateRecipient,
        true,
        {
          from: userOne,
          value: lockAmountWei
        }
      ).should.be.fulfilled;

    });

    it("inline assembly call should work", async function () {
      const userTwoBalanceBefore = parseInt(await web3.eth.getBalance(userTwo))

      const totalEthBefore = (await this.ethApp.totalETH.call()).toNumber();

      var abi = this.ethApp.abi
      var iChannel = new ethers.utils.Interface(abi)
      var calldata = iChannel.functions.unlockETH.encode([userTwo, 2]);
      const tx = await this.incentivizedReceiveChannel.test(calldata, this.ethApp.address)

      console.log({rawLogs: tx.receipt.rawLogs});

      const totalEthAfter = (await this.ethApp.totalETH.call()).toNumber();

      const userTwoBalanceAfter = parseInt(await web3.eth.getBalance(userTwo));

      totalEthAfter.should.be.equal(totalEthBefore - 2);
      userTwoBalanceAfter.should.be.equal(userTwoBalanceAfter + 2);
    });


    xit("should accept a new valid commitment and dispatch the contained messages to their respective destinations", async function () {
      const recipient = userTwo;
      const amount = 1;

      const abi = this.ethApp.abi
      const iChannel = new ethers.utils.Interface(abi)
      const testPayload = iChannel.functions.unlockETH.encode([userTwo, 100]);

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

      // Confirm Message delivered correctly
      const deliveryEvent = tx.logs.find(
        e => e.event === "MessageDelivered"
      );

      expect(deliveryEvent).to.not.be.equal(undefined);
      // console.log("deliveryEvent:", deliveryEvent);
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
