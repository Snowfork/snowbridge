const TestToken = require('../../../ethereum/build/contracts/TestToken.json');
const EthClient = require('../../src/ethclient').EthClient;
const SubClient = require('../../src/subclient').SubClient;

const polkadotRecipient = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const polkadotRecipientSS58 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const polkadotSenderSS58 = polkadotRecipientSS58;
const ethEndpoint = 'ws://localhost:8545';
const parachainEndpoint = 'ws://localhost:11144';
const testNetworkID = '344';

const TestTokenAddress = TestToken.networks[testNetworkID].address;

const ETH_TO_PARA_WAIT_TIME = 60000;
const PARA_TO_ETH_WAIT_TIME = 100000;

async function bootstrap() {
  const ethClient = new EthClient(ethEndpoint, testNetworkID);
  const subClient = new SubClient(parachainEndpoint);
  await subClient.connect();
  await ethClient.initialize();
  return { ethClient, subClient };
}

module.exports = {
  bootstrap, polkadotRecipient,
  polkadotRecipientSS58, polkadotSenderSS58,
  TestTokenAddress,
  ETH_TO_PARA_WAIT_TIME, PARA_TO_ETH_WAIT_TIME
};
