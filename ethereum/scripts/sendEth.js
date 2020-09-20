// Example usage:
// truffle exec sendEth.js [amount] [polkadot-recipient]
// truffle exec sendEth.js [amount] [polkadot-recipient] --network ropsten

module.exports = async () => {
  require("dotenv").config();
  const Web3 = require("web3");
  const HDWalletProvider = require("@truffle/hdwallet-provider");

  // Contract abstraction
  const truffleContract = require("@truffle/contract");
  const contract = truffleContract(
    require("../build/contracts/ETHApp.json")
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
  const recipient = Buffer.from(polkadotRecipient.replace(/^0x/, ""), "hex");

  // Set up provider and contracts
  let provider;
  const NETWORK_ROPSTEN = process.argv[6] === "--network" && process.argv[7] === "ropsten";
  if (NETWORK_ROPSTEN) {
      provider = new HDWalletProvider(
          process.env.MNEMONIC,
          "https://ropsten.infura.io/v3/".concat(process.env.INFURA_PROJECT_ID)
      );
  } else {
      provider = new Web3.providers.HttpProvider(process.env.LOCAL_PROVIDER);
  }

  const web3 = new Web3(provider);
  contract.setProvider(web3.currentProvider);

  try {
    // Get current accounts
    const accounts = await web3.eth.getAccounts();

    const weiAmount = web3.utils.toWei(ethAmountStr)

    // Send lock transaction
    console.log("Connecting to contract....");
    const instance = await contract.deployed()
    const { logs } = await instance.sendETH(recipient, {
        from: accounts[0],
        value: weiAmount,
        gas: 300000 // 300,000 Gwei
    });

    console.log("Sent eth...");

    // Get event logs
    const event = logs.find(e => e.event === "AppTransfer");

    console.log(event.args);
  } catch (error) {
    console.error({ error });
  }
  return;
};
