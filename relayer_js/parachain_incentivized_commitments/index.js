var Web3 = require('web3');
let { bundle } = require("@snowfork/snowbridge-types");
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { hexToU8a } = require("@polkadot/util");

const IncentivizedInboundChannel = require('../../ethereum/build/contracts/IncentivizedInboundChannel.json');

const endpoint = 'ws://localhost:8545';
const networkID = 344;

const PARACHAIN_ID = 200;
const PARACHAIN_RPC_ENDPOINT = 'ws://localhost:11144';

start();

async function start() {

  var web3 = new Web3(new Web3.providers.WebsocketProvider(endpoint));
  const incentivizedInboundChannel = await new web3.eth.Contract(IncentivizedInboundChannel.abi, IncentivizedInboundChannel.networks[networkID].address);

  const parachainWsProvider = new WsProvider(PARACHAIN_RPC_ENDPOINT);
  const parachainApi = await ApiPromise.create({
    provider: parachainWsProvider,
    typesBundle: bundle,
  });

  let ethereumDeliveredNonce = parseInt(await incentivizedInboundChannel.methods.nonce().call());
  if (ethereumDeliveredNonce === 0) {
    ethereumDeliveredNonce++
  }
  console.log(`Latest nonce delivered to ethereum is ${ethereumDeliveredNonce}`);

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
        console.log(logs)
        console.log(`Querying offchain storage for messages`);
        const commitment = logs[0]['Other'];
        messages = await queryOffchainMessagesForCommitment(commitment)
        const nonces = messages.map(m => parseInt(m.nonce))
        console.log(`Found nonces: ${nonces}`)
        headerFound = nonces.includes(nonceToFind);
        if (!headerFound) {
          console.log(`Nonce not found in messages for commitment at block ${currentBlockNumber}`)
          commitments.push({ block: currentBlockNumber, commitment, messages });
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

  async function queryOffchainMessagesForCommitment(commitment) {
    indexingPrefix = parachainApi.createType('IndexingPrefix', '(commitment');
    commitmentTyped = parachainApi.createType('Commitment', commitment);

    // TODO: This is done manually for now, because the aux digest item type doesn't work properly.
    const storageKey = Array.from(indexingPrefix.toU8a())
      .concat(Array.from(commitmentTyped[0].toU8a()))
      .concat(Array.from(hexToU8a(commitment).slice(2)));

    messages = await parachainApi.rpc.offchain.localStorageGet('PERSISTENT', storageKey);
    channelMessages = parachainApi.createType('ChannelMessages', messages.toString()).toHuman();
    return channelMessages;
  }

  async function deliverMessages(commitments, incentivizedInboundChannel) {
    for (const commitment of commitments) {
      console.log(`Delivering messages from parachain block ${commitment.block}`);
      tx = await incentivizedInboundChannel.methods.submit(commitment.messages, commitment.commitment).send();
      console.log(`Delivered, tx:`);
      console.dir(tx);
    }
  }
}
