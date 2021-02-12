const TestToken = require('../../../ethereum/build/contracts/TestToken.json');
const EthClient = require('../../src/ethclient').EthClient;
const SubClient = require('../../src/subclient').SubClient;

async function start() {
  const networkID = '344';
  ethClient = await new EthClient("ws://localhost:8545", networkID);
  await ethClient.initialize();

  const account0 = ethClient.accounts[0];
  const account1 = ethClient.accounts[1];
  const testAmount = '100' + '000000000000000000';

  const result = await ethClient.transferERC20(account0, account1, testAmount);
  console.log(result);
  process.exit(0);
}


start();
