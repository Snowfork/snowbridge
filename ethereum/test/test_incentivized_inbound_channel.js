const { ethers } = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .should();

const IncentivizedInboundChannel = artifacts.require("IncentivizedInboundChannel");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");

const MockApp = artifacts.require("MockApp");
const MockRewardSource = artifacts.require("MockRewardSource");
const {
  deployBeefyLightClient
} = require("./helpers");

describe("IncentivizedInboundChannel", function () {
  let accounts;
  let owner;
  let userOne;
  const interface = new ethers.utils.Interface(IncentivizedInboundChannel.abi)
  const mockAppInterface = new ethers.utils.Interface(MockApp.abi);
  const mockAppUnlock = mockAppInterface.functions['unlock(uint256)'];

  before(async function () {
    accounts = await web3.eth.getAccounts();
    owner = accounts[0];
    userOne = accounts[1];
    this.beefyLightClient = await deployBeefyLightClient();
    const merkleProof = await MerkleProof.new();
    const scaleCodec = await ScaleCodec.new();
    await IncentivizedInboundChannel.link(merkleProof);
    await IncentivizedInboundChannel.link(scaleCodec);
  });

  describe("submit", function () {
    beforeEach(async function () {
      const rewardSource = await MockRewardSource.new();
      this.channel = await IncentivizedInboundChannel.new(this.beefyLightClient.address,
        { from: owner }
      );
      await this.channel.initialize(owner, rewardSource.address);
      this.app = await MockApp.new();

      this.message1 = {
        target: this.app.address,
        nonce: 1,
        fee: 64,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [100])
      }

      this.message2 = {
        target: this.app.address,
        nonce: 2,
        fee: 64,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [200])
      }

      this.message3 = {
        target: this.app.address,
        nonce: 1,
        fee: 1024,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [100])
      }

      this.submitInput = {
        messages: [this.message1, this.message2],
        _ownParachainHeadPartial: {
          parentHash: "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
          number: 2,
          stateRoot: "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
          extrinsicsRoot: "0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7",
        },
        _parachainHeadsProof: [
          '0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7',
          '0x5e6f2ad7fd8ebf43b022baad65832bdc3616f562dfbff2721e29284c288111d7'
        ],
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
        fee: 64,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [100])
      }

      // Send commitment
      const { receipt } = await this.channel.submit(
        ...Object.values(this.submitInput),
        { from: userOne }
      ).should.be.fulfilled;

      let event;

      // Send commitment again - should revert
      await this.channel.submit(
        ...Object.values(this.submitInput),
        { from: userOne }
      ).should.not.be.fulfilled;

    });

  });
});
