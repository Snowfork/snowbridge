
const { sleep } = require('../src/helpers');
const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

const { TestTokenAddress, polkadotRecipientSS58, polkadotRecipient, ETH_TO_PARA_WAIT_TIME, PARA_TO_ETH_WAIT_TIME, bootstrap } = require('../src/fixtures');

describe('Bridge', function () {

  let ethClient, subClient;
  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
    this.erc20AssetId = subClient.api.createType('AssetId',
      { Token: TestTokenAddress }
    );
  });

  describe('ERC20 App', function () {
    it('should transfer ERC20 tokens from Ethereum to Substrate', async function () {
      let amount = BigNumber('1000');

      const account = ethClient.accounts[0];

      let beforeEthBalance = await ethClient.getErc20Balance(account);
      let beforeSubBalance = await subClient.queryAssetBalance(polkadotRecipientSS58, this.erc20AssetId);

      await ethClient.approveERC20(account, amount);
      await ethClient.lockERC20(account, amount, polkadotRecipient);

      await sleep(ETH_TO_PARA_WAIT_TIME);

      let afterEthBalance = await ethClient.getErc20Balance(account);
      let afterSubBalance = await subClient.queryAssetBalance(polkadotRecipientSS58, this.erc20AssetId);

      expect(afterEthBalance).to.be.bignumber.equal(beforeEthBalance.minus(amount));
      expect(afterSubBalance).to.be.bignumber.equal(beforeSubBalance.plus(amount));

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));
    });

    it('should transfer ERC20 from Substrate to Ethereum', async function () {
      let amount = BigNumber('1000');

      const account = ethClient.accounts[0];

      let beforeEthBalance = await ethClient.getErc20Balance(account);
      let beforeSubBalance = await subClient.queryAssetBalance(polkadotRecipientSS58, this.erc20AssetId);

      await subClient.burnERC20(subClient.alice, TestTokenAddress, account, amount.toFixed(), 1)
      await sleep(PARA_TO_ETH_WAIT_TIME);

      let afterEthBalance = await ethClient.getErc20Balance(account);
      let afterSubBalance = await subClient.queryAssetBalance(polkadotRecipientSS58, this.erc20AssetId);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amount);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.equal(amount);

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance));
    })
  })

});
