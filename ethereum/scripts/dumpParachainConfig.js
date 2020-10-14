const ETHApp = artifacts.require("ETHApp")
const ERC20App = artifacts.require("ERC20App")

module.exports = async (callback) => {
    try {
        let ethApp = await ETHApp.deployed()
        let erc20App = await ERC20App.deployed()
        console.log(`export ETH_APP_ID=${ethApp.address}\nexport ERC20_APP_ID=${erc20App.address}\n`)
    } catch (error) {
        callback(error)
    }
}
