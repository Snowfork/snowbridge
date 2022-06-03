const BigNumber = require('bignumber.js');
const { ethers } = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const IncentivizedInboundChannel = artifacts.require("IncentivizedInboundChannel");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");
const ParachainClient = artifacts.require("ParachainClient");

const MockRewardSource = artifacts.require("MockRewardController");
const {
  deployBeefyClient, printTxPromiseGas, createValidatorFixture, runBeefyClientFlow,
} = require("./helpers");

const fixture = require('./fixtures/beefy-relay-incentivized.json')
const submitInput = require('./fixtures/parachain-relay-incentivized.json')

describe("IncentivizedInboundChannel", function () {
  const interface = new ethers.utils.Interface(IncentivizedInboundChannel.abi)

  before(async function () {
    const numberOfSignatures = 8;
    const numberOfValidators = 24;
    const validatorFixture = await createValidatorFixture(fixture.params.commitment.validatorSetID-1, numberOfValidators)
    this.beefyClient = await deployBeefyClient(
      validatorFixture.validatorSetID,
      validatorFixture.validatorSetRoot,
      validatorFixture.validatorSetLength,
    );

    const merkleProof = await MerkleProof.new();
    const scaleCodec = await ScaleCodec.new();
    await ParachainClient.link(merkleProof);
    await ParachainClient.link(scaleCodec);
    this.parachainClient = await ParachainClient.new(this.beefyClient.address, 1000);

    await runBeefyClientFlow(fixture, this.beefyClient, validatorFixture, numberOfSignatures, numberOfValidators)
  });

  describe("submit", function () {
    beforeEach(async function () {
      const accounts = await web3.eth.getAccounts();
      const rewardSource = await MockRewardSource.new();
      this.channel = await IncentivizedInboundChannel.new(1, this.parachainClient.address,
        { from: accounts[0] }
      );
      await this.channel.initialize(accounts[0], rewardSource.address);
    });

    it("should accept a valid commitment and dispatch messages", async function () {
      const nonceBeforeSubmit = BigNumber(await this.channel.nonce());

      // Send commitment
      const tx = this.channel.submit(
        submitInput.params.bundle,
        submitInput.params.proof,
      ).should.be.fulfilled
      const { receipt } = await tx;

      const nonceAfterSubmit = BigNumber(await this.channel.nonce());
      nonceAfterSubmit.minus(nonceBeforeSubmit).should.be.bignumber.equal(1);

      let event;

      event = interface.decodeEventLog(
        'MessageDispatched(uint64,bool)',
        receipt.rawLogs[0].data,
        receipt.rawLogs[0].topics
      );
      event.id.eq(ethers.BigNumber.from(0)).should.be.true;
    });

    it("should refuse to replay commitments", async function () {
      // Submit messages
      await this.channel.submit(
        submitInput.params.bundle,
        submitInput.params.proof,
      ).should.be.fulfilled;

      // Submit messages again - should revert
      await this.channel.submit(
        submitInput.params.bundle,
        submitInput.params.proof,
      ).should.not.be.fulfilled;
    });

  });
});
