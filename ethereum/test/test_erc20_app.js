const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");
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

contract("ERC20App", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];

  // Constants
  const POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"
  const BYTES32_LENGTH = 64;

  describe("initialization and deployment", function () {
    beforeEach(async function () {
      const basicSendChannel = await BasicSendChannel.new();
      const incentivizedSendChannel = await IncentivizedSendChannel.new();
      this.erc20App = await ERC20App.new(basicSendChannel.address, incentivizedSendChannel.address);
    });

    it("should deploy and initialize the ERC20App contract", async function () {
      this.erc20App.should.exist;
    });
  });

  describe("deposits", function () {
    beforeEach(async function () {
      this.basicSendChannel = await BasicSendChannel.new();
      this.incentivizedSendChannel = await IncentivizedSendChannel.new();
      this.erc20App = await ERC20App.new(this.basicSendChannel.address, this.incentivizedSendChannel.address);
    });

    // Set up an ERC20 token for testing token deposits
    before(async function () {
      this.symbol = "TEST";
      this.token = await TestToken.new(100000, "Test Token", this.symbol);

      // Load user account with 'TEST' ERC20 tokens
      await this.token.transfer(userOne, 1000, {
        from: owner
      }).should.be.fulfilled;
    });

    it("should support ERC20 deposits", async function () {
      // Load initial state
      const beforeTotalERC20 = Number(await this.erc20App.totalTokens(this.token.address));
      const beforeTestTokenBalance = Number(await this.token.balanceOf(this.erc20App.address));
      const beforeUserBalance = Number(await this.token.balanceOf(userOne));

      // Prepare transaction parameters
      const amount = 100;
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");

      // Approve tokens to contract
      await this.token.approve(this.erc20App.address, amount, {
        from: userOne
      }).should.be.fulfilled;

      // Deposit ERC20 tokens to the contract and get the logs of the transaction
      const { logs } = await this.erc20App.sendERC20(
        recipient,
        this.token.address,
        amount,
        true,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      // Confirm app event emitted with expected values
      const appEvent = logs.find(
        e => e.event === "Locked"
      );

      // Check event fields
      appEvent.args._sender.should.be.equal(userOne);
      const expectedRecipient = Web3Utils.padRight(Web3Utils.toHex(recipient).toLowerCase(), BYTES32_LENGTH);
      appEvent.args._recipient.should.be.equal(expectedRecipient);
      appEvent.args._token.should.be.equal(this.token.address);
      Number(appEvent.args._amount).should.be.bignumber.equal(amount);

      //Get the user and ERC20App token balance after deposit
      const afterTestTokenBalance = Number(await this.token.balanceOf(this.erc20App.address));
      const afterUserBalance = Number(await this.token.balanceOf(userOne));

      //Confirm that the contract's token balance has increased
      afterTestTokenBalance.should.be.bignumber.equal(beforeTestTokenBalance + amount);
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance - amount);

      // Confirm contract's locked ERC20 counter has increased by amount locked
      const afterTotalERC20 = await this.erc20App.totalTokens(this.token.address);
      Number(afterTotalERC20).should.be.bignumber.equal(beforeTotalERC20 + amount);
    });

    it("should send lock payload to the correct channels", async function () {
      // Prepare transaction parameters
      const amount = 100;
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");
      const expectedPayload = web3.eth.abi.encodeParameters(
        ['address', 'bytes32', 'address', 'uint256'],
        [userOne, ethers.utils.formatBytes32String(recipient.toString()), this.token.address, amount]
      );

      // Approve tokens to contract
      await this.token.approve(this.erc20App.address, amount * 2, {
        from: userOne
      }).should.be.fulfilled;

      // Deposit ERC20 tokens for incentivized channel
      const tx_incentivized = await this.erc20App.sendERC20(
        recipient,
        this.token.address,
        amount,
        true,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      // Confirm payload submitted to incentivized channel
      confirmChannelSend(tx_incentivized.receipt.rawLogs[3], this.incentivizedSendChannel.address, this.erc20App.address, "erc20-app", expectedPayload)

      // Deposit ERC20 tokens for unincentivized channel
      const tx_basic = await this.erc20App.sendERC20(
        recipient,
        this.token.address,
        amount,
        false,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      // Confirm payload submitted to basic channel
      confirmChannelSend(tx_basic.receipt.rawLogs[3], this.basicSendChannel.address, this.erc20App.address, "erc20-app", expectedPayload)
    });
  });

  describe("handle received messages", function () {

    before(async function () {
      const basicSendChannel = await BasicSendChannel.new();
      const incentivizedSendChannel = await IncentivizedSendChannel.new();
      this.erc20App = await ERC20App.new(basicSendChannel.address, incentivizedSendChannel.address);
      await this.erc20App.register(owner);

      // Set up an ERC20 token for testing token deposits
      this.symbol = "TEST";
      this.token = await TestToken.new(100000, "Test Token", this.symbol);

      // Load user account with 'TEST' ERC20 tokens
      await this.token.transfer(userOne, 10000, {
        from: owner
      }).should.be.fulfilled;

      // Prepare transaction parameters
      const lockAmount = 10000;
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");

      // Approve tokens to contract
      await this.token.approve(this.erc20App.address, lockAmount, {
        from: userOne
      }).should.be.fulfilled;

      // Deposit ERC20 tokens to the contract (so the app has funds to be unlocked)
      await this.erc20App.sendERC20(
        recipient,
        this.token.address,
        lockAmount,
        true,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;
    });

    it("should support ERC20 unlocks", async function () {
      // Encoded data
      const encodedTokenAddress = this.token.address.slice(2, this.token.address.length);
      const encodedData = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27dcffeaaf7681c89285d65cfbe808b80e502696573" + encodedTokenAddress + "3412000000000000000000000000000000000000000000000000000000000000"
      // Decoded data
      const decodedSender = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
      const decodedRecipient = "0xCfFEAAf7681c89285D65CfbE808b80e502696573";
      const decodedTokenAddr = this.token.address;
      const decodedAmount = 4660;

      // Load initial state
      const beforeTotalERC20 = Number(await this.erc20App.totalTokens(this.token.address));
      const beforeTestTokenBalance = Number(await this.token.balanceOf(this.erc20App.address));
      const beforeUserBalance = Number(await this.token.balanceOf(decodedRecipient));

      const { logs } = await this.erc20App.handle(encodedData).should.be.fulfilled;

      // Confirm unlock event emitted with expected values
      const unlockEvent = logs.find(
        e => e.event === "Unlock"
      );

      unlockEvent.args._sender.should.be.equal(decodedSender);
      unlockEvent.args._recipient.should.be.equal(decodedRecipient);
      unlockEvent.args._token.should.be.equal(decodedTokenAddr);
      Number(unlockEvent.args._amount).should.be.bignumber.equal(decodedAmount);

      // Get the user and ERC20App token balance after unlock
      const afterTestTokenBalance = Number(await this.token.balanceOf(this.erc20App.address));
      const afterUserBalance = Number(await this.token.balanceOf(decodedRecipient));

      // Confirm that the user's token balance has increased
      afterTestTokenBalance.should.be.bignumber.equal(parseInt(beforeTestTokenBalance) - decodedAmount);
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance + decodedAmount);

      // Confirm contract's locked ERC20 counter has decreased by amount locked
      const afterTotalERC20 = await this.erc20App.totalTokens(this.token.address);
      Number(afterTotalERC20).should.be.bignumber.equal(beforeTotalERC20 - decodedAmount);
    });
  });

});
