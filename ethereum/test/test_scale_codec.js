const { ethers } = require("hardhat");
const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

describe("ScaleCodec", function () {

  async function fixture() {
    let ScaleCodec = await ethers.getContractFactory("ScaleCodec");
    let codec = await ScaleCodec.deploy();
    await codec.deployed();
    return { codec };
  }

  describe("encoding unsigned integers", function () {
    it("should encode uint256", async function () {
      let { codec } = await loadFixture(fixture);
      const output = await codec.encode256("12063978950259949786323707366460749298097791896371638493358994162204017315152");
      expect(output).to.be.equal("0x504d8a21dd3868465c8c9f2898b7f014036935fa9a1488629b109d3d59f8ab1a");
    });

    it("should encode uint128", async function () {
      let { codec } = await loadFixture(fixture);
      const output = await codec.encode128("35452847761173902980759433963665451267");
      expect(output).to.be.equal("0x036935fa9a1488629b109d3d59f8ab1a");
    });

    it("should encode uint64", async function () {
      let { codec } = await loadFixture(fixture);
      const output = await codec.encode64("1921902728173129883");
      expect(output).to.be.equal("0x9b109d3d59f8ab1a");
    });

    it("should encode uint32", async function () {
      let { codec } = await loadFixture(fixture);
      const output = await codec.encode32("447477849");
      expect(output).to.be.equal("0x59f8ab1a");
    });

    it("should encode uint16", async function () {
      let { codec } = await loadFixture(fixture);
      const output = await codec.encode16("6827");
      expect(output).to.be.equal("0xab1a");
    });
  });
});
