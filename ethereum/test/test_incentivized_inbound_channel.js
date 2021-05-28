const { ethers } = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .should();
const IncentivizedInboundChannel = artifacts.require("IncentivizedInboundChannel");
const MockApp = artifacts.require("MockApp");
const MockRewardSource = artifacts.require("MockRewardSource");
const {
  deployLightClientBridge
} = require("./helpers");

const makeCommitment = (messages) => {
  let encoded = ethers.utils.defaultAbiCoder.encode(
    ['tuple(address target, uint64 nonce, uint256 fee, bytes payload)[]'],
    [messages]
  )
  return ethers.utils.solidityKeccak256(["bytes"], [encoded])
}

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
    this.lightClientBridge = await deployLightClientBridge();
  });

  describe("submit", function () {
    beforeEach(async function () {
      const rewardSource = await MockRewardSource.new();
      this.channel = await IncentivizedInboundChannel.new(this.lightClientBridge.address,
        { from: owner }
      );
      await this.channel.initialize(owner, rewardSource.address);
      this.app = await MockApp.new();
    });

    it("should accept a valid commitment and dispatch messages", async function () {
      const message1 = {
        target: this.app.address,
        nonce: 1,
        fee: 64,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [100])
      }
      const message2 = {
        target: this.app.address,
        nonce: 2,
        fee: 64,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [200])
      }

      // Construct commitment hash from messages
      const commitment = makeCommitment([message1, message2]);

      // Send commitment
      const { receipt } = await this.channel.submit(
        [message1, message2],
        commitment, '0x0', 0, 0, [],
        { from: userOne }
      )

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

      // Construct commitment hash from messages
      const commitment = makeCommitment([message]);

      // Send commitment
      const { receipt } = await this.channel.submit(
        [message],
        commitment, '0x0', 0, 0, [],
        { from: userOne }
      ).should.be.fulfilled;

      let event;

      // Send commitment again - should revert
      await this.channel.submit(
        [message],
        commitment, '0x0', 0, 0, [],
        { from: userOne }
      ).should.not.be.fulfilled;

    });

    it("should not revert commitment submission if relayer cannot be rewarded", async function () {
      const message = {
        target: this.app.address,
        nonce: 1,
        fee: 1024,
        payload: mockAppInterface.encodeFunctionData(mockAppUnlock, [100])
      }

      // Construct commitment hash from messages
      const commitment = makeCommitment([message]);

      // Send commitment
      const { receipt } = await this.channel.submit(
        [message],
        commitment, '0x0', 0, 0, [],
        { from: userOne }
      ).should.be.fulfilled;

      let event = interface.decodeEventLog(
        'RelayerNotRewarded(address,uint256)',
        receipt.rawLogs[2].data,
        receipt.rawLogs[2].topics
      );
      (event.relayer === userOne).should.be.true
    });
  });
});
