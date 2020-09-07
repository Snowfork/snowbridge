module.exports = async () => {
    const Web3 = require("web3");

    const txHash = process.argv[4].toString();
    if (!txHash) {
        console.log("Please provide an transaction hash")
        return
    }

    let provider = new Web3.providers.HttpProvider("http://localhost:9545");
    const web3 = new Web3(provider);

    var receipt = await web3.eth.getTransactionReceipt(txHash);
    console.log(receipt);
};
