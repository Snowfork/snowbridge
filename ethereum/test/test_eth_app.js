const BigNumber = require('bignumber.js');
const MockOutboundChannel = artifacts.require("MockOutboundChannel");
const {
  deployAppWithMockChannels,
  addressBytes,
  ChannelId,
} = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { ethers } = require("ethers");

const ETHApp = artifacts.require("ETHApp");
const ScaleCodec = artifacts.require("ScaleCodec");

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

describe("ETHApp", function () {
  // Accounts
  let accounts;
  let owner;
  let inboundChannel;
  let userOne;

  // Constants
  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  before(async function () {
    const codec = await ScaleCodec.new();
    ETHApp.link(codec);
    accounts = await web3.eth.getAccounts();
    owner = accounts[0];
    inboundChannel = accounts[0];
    userOne = accounts[1];
  });

  describe("deposits", function () {
    beforeEach(async function () {
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [inboundChannel, outboundChannel.address], ETHApp, inboundChannel);
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
  })

  describe("withdrawals", function () {

    beforeEach(async function () {
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [inboundChannel, outboundChannel.address], ETHApp, inboundChannel);
    });

    it("should unlock", async function () {
      // Lockup funds in app
      const lockupAmount = BigNumber(web3.utils.toWei("2", "ether"));
      await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, lockupAmount, ChannelId.Incentivized)
        .should.be.fulfilled;

      // recipient on the ethereum side
      const recipient = "0xcCb3C82493AC988CEBE552779E7195A3a9DC651f";

      // expected amount to unlock
      const amount = web3.utils.toWei("1", "ether");

      const beforeBalance = BigNumber(await this.app.balance());
      const beforeRecipientBalance = BigNumber(await web3.eth.getBalance(recipient));

      let { receipt } = await this.app.unlock(
        addressBytes(POLKADOT_ADDRESS),
        recipient,
        amount.toString(),
        {
          from: inboundChannel,
        }
      ).should.be.fulfilled;

      // decode event
      var iface = new ethers.utils.Interface(ETHApp.abi);
      let event = iface.decodeEventLog(
        'Unlocked(bytes32,address,uint256)',
        receipt.rawLogs[0].data,
        receipt.rawLogs[0].topics
      );

      event.recipient.should.be.equal(recipient);
      event.amount.eq(ethers.BigNumber.from(amount)).should.be.true;

      const afterBalance = BigNumber(await this.app.balance());
      const afterRecipientBalance = BigNumber(await web3.eth.getBalance(recipient));

      afterBalance.should.be.bignumber.equal(beforeBalance.minus(amount));
      afterRecipientBalance.minus(beforeRecipientBalance).should.be.bignumber.equal(amount);
    });
  });
});
