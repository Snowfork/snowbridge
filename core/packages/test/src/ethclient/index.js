const Web3 = require('web3');
const BigNumber = require('bignumber.js');
const fs = require('fs');
const { env } = require('process');

const contracts = JSON.parse(fs.readFileSync('/tmp/snowbridge/contracts.json', 'utf8'));

const TestToken = contracts.contracts.TestToken;
const BasicOutboundChannel = contracts.contracts.BasicOutboundChannel;
const BasicInboundChannel = contracts.contracts.BasicInboundChannel;

/**
 * The Ethereum client for Bridge interaction
 */
class EthClient {

  constructor(endpoint, networkID) {
    var web3 = new Web3(new Web3.providers.WebsocketProvider(endpoint));
    this.web3 = web3;
    this.networkID = networkID;
    this.TestTokenAddress = TestToken.address;

    this.loadApplicationContracts(networkID);
  }

  loadApplicationContracts(_networkID) {
    const appBasicOutChan = new this.web3.eth.Contract(BasicOutboundChannel.abi, BasicOutboundChannel.address);
    this.appBasicOutChan = appBasicOutChan;

    const appBasicInChan = new this.web3.eth.Contract(BasicInboundChannel.abi,
      BasicInboundChannel.address);
    this.appBasicInChan = appBasicInChan;
  };

  async initialize() {
    if (env.E2E_TEST_ETH_KEY) {
      let keys = [
        env.ROPSTEN_PRIVATE_KEY,
        env.E2E_TEST_ETH_KEY,
        env.BEEFY_RELAY_ETH_KEY,
        env.PARACHAIN_RELAY_ETH_KEY,
      ]
      this.accounts = keys
        .map(k => this.web3.eth.accounts.wallet.add(k))
        .map(account => this.web3.utils.toChecksumAddress(account.address));
    } else {
      this.accounts = await this.web3.eth.getAccounts();
    }
    this.web3.eth.defaultAccount = this.accounts[1];
  };

  async getTx(txHash) {
    return await this.web3.eth.getTransaction(txHash);
  }

  async getEthBalance(account = this.web3.eth.defaultAccount) {
    return BigNumber(await this.web3.eth.getBalance(account));
  }

  async waitForNextEventData({ appName, eventName, eventData }) {
    let foundEvent = new Promise(async (resolve, reject) => {
      this[appName].once(eventName, (error, event) => {
        if (error) {
          reject(error)
        } else if (eventData) {
          resolve(event.returnValues[eventData]);
        } else {
          resolve(event.returnValues)
        }
      })
    });
    return foundEvent;
  }

}

module.exports.EthClient = EthClient;
