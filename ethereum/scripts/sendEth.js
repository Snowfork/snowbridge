// Simple script for sending transactions to a locally deployed Bank contract
module.exports = async () => {
    const Web3 = require("web3");
  
    // Contract abstraction
    const truffleContract = require("@truffle/contract");
    const contract = truffleContract(
      require("../build/contracts/Bank.json")
    );


    const RECIPIENT = Buffer.from(
       "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48", "hex"
    );

    //const RECIPIENT = Web3.utils.hexToBytes("0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48")

  
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
        return instance.sendETH(RECIPIENT, {
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
        name: event.args._tag,
        data: event.args._data,
      };
  
      console.log(appEvent);
    } catch (error) {
      console.error({ error });
    }
    return;
  };