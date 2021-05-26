require("chai")
  .use(require("chai-as-promised"))
  .should();

const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const MockApp = artifacts.require("MockApp");

const { ethers } = require("ethers");

const {
  deployLightClientBridge
} = require("./helpers");

const makeCommitment = (messages) => {
  let encoded = ethers.utils.defaultAbiCoder.encode(
    ['tuple(address target, uint64 nonce, bytes payload)[]'],
    [messages]
  )
  return ethers.utils.solidityKeccak256(["bytes"], [encoded])
}

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
    this.lightClientBridge = await deployLightClientBridge();
  });

  describe("submit", function () {
    beforeEach(async function () {
      this.channel = await BasicInboundChannel.new(this.lightClientBridge.address,
        { from: owner }
      );
      this.app = await MockApp.new();
    });

    it("should accept a valid commitment and dispatch messages", async function () {
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
  });
});
