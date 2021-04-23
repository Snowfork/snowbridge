const Web3 = require('web3');
const BigNumber = require('bignumber.js');

const ETHApp = require('../../../ethereum/build/contracts/ETHApp.json');
const ERC20App = require('../../../ethereum/build/contracts/ERC20App.json');
const ERC20 = require('../../../ethereum/build/contracts/ERC20.json');
const TestToken = require('../../../ethereum/build/contracts/TestToken.json');
const DOTApp = require('../../../ethereum/build/contracts/DOTApp.json');
const WrappedToken = require('../../../ethereum/build/contracts/WrappedToken.json');
const BasicOutboundChannel = require('../../../ethereum/build/contracts/BasicOutboundChannel.json');
const IncentivizedOutboundChannel = require('../../../ethereum/build/contracts/IncentivizedOutboundChannel.json');

/**
 * The Ethereum client for Bridge interaction
 */
class EthClient {

  constructor(endpoint, networkID) {
    var web3 = new Web3(new Web3.providers.WebsocketProvider(endpoint));
    this.web3 = web3;
    this.networkID = networkID;
    this.TestTokenAddress = TestToken.networks[this.networkID].address;

    this.loadApplicationContracts(networkID);
  }

  loadApplicationContracts(networkID) {
    const appETH = new this.web3.eth.Contract(ETHApp.abi, ETHApp.networks[networkID].address);
    this.appETH = appETH;

    const appERC20 = new this.web3.eth.Contract(ERC20App.abi, ERC20App.networks[networkID].address);
    this.appERC20 = appERC20;

    const appDOT = new this.web3.eth.Contract(DOTApp.abi, DOTApp.networks[networkID].address);
    this.appDOT = appDOT;

    const appBasicOutChan = new this.web3.eth.Contract(BasicOutboundChannel.abi,
      BasicOutboundChannel.networks[networkID].address);
    this.appBasicOutChan = appBasicOutChan;

    const appIncOutChan = new this.web3.eth.Contract(IncentivizedOutboundChannel.abi,
      IncentivizedOutboundChannel.networks[networkID].address);
    this.appIncOutChan = appIncOutChan;
  };

  loadERC20Contract() {
    return new this.web3.eth.Contract(ERC20.abi, this.TestTokenAddress);
  }

  async initialize() {
    this.accounts = await this.web3.eth.getAccounts();
    this.web3.eth.defaultAccount = this.accounts[1];

    const snowDotAddr = await this.appDOT.methods.token().call();
    const snowDOT = new this.web3.eth.Contract(WrappedToken.abi, snowDotAddr);
    this.snowDOT = snowDOT;
  };

  async getTx(txHash) {
    return await this.web3.eth.getTransaction(txHash);
  }

  async getEthBalance(account = this.web3.eth.defaultAccount) {
    return BigNumber(await this.web3.eth.getBalance(account));
  }

  async getErc20Balance(account) {
    const instance = this.loadERC20Contract();
    return BigNumber(await instance.methods.balanceOf(account).call());
  }

  async getDotBalance(account) {
    return BigNumber(await this.snowDOT.methods.balanceOf(account).call());
  }

  async getErc20Allowance(account) {
    const instance = this.loadERC20Contract();
    return BigNumber(await instance.methods.allowance(account, this.appERC20._address).call());
  }

  async lockETH(from, amount, polkadotRecipient) {
    const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

    let receipt = await this.appETH.methods.lock(recipientBytes, 0).send({
      from: from,
      gas: 500000,
      value: this.web3.utils.toBN(amount)
    });

    let tx = await this.web3.eth.getTransaction(receipt.transactionHash);
    let gasCost = BigNumber(tx.gasPrice).times(receipt.gasUsed);

    return { receipt, tx, gasCost }
  }

  async approveERC20(from, amount) {
    const erc20Instance = this.loadERC20Contract();
    return erc20Instance.methods.approve(this.appERC20._address, this.web3.utils.toBN(amount))
      .send({
        from
      });
  }

  async lockERC20(from, amount, polkadotRecipient) {
    const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

    return await this.appERC20.methods.lock(this.TestTokenAddress, recipientBytes, this.web3.utils.toBN(amount), 0)
      .send({
        from,
        gas: 500000,
        value: 0
      });
  }

  async burnDOT(from, amount, polkadotRecipient, channel) {
    const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

    return await this.appDOT.methods.burn(recipientBytes, this.web3.utils.toBN(amount), channel)
      .send({
        from,
        gas: 500000,
        value: 0
      });
  }

  async waitForNextEventData({ appName, eventName, eventData }) {
    let foundEvent = new Promise(async (resolve, reject) => {
      this[appName].once(eventName, (error, event) => {
        resolve(event.returnValues[eventData]);
      })
    });
    return foundEvent;
  }

}


module.exports.EthClient = EthClient;
