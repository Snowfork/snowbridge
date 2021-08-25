
async function mintNFTs() {
  const hre = require("hardhat");
  const { deployments, ethers } = hre;
  const [deployer, developer] = await hre.getUnnamedAccounts();

  const nft = await deployments.get('TestToken721Enumerable');
  const TestNft = await ethers.getContractAt('TestToken721Enumerable', nft.address);

  for (let i = 0; i < 10; i++) {
    await TestNft.mint(developer, Date.now().toString());
  }
}

mintNFTs()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
