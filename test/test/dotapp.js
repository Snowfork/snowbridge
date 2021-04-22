const Web3 = require('web3');

const { sleep } = require('../src/helpers');
const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

const { polkadotSenderSS58, polkadotRecipientSS58, polkadotRecipient, ETH_TO_PARA_WAIT_TIME, PARA_TO_ETH_WAIT_TIME, bootstrap } = require('../src/fixtures');

describe('Bridge', function () {

  let ethClient, subClient;
  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
  });

  describe('DOT App', function () {

    it('should transfer DOT from Substrate to Ethereum', async function () {

      let amount = BigNumber('100000000000000'); // 100 DOT (12 decimal places in this environment)
      let amountWrapped = BigNumber(Web3.utils.toWei('100', "ether")); // 100 SnowDOT (18 decimal places)

      const account = ethClient.accounts[1];

      let beforeEthBalance = await ethClient.getDotBalance(account);
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotSenderSS58);

      nextNonce = subClient.queryNextEventData({
        eventSection: 'basicOutboundChannel',
        eventMethod: 'MessageAccepted',
        eventDataType: 'MessageNonce'
      });

      // lock DOT using basic channel
      lockTx = await subClient.lockDOT(subClient.alice, account, amount.toFixed(), 0)
      nonce = await nextNonce;
      console.log({ nonce });
      await ethClient.waitForNextEvent({ appName: 'snowDOT', eventName: 'Minted' });
      await sleep(5000);

      let afterEthBalance = await ethClient.getDotBalance(account);
      let afterSubBalance = await subClient.queryAccountBalance(polkadotSenderSS58);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amountWrapped);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.greaterThan(amount);
    })

    xit('should transfer DOT from Ethereum to Substrate (basic channel)', async function () {

      let amount = BigNumber('1000000000000'); // 1 DOT (12 decimal places in this environment)
      let amountWrapped = BigNumber(Web3.utils.toWei('1', "ether")); // 1 SnowDOT (18 decimal places)

      const account = ethClient.accounts[1];

      let beforeEthBalance = await ethClient.getDotBalance(account);
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58);

      await ethClient.burnDOT(account, amountWrapped, polkadotRecipient, 0);
      await sleep(ETH_TO_PARA_WAIT_TIME);

      let afterEthBalance = await ethClient.getDotBalance(account);
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58);

      expect(beforeEthBalance.minus(afterEthBalance)).to.be.bignumber.equal(amountWrapped);
      expect(afterSubBalance.minus(beforeSubBalance)).to.be.bignumber.equal(amount);
    })

    xit('should transfer DOT from Ethereum to Substrate (incentivized channel)', async function () {

      let amount = BigNumber('1000000000000'); // 1 DOT (12 decimal places in this environment)
      let amountWrapped = BigNumber(Web3.utils.toWei('1', "ether")); // 1 SnowDOT (18 decimal places)

      let fee = BigNumber(Web3.utils.toWei('1', "ether")) // 1 SnowDOT
      let treasuryReward = BigNumber("200000000000") // 0.2 DOT

      const account = ethClient.accounts[1];

      let beforeEthBalance = await ethClient.getDotBalance(account);
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58);
      let beforeTreasuryBalance = await subClient.queryAccountBalance("5EYCAe5jHEaRUtbinpdbTLuTyGiVt2TJGQPi9fdvVpNLNfSS");

      await ethClient.burnDOT(account, amountWrapped, polkadotRecipient, 1);
      await sleep(ETH_TO_PARA_WAIT_TIME);

      let afterEthBalance = await ethClient.getDotBalance(account);
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58);
      let afterTreasuryBalance = await subClient.queryAccountBalance("5EYCAe5jHEaRUtbinpdbTLuTyGiVt2TJGQPi9fdvVpNLNfSS");

      expect(beforeEthBalance.minus(afterEthBalance)).to.be.bignumber.equal(amountWrapped.plus(fee));
      expect(afterSubBalance.minus(beforeSubBalance)).to.be.bignumber.equal(amount);
      expect(afterTreasuryBalance.minus(beforeTreasuryBalance)).to.be.bignumber.equal(treasuryReward);
    })

  })

});
