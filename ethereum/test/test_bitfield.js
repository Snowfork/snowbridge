const {
  deployBeefyLightClient, printBitfield, printTxPromiseGas
} = require("./helpers");

const { createRandomPositions } = require("./beefy-helpers");

require("chai")
  .use(require("chai-as-promised"))
  .should();

const { expect } = require("chai");

describe("Bitfield", function () {

  beforeEach(async function () {
    this.beefyLightClient = await deployBeefyLightClient(null,
      1);
  });

  it("creates initial bitfield correctly in simple case", async function () {
    const positions = [0, 5, 8]
    const expected = '100100001'
    const n = 9
    const bitfield = await this.beefyLightClient.createInitialBitfield(positions, n)

    expect(printBitfield(bitfield)).to.equal(expected)

  });

  it("creates initial bitfield correctly with bigger example", async function () {
    const positions = await createRandomPositions(140, 200)

    const bitfield = await this.beefyLightClient.createInitialBitfield(
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
