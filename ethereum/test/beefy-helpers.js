const { ethers } = require("ethers");
const _ = require("lodash");
const secp256k1 = require('secp256k1');

const {
  createMerkleTree, mine, printBitfield
} = require("./helpers");

const { keccakFromHexString } = require("ethereumjs-util");

async function createBeefyValidatorFixture(numberOfValidators) {

  let wallets = [];
  for (let i = 0; i < numberOfValidators; i++) {
    const wallet = ethers.Wallet.createRandom();
    wallets.push(wallet);
  }
  const walletsByLeaf = wallets.reduce((accum, wallet) => {
    const leaf = '0x' + keccakFromHexString(wallet.address).toString('hex')
    accum[leaf] = wallet
    return accum
  }, {})

  wallets = wallets.sort((a, b) => a.address < b.address)

  const validatorAddresses = wallets.map(wallet => {
    return wallet.address
  })

  const validatorAddressesHashed = validatorAddresses.map(address => {
    return '0x' + keccakFromHexString(address).toString('hex')
  })

  const validatorsMerkleTree = createMerkleTree(validatorAddresses);
  const validatorAddressProofs = validatorAddresses.map((address, index) => {
    return validatorsMerkleTree.getHexProof(address, index)
  })
  const root = validatorsMerkleTree.getHexRoot()

  return {
    wallets, walletsByLeaf, validatorAddresses, validatorAddressesHashed, root, validatorAddressProofs, validatorsMerkleTree
  }
}

async function createRandomPositions(numberOfPositions, numberOfValidators) {

  const positions = [];
  for (i = 0; i < numberOfValidators; i++) {
    positions.push(i);
  }

  const shuffled = _.shuffle(positions)

  return shuffled.slice(0, numberOfPositions)
}


const runBeefyLightClientFlow = async (realWorldFixture, beefyLightClient, beefyFixture, totalNumberOfSignatures, totalNumberOfValidators) => {
  const initialBitfieldPositions = await createRandomPositions(totalNumberOfSignatures, totalNumberOfValidators)

  const initialBitfield = await beefyLightClient.createInitialBitfield(
    initialBitfieldPositions, totalNumberOfValidators
  );

  const commitmentHash = await beefyLightClient.createCommitmentHash(realWorldFixture.completeSubmitInput.commitment);

  const allValidatorProofs = await createAllValidatorProofs(commitmentHash, beefyFixture);

  await beefyLightClient.newSignatureCommitment(
    commitmentHash,
    initialBitfield,
    allValidatorProofs[0].signature,
    allValidatorProofs[0].position,
    allValidatorProofs[0].address,
    allValidatorProofs[0].proof,
  )

  const lastId = (await beefyLightClient.currentId()).sub(new web3.utils.BN(1));

  await mine(45);

  const completeValidatorProofs = await createCompleteValidatorProofs(lastId, beefyLightClient, allValidatorProofs);

  await beefyLightClient.completeSignatureCommitment(
    lastId,
    realWorldFixture.completeSubmitInput.commitment,
    completeValidatorProofs,
    realWorldFixture.completeSubmitInput.latestMMRLeaf,
    realWorldFixture.completeSubmitInput.mmrLeafIndex,
    realWorldFixture.completeSubmitInput.mmrLeafCount,
    realWorldFixture.completeSubmitInput.mmrProofItems,
  )

}

async function createAllValidatorProofs(commitmentHash, beefyFixture) {
  let commitmentHashBytes = ethers.utils.arrayify(commitmentHash)
  const tree = beefyFixture.validatorsMerkleTree;
  const leaves = tree.getHexLeaves()

  return leaves.map((leaf, position) => {
    const wallet = beefyFixture.walletsByLeaf[leaf]
    const address = wallet.address
    const proof = tree.getHexProof(leaf, position)
    const privateKey = ethers.utils.arrayify(wallet.privateKey)
    const signatureECDSA = secp256k1.ecdsaSign(commitmentHashBytes, privateKey)
    const ethRecID = signatureECDSA.recid + 27
    const signature = Uint8Array.from(
      signatureECDSA.signature.join().split(',').concat(ethRecID)
    )
    return { signature: ethers.utils.hexlify(signature), position, address, proof };
  });
}

async function createCompleteValidatorProofs(id, beefyLightClient, allValidatorProofs) {
  const bitfieldInts = await beefyLightClient.createRandomBitfield(id);
  const bitfieldString = printBitfield(bitfieldInts);

  const completeValidatorProofs = {
    signatures: [],
    positions: [],
    publicKeys: [],
    publicKeyMerkleProofs: [],
  }

  const ascendingBitfield = bitfieldString.split('').reverse().join('');
  for (let position = 0; position < ascendingBitfield.length; position++) {
    const bit = ascendingBitfield[position]
    if (bit === '1') {
      completeValidatorProofs.signatures.push(allValidatorProofs[position].signature)
      completeValidatorProofs.positions.push(allValidatorProofs[position].position)
      completeValidatorProofs.publicKeys.push(allValidatorProofs[position].address)
      completeValidatorProofs.publicKeyMerkleProofs.push(allValidatorProofs[position].proof)
    }
  }

  return completeValidatorProofs
}

module.exports = {
  createBeefyValidatorFixture,
  createRandomPositions,
  createAllValidatorProofs,
  runBeefyLightClientFlow,
  createCompleteValidatorProofs,
}
