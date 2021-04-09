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
  const POLKADOT_ADDRESS = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

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

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ADDRESS, tokenId, ChannelId.Basic)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.tokenId).should.be.bignumber.equal(tokenId);

      let newOwner = await this.token.ownerOf(tokenId);
      newOwner.should.be.equal(this.app.address);
    });

    it("should lock token without tokenURI", async function () {
      await approveToken(this.token, this.app, userOne, anotherTokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ADDRESS, anotherTokenId, ChannelId.Basic)
          .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ADDRESS);
      BigNumber(event.args.tokenId).should.be.bignumber.equal(anotherTokenId);

      let newOwner = await this.token.ownerOf(anotherTokenId);
      newOwner.should.be.equal(this.app.address);
    });

    it("should send payload to the basic outbound channel", async function () {
      await approveToken(this.token, this.app, userOne, tokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ADDRESS, tokenId, ChannelId.Basic)
          .should.be.fulfilled;

      confirmBasicChannelSend(tx.receipt.rawLogs[3], this.channels.basic.outbound.address, this.app.address, 1)
    });

    it("should send payload to the incentivized outbound channel", async function () {
      await approveToken(this.token, this.app, userOne, tokenId)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, userOne, POLKADOT_ADDRESS, tokenId, ChannelId.Incentivized)
          .should.be.fulfilled;

      confirmIncentivizedChannelSend(tx.receipt.rawLogs[3], this.channels.incentivized.outbound.address, this.app.address, 1)
    });
  });
})
