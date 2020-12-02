const TOML = require('@iarna/toml');
const fs  = require('fs');
const os = require('os');
const path = require('path');

const Bridge = artifacts.require("Bridge")
const ETHApp = artifacts.require("ETHApp")
const ERC20App = artifacts.require("ERC20App")
const TestToken = artifacts.require("TestToken")

const jsonify = (abi) => JSON.stringify(abi, null, 2)

const dumpConfig = async (tmpDir, contracts) => {

    // Relayer Config
    let bridgeAbiFile = path.join(tmpDir, "Bridge.json")
    let ethAbiFile = path.join(tmpDir, "ETHApp.json")
    let erc20AbiFile = path.join(tmpDir, "ERC20App.json")

    fs.writeFileSync(bridgeAbiFile, jsonify(contracts.bridge.abi))
    fs.writeFileSync(ethAbiFile, jsonify(contracts.ethApp.abi))
    fs.writeFileSync(erc20AbiFile, jsonify(contracts.erc20App.abi))

    const config = {
        ethereum: {
            endpoint: "ws://localhost:8545/",
            bridge: {
                address: contracts.bridge.address,
                abi: bridgeAbiFile,
            },
            apps: {
                eth: {
                    address: contracts.ethApp.address,
                    abi: ethAbiFile,
                },
                erc20: {
                    address: contracts.erc20App.address,
                    abi: erc20AbiFile,
                }
            }
        },
        substrate: {
            endpoint: "ws://localhost:11144/"
        }
    }
    fs.writeFileSync(path.join(tmpDir, "config.toml"), TOML.stringify(config))
}

module.exports = async (callback) => {
    try {
        let configDir = process.argv[4].toString();
        if (!configDir) {
            console.log("Please provide a directory to write the config")
            return
        }

        let contracts = {
            bridge: await Bridge.deployed(),
            ethApp: await ETHApp.deployed(),
            erc20App: await ERC20App.deployed()
        }

        await dumpConfig(configDir, contracts)

    } catch (error) {
        callback(error)
    }
}
