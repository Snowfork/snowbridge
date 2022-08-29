let { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
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
    })

    this.keyring = new Keyring({ type: 'sr25519' });
    this.alice = this.keyring.addFromUri('//Alice', { name: 'Alice' });
    this.bob = this.keyring.addFromUri('//Bob', { name: 'Bob' });
  }

  async queryAssetsAccountBalance(assetId, accountId) {
    let account = await this.api.query.assets.account(assetId, accountId);
    if(account.isNone) return BigNumber(0);
    return BigNumber(account.value.balance.toBigInt())
  }

  async subscribeAssetsAccountBalances(assetId, accountId, length) {
    const [promises, resolvers] = createPromiseResolverMap(length)

    // Setup our balance subscription and resolve each promise one by one
    let count = 0;
    const unsubscribe = await this.api.query.assets.account(assetId, accountId, (account) => {
      if(account.isNone) {
        resolvers[count](BigNumber(0));
      }
      else {
        resolvers[count](BigNumber(account.value.balance.toBigInt()));
      }
      count++;
      if (count === length) {
        unsubscribe();
      }
    });

    return promises;
  }

  async queryAccountBalance(accountId) {
    let {
      data: {
        free: balance
      }
    } = await this.api.query.system.account(accountId);
    return BigNumber(balance.toBigInt())
  }

  async subscribeAccountBalances(accountId, length) {
    const [promises, resolvers] = createPromiseResolverMap(length)

    // Setup our balance subscription and resolve each promise one by one
    let count = 0;
    const unsubscribe = await this.api.query.system.account(accountId, account => {
      let {
        data: {
          free: balance
        }
      } = account;
      resolvers[count](BigNumber(balance.toBigInt()));
      count++;
      if (count === length) {
        unsubscribe();
      }
    });

    return promises;
  }

  async recordEvents(eventSection, eventMethod) {
    await this.waitForNextBlock(); // Clear previous blocks events.
    const recorded = [];
    const unsubscribe = await this.api.query.system.events(async (events) => {
      events.forEach((record) => {
          const { event } = record;
          if (event.section === eventSection && event.method === eventMethod) {
            event.data.forEach((d) => recorded.push(d));
          }
      });
    });
    var subClient = this;
    return async function () {
      await subClient.waitForNextBlock(); // Wait till finalized then unsubscribe.
      await unsubscribe();
      return recorded;
    };
  }

  async waitForNextEvent({ eventSection, eventMethod, eventDataType }) {
    let foundData = new Promise(async (resolve, reject) => {
      const unsubscribe = await this.api.query.system.events((events) => {
        events.forEach((record) => {
          const { event, phase } = record;
          const types = event.typeDef;
          if (event.section === eventSection && event.method === eventMethod) {
            if (eventDataType === undefined) {
              unsubscribe();
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

  async burnETH(account, recipient, amount, channelId) {
    return await this.api.tx.ethApp.burn(channelId, recipient, amount).signAndSend(account);
  }

  async burnERC20(account, assetId, recipient, amount, channelId) {
    return await this.api.tx.erc20App.burn(channelId, assetId, recipient, amount).signAndSend(account);
  }

  async lockDOT(account, recipient, amount, channelId) {
    return await this.api.tx.dotApp.lock(channelId, recipient, amount).signAndSend(account);
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

  async queryIncentivizedOutboundChannelFee() {
    let fee = await this.api.query.incentivizedOutboundChannel.fee();
    return BigNumber(fee.toBigInt())
  }

}

// Creates an array of length `length` promises and an array of corresponding resolvers for those promises
function createPromiseResolverMap(length) {
  const promisesResolvers = new Array(length).fill().map(i => {
    let resolver;
    const promise = new Promise(async (resolve, reject) => {
      resolver = resolve;
    });
    return { promise, resolver };
  });
  const promises = promisesResolvers.map(i => i.promise);
  const resolvers = promisesResolvers.map(i => i.resolver);

  return [promises, resolvers]
}

module.exports.SubClient = SubClient;
