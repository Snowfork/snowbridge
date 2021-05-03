const BigNumber = web3.BigNumber;
const {
  deployLightClientBridge, signatureSubstrateToEthereum, buildCommitment,
  createMerkleTree, deployGenericAppWithChannels, ChannelId, mine, lockupFunds
} = require("./helpers");
const ETHApp = artifacts.require("ETHApp");
const { keccakFromHexString, keccak } = require("ethereumjs-util");

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
        "0xbe5e918869d703844eb1ae212e52162825737036f83ce8febb41a65ce3c41656",
        [parseBitfield("11")],
        signatureSubstrateToEthereum("0x254b25cc1983e66edbc0e531f0ea11d848b27ccdacdc4cc44ac783aacaa7bf6d3a702aefb5e2250dacb647c22fa7710fadbbe42e36864635eec11fcd0d46788001"),
        0,
        "0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b",
        this.validator0PubKeyMerkleProof
      );
      await mine(45);
      const currentId = await this.lightClientBridge.currentId();
      const completeCommitment = await this.lightClientBridge.completeSignatureCommitment(
        currentId.sub(new web3.utils.BN(1)),
        "0xbe5e918869d703844eb1ae212e52162825737036f83ce8febb41a65ce3c41656",
        [signatureSubstrateToEthereum("0xa97a20d7df37b94696534c668400213a2759deb70c868b17b101837c6a7966792c5ceb72665e459c2e632ebe1b41a21a5ecb253932027e7d1a5a45d43072a98700")],
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
      const leaf = "0xc101140000006246f3967d617df1412181ba09847c156a12750d0111fc34dc679797d79eeb1fbc36789e7a1e281436464229828f817d6612f7b477d66591ff96a9e064bcc98a01000000000000000200000007b13d25743592825cea32c9a24ba67b50b7e90d92cbd1d0f4eab2dc94dba5c6";
      console.log(keccakFromHexString(leaf));
      const proofs = [
        "0x7ab091f10e52fc1aa828867cee14c250110c24ca7d1bc4481a911fcb83c1bac0",
        "0x3c480c16b80148cdcd62385a3835fa93ad2119268429b7593f9df0f425f61934",
        "0x023da63409fac7ff8427cdaf06e32fa94a968eb52df8e4b7c3b0b74e8b8f2af3"
      ];
      const tx = await this.inbound.submit(
        messages,
        commitment,
        keccakFromHexString(leaf),
        20,
        22,
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