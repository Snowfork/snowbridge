const Web3 = require('web3');
const truffleContract = require("@truffle/contract");

const DEFAULT_ETHEREUM_ENDPOINT = "http://localhost:8545";

/**
 * The bridge client
 */
class BridgeClient {
    /**
     * @param {String} endpoint Ethereum endpoint url
     * @param {Object} applications [{name: "ETHApp", artifact: {...}, ...}]
     */
    constructor(endpoint = DEFAULT_ETHEREUM_ENDPOINT, applications) {
      var web3 = new Web3(new Web3.providers.HttpProvider(endpoint));
      this.web3 = web3;

      // Set application contracts
      let ethApp = applications.find((contract) => contract.name().toUpperCase() === "ETHAPP");
      if(!ethApp) {
        throw new Error('ethereum app contract cannot be undefined');
      }
      const ethAppInstance = truffleContract(ethApp.artifact);
      ethAppInstance.setProvider(web3.currentProvider);
      this.ethApp = ethAppInstance;

      let erc20App = applications.find((contract) => contract.name().toUpperCase() === "ERC20APP");
      if(!erc20App) {
        throw new Error('erc20 app contract cannot be undefined');
      }
      const erc20AppInstance = truffleContract(erc20App.artifact);
      erc20AppInstance.setProvider(web3.currentProvider);
      this.erc20App = erc20AppInstance;
    }

    /**
     * Get information about a transaction on Ethereum
     * @param {String} txHash the transaction's hash
     */
    getTx(txHash) {
        if (!txHash) {
          throw new Error('tx hash cannot be undefined');
        }
        this.web3.eth.getTransaction(txHash, function(err, result) {
            if (!err) {
                console.log('Tx result:\n:' + result)
            }
            else {
                console.log('Error!', err);
            }
        });
    }

    /**
     * Sends ETH from Ethereum to Substrate
     * @param {String} amount the amount of Ethereum to be sent to Substrate
     * @param {String} polkadotRecipient the recipient's address on polkadot
     */
    sendEth(amount, polkadotRecipient) {
        if (!amount) {
          throw new Error('amount cannot be undefined');
        }
        if (!polkadotRecipient) {
          throw new Error('polkadot recipient cannot be undefined');
        }
        const recipient = Buffer.from(polkadotRecipient.replace(/^0x/, ""), "hex");
        const weiAmount = this.web3.utils.toWei(Number(amount));

        const instance = await this.ethApp.deployed();
        const { logs } = await instance.sendETH(recipient, {
            from: this.account, // TODO: this will fail
            value: weiAmount,
            gas: 300000 // 300,000 Gwei
        });

        console.log("logs:", logs);
    }

}

module.exports.BridgeClient = BridgeClient;
