const BigNumber = require('bignumber.js');
const { ethers } = require("ethers");
const {
  deployAppWithMockChannels,
  addressBytes,
  ChannelId,
} = require("./helpers");
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const MockOutboundChannel = artifacts.require("MockOutboundChannel");

const ScaleCodec = artifacts.require("ScaleCodec");
const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");

const {
  printTxPromiseGas
} = require("./helpers");

const approveFunds = (token, contract, account, amount) => {
  return token.approve(contract.address, amount, { from: account })
}

const lockupFunds = (contract, token, sender, recipient, amount, channel) => {
  return contract.lock(
    token.address,
    addressBytes(recipient),
    amount.toString(),
    channel,
    0, // paraId
    {
      from: sender,
      value: 0
    }
  )
}

describe("ERC20App", function () {
  // Accounts
  let accounts;
  let owner;
  let inboundChannel;
  let userOne;

  // Constants
  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  before(async function () {
    const codec = await ScaleCodec.new();
    ERC20App.link(codec);
    accounts = await web3.eth.getAccounts();
    owner = accounts[0];
    inboundChannel = accounts[0];
    userOne = accounts[1];
  });

  describe("deposits", function () {
    beforeEach(async function () {
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [inboundChannel, outboundChannel.address], ERC20App);
      this.symbol = "TEST";
      this.token = await TestToken.new("Test Token", this.symbol);

      await this.token.mint(userOne, "10000").should.be.fulfilled;
    });

    it("should lock funds", async function () {
      amount = 100;
      const beforeVaultBalance = BigNumber(await this.app.balances(this.token.address));
      const beforeUserBalance = BigNumber(await this.token.balanceOf(userOne));

      await approveFunds(this.token, this.app, userOne, amount * 2)
        .should.be.fulfilled;

      let tx = await lockupFunds(this.app, this.token, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.amount).should.be.bignumber.equal(amount);

      const afterVaultBalance = BigNumber(await this.app.balances(this.token.address));
      const afterUserBalance = BigNumber(await this.token.balanceOf(userOne));

      afterVaultBalance.should.be.bignumber.equal(beforeVaultBalance.plus(100));
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance.minus(100));
    });
  })

  describe("withdrawals", function () {

    beforeEach(async function () {
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [owner, outboundChannel.address], ERC20App);
      this.symbol = "TEST";
      this.token = await TestToken.new("Test Token", this.symbol);

      await this.token.mint(userOne, "10000").should.be.fulfilled;
    });

    it("should unlock funds", async function () {
      const lockupAmount = 200;
      await approveFunds(this.token, this.app, userOne, lockupAmount * 2)
        .should.be.fulfilled;
      let tx = await lockupFunds(this.app, this.token, userOne, POLKADOT_ADDRESS, lockupAmount, ChannelId.Basic)
        .should.be.fulfilled;

      // recipient on the ethereum side
      const recipient = "0xcCb3C82493AC988CEBE552779E7195A3a9DC651f";

      // expected amount to unlock
      const amount = ethers.BigNumber.from(100);

      let txPromise = this.app.unlock(
        this.token.address,
        addressBytes(POLKADOT_ADDRESS),
        recipient,
        amount.toString(),
        {
          from: inboundChannel,
        },
      ).should.be.fulfilled;
      printTxPromiseGas(txPromise)
      const { receipt } = await txPromise;

      // decode event
      var iface = new ethers.utils.Interface(ERC20App.abi);
      let event = iface.decodeEventLog(
        'Unlocked(address,bytes32,address,uint256)',
        receipt.rawLogs[1].data,
        receipt.rawLogs[1].topics
      );

      event.recipient.should.be.equal(recipient);
      event.amount.eq(amount).should.be.true;
    });

  });
  describe("upgradeability", function () {
    beforeEach(async function () {
      this.outboundChannel = await MockOutboundChannel.new()
      this.newInboundChannel = accounts[2];
      this.app = await deployAppWithMockChannels(owner, [owner, this.outboundChannel.address], ERC20App);
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
  });
  });

module.exports = { lockupERC20: lockupFunds };
