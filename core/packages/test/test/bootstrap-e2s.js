const Web3 = require('web3');

const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber));

const { treasuryAddressSS58, polkadotSenderSS58Alice,
  polkadotRecipientSS58, polkadotRecipient, bootstrap } = require('../src/fixtures');

const { ChannelId } = require("../src/helpers");

describe('Bootstrap-Ethereum-to-Substrate', function () {
  let ethClient, subClient;

  before(async function () {
    const clients = await bootstrap();
    ethClient = clients.ethClient;
    subClient = clients.subClient;
    this.testParaEthAssetId = 0;
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
