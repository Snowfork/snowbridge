const EthClient = require('./src/ethclient').EthClient;
const SubClient = require('./src/subclient').SubClient;

const { sleep } = require('./src/helpers');
const Web3Utils = require("web3-utils");
const BigNumber = require('bignumber.js');

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

describe('Bridge', function () {

  var ethClient;
  var subClient;
  const endpoint = "ws://localhost:8545";
  const ethAppAddress = "0x4283d8996E5a7F4BEa58c6052b1471a2a9524C87";
  const erc20AppAddress = "0x3f839E70117C64744930De8567Ae7A5363487cA3";
  const testTokenContractAddress = "0xA588C09D2fE853714d93347F5138FFAA3F7Bdf06";

  const polkadotRecipient = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
  const polkadotRecipientSS58 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

  beforeEach(async function () {
    ethClient = new EthClient(endpoint, ethAppAddress, erc20AppAddress);
    subClient = new SubClient("ws://localhost:9944");
    await subClient.connect();
    await ethClient.initWallet();
  });

  describe('#bridge()', function () {

    it('should transfer ETH from Ethereum to Substrate', async () => {

      // Amount to transfer
      let amount = BigNumber('10000000000000000'); // 0.01 ETH

      let beforeEthBalance = await ethClient.getEthBalance();
      let beforeSubBalance = await subClient.queryAccountBalance("0x00", polkadotRecipientSS58);

      let { gasCost } = await ethClient.sendEth(amount, polkadotRecipient).should.be.fulfilled;

      // Sleep to allow relaying
      await sleep(5000)

      let afterEthBalance = await ethClient.getEthBalance();
      let afterSubBalance = await subClient.queryAccountBalance("0x00", polkadotRecipientSS58);

      (beforeEthBalance.minus(afterEthBalance)).should.be.bignumber.equal(amount.plus(gasCost));

      (afterSubBalance.minus(beforeSubBalance)).should.be.bignumber.equal(amount);

    });

    it('should transfer ERC20 tokens from Ethereum to Substrate', async function () {
      const amount = 500;
      const beforeBalanceEthereum = Number(await ethClient.getErc20Balance(testTokenContractAddress));
      // const beforeBalanceSubstrate = Number(await subClient.getBalance(polkadotRecipient, testTokenContractAddress));

      await ethClient.approveERC20(amount, testTokenContractAddress);
      const res = await ethClient.sendERC20(amount, testTokenContractAddress, polkadotRecipient);

      const event = res.events && res.events.AppTransfer;

      event.returnValues._sender.should.be.equal(await ethClient.getWallet());
      // event._recipient.should.be.equal(polkadotRecipient);
      event.returnValues._token.should.be.equal(testTokenContractAddress);
      Number(event.returnValues._amount).should.be.bignumber.equal(amount);

      // Wait 10 seconds for the Relayer to process the transfer
      // await sleep(10000);

      const afterBalanceEthereum = Number(await ethClient.getErc20Balance(testTokenContractAddress));
      afterBalanceEthereum.should.be.bignumber.equal(beforeBalanceEthereum - amount);

      // const afterBalanceSubstrate = Number(await subClient.getBalance(polkadotRecipient, testTokenContractAddress));
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

      const beforeBalanceERC20= Number(await ethClient.getErc20Balance(testTokenContractAddress));
      // const beforeBalanceSubstrate = Number(await subClient.getErc20Balance(polkadotRecipient, testTokenContractAddress));

      // const res = await subClient.burnETH(amount, testTokenContractAddress, ethereumRecipient);
      // TODO: check event emitted and event fields

      // Wait 10 seconds for the Relayer to process the transfer
      // await sleep(10000);

      const afterBalanceERC20 = Number(await ethClient.getErc20Balance(testTokenContractAddress));
      afterBalanceERC20.should.be.bignumber.equal(beforeBalanceERC20 + amount);

      // const afterBalanceSubstrate = Number(await subClient.getErc20Balance(polkadotRecipient, testTokenContractAddress));
      // afterBalanceSubstrate.should.be.bignumber.equal(beforeBalanceSubstrate - amount);
    });
  });
});
