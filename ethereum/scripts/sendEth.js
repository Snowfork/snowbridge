// Example usage:
// truffle exec sendEth.js [amount] [polkadot-recipient]
// truffle exec sendEth.js [amount] [polkadot-recipient] --network ropsten

const ETHApp = artifacts.require("ETHApp")

module.exports = async () => {
  // Parameters
  const ethAmountStr = process.argv[4].toString();
  if (!ethAmountStr) {
      console.log("Must provide an Ethereum amount")
      return
  }

  const polkadotRecipient = process.argv[5].toString();
  if (!polkadotRecipient) {
    console.log("Must provide a Polkadot recipient")
    return
  }
  const recipient = Buffer.from(polkadotRecipient.replace(/^0x/, ""), "hex");

  try {
    // Get current accounts
    const accounts = await web3.eth.getAccounts();

    const weiAmount = web3.utils.toWei(ethAmountStr)

    const ethApp = await ETHApp.deployed()
    const { logs } = await ethApp.sendETH(recipient, {
        from: accounts[0],
        value: weiAmount,
        gas: 300000 // 300,000 Gwei
    });

    console.log("Locked up ETH ...");

    // Get event logs
    const event = logs.find(e => e.event === "AppTransfer");

    console.log(event.args);
  } catch (error) {
    console.error({ error });
  }
  return;
};
