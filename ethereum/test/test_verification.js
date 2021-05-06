const BigNumber = web3.BigNumber;
const {
  deployLightClientBridge, signatureSubstrateToEthereum, buildCommitment,
  createMerkleTree, deployGenericAppWithChannels, ChannelId, mine, lockupFunds
} = require("./helpers");
const ETHApp = artifacts.require("ETHApp");
const { keccakFromHexString, keccak } = require("ethereumjs-util");
const { blake2AsHex } = require("@polkadot/util-crypto");


require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const ethers = require("ethers");

contract("IncentivizedInboundChannel", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const userOne = accounts[1];
  const userTwo = accounts[2];
  const userThree = accounts[3];

  describe("initialize LightClientBridge", function () {
    beforeEach(async function () {
      const validatorsMerkleTree = createMerkleTree(["0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b", "0x25451A4de12dcCc2D166922fA938E900fCc4ED24"]);
      this.validatorsLeaf0 = validatorsMerkleTree.getHexLeaves()[0];
      this.validatorsLeaf1 = validatorsMerkleTree.getHexLeaves()[1];
      this.validator0PubKeyMerkleProof = validatorsMerkleTree.getHexProof(this.validatorsLeaf0);
      this.validator1PubKeyMerkleProof = validatorsMerkleTree.getHexProof(this.validatorsLeaf1);

      this.lightClientBridge = await deployLightClientBridge(validatorsMerkleTree.getHexRoot());
      const newCommitment = await this.lightClientBridge.newSignatureCommitment(
        blake2AsHex("0x8f667b0d8050a5d544d6fb139485847ea3f654bbbd065b08aad2f04d93ba2b7cae0000000000000000000000", 256),
        [parseBitfield("11")],
        signatureSubstrateToEthereum("0xbf866f5a4c66d5716e736d8e4737c8386c59e47fb618a03fc4cc3716f8e0a0987eec8076ee5824600d6f84825d693e4e5dac8f90ca27d639b520ab435e2d221900"),
        0,
        "0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b",
        this.validator0PubKeyMerkleProof
      );
      await mine(45);
      const currentId = await this.lightClientBridge.currentId();
      const completeCommitment = await this.lightClientBridge.completeSignatureCommitment(
        currentId.sub(new web3.utils.BN(1)),
        blake2AsHex("0x8f667b0d8050a5d544d6fb139485847ea3f654bbbd065b08aad2f04d93ba2b7cae0000000000000000000000", 256),
        [signatureSubstrateToEthereum("0x9646258c491fc1d32c03521d468b530ca0eb9f3acd0de9fea7255dc21ff63ddb2a45c06552a0f5cc079c9df7b320eca99c9819619a4efa816f1593cb908cd40901")],
        [1],
        ["0x25451A4de12dcCc2D166922fA938E900fCc4ED24"],
        [this.validator1PubKeyMerkleProof]
      );
      console.log(await this.lightClientBridge.latestMMRRoot());
      [channels, this.ethApp] = await deployGenericAppWithChannels(owner, this.lightClientBridge.address, ETHApp);
      this.inbound = channels.incentivized.inbound;
      this.POLKADOT_ADDRESS = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK"
      await lockupFunds(this.ethApp, userOne, this.POLKADOT_ADDRESS, 5000, ChannelId.Incentivized);
    });

    it("should successfully verify a commitment", async function() {
      const abi = this.ethApp.abi;
      const iChannel = new ethers.utils.Interface(abi);
      const polkadotSender = ethers.utils.formatBytes32String('fake-polkadot-address');
      const unlockFragment = iChannel.functions['unlock(bytes32,address,uint256)'];
      const payloadOne = iChannel.encodeFunctionData(unlockFragment, [polkadotSender, userTwo, 2]);
      const messageOne = {
        target: this.ethApp.address,
        nonce: 1,
        payload: payloadOne
      };
      const payloadTwo = iChannel.encodeFunctionData(unlockFragment, [polkadotSender, userThree, 5]);
      const messageTwo = {
        target: this.ethApp.address,
        nonce: 2,
        payload: payloadTwo
      };
      const messages = [messageOne, messageTwo];
      const commitment = buildCommitment(messages);
      const leaf = "0x3c390519c0130a826f3b96ec71ef9ce43f9fbc67fb0fc22ab4aaa1b3293394f7";
      const proofs = [
        "0xfe4a3df5f147d73b9edd54b6f665f68738a475243b0f5c583495b7fc34d7e127",
        "0x890fc5d842bac18f3af8ddabea7a28e39cd05fe13a5132460aaa5ef5c27c5969",
        "0xbe855ba734eef6617931c4bab11ce648892cad733df6a1c8287872eef656528a",
        "0x4bcd5e2699bd1870f4a85e0c91ebab5198b419b71b89c81b14e251b57b0f4348",
        "0xc1aad420d483a83b4dce6bf5e44c7af18653b46b0f6ca875ca0cb0ef6371adf7"
      ];
      const tx = await this.inbound.submit(
        messages,
        commitment,
        leaf,
        174,
        175,
        proofs,
        { from: userOne }
      );
      console.log(tx);
    });
  });
});

function parseBitfield (s) {
  return parseInt(s, 2)
}