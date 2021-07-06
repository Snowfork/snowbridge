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
  while (positions.length < numberOfPositions) {
    const position = Math.floor(Math.random() * numberOfValidators);
    if (positions.indexOf(position) === -1) {
      positions.push(position);
    }
  }
  return positions.sort((a, b) => a - b)
}

module.exports = {
  createBeefyValidatorFixture,
  createRandomPositions,
}
