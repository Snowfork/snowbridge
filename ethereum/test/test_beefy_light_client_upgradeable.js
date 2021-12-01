// const { deployBeefyLightClient } = require("./helpers");

const { ethers, deployments } = require("hardhat");
const { expect } = require("chai");
const fixture = require("./fixtures/full-flow-basic.json");

const setupTest = deployments.createFixture(async ({deployments, getNamedAccounts, ethers}, options) => {
  await deployments.fixture(); // ensure you start from a fresh deployments
  const { deployer } = await getNamedAccounts();
  const BeefyLightClient = await ethers.getContract("BeefyLightClient", deployer);
  return {
    BeefyLightClient
  };
});

describe("Beefy Light Client Upgradeability", function () {
  beforeEach(async function () {
    // this.beefyLightClient = await deployBeefyLightClient();
    [this.deployer, this.newOwner] = await ethers.getSigners();
    this.beefyLightClient = (await setupTest()).BeefyLightClient
  });

  it("only owner can change owner", async function () {
    const owner = await this.beefyLightClient.owner();
    await this.beefyLightClient.transferOwnership(this.newOwner.address);
    const newOwner = await this.beefyLightClient.owner();
    expect(owner !== newOwner).to.be.true;
    await expect(
      this.beefyLightClient.transferOwnership(this.deployer.address)
    ).to.be.revertedWith("Ownable: caller is not the owner");
  });

  it("preserves state", async function () {
    const initialBitfield = await this.beefyLightClient.createInitialBitfield(
      fixture.completeSubmitInput.validatorProof.positions,
      2
    );

    const commitmentHash = await this.beefyLightClient.createCommitmentHash(
      fixture.completeSubmitInput.commitment
    );

    await this.beefyLightClient.newSignatureCommitment(
      commitmentHash,
      initialBitfield,
      fixture.completeSubmitInput.validatorProof.signatures[0],
      fixture.completeSubmitInput.validatorProof.positions[0],
      fixture.completeSubmitInput.validatorProof.publicKeys[0],
      fixture.completeSubmitInput.validatorProof.publicKeyMerkleProofs[0]
    );

    // expect(await this.beefyLightClient.currentId()).to.equal(1);
    // const bitfield = await ethers
    //   .getContractFactory("Bitfield")
    //   .then((c) => c.deploy());
    // const scaleCodec = await ethers
    //   .getContractFactory("ScaleCodec")
    //   .then((c) => c.deploy());
    // const BeefyLightClientV2 = await ethers.getContractFactory(
    //   "BeefyLightClientV2",
    //   {
    //     libraries: {
    //       ScaleCodec: scaleCodec.address,
    //       Bitfield: bitfield.address,
    //     },
    //   }
    // );
    // // upgrade
    // const beefyv2 = await upgrades.upgradeProxy(
    //   this.beefyLightClient.address,
    //   BeefyLightClientV2,
    //   { kind: "uups", unsafeAllowLinkedLibraries: true }
    // );

    // // expect to have the same state
    // expect(await beefyv2.currentId()).to.equal(1);
  });

  it("cannot reinitialize", async function () {
    const bitfield = await ethers
      .getContractFactory("Bitfield")
      .then((c) => c.deploy());
    const scaleCodec = await ethers
      .getContractFactory("ScaleCodec")
      .then((c) => c.deploy());
    const BeefyLightClientV2 = await ethers.getContractFactory(
      "BeefyLightClientFakeUpgrade",
      {
        libraries: {
          ScaleCodec: scaleCodec.address,
          Bitfield: bitfield.address,
        },
      }
    );
    // upgrade
    const v2 = await BeefyLightClientV2.deploy();
    await this.beefyLightClient.upgradeTo(v2.address);
    
    
    // upgrades.upgradeProxy(
    //   this.beefyLightClient.address,
    //   BeefyLightClientV2,
    //   { kind: "uups", unsafeAllowLinkedLibraries: true }
    // );

    // expect to have the same state
    expect(await this.beefyLightClient.currentId()).to.equal(0);
    await expect(
      this.beefyLightClient.initialize(this.beefyLightClient.address, this.beefyLightClient.address, 0)
    ).to.be.revertedWith("Initializable: contract is already initialized");
  });
});
