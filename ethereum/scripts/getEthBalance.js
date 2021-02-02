// Example usage:
// truffle exec getEthBalance.js [user-address]
// truffle exec getEthBalance.js [user-address] --network ropsten

module.exports = async () => {
    try {
        const account = process.argv[4].toString();
        if (!account) {
            console.log("Please provide an Ethereum address to check balance")
            return
        }

        const balanceWei = await web3.eth.getBalance(account)
        const balanceEth = web3.utils.fromWei(balanceWei)

        return console.log(`${account} has ${balanceEth} Eth (${balanceWei} Wei)`)
    } catch (error) {
        console.error({error})
    }
};
