// Simple script for sending transactions to a locally deployed Bank contract
module.exports = async () => {
    const Web3 = require("web3");

    // Contract abstraction
    const truffleContract = require("@truffle/contract");
    const contract = truffleContract(
      require("../build/contracts/EthereumApp.json")
    );

    // Parameters
    const ethAmountStr = process.argv[4].toString();
    if (!ethAmountStr) {
        console.log("Must provide an Ethereum amount")
        return
    }

    const polkadotRecipient = process.argv[5].toString();
    if (!polkadotRecipient) {
      console.log("Must provide a Polkadot recipient")
      return
    }
    const recipient = Buffer.from(polkadotRecipient, "hex");

    let provider = new Web3.providers.HttpProvider("http://localhost:9545");
    const web3 = new Web3(provider);
    contract.setProvider(web3.currentProvider);

    const AMOUNT = web3.utils.toWei("10");

    try {
      // Get current accounts
      const accounts = await web3.eth.getAccounts();

      const weiAmount = web3.utils.toWei(ethAmountStr)

      // Send lock transaction
      console.log("Connecting to contract....");
      const { logs } = await contract.deployed().then(function (instance) {
        console.log("Connected to contract, sending...");
        return instance.sendETH(recipient, {
          from: accounts[0],
          value: weiAmount,
          gas: 300000 // 300,000 Gwei
        });
      });

      console.log("Sent eth...");

      // Get event logs
      const event = logs.find(e => e.event === "Transfer");

      console.log(event.args);
    } catch (error) {
      console.error({ error });
    }
    return;
  };
