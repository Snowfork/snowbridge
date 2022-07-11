const fs = require('fs');
const { env } = require('process');

const contracts = JSON.parse(fs.readFileSync('/tmp/snowbridge/contracts.json', 'utf8'));

const TestToken = contracts.contracts.TestToken;
const EthClient = require('../../src/ethclient').EthClient;
const SubClient = require('../../src/subclient').SubClient;

const polkadotRecipient = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const polkadotRecipientSS58 = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const polkadotSenderSS58 = polkadotRecipientSS58;
const treasuryAddressSS58 = "5EYCAe5jHEaRUtbinpdbTLuTyGiVt2TJGQPi9fdvVpNLNfSS";
const parachainEndpoint = 'ws://localhost:11144';
const testParachainEndpoint = 'ws://localhost:13144';
const infuraProjectId = env.INFURA_PROJECT_ID || '';
const ethEndpoint = env.ETH_WS_ENDPOINT || 'ws://localhost:8546';
const testNetworkID = env.ETH_NETWORK_ID || '15';

const TestTokenAddress = TestToken.address;

const ETH_TO_PARA_WAIT_TIME = 60000;
const PARA_TO_ETH_WAIT_TIME = 100000;

async function bootstrap() {
  const ethClient = new EthClient(ethEndpoint + '/' + infuraProjectId, testNetworkID);
  const subClient = new SubClient(parachainEndpoint);
  await subClient.connect();
  const testSubClient = new SubClient(testParachainEndpoint);
  await testSubClient.connect();
  await ethClient.initialize();
  return { ethClient, subClient, testSubClient };
}

module.exports = {
  bootstrap, polkadotRecipient,
  polkadotRecipientSS58, polkadotSenderSS58, treasuryAddressSS58,
  TestTokenAddress, ETH_TO_PARA_WAIT_TIME, PARA_TO_ETH_WAIT_TIME
};
