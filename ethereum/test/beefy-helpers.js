const { ethers } = require("ethers");
const _ = require("lodash");

const {
  createMerkleTree
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

async function createFullPositions(numberOfValidators) {
  const positions = [];
  for (let i = 0; i < numberOfValidators; i++) {
    positions.push(i);
  }
  return positions.sort((a, b) => b - a)
}

module.exports = {
  createBeefyValidatorFixture,
  createRandomPositions,
  createFullPositions,
}
