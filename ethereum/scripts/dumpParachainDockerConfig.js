const fs  = require('fs');
const path = require('path');

const ETHApp = artifacts.require("ETHApp")
const ERC20App = artifacts.require("ERC20App")

module.exports = async (callback) => {
    try {
        let ethApp = await ETHApp.deployed()
        let erc20App = await ERC20App.deployed()

        fs.writeFileSync("../test/build/parachain.env", `ETH_APP_ID=${ethApp.address}\nERC20_APP_ID=${erc20App.address}\n`)

    } catch (error) {
        callback(error)
    }
}
