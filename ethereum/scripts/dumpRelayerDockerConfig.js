const TOML = require('@iarna/toml');
const temp = require('temp');
const fs  = require('fs');
const os = require('os');
const util = require('util');
const path = require('path');
const exec = require('child_process').exec;

const Bridge = artifacts.require("Bridge")
const ETHApp = artifacts.require("ETHApp")
const ERC20App = artifacts.require("ERC20App")

const dump = (bridge, ethApp, erc20App) => {

    temp.track();

    let cfgdir = "../build/tmp/relayer-config"

    let bridgeAbiFile = path.join(cfgdir, "Bridge.json")
    let ethAbiFile = path.join(cfgdir, "ETHApp.json")
    let erc20AbiFile = path.join(cfgdir, "ERC20App.json")

    fs.writeFileSync(bridgeAbiFile, JSON.stringify(bridge.abi, null, 2))
    fs.writeFileSync(ethAbiFile, JSON.stringify(ethApp.abi, null, 2))
    fs.writeFileSync(erc20AbiFile, JSON.stringify(erc20App.abi, null, 2))

    const config = {
        ethereum: {
            endpoint: "ws://ganache:8545/",
            bridge: {
                address: bridge.address,
                abi: "/opt/config/Bridge.json",
            },
            apps:{
                eth:{
                    address: ethApp.address,
                    abi: "/opt/config/ETHApp.json",
                },
                erc20:{
                    address: erc20App.address,
                    abi: "/opt/config/ERC20App.json",
                }
            }
        },
        substrate: {
            endpoint: "ws://parachain:9944/"
        }
    }
    fs.writeFileSync(path.join(cfgdir, "config.toml"), TOML.stringify(config))
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
