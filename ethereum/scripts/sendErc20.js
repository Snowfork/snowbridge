// Example usage:
// truffle exec sendErc20.js [amount] [polkadot-recipient]
// truffle exec sendErc20.js [amount] [polkadot-recipient] --network ropsten

const ERC20App = artifacts.require("ERC20App")
const TestToken = artifacts.require("TestToken")

module.exports = async () => {

  // Parameters
  const amount = Number(process.argv[4]);
  if (amount < 1) {
    console.log("Must provide a valid token amount")
    return
  }

  const polkadotRecipient = process.argv[5].toString();
  if (!polkadotRecipient) {
    console.log("Must provide a Polkadot recipient")
    return
  }
  const recipient = Buffer.from(polkadotRecipient.replace(/^0x/, ""), "hex");

  const accounts = await web3.eth.getAccounts();


  try {

    const erc20App = await ERC20App.deployed()
    const testToken = await TestToken.deployed()

    let result
    let event

    // Send approval transaction
    result = await testToken.approve(erc20App.address, amount, {
      from: accounts[0],
      value: 0,
      gas: 300000 // 300,000 Gwei
    });

    // Get event logs
    event = result.logs.find(e => e.event === "Approval");

    // Parse event fields
    const approvalEvent = {
      owner: event.args.owner,
      spender: event.args.spender,
      value: Number(event.args.value)
    };

    console.log("ERC20 tokens successfully approved to ERC20App:\n", approvalEvent);

    // Send ERC20 tokens to ERC20App
    result = await erc20App.lock(testToken.address, recipient, amount, 0, {
      from: accounts[0],
      value: 0,
      gas: 300000 // 300,000 Gwei
    });

    console.log("TestTokens succesfully sent to ERC20App:")

    // Get event logs
    event = result.logs.find(e => e.event === "Locked");

    // Parse event fields
    const appEvent = {
      data: event.args,
    };

    console.log(appEvent);
  } catch (error) {
    console.error({ error });
  }

  return
};
