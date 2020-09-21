require("dotenv").config();

const Web3 = require("web3");
const HDWalletProvider = require("@truffle/hdwallet-provider");
const truffleContract = require("@truffle/contract");
const uniqueFilename = require('unique-filename');
const TOML = require('@iarna/toml');
const fs = require('fs');
const os = require('os');

const dump = (bridge, ethApp, erc20App) => {

    const bridgeAbiFile = uniqueFilename(os.tmpdir(), "Bridge")
    const ethAbiFile = uniqueFilename(os.tmpdir(), "ETHApp")
    const erc20AbiFile = uniqueFilename(os.tmpdir(), "ERC20App")

    fs.writeFileSync(bridgeAbiFile, JSON.stringify(bridge.abi, null, 2))
    fs.writeFileSync(ethAbiFile, JSON.stringify(ethApp.abi, null, 2))
    fs.writeFileSync(erc20AbiFile, JSON.stringify(erc20App.abi, null, 2))

    const config = {
        ethereum: {
            endpoint: "ws://localhost:9545/",
            bridge: {
                address: bridge.address,
                abi: bridgeAbiFile,
            },
            apps:{
                eth:{
                    address: ethApp.address,
                    abi: ethAbiFile,
                },
                erc20:{
                    address: erc20App.address,
                    abi: erc20AbiFile,
                }
            }
        },
        substrate: {
            endpoint: "ws://127.0.0.1:9944/"
        }
    }
    console.log(TOML.stringify(config))
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

        dump(bridge, ethApp, erc20App)

    } catch (error) {
        callback(error)
    }
}
