// Simple script for sending transactions to a locally deployed Bank contract
module.exports = async () => {
    const Web3 = require("web3");
  
    // Contract abstraction
    const truffleContract = require("@truffle/contract");
    const contract = truffleContract(
      require("../build/contracts/Bank.json")
    );

    // Default testing params
    const TARGET_APP_ID = Web3.utils.utf8ToHex(
      "target application's unique substrate identifier"
    );
    const RECIPIENT = Web3.utils.utf8ToHex(
        "1FRMM8PEiWXYax7rpS6X4XZX1aAAxSWx1CrKTyrVYhV24fg"
      );
    const AMOUNT = 10;

    let provider = new Web3.providers.HttpProvider("http://localhost:7545");
  
    const web3 = new Web3(provider);
    contract.setProvider(web3.currentProvider);

    try {
      // Get current accounts
      const accounts = await web3.eth.getAccounts();
  
      // Send lock transaction
      console.log("Connecting to contract....");
      const { logs } = await contract.deployed().then(function (instance) {
        console.log("Connected to contract, sending...");
        return instance.sendETH(TARGET_APP_ID, RECIPIENT, {
          from: accounts[0],
          value: AMOUNT,
          gas: 300000 // 300,000 Gwei
        });
      });
  
      console.log("Sent eth...");
  
      // Get event logs
      const event = logs.find(e => e.event === "AppEvent");
  
      // Parse event fields
      const appEvent = {
        target_app_ID: event.args._targetAppID,
        name: event.args._name,
        data: event.args._data,
      };
  
      console.log(appEvent);
    } catch (error) {
      console.error({ error });
    }
    return;
  };