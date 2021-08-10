const { ethers } = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .should();

const IncentivizedInboundChannel = artifacts.require("IncentivizedInboundChannel");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");
const { createBeefyValidatorFixture, runBeefyLightClientFlow } = require("./beefy-helpers");


const MockRewardSource = artifacts.require("MockRewardSource");
const {
  deployBeefyLightClient, printTxPromiseGas
} = require("./helpers");
const fixture = require('./fixtures/full-flow.json');

describe("IncentivizedInboundChannel", function () {
  const interface = new ethers.utils.Interface(IncentivizedInboundChannel.abi)

  before(async function () {
    const totalNumberOfValidatorSigs = 100;
    const beefyFixture = await createBeefyValidatorFixture(
      totalNumberOfValidatorSigs
    )
    this.beefyLightClient = await deployBeefyLightClient(beefyFixture.root,
      totalNumberOfValidatorSigs);
    const merkleProof = await MerkleProof.new();
    const scaleCodec = await ScaleCodec.new();

    await IncentivizedInboundChannel.link(merkleProof);
    await IncentivizedInboundChannel.link(scaleCodec);
    runBeefyLightClientFlow(this.beefyLightClient, beefyFixture, totalNumberOfValidatorSigs, totalNumberOfValidatorSigs)
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

      // Send commitment
      const tx = this.channel.submit(
        ...Object.values(fixture.incentivizedSubmitInput),
      ).should.be.fulfilled
      printTxPromiseGas(tx)
      const { receipt } = await tx;

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
});
