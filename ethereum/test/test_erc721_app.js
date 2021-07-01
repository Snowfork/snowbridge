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

const ERC721App = artifacts.require("ERC721App");
const TestToken = artifacts.require("TestToken721");

const approveToken = (token, contract, account, tokenId) => {
  return token.approve(contract.address, tokenId, { from: account })
}

const lockupToken = (contract, token, sender, recipient, tokenId, channel) => {
  return contract.lock(
    token.address,
    addressBytes(recipient),
    tokenId.toString(),
    channel,
    {
      from: sender,
      value: 0
    }
  )
}

contract("ERC721", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];
  const tokenId = 1;
  const anotherTokenId = 2;

  // Constants
  const POLKADOT_ACCOUNT_ID = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  describe("lock", function () {
    beforeEach(async function () {
      [this.channels, this.app] = await deployAppContractWithChannels(ERC721App);
      this.symbol = "TEST";
      this.token = await TestToken.new("Test Token", this.symbol);

      await this.token.mintWithTokenURI(userOne, tokenId, "http://testuri.com/nft.json", {
        from: owner
      }).should.be.fulfilled;

      await this.token.mint(userOne, anotherTokenId, {
        from: owner
      }).should.be.fulfilled;
    });

    it("should lock token with tokenURI metadata", async function () {
      await approveToken(this.token, this.app, userOne, tokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ACCOUNT_ID, tokenId, ChannelId.Basic)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ACCOUNT_ID);
      BigNumber(event.args.tokenId).should.be.bignumber.equal(tokenId);

      let newOwner = await this.token.ownerOf(tokenId);
      newOwner.should.be.equal(this.app.address);
    });

    it("should lock token without tokenURI", async function () {
      await approveToken(this.token, this.app, userOne, anotherTokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ACCOUNT_ID, anotherTokenId, ChannelId.Basic)
          .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ACCOUNT_ID);
      BigNumber(event.args.tokenId).should.be.bignumber.equal(anotherTokenId);

      let newOwner = await this.token.ownerOf(anotherTokenId);
      newOwner.should.be.equal(this.app.address);
    });

    it("should send payload to the basic outbound channel", async function () {
      await approveToken(this.token, this.app, userOne, tokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ACCOUNT_ID, tokenId, ChannelId.Basic)
          .should.be.fulfilled;

      confirmBasicChannelSend(tx.receipt.rawLogs[3], this.channels.basic.outbound.address, this.app.address, 1)
    });

    it("should send payload to the incentivized outbound channel", async function () {
      await approveToken(this.token, this.app, userOne, tokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ACCOUNT_ID, tokenId, ChannelId.Incentivized)
          .should.be.fulfilled;

      confirmIncentivizedChannelSend(tx.receipt.rawLogs[3], this.channels.incentivized.outbound.address, this.app.address, 1)
    });
  });

  describe("unlock", function () {
    beforeEach(async function () {
      [this.channels, this.app] = await deployAppContractWithChannels(ERC721App);
      this.symbol = "TEST";
      this.token = await TestToken.new("Test Token", this.symbol);

      await this.token.mintWithTokenURI(userOne, tokenId, "http://testuri.com/nft.json", {
        from: owner
      }).should.be.fulfilled;
    });

    it("should unlock via the basic inbound channel", async function () {
      await approveToken(this.token, this.app, userOne, tokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ACCOUNT_ID, tokenId, ChannelId.Basic)
          .should.be.fulfilled;

      // recipient on the ethereum side
      const recipient = "0xcCb3C82493AC988CEBE552779E7195A3a9DC651f";
      const expectedTokenId = BigNumber(tokenId);
      token_addr = this.token.address.replace(/^0x/, "");
      const messages = [
        {
          target: this.app.address,
          nonce: 1,
          payload: `0x010ce3c7000000000000000000000000${token_addr}1aabf8593d9d109b6288149afa35690314f0b798289f8c5c466838dd218a4d50000000000000000000000000ccb3c82493ac988cebe552779e7195a3a9dc651f0000000000000000000000000000000000000000000000000000000000000001`
        }
      ];
      const commitment = buildCommitment(messages);

      tx = await this.channels.basic.inbound.submit(messages, commitment).should.be.fulfilled;

      confirmUnlockTokens(
        tx.receipt.rawLogs[2],
        this.app.address,
        recipient,
        expectedTokenId,
      );
    });

    it("should unlock via the incentivized inbound channel", async function () {
      await approveToken(this.token, this.app, userOne, tokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ACCOUNT_ID, tokenId, ChannelId.Incentivized)
          .should.be.fulfilled;

      // recipient on the ethereum side
      const recipient = "0xcCb3C82493AC988CEBE552779E7195A3a9DC651f";
      const expectedTokenId = BigNumber(tokenId);
      token_addr = this.token.address.replace(/^0x/, "");
      const messages = [
        {
          target: this.app.address,
          nonce: 1,
          payload: `0x010ce3c7000000000000000000000000${token_addr}1aabf8593d9d109b6288149afa35690314f0b798289f8c5c466838dd218a4d50000000000000000000000000ccb3c82493ac988cebe552779e7195a3a9dc651f0000000000000000000000000000000000000000000000000000000000000001`
        }
      ];
      const commitment = buildCommitment(messages);

      tx = await this.channels.incentivized.inbound.submit(messages, commitment).should.be.fulfilled;

      confirmUnlockTokens(
        tx.receipt.rawLogs[2],
        this.app.address,
        recipient,
        expectedTokenId,
      );
    });
  });
})
