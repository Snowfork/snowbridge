const BigNumber = require('bignumber.js');
const { ethers } = require("ethers");
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const BasicInboundChannelV2 = artifacts.require("BasicInboundChannelV2");
const MerkleProof = artifacts.require("MerkleProof");
const ScaleCodec = artifacts.require("ScaleCodec");
const { createBeefyValidatorFixture, runBeefyLightClientFlow } = require("./beefy-helpers");

const {
  deployBeefyLightClient
} = require("./helpers");
const fixture = require('./fixtures/full-flow-basicV2.json');

describe("BasicInboundChannelV2", function () {
  const interface = new ethers.utils.Interface(BasicInboundChannelV2.abi)

  before(async function () {
    const merkleProof = await MerkleProof.new();
    const scaleCodec = await ScaleCodec.new();
    await BasicInboundChannelV2.link(merkleProof);
    await BasicInboundChannelV2.link(scaleCodec);

    const totalNumberOfValidatorSigs = 100;
    const beefyFixture = await createBeefyValidatorFixture(
      totalNumberOfValidatorSigs
    )
    this.beefyLightClient = await deployBeefyLightClient(beefyFixture.root,
      totalNumberOfValidatorSigs);

    await runBeefyLightClientFlow(fixture, this.beefyLightClient, beefyFixture, totalNumberOfValidatorSigs, totalNumberOfValidatorSigs)
  });

  describe.skip("submit", function () {
    beforeEach(async function () {
      this.channel = await BasicInboundChannelV2.new(this.beefyLightClient.address);
    });

    it("should accept a valid commitment and dispatch messages", async function () {
        const userAccount = fixture.basicSubmitInputU1M1._leaf.account;
        const userNonceBeforeSubmit = BigNumber(await this.channel.userNonce(userAccount));
  
        await this.channel.submit(
            ...Object.values(fixture.basicSubmitInputU1M2),
          ).should.not.be.fulfilled
    
        const { receipt } = await this.channel.submit(
        ...Object.values(fixture.basicSubmitInputU1M1),
      ).should.be.fulfilled

      const userNonceAfterSubmit = BigNumber(await this.channel.userNonce(userAccount));
      userNonceAfterSubmit.minus(userNonceBeforeSubmit).should.be.bignumber.equal(1);

      const event = interface.decodeEventLog(
        'MessageDispatched(uint64,bool)',
        receipt.rawLogs[0].data,
        receipt.rawLogs[0].topics
      );
      event.nonce.eq(ethers.BigNumber.from(1)).should.be.true;
      event.result.should.be.true;

      const anotherUserAccount = fixture.basicSubmitInputU2M1._leaf.account;
      const anotherUserInitialNonce = BigNumber(await this.channel.userNonce(anotherUserAccount));

      await this.channel.submit(
        ...Object.values(fixture.basicSubmitInputU1M2),
      ).should.be.fulfilled

      const finaluserNonce = BigNumber(await this.channel.userNonce(userAccount));
      finaluserNonce.minus(userNonceAfterSubmit).should.be.bignumber.equal(1);

     await this.channel.submit(
      ...Object.values(fixture.basicSubmitInputU2M1),
    ).should.be.fulfilled

    const anotherUserFinalNonce = BigNumber(await this.channel.userNonce(anotherUserAccount));
    anotherUserFinalNonce.minus(anotherUserInitialNonce).should.be.bignumber.equal(1);

    });

    it("should refuse to replay commitments", async function () {
        const userAccount = fixture.basicSubmitInputU1M1._leaf.account;
        const nonceBeforeSubmit = BigNumber(await this.channel.userNonce(userAccount));

      // Submit messages
      await this.channel.submit(
        ...Object.values(fixture.basicSubmitInputU1M1)
      ).should.be.fulfilled;

      const nonceAfterSubmit = BigNumber(await this.channel.userNonce(userAccount));
      nonceAfterSubmit.minus(nonceBeforeSubmit).should.be.bignumber.equal(1);

      // Submit messages again - should revert
      await this.channel.submit(
        ...Object.values(fixture.basicSubmitInputU1M1),
      ).should.not.be.fulfilled;

      const nonceAfterFailedSubmit = BigNumber(await this.channel.userNonce(userAccount));
      nonceAfterFailedSubmit.minus(nonceAfterSubmit).should.be.bignumber.equal(0);

    });
  });
});
