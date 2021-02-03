const BigNumber = require('bignumber.js');
const {
  confirmChannelSend,
  confirmUnlock,
  deployAppContractWithChannels,
  addressBytes,
  ChannelId
} = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const ETHApp = artifacts.require("ETHApp");

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

contract("ETHApp", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];

  // Constants
  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  describe("deposits", function () {
    beforeEach(async function () {
      [this.channels, this.app] = await deployAppContractWithChannels(ETHApp);
    });

    it("should lock funds", async function () {
      const beforeBalance = BigNumber(await this.app.balance());
      const amount = BigNumber(web3.utils.toWei("0.25", "ether"));

      const tx = await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic)
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

    it("should send payload to the basic outbound channel", async function () {
      const amount = BigNumber(web3.utils.toWei("0.25", "ether"));

      const tx = await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic)
        .should.be.fulfilled;

      confirmChannelSend(tx.receipt.rawLogs[1], this.channels.basic.outbound.address, this.app.address, 0)
    });

    it("should send payload to the incentivized outbound channel", async function () {
      const amount = BigNumber(web3.utils.toWei("0.25", "ether"));

      const tx = await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, amount, ChannelId.Incentivized)
        .should.be.fulfilled;

      confirmChannelSend(tx.receipt.rawLogs[1], this.channels.incentivized.outbound.address, this.app.address, 0)
    });

  })

  describe("withdrawals", function () {

    beforeEach(async function () {
      [this.channels, this.app] = await deployAppContractWithChannels(ETHApp);
    });

    it("should unlock via the basic inbound channel", async function () {
      // Lockup funds in app
      const lockupAmount = BigNumber(web3.utils.toWei("2", "ether"));
      await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, lockupAmount, ChannelId.Basic)
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

    it("should unlock via the incentivized inbound channel", async function () {
      // Lockup funds in app
      const lockupAmount = BigNumber(web3.utils.toWei("2", "ether"));
      await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, lockupAmount, ChannelId.Incentivized)
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

      tx = await this.channels.incentivized.inbound.submit(commitment).should.be.fulfilled;

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

module.exports = { lockupFunds };
