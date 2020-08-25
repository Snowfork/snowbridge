const EthereumApp = artifacts.require("EthereumApp");

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
  const BYTES32_LENGTH = 64;

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.ethereumApp = await EthereumApp.new();
    });

    it("should deploy and initialize the EthereumApp contract", async function () {
      this.ethereumApp.should.exist;

      const nonce = Number(await this.ethereumApp.nonce());
      nonce.should.be.bignumber.equal(0);
    });
  });

  describe("deposits", function () {
    beforeEach(async function () {
      this.ethereumApp = await EthereumApp.new();
    });

    it("should support Ethereum deposits", async function () {
      // Load initial contract state
      const beforeTotalETH = Number(await this.ethereumApp.totalETH());
      const beforeNonce = Number(await this.ethereumApp.nonce());

      // Prepare transaction parameters
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");
      const weiAmount = web3.utils.toWei("0.25", "ether");

      // Deposit Ethereum to the contract and get the logs of the transaction
      const { logs } = await this.ethereumApp.sendETH(
        recipient,
        {from: userOne, value: weiAmount}
      ).should.be.fulfilled;

      // Confirm app event emitted with expected values
      const appEvent = logs.find(
          e => e.event === "AppEvent"
      );

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

      // Confirm contract's Ethereum balance has increased
      const contractBalanceWei = await web3.eth.getBalance(this.ethereumApp.address);
      const contractBalance = Web3Utils.fromWei(contractBalanceWei, "ether");
      contractBalance.should.be.bignumber.equal(Web3Utils.fromWei(weiAmount, "ether"));

      // Confirm contract's locked Ethereum counter has increased by amount locked
      const afterTotalETH = await this.ethereumApp.totalETH();
      Number(afterTotalETH).should.be.bignumber.equal(beforeTotalETH+Number(weiAmount));

      // Confirm contract's nonce has been incremented
      const nonce = Number(await this.ethereumApp.nonce());
      nonce.should.be.bignumber.equal(beforeNonce+1);
    });
  });
});
