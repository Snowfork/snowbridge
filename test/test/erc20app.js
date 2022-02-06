const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

const { TestTokenAddress, polkadotRecipientSS58, polkadotRecipient, bootstrap } = require('../src/fixtures');

const { ChannelId, sleep } = require("../src/helpers");

describe('Bridge', function () {

  let ethClient, subClient;
  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;

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
      await ethClient.lockERC20(ethAccount, amount, polkadotRecipient, ChannelId.BASIC);

      await sleep(90 * 1000)

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
  })

});
