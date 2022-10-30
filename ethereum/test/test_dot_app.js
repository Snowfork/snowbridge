const { ethers } = require("hardhat");
const BigNumber = require('bignumber.js');
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { expect } = require("chai");


const DOT_DECIMALS = 10;
const ETHER_DECIMALS = 18;

const granularity = Math.pow(10, ETHER_DECIMALS - DOT_DECIMALS);

const wrapped = (amount) =>
  amount.multipliedBy(granularity);

const unwrapped = (amount) =>
  amount.dividedToIntegerBy(granularity);

const burnTokens = (contract, sender, recipient, amount, channel) => {
  return contract.burn(
    recipient,
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
  let userOne;

  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  before(async function () {
    this.DOTApp = await ethers.getContractFactory("DOTApp");
    this.ScaleCodec = await ethers.getContractFactory("ScaleCodec");
    this.WrappedToken = await ethers.getContractFactory("WrappedToken");
    this.MockOutboundChannel = await ethers.getContractFactory("MockOutboundChannel");
    this.Registry = await ethers.getContractFactory("ChannelRegistry");

    const codec = await ScaleCodec.new();
    DOTApp.link(codec);

    accounts = await web3.eth.getAccounts();
    owner = accounts[0];
    inboundChannel = accounts[0];
    userOne = accounts[1];
  });

  describe("minting", function () {
    beforeEach(async function () {
      this.token = await WrappedToken.new("Wrapped DOT", "WDOT")

      this.registry = await Registry.new();

      let outboundChannel = await MockOutboundChannel.new()
      await this.registry.updateChannel(0, owner, outboundChannel.address)

      this.app = await DOTApp.new(
        this.token.address,
        outboundChannel.address,
        this.registry.address,
        {
          from: owner,
        }
      )

      await this.token.transferOwnership(this.app.address)
    });

    it("should mint funds", async function () {
      const beforeTotalSupply = BigNumber(await this.token.totalSupply());
      const beforeUserBalance = BigNumber(await this.token.balanceOf(userOne));
      const amountNative = BigNumber("10000000000"); // 1 DOT, uint128
      const amountWrapped = wrapped(amountNative);

      let tx = await this.app.mint(
        POLKADOT_ADDRESS,
        userOne,
        amountWrapped.toString(),
        {
          from: owner,
        }
      ).should.be.fulfilled;

      // decode expected IERC20.Transfer event
      var abi = ["event Transfer(address indexed from, address indexed to, uint256 value)"];
      var iface = new ethers.utils.Interface(abi);
      let event = iface.decodeEventLog('Transfer(address,address,uint256)', tx.receipt.rawLogs[0].data, tx.receipt.rawLogs[0].topics);

      const afterTotalSupply = BigNumber(await this.token.totalSupply());
      const afterUserBalance = BigNumber(await this.token.balanceOf(userOne));

      event.to.should.be.equal(userOne);
      BigNumber(event.value.toString()).should.be.bignumber.equal(amountWrapped);

      afterTotalSupply.minus(beforeTotalSupply).should.be.bignumber.equal(amountWrapped);
      afterUserBalance.minus(beforeUserBalance).should.be.bignumber.equal(amountWrapped);
    });
  });

  describe("burning", function () {
    beforeEach(async function () {
      this.token = await WrappedToken.new("Wrapped DOT", "WDOT")

      this.registry = await Registry.new();

      let outboundChannel = await MockOutboundChannel.new()
      await this.registry.updateChannel(0, owner, outboundChannel.address)

      this.app = await DOTApp.new(
        this.token.address,
        outboundChannel.address,
        this.registry.address,
        {
          from: owner,
        }
      )

      await this.token.transferOwnership(this.app.address)

      // Mint 2 wrapped DOT
      let amountNative = BigNumber("20000000000"); // 2 DOT, uint128
      let amountWrapped = wrapped(amountNative);
      await this.app.mint(
        POLKADOT_ADDRESS,
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

      let tx = await burnTokens(this.app, userOne, POLKADOT_ADDRESS, amountWrapped, 0).should.be.fulfilled;

      // decode expected IERC20.Transfer event
      var abi = ["event Transfer(address indexed from, address indexed to, uint256 value)"];
      var iface = new ethers.utils.Interface(abi);
      let event = iface.decodeEventLog('Transfer(address,address,uint256)', tx.receipt.rawLogs[0].data, tx.receipt.rawLogs[0].topics);

      const afterTotalSupply = BigNumber(await this.token.totalSupply());
      const afterUserBalance = BigNumber(await this.token.balanceOf(userOne));

      event.from.should.be.equal(userOne);
      BigNumber(event.value.toString()).should.be.bignumber.equal(amountWrapped);

      beforeTotalSupply.minus(afterTotalSupply).should.be.bignumber.equal(amountWrapped);
      beforeUserBalance.minus(afterUserBalance).should.be.bignumber.equal(amountWrapped);
    });
  });
});
