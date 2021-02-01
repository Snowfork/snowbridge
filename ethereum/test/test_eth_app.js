const ETHApp = artifacts.require("ETHApp");

const Web3Utils = require("web3-utils");
const ethers = require("ethers");
const BigNumber = require('bignumber.js');
const rlp = require("rlp");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { confirmChannelSend, confirmUnlock } = require("./helpers");

const BASIC_CHANNEL = 0;
const INCENTIVIZED_CHANNEL = 1;

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

const addressBytes = (address) => Buffer.from(address.replace(/^0x/, ""), "hex");

const lockupFunds = (contract, sender, recipient, amount, channel) => {
  return contract.lock(
    addressBytes(recipient),
    channel,
    {
      from: sender,
      value: amount.toString(),
    }
  )
}

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

    it("should lock funds", async function () {
      const beforeBalance = BigNumber(await this.app.balance());
      const amount = BigNumber(web3.utils.toWei("0.25", "ether"));

      const tx = await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, amount, BASIC_CHANNEL)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.amount).should.be.bignumber.equal(amount);

      // Confirm contract's balance has increased
      const afterBalance = await web3.eth.getBalance(this.app.address);
      afterBalance.should.be.bignumber.equal(amount);

      // Confirm contract's locked balance state has increased by amount locked
      const afterBalanceState = BigNumber(await this.app.balance());
      afterBalanceState.should.be.bignumber.equal(beforeBalance.plus(amount));
    });

    it("should send payload to the basic channel", async function () {
      const amount = BigNumber(web3.utils.toWei("0.25", "ether"));

      // Deposit Ethereum to the contract (incentivized channel)
      const tx = await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, amount, BASIC_CHANNEL)
        .should.be.fulfilled;

      // Confirm payload submitted to incentivized channel
      confirmChannelSend(tx.receipt.rawLogs[1], this.channels.basic.outbound.address, this.app.address, 0)
    });

    it("should send payload to the basic channel", async function () {
      const amount = BigNumber(web3.utils.toWei("0.25", "ether"));

      // Deposit Ethereum to the contract (incentivized channel)
      const tx = await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, amount, INCENTIVIZED_CHANNEL)
        .should.be.fulfilled;

      // Confirm payload submitted to incentivized channel
      confirmChannelSend(tx.receipt.rawLogs[1], this.channels.incentivized.outbound.address, this.app.address, 0)
    });

  })

  describe("withdrawals", function () {

    beforeEach(async function () {
      [this.channels, this.app] = await deployContracts(channelContracts);
    });

    it("should support unlocks via the basic inbound channel", async function () {
      // Lockup funds in app
      const lockupAmount = BigNumber(web3.utils.toWei("2", "ether"));
      await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, lockupAmount, BASIC_CHANNEL)
        .should.be.fulfilled;

      // recipient on the ethereum side
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

    it("should support unlocks via the incentivized inbound channel", async function () {
      // Lockup funds in app
      const lockupAmount = BigNumber(web3.utils.toWei("2", "ether"));
      await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, lockupAmount, BASIC_CHANNEL)
        .should.be.fulfilled;

      // recipient on the ethereum side
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
