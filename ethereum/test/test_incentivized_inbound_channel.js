const { ethers } = require("hardhat");
const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

const {deployMockContract} = require('@ethereum-waffle/mock-contract');

const submitInput = require('./fixtures/parachain-relay-incentivized.json')

describe("IncentivizedInboundChannel", function () {

  async function fixture() {
    let [owner, user] = await ethers.getSigners();

    let iface, abi;

    // mock parachain client
    iface = new ethers.utils.Interface([
      "function verifyCommitment(bytes32 commitment, bytes calldata opaqueProof) returns (bool)",
    ]);
    abi = JSON.parse(iface.format(ethers.utils.FormatTypes.json));
    let mockParachainClient = await deployMockContract(owner, abi);
    await mockParachainClient.mock.verifyCommitment.returns(true);

    // mock reward source
    iface = new ethers.utils.Interface([
      "function handleReward(address payable, uint128 _amount)",
    ]);
    abi = JSON.parse(iface.format(ethers.utils.FormatTypes.json));
    let mockRewardSource = await deployMockContract(owner, abi);
    await mockRewardSource.mock.handleReward.returns();

    let IncentivizedInboundChannel = await ethers.getContractFactory("IncentivizedInboundChannel");
    let channel = await IncentivizedInboundChannel.deploy(1, mockParachainClient.address);
    await channel.deployed();

    await channel.initialize(owner.address, mockRewardSource.address);

    return { channel, user };
  }

  describe("submit", function () {

    it("should accept a valid commitment and dispatch messages", async function () {
      let { channel } = await loadFixture(fixture);

      const nonceBeforeSubmit = await channel.nonce();

      await expect(channel.submit(
        submitInput.params.bundle,
        submitInput.params.proof,
      )).to.emit(channel, "MessageDispatched").withArgs(ethers.BigNumber.from(0), true);

      const nonceAfterSubmit = await channel.nonce();
      expect(nonceAfterSubmit.sub(nonceBeforeSubmit)).to.be.equal(1);
    });

    it("should refuse to replay commitments", async function () {
      let { channel } = await loadFixture(fixture);

      // Submit messages
      await channel.submit(
        submitInput.params.bundle,
        submitInput.params.proof,
      );

      // Submit messages again - should revert
      await expect(channel.submit(
        submitInput.params.bundle,
        submitInput.params.proof,
      )).to.be.reverted;

    });
  });
});
