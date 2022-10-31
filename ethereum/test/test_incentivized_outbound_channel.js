const { ethers } = require("hardhat");
const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

const {deployMockContract} = require('@ethereum-waffle/mock-contract');

const testPayload = ethers.utils.formatBytes32String("arbitrary-payload");

describe("IncentivizedOutboundChannel", function () {
  async function fixture() {
    let [owner, app, user] = await ethers.getSigners();

    let iface, abi;

    // mock reward source
    iface = new ethers.utils.Interface([
      "function handleFee(address feePayer, uint256 _amount)",
    ]);
    abi = JSON.parse(iface.format(ethers.utils.FormatTypes.json));
    let mockFeeController = await deployMockContract(owner, abi);
    await mockFeeController.mock.handleFee.returns();

    let IncentivizedOutboundChannel = await ethers.getContractFactory("IncentivizedOutboundChannel");
    let channel = await IncentivizedOutboundChannel.deploy();
    await channel.deployed();

    await channel.initialize(owner.address, mockFeeController.address, [app.address]);

    await channel.setFee(10);

    return { channel, app, user, mockFeeController };
  }

  describe("send", function () {
    it("should send messages out with the correct event and fields", async function () {
      let {channel, app, user} = await loadFixture(fixture);

      await expect(channel.connect(app).submit(
        user.address,
        testPayload,
        0,
      )).to.emit(channel, 'Message').withArgs(app.address, 1, 10, testPayload);
    });

    it("should increment nonces correctly", async function () {
      let {channel, app, user} = await loadFixture(fixture);

      await expect(channel.connect(app).submit(
        user.address,
        testPayload,
        0
      )).to.emit(channel, "Message").withArgs(app.address, 1, 10, testPayload);

      await expect(channel.connect(app).submit(
        user.address,
        testPayload,
        0
      )).to.emit(channel, "Message").withArgs(app.address, 2, 10, testPayload);

      await expect(channel.connect(app).submit(
        user.address,
        testPayload,
        0
      )).to.emit(channel, "Message").withArgs(app.address, 3, 10, testPayload);
    });

    it("should not send message if user cannot pay fee", async function () {
      let {channel, app, user, mockFeeController} = await loadFixture(fixture);

      await mockFeeController.mock.handleFee.reverts();

      await expect(channel.connect(app).submit(
        user.address,
        testPayload,
        0,
      )).to.be.reverted;

    });

  });

});
