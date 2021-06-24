const fixture = require('./beefy-fixture-data.json');
const {
  createMerkleTree,
} = require("../helpers");

function createFixtureData(numberOfSignatures) {
  let signatures = [];
  let positions = [];
  let validatorPublicKeys = [];
  let validatorPublicKeyProofs = [];
  for (let i = 0; i < numberOfSignatures; i++) {
    signatures.push(fixture.signatures[0]);
    positions.push(i)
    validatorPublicKeys.push(fixture.validatorPublicKeys[0])
  }

  this.validatorsMerkleTree = createMerkleTree(validatorPublicKeys);

  for (let i = 0; i < 200; i++) {
    proof = this.validatorsMerkleTree.getHexProof(validatorPublicKeys[0], 0)
    validatorPublicKeyProofs.push(proof)
  }

  return {
    signatures, positions, validatorPublicKeys, validatorPublicKeyProofs
  }
}

module.exports = createFixtureData(2)
