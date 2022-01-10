const { ethers } = require("ethers");
const { singletons } = require('@openzeppelin/test-helpers');
const BigNumber = require('bignumber.js');
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const {
  deployAppWithMockChannels,
  addressBytes,
  ChannelId,
} = require("./helpers");
const { expect } = require("chai");

const DOTApp = artifacts.require("DOTApp");
const ScaleCodec = artifacts.require("ScaleCodec");
const Token = artifacts.require("WrappedToken");
const MockOutboundChannel = artifacts.require("MockOutboundChannel");

const DOT_DECIMALS = 10;
const ETHER_DECIMALS = 18;

const granularity = Math.pow(10, ETHER_DECIMALS - DOT_DECIMALS);

const wrapped = (amount) =>
  amount.multipliedBy(granularity);

const unwrapped = (amount) =>
  amount.dividedToIntegerBy(granularity);

const burnTokens = (contract, sender, recipient, amount, channel) => {
  return contract.burn(
    addressBytes(recipient),
    amount.toString(),
    channel,
    {
      from: sender,
      value: 0
    }
  )
}

describe("DOTApp", function () {
  // Accounts
  let accounts;
  let owner;
  let inboundChannel;
  let userOne;

  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  before(async function() {
    const codec = await ScaleCodec.new();
    DOTApp.link(codec);
    accounts = await web3.eth.getAccounts();
    owner = accounts[0];
    inboundChannel =  accounts[0];
    userOne = accounts[1];
  });

  describe("minting", function () {
    beforeEach(async function () {
      this.erc1820 = await singletons.ERC1820Registry(owner);
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(
        owner,
        [inboundChannel, outboundChannel.address],
        DOTApp,
        "Snowfork DOT", "SnowDOT", outboundChannel.address
      );

      this.token = await Token.at(await this.app.token());
    });

    it("should mint funds", async function () {
      const beforeTotalSupply = BigNumber(await this.token.totalSupply());
      const beforeUserBalance = BigNumber(await this.token.balanceOf(userOne));
      const amountNative = BigNumber("10000000000"); // 1 DOT, uint128
      const amountWrapped = wrapped(amountNative);

      let tx = await this.app.mint(
        addressBytes(POLKADOT_ADDRESS),
        userOne,
        amountWrapped.toString(),
        {
          from: inboundChannel,
        }
      ).should.be.fulfilled;

      // decode expected IERC777.Minted event
      var abi = ["event Minted(address indexed operator, address indexed to, uint256 amount, bytes data, bytes operatorData)"];
      var iface = new ethers.utils.Interface(abi);
      let event = iface.decodeEventLog('Minted(address,address,uint256,bytes,bytes)', tx.receipt.rawLogs[0].data, tx.receipt.rawLogs[0].topics);

      const afterTotalSupply = BigNumber(await this.token.totalSupply());
      const afterUserBalance = BigNumber(await this.token.balanceOf(userOne));

      event.operator.should.be.equal(this.app.address);
      event.to.should.be.equal(userOne);
      BigNumber(event.amount.toString()).should.be.bignumber.equal(amountWrapped);

      afterTotalSupply.minus(beforeTotalSupply).should.be.bignumber.equal(amountWrapped);
      afterUserBalance.minus(beforeUserBalance).should.be.bignumber.equal(amountWrapped);
    });
  });

  describe("burning", function () {
    beforeEach(async function () {
      this.erc1820 = await singletons.ERC1820Registry(owner);
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(
        owner,
        [owner, outboundChannel.address],
        DOTApp,
        "Snowfork DOT", "SnowDOT", outboundChannel.address
      );
      this.token = await Token.at(await this.app.token());

      // Mint 2 wrapped DOT
      let amountNative = BigNumber("20000000000"); // 2 DOT, uint128
      let amountWrapped = wrapped(amountNative);
      await this.app.mint(
        addressBytes(POLKADOT_ADDRESS),
        userOne,
        amountWrapped.toString(),
        {
          from: owner,
          value: 0
        }
      );
    });

    it("should burn funds", async function () {
      const beforeTotalSupply = BigNumber(await this.token.totalSupply());
      const beforeUserBalance = BigNumber(await this.token.balanceOf(userOne));
      const amountWrapped = wrapped(BigNumber("10000000000"));

      let tx = await burnTokens(this.app, userOne, POLKADOT_ADDRESS, amountWrapped, ChannelId.Basic).should.be.fulfilled;

      // decode expected IERC777.Burned event
      var abi = ["event Burned(address indexed operator, address indexed from, uint256 amount, bytes data, bytes operatorData)"];
      var iface = new ethers.utils.Interface(abi);
      let event = iface.decodeEventLog('Burned(address,address,uint256,bytes,bytes)', tx.receipt.rawLogs[0].data, tx.receipt.rawLogs[0].topics);

      const afterTotalSupply = BigNumber(await this.token.totalSupply());
      const afterUserBalance = BigNumber(await this.token.balanceOf(userOne));

      event.operator.should.be.equal(this.app.address);
      event.from.should.be.equal(userOne);
      BigNumber(event.amount.toString()).should.be.bignumber.equal(amountWrapped);

      beforeTotalSupply.minus(afterTotalSupply).should.be.bignumber.equal(amountWrapped);
      beforeUserBalance.minus(afterUserBalance).should.be.bignumber.equal(amountWrapped);
    });
  });

  describe("upgradeability", function () {
    beforeEach(async function () {
      this.outboundChannel = await MockOutboundChannel.new()
      this.newInboundChannel = accounts[2];
      this.app = await deployAppWithMockChannels(
        owner,
        [owner, this.outboundChannel.address],
        DOTApp,
        "Snowfork DOT", "SnowDOT", this.outboundChannel.address
      );
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
