const Web3 = require('web3');
const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber));

const { polkadotRecipientSS58, polkadotRecipient, ETH_TO_PARA_WAIT_TIME, PARA_TO_ETH_WAIT_TIME, bootstrap } = require('../src/fixtures');

describe('Bridge', function () {

  let ethClient, subClient;
  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
    this.ethAssetId = subClient.api.createType('AssetId', 'ETH');
  });

  afterEach(async function () {
    // Wait for new substrate block between tests, as queries sometimes go to old blocks
    await subClient.waitForNextBlock();
  });

  describe('ETH App', function () {
    it('should transfer ETH from Ethereum to Substrate', async function () {
      const amount = BigNumber(Web3.utils.toWei('0.01', "ether"));
      const ethAccount = ethClient.accounts[1];

      const subBalances = await subClient.subscribeAssetBalances(
        polkadotRecipientSS58, this.ethAssetId, 2
      );

      const beforeEthBalance = await ethClient.getEthBalance(ethAccount);
      const beforeSubBalance = await subBalances[0];

      const { gasCost } = await ethClient.lockETH(ethAccount, amount, polkadotRecipient);

      const afterEthBalance = await ethClient.getEthBalance(ethAccount);
      const afterSubBalance = await subBalances[1];

      expect(beforeEthBalance.minus(afterEthBalance)).to.be.bignumber.equal(amount.plus(gasCost));
      expect(afterSubBalance.minus(beforeSubBalance)).to.be.bignumber.equal(amount);
      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance).plus(gasCost));
    });

    it('should transfer ETH from Substrate to Ethereum', async function () {
      const amount = BigNumber(Web3.utils.toWei('0.01', "ether"));
      const ethAccount = ethClient.accounts[1];

      const beforeEthBalance = await ethClient.getEthBalance(ethAccount);
      const beforeSubBalance = await subClient.queryAssetBalance(polkadotRecipientSS58, this.ethAssetId);

      await subClient.burnETH(subClient.alice, ethAccount, amount.toFixed(), 0)
      await ethClient.waitForNextEventData({ appName: 'appETH', eventName: 'Unlocked' });

      const afterEthBalance = await ethClient.getEthBalance(ethAccount);
      const afterSubBalance = await subClient.queryAssetBalance(polkadotRecipientSS58, this.ethAssetId);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amount);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.equal(amount);
      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));
    })
  });

});
