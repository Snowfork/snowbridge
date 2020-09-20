// Example usage:
// truffle exec getERC20Balance.js [user-address] [token-address]
// truffle exec getERC20Balance.js [user-address] [token-address] --network ropsten

module.exports = async () => {
    require("dotenv").config();
    const Web3 = require("web3");
    const HDWalletProvider = require("@truffle/hdwallet-provider");
    const BigNumber = require("bignumber.js")

    try {
        // Contract abstraction
        const truffleContract = require("@truffle/contract");
        const contract = truffleContract(require("../build/contracts/TestToken.json"));

        const account = process.argv[4].toString();
        const token = process.argv[5].toString();

        if (!account) {
            console.log("Please provide an Ethereum address to check balance")
            return
        }
        if (!token) {
            console.log("Please provide an ERC20 token address")
            return
        }

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

        const tokenInstance = await contract.at(token);

        const symbol = await tokenInstance.symbol()
        const balance = new BigNumber(await tokenInstance.balanceOf(account));

        return console.log(`${account} has ${balance} ${symbol} tokens`);
    } catch (error) {
        console.error({error})
    }
};
