const EthClient = require('./src/ethclient').EthClient;
const { sleep, fetchDeployedAddrs } = require('./src/helpers');
const Web3Utils = require("web3-utils");

const BigNumber = require('bignumber.js');
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

describe('Bridge', function () {

  var ethClient;
  const endpoint = "http://localhost:9545";
  const gasPrice = 200000000000; //From truffle config
  const polkadotRecipient = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK";


  before(async function () {
    var addrs = await fetchDeployedAddrs();
    this.ethAppAddress = addrs.ethApp;
    this.erc20AppAddress = addrs.erc20App;
    this.tokenAddress = addrs.token;
  });

  beforeEach(async function () {
    ethClient = new EthClient(endpoint, this.ethAppAddress, this.erc20AppAddress);
    await ethClient.initWallet();
  });

  describe('#bridge()', function () {
    it('should transfer ETH from Ethereum to Substrate', async function () {
      const beforeEthBalance = Number(await ethClient.getEthBalance());
      const amountEth = 1;
      const receipt = await ethClient.sendEth(String(amountEth), polkadotRecipient).should.be.fulfilled;

      const gasCost = gasPrice * Number(receipt.gasUsed);
      const gasCostEth = Number(Web3Utils.fromWei(String(gasCost/1000), 'ether'))

      // Factor in gas expenditures and compare results to 3 decimal places due to fluxuation
      const afterEthBalance = Number(await ethClient.getEthBalance());
      const expectedEthBalance = beforeEthBalance - amountEth - gasCostEth;
      afterEthBalance.toFixed(3).should.be.bignumber.equal(expectedEthBalance.toFixed(3));

    });

    it('should transfer ERC20 tokens from Ethereum to Substrate', async function () {
      const amount = 500;
      const beforeBalanceEthereum = Number(await ethClient.getErc20Balance(this.tokenAddress));
      // const beforeBalanceSubstrate = Number(await subClient.getBalance(polkadotRecipient, this.tokenAddress));

      await ethClient.approveERC20(amount, this.tokenAddress);
      const res = await ethClient.sendERC20(amount, this.tokenAddress, polkadotRecipient);

      const event = res.events && res.events.AppTransfer;

      event.returnValues._sender.should.be.equal(await ethClient.getWallet());
      // event._recipient.should.be.equal(polkadotRecipient);
      event.returnValues._token.should.be.equal(this.tokenAddress);
      Number(event.returnValues._amount).should.be.bignumber.equal(amount);

      // Wait 10 seconds for the Relayer to process the transfer
      // await sleep(10000);

      const afterBalanceEthereum = Number(await ethClient.getErc20Balance(this.tokenAddress));
      afterBalanceEthereum.should.be.bignumber.equal(beforeBalanceEthereum - amount);

      // const afterBalanceSubstrate = Number(await subClient.getBalance(polkadotRecipient, this.tokenAddress));
      // afterBalanceSubstrate.should.be.bignumber.equal(beforeBalanceSubstrate + amount);
    });


    it('should transfer ETH from Substrate to Ethereum', async function () {
      const ethAmount = 0.25;
      const weiAmount = Web3Utils.toWei(String(ethAmount), "ether");
      const ethereumRecipient = await ethClient.getWallet();

      const beforeBalanceEthereum = Number(await ethClient.getEthBalance());
      // const beforeBalanceSubstrate = Number(await subClient.getBalance(polkadotRecipient));

      // const res = await ethClient.burnETH(weiAmount, ethereumRecipient);
      // TODO: check event emitted and event fields

      // Wait 10 seconds for the Relayer to process the transfer
      // await sleep(10000);

      const afterBalanceEthereum = Number(await ethClient.getEthBalance());
      afterBalanceEthereum.should.be.bignumber.equal(beforeBalanceEthereum - ethAmount);

      // const afterBalanceSubstrate = Number(await subClient.getBalance(polkadotRecipient));
      // afterBalanceSubstrate.should.be.bignumber.equal(beforeBalanceSubstrate + ethAmount);
    });

    it('should transfer ERC20 tokens from Substrate to Ethereum', async function () {
      const amount = 500;
      const ethereumRecipient = await ethClient.getWallet();

      const beforeBalanceERC20= Number(await ethClient.getErc20Balance(this.tokenAddress));
      // const beforeBalanceSubstrate = Number(await subClient.getErc20Balance(polkadotRecipient, this.tokenAddress));

      // const res = await subClient.burnETH(amount, this.tokenAddress, ethereumRecipient);
      // TODO: check event emitted and event fields

      // Wait 10 seconds for the Relayer to process the transfer
      // await sleep(10000);

      const afterBalanceERC20 = Number(await ethClient.getErc20Balance(this.tokenAddress));
      afterBalanceERC20.should.be.bignumber.equal(beforeBalanceERC20 + amount);

      // const afterBalanceSubstrate = Number(await subClient.getErc20Balance(polkadotRecipient, this.tokenAddress));
      // afterBalanceSubstrate.should.be.bignumber.equal(beforeBalanceSubstrate - amount);
    });
  });
});
