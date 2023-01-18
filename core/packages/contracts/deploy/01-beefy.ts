import { HardhatRuntimeEnvironment } from "hardhat/types"
import { getConfigForNetwork } from "../config"

module.exports = async ({ ethers, deployments, getUnnamedAccounts, network }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let config = getConfigForNetwork(network.name)

    let bitFieldLibrary = await deployments.get("Bitfield")
    let merkleProofLibrary = await deployments.get("MerkleProof")

    const feeData = await ethers.provider.getFeeData()

    await deployments.deploy("BeefyClient", {
        from: deployer,
        args: [config.randaoCommitDelay, config.randaoCommitExpiration],
        libraries: {
            MerkleProof: merkleProofLibrary.address,
            Bitfield: bitFieldLibrary.address
        },
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })
}
