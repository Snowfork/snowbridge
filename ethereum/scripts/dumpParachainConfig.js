const ETHApp = artifacts.require("ETHApp")
const ERC20App = artifacts.require("ERC20App")

module.exports = async (callback) => {
    try {
        let ethApp = await ETHApp.deployed()
        let erc20App = await ERC20App.deployed()
        console.log(`export ETH_APP_ID=${ethAddr}\nexport ERC20_APP_ID=${erc20addr}\n`)
    } catch (error) {
        callback(error)
    }
}
