const Web3 = require('web3');

const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

const { treasuryAddressSS58, polkadotSenderSS58,
  polkadotRecipientSS58, polkadotRecipient, bootstrap } = require('../src/fixtures');

const { ChannelId } = require("../src/helpers");

describe('Bridge', function () {

  let ethClient, subClient;
  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
  });

  describe('DOT App', function () {

    it('should transfer DOT from Substrate to Ethereum (basic channel)', async function () {
      const amount = BigNumber('100000000000000'); // 100 DOT (12 decimal places in this environment)
      const amountWrapped = BigNumber(Web3.utils.toWei('100', "ether")); // 100 SnowDOT (18 decimal places)
      const ethAccount = ethClient.accounts[1];

      const beforeEthBalance = await ethClient.getDotBalance(ethAccount);
      const beforeSubBalance = await subClient.queryAccountBalance(polkadotSenderSS58);

      // lock DOT using basic channel
      await subClient.lockDOT(subClient.alice, ethAccount, amount.toFixed(), ChannelId.BASIC)
      await ethClient.waitForNextEventData({ appName: 'snowDOT', eventName: 'Minted' });

      const afterEthBalance = await ethClient.getDotBalance(ethAccount);
      const afterSubBalance = await subClient.queryAccountBalance(polkadotSenderSS58);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amountWrapped);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.greaterThan(amount);
    })

    it('should transfer DOT from Ethereum to Substrate (basic channel)', async function () {
      const amount = BigNumber('1000000000000'); // 1 DOT (12 decimal places in this environment)
      const amountWrapped = BigNumber(Web3.utils.toWei('1', "ether")); // 1 SnowDOT (18 decimal places)
      const ethAccount = ethClient.accounts[1];
      const subBalances = await subClient.subscribeAccountBalances(
        polkadotRecipientSS58, 2
      );

      const beforeEthBalance = await ethClient.getDotBalance(ethAccount);
      const beforeSubBalance = await subBalances[0];

      await ethClient.burnDOT(ethAccount, amountWrapped, polkadotRecipient, ChannelId.BASIC);

      const afterEthBalance = await ethClient.getDotBalance(ethAccount);
      const afterSubBalance = await subBalances[1];

      expect(beforeEthBalance.minus(afterEthBalance)).to.be.bignumber.equal(amountWrapped);
      expect(afterSubBalance.minus(beforeSubBalance)).to.be.bignumber.equal(amount);
    })

    it('should transfer DOT from Ethereum to Substrate (incentivized channel)', async function () {
      const amount = BigNumber('1000000000000'); // 1 DOT (12 decimal places in this environment)
      const amountWrapped = BigNumber(Web3.utils.toWei('1', "ether")); // 1 SnowDOT (18 decimal places)
      const ethAccount = ethClient.accounts[1];
      const fee = BigNumber(Web3.utils.toWei('1', "ether")) // 1 SnowDOT
      const treasuryReward = BigNumber("200000000000") // 0.2 DOT
      const subBalances = await subClient.subscribeAccountBalances(
        polkadotRecipientSS58, 2
      );
      const treasuryBalances = await subClient.subscribeAccountBalances(
        treasuryAddressSS58, 2
      );

      const beforeEthBalance = await ethClient.getDotBalance(ethAccount);
      const beforeSubBalance = await subBalances[0];
      const beforeTreasuryBalance = await treasuryBalances[0];

      await ethClient.burnDOT(ethAccount, amountWrapped, polkadotRecipient, ChannelId.INCENTIVIZED);

      const afterEthBalance = await ethClient.getDotBalance(ethAccount);
      const afterSubBalance = await subBalances[1];
      const afterTreasuryBalance = await treasuryBalances[1];

      expect(beforeEthBalance.minus(afterEthBalance)).to.be.bignumber.equal(amountWrapped.plus(fee));
      expect(afterSubBalance.minus(beforeSubBalance)).to.be.bignumber.equal(amount);
      expect(afterTreasuryBalance.minus(beforeTreasuryBalance)).to.be.bignumber.equal(treasuryReward);
    })

  })

});
