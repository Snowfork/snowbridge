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
};

const deployContracts = async (channelContracts) => {
  const channels = {
    basic: {
      inbound: await channelContracts.basic.inbound.new(),
      outbound: await channelContracts.basic.outbound.new(),
    },
    incentivized: {
      inbound: await channelContracts.incentivized.inbound.new(),
      outbound: await channelContracts.incentivized.outbound.new(),
    },
  };

  const app = await ETHApp.new(
    {
      inbound: channels.basic.inbound.address,
      outbound: channels.basic.outbound.address,
    },
    {
      inbound: channels.incentivized.inbound.address,
      outbound: channels.incentivized.outbound.address,
    },
  );

  return [channels, app]
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
      [this.channels, this.app] = await deployContracts(channelContracts);
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
      [this.channels, this.app] = await deployContracts(channelContracts);

      // Lockup funds in app
      const amount = web3.utils.toWei("2", "ether");
      const recipient = Buffer.from(POLKADOT_ADDRESS.replace(/^0x/, ""), "hex");

      await this.app.lock(
        recipient,
        0,
        {
          from: userOne,
          value: amount
        }
      ).should.be.fulfilled;

    });

    it("should support ETH unlocks", async function () {

      // receipt on the ethereum side
      const recipient = "0xcCb3C82493AC988CEBE552779E7195A3a9DC651f";

      // expected amount to unlock
      const amount = BigNumber(web3.utils.toWei("1", "ether"));

      const beforeBalance = BigNumber(await this.app.balance());
      const beforeRecipientBalance = BigNumber(await web3.eth.getBalance(recipient));

      // Commitment payload generated using:
      //   cd parachain/pallets/eth-app
      //   cargo test test_outbound_payload_encode -- --nocapture
      const commitment = [
        {
          target: this.app.address,
          nonce: 0,
          payload: "0x6dea30e71aabf8593d9d109b6288149afa35690314f0b798289f8c5c466838dd218a4d50000000000000000000000000ccb3c82493ac988cebe552779e7195a3a9dc651f0000000000000000000000000000000000000000000000000de0b6b3a7640000"
        }
      ]

      tx = await this.channels.basic.inbound.submit(commitment).should.be.fulfilled;

      confirmUnlock(
        tx.receipt.rawLogs[0],
        this.app.address,
        recipient,
        amount,
      );

      const afterBalance = BigNumber(await this.app.balance());
      const afterRecipientBalance = BigNumber(await web3.eth.getBalance(recipient));

      afterBalance.should.be.bignumber.equal(beforeBalance.minus(amount));
      afterRecipientBalance.minus(beforeRecipientBalance).should.be.bignumber.equal(amount);

    });
  });
});
