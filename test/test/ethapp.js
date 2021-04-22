const Web3 = require('web3');

const { sleep } = require('../src/helpers');
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

  describe('ETH App', function () {
    it('should transfer ETH from Ethereum to Substrate', async function () {
      const sendAmount = BigNumber(Web3.utils.toWei('0.01', "ether"));
      const fromAccount = ethClient.accounts[1];

      await ethClient.lockETH(fromAccount, sendAmount, polkadotRecipient);

      nextSubEvent = await subClient.queryNextEventData({
        eventSection: 'eth',
        eventMethod: 'Minted'
      });

      const receivedFromAccount = nextSubEvent.toHuman()[0];
      expect(receivedFromAccount).to.be.equal(fromAccount.toLowerCase());

      const receivedRecipient = nextSubEvent.toHuman()[1];
      expect(receivedRecipient).to.be.equal(polkadotRecipientSS58);

      const receivedAmountHex = nextSubEvent.toJSON()[2];
      const receivedAmount = new BigNumber(receivedAmountHex);
      expect(receivedAmount).to.be.bignumber.equal(sendAmount);
    });

    it('should transfer ETH from Substrate to Ethereum', async function () {

      let amount = BigNumber('10000000000000000'); // 0.01 ETH

      const account = ethClient.accounts[1];

      let beforeEthBalance = await ethClient.getEthBalance(account);
      let beforeSubBalance = await subClient.queryAssetBalance(polkadotRecipientSS58, this.ethAssetId);

      await subClient.burnETH(subClient.alice, account, amount.toFixed(), 0)
      await sleep(PARA_TO_ETH_WAIT_TIME);

      let afterEthBalance = await ethClient.getEthBalance(account);
      let afterSubBalance = await subClient.queryAssetBalance(polkadotRecipientSS58, this.ethAssetId);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amount);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.equal(amount);

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));

    })
  });

});
