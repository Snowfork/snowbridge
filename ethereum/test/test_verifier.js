const Verifier = artifacts.require("Verifier");

const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("Verifier", function (accounts) {
  const userOne = accounts[1];

  describe("Signature verification", function () {
    beforeEach(async function () {
      this.verifier = await Verifier.new();
    });

    it("should correctly validate signatures", async function () {
      const message =  web3.utils.soliditySha3("test123xyz~~")
      const signature = await web3.eth.sign(message, userOne);

      // Recover the signer address from the generated message and signature.
      const signer = await this.verifier.recover(message, signature);
      signer.should.be.equal(userOne);
    });
  });
});