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
        blake2AsHex("0x3f1cbcda605962d8d409020591364c90014d048c590d233e639b02ed6bf27d150a0000000000000000000000", 256),
        "0x3f1cbcda605962d8d409020591364c90014d048c590d233e639b02ed6bf27d15",
        [parseBitfield("11")],
        signatureSubstrateToEthereum("0x7d19a8bda8914787cc73dfc4ca4580b579c099d5799c92ee9fc32452b50c2fd47828377cd3ccdcf7d2e578bec821fff23e48630f32bda9001785829f942d32c201"),
        0,
        "0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b",
        this.validator0PubKeyMerkleProof
      );
      await mine(45);
      const currentId = await this.lightClientBridge.currentId();
      const completeCommitment = await this.lightClientBridge.completeSignatureCommitment(
        currentId.sub(new web3.utils.BN(1)),
        blake2AsHex("0x3f1cbcda605962d8d409020591364c90014d048c590d233e639b02ed6bf27d150a0000000000000000000000", 256),
        [signatureSubstrateToEthereum("0xbeb718a5f6e109750b06a6c741ba6443b821b23d58899f71b16833f41429a2426f42838873a484e894b53cc8d8179ec387b6d612125bedc686a329b18ba9ec0d00")],
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
      const leaf = "0x691847389270c5a694cb03842e1948a6908da32330491d6446f0b2bde54ff0ed";
      const proofs = [
        "0xdb88e2974042621e001e27d98ed92b371c799bb44ff3ca1f5c5302b0b5de3f97",
        "0xa91d9032932c55fd562b886b21e1c235b8084f1e53216d2adf54c49d5f03c030",
      ];
      const tx = await this.inbound.submit(
        messages,
        commitment,
        leaf,
        10,
        11,
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