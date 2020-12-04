const EthClient = require('../src/ethclient').EthClient;
const SubClient = require('../src/subclient').SubClient;

const { sleep } = require('../src/helpers');
const Web3Utils = require("web3-utils");
const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

describe('Bridge', function () {

  var ethClient;
  var subClient;

  // Address for //Alice on Substrate
  const polkadotRecipient = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
  const polkadotRecipientSS58 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

  // Contract addresses (hardcoded since we've set them up to be generated deterministically)
  const ethAppAddress = "0xFc97A6197dc90bef6bbEFD672742Ed75E9768553"
  const erc20AppAddress = "0xEDa338E4dC46038493b885327842fD3E301CaB39"
  const tokenAddress = "0x774667629726ec1FaBEbCEc0D9139bD1C8f72a23"

  const ETH_ASSET_ID = "0x00"

  before(async function () {
    ethClient = new EthClient("ws://localhost:8545", ethAppAddress, erc20AppAddress);
    subClient = new SubClient("ws://localhost:11144");
    await subClient.connect();
    await ethClient.initialize();

  });

  describe('ETH App', function () {
    it('should transfer ETH from Ethereum to Substrate', async function() {
      let amount = BigNumber('10000000000000000'); // 0.01 ETH

      let beforeEthBalance = await ethClient.getEthBalance(ethClient.accounts[1]);
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, ETH_ASSET_ID);

      let { gasCost } = await ethClient.sendEth(ethClient.accounts[1], amount, polkadotRecipient);
      await sleep(30000);

      let afterEthBalance = await ethClient.getEthBalance(ethClient.accounts[1]);
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, ETH_ASSET_ID);

      expect(beforeEthBalance.minus(afterEthBalance)).to.be.bignumber.equal(amount.plus(gasCost));
      expect(afterSubBalance.minus(beforeSubBalance)).to.be.bignumber.equal(amount);

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance).plus(gasCost))
    });

    it('should transfer ETH from Substrate to Ethereum', async function () {

      let amount = BigNumber('10000000000000000'); // 0.01 ETH

      let beforeEthBalance = await ethClient.getEthBalance(ethClient.accounts[1]);
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, "0x00");

      await subClient.burnETH(subClient.alice, ethClient.accounts[1], amount.toFixed())
      await sleep(30000);

      let afterEthBalance = await ethClient.getEthBalance(ethClient.accounts[1]);
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, "0x00");

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amount);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.equal(amount);

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance))

    })
  });

  describe('ERC20 App', function () {
    it('should transfer ERC20 tokens from Ethereum to Substrate', async function () {
      let amount = BigNumber('1000');

      let beforeEthBalance = await ethClient.getErc20Balance(ethClient.accounts[0], tokenAddress);
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, tokenAddress);

      await ethClient.approveERC20(ethClient.accounts[0], amount, tokenAddress);
      await ethClient.sendERC20(ethClient.accounts[0], amount, tokenAddress, polkadotRecipient);
      await sleep(30000);

      let afterEthBalance = await ethClient.getErc20Balance(ethClient.accounts[0], tokenAddress);
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, tokenAddress);

      expect(afterEthBalance).to.be.bignumber.equal(beforeEthBalance.minus(amount));
      expect(afterSubBalance).to.be.bignumber.equal(beforeSubBalance.plus(amount));

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance))
    });

    it('should transfer ERC20 from Substrate to Ethereum', async function () {
      let amount = BigNumber('1000');

      let beforeEthBalance = await ethClient.getErc20Balance(ethClient.accounts[0], tokenAddress);
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, tokenAddress);

      await subClient.burnERC20(subClient.alice, tokenAddress, ethClient.accounts[0], amount.toFixed())
      await sleep(30000);

      let afterEthBalance = await ethClient.getErc20Balance(ethClient.accounts[0], tokenAddress);
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, tokenAddress);

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amount);
      expect(beforeSubBalance.minus(afterSubBalance)).to.be.bignumber.equal(amount);

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance))
    })
  })
});
