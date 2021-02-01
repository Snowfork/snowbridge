const ETHApp = artifacts.require("ETHApp");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = require('bignumber.js');
const rlp = require("rlp");

const { confirmChannelSend, confirmUnlock } = require("./helpers");

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

contract("EthApp", function (accounts) {
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

      this.app = await ETHApp.new(
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

    it("should support Ethereum deposits", async function () {

      // Load initial contract state
      const beforeTotalETH = BigNumber(await this.app.balance());

      // Prepare transaction parameters
      const recipient = Buffer.from(POLKADOT_ADDRESS.replace(/^0x/, ""), "hex");
      const weiAmount = web3.utils.toWei("0.25", "ether");

      // Deposit Ethereum to the contract and get the logs of the transaction
      const tx = await this.app.lock(
        recipient,
        0,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.amount).should.be.bignumber.equal(BigNumber(weiAmount));

      // Confirm contract's Ethereum balance has increased
      const contractBalanceWei = await web3.eth.getBalance(this.app.address);
      const contractBalance = Web3Utils.fromWei(contractBalanceWei, "ether");
      contractBalance.should.be.bignumber.equal(Web3Utils.fromWei(weiAmount, "ether"));

      // Confirm contract's locked Ethereum counter has increased by amount locked
      const afterTotalETH = BigNumber(await this.app.balance());
      BigNumber(afterTotalETH).should.be.bignumber.equal(beforeTotalETH.plus(BigNumber(weiAmount)));
    });

    it("should send outbound payload to the correct channel", async function () {
      // Prepare transaction parameters
      const recipient = Buffer.from(POLKADOT_ADDRESS.replace(/^0x/, ""), "hex");
      const weiAmount = web3.utils.toWei("0.25", "ether");

      let tx;

      // Deposit Ethereum to the contract (basic channel)
      tx = await this.app.lock(
        recipient,
        0,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;
      // encodedLog = rlp.encode([tx.receipt.rawLogs[1].address, tx.receipt.rawLogs[1].topics,tx.receipt.rawLogs[1].data])
      // console.log(encodedLog.toString('hex'))

      // Confirm payload submitted to basic channel
      confirmChannelSend(tx.receipt.rawLogs[1], this.channels.basic.outbound.address, this.app.address, 0)

      // Deposit Ethereum to the contract (incentivized channel)
      tx = await this.app.lock(
        recipient,
        1,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;

      // Confirm payload submitted to incentivized channel
      confirmChannelSend(tx.receipt.rawLogs[1], this.channels.incentivized.outbound.address, this.app.address, 0)
    });

  })

  describe("withdrawals", function () {

    before(async function () {

      this.channels = {
        basic: {
            inbound: await channelContracts.basic.inbound.new(),
            outbound: await channelContracts.basic.outbound.new()
        },
        incentivized: {
            inbound: await channelContracts.incentivized.inbound.new(),
            outbound: await channelContracts.incentivized.outbound.new()
        },
      };

      this.app = await ETHApp.new(
        {
          inbound: this.channels.basic.inbound.address,
          outbound: this.channels.basic.outbound.address,
        },
        {
          inbound: this.channels.incentivized.inbound.address,
          outbound: this.channels.incentivized.outbound.address,
        },
      );

      // Prepare transaction parameters
      const lockAmountWei = 50000;
      const substrateRecipient = Buffer.from(POLKADOT_ADDRESS.replace(/^0x/, ""), "hex");

      // Send to a substrate recipient to load contract with unlockable ETH
      await this.app.lock(
        substrateRecipient,
        0,
        {
          from: userOne,
          value: lockAmountWei
        }
      ).should.be.fulfilled;

    });

    it("should support ETH unlocks", async function () {

      const recipient = "0xBFC3bfA25613416ED7C8b2a05c3902afd9764880";

      const beforeBalance = BigNumber(await this.app.balance());
      const beforeRecipientBalance = BigNumber(await web3.eth.getBalance(recipient));

      const commitment = [
        {
          target: this.app.address,
          nonce: 0,
          payload: "0x6dea30e7d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d000000000000000000000000bfc3bfa25613416ed7c8b2a05c3902afd97648800000000000000000000000000000000000000000000000000000000000002710"
        }
      ]

      tx = await this.channels.basic.inbound.submit(commitment).should.be.fulfilled;

      confirmUnlock(
        tx.receipt.rawLogs[0],
        this.app.address,
        "0xBFC3bfA25613416ED7C8b2a05c3902afd9764880",
        BigNumber(10000),
      );

      const afterBalance = BigNumber(await this.app.balance());
      const afterRecipientBalance = BigNumber(await web3.eth.getBalance(recipient));


      afterBalance.should.be.bignumber.equal(beforeBalance.minus(BigNumber(10000)));
      //afterRecipientBalance.should.be.bignumber.equal(beforeRecipientBalance.plus(BigNumber(10000)));

    });
  });
});
