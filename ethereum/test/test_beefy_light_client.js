const BigNumber = web3.BigNumber;
const {
  deployBeefyLightClient,
  createMerkleTree, mine,
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

    this.validatorsMerkleTree = createMerkleTree(fixture.validatorPublicKeys);
    this.beefyLightClient = await deployBeefyLightClient(this.validatorsMerkleTree.getHexRoot(),
      fixture.validatorPublicKeys.length);
  });

  it("encodes beefy mmr leaves correctly", async function () {
    encodedLeaf = await this.beefyLightClient.encodeMMRLeaf(fixture.beefyMMRLeaf)
    expect(encodedLeaf).to.eq(fixture.encodedBeefyMMRLeaf)
  });

  it("hashes beefy mmr leaves correctly", async function () {
    hashedLeaf = await this.beefyLightClient.hashMMRLeaf(encodedLeaf)
    expect(hashedLeaf).to.eq(fixture.hashedBeefyMMRLeaf)
  });

  it("runs new signature commitment and complete signature commitment correctly", async function () {
    const initialBitfield = await this.beefyLightClient.createInitialBitfield(fixture.validatorBitfield, 2);
    expect(printBitfield(initialBitfield)).to.eq('11')

    const commitmentHash = await this.beefyLightClient.createCommitmentHash(fixture.commitment);

    const tx = await this.beefyLightClient.newSignatureCommitment(
      commitmentHash,
      initialBitfield,
      fixture.signatures[0],
      0,
      fixture.validatorPublicKeys[0],
      fixture.validatorPublicKeyProofs[0]
    ).should.be.fulfilled

    const lastId = (await this.beefyLightClient.currentId()).sub(new web3.utils.BN(1));

    await mine(45);

    const bitfield = await this.beefyLightClient.createRandomBitfield(lastId);
    expect(printBitfield(bitfield)).to.eq('11')

    const validatorProof = {
      signatures: fixture.signatures,
      positions: [0, 1],
      publicKeys: fixture.validatorPublicKeys,
      publicKeyMerkleProofs: fixture.validatorPublicKeyProofs
    }

    await this.beefyLightClient.completeSignatureCommitment(
      lastId,
      fixture.commitment,
      validatorProof,
      fixture.beefyMMRLeaf,
      fixture.leafProof,
    )

    latestMMRRoot = await this.beefyLightClient.latestMMRRoot()
    expect(latestMMRRoot).to.eq(fixture.commitment.payload)
  });


});

function printBitfield(s) {
  return parseInt(s.toString(), 10).toString(2)
}
