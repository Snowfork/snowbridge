const BigNumber = web3.BigNumber;
const {
  deployBeefyLightClient, signatureSubstrateToEthereum,
  createMerkleTree, mine, catchRevert
} = require("./helpers");
const fixture = require('./fixtures/beefy-fixture-data.json');

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { expect } = require("chai");

describe("Beefy Light Client", function () {
  let accounts;
  let owner;
  let userOne;
  let userTwo;
  let userThree;

  before(async function () {
    accounts = await web3.eth.getAccounts();
    owner = accounts[0];
    userOne = accounts[1];
    userTwo = accounts[2];
    userThree = accounts[3];

    this.timeout(10 * 1000)

    this.validatorAddressOne = "0xE04CC55ebEE1cBCE552f250e85c57B70B2E2625b";
    this.validatorAddressTwo = "0x25451A4de12dcCc2D166922fA938E900fCc4ED24";
    this.validatorsMerkleTree = createMerkleTree([this.validatorAddressOne, this.validatorAddressTwo]);
    this.validatorsLeaf0 = this.validatorsMerkleTree.getHexLeaves()[0];
    this.validatorsLeaf1 = this.validatorsMerkleTree.getHexLeaves()[1];
    this.validator0PubKeyMerkleProof = this.validatorsMerkleTree.getHexProof(this.validatorsLeaf0);
    this.validator1PubKeyMerkleProof = this.validatorsMerkleTree.getHexProof(this.validatorsLeaf1);

    this.beefyLightClient = await deployBeefyLightClient(this.validatorsMerkleTree.getHexRoot(), this.validatorsMerkleTree.getLeaves().length);
  });

  it("runs new signature commitment and complete signature commitment correctly", async function () {
    const initialBitfield = await this.beefyLightClient.createInitialBitfield([0, 1], 2);
    expect(printBitfield(initialBitfield)).to.eq('11')

    await this.beefyLightClient.newSignatureCommitment(
      fixture.commitmentHash,
      initialBitfield,
      signatureSubstrateToEthereum(fixture.signature0),
      0,
      this.validatorAddressOne,
      this.validator0PubKeyMerkleProof
    ).should.be.fulfilled;

    const lastId = (await this.beefyLightClient.currentId()).sub(new web3.utils.BN(1));

    await mine(45);

    const bitfield = await this.beefyLightClient.createRandomBitfield(lastId);
    expect(printBitfield(bitfield)).to.eq('11')

    const validatorProof = {
      signatures: [signatureSubstrateToEthereum(fixture.signature0), signatureSubstrateToEthereum(fixture.signature1)],
      positions: [0, 1],
      publicKeys: [this.validatorAddressOne, this.validatorAddressTwo],
      publicKeyMerkleProofs: [this.validator0PubKeyMerkleProof, this.validator1PubKeyMerkleProof]
    }

    const beefyMMRLeaf = {
      parentNumber: 0,
      parentHash: '0x3dfb7482c3cbdce00c297b48c668e8e5fcfedac07296a9011e438e033c96de4a',
      parachainHeadsRoot: '0x3dfb7482c3cbdce00c297b48c668e8e5fcfedac07296a9011e438e033c96de4a',
      nextAuthoritySetId: 0,
      nextAuthoritySetLen: 0,
      nextAuthoritySetRoot: '0x3dfb7482c3cbdce00c297b48c668e8e5fcfedac07296a9011e438e033c96de4a',
    }

    const mmrProofItems = []

    await this.beefyLightClient.completeSignatureCommitment(
      lastId,
      fixture.commitment,
      validatorProof,
      beefyMMRLeaf,
      mmrProofItems,
    ).should.be.fulfilled;

    latestMMRRoot = await this.beefyLightClient.latestMMRRoot()
    expect(latestMMRRoot).to.eq('0xfab049d511b54f8d1169f85fe8add36c54a76c36d20737a80b1f0e72179b7d5f')
  });
});

function printBitfield(s) {
  return parseInt(s.toString(), 10).toString(2)
}
