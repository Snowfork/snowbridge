
const ERC20App = artifacts.require("ERC20App")
const TestToken = artifacts.require("TestToken")

module.exports = async () => {
  const accounts = await web3.eth.getAccounts();

  const account0 = accounts[0];
  const account1 = accounts[1];
  const testAmount = '100' + '000000000000000000';
  const testToken = await TestToken.deployed()

  const result = testToken.transfer(account1, testAmount, {
    from: account0
  });
  console.log({ result });
  const complete = await result;
  console.log({ complete });
}
