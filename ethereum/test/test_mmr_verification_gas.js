const BigNumber = require('bignumber.js');
const { mergeKeccak256 } = require('./helpers');

require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const MMRVerification = artifacts.require("MMRVerification");
const SimpleMMRVerification = artifacts.require("SimplifiedMMRVerification");
const fixture15leaves = require('./fixtures/mmr-fixture-data-15-leaves.json');
const fixture15leavesSimplified = require('./fixtures/simplified-mmr-fixture-data-15-leaves.json');

describe("SimpleMMRVerification Contract", function () {
  describe("15-leaf, 26-node MMR", function () {

    context("15-leaf MMR from fixture gas tests", function () {

      let simplifiedMMRVerification;
      beforeEach(async function () {
        simplifiedMMRVerification = await SimpleMMRVerification.new();
      })

      fixture15leavesSimplified.proofs.forEach((proof, i) => {
        it(`should verify valid proof for leaf index ${i}`, async () => {
          const gas = await simplifiedMMRVerification.verifyLeafProof.estimateGas(
            fixture15leavesSimplified.rootHash, fixture15leavesSimplified.leaves[i],
            {
              items: fixture15leavesSimplified.proofs[i].items,
              order: fixture15leavesSimplified.proofs[i].order
            })
          console.log(`Gas used: ${gas}`);
        });
      });

    });
  });
});

describe("MMRVerification Contract", function () {
  describe("15-leaf, 26-node MMR", function () {

    context("15-leaf MMR from fixture gas tests", function () {
      let mmrVerification;
      beforeEach(async function () {
        mmrVerification = await MMRVerification.new();
      })

      fixture15leaves.proofs.forEach((proof, i) => {
        it(`should verify valid proof for leaf index ${i}`, async () => {
          const gas = await mmrVerification.verifyLeafProof.estimateGas(
            fixture15leaves.rootHash, fixture15leaves.leaves[i],
            proof.leafIndex, proof.leafCount, proof.items);
          console.log(`Gas used: ${gas}`);
        });
      });
    });
  });
});
