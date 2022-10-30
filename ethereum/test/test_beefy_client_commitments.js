const { ethers, contract } = require("hardhat");
const { expect } = require("chai");
const { loadFixture, mine } = require("@nomicfoundation/hardhat-network-helpers");
const { beefyClientFixture1 } = require("./fixtures");

let SUBMIT_FINAL_2 = "submitFinal(uint256,(uint32,uint64,(bytes32,bytes,bytes)),(bytes[],uint256[],address[],bytes32[][]),(uint8,uint32,bytes32,uint64,uint32,bytes32,bytes32),(bytes32[],uint64))";

describe("BeefyClient", function () {

  it("runs commitment submission flow", async function () {
    let { beefyClient, fixtureData, user } = await loadFixture(beefyClientFixture1);

    const bitfield = await beefyClient.createInitialBitfield(
      fixtureData.params.proof.indices,
      3
    );

    await expect(beefyClient.connect(user).submitInitial(
      fixtureData.commitmentHash,
      fixtureData.params.commitment.validatorSetID,
      bitfield,
      {
        signature: fixtureData.params.proof.signatures[0],
        index: fixtureData.params.proof.indices[0],
        addr: fixtureData.params.proof.addrs[0],
        merkleProof: fixtureData.params.proof.merkleProofs[0]
      }
    )).to.emit(beefyClient, "NewRequest").withArgs(0, user.address);

    await mine(3);

    await expect(beefyClient.connect(user)[SUBMIT_FINAL_2](
      0,
      fixtureData.params.commitment,
      fixtureData.params.proof,
      fixtureData.params.leaf,
      fixtureData.params.leafProof
    )).to.emit(beefyClient, "NewMMRRoot").withArgs(
      fixtureData.params.commitment.payload.mmrRootHash,
      fixtureData.params.commitment.blockNumber,
    );

    let root = await beefyClient.latestMMRRoot()
    expect(root).to.eq(fixtureData.params.commitment.payload.mmrRootHash)
  });
});
