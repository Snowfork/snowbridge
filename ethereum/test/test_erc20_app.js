const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");

const Web3Utils = require("web3-utils");
const BigNumber = web3.BigNumber;

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
      this.erc20App = await ERC20App.new();
    });

    it("should deploy and initialize the ERC20App contract", async function () {
      this.erc20App.should.exist;
    });
  });

  describe("deposits", function () {
    beforeEach(async function () {
      this.erc20App = await ERC20App.new();
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
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      // Confirm app event emitted with expected values
      const appEvent = logs.find(
          e => e.event === "AppTransfer"
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
      Number(afterTotalERC20).should.be.bignumber.equal(beforeTotalERC20+amount);
    });
  });

  describe("handle received messages", function () {

    before(async function () {
        this.erc20App = await ERC20App.new();
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
      afterTestTokenBalance.should.be.bignumber.equal(beforeTestTokenBalance - decodedAmount);
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance + decodedAmount);

      // Confirm contract's locked ERC20 counter has decreased by amount locked
      const afterTotalERC20 = await this.erc20App.totalTokens(this.token.address);
      Number(afterTotalERC20).should.be.bignumber.equal(beforeTotalERC20 - decodedAmount);
    });
  });

});
