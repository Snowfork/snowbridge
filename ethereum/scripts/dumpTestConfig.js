const TOML = require('@iarna/toml');
const fs  = require('fs');
const os = require('os');
const path = require('path');

const Bridge = artifacts.require("Bridge")
const ETHApp = artifacts.require("ETHApp")
const ERC20App = artifacts.require("ERC20App")
const TestToken = artifacts.require("TestToken")

const jsonify = (abi) => JSON.stringify(abi, null, 2)

const dumpConfig = async (tmpDir, bridge, ethApp, erc20App, testToken) => {

    // Parachain Config
    fs.writeFileSync(path.join(tmpDir, "parachain.env"), `ETH_APP_ID=${ethApp.address}\nERC20_APP_ID=${erc20App.address}\n`)

    // Relayer Config
    let bridgeAbiFile = path.join(tmpDir, "Bridge.json")
    let ethAbiFile = path.join(tmpDir, "ETHApp.json")
    let erc20AbiFile = path.join(tmpDir, "ERC20App.json")

    fs.writeFileSync(bridgeAbiFile, jsonify(bridge.abi))
    fs.writeFileSync(ethAbiFile, jsonify(ethApp.abi))
    fs.writeFileSync(erc20AbiFile, jsonify(erc20App.abi))

    const config = {
        ethereum: {
            endpoint: "ws://localhost:8545/",
            bridge: {
                address: bridge.address,
                abi: bridgeAbiFile,
            },
            apps: {
                eth: {
                    address: ethApp.address,
                    abi: ethAbiFile,
                },
                erc20: {
                    address: erc20App.address,
                    abi: erc20AbiFile,
                }
            }
        },
        substrate: {
            endpoint: "ws://localhost:9944/"
        }
    }
    fs.writeFileSync(path.join(tmpDir, "config.toml"), TOML.stringify(config))

    // Test configuration
    const address = {
        ETHApp: ethApp.address,
        ERC20App: erc20App.address,
        TestToken: testToken.address
    }
    fs.writeFileSync(path.join(tmpDir, "test-config.json"), jsonify(address))

    return tmpDir
}

module.exports = async (callback) => {
    try {
        let configDir = process.argv[4].toString();
        if (!configDir) {
            console.log("Please provide a directory to write the config")
            return
        }

        let bridge = await Bridge.deployed()
        let ethApp = await ETHApp.deployed()
        let erc20App = await ERC20App.deployed()
        let testToken = await TestToken.deployed()

        await dumpConfig(configDir, bridge, ethApp, erc20App, testToken)

    } catch (error) {
        callback(error)
    }
}
