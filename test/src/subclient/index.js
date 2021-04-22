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

  async subscribeAssetBalances(accountId, assetId, length) {
    // Create an array of promises and resolvers for the balances
    const balancePromiseItems = new Array(length).fill().map(i => {
      let resolver;
      const promise = new Promise(async (resolve, reject) => {
        resolver = resolve;
      });
      return { promise, resolver };
    });
    const balancePromises = balancePromiseItems.map(i => i.promise);
    const resolveBalance = balancePromiseItems.map(i => i.resolver);

    // Setup our balance subscription and resolve each promise one by one
    let count = 0;
    const unsubscribe = await this.api.query.assets.balances(assetId, accountId, newBalance => {
      resolveBalance[count](BigNumber(newBalance.toBigInt()));
      count++;
      if (count === length) {
        unsubscribe();
      }
    });

    return balancePromises;
  }

  async queryAccountBalance(accountId) {
    let {
      data: {
        free: balance
      }
    } = await this.api.query.system.account(accountId);
    return BigNumber(balance.toBigInt())
  }

  async waitForNextEvent({ eventSection, eventMethod, eventDataType }) {
    let foundData = new Promise(async (resolve, reject) => {
      const unsubscribe = await this.api.query.system.events((events) => {
        events.forEach((record) => {
          const { event, phase } = record;
          const types = event.typeDef;
          if (event.section === eventSection && event.method === eventMethod) {
            if (eventDataType === undefined) {
              resolve(event.data);
            } else {
              event.data.forEach((data, index) => {
                if (types[index].type === eventDataType) {
                  unsubscribe();
                  resolve(data);
                }
              });
            }
          }
        });
      });
    });
    return foundData;
  }

  async burnETH(account, recipient, amount, channel) {
    return await this.api.tx.eth.burn(channel, recipient, amount).signAndSend(account);
  }

  async burnERC20(account, assetId, recipient, amount, channel) {
    return await this.api.tx.erc20.burn(channel, assetId, recipient, amount).signAndSend(account);
  }

  async lockDOT(account, recipient, amount, channel) {
    return await this.api.tx.dot.lock(channel, recipient, amount).signAndSend(account);
  }

  async waitForNextBlock() {
    const wait = new Promise(async (resolve, reject) => {
      let count = 0;
      const unsubscribe = await this.api.rpc.chain.subscribeNewHeads((header) => {
        count++
        if (count === 2) {
          unsubscribe();
          resolve();
        }
      });
    });
    return wait;
  }

}

module.exports.SubClient = SubClient;
