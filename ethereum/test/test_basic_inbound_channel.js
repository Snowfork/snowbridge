const BigNumber = require('bignumber.js');
const { ethers } = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");
const { createBeefyValidatorFixture, runBeefyLightClientFlow } = require("./beefy-helpers");

const {
  deployBeefyLightClient
} = require("./helpers");
const fixture = require('./fixtures/full-flow-basic.json');

describe("BasicInboundChannel", function () {
  const interface = new ethers.utils.Interface(BasicInboundChannel.abi)

  before(async function () {
    const merkleProof = await MerkleProof.new();
    const scaleCodec = await ScaleCodec.new();
    await BasicInboundChannel.link(merkleProof);
    await BasicInboundChannel.link(scaleCodec);

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
      this.channel = await BasicInboundChannel.new(this.beefyLightClient.address);
    });

    it("should accept a valid commitment and dispatch messages", async function () {
      const nonceBeforeSubmit = BigNumber(await this.channel.nonce());

      const { receipt } = await this.channel.submit(
        ...Object.values(fixture.basicSubmitInput),
      ).should.be.fulfilled

      const nonceAfterSubmit = BigNumber(await this.channel.nonce());
      nonceAfterSubmit.minus(nonceBeforeSubmit).should.be.bignumber.equal(1);

      const event = interface.decodeEventLog(
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
        ...Object.values(fixture.basicSubmitInput)
      ).should.be.fulfilled;

      // Submit messages again - should revert
      await this.channel.submit(
        ...Object.values(fixture.basicSubmitInput),
      ).should.not.be.fulfilled;
    });
  });
});
