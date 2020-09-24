// Example usage:
// truffle exec getERC20Balance.js [user-address]
// truffle exec getERC20Balance.js [user-address] --network ropsten

const BigNumber = require("bignumber.js")
const TestToken = artifacts.require("TestToken")

module.exports = async () => {
    try {
        const account = process.argv[4].toString();
        const token = process.argv[5].toString();

        if (!account) {
            console.log("Please provide an Ethereum address to check balance")
            return
        }
        if (!token) {
            console.log("Please provide an ERC20 token address")
            return
        }

        const tokenInstance = await TestToken.deployed();

        const symbol = await tokenInstance.symbol()
        const balance = new BigNumber(await tokenInstance.balanceOf(account));

        return console.log(`${account} has ${balance} ${symbol} tokens`);
    } catch (error) {
        console.error({error})
    }
};
