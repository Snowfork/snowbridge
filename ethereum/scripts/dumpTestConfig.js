const TOML = require('@iarna/toml');
const fs = require('fs');
const path = require('path');

const channelContracts = {
    basic: {
        inbound: artifacts.require("BasicInboundChannel"),
        outbound: artifacts.require("BasicOutboundChannel")
    },
    incentivized: {
        inbound: artifacts.require("IncentivizedInboundChannel"),
        outbound: artifacts.require("IncentivizedOutboundChannel")
    },
}

const ethAppContract = artifacts.require("ETHApp");
const erc20AppContract = artifacts.require('ERC20App');

const channels = {
    basic: {
        inbound: null,
        outbound: null,
        account_whitelist: null
    },
    incentivized: {
        inbound: null,
        outbound: null
    },
}

const dump = (tmpDir, channels) => {
    const config = {
        ethereum: {
            endpoint: "ws://localhost:8545/",
            "descendants-until-final": 3,
            channels: {
                basic: {
                    inbound: channels.basic.inbound.address,
                    outbound: channels.basic.outbound.address,
                    account_whitelist: [channels.ethApp.address, channels.erc20App.address]
                },
                incentivized: {
                    inbound: channels.incentivized.inbound.address,
                    outbound: channels.incentivized.outbound.address,
                },
            },
        },
        substrate: {
            endpoint: "ws://127.0.0.1:11144/"
        }
    }
    fs.writeFileSync(path.join(tmpDir, "config.toml"), TOML.stringify(config));
}

module.exports = async (callback) => {
    try {
        let configDir = process.argv[4].toString();
        channels.basic.inbound = await channelContracts.basic.inbound.deployed();
        channels.basic.outbound = await channelContracts.basic.outbound.deployed();
        channels.basic.account_whitelist = await web3.eth.getAccounts();
        channels.ethApp = await ethAppContract.deployed();
        channels.erc20App = await erc20AppContract.deployed();
        channels.incentivized.inbound = await channelContracts.incentivized.inbound.deployed();
        channels.incentivized.outbound = await channelContracts.incentivized.outbound.deployed();
        await dump(configDir, channels);
    } catch (error) {
        callback(error)
    }
}
