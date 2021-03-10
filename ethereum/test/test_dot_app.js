const { singletons } = require('@openzeppelin/test-helpers');
const { ethers } = require("ethers");
const BigNumber = require('bignumber.js');
const AssertionError = require('assert').AssertionError;
const {
  confirmBasicChannelSend,
  confirmIncentivizedChannelSend,
  confirmUnlock,
  deployAppContractWithChannels,
  addressBytes,
  ChannelId,
  encodeLog
} = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const DOTApp = artifacts.require("DOTApp");
const Token = artifacts.require("WrappedToken");

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


contract("DOTApp", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const user = accounts[1];

  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  describe("minting", function () {
    beforeEach(async function () {
      this.erc1820 = await singletons.ERC1820Registry(owner);
      [this.channels, this.app] = await deployAppContractWithChannels(DOTApp, "Snowfork DOT", "SnowDOT");
      this.token = await Token.at(await this.app.token());
    });

    it("should mint funds", async function () {
      const beforeTotalSupply = BigNumber(await this.token.totalSupply());
      const beforeUserBalance = BigNumber(await this.token.balanceOf(user));
      const amountNative = BigNumber("10000000000"); // 1 DOT, uint128
      const amountWrapped = wrapped(amountNative);

      let tx = await this.app.mint(
        addressBytes(POLKADOT_ADDRESS),
        user,
        amountWrapped.toString(),
        {
          from: owner,
          value: 0
        }
      ).should.be.fulfilled;

      // decode expected IERC777.Minted event
      var abi = ["event Minted(address indexed operator, address indexed to, uint256 amount, bytes data, bytes operatorData)"];
      var iface = new ethers.utils.Interface(abi);
      let event = iface.decodeEventLog('Minted(address,address,uint256,bytes,bytes)', tx.receipt.rawLogs[0].data, tx.receipt.rawLogs[0].topics);

      const afterTotalSupply = BigNumber(await this.token.totalSupply());
      const afterUserBalance = BigNumber(await this.token.balanceOf(user));

      event.operator.should.be.equal(this.app.address);
      event.to.should.be.equal(user);
      BigNumber(event.amount.toString()).should.be.bignumber.equal(amountWrapped);

      afterTotalSupply.minus(beforeTotalSupply).should.be.bignumber.equal(amountWrapped);
      afterUserBalance.minus(beforeUserBalance).should.be.bignumber.equal(amountWrapped);
    });
  });

  describe("burning", function () {
    beforeEach(async function () {
      this.erc1820 = await singletons.ERC1820Registry(owner);
      [this.channels, this.app] = await deployAppContractWithChannels(DOTApp, "Snowfork DOT", "SnowDOT");
      this.token = await Token.at(await this.app.token());

      // Mint 2 wrapped DOT
      let amountNative = BigNumber("20000000000"); // 2 DOT, uint128
      let amountWrapped = wrapped(amountNative);
      await this.app.mint(
        addressBytes(POLKADOT_ADDRESS),
        user,
        amountWrapped.toString(),
        {
          from: owner,
          value: 0
        }
      )
    });

    it("should burn funds", async function () {
      const beforeTotalSupply = BigNumber(await this.token.totalSupply());
      const beforeUserBalance = BigNumber(await this.token.balanceOf(user));
      const amountWrapped = wrapped(BigNumber("10000000000"));

      let tx = await burnTokens(this.app, user, POLKADOT_ADDRESS, amountWrapped, ChannelId.Basic).should.be.fulfilled;

      // decode expected IERC777.Burned event
      var abi = ["event Burned(address indexed operator, address indexed from, uint256 amount, bytes data, bytes operatorData)"];
      var iface = new ethers.utils.Interface(abi);
      let event = iface.decodeEventLog('Burned(address,address,uint256,bytes,bytes)', tx.receipt.rawLogs[0].data, tx.receipt.rawLogs[0].topics);

      const afterTotalSupply = BigNumber(await this.token.totalSupply());
      const afterUserBalance = BigNumber(await this.token.balanceOf(user));

      event.operator.should.be.equal(this.app.address);
      event.from.should.be.equal(user);
      BigNumber(event.amount.toString()).should.be.bignumber.equal(amountWrapped);

      beforeTotalSupply.minus(afterTotalSupply).should.be.bignumber.equal(amountWrapped);
      beforeUserBalance.minus(afterUserBalance).should.be.bignumber.equal(amountWrapped);
    });

    it("should send payload to the basic outbound channel", async function () {
      const amountWrapped = wrapped(BigNumber("10000000000"));
      let { receipt } = await burnTokens(this.app, user, POLKADOT_ADDRESS, amountWrapped, ChannelId.Basic).should.be.fulfilled;
      confirmBasicChannelSend(receipt.rawLogs[2], this.channels.basic.outbound.address, this.app.address, 1)
    });

    it("should send payload to the incentivized outbound channel", async function () {
      const amountWrapped = wrapped(BigNumber("10000000000"));
      let { receipt } = await burnTokens(this.app, user, POLKADOT_ADDRESS, amountWrapped, ChannelId.Incentivized).should.be.fulfilled;
      confirmIncentivizedChannelSend(receipt.rawLogs[2], this.channels.incentivized.outbound.address, this.app.address, 1)
    });
  });
});
