var Web3 = require('web3');
let { bundle } = require("@snowfork/snowbridge-types");
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { hexToU8a } = require("@polkadot/util");
const IncentivizedInboundChannel = require('../../ethereum/build/contracts/IncentivizedInboundChannel.json');

const endpoint = 'ws://localhost:8545';
const networkID = 344;
const PARACHAIN_ID = 200;
const PARACHAIN_RPC_ENDPOINT = 'ws://localhost:11144';
const ethereumSenderKey = '0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77';

start();

async function start() {

  var web3 = new Web3(new Web3.providers.WebsocketProvider(endpoint));
  const incentivizedInboundChannel = await new web3.eth.Contract(IncentivizedInboundChannel.abi, IncentivizedInboundChannel.networks[networkID].address);

  const parachainWsProvider = new WsProvider(PARACHAIN_RPC_ENDPOINT);
  const parachainApi = await ApiPromise.create({
    provider: parachainWsProvider,
    typesBundle: bundle,
  });
  const indexingPrefix = parachainApi.createType('IndexingPrefix', '(commitment');

  let ethereumDeliveredNonce = parseInt(await incentivizedInboundChannel.methods.nonce().call());
  console.log(`Latest nonce delivered to ethereum is ${ethereumDeliveredNonce}`);

  const latestParachainNonce = parseInt(await (await parachainApi.query.basicOutboundChannel.nonce()).toBigInt());
  console.log(`Latest nonce on parachain is: ${latestParachainNonce}`)

  if (ethereumDeliveredNonce === latestParachainNonce) {
    console.log(`All up to date`);
    process.exit(0);
  }

  const latestParachainBlockNumber = parseInt(await (await parachainApi.query.system.number()).toBigInt());
  console.log(`Latest block number on parachain is: ${latestParachainBlockNumber}`)

  // Search parachain blocks backwards until we find the commitment
  newCommitments = await searchForCommitment(latestParachainBlockNumber, ethereumDeliveredNonce);
  console.log(`Done searching, found the following new commitments and messages:`);
  console.dir(newCommitments, { depth: null })

  console.log(`Delivering each commitment and messages to Ethereum...`);
  await deliverMessages(newCommitments, incentivizedInboundChannel)

  process.exit(0);

  async function searchForCommitment(lastBlockNumber, nonceToFind) {
    let commitments = [];
    console.log(`Searching for commitment on parachain with nonce ${ethereumDeliveredNonce}, backwards from ${lastBlockNumber}`);
    let currentBlockNumber = lastBlockNumber;
    let headerFound = false;
    while (headerFound === false && currentBlockNumber !== -1) {
      console.log(`Checking header ${currentBlockNumber}`)
      digestLogs = await queryHeaderForDigest(currentBlockNumber)
      if (digestLogs.length > 0) {
        console.log(`Found logs at block ${currentBlockNumber}`)
        const logs = digestLogs.toHuman();
        const commitmentLog = logs[0]['Other'];
        digestItem = parachainApi.createType('AuxiliaryDigestItem', commitmentLog);
        console.log(digestItem.toHuman());
        console.log(`Querying offchain storage for messages`);
        messages = await queryOffchainMessagesForCommitment(digestItem)
        const nonces = messages.map(m => parseInt(m.nonce))
        console.log(`Found nonces: ${nonces}`)
        headerFound = nonces.includes(nonceToFind);
        if (!headerFound) {
          console.log(`Nonce not found in messages for commitment at block ${currentBlockNumber}`)
          commitments.push({ block: currentBlockNumber, hash: digestItem.commitment[1].toString(), messages });
        }
      }
      currentBlockNumber--;
    }
    return commitments.reverse();
  }

  async function queryHeaderForDigest(blockNumber) {
    const blockHash = await parachainApi.rpc.chain.getBlockHash(blockNumber);
    const header = await parachainApi.rpc.chain.getHeader(blockHash);
    return header.digest.logs
  }

  async function queryOffchainMessagesForCommitment(digestItem) {
    channel = digestItem.commitment[0];
    hash = digestItem.commitment[1];
    storageKey = parachainApi.createType('OffchainCommitmentKey', [indexingPrefix, channel, hash]);

    messages = await parachainApi.rpc.offchain.localStorageGet('PERSISTENT', Array.from(storageKey.toU8a()));
    channelMessages = parachainApi.createType('ChannelMessages', messages.toString()).toHuman();
    console.log({ channelMessages })
    return channelMessages;
  }

  async function deliverMessages(commitments, incentivizedInboundChannel) {
    const ethereumSenderAccount = web3.eth.accounts.privateKeyToAccount(ethereumSenderKey);
    for (const commitment of commitments) {
      console.log(`Delivering messages from parachain block ${commitment.block}`);
      tx = await incentivizedInboundChannel.methods.submit(commitment.messages, commitment.hash).send({ from: ethereumSenderAccount.address, gas: 5000000 });
      console.log(`Delivered, tx:`);
      console.dir(tx);
    }
  }
}
