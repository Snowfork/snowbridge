const BigNumber = require('bignumber.js');
const { mergeKeccak256 } = require('./helpers');

require("chai")
    .use(require("chai-as-promised"))
    .use(require("chai-bignumber")(BigNumber))
    .should();

const MMRVerification = artifacts.require("MMRVerification");
const fixture7leaves = require('./fixtures/mmr-fixture-data-7-leaves.json');
const fixture15leaves = require('./fixtures/mmr-fixture-data-15-leaves.json');

describe("MMRVerification Contract", function () {
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

        // ---------------------------- Tree contents ----------------------------
        //  - For leaf nodes, node hash is the SCALE-encoding of the leaf data.
        //  - For parent nodes, node hash is the hash of it's children (left, right).
        //
        // 0xda5e6d0616e05c6a6348605a37ca33493fc1a15ad1e6a405ee05c17843fdafed // 1  LEAF NODE
        // 0xff5d891b28463a3440e1b650984685efdf260e482cb3807d53c49090841e755f // 2  LEAF NODE
        // 0xbc54778fab79f586f007bd408dca2c4aa07959b27d1f2c8f4f2549d1fcfac8f8 // 3  PARENT[1, 2] NODE
        // 0x7a84d84807ce4bbff8fb84667edf82aff5f2c5eb62e835f32093ee19a43c2de7 // 4  LEAF NODE
        // 0x27d8f4221cd6f7fc141ea20844c92aa8f647ac520853fbded619a46b1146ab8a // 5  LEAF NODE
        // 0x00b0046bd2d63fcb760cf50a262448bb2bbf9a264b0b0950d8744044edf00dc3 // 6  PARENT[4, 5] NODE
        // 0xe53ee36ba6c068b1a6cfef7862fed5005df55615e1c9fa6eeefe08329ac4b94b // 7  PARENT[3, 6] NODE
        // 0x99af07747700389aba6e6cb0ee5d553fa1241688d9f96e48987bca1d7f275cbe // 8  LEAF NODE
        // 0xc09d4a008a0f1ef37860bef33ec3088ccd94268c0bfba7ff1b3c2a1075b0eb92 // 9  LEAF NODE
        // 0xdad09f50b41822fc5ecadc25b08c3a61531d4d60e962a5aa0b6998fad5c37c5e // 10 PARENT[8, 9] NODE
        // 0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c // 11 LEAF NODE

        let mmrVerification;
        beforeEach(async function () {
            mmrVerification = await MMRVerification.new();
        })

        const root = "0xfc4f9042bd2f73feb26f3fc42db834c5f1943fa20070ddf106c486a478a0d561"

        it('should verify valid proof for leaf index 0 (node 1)', async function () {
            let leafNodeHash = "0xda5e6d0616e05c6a6348605a37ca33493fc1a15ad1e6a405ee05c17843fdafed"
            let proof = {
                leaf_index: 0,
                leaf_count: 7,
                items: [
                    "0xff5d891b28463a3440e1b650984685efdf260e482cb3807d53c49090841e755f",
                    "0x00b0046bd2d63fcb760cf50a262448bb2bbf9a264b0b0950d8744044edf00dc3",
                    mergeKeccak256("0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c", // bag right hand side peaks keccak(right, left)
                        "0xdad09f50b41822fc5ecadc25b08c3a61531d4d60e962a5aa0b6998fad5c37c5e")
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 1 (node 2)', async () => {
            let leafNodeHash = "0xff5d891b28463a3440e1b650984685efdf260e482cb3807d53c49090841e755f"
            let proof = {
                leaf_index: 1,
                leaf_count: 7,
                items: [
                    "0xda5e6d0616e05c6a6348605a37ca33493fc1a15ad1e6a405ee05c17843fdafed", // node 1
                    "0x00b0046bd2d63fcb760cf50a262448bb2bbf9a264b0b0950d8744044edf00dc3", // node 6
                    mergeKeccak256("0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c", // bag right hand side peaks keccak(right, left)
                        "0xdad09f50b41822fc5ecadc25b08c3a61531d4d60e962a5aa0b6998fad5c37c5e")
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 2 (node 4)', async () => {
            let leafNodeHash = "0x7a84d84807ce4bbff8fb84667edf82aff5f2c5eb62e835f32093ee19a43c2de7"
            let proof = {
                leaf_index: 2,
                leaf_count: 7,
                items: [
                    "0x27d8f4221cd6f7fc141ea20844c92aa8f647ac520853fbded619a46b1146ab8a",
                    "0xbc54778fab79f586f007bd408dca2c4aa07959b27d1f2c8f4f2549d1fcfac8f8",
                    mergeKeccak256("0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c", // bag right hand side peaks keccak(right, left)
                        "0xdad09f50b41822fc5ecadc25b08c3a61531d4d60e962a5aa0b6998fad5c37c5e")
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 3 (node 5)', async () => {
            let leafNodeHash = "0x27d8f4221cd6f7fc141ea20844c92aa8f647ac520853fbded619a46b1146ab8a"
            let proof = {
                leaf_index: 3,
                leaf_count: 7,
                items: [
                    "0x7a84d84807ce4bbff8fb84667edf82aff5f2c5eb62e835f32093ee19a43c2de7",
                    "0xbc54778fab79f586f007bd408dca2c4aa07959b27d1f2c8f4f2549d1fcfac8f8",
                    mergeKeccak256("0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c", // bag right hand side peaks keccak(right, left)
                        "0xdad09f50b41822fc5ecadc25b08c3a61531d4d60e962a5aa0b6998fad5c37c5e")
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 4 (node 8)', async () => {
            let leafNodeHash = "0x99af07747700389aba6e6cb0ee5d553fa1241688d9f96e48987bca1d7f275cbe"
            let proof = {
                leaf_index: 4,
                leaf_count: 7,
                items: [
                    "0xe53ee36ba6c068b1a6cfef7862fed5005df55615e1c9fa6eeefe08329ac4b94b",
                    "0xc09d4a008a0f1ef37860bef33ec3088ccd94268c0bfba7ff1b3c2a1075b0eb92",
                    "0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c"
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 5 (node 9)', async () => {
            let leafNodeHash = "0xc09d4a008a0f1ef37860bef33ec3088ccd94268c0bfba7ff1b3c2a1075b0eb92"
            let proof = {
                leaf_index: 5,
                leaf_count: 7,
                items: [
                    "0xe53ee36ba6c068b1a6cfef7862fed5005df55615e1c9fa6eeefe08329ac4b94b",
                    "0x99af07747700389aba6e6cb0ee5d553fa1241688d9f96e48987bca1d7f275cbe",
                    "0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c"
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 6 (node 11)', async () => {
            let leafNodeHash = "0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c"
            let proof = {
                leaf_index: 6,
                leaf_count: 7,
                items: [
                    "0xe53ee36ba6c068b1a6cfef7862fed5005df55615e1c9fa6eeefe08329ac4b94b",
                    "0xdad09f50b41822fc5ecadc25b08c3a61531d4d60e962a5aa0b6998fad5c37c5e"
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should not verify invalid proofs', async () => {
            let leafNodeHash = "0x0000000000000000000000000000000000000000000000000000000000123456"
            let proof = {
                leaf_index: 5,
                leaf_count: 7,
                items: [
                    "0xe53ee36ba6c068b1a6cfef7862fed5005df55615e1c9fa6eeefe08329ac4b94b",
                    "0x99af07747700389aba6e6cb0ee5d553fa1241688d9f96e48987bca1d7f275cbe",
                    "0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c"
                ]
            }

            // Stored value is not 0x000123
            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.false
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

        // ---------------------------- Tree contents ----------------------------
        //  - For leaf nodes, node hash is the SCALE-encoding of the leaf data.
        //  - For parent nodes, node hash is the hash of it's children (left, right).
        //
        // 0xda5e6d0616e05c6a6348605a37ca33493fc1a15ad1e6a405ee05c17843fdafed // 1  LEAF NODE
        // 0xff5d891b28463a3440e1b650984685efdf260e482cb3807d53c49090841e755f // 2  LEAF NODE
        // 0xbc54778fab79f586f007bd408dca2c4aa07959b27d1f2c8f4f2549d1fcfac8f8 // 3  PARENT[1, 2] NODE
        // 0x7a84d84807ce4bbff8fb84667edf82aff5f2c5eb62e835f32093ee19a43c2de7 // 4  LEAF NODE
        // 0x27d8f4221cd6f7fc141ea20844c92aa8f647ac520853fbded619a46b1146ab8a // 5  LEAF NODE
        // 0x00b0046bd2d63fcb760cf50a262448bb2bbf9a264b0b0950d8744044edf00dc3 // 6  PARENT[4, 5] NODE
        // 0xe53ee36ba6c068b1a6cfef7862fed5005df55615e1c9fa6eeefe08329ac4b94b // 7  PARENT[3, 6] NODE
        // 0x99af07747700389aba6e6cb0ee5d553fa1241688d9f96e48987bca1d7f275cbe // 8  LEAF NODE
        // 0xc09d4a008a0f1ef37860bef33ec3088ccd94268c0bfba7ff1b3c2a1075b0eb92 // 9  LEAF NODE
        // 0xdad09f50b41822fc5ecadc25b08c3a61531d4d60e962a5aa0b6998fad5c37c5e // 10 PARENT[8, 9] NODE
        // 0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c // 11 LEAF NODE
        // 0x643609ae1433f1d6caf366bb917873c3a3d82d7dc30e1c5e9a224d537f630dab // 12 LEAF NODE
        // 0x7fde31376facc58f621bacd80dfd77166544c84155bf1b82bf32281b93feaf78 // 13 PARENT[11, 12] NODE
        // 0xa63c4ec7ed257b6b4ab4fab3676f70b3b7c717357b537c0321d766de0e9e5312 // 14 PARENT[10, 13] NODE
        // 0xea97f06e80ac768687e72d4224999a51d272e1b4cafcbc64bd3ce63357119954 // 15 PARENT[7, 14] NODE
        // 0xbf5f579a06beced3256538b161b5096839db4b94ea1d3862bbe1fa5a2182e074 // 16 LEAF NODE
        // 0x7d8a0fe1021702eada6c608f3e09f833b63f21fdfe60f3bbb3401d5add4479af // 17 LEAF NODE
        // 0xa9ef6dd0b19d56f48a05c2475629c59713d0a992d335917135029432d611533d // 18 PARENT[16, 17] NODE
        // 0x2fd49d6e84591c6cc1fc38189b806dec1a1cb00c62727b63ac1cb9a37022c0fe // 19 LEAF NODE
        // 0x365f9e095800bd03add9be88b7f7bb06ff644ac2b77ce5da6a7c77e2fb19f1fb // 20 LEAF NODE
        // 0x3f7b0534bf60f62057a1ab9a0bf4751014d4d464245b5a7ad86801c9bac21b15 // 21 PARENT[19, 20] NODE
        // 0x16c5d5eb80eec816ca1804cd15705ac2418325b51b57a272e5e7f119e197c31f // 22 PARENT[18, 21] NODE
        // 0x94014b81bc56d64cac8dcde8eee47da0ed9b1319dccd9e86ad8d2266d8ef060a // 23 LEAF NODE
        // 0x883f1aca23002690575957cc85663774bbd3b9549ba5f0ee0fcc8aed9c88cf99 // 24 LEAF NODE
        // 0x1ce766309c74f07f3dc0839080f518ddcb6500d31fc4e0cf21534bad0785dfc4 // 25 PARENT[23, 24] NODE
        // 0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68 // 26 LEAF NODE

        let mmrVerification;
        beforeEach(async function () {
            mmrVerification = await MMRVerification.new();
        })

        const root = "0x197fbc87461398680c858f1daf61e719a1865edd96db34cca3b48c4b43d82e74";

        it('should verify valid proof for leaf index 7 (node 12)', async () => {
            let leafNodeHash = "0x643609ae1433f1d6caf366bb917873c3a3d82d7dc30e1c5e9a224d537f630dab"
            let proof = {
                leaf_index: 7,
                leaf_count: 15,
                items: [
                    "0xaf3327deed0515c8d1902c9b5cd375942d42f388f3bfe3d1cd6e1b86f9cc456c", // 11
                    "0xdad09f50b41822fc5ecadc25b08c3a61531d4d60e962a5aa0b6998fad5c37c5e", // 10
                    "0xe53ee36ba6c068b1a6cfef7862fed5005df55615e1c9fa6eeefe08329ac4b94b", // 7
                    mergeKeccak256( // bag right hand side peaks keccak(right, left)
                        mergeKeccak256(
                            "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68",  // 26
                            "0x1ce766309c74f07f3dc0839080f518ddcb6500d31fc4e0cf21534bad0785dfc4", // 25
                        ),
                        "0x16c5d5eb80eec816ca1804cd15705ac2418325b51b57a272e5e7f119e197c31f", // 22
                    ),
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 8 (node 16)', async () => {
            let leafNodeHash = "0xbf5f579a06beced3256538b161b5096839db4b94ea1d3862bbe1fa5a2182e074"
            let proof = {
                leaf_index: 8,
                leaf_count: 15,
                items: [
                    "0xea97f06e80ac768687e72d4224999a51d272e1b4cafcbc64bd3ce63357119954", // 15
                    "0x7d8a0fe1021702eada6c608f3e09f833b63f21fdfe60f3bbb3401d5add4479af", // 17
                    "0x3f7b0534bf60f62057a1ab9a0bf4751014d4d464245b5a7ad86801c9bac21b15", // 21
                    mergeKeccak256( // bag right hand side peaks keccak(right, left)
                        "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68",  // 26
                        "0x1ce766309c74f07f3dc0839080f518ddcb6500d31fc4e0cf21534bad0785dfc4", // 25
                    )
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 9 (node 17)', async () => {
            let leafNodeHash = "0x7d8a0fe1021702eada6c608f3e09f833b63f21fdfe60f3bbb3401d5add4479af"
            let proof = {
                leaf_index: 9,
                leaf_count: 15,
                items: [
                    "0xea97f06e80ac768687e72d4224999a51d272e1b4cafcbc64bd3ce63357119954", // 15
                    "0xbf5f579a06beced3256538b161b5096839db4b94ea1d3862bbe1fa5a2182e074", // 16
                    "0x3f7b0534bf60f62057a1ab9a0bf4751014d4d464245b5a7ad86801c9bac21b15", // 21
                    mergeKeccak256( // bag right hand side peaks keccak(right, left)
                        "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68",  // 26
                        "0x1ce766309c74f07f3dc0839080f518ddcb6500d31fc4e0cf21534bad0785dfc4", // 25
                    )
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 10 (node 19)', async () => {
            let leafNodeHash = "0x2fd49d6e84591c6cc1fc38189b806dec1a1cb00c62727b63ac1cb9a37022c0fe"
            let proof = {
                leaf_index: 10,
                leaf_count: 15,
                items: [
                    "0xea97f06e80ac768687e72d4224999a51d272e1b4cafcbc64bd3ce63357119954", // 15
                    "0x365f9e095800bd03add9be88b7f7bb06ff644ac2b77ce5da6a7c77e2fb19f1fb", // 20
                    "0xa9ef6dd0b19d56f48a05c2475629c59713d0a992d335917135029432d611533d", // 18
                    mergeKeccak256( // bag right hand side peaks keccak(right, left)
                        "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68",  // 26
                        "0x1ce766309c74f07f3dc0839080f518ddcb6500d31fc4e0cf21534bad0785dfc4", // 25
                    )
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 11 (node 20)', async () => {
            let leafNodeHash = "0x365f9e095800bd03add9be88b7f7bb06ff644ac2b77ce5da6a7c77e2fb19f1fb"
            let proof = {
                leaf_index: 11,
                leaf_count: 15,
                items: [
                    "0xea97f06e80ac768687e72d4224999a51d272e1b4cafcbc64bd3ce63357119954", // 15
                    "0x2fd49d6e84591c6cc1fc38189b806dec1a1cb00c62727b63ac1cb9a37022c0fe", // 19
                    "0xa9ef6dd0b19d56f48a05c2475629c59713d0a992d335917135029432d611533d", // 18
                    mergeKeccak256( // bag right hand side peaks keccak(right, left)
                        "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68",  // 26
                        "0x1ce766309c74f07f3dc0839080f518ddcb6500d31fc4e0cf21534bad0785dfc4", // 25
                    )
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 12 (node 23)', async () => {
            let leafNodeHash = "0x94014b81bc56d64cac8dcde8eee47da0ed9b1319dccd9e86ad8d2266d8ef060a"
            let proof = {
                leaf_index: 12,
                leaf_count: 15,
                items: [
                    "0xea97f06e80ac768687e72d4224999a51d272e1b4cafcbc64bd3ce63357119954", // 15
                    "0x16c5d5eb80eec816ca1804cd15705ac2418325b51b57a272e5e7f119e197c31f", // 22
                    "0x883f1aca23002690575957cc85663774bbd3b9549ba5f0ee0fcc8aed9c88cf99", // 24
                    "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68"  // 26
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should verify valid proof for leaf index 13 (node 24)', async () => {
            let leafNodeHash = "0x883f1aca23002690575957cc85663774bbd3b9549ba5f0ee0fcc8aed9c88cf99"
            let proof = {
                leaf_index: 13,
                leaf_count: 15,
                items: [
                    "0xea97f06e80ac768687e72d4224999a51d272e1b4cafcbc64bd3ce63357119954", // 15
                    "0x16c5d5eb80eec816ca1804cd15705ac2418325b51b57a272e5e7f119e197c31f", // 22
                    "0x94014b81bc56d64cac8dcde8eee47da0ed9b1319dccd9e86ad8d2266d8ef060a", // 23
                    "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68"  // 26
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });

        it('should NOT verify an invalid proof missing a left root proof item for leaf index 13 (node 24)', async () => {
            let leafNodeHash = "0x883f1aca23002690575957cc85663774bbd3b9549ba5f0ee0fcc8aed9c88cf99"
            let proof = {
                leaf_index: 13,
                leaf_count: 15,
                items: [
                    "0x16c5d5eb80eec816ca1804cd15705ac2418325b51b57a272e5e7f119e197c31f",
                    "0x94014b81bc56d64cac8dcde8eee47da0ed9b1319dccd9e86ad8d2266d8ef060a",
                    "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68"
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.false
        });

        it('should verify valid proof for leaf index 14 (node 26)', async () => {
            let leafNodeHash = "0x0a73e5a8443de3fcb6f918d786ad6dece6733ec936aa6b1b79beaab19e269d68"
            let proof = {
                leaf_index: 14,
                leaf_count: 15,
                items: [
                    "0xea97f06e80ac768687e72d4224999a51d272e1b4cafcbc64bd3ce63357119954", // 15
                    "0x16c5d5eb80eec816ca1804cd15705ac2418325b51b57a272e5e7f119e197c31f", // 22
                    "0x1ce766309c74f07f3dc0839080f518ddcb6500d31fc4e0cf21534bad0785dfc4"  // 25
                ]
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });
    })

    context("1-leaf, 1-node MMR", function () {

        before(function () {
            console.log('                 1-leaf MMR:           ');
            console.log('                                       ');
            console.log('    Height 1 | 1                       ');
            console.log('             | |                       ');
            console.log('Leaf indexes | 0                       ');
        })

        // ---------------------------- Tree contents ----------------------------
        //  - For leaf nodes, node hash is the SCALE-encoding of the leaf data.
        //  - For parent nodes, node hash is the hash of it's children (left, right).
        //
        // 0xda5e6d0616e05c6a6348605a37ca33493fc1a15ad1e6a405ee05c17843fdafed // 1  LEAF NODE

        let mmrVerification;
        beforeEach(async function () {
            mmrVerification = await MMRVerification.new();
        })

        const root = "0xda5e6d0616e05c6a6348605a37ca33493fc1a15ad1e6a405ee05c17843fdafed"

        it('should verify valid proof for leaf index 0 (node 1)', async () => {
            let leafNodeHash = "0xda5e6d0616e05c6a6348605a37ca33493fc1a15ad1e6a405ee05c17843fdafed"
            let proof = {
                leaf_index: 0,
                leaf_count: 1,
                items: []
            }

            expect(await mmrVerification.verifyInclusionProof.call(root, leafNodeHash, proof.leaf_index, proof.leaf_count, proof.items)).to.be.true
        });
    })

    context("7-leaf MMR from fixture", function () {
        let mmrVerification;
        beforeEach(async function () {
            mmrVerification = await MMRVerification.new();
        })

        fixture7leaves.proofs.forEach((proof, i) => {
            it(`should verify valid proof for leaf index ${i}`, async () => {
                expect(await mmrVerification.verifyInclusionProof.call(fixture7leaves.rootHash, fixture7leaves.leaves[i],
                    proof.leafIndex, proof.leafCount, proof.items)).to.be.true;
            });
        });
    });

    context("15-leaf MMR from fixture", function () {
        let mmrVerification;
        beforeEach(async function () {
            mmrVerification = await MMRVerification.new();
        })

        fixture15leaves.proofs.forEach((proof, i) => {
            it(`should verify valid proof for leaf index ${i}`, async () => {
                expect(await mmrVerification.verifyInclusionProof.call(fixture15leaves.rootHash, fixture15leaves.leaves[i],
                    proof.leafIndex, proof.leafCount, proof.items)).to.be.true;
            });
        });
    });
});
