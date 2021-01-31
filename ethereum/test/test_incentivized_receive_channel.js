const IncentivizedReceiveChannel = artifacts.require("IncentivizedReceiveChannel");
const ETHApp = artifacts.require("ETHApp");
const IncentivizedSendChannel = artifacts.require("IncentivizedSendChannel");

const Web3Utils = require("web3-utils");
const BigNumber = web3.BigNumber;

const { confirmUnlock, confirmMessageDelivered, buildCommitment } = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const ethers = require("ethers");

contract("IncentivizedReceiveChannel", function (accounts) {
  // Accounts
  const userOne = accounts[1];
  const userTwo = accounts[2];
  const userThree = accounts[3];

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


    it("should accept a new valid commitment and dispatch the contained messages to their respective destinations", async function () {
      const abi = this.ethApp.abi;
      const iChannel = new ethers.utils.Interface(abi);

      // Construct first message
      const payloadOne = iChannel.functions.unlockETH.encode([userTwo, 2]);
      const messageOne = {
        nonce: 1,
        senderApplicationId: 'eth-app',
        targetApplicationAddress: this.ethApp.address,
        payload: payloadOne
      }

      // Construct second message
      const payloadTwo = iChannel.functions.unlockETH.encode([userThree, 5]);
      const messageTwo = {
        nonce: 2,
        senderApplicationId: 'eth-app',
        targetApplicationAddress: this.ethApp.address,
        payload: payloadTwo
      }

       // Construct commitment hash from messages
      const messages = [messageOne, messageTwo];
      const commitment = buildCommitment(messages);

      // Send commitment including one payload for the ETHApp
      const tx = await this.incentivizedReceiveChannel.newParachainCommitment(
        { commitmentHash: commitment },
        { messages: messages },
        5,
        ethers.utils.formatBytes32String("fake-proof1"),
        ethers.utils.formatBytes32String("fake-proof2"),
        { from: userOne }
      )

      // Confirm ETHApp and IncentivizedReceiveChannel processed messages correctly
      const firstRawUnlockLog = tx.receipt.rawLogs[0];
      confirmUnlock(firstRawUnlockLog, this.ethApp.address, userTwo, 2);
      const firstMessageDeliveredLog = tx.receipt.rawLogs[1];
      confirmMessageDelivered(firstMessageDeliveredLog, 1, true);

      const secondRawUnlockLog = tx.receipt.rawLogs[2];
      confirmUnlock(secondRawUnlockLog, this.ethApp.address, userThree, 5);
      const secondMessageDeliveredLog = tx.receipt.rawLogs[3];
      confirmMessageDelivered(secondMessageDeliveredLog, 2, true);
    });
  });
});
