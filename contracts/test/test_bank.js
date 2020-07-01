const Bank = artifacts.require("Bank");

const Web3Utils = require("web3-utils");
const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("Bank", function (accounts) {
  // User accounts
  const userOne = accounts[1];

  // Constants
  const NULL_ADDRESS = "0x0000000000000000000000000000000000000000";
  const POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"

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
      const beforeLockedEth = Number(await this.bank.lockedEthereum());
      const beforeNonce = Number(await this.bank.nonce());

      // Prepare transaction parameters
      const recipient = web3.utils.utf8ToHex(POLKADOT_ADDRESS);
      const weiAmount = web3.utils.toWei("0.25", "ether");
      
      // Deposit Ethereum to the contract and get the logs of the transaction
      const { logs } = await this.bank.depositETH(
        recipient,
        {from: userOne, value: weiAmount}
      ).should.be.fulfilled;

      // Confirm Deposit event emitted with expected values
      const eventDeposit = logs.find(
          e => e.event === "Deposit"
      );
      eventDeposit.args._sender.should.be.equal(userOne);
      eventDeposit.args._recipient.should.be.equal(recipient);
      eventDeposit.args._token.should.be.equal(NULL_ADDRESS);
      eventDeposit.args._symbol.should.be.equal("ETH");
      Number(eventDeposit.args._amount).should.be.bignumber.equal(Number(weiAmount));
      Number(eventDeposit.args._nonce).should.be.equal(beforeNonce+1);
      
      // Confirm contract's Ethereum balance has increased
      const contractBalanceWei = await web3.eth.getBalance(this.bank.address);
      const contractBalance = Web3Utils.fromWei(contractBalanceWei, "ether");
      contractBalance.should.be.bignumber.equal(Web3Utils.fromWei(weiAmount, "ether"));

      // Confirm contract's locked Ethereum counter has increased by amount locked
      const afterLockedEth = await this.bank.lockedEthereum();
      Number(afterLockedEth).should.be.bignumber.equal(beforeLockedEth+Number(weiAmount));

      // Confirm contract's nonce has been incremented
      const nonce = Number(await this.bank.nonce());
      nonce.should.be.bignumber.equal(beforeNonce+1);
    });

  });
});