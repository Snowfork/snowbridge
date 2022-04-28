const { ethers } = require("ethers");
const _ = require("lodash");
const secp256k1 = require('secp256k1');

const {
  createMerkleTree, mine, printBitfield
} = require("./helpers");

const { keccakFromHexString } = require("ethereumjs-util");

async function createValidatorFixture(validatorSetId, validatorSetLength) {

  let wallets = [];
  for (let i = 0; i < validatorSetLength; i++) {
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

  const validatorMerkleTree = createMerkleTree(validatorAddresses);
  const validatorAddressProofs = validatorAddresses.map((address, index) => {
    return validatorMerkleTree.getHexProof(address, index)
  })

  return {
    wallets,
    walletsByLeaf,
    validatorAddresses,
    validatorAddressesHashed,
    validatorSetId,
    validatorSetRoot: validatorMerkleTree.getHexRoot(),
    validatorSetLength,
    validatorAddressProofs,
    validatorMerkleTree
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


const runBeefyClientFlow = async (fixture, beefyClient, validatorFixture, totalNumberOfSignatures, totalNumberOfValidators) => {
  const initialBitfieldPositions = await createRandomPositions(totalNumberOfSignatures, totalNumberOfValidators)

  const initialBitfield = await beefyClient.createInitialBitfield(
    initialBitfieldPositions, totalNumberOfValidators
  );

  const proofs = await createInitialValidatorProofs(fixture.commitmentHash, validatorFixture);

  await beefyClient.newSignatureCommitment(
    fixture.commitmentHash,
    fixture.transactionParams.commitment.validatorSetId,
    initialBitfield,
    proofs[0].signature,
    proofs[0].position,
    proofs[0].address,
    proofs[0].proof,
  )

  const lastId = (await beefyClient.nextID()).sub(new web3.utils.BN(1));

  await mine(45);

  const completeProofs = await createFinalValidatorProofs(lastId, beefyClient, proofs);

  await beefyClient.completeSignatureCommitment(
    lastId,
    fixture.transactionParams.commitment,
    completeProofs,
  )

}

async function createInitialValidatorProofs(commitmentHash, validatorFixture) {
  let commitmentHashBytes = ethers.utils.arrayify(commitmentHash)
  const tree = validatorFixture.validatorMerkleTree;
  const leaves = tree.getHexLeaves()

  return leaves.map((leaf, position) => {
    const wallet = validatorFixture.walletsByLeaf[leaf]
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

async function createFinalValidatorProofs(id, beefyClient, initialProofs) {
  const bitfieldInts = await beefyClient.createRandomBitfield(id);
  const bitfieldString = printBitfield(bitfieldInts);

  const proofs = {
    signatures: [],
    positions: [],
    publicKeys: [],
    publicKeyMerkleProofs: [],
  }

  const ascendingBitfield = bitfieldString.split('').reverse().join('');
  for (let position = 0; position < ascendingBitfield.length; position++) {
    const bit = ascendingBitfield[position]
    if (bit === '1') {
      proofs.signatures.push(initialProofs[position].signature)
      proofs.positions.push(initialProofs[position].position)
      proofs.publicKeys.push(initialProofs[position].address)
      proofs.publicKeyMerkleProofs.push(initialProofs[position].proof)
    }
  }

  return proofs
}

module.exports = {
  createValidatorFixture,
  createRandomPositions,
  createInitialValidatorProofs,
  runBeefyClientFlow,
  createFinalValidatorProofs,
}
