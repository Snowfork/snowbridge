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
});