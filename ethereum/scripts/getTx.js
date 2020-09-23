// Example usage:
// truffle exec getTx.js [tx-hash]
// truffle exec getTx.js [tx-hash] --network ropsten

module.exports = async () => {
    require('dotenv').config({ path: require('find-config')('.env') })
    const Web3 = require("web3");
    const HDWalletProvider = require("@truffle/hdwallet-provider");

    const txHash = process.argv[4].toString();

    if (!txHash) {
        console.log("Please provide an transaction hash")
        return
    }

    let provider;
    const NETWORK_ROPSTEN = process.argv[5] === "--network" && process.argv[6] === "ropsten";
    if (NETWORK_ROPSTEN) {
        provider = new HDWalletProvider(
            process.env.MNEMONIC,
            "https://ropsten.infura.io/v3/".concat(process.env.INFURA_PROJECT_ID)
        );
    } else {
        provider = new Web3.providers.HttpProvider(process.env.LOCAL_PROVIDER);
    }

    const web3 = new Web3(provider);

    var receipt = await web3.eth.getTransactionReceipt(txHash);
    console.log(receipt);
};
