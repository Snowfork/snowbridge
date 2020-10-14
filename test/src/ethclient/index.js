const Web3 = require('web3');
const BigNumber = require('bignumber.js');

const ETHApp = require('../../../ethereum/build/contracts/ETHApp.json');
const ERC20App = require('../../../ethereum/build/contracts/ERC20App.json');
const ERC20 = require('../../../ethereum/build/contracts/ERC20.json');

/**
 * The Ethereum client for Bridge interaction
 */
class EthClient {

    constructor(endpoint, ethAppAddress, erc20AppAddress) {
      var web3 = new Web3(new Web3.providers.WebsocketProvider(endpoint));
      this.web3 = web3;

      this.loadApplicationContracts(ethAppAddress, erc20AppAddress);
    }

    loadApplicationContracts(ethAppAddress, erc20AppAddress) {
      const appETH = new this.web3.eth.Contract(ETHApp.abi, ethAppAddress);
      this.appETH = appETH;

      const appERC20 = new this.web3.eth.Contract(ERC20App.abi, erc20AppAddress);
      this.appERC20 = appERC20;
    };

    loadERC20Contract(tokenContractAddress) {
      return new this.web3.eth.Contract(ERC20.abi, tokenContractAddress);
    }

    async initialize() {
      this.accounts = await this.web3.eth.getAccounts();
      this.web3.eth.defaultAccount = this.accounts[0]
    };

    async getTx(txHash) {
      return await this.web3.eth.getTransaction(txHash);
    }

    async getEthBalance(account = this.web3.eth.defaultAccount) {
      return BigNumber(await this.web3.eth.getBalance(account));
    }

    async getErc20Balance(account, address) {
      const instance = this.loadERC20Contract(address);
      return BigNumber(await instance.methods.balanceOf(account).call());
    }

    async getErc20Allowance(account, address) {
      const instance = this.loadERC20Contract(address);
      return BigNumber(await instance.methods.allowance(account, this.appERC20._address).call());
    }

    async sendEth(from, amount, polkadotRecipient) {

      const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

      let receipt = await this.appETH.methods.sendETH(recipientBytes).send({
        from: from,
        gas: 500000,
        value: this.web3.utils.toBN(amount)
      });

      let tx = await this.web3.eth.getTransaction(receipt.transactionHash);
      let gasCost = BigNumber(tx.gasPrice).times(receipt.gasUsed);

      return { receipt, tx, gasCost }
    }

    async approveERC20(from, amount, address) {
      const erc20Instance = this.loadERC20Contract(address);
      return erc20Instance.methods.approve(this.appERC20._address, this.web3.utils.toBN(amount))
        .send({
          from
        });
    }

    async sendERC20(from, amount, address, recipient) {
      const recipientBytes = Buffer.from(recipient.replace(/^0x/, ""), 'hex');

      return await this.appERC20.methods.sendERC20(recipientBytes, address, this.web3.utils.toBN(amount))
        .send({
          from
        });
    }
}

module.exports.EthClient = EthClient;
