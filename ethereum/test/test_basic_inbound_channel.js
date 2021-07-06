require("chai")
  .use(require("chai-as-promised"))
  .should();

const BasicInboundChannel = artifacts.require("BasicInboundChannel");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");

const { ethers } = require("ethers");

const {
  deployBeefyLightClient, runBeefyLighClientFlow
} = require("./helpers");
const fixture = require('./fixtures/full-flow.json');

describe("BasicInboundChannel", function () {
  const interface = new ethers.utils.Interface(BasicInboundChannel.abi)

  before(async function () {
    this.beefyLightClient = await deployBeefyLightClient();
    const merkleProof = await MerkleProof.new();
    const scaleCodec = await ScaleCodec.new();
    await BasicInboundChannel.link(merkleProof);
    await BasicInboundChannel.link(scaleCodec);

    runBeefyLighClientFlow(this.beefyLightClient)
  });

  describe("submit", function () {
    beforeEach(async function () {
      this.channel = await BasicInboundChannel.new(this.beefyLightClient.address);
    });

    it("should accept a valid commitment and dispatch messages", async function () {
      const { receipt } = await this.channel.submit(
        ...Object.values(fixture.basicSubmitInput),
      ).should.be.fulfilled

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
