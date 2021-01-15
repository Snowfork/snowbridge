const uniqueFilename = require('unique-filename');
const TOML = require('@iarna/toml');
const fs = require('fs');
const os = require('os');

const Bridge = artifacts.require("Bridge")
const BasicOutChannel = artifacts.require("BasicOutChannel")
const IncentivizedOutChannel = artifacts.require("IncentivizedOutChannel")

const dump = (bridge, basicOutChannel, incentivizedOutChannel) => {

    const bridgeAbiFile = uniqueFilename(os.tmpdir(), "Bridge")
    const basicOutChannelAbiFile = uniqueFilename(os.tmpdir(), "BasicOutChannel")
    const incentivizedOutChannelAbiFile = uniqueFilename(os.tmpdir(), "IncentivizedOutChannel")

    fs.writeFileSync(bridgeAbiFile, JSON.stringify(bridge.abi, null, 2))
    fs.writeFileSync(basicOutChannelAbiFile, JSON.stringify(basicOutChannel.abi, null, 2))
    fs.writeFileSync(incentivizedOutChannelAbiFile, JSON.stringify(incentivizedOutChannel.abi, null, 2))

    const config = {
        ethereum: {
            endpoint: "ws://localhost:9545/",
            bridge: {
                address: bridge.address,
                abi: bridgeAbiFile,
            },
            apps:{
                basicOutChannel: {
                    address: basicOutChannel.address,
                    abi: basicOutChannelAbiFile,
                },
                incentivizedOutChannel: {
                    address: incentivizedOutChannel.address,
                    abi: incentivizedOutChannelAbiFile,
                }
            },
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
        let basicOutChannel = await BasicOutChannel.deployed()
        let incentivizedOutChannel = await IncentivizedOutChannel.deployed()

        dump(bridge, basicOutChannel, incentivizedOutChannel)

    } catch (error) {
        callback(error)
    }
}
