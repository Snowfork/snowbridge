const BigNumber = require('bignumber.js');
const { ethers } = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const IncentivizedInboundChannel = artifacts.require("IncentivizedInboundChannel");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");
const { createBeefyValidatorFixture, runBeefyLightClientFlow } = require("./beefy-helpers");

const MockRewardSource = artifacts.require("MockRewardSource");
const {
  deployBeefyLightClient, printTxPromiseGas
} = require("./helpers");
const fixture = require('./fixtures/full-flow-incentivized.json');

describe("IncentivizedInboundChannel", function () {
  let owner, userOne;
  const interface = new ethers.utils.Interface(IncentivizedInboundChannel.abi)

  before(async function () {
    [owner, userOne] = await web3.eth.getAccounts();
    const merkleProof = await MerkleProof.new();
    const scaleCodec = await ScaleCodec.new();
    await IncentivizedInboundChannel.link(merkleProof);
    await IncentivizedInboundChannel.link(scaleCodec);

    const totalNumberOfValidatorSigs = 100;
    const beefyFixture = await createBeefyValidatorFixture(
      totalNumberOfValidatorSigs
    )

    this.beefyLightClient = await deployBeefyLightClient(beefyFixture.root,
      totalNumberOfValidatorSigs);

    await runBeefyLightClientFlow(fixture, this.beefyLightClient, beefyFixture, totalNumberOfValidatorSigs, totalNumberOfValidatorSigs)
  });

  describe("submit", function () {
    beforeEach(async function () {
      const accounts = await web3.eth.getAccounts();
      const rewardSource = await MockRewardSource.new();
      this.channel = await IncentivizedInboundChannel.new(this.beefyLightClient.address,
        { from: accounts[0] }
      );
      await this.channel.initialize(accounts[0], rewardSource.address);
    });

    it("should accept a valid commitment and dispatch messages", async function () {
      const nonceBeforeSubmit = BigNumber(await this.channel.nonce());

      // Send commitment
      const tx = this.channel.submit(
        ...Object.values(fixture.incentivizedSubmitInput),
      ).should.be.fulfilled
      printTxPromiseGas(tx)
      const { receipt } = await tx;

      const nonceAfterSubmit = BigNumber(await this.channel.nonce());
      nonceAfterSubmit.minus(nonceBeforeSubmit).should.be.bignumber.equal(1);

      let event;

      event = interface.decodeEventLog(
        'MessageDispatched(uint64,bool)',
        receipt.rawLogs[0].data,
        receipt.rawLogs[0].topics
      );
      event.nonce.eq(ethers.BigNumber.from(1)).should.be.true;
      event.result.should.be.true;
    });

    it("should refuse to replay commitments", async function () {
      // Submit messages
      await this.channel.submit(
        ...Object.values(fixture.incentivizedSubmitInput),
      ).should.be.fulfilled;

      // Submit messages again - should revert
      await this.channel.submit(
        ...Object.values(fixture.incentivizedSubmitInput),
      ).should.not.be.fulfilled;
    });

  });

  describe("upgradeability", function () {
    beforeEach(async function () {
      const rewardSource = await MockRewardSource.new();
      this.channel = await IncentivizedInboundChannel.new(
        this.beefyLightClient.address,
        { from: owner }
      );
      await this.channel.initialize(owner, rewardSource.address);
      const abi = ["event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender)"];
      this.newBeefy = ethers.Wallet.createRandom().address;
      this.iface = new ethers.utils.Interface(abi);
    });
    
    it("should revert when called by non-admin", async function () {
      await this.channel.upgrade(
        this.newBeefy,
        {from: userOne}).should.be.rejectedWith(/AccessControl/);
    });
    
    it("should revert once BEEFY_UPGRADE_ROLE has been renounced", async function () {
      await this.channel.renounceRole(web3.utils.soliditySha3("BEEFY_UPGRADE_ROLE"), owner, {from: owner});
      await this.channel.upgrade(
        this.newBeefy,
        {from: owner}
      ).should.be.rejectedWith(/AccessControl/)
    })

    it("should succeed when called by BEEFY_UPGRADE_ROLE", async function () {
      const oldBeefy = await this.channel.beefyLightClient();
      await this.channel.upgrade(
        this.newBeefy,
        {from: owner}
      );
      const newBeefy = await this.channel.beefyLightClient();
      expect(newBeefy !== oldBeefy).to.be.true;
      expect(newBeefy === this.newBeefy).to.be.true;
    });

    it("BEEFY_UPGRADE_ROLE can change BEEFY_UPGRADE_ROLE", async function () {
      const newUpgrader = ethers.Wallet.createRandom().address;
      const tx = await this.channel.grantRole(web3.utils.soliditySha3("BEEFY_UPGRADE_ROLE"), newUpgrader);
      const event = this.iface.decodeEventLog('RoleGranted', tx.receipt.rawLogs[0].data, tx.receipt.rawLogs[0].topics);
      expect(event.account).to.equal(newUpgrader);
    });

    it("reverts when non-upgrader attempts to change BEEFY_UPGRADE_ROLE", async function () {
      const newUpgrader = ethers.Wallet.createRandom().address;
      await this.channel.grantRole(
        web3.utils.soliditySha3("BEEFY_UPGRADE_ROLE"),
        newUpgrader,
        {from: userOne}
      ).should.be.rejectedWith(/AccessControl/);
    })
  });
});
