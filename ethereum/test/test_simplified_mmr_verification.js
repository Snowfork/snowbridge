const BigNumber = require('bignumber.js');

require("chai")
    .use(require("chai-as-promised"))
    .use(require("chai-bignumber")(BigNumber))
    .should();

const SimpleMMRVerification = artifacts.require("SimplifiedMMRVerification");
const fixture7leaves = require('./fixtures/simplified-mmr-fixture-data-7-leaves.json');
const fixture15leaves = require('./fixtures/simplified-mmr-fixture-data-15-leaves.json');


describe("Simple MMR Verification", function () {
   describe("7-leaf, 11-node MMR", function () {
       before(function () {
           console.log('                 7-leaf MMR:           ');
           console.log('                                       ');
           console.log('    Height 3 |      7');
           console.log('    Height 2 |   3      6     10');
           console.log('    Height 1 | 1  2   4  5   8  9    11');
           console.log('             | |--|---|--|---|--|-----|-');
           console.log('Leaf indexes | 0  1   2  3   4  5     6');
       })

       let simplifiedMMRVerification;
       beforeEach(async function () {
           simplifiedMMRVerification = await SimpleMMRVerification.new();
       })

       fixture7leaves.proofs.forEach((proof, i) => {
           it(`should verify valid proof for leaf index ${i}`, async () => {
               expect(await simplifiedMMRVerification.verifyInclusionProof.call(fixture7leaves.rootHash, fixture7leaves.leaves[i],
                   {
                       merkleProofItems: fixture7leaves.proofs[i].merkleProofItems,
                       merkleProofOrderBitField: fixture7leaves.proofs[i].merkleProofOrderBitField
                   })).to.be.true;
           });

           it(`should reject invalid proof for leaf index ${i}`, async () => {
               let j = i + 1;
               if (j >= fixture7leaves.proofs.length) {
                   j = 0;
               }
               expect(await simplifiedMMRVerification.verifyInclusionProof.call(fixture7leaves.rootHash, fixture7leaves.leaves[i],
                   {
                       merkleProofItems: fixture7leaves.proofs[j].merkleProofItems,
                       merkleProofOrderBitField: fixture7leaves.proofs[j].merkleProofOrderBitField
                   })).to.be.false;
           });
       });
   })

    describe("15-leaf, 26-node MMR", function () {
        before(function () {
            console.log('                                    15-leaf MMR:                            ');
            console.log('                                                                            ');
            console.log('    Height 4 |             15                                               ');
            console.log('    Height 3 |      7             14                22                      ');
            console.log('    Height 2 |   3      6     10      13       18        21       25        ');
            console.log('    Height 1 | 1  2   4  5   8  9   11  12   16  17   19   20   23  24  26  ');
            console.log('             | |--|---|--|---|--|-----|---|---|---|----|---|----|---|---|---');
            console.log('Leaf indexes | 0  1   2  3   4  5     6   7   8   9   10   11   12  13  14  ');
        })

        let simplifiedMMRVerification;
        beforeEach(async function () {
            simplifiedMMRVerification = await SimpleMMRVerification.new();
        })

        fixture15leaves.proofs.forEach((proof, i) => {
            it(`should verify valid proof for leaf index ${i}`, async () => {
                expect(await simplifiedMMRVerification.verifyInclusionProof.call(fixture15leaves.rootHash, fixture15leaves.leaves[i],
                    {
                        merkleProofItems: fixture15leaves.proofs[i].merkleProofItems,
                        merkleProofOrderBitField: fixture15leaves.proofs[i].merkleProofOrderBitField
                    })).to.be.true;
            });

            it(`should reject invalid proof for leaf index ${i}`, async () => {
                let j = i + 1;
                if (j >= fixture15leaves.proofs.length) {
                    j = 0;
                }
                expect(await simplifiedMMRVerification.verifyInclusionProof.call(fixture15leaves.rootHash, fixture15leaves.leaves[i],
                    {
                        merkleProofItems: fixture15leaves.proofs[j].merkleProofItems,
                        merkleProofOrderBitField: fixture15leaves.proofs[j].merkleProofOrderBitField
                    })).to.be.false;
            });
        });
    })

    describe("fooble", function () {

        let simplifiedMMRVerification;
        beforeEach(async function () {
            simplifiedMMRVerification = await SimpleMMRVerification.new();
        })

        it("foo", async () => {
            expect(await simplifiedMMRVerification.verifyInclusionProof.call(
                "0x4172f9eee09024a2002617d2c49f02747f1936ca5d953a561a91ab3e61c72fae",
                "0xf107355276f363ffb71384a57315ac8b62077ead1f807a2d68623b101a78978d",
                {
                    merkleProofItems: [
                        "0xdba9f82074947ab29470beb04898963588ea6b066c890c11ff58670a49d98b3b",
                        "0xf7623d5b5882972b619c55833566dcf943ac6641a7417ecc78dfe8619d7ac7a5",
                        "0x961265e4f829b417e83d1d5497c48d365d17b16baa3e1c90d384261e929cc28a",
                        "0x9838fe4a07b880e0a71a95d6349ae7411dce6e0bee5c6c823b5f685e759861e9",
                        "0xfcf54f17196c2e7f57e096b9c3270a44a8a02935d7044769d4f1a22d4ee0356c",
                        "0x87e640a6935602495b2329c9379c60fdcd53e961e6bd4f41647ee3ab47ffb4c8",
                        "0xc06c77e05586ee31f52adea0ffc4e473bdc8a1b3e85ec1e5850fa2bd020c8594"
                    ],
                    merkleProofOrderBitField: 5
                })).to.be.true;
        });

    })

});
