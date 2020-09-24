// Example usage:
// truffle exec fetchDeployedAddrs.js [user-address]
// truffle exec fetchDeployedAddrs.js --network ropsten

const ETHApp = artifacts.require("ETHApp")
const ERC20App = artifacts.require("ERC20App")
const TestToken = artifacts.require("TestToken")

module.exports = async () => {
    try {
        const eth20AppInstance = await ETHApp.deployed()
        const erc20AppInstance = await ERC20App.deployed()
        const tokenInstance = await TestToken.deployed();

        const jsonStr = "{" +
            "\"ethApp\":\"" + eth20AppInstance.address + "\"," +
            "\"erc20App\":\"" + erc20AppInstance.address + "\"," +
            "\"token\":\"" + tokenInstance.address + "\"" +
            "}"

        return console.log(jsonStr);
    } catch (error) {
        return console.error({error})
    }
};
