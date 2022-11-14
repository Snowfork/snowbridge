const Web3 = require('web3');
const BigNumber = require('bignumber.js');
const fs = require('fs');
const { env } = require('process');

const contracts = JSON.parse(fs.readFileSync('/tmp/snowbridge/contracts.json', 'utf8'));

const ETHApp = contracts.contracts.ETHApp;
const ERC20Vault = contracts.contracts.ERC20Vault;
const ERC20App = contracts.contracts.ERC20App;
const TestToken = contracts.contracts.TestToken;
const DOTApp = contracts.contracts.DOTApp;
const BasicOutboundChannel = contracts.contracts.BasicOutboundChannel;
const IncentivizedOutboundChannel = contracts.contracts.IncentivizedOutboundChannel;
const BasicInboundChannel = contracts.contracts.BasicInboundChannel;
const IncentivizedInboundChannel = contracts.contracts.IncentivizedInboundChannel;

const IERC20 = require("../../../contracts/artifacts/@openzeppelin/contracts/token/ERC20/IERC20.sol/IERC20.json")
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

  loadApplicationContracts(networkID) {
    const appETH = new this.web3.eth.Contract(ETHApp.abi, ETHApp.address);
    this.appETH = appETH;

    const appERC20 = new this.web3.eth.Contract(ERC20App.abi, ERC20App.address);
    this.appERC20 = appERC20;

    const vaultERC20 = new this.web3.eth.Contract(ERC20Vault.abi, ERC20Vault.address);
    this.vaultERC20 = vaultERC20;

    const appDOT = new this.web3.eth.Contract(DOTApp.abi, DOTApp.address);
    this.appDOT = appDOT;

    const appBasicOutChan = new this.web3.eth.Contract(BasicOutboundChannel.abi,
      BasicOutboundChannel.address);
    this.appBasicOutChan = appBasicOutChan;

    const appIncOutChan = new this.web3.eth.Contract(IncentivizedOutboundChannel.abi,
      IncentivizedOutboundChannel.address);
    this.appIncOutChan = appIncOutChan;

    const appBasicInChan = new this.web3.eth.Contract(BasicInboundChannel.abi,
      BasicInboundChannel.address);
    this.appBasicInChan = appBasicInChan;

    const appIncentivizedInChan = new this.web3.eth.Contract(IncentivizedInboundChannel.abi,
      IncentivizedInboundChannel.address);
    this.appIncentivizedInChan = appIncentivizedInChan;
  };

  loadERC20Contract() {
    return new this.web3.eth.Contract(TestToken.abi, this.TestTokenAddress);
  }

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

    const snowDotAddr = await this.appDOT.methods.token().call();
    const snowDOT = new this.web3.eth.Contract(IERC20.abi, snowDotAddr);
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

  async lockETH(from, amount, polkadotRecipient, channelId, paraId, fee) {
    const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

    let receipt = await this.appETH.methods.lock(recipientBytes, channelId, paraId, fee).send({
      from: from,
      gas: 500000,
      value: this.web3.utils.toBN(amount)
    });

    let tx = await this.web3.eth.getTransaction(receipt.transactionHash);
    let gasCost = BigNumber(tx.gasPrice).times(receipt.gasUsed);

    return { receipt, tx, gasCost }
  }

  async mintERC20(amount, to, owner) {
    const erc20Instance = this.loadERC20Contract();
    return erc20Instance.methods.mint(to, amount)
      .send({
        from: owner,
        gasLimit: 300000
      });
  }

  async approveERC20(from, amount) {
    const erc20Instance = this.loadERC20Contract();
    return erc20Instance.methods.approve(this.vaultERC20._address, this.web3.utils.toBN(amount))
      .send({
        from,
        gasLimit: 300000
      });
  }

  async lockERC20(from, amount, polkadotRecipient, channelId, paraId, fee) {
    const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

    return await this.appERC20.methods.lock(
      this.TestTokenAddress,
      recipientBytes,
      this.web3.utils.toBN(amount),
      channelId,
      paraId,
      fee
    ).send({
      from,
      gas: 500000,
      value: 0
    });
  }

  async burnDOT(from, amount, polkadotRecipient, channelId) {
    const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

    return await this.appDOT.methods.burn(recipientBytes, this.web3.utils.toBN(amount), channelId)
      .send({
        from,
        gas: 500000,
        value: 0
      });
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
