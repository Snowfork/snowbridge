const { ethers } = require("hardhat");
const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

const POLKADOT_ACCOUNT = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

describe("ETHApp", function () {

  async function baseFixture() {
    let [owner, user] = await ethers.getSigners();

    let ScaleCodec = await ethers.getContractFactory("ScaleCodec");
    let codec = await ScaleCodec.deploy();

    let MockOutboundChannel = await ethers.getContractFactory("MockOutboundChannel");
    let outboundChannel = await MockOutboundChannel.deploy()

    let Registry = await ethers.getContractFactory("ChannelRegistry");
    let registry = await Registry.deploy()

    await Promise.all([codec.deployed(), outboundChannel.deployed(), registry.deployed()])

    // Add mock inbound and outbound channels to registry
    await registry.updateChannel(0, owner.address, outboundChannel.address);

    let ETHApp = await ethers.getContractFactory("ETHApp", {
      signer: owner,
      libraries: {
        ScaleCodec: codec.address,
      },
    });

    let app = await ETHApp.deploy(
      owner.address,
      registry.address
    )
    await app.deployed();

    return {
      app, owner, user, channelID: 0
    }
  }

  describe("deposits", function () {

    async function depositsFixture() {
      return baseFixture();
    }

    it("should lock funds", async function () {
      const { app, user, channelID } = await loadFixture(depositsFixture);

      let beforeBalance = await ethers.provider.getBalance(app.address);
      let amount = ethers.utils.parseEther("0.25");

      await expect(app.connect(user).lock(
        POLKADOT_ACCOUNT,
        0,
        0,
        channelID,
        {
          value: amount,
        }
      )).to.emit(app, "Locked").withArgs(user.address, POLKADOT_ACCOUNT, amount, 0, 0)

      // Confirm contract's balance has increased
      const afterBalance = await ethers.provider.getBalance(app.address);
      expect(afterBalance).to.equal(beforeBalance.add(amount));
    });

    it("should lock funds and forward to destination parachain", async function () {
      const { app, user, channelID } = await loadFixture(depositsFixture);

      let beforeBalance = await ethers.provider.getBalance(app.address);
      let amount = ethers.utils.parseEther("0.25");

      await expect(app.connect(user).lock(
        POLKADOT_ACCOUNT,
        2048,
        0,
        channelID,
        {
          value: amount,
        }
      )).to.emit(app, "Locked").withArgs(user.address, POLKADOT_ACCOUNT, amount, 2048, 0)

      // Confirm contract's balance has increased
      const afterBalance = await ethers.provider.getBalance(app.address);
      expect(afterBalance).to.equal(beforeBalance.add(amount));
    });

    it("should not lock funds if amount is zero", async function() {
      const { app, user, channelID } = await loadFixture(depositsFixture);
      await expect(app.connect(user).lock(
        POLKADOT_ACCOUNT,
        0,
        0,
        channelID,
        {
          value: 0,
        }
      )).to.be.revertedWithCustomError(app, "MinimumAmount");
    });

    it("should not lock funds if amount is greater than 128-bits", async function() {
      const { app, user, channelID } = await loadFixture(depositsFixture);
      await expect(app.connect(user).lock(
        POLKADOT_ACCOUNT,
        0,
        0,
        channelID,
        {
          value: ethers.BigNumber.from("340282366920938463463374607431768211457"),
        }
      )).to.be.revertedWithCustomError(app, "MaximumAmount");
    });
  });

  describe("withdrawals", function () {

    async function withdrawalsFixture() {
      let { app, owner, user, channelID } = await baseFixture();
      await app.connect(user).lock(
        POLKADOT_ACCOUNT,
        0,
        0,
        channelID,
        {
          value: ethers.utils.parseEther("2"),
        }
      );
      return { app, owner, user };
    }

    it("should unlock", async function () {
      const { app, user } = await loadFixture(withdrawalsFixture);

      const amount = ethers.utils.parseEther("1")
      const beforeBalance = await ethers.provider.getBalance(app.address);
      const beforeRecipientBalance = await ethers.provider.getBalance(user.address);

      await expect(app.unlock(
        POLKADOT_ACCOUNT,
        user.address,
        amount,
      )).to.emit(app, "Unlocked").withArgs(POLKADOT_ACCOUNT, user.address, amount);

      const afterBalance = await ethers.provider.getBalance(app.address);
      const afterRecipientBalance = await ethers.provider.getBalance(user.address);

      expect(afterBalance).to.be.equal(beforeBalance.sub(amount));
      expect(afterRecipientBalance.sub(beforeRecipientBalance)).to.be.equal(amount);
    });

    it("should not unlock amounts greater than locked balance", async function () {
      const { app, user } = await loadFixture(withdrawalsFixture);

      const unlockAmount = ethers.utils.parseEther("2").add(1);

      await expect(app.unlock(
        POLKADOT_ACCOUNT,
        user.address,
        unlockAmount,
      )).to.be.revertedWithCustomError(app, "ExceedsBalance");
    });
  });
});
