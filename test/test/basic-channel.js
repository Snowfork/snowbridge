const Web3 = require('web3');

const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber));

const {
  bootstrap,
  polkadotSenderSS58Alice,
  polkadotSenderSS58Bob,
  polkadotSenderSS58Charlie,
  polkadotSenderSS58Dave,
  polkadotSenderSS58Eve,
  polkadotSenderSS58Ferdie,
} = require('../src/fixtures');

const { ChannelId } = require("../src/helpers");

describe('Bridge', function () {
  let ethClient, subClient;

  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
    this.testParaEthAssetId = 0;
  });

  describe('Basic Channel', function () {
    it('should transfer DOT for 6 accounts from Substrate to Ethereum (basic channel)', async function () {
      const amount = BigNumber('10e+12'); // 10 DOT (12 decimal places in this environment)
      const amountWrapped = BigNumber(Web3.utils.toWei('10', "ether")); // 10 SnowDOT (18 decimal places)
      const ethAccount = ethClient.accounts[1];

      const sudoKey = await subClient.api.query.sudo.key();
      const sudoPair = subClient.keyring.getPair(sudoKey.toString());

      const beforeEthBalance = await ethClient.getDotBalance(ethAccount);
      const beforeSubBalanceAlice = await subClient.queryAccountBalance(polkadotSenderSS58Alice);
      const beforeSubBalanceBob = await subClient.queryAccountBalance(polkadotSenderSS58Bob);
      const beforeSubBalanceCharlie = await subClient.queryAccountBalance(polkadotSenderSS58Charlie);
      const beforeSubBalanceDave = await subClient.queryAccountBalance(polkadotSenderSS58Dave);
      const beforeSubBalanceEve = await subClient.queryAccountBalance(polkadotSenderSS58Eve);
      const beforeSubBalanceFerdie = await subClient.queryAccountBalance(polkadotSenderSS58Ferdie);

      // lock DOT using basic channel
      await subClient.api.tx.sudo.sudo(
        subClient.api.tx.utility.batchAll([
          subClient.api.tx.utility.dispatchAs(
            {system: {signed: subClient.alice.address}},
            subClient.api.tx.dotApp.lock(ChannelId.BASIC, ethAccount, amount.toFixed())
          ),
          subClient.api.tx.utility.dispatchAs(
            {system: {signed: subClient.bob.address}},
            subClient.api.tx.dotApp.lock(ChannelId.BASIC, ethAccount, amount.toFixed())
          ),
          subClient.api.tx.utility.dispatchAs(
            {system: {signed: subClient.charlie.address}},
            subClient.api.tx.dotApp.lock(ChannelId.BASIC, ethAccount, amount.toFixed())
          ),
          subClient.api.tx.utility.dispatchAs(
            {system: {signed: subClient.dave.address}},
            subClient.api.tx.dotApp.lock(ChannelId.BASIC, ethAccount, amount.toFixed())
          ),
          subClient.api.tx.utility.dispatchAs(
            {system: {signed: subClient.eve.address}},
            subClient.api.tx.dotApp.lock(ChannelId.BASIC, ethAccount, amount.toFixed())
          ),
          subClient.api.tx.utility.dispatchAs(
            {system: {signed: subClient.ferdie.address}},
            subClient.api.tx.dotApp.lock(ChannelId.BASIC, ethAccount, amount.toFixed())
          ),
        ])
      ).signAndSend(sudoPair)
      await ethClient.waitForNextEventData({ appName: 'snowDOT', eventName: 'Minted' });

      const afterEthBalance = await ethClient.getDotBalance(ethAccount);
      const afterSubBalanceAlice = await subClient.queryAccountBalance(polkadotSenderSS58Alice);
      const afterSubBalanceBob = await subClient.queryAccountBalance(polkadotSenderSS58Bob);
      const afterSubBalanceCharlie = await subClient.queryAccountBalance(polkadotSenderSS58Charlie);
      const afterSubBalanceDave = await subClient.queryAccountBalance(polkadotSenderSS58Dave);
      const afterSubBalanceEve = await subClient.queryAccountBalance(polkadotSenderSS58Eve);
      const afterSubBalanceFerdie = await subClient.queryAccountBalance(polkadotSenderSS58Ferdie);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(6 * amountWrapped);
      expect(beforeSubBalanceAlice.minus(afterSubBalanceAlice)).to.be.bignumber.gte(amount);
      expect(beforeSubBalanceBob.minus(afterSubBalanceBob)).to.be.bignumber.gte(amount);
      expect(beforeSubBalanceCharlie.minus(afterSubBalanceCharlie)).to.be.bignumber.gte(amount);
      expect(beforeSubBalanceDave.minus(afterSubBalanceDave)).to.be.bignumber.gte(amount);
      expect(beforeSubBalanceEve.minus(afterSubBalanceEve)).to.be.bignumber.gte(amount);
      expect(beforeSubBalanceFerdie.minus(afterSubBalanceFerdie)).to.be.bignumber.gte(amount);
    });
  });
});
