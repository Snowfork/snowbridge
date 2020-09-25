const EthClient = require('./src/ethclient').EthClient;
const SubClient = require('./src/subclient').SubClient;

const { sleep } = require('./src/helpers');
const Web3Utils = require("web3-utils");
const BigNumber = require('bignumber.js');

const { expect } = require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))

describe('Bridge', function () {

  var ethClient;
  var subClient;
  const endpoint = "ws://localhost:8545";
  const ethAppAddress = "0x4283d8996E5a7F4BEa58c6052b1471a2a9524C87";
  const erc20AppAddress = "0x3f839E70117C64744930De8567Ae7A5363487cA3";
  const testTokenContractAddress = "0xA588C09D2fE853714d93347F5138FFAA3F7Bdf06";

  const polkadotRecipient = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
  const polkadotRecipientSS58 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

  const ETH_ASSET_ID = "0x00"

  beforeEach(async function () {
    ethClient = new EthClient(endpoint, ethAppAddress, erc20AppAddress);
    subClient = new SubClient("ws://localhost:9944");
    await subClient.connect();
    await ethClient.initWallet();
  });

  describe('#bridge()', function () {

    it('should transfer ETH from Ethereum to Substrate', async function() {
      let amount = BigNumber('10000000000000000'); // 0.01 ETH

      let beforeEthBalance = await ethClient.getEthBalance();
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, ETH_ASSET_ID);

      let { gasCost } = await ethClient.sendEth(amount, polkadotRecipient);
      await sleep(5000);

      let afterEthBalance = await ethClient.getEthBalance();
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, ETH_ASSET_ID);

      expect(beforeEthBalance.minus(afterEthBalance)).to.be.bignumber.equal(amount.plus(gasCost));
      expect(afterSubBalance.minus(beforeSubBalance)).to.be.bignumber.equal(amount);

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance).plus(gasCost))
    });

    it('should transfer ERC20 tokens from Ethereum to Substrate', async function () {
      let amount = BigNumber('1000');

      let beforeEthBalance = await ethClient.getErc20Balance(testTokenContractAddress);
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, testTokenContractAddress);

      await ethClient.approveERC20(amount, testTokenContractAddress);
      await ethClient.sendERC20(amount, testTokenContractAddress, polkadotRecipient);
      await sleep(5000);

      let afterEthBalance = await ethClient.getErc20Balance(testTokenContractAddress);
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, testTokenContractAddress);

      expect(afterEthBalance).to.be.bignumber.equal(beforeEthBalance.minus(amount));
      expect(afterSubBalance).to.be.bignumber.equal(beforeSubBalance.plus(amount));

      // conservation of value
      expect(beforeEthBalance.plus(beforeSubBalance)).to.be.bignumber.equal(afterEthBalance.plus(afterSubBalance))
    });


    it('should transfer ETH from Substrate to Ethereum', async function () {

      let amount = BigNumber('10000000000000000'); // 0.01 ETH
      const recipient = ethClient.getWallet()

      let beforeEthBalance = await ethClient.getEthBalance();
      let beforeSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, "0x00");

      await subClient.burnETH(subClient.alice, recipient, amount.toFixed())
      await sleep(30000);

      let afterEthBalance = await ethClient.getEthBalance();
      let afterSubBalance = await subClient.queryAccountBalance(polkadotRecipientSS58, "0x00");

      console.log(beforeEthBalance.toFixed(), " ", afterEthBalance.toFixed());
      console.log(beforeSubBalance.toFixed(), " ", afterSubBalance.toFixed());

      expect(afterEthBalance.minus(beforeEthBalance)).to.be.bignumber.equal(amount);


    })

  });
});
