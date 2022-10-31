const rlp = require("rlp");
const { keccakFromHexString, keccak } = require("ethereumjs-util");
const MerkleTree = require("merkletreejs").MerkleTree;
const { ethers } = require("ethers");
const _ = require("lodash");
const secp256k1 = require('secp256k1');

const fixture = require('./fixtures/beefy-relay-basic.json');

const makeBasicCommitment = (messages) => {
  let encoded = ethers.utils.defaultAbiCoder.encode(
    ['tuple(address target, uint64 nonce, bytes payload)[]'],
    [messages]
  )
  return ethers.utils.solidityKeccak256(["bytes"], [encoded])
}

const makeIncentivizedCommitment = (messages) => {
  let encoded = ethers.utils.defaultAbiCoder.encode(
    ['tuple(address target, uint64 nonce, uint256 fee, bytes payload)[]'],
    [messages]
  )
  return ethers.utils.solidityKeccak256(["bytes"], [encoded])
}

function signatureSubstrateToEthereum(sig) {
  const recoveryId0 = web3.utils.hexToNumber(`0x${sig.slice(130)}`);
  const newRecoveryId0 = web3.utils.numberToHex(recoveryId0 + 27);
  const res = sig.slice(0, 130).concat(newRecoveryId0.slice(2));

  return res;
}

function createMerkleTree(values) {
  const leaves = values.map(value => keccakFromHexString(value));
  const merkleTree = new MerkleTree(leaves, keccak, {
    sortLeaves: false,
    sortPairs: false
  });
  return merkleTree;
}

const encodeLog = (log) => {
  return rlp.encode([log.address, log.topics, log.data]).toString("hex")
}

const hexPrefix = /^(0x)/i

const mergeKeccak256 = (left, right) =>
  '0x' + keccakFromHexString('0x' + left.replace(hexPrefix, "") + right.replace(hexPrefix, ''), 256).toString('hex')

const PREFIX = "VM Exception while processing transaction: ";
const PREFIX_2 = "Returned error: VM Exception while processing transaction: ";

async function tryCatch(promise, type, message) {
  try {
    await promise;
    throw null;
  }
  catch (error) {
    assert(error, "Expected an error but did not get one");
    if (message) {
      assert(error.message === (PREFIX + type + ' \'' + message + '\''), "Expected error '" + PREFIX + type + ' ' + message +
        "' but got '" + error.message + "' instead");
    } else {
      assert(error && error.message && error.message.startsWith(PREFIX + type), "Expected an error starting with '" + PREFIX + type +
        "' but got '" + error.message + "' instead");
    }
  }
};

function printBitfield(bitfield) {
  return bitfield.map(i => {
    const bf = BigInt(i.toString(), 10).toString(2).split('')
    while (bf.length < 256) {
      bf.unshift('0')
    }
    return bf.join('')
  }).reverse().join('').replace(/^0*/g, '')
}

async function createValidatorFixture(validatorSetID, validatorSetLength) {

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
    validatorSetID,
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
  const bitfieldInts = await beefyClient.createFinalBitfield(id);
  const bitfieldString = printBitfield(bitfieldInts);

  const proofs = {
    signatures: [],
    indices: [],
    addrs: [],
    merkleProofs: [],
  }

  const ascendingBitfield = bitfieldString.split('').reverse().join('');
  for (let position = 0; position < ascendingBitfield.length; position++) {
    const bit = ascendingBitfield[position]
    if (bit === '1') {
      proofs.signatures.push(initialProofs[position].signature)
      proofs.indices.push(initialProofs[position].position)
      proofs.addrs.push(initialProofs[position].address)
      proofs.merkleProofs.push(initialProofs[position].proof)
    }
  }

  return proofs
}

module.exports = {
  createMerkleTree,
  signatureSubstrateToEthereum,
  encodeLog,
  mergeKeccak256,
  printBitfield,
  createValidatorFixture,
  createRandomPositions,
  createInitialValidatorProofs,
  createFinalValidatorProofs,
  catchRevert: async (promise, message) => await tryCatch(promise, "reverted with reason string", message),
  catchOutOfGas: async (promise, message) => await tryCatch(promise, "out of gas", message),
  catchInvalidJump: async (promise, message) => await tryCatch(promise, "invalid JUMP", message),
  catchInvalidOpcode: async (promise, message) => await tryCatch(promise, "invalid opcode", message),
  catchStackOverflow: async (promise, message) => await tryCatch(promise, "stack overflow", message),
  catchStackUnderflow: async (promise, message) => await tryCatch(promise, "stack underflow", message),
  catchStaticStateChange: async (promise, message) => await tryCatch(promise, "static state change", message),
};
