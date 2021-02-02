// Example usage:
// truffle exec sendEth.js [amount] [polkadot-recipient]
// truffle exec sendEth.js [amount] [polkadot-recipient] --network ropsten

const ETHApp = artifacts.require("ETHApp")

module.exports = async () => {
  // Parameters
  const amountEther = process.argv[4].toString();
  if (!amountEther) {
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

    console.log("Node-controlled accounts: ");
    console.log(accounts);

    const account = accounts[0];
    const amountWei = web3.utils.toWei(amountEther, "ether");

    let balanceWei = await web3.eth.getBalance(account)
    let balanceEth = web3.utils.fromWei(balanceWei)

    console.log(`Balance for sending account ${account}: ${balanceWei} wei (${balanceEth} ether)`);

    const ethApp = await ETHApp.deployed()
    const { logs } = await ethApp.lock(recipient, 0, {
        from: account,
        value: amountWei,
        gas: 300000 // 300,000 Gwei
    });

    console.log("Submitted transaction");


    const event = logs.find(e => e.event === "Locked");
    const event_decoded = {
      [event.event]: {
        sender: event.args.sender,
        recipient: event.args.recipient,
        amount: event.args.amount.toString(),
      }
    }
    console.log("Events:")
    console.log(JSON.stringify(event_decoded, null, 2));

    balanceWei = await web3.eth.getBalance(account)
    balanceEth = web3.utils.fromWei(balanceWei)
    console.log(`Balance for ${account} is now: ${balanceWei} wei (${balanceEth} ether)`);


  } catch (error) {
    console.error({ error });
  }
  return;
};
