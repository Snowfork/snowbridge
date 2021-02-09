const ETHApp = artifacts.require("ETHApp");

const BigNumber = web3.BigNumber;

const { confirmUnlock, confirmMessageDispatched, deployAppContractWithChannels, ChannelId, buildCommitment } = require("./helpers");
const { lockupETH } = require('./test_eth_app');

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const ethers = require("ethers");

contract("IncentivizedInboundChannel", function (accounts) {
  // Accounts
  const userOne = accounts[1];
  const userTwo = accounts[2];
  const userThree = accounts[3];

  describe("newParachainCommitment", function () {
    beforeEach(async function () {
      [channels, this.ethApp] = await deployAppContractWithChannels(ETHApp);

      this.incentivizedInboundChannel = channels.incentivized.inbound;

      // Prepare ETHApp with some liquidity for testing
      this.POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"
      await lockupETH(this.ethApp, userOne, this.POLKADOT_ADDRESS, 5000, ChannelId.Incentivized)
    });

    it("should accept a new valid commitment and dispatch the contained messages to their respective destinations", async function () {
      const abi = this.ethApp.abi;
      const iChannel = new ethers.utils.Interface(abi);
      const polkadotSender = ethers.utils.formatBytes32String('fake-polkadot-address');

      const unlockFragment = iChannel.functions['unlock(bytes32,address,uint256)'];

      // Construct first message
      const payloadOne = iChannel.encodeFunctionData(unlockFragment, [polkadotSender, userTwo, 2]);
      const messageOne = {
        target: this.ethApp.address,
        nonce: 1,
        payload: payloadOne
      }

      // Construct second message
      const payloadTwo = iChannel.encodeFunctionData(unlockFragment, [polkadotSender, userThree, 5]);
      const messageTwo = {
        target: this.ethApp.address,
        nonce: 2,
        payload: payloadTwo
      }

      // Construct third message
      const payloadThree = iChannel.encodeFunctionData(unlockFragment, [polkadotSender, userTwo, 7]);
      const messageThree = {
        target: this.ethApp.address,
        nonce: 3,
        payload: payloadThree
      }

      // Construct third message
      const payloadFour = iChannel.encodeFunctionData(unlockFragment, [polkadotSender, userThree, 9]);
      const messageFour = {
        target: this.ethApp.address,
        nonce: 4,
        payload: payloadFour
      }

       // Construct commitment hash from messages
       const messages = [messageOne, messageTwo, messageThree, messageFour];
       const commitment = buildCommitment(messages);

      // Send commitment including one payload for the ETHApp
      const tx = await this.incentivizedInboundChannel.submit(
        messages,
        commitment,
        { from: userOne }
      )

      // Confirm ETHApp and IncentivizedInboundChannel processed messages correctly
      const firstRawUnlockLog = tx.receipt.rawLogs[0];
      confirmUnlock(firstRawUnlockLog, this.ethApp.address, userTwo, 2);
      const firstMessageDispatchedLog = tx.receipt.rawLogs[1];
      confirmMessageDispatched(firstMessageDispatchedLog, 1, true);

      const secondRawUnlockLog = tx.receipt.rawLogs[2];
      confirmUnlock(secondRawUnlockLog, this.ethApp.address, userThree, 5);
      const secondMessageDispatchedLog = tx.receipt.rawLogs[3];
      confirmMessageDispatched(secondMessageDispatchedLog, 2, true);

      const thirdRawUnlockLog = tx.receipt.rawLogs[4];
      confirmUnlock(thirdRawUnlockLog, this.ethApp.address, userTwo, 7);
      const thirdMessageDispatchedLog = tx.receipt.rawLogs[5];
      confirmMessageDispatched(thirdMessageDispatchedLog, 3, true);

      const fourthRawUnlockLog = tx.receipt.rawLogs[6];
      confirmUnlock(fourthRawUnlockLog, this.ethApp.address, userThree, 9);
      const fourthMessageDispatchedLog = tx.receipt.rawLogs[7];
      confirmMessageDispatched(fourthMessageDispatchedLog, 4, true);

    });
  });
});
