const Web3 = require('web3');
const BigNumber = require('bignumber.js');

const ETHApp = require('../../../ethereum/build/contracts/ETHApp.json');
const ERC20App = require('../../../ethereum/build/contracts/ERC20App.json');
const ERC20 = require('../../../ethereum/build/contracts/ERC20.json');

/**
 * The Ethereum client for Bridge interaction
 */
class EthClient {
    /**
     * @param {String} endpoint Ethereum endpoint url
     * @param {String} ethAppAddress contract address of the deployed ETH Bridge application
     * @param {String} erc20AppAddress contract address of the deployed ERC20 Bridge application
     */
    constructor(endpoint, ethAppAddress, erc20AppAddress) {
      if(!endpoint) {
        throw new Error('network endpoint cannot be undefined');
      }
      if(!ethAppAddress) {
        throw new Error('ETH application contract address cannot be undefined');
      }
      if(!erc20AppAddress) {
        throw new Error('ERC20 application contract address cannot be undefined');
      }
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
      if(!tokenContractAddress) {
        throw new Error('erc20 token contract address cannot be undefined');
      }
      return new this.web3.eth.Contract(ERC20.abi, tokenContractAddress);
    }

    // TODO: Support wallet initialization by mnemonic
    async initWallet() {
      const accs = await this.web3.eth.getAccounts();
      this.web3.eth.defaultAccount = accs[0];
    };

    async getWallet() {
      return this.web3.eth.defaultAccount;
    };

    /**
     * Get information about a transaction on Ethereum
     * @param {String} txHash the transaction's hash
     */
    async getTx(txHash) {
        if (!txHash) {
          throw new Error('tx hash cannot be undefined');
        }
        return await this.web3.eth.getTransaction(txHash);
    }

    /**
     * Get the current ETH balance of an address
     * @param {String} account optional address to query, defaults to current wallet
     */
    async getEthBalance(account = this.web3.eth.defaultAccount) {
      return new BigNumber(await this.web3.eth.getBalance(account));
    }

    /**
     * Get the current ERC20 token balance of an address
     * @param {String} tokenContractAddress the ERC20 token's deployed contract address
     * @param {String} account optional address to query, defaults to current wallet
     */
    async getErc20Balance(tokenContractAddress, account = this.web3.eth.defaultAccount) {
      const erc20Instance = this.loadERC20Contract(tokenContractAddress);
      return await erc20Instance.methods.balanceOf(account).call();
    }

    /**
     * Get the current ERC20 token allowance of the appERC20 bridge application for the given account
     * @param {String} tokenContractAddress the ERC20 token's deployed contract address
     * @param {String} account optional address to query, defaults to current wallet
     */
    async getErc20Allowance(tokenContractAddress, account = this.web3.eth.defaultAccount) {
      const erc20Instance = this.loadERC20Contract(tokenContractAddress);
      return await erc20Instance.methods.allowance(account, this.appERC20._address).call();
    }

    /**
     * Sends ETH from Ethereum to Substrate
     * @param {String} amount the amount of ETH to be sent to Substrate
     * @param {String} polkadotRecipient the recipient's address on polkadot
     */
    async sendEth(amount, polkadotRecipient) {
      if (!amount) {
        throw new Error('amount cannot be undefined');
      }
      if (!polkadotRecipient) {
        throw new Error('polkadot recipient cannot be undefined');
      }
      const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

      let receipt = await this.appETH.methods.sendETH(recipientBytes).send({
        from: this.web3.eth.defaultAccount,
        gas: 500000,
        value: this.web3.utils.toBN(amount)
      });

      let tx = await this.web3.eth.getTransaction(receipt.transactionHash);

      let gasCost = BigNumber(tx.gasPrice).times(receipt.gasUsed);

      return { receipt, tx, gasCost }
    }

    /**
     * Approve ERC20 tokens to the appERC20 Bridge application
     * @param {String} amount the amount of ERC20 tokens to be approved
     * @param {String} tokenContractAddress the ERC20 token's deployed contract address
     */
    async approveERC20(amount, tokenContractAddress) {
      if (!amount) {
        throw new Error('amount cannot be undefined');
      }
      if (!tokenContractAddress) {
        throw new Error('token contract address cannot be undefined');
      }

      const erc20Instance = this.loadERC20Contract(tokenContractAddress);
      return await erc20Instance.methods.approve(this.appERC20._address, amount).send({
        from: this.web3.eth.defaultAccount
      });
    }

    /**
     * Sends ERC20 tokens from Ethereum to Substrate
     * @param {String} amount the amount of ERC20 tokens to be sent to Substrate
     * @param {String} tokenContractAddress the ERC20 token's deployed contract address
     * @param {String} polkadotRecipient the recipient's address on polkadot
     */
    async sendERC20(amount, tokenContractAddress, polkadotRecipient) {
      if (!amount) {
        throw new Error('amount cannot be undefined');
      }
      if (!tokenContractAddress) {
        throw new Error('token contract address cannot be undefined');
      }
      if (!polkadotRecipient) {
        throw new Error('polkadot recipient cannot be undefined');
      }
      const recipientBytes = Buffer.from(polkadotRecipient.replace(/^0x/, ""), 'hex');

      return await this.appERC20.methods.sendERC20(recipientBytes, tokenContractAddress, amount).send({
        from: this.web3.eth.defaultAccount
      });
    }
}

module.exports.EthClient = EthClient;
