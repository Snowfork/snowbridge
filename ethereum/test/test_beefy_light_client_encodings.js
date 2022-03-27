const BigNumber = web3.BigNumber;
const Bitfield = artifacts.require("Bitfield");
const ScaleCodec = artifacts.require("ScaleCodec");
const ExposedBeefyLightClient = artifacts.require("ExposedBeefyLightClient");


require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const { expect } = require("chai");

describe("Beefy Light Client", function () {

  const iface = new ethers.utils.Interface(ExposedBeefyLightClient.abi);

  before(async function () {
    const bitfield = await Bitfield.new();
    const scaleCodec = await ScaleCodec.new();
    await ExposedBeefyLightClient.link(bitfield);
    await ExposedBeefyLightClient.link(scaleCodec);
    this.beefyLightClient = await ExposedBeefyLightClient.new();
  });

  it("encodes beefy commitment to SCALE-format (slow path)", async function () {
    let commitment = {
      payload: [{
        id: "0x6d68",
        data: "0x3048656c6c6f20576f726c6421"
      }],
      blockNumber: 5,
      validatorSetId: 0
    }

    let foo = await this.beefyLightClient.encodeCommitmentExposed.estimateGas(commitment);
    console.log(`Gas: ${foo}`);

    let encoded = await this.beefyLightClient.encodeCommitmentExposed(commitment).should.be.fulfilled

    expect(encoded).to.eq("0x046d68343048656c6c6f20576f726c6421050000000000000000000000");
  });

  it("encodes beefy commitment to SCALE-format (fast path)", async function () {
    let commitment = {
      payload: [{
        id: "0x6d68",
        data: "0xb5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c"
      }],
      blockNumber: 5,
      validatorSetId: 3
    }

    let foo = await this.beefyLightClient.encodeCommitmentExposed.estimateGas(commitment);
    console.log(`Gas: ${foo}`);

    let encoded = await this.beefyLightClient.encodeCommitmentExposed(commitment).should.be.fulfilled

    expect(encoded).to.eq("0x046d6880b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c050000000300000000000000");
  });

  it("encodes large beefy commitment to SCALE-format", async function () {
    let commitment = {
      payload: [
        {
          id: "0x6d68",
          data: "0x0707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707"
        },
        {
          id: "0x6d68",
          data: "0x0707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707"
        },
        {
          id: "0x6d68",
          data: "0x0707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707"
        },
    ],
      blockNumber: 5,
      validatorSetId: 3
    }

    let encoded = await this.beefyLightClient.encodeCommitmentExposed(commitment).should.be.fulfilled

    expect(encoded).to.eq("0x0c6d68050107070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707076d68050107070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707076d6805010707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707070707050000000300000000000000");
  });
});
