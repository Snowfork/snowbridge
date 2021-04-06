let { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
let { bundle } = require("@snowfork/snowbridge-types");
const { default: BigNumber } = require('bignumber.js');

class SubClient {

  constructor(endpoint) {
    this.endpoint = endpoint;
    this.api = null;
    this.keyring = null;
  }

  async connect() {
    const provider = new WsProvider(this.endpoint);
    this.api = await ApiPromise.create({
      provider,
      typesBundle: bundle
    })

    this.keyring = new Keyring({ type: 'sr25519' });
    this.alice = this.keyring.addFromUri('//Alice', { name: 'Alice' });
  }

  async queryAssetBalance(accountId, assetId) {
    let balance = await this.api.query.assets.balances(assetId, accountId);
    return BigNumber(balance.toBigInt())
  }

  async queryAccountBalance(accountId) {
    let {
      data: {
        free: balance
      }
    } = await this.api.query.system.account(accountId);
    return BigNumber(balance.toBigInt())
  }

  async burnETH(account, recipient, amount) {
    const txHash = await this.api.tx.eth.burn(0, recipient, amount).signAndSend(account);
  }

  async burnERC20(account, assetId, recipient, amount) {
    const txHash = await this.api.tx.erc20.burn(1, assetId, recipient, amount).signAndSend(account);
  }

  async lockDOT(account, recipient, amount) {
    const txHash = await this.api.tx.dot.lock(0, recipient, amount).signAndSend(account);
  }

}

module.exports.SubClient = SubClient;
