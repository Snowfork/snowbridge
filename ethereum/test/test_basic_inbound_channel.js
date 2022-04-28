const BigNumber = require('bignumber.js');
const { ethers } = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");
const ParachainClient = artifacts.require("ParachainClient");
const { createValidatorFixture, runBeefyClientFlow } = require("./beefy-helpers");

const {
  deployBeefyClient
} = require("./helpers");

const fixture = require('./fixtures/beefy-relay-basic.json')
const submitInput = require('./fixtures/parachain-relay-basic.json')

describe("BasicInboundChannel", function () {
  const interface = new ethers.utils.Interface(BasicInboundChannel.abi)

  before(async function () {
    const numberOfSignatures = 8;
    const numberOfValidators = 24;
    const validatorFixture = await createValidatorFixture(fixture.transactionParams.commitment.validatorSetId, numberOfValidators)
    this.beefyClient = await deployBeefyClient(
      validatorFixture.validatorSetId,
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
      this.channel = await BasicInboundChannel.new(this.parachainClient.address);
    });

    it("should accept a valid commitment and dispatch messages", async function () {
      const nonceBeforeSubmit = BigNumber(await this.channel.nonce());

      const { receipt } = await this.channel.submit(
        submitInput.transactionParams.bundle,
        submitInput.transactionParams.proof,
      ).should.be.fulfilled

      const nonceAfterSubmit = BigNumber(await this.channel.nonce());
      nonceAfterSubmit.minus(nonceBeforeSubmit).should.be.bignumber.equal(1);

      const event = interface.decodeEventLog(
        'MessageDispatched(uint64,bool)',
        receipt.rawLogs[0].data,
        receipt.rawLogs[0].topics
      );
      event.id.eq(ethers.BigNumber.from(0)).should.be.true;
    });

    it("should refuse to replay commitments", async function () {
      // Submit messages
      await this.channel.submit(
        submitInput.transactionParams.bundle,
        submitInput.transactionParams.proof,
      ).should.be.fulfilled;

      // Submit messages again - should revert
      await this.channel.submit(
        submitInput.transactionParams.bundle,
        submitInput.transactionParams.proof,
      ).should.not.be.fulfilled;
    });
  });
});
