const Web3 = require('web3');

const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

const { polkadotSenderSS58, bootstrap } = require('../src/fixtures');

const { ChannelId } = require("../src/helpers");

describe.skip('Bridge', function () {

  let ethClient, subClient;
  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
  });

  describe('Malicious DOT App', function () {

    it('should deliver message but fail to transfer DOT from Substrate to Ethereum (basic channel)', async function () {
      const amount = BigNumber('100000000000000'); // 100 DOT (12 decimal places in this environment)
      const ethAccount = ethClient.accounts[1];

      const beforeEthBalance = await ethClient.getDotBalance(ethAccount);
      const beforeSubBalance = await subClient.queryAccountBalance(polkadotSenderSS58);

      // lock DOT using basic channel
      await subClient.lockDOT(subClient.alice, ethAccount, amount.toFixed(), ChannelId.BASIC)
      const event = await ethClient.waitForNextEventData({ appName: 'appBasicInChan', eventName: 'MessageDispatched' });

      expect(event.result).to.be.false;

      const afterEthBalance = await ethClient.getDotBalance(ethAccount);
      const afterSubBalance = await subClient.queryAccountBalance(polkadotSenderSS58);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(0);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.greaterThan(0);
    })

  })

});
