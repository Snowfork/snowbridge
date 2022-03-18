const BigNumber = require('bignumber.js');
const MockOutboundChannel = artifacts.require("MockOutboundChannel");
const {
  deployAppWithMockChannels,
  ChannelId,
} = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { ethers } = require("ethers");
const { expect } = require("chai");

const ETHApp = artifacts.require("ETHApp");
const ScaleCodec = artifacts.require("ScaleCodec");

const lockupFunds = (contract, sender, recipient, amount, channel, paraId, fee) => {
  return contract.lock(
    recipient,
    channel,
    paraId,
    fee,
    {
      from: sender,
      value: BigNumber(amount),
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

      const beforeBalance = BigNumber(await web3.eth.getBalance(this.app.address));
      const amount = BigNumber(web3.utils.toWei("0.25", "ether"));

      const tx = await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic, 0, 0)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.paraId).should.be.bignumber.equal(0);
      BigNumber(event.args.fee).should.be.bignumber.equal(0);
      BigNumber(event.args.amount).should.be.bignumber.equal(amount);

      // Confirm contract's balance has increased
      const afterBalance = await web3.eth.getBalance(this.app.address);
      afterBalance.should.be.bignumber.equal(beforeBalance.plus(amount));

    });

    it("should lock funds to destination parachain", async function () {
      const beforeBalance = BigNumber(await web3.eth.getBalance(this.app.address));
      const amount = BigNumber(web3.utils.toWei("0.25", "ether"));

      const tx = await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic, 1001, 4_000_000)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.paraId).should.be.bignumber.equal(1001);
      BigNumber(event.args.fee).should.be.bignumber.equal(4_000_000);
      BigNumber(event.args.amount).should.be.bignumber.equal(amount);

      // Confirm contract's balance has increased
      const afterBalance = await web3.eth.getBalance(this.app.address);
      afterBalance.should.be.bignumber.equal(amount);

      // Confirm contract's locked balance state has increased by amount locked
      const afterBalanceState = BigNumber(await web3.eth.getBalance(this.app.address));
      afterBalanceState.should.be.bignumber.equal(beforeBalance.plus(amount));
    });

    it("should not lock funds for amounts greater than 128-bits", async function() {
      await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, "340282366920938463463374607431768211457", ChannelId.Basic, 0, 0)
        .should.be.rejectedWith(/SafeCast: value doesn\'t fit in 128 bits/);
    });
  });

  describe("withdrawals", function () {

    beforeEach(async function () {
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [inboundChannel, outboundChannel.address], ETHApp, inboundChannel);
    });

    it("should unlock", async function () {
      // Lockup funds in app
      const lockupAmount = BigNumber(web3.utils.toWei("2", "ether"));
      await lockupFunds(this.app, userOne, POLKADOT_ADDRESS, lockupAmount, ChannelId.Incentivized, 0, 0)
        .should.be.fulfilled;

      // recipient on the ethereum side
      const recipient = "0xcCb3C82493AC988CEBE552779E7195A3a9DC651f";

      // expected amount to unlock
      const amount = web3.utils.toWei("1", "ether");

      const beforeBalance = BigNumber(await web3.eth.getBalance(this.app.address));
      const beforeRecipientBalance = BigNumber(await web3.eth.getBalance(recipient));

      const unlockAmount = web3.utils.toBN( web3.utils.toWei("2", "ether")).add(web3.utils.toBN(1))

       await this.app.unlock(
        POLKADOT_ADDRESS,
        recipient,
        unlockAmount.toString(),
        {
          from: inboundChannel,
        }
      ).should.be.rejectedWith(/Unable to send Ether/);

      let { receipt } = await this.app.unlock(
        POLKADOT_ADDRESS,
        recipient,
        amount.toString(),
        {
          from: inboundChannel,
        }
      ).should.be.fulfilled;

      // decode event
      var iface = new ethers.utils.Interface(ETHApp.abi);
      let event = iface.decodeEventLog(
        'Unlocked(bytes32,address,uint128)',
        receipt.rawLogs[0].data,
        receipt.rawLogs[0].topics
      );

      event.recipient.should.be.equal(recipient);
      event.amount.eq(ethers.BigNumber.from(amount)).should.be.true;

      const afterBalance = BigNumber(await web3.eth.getBalance(this.app.address));
      const afterRecipientBalance = BigNumber(await web3.eth.getBalance(recipient));

      afterBalance.should.be.bignumber.equal(beforeBalance.minus(amount));
      afterRecipientBalance.minus(beforeRecipientBalance).should.be.bignumber.equal(amount);
    });
  });

  describe("upgradeability", function () {
    beforeEach(async function () {
      this.outboundChannel = await MockOutboundChannel.new()
      this.newInboundChannel = accounts[2];
      this.app = await deployAppWithMockChannels(owner, [inboundChannel, this.outboundChannel.address], ETHApp, inboundChannel);
      const abi = ["event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)"];
      this.iface = new ethers.utils.Interface(abi);
    });

    it("should revert when called by non-admin", async function () {
      await this.app.upgrade(
        [this.newInboundChannel, this.outboundChannel.address],
        [this.newInboundChannel, this.outboundChannel.address],
        {from: userOne}).should.be.rejectedWith(/AccessControl/);
    });

    it("should revert once CHANNEL_UPGRADE_ROLE has been renounced", async function () {
      await this.app.renounceRole(web3.utils.soliditySha3("CHANNEL_UPGRADE_ROLE"), owner, {from: owner});
      await this.app.upgrade(
        [this.newInboundChannel, this.outboundChannel.address],
        [this.newInboundChannel, this.outboundChannel.address],
        {from: owner}
      ).should.be.rejectedWith(/AccessControl/)
    })

    it("should succeed when called by CHANNEL_UPGRADE_ROLE", async function () {
      const oldBasic = await this.app.channels(0);
      const oldIncentivized = await this.app.channels(1);
      await this.app.upgrade(
        [this.newInboundChannel, this.outboundChannel.address],
        [this.newInboundChannel, this.outboundChannel.address],
        {from: owner}
      );
      const newBasic = await this.app.channels(0);
      const newIncentivized = await this.app.channels(1);
      expect(newBasic.inbound !== oldBasic.inbound).to.be.true;
      expect(newIncentivized.inbound !== oldIncentivized.inbound).to.be.true;
    });

    it("CHANNEL_UPGRADE_ROLE can change CHANNEL_UPGRADE_ROLE", async function () {
      const newUpgrader = ethers.Wallet.createRandom().address;
      const tx = await this.app.grantRole(web3.utils.soliditySha3("CHANNEL_UPGRADE_ROLE"), newUpgrader);
      const event = this.iface.decodeEventLog('RoleGranted', tx.receipt.rawLogs[0].data, tx.receipt.rawLogs[0].topics);
      expect(event.account).to.equal(newUpgrader);
    });

    it("reverts when non-upgrader attempts to change CHANNEL_UPGRADE_ROLE", async function () {
      const newUpgrader = ethers.Wallet.createRandom().address;
      await this.app.grantRole(
        web3.utils.soliditySha3("CHANNEL_UPGRADE_ROLE"),
        newUpgrader,
        {from: userOne}
      ).should.be.rejectedWith(/AccessControl/);
    })
  })
});
