require("chai")
  .use(require("chai-as-promised"))
  .should();

const MockApp = artifacts.require("MockApp");
const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");

const { ethers } = require("ethers");

const {
  deployBeefyLightClient
} = require("./helpers");

describe("BasicInboundChannel", function () {
  let accounts;
  let owner;
  let userOne;
  const interface = new ethers.utils.Interface(BasicInboundChannel.abi)
  const mockAppInterface = new ethers.utils.Interface(MockApp.abi);
  const mockAppUnlock = mockAppInterface.functions['unlock(uint256)'];

  before(async function () {
    accounts = await web3.eth.getAccounts();
    owner = accounts[0];
    userOne = accounts[1];
    this.beefyLightClient = await deployBeefyLightClient();
    const merkleProof = await MerkleProof.new();
    const scaleCodec = await ScaleCodec.new();
    await BasicInboundChannel.link(merkleProof);
    await BasicInboundChannel.link(scaleCodec);
  });

  describe("submit", function () {
    beforeEach(async function () {
      this.channel = await BasicInboundChannel.new(this.beefyLightClient.address,
        { from: owner }
      );
      this.app = await MockApp.new();

      const message1 = {
        target: this.app.address,
        nonce: 1,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [100])
      }
      const message2 = {
        target: this.app.address,
        nonce: 2,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [200])
      }

      this.submitInput = {
        messages: [message1, message2],
        _ownParachainHeadPartial: {
          parentHash: "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
          number: 2,
          stateRoot: "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
          extrinsicsRoot: "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
        },
        _parachainHeadProof: {
          pos: 0,
          width: 1,
          proof: [
            '0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7',
            '0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7'
          ]
        },
        _beefyMMRLeafPartial: {
          parentNumber: 2,
          parentHash: "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
          nextAuthoritySetId: 2,
          nextAuthoritySetLen: 2,
          nextAuthoritySetRoot: "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
        },
        _beefyMMRLeafIndex: 2,
        _beefyMMRLeafCount: 2,
        _beefyMMRLeafProof: [
          "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
          "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7"
        ]
      };
    });

    it("should accept a valid commitment and dispatch messages", async function () {

      // Send commitment
      const { receipt } = await this.channel.submit(
        ...Object.values(this.submitInput),
        { from: userOne }
      ).should.be.fulfilled

      let event;

      event = mockAppInterface.decodeEventLog(
        'Unlocked(uint256)',
        receipt.rawLogs[0].data,
        receipt.rawLogs[0].topics
      );
      event.amount.eq(ethers.BigNumber.from(100)).should.be.true;

      event = interface.decodeEventLog(
        'MessageDispatched(uint64,bool)',
        receipt.rawLogs[1].data,
        receipt.rawLogs[1].topics
      );
      event.nonce.eq(ethers.BigNumber.from(1)).should.be.true;

      event = mockAppInterface.decodeEventLog(
        'Unlocked(uint256)',
        receipt.rawLogs[2].data,
        receipt.rawLogs[2].topics
      );
      event.amount.eq(ethers.BigNumber.from(200)).should.be.true;

      event = interface.decodeEventLog(
        'MessageDispatched(uint64,bool)',
        receipt.rawLogs[3].data,
        receipt.rawLogs[3].topics
      );
      event.nonce.eq(ethers.BigNumber.from(2)).should.be.true;

    });

    it("should refuse to replay commitments", async function () {
      const message = {
        target: this.app.address,
        nonce: 1,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [100])
      }

      // Send commitment
      await this.channel.submit(
        ...Object.values(this.submitInput),
        { from: userOne }
      ).should.be.fulfilled;

      // Send commitment again - should revert
      await this.channel.submit(
        ...Object.values(this.submitInput),
        { from: userOne }
      ).should.not.be.fulfilled;

    });
  });
});
