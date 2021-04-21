const ETHApp = artifacts.require('ETHApp');
const ERC20App = artifacts.require('ERC20App');
const TestToken = artifacts.require('TestToken');

const BigNumber = web3.BigNumber;

const { lockupETH } = require('./test_eth_app');
const { lockupERC20 } = require('./test_erc20_app');
const { deployGenericAppWithChannels, ChannelId } = require("./helpers");

require('chai')
  .use(require('chai-as-promised'))
  .use(require('chai-bignumber')(BigNumber))
  .should();

contract('Gas expenditures', function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];

  // Constants
  const POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"

  describe('Gas costs', function () {

    beforeEach(async function () {
      [, this.ethApp] = await deployGenericAppWithChannels(owner, ETHApp);
      [, this.erc20App] = await deployGenericAppWithChannels(owner, ERC20App);
    });

    it('lock eth gas usage', async function () {
      // Prepare transaction parameters
      const weiAmount = web3.utils.toWei("0.25", "ether");

      // Deposit Ethereum to the contract

      const result = await lockupETH(this.ethApp, userOne, POLKADOT_ADDRESS, weiAmount,
        ChannelId.Basic).should.be.fulfilled;

      console.log('\lock eth gas: ' + result.receipt.gasUsed);
    });

    // Set up an ERC20 token for testing token deposits
    before(async function () {
      this.symbol = "TEST";
      this.token = await TestToken.new("Test Token", this.symbol);

      // Load user account with 'TEST' ERC20 tokens
      await this.token.mint("1000", {
        from: userOne
      }).should.be.fulfilled;
    });

    it('lock erc20 gas usage', async function () {
      // Prepare transaction parameters
      const amount = 100;

      // Approve tokens to contract
      await this.token.approve(this.erc20App.address, amount, {
        from: userOne
      }).should.be.fulfilled;

      const result = await lockupERC20(this.erc20App, this.token, userOne,
        POLKADOT_ADDRESS, amount, ChannelId.Basic).should.be.fulfilled;

      console.log('\lock erc20 gas: ' + result.receipt.gasUsed);
    });
  });
});
