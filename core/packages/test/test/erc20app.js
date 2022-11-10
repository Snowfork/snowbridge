const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

const { TestTokenAddress, polkadotRecipientSS58, polkadotRecipient, bootstrap } = require('../src/fixtures');

const { ChannelId } = require("../src/helpers");

describe('Bridge', function () {

  let ethClient, subClient, testSubClient;
  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
    testSubClient = clients.testSubClient;
    await ethClient.mintERC20("10000", ethClient.accounts[1], ethClient.accounts[0]);
  });

  describe('ERC20 App', function () {
    it('should transfer ERC20 tokens from Ethereum to Substrate', async function () {
      const amount = BigNumber('1000');
      const ethAccount = ethClient.accounts[1];

      // Check if there is already a registered asset for the token
      let maybeAssetId = await subClient.api.query.erc20App.assetId(TestTokenAddress);
      let assetId = maybeAssetId.unwrapOr(null)

      // Query the account balance for the asset if it exists
      let beforeSubBalance;
      if (assetId) {
        beforeSubBalance = await subClient.queryAssetsAccountBalance(assetId, polkadotRecipientSS58)
      } else {
        beforeSubBalance = await BigNumber('0');
      }

      let beforeEthBalance = await ethClient.getErc20Balance(ethAccount);

      await ethClient.approveERC20(ethAccount, amount);
      await ethClient.lockERC20(ethAccount, amount, polkadotRecipient, ChannelId.BASIC, 0, 0);

      await subClient.waitForNextEvent({ eventSection: 'erc20App', eventMethod: 'Minted' });
      await subClient.waitForNextBlock();

      // Ensure there is now a registered asset for the token
      maybeAssetId = await subClient.api.query.erc20App.assetId(TestTokenAddress);
      assetId = maybeAssetId.unwrap()

      let afterEthBalance = await ethClient.getErc20Balance(ethAccount);
      let afterSubBalance = await subClient.queryAssetsAccountBalance(assetId, polkadotRecipientSS58)

      expect(afterEthBalance).to.be.bignumber.equal(beforeEthBalance.minus(amount));
      expect(afterSubBalance).to.be.bignumber.equal(beforeSubBalance.plus(amount));

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));
    });

    it('should transfer ERC20 from Substrate to Ethereum', async function () {
      // Wait for new substrate block before tests, as queries may go to old block
      await subClient.waitForNextBlock();

      const amount = BigNumber('1000');
      const ethAccount = ethClient.accounts[1];

      // Query the asset id for the token
      maybeAssetId = await subClient.api.query.erc20App.assetId(TestTokenAddress);
      assetId = maybeAssetId.unwrap()

      const beforeEthBalance = await ethClient.getErc20Balance(ethAccount);
      const beforeSubBalance = await subClient.queryAssetsAccountBalance(assetId, polkadotRecipientSS58);

      await subClient.burnERC20(subClient.alice, TestTokenAddress, ethAccount, amount.toFixed(), ChannelId.BASIC);
      await ethClient.waitForNextEventData({ appName: 'appERC20', eventName: 'Unlocked' });

      const afterEthBalance = await ethClient.getErc20Balance(ethAccount);
      const afterSubBalance = await subClient.queryAssetsAccountBalance(assetId, polkadotRecipientSS58);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amount);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.equal(amount);
      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));
    })
  });

  describe.skip('ERC20 App XCM', function () {
    it('should transfer ERC20 tokens from Ethereum to Parachain 1001', async function () {
      const amount = BigNumber('1000');
      const xcmFee = 4_000_000;
      const paraId = 1001;
      const ethAccount = ethClient.accounts[1];

      // Check if there is already a registered asset for the token
      let maybeAssetId = await subClient.api.query.erc20App.assetId(TestTokenAddress);
      let assetId = maybeAssetId.unwrapOr(null)

      const testSubBalances = await testSubClient.subscribeAssetsAccountBalances(
        assetId, polkadotRecipientSS58, 2
      );

      const beforeEthBalance = await ethClient.getErc20Balance(ethAccount);
      const beforeSubBalance = await testSubBalances[0];

      await ethClient.approveERC20(ethAccount, amount);
      await ethClient.lockERC20(ethAccount, amount, polkadotRecipient, ChannelId.BASIC, paraId, xcmFee);

      const stopRecording = await subClient.recordEvents('xcmSupport', 'TransferSent');
      const afterEthBalance = await ethClient.getErc20Balance(ethAccount);
      const afterSubBalance = await testSubBalances[1];

      expect(afterEthBalance).to.be.bignumber.equal(beforeEthBalance.minus(amount));
      expect(afterSubBalance).to.be.bignumber.equal(beforeSubBalance.plus(amount));

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));

      const events = await stopRecording();
      expect(events.length).to.be.greaterThan(0);
      const firstEvent = events[0];
      expect(firstEvent.assetId.toString()).to.be.equal(assetId.toString());
      expect(firstEvent.sender.toHex().toLowerCase()).to.be.equal(ethAccount.toString().toLowerCase());
      expect(firstEvent.recipient.toHex().toLowerCase()).to.be.equal(polkadotRecipient.toString().toLowerCase());
      expect(firstEvent.fee.toString()).to.be.equal(xcmFee.toString());
      expect(firstEvent.paraId.toString()).to.be.equal(paraId.toString());
      expect(firstEvent.amount.toString()).to.be.equal(amount.toString());
    });

    it('should not transfer ERC20 tokens from Ethereum to Parachain 1001 without fee', async function () {
      const amount = BigNumber('1000');
      const xcmFee = 0;
      const paraId = 1001;
      const ethAccount = ethClient.accounts[1];

      // Check if there is already a registered asset for the token
      let maybeAssetId = await subClient.api.query.erc20App.assetId(TestTokenAddress);
      let assetId = maybeAssetId.unwrapOr(null)

      // Query the account balance for the asset if it exists
      let beforeSubBalance;
      if (assetId) {
        beforeSubBalance = await subClient.queryAssetsAccountBalance(assetId, polkadotRecipientSS58)
      } else {
        beforeSubBalance = await BigNumber('0');
      }

      let beforeEthBalance = await ethClient.getErc20Balance(ethAccount);

      await ethClient.approveERC20(ethAccount, amount);
      await ethClient.lockERC20(ethAccount, amount, polkadotRecipient, ChannelId.BASIC, paraId, xcmFee);

      const stopRecording = await subClient.recordEvents('xcmSupport', 'TransferFailed');
      await subClient.waitForNextEvent({ eventSection: 'erc20App', eventMethod: 'Minted' });
      await subClient.waitForNextBlock();

      // Ensure there is now a registered asset for the token
      maybeAssetId = await subClient.api.query.erc20App.assetId(TestTokenAddress);
      assetId = maybeAssetId.unwrap()

      let afterEthBalance = await ethClient.getErc20Balance(ethAccount);
      let afterSubBalance = await subClient.queryAssetsAccountBalance(assetId, polkadotRecipientSS58)

      expect(afterEthBalance).to.be.bignumber.equal(beforeEthBalance.minus(amount));
      expect(afterSubBalance).to.be.bignumber.equal(beforeSubBalance.plus(amount));

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));

      const events = await stopRecording();
      expect(events.length).to.be.greaterThan(0);
      const firstEvent = events[0];
      expect(firstEvent.assetId.toString()).to.be.equal(assetId.toString());
      expect(firstEvent.sender.toHex().toLowerCase()).to.be.equal(ethAccount.toString().toLowerCase());
      expect(firstEvent.recipient.toHex().toLowerCase()).to.be.equal(polkadotRecipient.toString().toLowerCase());
      expect(firstEvent.fee.toString()).to.be.equal(xcmFee.toString());
      expect(firstEvent.paraId.toString()).to.be.equal(paraId.toString());
      expect(firstEvent.amount.toString()).to.be.equal(amount.toString());
    });

    it('should not transfer ERC20 tokens from Ethereum to non-existent Parachain 2001', async function () {
      const amount = BigNumber('1000');
      const xcmFee = 4_000_0000;
      const paraId = 2001;
      const ethAccount = ethClient.accounts[1];

      // Check if there is already a registered asset for the token
      let maybeAssetId = await subClient.api.query.erc20App.assetId(TestTokenAddress);
      let assetId = maybeAssetId.unwrapOr(null)

      // Query the account balance for the asset if it exists
      let beforeSubBalance;
      if (assetId) {
        beforeSubBalance = await subClient.queryAssetsAccountBalance(assetId, polkadotRecipientSS58)
      } else {
        beforeSubBalance = await BigNumber('0');
      }

      let beforeEthBalance = await ethClient.getErc20Balance(ethAccount);

      await ethClient.approveERC20(ethAccount, amount);
      await ethClient.lockERC20(ethAccount, amount, polkadotRecipient, ChannelId.BASIC, paraId, xcmFee);

      const stopRecording = await subClient.recordEvents('xcmSupport', 'TransferFailed');
      await subClient.waitForNextEvent({ eventSection: 'erc20App', eventMethod: 'Minted' });
      await subClient.waitForNextBlock();

      // Ensure there is now a registered asset for the token
      maybeAssetId = await subClient.api.query.erc20App.assetId(TestTokenAddress);
      assetId = maybeAssetId.unwrap()

      let afterEthBalance = await ethClient.getErc20Balance(ethAccount);
      let afterSubBalance = await subClient.queryAssetsAccountBalance(assetId, polkadotRecipientSS58)

      expect(afterEthBalance).to.be.bignumber.equal(beforeEthBalance.minus(amount));
      expect(afterSubBalance).to.be.bignumber.equal(beforeSubBalance.plus(amount));

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));

      const events = await stopRecording();
      expect(events.length).to.be.greaterThan(0);
      const firstEvent = events[0];
      expect(firstEvent.assetId.toString()).to.be.equal(assetId.toString());
      expect(firstEvent.sender.toHex().toLowerCase()).to.be.equal(ethAccount.toString().toLowerCase());
      expect(firstEvent.recipient.toHex().toLowerCase()).to.be.equal(polkadotRecipient.toString().toLowerCase());
      expect(firstEvent.fee.toString()).to.be.equal(xcmFee.toString());
      expect(firstEvent.paraId.toString()).to.be.equal(paraId.toString());
      expect(firstEvent.amount.toString()).to.be.equal(amount.toString());
    });
  })
});
