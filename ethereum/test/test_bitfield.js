const { ethers } = require("hardhat");
const { expect } = require("chai");
const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");

const { printBitfield, createRandomPositions } = require("./helpers");

describe("Bitfield", function () {

  async function fixture() {
    let Bitfield = await ethers.getContractFactory("Bitfield");
    let bitfieldLib = await Bitfield.deploy();
    await bitfieldLib.deployed();
    return { bitfieldLib };
  }

  it("creates initial bitfield correctly in simple case", async function () {
    let { bitfieldLib } = await loadFixture(fixture);

    const positions = [0, 5, 8]
    const expected = '100100001'
    const n = 9
    const bitfield = await bitfieldLib.createBitfield(positions, n)

    expect(printBitfield(bitfield)).to.equal(expected)

  });

  it("creates initial bitfield correctly with bigger example", async function () {
    let { bitfieldLib } = await loadFixture(fixture);

    const positions = await createRandomPositions(140, 200)
    const bitfield = await bitfieldLib.createBitfield(
      positions, 200
    );

    expect(printBitfield(bitfield)).to.equal(positionsToBitfield(positions, 200))
  });

});

const positionsToBitfield = (positions) => {
  let bitfield = []
  for (let i = 0; i < positions.length; i++) {
    const position = positions[i];
    bitfield[position] = '1'
  }
  for (let i = 0; i < bitfield.length; i++) {
    if (bitfield[i] !== '1') {
      bitfield[i] = '0'
    }
  }
  return bitfield.reverse().join('')
}
