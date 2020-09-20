// Example usage:
// truffle exec sendErc20.js [amount] [token-contract-address] [polkadot-recipient]
// truffle exec sendErc20.js [amount] [token-contract-address] [polkadot-recipient] --network ropsten

module.exports = async () => {
  // Imports
  require("dotenv").config();
  const Web3 = require("web3");
  const HDWalletProvider = require("@truffle/hdwallet-provider");
  const truffleContract = require("@truffle/contract");
  const tokenContract = truffleContract(
    require("../build/contracts/TestToken.json")
  );
  const erc20AppContract = truffleContract(
    require("../build/contracts/ERC20App.json")
  );

  // Parameters
  const amount = Number(process.argv[4]);
  if (amount < 1) {
      console.log("Must provide a valid token amount")
      return
  }

  const tokenContractAddress = process.argv[5].toString();
  if (!tokenContractAddress) {
      console.log("Must provide a valid token contract address")
      return
  }

  const polkadotRecipient = process.argv[6].toString();
  if (!polkadotRecipient) {
    console.log("Must provide a Polkadot recipient")
    return
  }
  const recipient = Buffer.from(polkadotRecipient.replace(/^0x/, ""), "hex");

  // Set up provider and contracts
  let provider;
  const NETWORK_ROPSTEN = process.argv[7] === "--network" && process.argv[8] === "ropsten";
  if (NETWORK_ROPSTEN) {
      provider = new HDWalletProvider(
          process.env.MNEMONIC,
          "https://ropsten.infura.io/v3/".concat(process.env.INFURA_PROJECT_ID)
      );
  } else {
      provider = new Web3.providers.HttpProvider(process.env.LOCAL_PROVIDER);
  }

  const web3 = new Web3(provider);
  erc20AppContract.setProvider(web3.currentProvider);
  tokenContract.setProvider(web3.currentProvider);

  const accounts = await web3.eth.getAccounts();

  // 1. Send approval transaction
  try {
    const erc20AppAddress = await erc20AppContract.deployed().then(function(instance) {
      return instance.address;
    });

    console.log("1. Connected to TestToken contract, approving tokens to ERC20App contract...");
    const { logs } = await tokenContract.deployed().then(function (instance) {
      return instance.approve(erc20AppAddress, amount, {
        from: accounts[0],
        value: 0,
        gas: 300000 // 300,000 Gwei
      });
    });

    // Get event logs
    const event = logs.find(e => e.event === "Approval");

    // Parse event fields
    const approvalEvent = {
      owner: event.args.owner,
      spender: event.args.spender,
      value: Number(event.args.value)
    };

    console.log("ERC20 tokens successfully approved to ERC20App:\n", approvalEvent);
  } catch (error) {
    console.error({error})
  }

  // 2. Send ERC20 tokens to ERC20App
  try {
    const { logs } = await erc20AppContract.deployed().then(function (instance) {
      console.log("\n2. Connected to ERC20App contract, sending TestTokens...");
      return instance.sendERC20(recipient, tokenContractAddress, amount, {
        from: accounts[0],
        value: 0,
        gas: 300000 // 300,000 Gwei
      });
    });

    console.log("TestTokens succesfully sent to ERC20App:")

    // Get event logs
    const event = logs.find(e => e.event === "AppTransfer");

    // Parse event fields
    const appEvent = {
      data: event.args._data,
    };

    console.log(appEvent);
  } catch (error) {
    console.error({ error });
  }

  return
};
