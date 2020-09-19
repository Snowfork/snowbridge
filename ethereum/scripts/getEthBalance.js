// Example usage:
// truffle exec getEthBalance.js [user-address]
// truffle exec getEthBalance.js [user-address] --network ropsten

module.exports = async () => {
    require("dotenv").config();
    const Web3 = require("web3");
    const HDWalletProvider = require("@truffle/hdwallet-provider");

    try {
        const account = process.argv[4].toString();
        if (!account) {
            console.log("Please provide an Ethereum address to check balance")
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

        const balanceWei = await web3.eth.getBalance(account)
        const balanceEth = web3.utils.fromWei(balanceWei)

        return console.log(`${account} has ${balanceEth} Eth (${balanceWei} Wei)`)
    } catch (error) {
        console.error({error})
    }
};
