module.exports = async () => {
    const Web3 = require("web3");

    try {
        const account = process.argv[4].toString();
        if (!account) {
            console.log("Please provide an Ethereum address to check balance")
            return
        }

        let provider = new Web3.providers.HttpProvider("http://localhost:9545");
        const web3 = new Web3(provider);

        const balanceWei = await web3.eth.getBalance(account)
        const balanceEth = web3.utils.fromWei(balanceWei)

        return console.log(`${account} has ${balanceEth} Eth (${balanceWei} Wei)`)
    } catch (error) {
        console.error({error})
    }
};
