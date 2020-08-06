const Verifier = artifacts.require("VerifierWeb3");

const BigNumber = web3.BigNumber;

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

contract("Verifier", function (accounts) {
  const operator = accounts[1];
  const userTwo = accounts[2];

  describe("Verifier contract deployment", function () {
    beforeEach(async function () {
      this.verifier = await Verifier.new(operator);
    });

    it("should deploy and initialize the contract", async function () {
      this.verifier.should.exist;

      const contractOperator = await this.verifier.operator();
      contractOperator.should.be.equal(operator);
    });
  });

  describe("Signature verification", function () {
    beforeEach(async function () {
      this.verifier = await Verifier.new(operator);
    });

    it("should recover a signer's address from a signed hash and signature", async function () {
      const message =  web3.utils.soliditySha3("test123xyz~~")
      const signature = await web3.eth.sign(message, userTwo);

      const prefixedMsg = await this.verifier.ethMessageHash(message)
      const signer = await this.verifier.recover(prefixedMsg, signature);
      signer.should.be.equal(userTwo);
    });

    it("should validate messages signed by the operator", async function () {
      const message =  web3.utils.soliditySha3("test123xyz~~")
      const signature = await web3.eth.sign(message, operator);

      const isOperator = await this.verifier.verify(message, signature);
      isOperator.should.be.equal(true);
    });
  });
});
