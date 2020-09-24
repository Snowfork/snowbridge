// Example usage:
// truffle exec getTx.js [tx-hash]
// truffle exec getTx.js [tx-hash] --network ropsten

module.exports = async () => {

    const txHash = process.argv[4].toString();

    if (!txHash) {
        console.log("Please provide an transaction hash")
        return
    }

    var receipt = await web3.eth.getTransactionReceipt(txHash);
    console.log(receipt);
};
