const Web3 = require('web3');

const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber));

const { treasuryAddressSS58, polkadotSenderSS58Alice,
  polkadotRecipientSS58, polkadotRecipient, bootstrap } = require('../src/fixtures');

const { ChannelId } = require("../src/helpers");

describe('Bridge', function () {
  let ethClient, subClient;

  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
    this.testParaEthAssetId = 0;
  });

  describe('Bootstrap', function () {
    it('should transfer DOT from Substrate to Ethereum (basic channel)', async function () {
      const amount = BigNumber('100000000000000'); // 100 DOT (12 decimal places in this environment)
      const amountWrapped = BigNumber(Web3.utils.toWei('100', "ether")); // 100 SnowDOT (18 decimal places)
      const ethAccount = ethClient.accounts[1];

      const beforeEthBalance = await ethClient.getDotBalance(ethAccount);
      const beforeSubBalanceAlice = await subClient.queryAccountBalance(polkadotSenderSS58Alice);

      // lock DOT using basic channel
      await subClient.lockDOT(subClient.alice, ethAccount, amount.toFixed(), ChannelId.BASIC)
      await ethClient.waitForNextEventData({ appName: 'appDOT', eventName: 'Minted' });

      const afterEthBalance = await ethClient.getDotBalance(ethAccount);
      const afterSubBalanceAlice = await subClient.queryAccountBalance(polkadotSenderSS58Alice);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amountWrapped);
      expect(beforeSubBalanceAlice.minus(afterSubBalanceAlice)).to.be.bignumber.greaterThan(amount);
    });

    it('should transfer ETH from Ethereum to Substrate (basic channel)', async function () {
      const amount = BigNumber(Web3.utils.toWei('0.001', "ether"));
      const ethAccount = ethClient.accounts[1];

      const subBalances = await subClient.subscribeAssetsAccountBalances(
        this.testParaEthAssetId, polkadotRecipientSS58, 2
      );

      const beforeEthBalance = await ethClient.getEthBalance(ethAccount);
      const beforeSubBalance = await subBalances[0];

      const { gasCost } = await ethClient.lockETH(ethAccount, amount, polkadotRecipient, ChannelId.BASIC, 0, 0);

      const afterEthBalance = await ethClient.getEthBalance(ethAccount);
      const afterSubBalance = await subBalances[1];

      expect(beforeEthBalance.minus(afterEthBalance)).to.be.bignumber.equal(amount.plus(gasCost));
      expect(afterSubBalance.minus(beforeSubBalance)).to.be.bignumber.equal(amount);
      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance).plus(gasCost));
    });
	});
});
