const uniqueFilename = require('unique-filename');
const TOML = require('@iarna/toml');
const fs = require('fs');
const os = require('os');

const Bridge = artifacts.require("Bridge")
const ETHApp = artifacts.require("ETHApp")
const ERC20App = artifacts.require("ERC20App")

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
            "descendants-until-final": 35,
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
        let bridge = await Bridge.deployed()
        let ethApp = await ETHApp.deployed()
        let erc20App = await ERC20App.deployed()

        dump(bridge, ethApp, erc20App)

    } catch (error) {
        callback(error)
    }
}
