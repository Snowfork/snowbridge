const ETHApp = artifacts.require('ETHApp');
const ERC20App = artifacts.require('ERC20App');
const BasicSendChannel = artifacts.require('BasicSendChannel');
const IncentivizedSendChannel = artifacts.require('IncentivizedSendChannel');
const TestToken = artifacts.require('TestToken');

const BigNumber = web3.BigNumber;

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
      const basicSendChannel = await BasicSendChannel.new();
      const incentivizedSendChannel = await IncentivizedSendChannel.new();
      this.ethApp = await ETHApp.new(basicSendChannel.address, incentivizedSendChannel.address);
      this.erc20App = await ERC20App.new(basicSendChannel.address, incentivizedSendChannel.address);
    });

    it('sendETH gas usage', async function () {
      // Prepare transaction parameters
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");
      const weiAmount = web3.utils.toWei("0.25", "ether");

      // Deposit Ethereum to the contract
      const result = await this.ethApp.sendETH(
        recipient,
        true,
        { from: userOne, value: weiAmount }
      ).should.be.fulfilled;

      console.log('\tsendETH gas: ' + result.receipt.gasUsed);
    });

    // Set up an ERC20 token for testing token deposits
    before(async function () {
      this.symbol = "TEST";
      this.token = await TestToken.new(100000, "Test Token", this.symbol);

      // Load user account with 'TEST' ERC20 tokens
      await this.token.transfer(userOne, 1000, {
        from: owner
      }).should.be.fulfilled;
    });

    it('sendERC20 gas usage', async function () {
      // Prepare transaction parameters
      const recipient = Buffer.from(POLKADOT_ADDRESS, "hex");
      const amount = 100;

      // Approve tokens to contract
      await this.token.approve(this.erc20App.address, amount, {
        from: userOne
      }).should.be.fulfilled;

      // Deposit ERC20 tokens to the contract
      const result = await this.erc20App.sendERC20(
        recipient,
        this.token.address,
        amount,
        true,
        {
          from: userOne,
          value: 0
        }
      ).should.be.fulfilled;

      console.log('\tsendERC20 gas: ' + result.receipt.gasUsed);
    });
  });
});
