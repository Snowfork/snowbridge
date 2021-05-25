const rlp = require("rlp");

const deployAppWithMockChannels = async (deployer, channels, appContract, ...appContractArgs) => {
  const app = await appContract.new(
    ...appContractArgs,
    {
      inbound: channels[0],
      outbound: channels[1],
    },
    {
      inbound: channels[0],
      outbound: channels[1],
    },
    {
      from: deployer,
    }
  );

  return app;
}

const deployLightClientBridge = async (validatorsMerkleRoot) => {
  const validator = await ValidatorRegistry.new(
    validatorsMerkleRoot,
    2
  );
  const mmrVerification = await MMRVerification.new();
  const blake2b = await Blake2b.new();
  const lightClientBridge = await LightClientBridge.new(
    validator.address,
    mmrVerification.address,
    blake2b.address
  );

  return lightClientBridge;
}

function signatureSubstrateToEthereum(sig) {
  const recoveryId0 = web3.utils.hexToNumber(`0x${sig.slice(130)}`);
  const newRecoveryId0 = web3.utils.numberToHex(recoveryId0 + 27);
  const res = sig.slice(0, 130).concat(newRecoveryId0.slice(2));

  return res;
}

function createMerkleTree(leavesHex) {
  const leavesHashed = leavesHex.map(leaf => keccakFromHexString(leaf));
  const merkleTree = new MerkleTree(leavesHashed, keccak, { sort: false });

  return merkleTree;
}

async function mine(n) {
  for (let i = 0; i < n; i++) {
    web3.currentProvider.send({
      jsonrpc: '2.0',
      method: 'evm_mine',
      params: [],
      id: new Date().getTime()
    }, (err, res) => { });
  }
}

const lockupFunds = (contract, sender, recipient, amount, channel) => {
  return contract.lock(
    addressBytes(recipient),
    channel,
    {
      from: sender,
      value: amount.toString(),
    }
  )
}

const addressBytes = (address) => Buffer.from(address.replace(/^0x/, ""), "hex");

const encodeLog = (log) => {
  return rlp.encode([log.address, log.topics, log.data]).toString("hex")
}

const ChannelId = {
  Basic: 0,
  Incentivized: 1,
}

module.exports = {
  deployAppWithMockChannels,
  addressBytes,
  ChannelId,
  encodeLog,
};
