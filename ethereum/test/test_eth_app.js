const ETHApp = artifacts.require("ETHApp");
const BasicSendChannel = artifacts.require("BasicSendChannel");
const IncentivizedSendChannel = artifacts.require("IncentivizedSendChannel");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = web3.BigNumber;

const { confirmChannelSend } = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("EthApp", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];

  // Constants
  const POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"
  const BYTES32_LENGTH = 64;

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      const basicSendChannel = await BasicSendChannel.new();
      const incentivizedSendChannel = await IncentivizedSendChannel.new();
      this.ethApp = await ETHApp.new(basicSendChannel.address, incentivizedSendChannel.address);
    });

    it("should deploy and initialize the ETHApp contract", async function () {
      this.ethApp.should.exist;
    });
  });

  describe("deposits", function () {
    beforeEach(async function () {
      this.basicSendChannel = await BasicSendChannel.new();
      this.incentivizedSendChannel = await IncentivizedSendChannel.new();
      this.ethApp = await ETHApp.new(this.basicSendChannel.address, this.incentivizedSendChannel.address);
    });

    it("should support Ethereum deposits", async function () {
      // Load initial contract state
      const beforeTotalETH = Number(await this.ethApp.totalETH());

      // Prepare transaction parameters
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");
      const weiAmount = web3.utils.toWei("0.25", "ether");

      // Deposit Ethereum to the contract and get the logs of the transaction
      const tx = await this.ethApp.sendETH(
        recipient,
        true,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;

      const logs = tx.logs;

      // Confirm app event emitted with expected values
      const appEvent = logs.find(
        e => e.event === "Locked"
      );

      appEvent.args._sender.should.be.equal(userOne);
      const expectedRecipient = Web3Utils.padRight(Web3Utils.toHex(recipient).toLowerCase(), BYTES32_LENGTH);
      appEvent.args._recipient.should.be.equal(expectedRecipient);
      Number(appEvent.args._amount).should.be.bignumber.equal(weiAmount);

      // Confirm contract's Ethereum balance has increased
      const contractBalanceWei = await web3.eth.getBalance(this.ethApp.address);
      const contractBalance = Web3Utils.fromWei(contractBalanceWei, "ether");
      contractBalance.should.be.bignumber.equal(Web3Utils.fromWei(weiAmount, "ether"));

      // Confirm contract's locked Ethereum counter has increased by amount locked
      const afterTotalETH = await this.ethApp.totalETH();
      Number(afterTotalETH).should.be.bignumber.equal(beforeTotalETH + Number(weiAmount));
    });

    it("should send lock payload to the correct channels", async function () {
      // Prepare transaction parameters
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");
      const weiAmount = web3.utils.toWei("0.25", "ether");
      const expectedPayload = web3.eth.abi.encodeParameters(['address', 'bytes32', 'uint256'], [userOne, ethers.utils.formatBytes32String(recipient.toString()), weiAmount]);

      // Deposit Ethereum to the contract via incentivized channel
      const tx_incentivized = await this.ethApp.sendETH(
        recipient,
        true,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;


      // Confirm payload submitted to incentivized channel
      confirmChannelSend(tx_incentivized.receipt.rawLogs[1], this.incentivizedSendChannel.address, this.ethApp.address, "eth-app", expectedPayload)

      // Deposit Ethereum to the contract via basic channel
      const tx_basic = await this.ethApp.sendETH(
        recipient,
        false,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;

      // Confirm payload submitted to basic channel
      confirmChannelSend(tx_basic.receipt.rawLogs[1], this.basicSendChannel.address, this.ethApp.address, "eth-app", expectedPayload)
    });
  });


  describe("send ETH correctly (for unlocks)", function () {

    before(async function () {

      const basicSendChannel = await BasicSendChannel.new();
      const incentivizedSendChannel = await IncentivizedSendChannel.new();
      this.ethApp = await ETHApp.new(basicSendChannel.address, incentivizedSendChannel.address);

      await this.ethApp.register(owner);

      // Prepare transaction parameters
      const lockAmountWei = 5000;
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

    it("should support ETH unlocks", async function () {
      const polkadotSender = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
      const recipient = "0xCfFEAAf7681c89285D65CfbE808b80e502696573";
      const amount = 4660;

      // Load initial state
      const beforeTotalETH = Number(await this.ethApp.totalETH());
      const beforeContractBalanceWei = await web3.eth.getBalance(this.ethApp.address);
      const beforeUserBalanceWei = await web3.eth.getBalance(recipient);

      const { logs } = await this.ethApp.unlockETH(
        polkadotSender,
        recipient,
        amount
      ).should.be.fulfilled;

      // Confirm unlock event emitted with expected values
      const unlockedEvent = logs.find(
        e => e.event === "Unlocked"
      );

      unlockedEvent.args._polkadotSender.should.be.equal(polkadotSender);
      unlockedEvent.args._recipient.should.be.equal(recipient);
      Number(unlockedEvent.args._amount).should.be.bignumber.equal(amount);

      // Get the user and ETHApp's Ethereum balance after unlock
      const afterContractBalanceWei = await web3.eth.getBalance(this.ethApp.address);
      const afterUserBalanceWei = await web3.eth.getBalance(recipient);

      // Confirm user's balance increased and contract's Ethereum balance has decreased
      afterUserBalanceWei.should.be.bignumber.equal(parseInt(beforeUserBalanceWei) + amount);
      afterContractBalanceWei.should.be.bignumber.equal(beforeContractBalanceWei - amount);

      // Confirm contract's locked Ethereum counter has decreased by amount unlocked
      const afterTotalETH = await this.ethApp.totalETH();
      Number(afterTotalETH).should.be.bignumber.equal(beforeTotalETH - Number(amount));
    });
  });
});
