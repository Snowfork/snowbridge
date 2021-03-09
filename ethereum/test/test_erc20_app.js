const BigNumber = require('bignumber.js');
const {
  confirmBasicChannelSend,
  confirmIncentivizedChannelSend,
  confirmUnlockTokens,
  deployAppContractWithChannels,
  addressBytes,
  ChannelId,
  buildCommitment
} = require("./helpers");

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");

const approveFunds = (token, contract, account, amount) => {
  return token.approve(contract.address, amount, { from: account })
}

const lockupFunds = (contract, token, sender, recipient, amount, channel) => {
  return contract.lock(
    token.address,
    addressBytes(recipient),
    amount.toString(),
    channel,
    {
      from: sender,
      value: 0
    }
  )
}

contract("ERC20App", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];

  // Constants
  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  describe("deposits", function () {
    beforeEach(async function () {
      [this.channels, this.app] = await deployAppContractWithChannels(ERC20App);
      this.symbol = "TEST";
      this.token = await TestToken.new(100000, "Test Token", this.symbol);

      // Load user account with 'TEST' ERC20 tokens
      await this.token.transfer(userOne, 1000, {
        from: owner
      }).should.be.fulfilled;
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

    it("should send payload to the basic outbound channel", async function () {
      const amount = 100;

      await approveFunds(this.token, this.app, userOne, amount * 2)
        .should.be.fulfilled;

      let tx = await lockupFunds(this.app, this.token, userOne, POLKADOT_ADDRESS, amount, ChannelId.Basic)
        .should.be.fulfilled;

      confirmBasicChannelSend(tx.receipt.rawLogs[3], this.channels.basic.outbound.address, this.app.address, 1)
    });

    it("should send payload to the incentivized outbound channel", async function () {
      const amount = 100;

      await approveFunds(this.token, this.app, userOne, amount * 2)
        .should.be.fulfilled;

      let tx = await lockupFunds(this.app, this.token, userOne, POLKADOT_ADDRESS, amount, ChannelId.Incentivized)
        .should.be.fulfilled;

      confirmIncentivizedChannelSend(tx.receipt.rawLogs[3], this.channels.incentivized.outbound.address, this.app.address, 1)
    });
  })

  describe("withdrawals", function () {

    beforeEach(async function () {
      [this.channels, this.app] = await deployAppContractWithChannels(ERC20App);
      this.symbol = "TEST";
      this.token = await TestToken.new(100000, "Test Token", this.symbol);

      // Load user account with 'TEST' ERC20 tokens
      await this.token.transfer(userOne, 1000, {
        from: owner
      }).should.be.fulfilled;
    });

    it("should unlock via the basic inbound channel", async function () {
      const lockupAmount = 200;
      await approveFunds(this.token, this.app, userOne, lockupAmount * 2)
        .should.be.fulfilled;
      let tx = await lockupFunds(this.app, this.token, userOne, POLKADOT_ADDRESS, lockupAmount, ChannelId.Basic)
        .should.be.fulfilled;

      // recipient on the ethereum side
      const recipient = "0xcCb3C82493AC988CEBE552779E7195A3a9DC651f";

      // expected amount to unlock
      const amount = BigNumber(100);

      // Commitment payload generated using:
      //   cd parachain/pallets/erc20-app
      //   cargo test test_outbound_payload_encode -- --nocapture
      token_addr = this.token.address.replace(/^0x/, "");
      const messages = [
        {
          target: this.app.address,
          nonce: 1,
          payload: `0x010ce3c7000000000000000000000000${token_addr}1aabf8593d9d109b6288149afa35690314f0b798289f8c5c466838dd218a4d50000000000000000000000000ccb3c82493ac988cebe552779e7195a3a9dc651f0000000000000000000000000000000000000000000000000000000000000064`,
        }
      ]
      const commitment = buildCommitment(messages);

      tx = await this.channels.basic.inbound.submit(messages, commitment).should.be.fulfilled;

      confirmUnlockTokens(
        tx.receipt.rawLogs[1],
        this.app.address,
        recipient,
        amount,
      );
    });

    it("should unlock via the incentivized inbound channel", async function () {
      const lockupAmount = 200;
      await approveFunds(this.token, this.app, userOne, lockupAmount * 2)
        .should.be.fulfilled;
      let tx = await lockupFunds(this.app, this.token, userOne, POLKADOT_ADDRESS, lockupAmount, ChannelId.Incentivized)
        .should.be.fulfilled;

      // recipient on the ethereum side
      const recipient = "0xcCb3C82493AC988CEBE552779E7195A3a9DC651f";

      // expected amount to unlock
      const amount = BigNumber(100);

      // Commitment payload generated using:
      //   cd parachain/pallets/erc20-app
      //   cargo test test_outbound_payload_encode -- --nocapture
      token_addr = this.token.address.replace(/^0x/, "");
      const messages = [
        {
          target: this.app.address,
          nonce: 1,
          payload: `0x010ce3c7000000000000000000000000${token_addr}1aabf8593d9d109b6288149afa35690314f0b798289f8c5c466838dd218a4d50000000000000000000000000ccb3c82493ac988cebe552779e7195a3a9dc651f0000000000000000000000000000000000000000000000000000000000000064`,
        }
      ]
      const commitment = buildCommitment(messages);

      tx = await this.channels.incentivized.inbound.submit(messages, commitment).should.be.fulfilled;

      confirmUnlockTokens(
        tx.receipt.rawLogs[1],
        this.app.address,
        recipient,
        amount,
      );
    });
  });
});

module.exports = { lockupERC20: lockupFunds };
