const BigNumber = require('bignumber.js');
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
const TestNoNameToken = artifacts.require("TestToken20");

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
      this.outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [inboundChannel, this.outboundChannel.address], ERC20App);
      this.symbol = "TEST";
      this.name = "Test Token";
      this.decimals = 18;
      this.token = await TestToken.new(this.name, this.symbol);
      this.token1 = await TestNoNameToken.new();
      await this.token1.mint(userOne, "10000").should.be.fulfilled;
      await this.token.mint(userOne, "10000").should.be.fulfilled;
    });

    it("should lock funds", async function () {
      amount = 100;
      const beforeVaultBalance = BigNumber(await this.app.balances(this.token.address));
      const beforeUserBalance = BigNumber(await this.token.balanceOf(userOne));

      await approveFunds(this.token, this.app, userOne, amount * 2)
        .should.be.fulfilled;

      let createMintTokenTransaction = await lockupFunds(this.app, this.token, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = createMintTokenTransaction.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.amount).should.be.bignumber.equal(amount);

      const afterVaultBalance = BigNumber(await this.app.balances(this.token.address));
      const afterUserBalance = BigNumber(await this.token.balanceOf(userOne));

      afterVaultBalance.should.be.bignumber.equal(beforeVaultBalance.plus(100));
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance.minus(100));
 
      let MyContract = new web3.eth.Contract(this.outboundChannel.abi, this.outboundChannel.address);

      (await this.app.wrappedTokenList(this.token.address))
      .should.be.equal(true);
      
      await approveFunds(this.token, this.app, userOne, amount * 2)
      .should.be.fulfilled;

      let mintOnlyTokenTransaction = await lockupFunds(this.app, this.token, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic)
        .should.be.fulfilled;

      const pastEvents = await MyContract.getPastEvents({fromBlock: 0})

      let messageEventCountforminttx = 0, messageEventCountforMintNcreateTx = 0;

      pastEvents.forEach(event => {
          if(event.transactionHash === createMintTokenTransaction.tx)
            messageEventCountforMintNcreateTx++;

          if(event.transactionHash === mintOnlyTokenTransaction.tx)
            messageEventCountforminttx++;
        });

        // Confirm message event emitted only twice for 1.create token and 2.mint call.
      messageEventCountforMintNcreateTx.should.be.equal(2)

      // Confirm message event emitted only once for 1.mint call.
      messageEventCountforminttx.should.be.equal(1)
    });
    it("should lock funds with token with No Name,Symbol function", async function () {
      amount = 100;
      const beforeVaultBalance = BigNumber(await this.app.balances(this.token1.address));
      const beforeUserBalance = BigNumber(await this.token1.balanceOf(userOne));

      await approveFunds(this.token1, this.app, userOne, amount * 2)
        .should.be.fulfilled;

      let createMintTokenTransaction = await lockupFunds(this.app, this.token1, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = createMintTokenTransaction.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.amount).should.be.bignumber.equal(amount);

      const afterVaultBalance = BigNumber(await this.app.balances(this.token1.address));
      const afterUserBalance = BigNumber(await this.token1.balanceOf(userOne));

      afterVaultBalance.should.be.bignumber.equal(beforeVaultBalance.plus(100));
      afterUserBalance.should.be.bignumber.equal(beforeUserBalance.minus(100));

      let MyContract = new web3.eth.Contract(this.outboundChannel.abi, this.outboundChannel.address); 
      (await this.app.wrappedTokenList(this.token1.address))
      .should.be.equal(true);

      await approveFunds(this.token1, this.app, userOne, amount * 2)
      .should.be.fulfilled;

      let mintOnlyTokenTransaction = await lockupFunds(this.app, this.token1, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic)
        .should.be.fulfilled;
      const txHash =  await web3.eth.getTransactionReceipt(mintOnlyTokenTransaction.tx);
      const pastEvents = await MyContract.getPastEvents({fromBlock: 0})

      let messageEventCountforminttx = 0, messageEventCountforMintNcreateTx = 0;
      pastEvents.forEach(event => {
          if(event.transactionHash === createMintTokenTransaction.tx){
            messageEventCountforMintNcreateTx++;
          }
          
          if(event.transactionHash === mintOnlyTokenTransaction.tx)
            messageEventCountforminttx++;
        });

        // Confirm message event emitted only twice for 1.create token and 2.mint call.
      messageEventCountforMintNcreateTx.should.be.equal(2)

      // Confirm message event emitted only once for 1.mint call.
      messageEventCountforminttx.should.be.equal(1)
    });
  })

  describe("withdrawals", function () {

    beforeEach(async function () {
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [owner, outboundChannel.address], ERC20App);
      this.symbol = "TEST";
      this.name = "Test Token";
      this.decimals = 18;
      this.token = await TestToken.new(this.name, this.symbol);

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
});

module.exports = { lockupERC20: lockupFunds };
