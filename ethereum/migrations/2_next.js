const ScaleCodec = artifacts.require("ScaleCodec");
const ETHApp = artifacts.require("ETHApp");
const ERC20App = artifacts.require("ERC20App");
const TestToken = artifacts.require("TestToken");
const ValidatorRegistry = artifacts.require("ValidatorRegistry");
const MerkleProof = artifacts.require("MerkleProof");
const Bitfield = artifacts.require("Bitfield");
const Web3Utils = require("web3-utils");

const channels = {
  basic: {
    inbound: {
      contract: artifacts.require("BasicInboundChannel"),
      instance: null
    },
    outbound: {
      contract: artifacts.require("BasicOutboundChannel"),
      instance: null,
    }
  },
  incentivized: {
    inbound: {
      contract: artifacts.require("IncentivizedInboundChannel"),
      instance: null
    },
    outbound: {
      contract: artifacts.require("IncentivizedOutboundChannel"),
      instance: null
    }
  },
}

const contracts = {
  polkadotrelaychainbridge: {
    contract: artifacts.require("PolkadotRelayChainBridge"),
    instance: null
  }
}

module.exports = function (deployer, network, accounts) {
  deployer.then(async () => {
    channels.basic.inbound.instance = await deployer.deploy(channels.basic.inbound.contract)
    channels.basic.outbound.instance = await deployer.deploy(channels.basic.outbound.contract)
    channels.incentivized.inbound.instance = await deployer.deploy(channels.incentivized.inbound.contract)
    channels.incentivized.outbound.instance = await deployer.deploy(channels.incentivized.outbound.contract)

    // Link libraries to applications
    await deployer.deploy(ScaleCodec);
    deployer.link(ScaleCodec, [ETHApp, ERC20App]);

    // Deploy applications
    await deployer.deploy(
      ETHApp,
      {
        inbound: channels.basic.inbound.instance.address,
        outbound: channels.basic.outbound.instance.address,
      },
      {
        inbound: channels.incentivized.inbound.instance.address,
        outbound: channels.incentivized.outbound.instance.address,
      },
    );

    await deployer.deploy(
      ERC20App,
      {
        inbound: channels.basic.inbound.instance.address,
        outbound: channels.basic.outbound.instance.address,
      },
      {
        inbound: channels.incentivized.inbound.instance.address,
        outbound: channels.incentivized.outbound.instance.address,
      },
    );

    await deployer.deploy(TestToken, 100000000, "Test Token", "TEST");

    // Link MerkleProof library to ValidatorRegistry
    await deployer.deploy(MerkleProof);
    deployer.link(MerkleProof, [ValidatorRegistry]);

    // TODO: Hardcoded for testing
    const root = "0xc1490f71b21f5700063d93546dbe860cc190e883734ee3f490b76de9e028db99";
    const numValidators = 2;
    const valRegistry = await deployer.deploy(ValidatorRegistry, root, numValidators);

    // Link Bitfield library to PolkadotRelayChainBridge
    await deployer.deploy(Bitfield);
    deployer.link(Bitfield, [contracts.polkadotrelaychainbridge.contract]);

    contracts.polkadotrelaychainbridge.instance = await deployer.deploy(contracts.polkadotrelaychainbridge.contract, valRegistry.address)
  })
};
