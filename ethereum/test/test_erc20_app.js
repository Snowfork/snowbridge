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

      const nonce = Number(await this.erc20App.nonce());
      nonce.should.be.bignumber.equal(0);
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
      const beforeNonce = Number(await this.erc20App.nonce());
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
          e => e.event === "AppEvent"
      );

      Number(appEvent.args._tag).should.be.bignumber.equal(1);

      // Clean data by removing '0x' prefix
      const data = appEvent.args._data.slice(2, appEvent.args._data.length);

      // Sender's Ethereum address
      let start = 0;
      let end = BYTES32_LENGTH;
      const expectedSender = Web3Utils.padLeft(userOne.toLowerCase().slice(2, userOne.length), BYTES32_LENGTH);
      data.slice(start, end).should.be.equal(expectedSender);

      // Move forward one byte slice
      start = end;
      end = end + BYTES32_LENGTH;

      // ERC20 token address
      const expectedTokenAddr =  Web3Utils.padLeft(this.token.address.toLowerCase().slice(2, this.token.address.length), BYTES32_LENGTH);
      start = end;
      end = end + BYTES32_LENGTH;
      data.slice(start, end).should.be.equal(expectedTokenAddr);

      // Uint256 amount
      const encodedAmount = Web3Utils.padLeft(Web3Utils.numberToHex(amount), 64);
      const expectedAmount = encodedAmount.slice(2, encodedAmount.length);
      start = end;
      end = end + BYTES32_LENGTH;
      data.slice(start, end).should.be.equal(expectedAmount);

      // Uint256 nonce
      const encodedNonce = Web3Utils.padLeft(Web3Utils.numberToHex(beforeNonce+1), 64);
      const expectedNonce = encodedNonce.slice(2, encodedNonce.length);
      start = end;
      end = end + BYTES32_LENGTH;
      data.slice(start, end).should.be.equal(expectedNonce);

      //Get the user and ERC20App token balance after deposit
      const afterTestTokenBalance = Number(await this.token.balanceOf(this.erc20App.address));
      const afterUserBalance = Number(await this.token.balanceOf(userOne));

      //Confirm that the contract's token balance has increased
      afterTestTokenBalance.should.be.bignumber.equal(beforeTestTokenBalance + amount);
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance - amount);

      // Confirm contract's locked ERC20 counter has increased by amount locked
      const afterTotalERC20 = await this.erc20App.totalTokens(this.token.address);
      Number(afterTotalERC20).should.be.bignumber.equal(beforeTotalERC20+amount);

      // Confirm contract's nonce has been incremented
      const nonce = Number(await this.erc20App.nonce());
      nonce.should.be.bignumber.equal(beforeNonce+1);
    });
  });

  describe("unlocks", function () {

    before(async function () {
        this.erc20App = await ERC20App.new();

        // Set up an ERC20 token for testing token deposits
        this.symbol = "TEST";
        this.token = await TestToken.new(100000, "Test Token", this.symbol);

        // Load user account with 'TEST' ERC20 tokens
        await this.token.transfer(userOne, 10000, {
            from: owner
        }).should.be.fulfilled;

        // Prepare transaction parameters
        const lockAmount = 5000;
        const substrateRecipient = Buffer.from(POLKADOT_ADDRESS, "hex");

        // Approve tokens to contract
        await this.token.approve(this.erc20App.address, lockAmount, {
          from: userOne
        }).should.be.fulfilled;

        // Send ERC20 tokens to a substrate recipient
        await this.erc20App.sendERC20(
          substrateRecipient,
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
      const encodedData = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27dcffeaaf7681c89285d65cfbe808b80e502696573dda6327139485221633a1fcd65f4ac932e60a2e13412000000000000000000000000000000000000000000000000000000000000";
      // Decoded data
      const decodedSender = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
      const decodedRecipient = "0xCfFEAAf7681c89285D65CfbE808b80e502696573";
      const decodedTokenAddr = this.token.address;
      const decodedAmount = 4660;

      // Load initial state
      const beforeTotalERC20 = Number(await this.erc20App.totalTokens(this.token.address));
      const beforeTestTokenBalance = Number(await this.token.balanceOf(this.erc20App.address));
      const beforeUserBalance = Number(await this.token.balanceOf(decodedRecipient));

     const { logs } = await this.erc20App.unlockERC20(encodedData).should.be.fulfilled;

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
