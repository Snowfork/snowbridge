require('dotenv').config({ path: require('find-config')('.env') })
const Web3 = require("web3");
const HDWalletProvider = require("@truffle/hdwallet-provider");
const truffleContract = require("@truffle/contract");

// const web3 = new Web3(provider);
// erc20AppContract.setProvider(web3.currentProvider);
// tokenContract.setProvider(web3.currentProvider);

const dump = (ethAddr, erc20addr) => {
    let tmpl = `export ETH_APP_ID=${ethAddr}\nexport ERC20_APP_ID=${erc20addr}\n`
    console.log(tmpl)
}


module.exports = async (callback) => {
    try {
        let provider;
        const NETWORK_ROPSTEN = process.argv[4] === "--network" && process.argv[5] === "ropsten";
        if (NETWORK_ROPSTEN) {
            provider = new HDWalletProvider(
                process.env.MNEMONIC,
                "https://ropsten.infura.io/v3/".concat(process.env.INFURA_PROJECT_ID)
            );
        } else {
            provider = new Web3.providers.HttpProvider(process.env.LOCAL_PROVIDER);
        }

        const web3 = new Web3(provider);

        const bridgeContract = truffleContract(
            require("../build/contracts/Bridge.json")
        );
        bridgeContract.setProvider(web3.currentProvider)

        const ethContract = truffleContract(
            require("../build/contracts/ETHApp.json")
        );
        ethContract.setProvider(web3.currentProvider)

        const erc20Contract = truffleContract(
            require("../build/contracts/ERC20App.json")
        );
        erc20Contract.setProvider(web3.currentProvider)

        let bridge = await bridgeContract.deployed()
        let ethApp = await ethContract.deployed()
        let erc20App = await erc20Contract.deployed()

        dump(ethApp.address, erc20App.address)

    } catch (error) {
        callback(error)
    }
}
