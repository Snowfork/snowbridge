const ETHApp = artifacts.require("ETHApp");

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
  const POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"
  const BYTES32_LENGTH = 64;

  describe("deployment and initialization", function () {
    beforeEach(async function () {
      this.ethApp = await ETHApp.new();
    });

    it("should deploy and initialize the ETHApp contract", async function () {
      this.ethApp.should.exist;

      const nonce = Number(await this.ethApp.nonce());
      nonce.should.be.bignumber.equal(0);
    });
  });

  describe("deposits", function () {
    beforeEach(async function () {
      this.ethApp = await ETHApp.new();
    });

    it("should support Ethereum deposits", async function () {
      // Load initial contract state
      const beforeTotalETH = Number(await this.ethApp.totalETH());
      const beforeNonce = Number(await this.ethApp.nonce());

      // Prepare transaction parameters
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");
      const weiAmount = web3.utils.toWei("0.25", "ether");

      // Deposit Ethereum to the contract and get the logs of the transaction
      const { logs } = await this.ethApp.sendETH(
        recipient,
        {from: userOne, value: weiAmount}
      ).should.be.fulfilled;

      // Confirm app event emitted with expected values
      const appEvent = logs.find(
          e => e.event === "AppTransfer"
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
      Number(afterTotalETH).should.be.bignumber.equal(beforeTotalETH+Number(weiAmount));

      // Confirm contract's nonce has been incremented
      const nonce = Number(await this.ethApp.nonce());
      nonce.should.be.bignumber.equal(beforeNonce+1);
    });
  });


  describe("handle received messages", function () {

    before(async function () {
        this.ethApp = await ETHApp.new();

        // Prepare transaction parameters
        const lockAmountWei = 5000;
        const substrateRecipient = Buffer.from(POLKADOT_ADDRESS, "hex");

        // Send to a substrate recipient to load contract with unlockable ETH
        await this.ethApp.sendETH(
          substrateRecipient,
          {
            from: userOne,
            value: lockAmountWei
          }
        ).should.be.fulfilled;
    });

    it("should support ETH unlocks", async function () {
      // Encoded data
      const encodedData = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27dcffeaaf7681c89285d65cfbe808b80e5026965733412000000000000000000000000000000000000000000000000000000000000";
      // Decoded data
      const decodedSender = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
      const decodedRecipient = "0xCfFEAAf7681c89285D65CfbE808b80e502696573";
      const decodedAmount = 4660;

      // Load initial state
      const beforeTotalETH = Number(await this.ethApp.totalETH());
      const beforeContractBalanceWei = await web3.eth.getBalance(this.ethApp.address);
      const beforeUserBalanceWei = await web3.eth.getBalance(decodedRecipient);

     const { logs } = await this.ethApp.handle(encodedData).should.be.fulfilled;

      // Confirm unlock event emitted with expected values
      const unlockEvent = logs.find(
          e => e.event === "Unlock"
      );

      unlockEvent.args._sender.should.be.equal(decodedSender);
      unlockEvent.args._recipient.should.be.equal(decodedRecipient);
      Number(unlockEvent.args._amount).should.be.bignumber.equal(decodedAmount);

      // Get the user and ETHApp's Ethereum balance after unlock
      const afterContractBalanceWei = await web3.eth.getBalance(this.ethApp.address);
      const afterUserBalanceWei = await web3.eth.getBalance(decodedRecipient);

      // Confirm user's balance increased and contract's Ethereum balance has decreased
      afterUserBalanceWei.should.be.bignumber.equal(beforeUserBalanceWei + decodedAmount);
      afterContractBalanceWei.should.be.bignumber.equal(beforeContractBalanceWei - decodedAmount);

      // Confirm contract's locked Ethereum counter has decreased by amount unlocked
      const afterTotalETH = await this.ethApp.totalETH();
      Number(afterTotalETH).should.be.bignumber.equal(beforeTotalETH-Number(decodedAmount));
    });
  });
});
