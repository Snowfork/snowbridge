const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = require('bignumber.js');

const { confirmChannelSend } = require("./helpers");

const channelContracts = {
  basic: {
    inbound: artifacts.require("BasicInboundChannel"),
    outbound: artifacts.require("BasicOutboundChannel"),
  },
  incentivized: {
    inbound: artifacts.require("IncentivizedInboundChannel"),
    outbound: artifacts.require("IncentivizedOutboundChannel"),
  },
}

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("ERC20App", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];

  // Constants
  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  describe("deposits", function () {
    beforeEach(async function () {
      this.channels = {
        basic: {
          inbound: await channelContracts.basic.inbound.new(),
          outbound: await channelContracts.basic.outbound.new(),
        },
        incentivized: {
          inbound: await channelContracts.incentivized.inbound.new(),
          outbound: await channelContracts.incentivized.outbound.new(),
        },
      };

      this.app = await ERC20App.new(
        {
          inbound: this.channels.basic.inbound.address,
          outbound: this.channels.basic.outbound.address,
        },
        {
          inbound: this.channels.incentivized.inbound.address,
          outbound: this.channels.incentivized.outbound.address,
        },
      );
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
      const beforeTotalERC20 = BigNumber(await this.app.balances(this.token.address));
      const beforeTestTokenBalance = BigNumber(await this.token.balanceOf(this.app.address));
      const beforeUserBalance = BigNumber(await this.token.balanceOf(userOne));

      // Prepare transaction parameters
      const amount = 100;
      const recipient = Buffer.from(POLKADOT_ADDRESS.replace(/^0x/, ""), "hex");

      // Approve tokens to contract
      await this.token.approve(this.app.address, amount, {
        from: userOne
      }).should.be.fulfilled;

      // Deposit ERC20 tokens to the contract and get the logs of the transaction
      const { logs } = await this.app.lock(
        this.token.address,
        recipient,
        amount,
        1,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = logs.find(
        e => e.event === "Locked"
      );

      // Check event fields
      event.args.token.should.be.equal(this.token.address);
      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.amount).should.be.bignumber.equal(BigNumber(amount));

      //Get the user and ERC20App token balance after deposit
      const afterTestTokenBalance = BigNumber(await this.token.balanceOf(this.app.address));
      const afterUserBalance = BigNumber(await this.token.balanceOf(userOne));

      //Confirm that the contract's token balance has increased
      afterTestTokenBalance.should.be.bignumber.equal(beforeTestTokenBalance + amount);
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance - amount);

      // Confirm contract's locked ERC20 counter has increased by amount locked
      const afterTotalERC20 = await this.app.balances(this.token.address);
      BigNumber(afterTotalERC20).should.be.bignumber.equal(beforeTotalERC20.plus(BigNumber(amount)));
    });

    it("should send lock payload to the correct channels", async function () {
      // Prepare transaction parameters
      const amount = 100;
      const recipient = Buffer.from(POLKADOT_ADDRESS.replace(/^0x/, ""), "hex");

      // Approve tokens to contract
      await this.token.approve(this.app.address, amount * 2, {
        from: userOne
      }).should.be.fulfilled;

      let tx;

      // Deposit ERC20 tokens for incentivized channel
      tx = await this.app.lock(
        this.token.address,
        recipient,
        amount,
        0,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      // Confirm payload submitted to incentivized channel
      confirmChannelSend(
        tx.receipt.rawLogs[3],
        this.channels.basic.outbound.address,
        this.app.address,
        0
      );

      // Deposit ERC20 tokens for unincentivized channel
      tx = await this.app.lock(
        this.token.address,
        recipient,
        amount,
        1,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      // Confirm payload submitted to basic channel
      confirmChannelSend(
        tx.receipt.rawLogs[3],
        this.channels.incentivized.outbound.address,
        this.app.address,
        0
      )
    });
  });

  // FIXME!
  describe.skip("handle received messages", function () {

    before(async function () {
      const basicSendChannel = await BasicOutboundChannel.new();
      const incentivizedSendChannel = await IncentivizedOutboundChannel.new();
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
