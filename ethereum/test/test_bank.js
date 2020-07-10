const Bank = artifacts.require("Bank");
const TestToken = artifacts.require("TestToken");

const Web3Utils = require("web3-utils");
const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("Bank", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];

  // Constants
  const NULL_ADDRESS = "0x0000000000000000000000000000000000000000";
  const POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"
  const ADDRESS_BYTE_LENGTH = 40;
  const RECIPIENT_BYTE_LENGTH = 96;
  const UINT256_BYTE_LENGTH = 64;

  describe("Bank contract deployment", function () {
    beforeEach(async function () {
      this.bank = await Bank.new();
    });

    it("should deploy and initialize the contract", async function () {
      this.bank.should.exist;

      const nonce = Number(await this.bank.nonce());
      nonce.should.be.bignumber.equal(0);
    });
  });

  describe("should support deposits", function () {
    beforeEach(async function () {
      this.bank = await Bank.new();
    });

    it("should support Ethereum deposits", async function () {
      // Load initial contract state
      const beforeTotalETH = Number(await this.bank.totalETH());
      const beforeNonce = Number(await this.bank.nonce());

      // Prepare transaction parameters
      const recipient = web3.utils.utf8ToHex(POLKADOT_ADDRESS);
      const weiAmount = web3.utils.toWei("0.25", "ether");
      
      // Deposit Ethereum to the contract and get the logs of the transaction
      const { logs } = await this.bank.sendETH(
        recipient,
        {from: userOne, value: weiAmount}
      ).should.be.fulfilled;
      
      // Confirm Deposit event emitted with expected values
      const eventDeposit = logs.find(
          e => e.event === "Deposit"
      );
      // Clean data by removing '0x' prefix
      const data = eventDeposit.args._data.slice(2, eventDeposit.args._data.length);

      // Sender's Ethereum address
      const expectedSender = userOne.toLowerCase().slice(2, userOne.length);
      let start = 0;
      let end = ADDRESS_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedSender);

      // Recipient's Polkadot address
      const expectedRecipient = recipient.slice(2, recipient.length);
      start = end;
      end = end + RECIPIENT_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedRecipient);

      // ERC20 token address
      const expectedTokenAddr = NULL_ADDRESS.slice(2, NULL_ADDRESS.length);
      start = end;
      end = end + ADDRESS_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedTokenAddr);

      // Uint256 amount
      const encodedAmount = Web3Utils.padLeft(Web3Utils.numberToHex(weiAmount), 64);
      const expectedAmount = encodedAmount.slice(2, encodedAmount.length);
      start = end;
      end = end + UINT256_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedAmount);

      // Uint256 nonce
      const encodedNonce = Web3Utils.padLeft(Web3Utils.numberToHex(beforeNonce+1), 64);
      const expectedNonce = encodedNonce.slice(2, encodedNonce.length);
      start = end;
      end = end + UINT256_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedNonce);
      
      // Confirm contract's Ethereum balance has increased
      const contractBalanceWei = await web3.eth.getBalance(this.bank.address);
      const contractBalance = Web3Utils.fromWei(contractBalanceWei, "ether");
      contractBalance.should.be.bignumber.equal(Web3Utils.fromWei(weiAmount, "ether"));

      // Confirm contract's locked Ethereum counter has increased by amount locked
      const afterTotalETH = await this.bank.totalETH();
      Number(afterTotalETH).should.be.bignumber.equal(beforeTotalETH+Number(weiAmount));

      // Confirm contract's nonce has been incremented
      const nonce = Number(await this.bank.nonce());
      nonce.should.be.bignumber.equal(beforeNonce+1);
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
      const beforeTotalERC20 = Number(await this.bank.totalTokens(this.token.address));
      const beforeNonce = Number(await this.bank.nonce());
      const beforeTestTokenBalance = Number(await this.token.balanceOf(this.bank.address));
      const beforeUserBalance = Number(await this.token.balanceOf(userOne));

      // Prepare transaction parameters
      const amount = 100;
      const recipient = web3.utils.utf8ToHex(POLKADOT_ADDRESS);
      
      // Approve tokens to contract
      await this.token.approve(this.bank.address, amount, {
        from: userOne
      }).should.be.fulfilled;

      // Deposit ERC20 tokens to the contract and get the logs of the transaction
      const { logs } = await this.bank.sendERC20(
        recipient,
        this.token.address,
        amount,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      // Confirm Deposit event emitted with expected values
      const eventDeposit = logs.find(
          e => e.event === "Deposit"
      );
      // Clean data by removing '0x' prefix
      const data = eventDeposit.args._data.slice(2, eventDeposit.args._data.length);

      // Sender's Ethereum address
      let start = 0;
      let end = ADDRESS_BYTE_LENGTH;
      const expectedSender = userOne.toLowerCase().slice(2, userOne.length);
      data.slice(start, end).should.be.equal(expectedSender);

      // Recipient's Polkadot address
      const expectedRecipient = recipient.slice(2, recipient.length);
      start = end;
      end = end + RECIPIENT_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedRecipient);

      // ERC20 token address
      const expectedTokenAddr = this.token.address.toLowerCase().slice(2, this.token.address.length);
      start = end;
      end = end + ADDRESS_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedTokenAddr);

      // Uint256 amount
      const encodedAmount = Web3Utils.padLeft(Web3Utils.numberToHex(amount), 64);
      const expectedAmount = encodedAmount.slice(2, encodedAmount.length);
      start = end;
      end = end + UINT256_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedAmount);

      // Uint256 nonce
      const encodedNonce = Web3Utils.padLeft(Web3Utils.numberToHex(beforeNonce+1), 64);
      const expectedNonce = encodedNonce.slice(2, encodedNonce.length);
      start = end;
      end = end + UINT256_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedNonce);

      //Get the user and Bank token balance after deposit
      const afterTestTokenBalance = Number(await this.token.balanceOf(this.bank.address));
      const afterUserBalance = Number(await this.token.balanceOf(userOne));

      //Confirm that the contract's token balance has increased
      afterTestTokenBalance.should.be.bignumber.equal(beforeTestTokenBalance + amount);
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance - amount);

      // Confirm contract's locked ERC20 counter has increased by amount locked
      const afterTotalERC20 = await this.bank.totalTokens(this.token.address);
      Number(afterTotalERC20).should.be.bignumber.equal(beforeTotalERC20+amount);

      // Confirm contract's nonce has been incremented
      const nonce = Number(await this.bank.nonce());
      nonce.should.be.bignumber.equal(beforeNonce+1);
    });
  });
});