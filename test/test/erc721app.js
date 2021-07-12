const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

const { TestToken721Address, polkadotRecipientSS58, polkadotRecipient, bootstrap } = require('../src/fixtures');

const { ChannelId } = require("../src/helpers");

describe('Bridge', function () {
  const ethTokenContract = TestToken721Address;
  const ethTokenId = Date.now();
  let ethOwner;
  let ethUser;
  let ethClient;
  let subClient;

  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
    ethOwner = ethClient.accounts[0];
    ethUser = ethClient.accounts[1];
  });

  describe('ERC721 App', function () {

    it('should transfer ERC721 token from Ethereum to Substrate', async function () {
      const subTokenIds = await subClient.subscribeNFTTokenIdByERC721Id(ethTokenContract, ethTokenId, 2);

      await ethClient.mintERC721(ethTokenId, ethUser, ethOwner);

      const beforeEthOwnerOf = await ethClient.getErc721OwnerOf(ethTokenId);
      expect(beforeEthOwnerOf).to.be.equal(ethUser);
      const beforeSubTokenId = await subTokenIds[0]
      expect(beforeSubTokenId.isNone).to.be.true;

      await ethClient.approveERC721(ethTokenId, ethUser);
      await ethClient.lockERC721(ethTokenId, ethUser, polkadotRecipient, ChannelId.BASIC);

      const afterEthOwnerOf = await ethClient.getErc721OwnerOf(ethTokenId)
      expect(afterEthOwnerOf).to.be.equal(ethClient.ERC721AppAddress);
      const afterSubTokenId = await subTokenIds[1];
      expect(afterSubTokenId.isSome).to.be.true;

      // wait for one block before running the queryNFT command
      await subClient.waitForNextBlock();

      const subNFT2 = await subClient.queryNFTTokenIdByERC721Id(ethTokenContract, ethTokenId);
      expect(subNFT2.isSome).to.be.true;
      expect(subNFT2.unwrap().toString()).to.be.equal(afterSubTokenId.unwrap().toString());

      const subNFT = await subClient.queryNFT(subNFT2.unwrap());
      expect(subNFT.isSome).to.be.true;
      expect(subNFT.unwrap().owner.toHuman()).to.be.equal(polkadotRecipientSS58);
    });

    it('should transfer ERC721 token from Substrate to Ethereum', async function () {
      // Wait for new substrate block before tests, as queries may go to old block
      await subClient.waitForNextBlock();

      const beforeEthOwnerOf = await ethClient.getErc721OwnerOf(ethTokenId);
      expect(beforeEthOwnerOf).to.be.equal(ethClient.ERC721AppAddress);
      const beforeSubTokenId = await subClient.queryNFTTokenIdByERC721Id(ethTokenContract, ethTokenId);
      expect(beforeSubTokenId.isSome).to.be.true;

      await subClient.burnERC721(subClient.alice, beforeSubTokenId.unwrap().toString(), ethUser, ChannelId.BASIC)
      await ethClient.waitForNextEventData({ appName: 'appERC721', eventName: 'Unlocked' });

      const afterEthOwnerOf = await ethClient.getErc721OwnerOf(ethTokenId);
      expect(afterEthOwnerOf).to.be.equal(ethUser);
      const afterSubTokenId = await subClient.queryNFTTokenIdByERC721Id(ethTokenContract, ethTokenId);
      expect(afterSubTokenId.isNone).to.be.true;
    })
  })
});
