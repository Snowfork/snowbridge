const ETHApp = artifacts.require("ETHApp");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = require('bignumber.js');
const rlp = require("rlp");

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
      const weiAmount = "256"//web3.utils.toWei("0.25", "ether");

      let tx;



      // Deposit Ethereum to the contract (basic channel)

      console.log("basic outbound channel 1")

      tx = await this.app.lock(
        recipient,
        0,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;

      encodedLog = rlp.encode([tx.receipt.rawLogs[1].address, tx.receipt.rawLogs[1].topics,tx.receipt.rawLogs[1].data])
      console.log(encodedLog.toString('hex'))

      tx = await this.app.lock(
        recipient,
        0,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;
      console.log()

      encodedLog = rlp.encode([tx.receipt.rawLogs[1].address, tx.receipt.rawLogs[1].topics,tx.receipt.rawLogs[1].data])
      console.log(encodedLog.toString('hex'))










      // // Confirm payload submitted to basic channel
      // confirmChannelSend(tx.receipt.rawLogs[1], this.channels.basic.outbound.address, this.app.address, 0)

      // // Deposit Ethereum to the contract (incentivized channel)
      // tx = await this.app.lock(
      //   recipient,
      //   1,
      //   { from: userOne, value: weiAmount }
      // ).should.be.fulfilled;

      // // Confirm payload submitted to incentivized channel
      // confirmChannelSend(tx.receipt.rawLogs[1], this.channels.incentivized.outbound.address, this.app.address, 0)
    });

  })

  // describe("handle received messages", function () {

  //   before(async function () {

  //     this.channels = {
  //       basic: {
  //           inbound: channelContracts.basic.inbound.new(),
  //           outbound: channelContracts.basic.outbound.new()
  //       },
  //       incentivized: {
  //           inbound: channelContracts.incentivized.inbound.new(),
  //           outbound: channelContracts.incentivized.outbound.new()
  //       },
  //     };

  //     this.ethApp = await ETHApp.new(
  //       {
  //         inbound: this.channels.basic.inbound.address,
  //         outbound: this.channels.basic.outbound.address,
  //       },
  //       {
  //         inbound: this.channels.incentivized.inbound.address,
  //         outbound: this.channels.incentivized.outbound.address,
  //       },
  //     );

  //     // Prepare transaction parameters
  //     const lockAmountWei = 5000;
  //     const substrateRecipient = Buffer.from(POLKADOT_ADDRESS, "hex");

  //     // Send to a substrate recipient to load contract with unlockable ETH
  //     await this.ethApp.lock(
  //       substrateRecipient,
  //       true,
  //       {
  //         from: userOne,
  //         value: lockAmountWei
  //       }
  //     ).should.be.fulfilled;
  //   });

  //   it("should support ETH unlocks", async function () {
  //     // Encoded data
  //     const encodedData = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27dcffeaaf7681c89285d65cfbe808b80e5026965733412000000000000000000000000000000000000000000000000000000000000";
  //     // Decoded data
  //     const decodedSender = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
  //     const decodedRecipient = "0xCfFEAAf7681c89285D65CfbE808b80e502696573";
  //     const decodedAmount = 4660;

  //     // Load initial state
  //     const beforeTotalETH = Number(await this.ethApp.balance());
  //     const beforeContractBalanceWei = await web3.eth.getBalance(this.ethApp.address);
  //     const beforeUserBalanceWei = await web3.eth.getBalance(decodedRecipient);

  //     const { logs } = await this.ethApp.handle(encodedData).should.be.fulfilled;

  //     // Confirm unlock event emitted with expected values
  //     const unlockEvent = logs.find(
  //       e => e.event === "Unlocked"
  //     );

  //     unlockEvent.args.sender.should.be.equal(decodedSender);
  //     unlockEvent.args.recipient.should.be.equal(decodedRecipient);
  //     Number(unlockEvent.args.amount).should.be.bignumber.equal(decodedAmount);

  //     // Get the user and ETHApp's Ethereum balance after unlock
  //     const afterContractBalanceWei = await web3.eth.getBalance(this.ethApp.address);
  //     const afterUserBalanceWei = await web3.eth.getBalance(decodedRecipient);

  //     // Confirm user's balance increased and contract's Ethereum balance has decreased
  //     afterUserBalanceWei.should.be.bignumber.equal(parseInt(beforeUserBalanceWei) + decodedAmount);
  //     afterContractBalanceWei.should.be.bignumber.equal(beforeContractBalanceWei - decodedAmount);

  //     // Confirm contract's locked Ethereum counter has decreased by amount unlocked
  //     const afterTotalETH = await this.ethApp.balance();
  //     Number(afterTotalETH).should.be.bignumber.equal(beforeTotalETH - Number(decodedAmount));
  //   });
  // });
});
