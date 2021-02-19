// Example usage:
// truffle exec burnWrappedDOT.js [amount] [polkadot-recipient]
// truffle exec burnWrappedDOT.js [amount] [polkadot-recipient] --network ropsten

const DOTApp = artifacts.require("DOTApp");
const Token = artifacts.require("Token");

const BigNumber = require('bignumber.js');

const DOT_DECIMALS = 10;
const ETHER_DECIMALS = 18;

const wrapped = (amount) =>
  amount.multipliedBy(BigNumber(10).exponentiatedBy(ETHER_DECIMALS - DOT_DECIMALS));

const unwrapped = (amount) =>
  amount.dividedToIntegerBy(BigNumber(10).exponentiatedBy(ETHER_DECIMALS - DOT_DECIMALS));

module.exports = async () => {
  // Parameters
  const amount = process.argv[4].toString();
  if (!amount) {
      console.log("Must provide an amount (wrapped dots)")
      return
  }
  const amountWei = web3.utils.toWei(amount, "ether");

  const polkadotRecipient = process.argv[5].toString();
  if (!polkadotRecipient) {
    console.log("Must provide a Polkadot recipient")
    return
  }
  const recipient = Buffer.from(polkadotRecipient.replace(/^0x/, ""), "hex");

  try {
    // Get current accounts
    const accounts = await web3.eth.getAccounts();


    const account = accounts[1];

    const app = await DOTApp.deployed();
    const token = await Token.at(await app.token());

    let totalSupply = BigNumber(await token.totalSupply());
    console.log("Total Supply ", unwrapped(totalSupply).toString());

    let balanceWei = await token.balanceOf(account);
    let balance = unwrapped(BigNumber(balanceWei));

    console.log(`Balance for sending account ${account}: ${balanceWei} wei (${balance} wdot)`);

    return;

    const { logs } = await app.burn(
      recipient,
      amountWei,
      0,
      {
        from: account,
        gas: 300000 // 300,000 Gwei
      }
    );

    console.log("Submitted transaction");

    const event = logs.find(e => e.event === "Burned");
    const event_decoded = {
      [event.event]: {
        sender: event.args.sender,
        recipient: event.args.recipient,
        amount: event.args.amount.toString(),
      }
    }
    console.log("Events:")
    console.log(JSON.stringify(event_decoded, null, 2));

    balanceWei = await token.balanceOf(account);
    balance = web3.utils.fromWei(balanceWei);
    console.log(`Balance for ${account} is now: ${balanceWei} wei (${balance} wdot)`);

  } catch (error) {
    console.error({ error });
  }
  return;
};
