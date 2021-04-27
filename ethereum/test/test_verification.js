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
        "0xee236d6ab042180c67b3f4eb84d9ae3965497104b80b51cc0de1f72feb03d618",
        [3],
        signatureSubstrateToEthereum("0x3b721631b6dc6610c96d0b46a800fc292872558debe53927fec1728e6113aa8b05541728a9209f784dc1e893b904e55773b2bcef6eaebba00a07e188d8287fb200"),
        0,
        "0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b",
        this.validator0PubKeyMerkleProof
      );
      await mine(45);
      const currentId = await this.lightClientBridge.currentId();
      const completeCommitment = await this.lightClientBridge.completeSignatureCommitment(
        currentId.sub(new web3.utils.BN(1)),
        "0xee236d6ab042180c67b3f4eb84d9ae3965497104b80b51cc0de1f72feb03d618",
        [signatureSubstrateToEthereum("0x6129bff6e2b840dc43652add19fdcec0a1a271a96f5c382ecccded46e77e5e2c3c1e488134b1b634527bfd7da9903b302fe795ce9c0366cd6065a46e6f10066800")],
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
      const leaf = "0xc101580000006b298847473d05ebb1f1922abe40c12090299230e100c255ddefddb3ed0de1e284e929d0d7e236cc27f2e17380609cdaa2290d1cea197d14408b72223ee293a401000000000000000200000007b13d25743592825cea32c9a24ba67b50b7e90d92cbd1d0f4eab2dc94dba5c6";
      const proof = "0x58000000000000005a000000000000001006c6d6ef111e790dd52dd73b634046e52e10844a30db675bb076f101df4e578539ac2e734a4acd4dd720d218e59df8bb739df83904e2dd0e916dcf373a1e18bfb9102f03203bf2db14b09366c5e6ee08327490bda6287042a2698e36c951cbc2433e8086408f459e9951e50cb9e2537110ae7b25f3fb33022434403fd6fcd5ca";
      const proofs = proof.split("0x")[1].match(/.{1,64}/g).map(function(proof) {
        if (proof.length < 64) {
          return `0x${proof.padEnd(64, "0")}`;
        }
        return `0x${proof}`;
      });
      const tx = await this.inbound.submit(
        messages,
        commitment,
        keccakFromHexString(leaf),
        1,
        1,
        proofs,
        { from: userOne }
      );
      console.log(tx);
    });
  });
});
