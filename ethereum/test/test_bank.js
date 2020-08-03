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
  // TODO: implement on contract base58 conversion as recipient's encoded bytes vary in length
  const POLKADOT_ADDRESS2 = "1FRMM8PEiWXYax7rpS6X4XZX1aAAxSWx1CrKTyrVYhV24fg"

  const RECIPIENT_BYTE_LENGTH = 128;
  const BYTES32_LENGTH = 64;

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
      const targetAppID = web3.utils.utf8ToHex("targetapp123");
      const recipient = web3.utils.utf8ToHex(POLKADOT_ADDRESS);
      const weiAmount = web3.utils.toWei("0.25", "ether");

      // Deposit Ethereum to the contract and get the logs of the transaction
      const { logs } = await this.bank.sendETH(
        targetAppID,
        recipient,
        {from: userOne, value: weiAmount}
      ).should.be.fulfilled;

      // Confirm app event emitted with expected values
      const appEvent = logs.find(
          e => e.event === "AppEvent"
      );
      appEvent.args._targetAppID.should.be.equal(targetAppID);
      appEvent.args._name.should.be.equal("sendETH");
      
      // Clean data by removing '0x' prefix
      const data = appEvent.args._data.slice(2, appEvent.args._data.length);

      // Sender's Ethereum address
      const expectedSender = Web3Utils.padLeft(userOne.toLowerCase().slice(2, userOne.length), BYTES32_LENGTH);
      let start = 0;
      let end = BYTES32_LENGTH;
      data.slice(start, end).should.be.equal(expectedSender);

      // Move forward one byte slice
      start = end;
      end = end + BYTES32_LENGTH;

      // ERC20 token address
      const expectedTokenAddr = Web3Utils.padLeft(NULL_ADDRESS.slice(2, NULL_ADDRESS.length), BYTES32_LENGTH);;
      start = end;
      end = end + BYTES32_LENGTH;
      data.slice(start, end).should.be.equal(expectedTokenAddr);

      // Uint256 amount
      const encodedAmount = Web3Utils.padLeft(Web3Utils.numberToHex(weiAmount), 64);
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

<<<<<<< HEAD
      // Move forward one byte slice
      start = end;
      end = end + BYTES32_LENGTH;
      
      // Recipient's Polkadot address
      const expectedRecipient =  Web3Utils.padRight(recipient.slice(2, recipient.length), RECIPIENT_BYTE_LENGTH);
      start = end;
      end = end + RECIPIENT_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedRecipient);

=======
>>>>>>> 54c1a9a... add newlines
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
      const targetAppID = web3.utils.utf8ToHex("tokendexapp987");
      const amount = 100;
      const recipient = web3.utils.utf8ToHex(POLKADOT_ADDRESS);

      // Approve tokens to contract
      await this.token.approve(this.bank.address, amount, {
        from: userOne
      }).should.be.fulfilled;

      // Deposit ERC20 tokens to the contract and get the logs of the transaction
      const { logs } = await this.bank.sendERC20(
        targetAppID,
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
      appEvent.args._targetAppID.should.be.equal(targetAppID);
      appEvent.args._name.should.be.equal("sendERC20");

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

      // Move forward one byte slice
      start = end;
      end = end + BYTES32_LENGTH;

      // Recipient's Polkadot address
      const expectedRecipient = Web3Utils.padRight(recipient.slice(2, recipient.length), RECIPIENT_BYTE_LENGTH);
      start = end;
      end = end + RECIPIENT_BYTE_LENGTH;
      data.slice(start, end).should.be.equal(expectedRecipient);
   
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
