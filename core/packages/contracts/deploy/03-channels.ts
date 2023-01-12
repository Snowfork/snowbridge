import { HardhatRuntimeEnvironment } from "hardhat/types"
import { getConfigForNetwork } from "../config"

module.exports = async ({ ethers, deployments, getUnnamedAccounts, network }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    const config = getConfigForNetwork(network.name)

    let parachainClient = await deployments.get("ParachainClient")
    let merkleProof = await deployments.get("MerkleProof")

    const feeData = await ethers.provider.getFeeData()

    await deployments.deploy("BasicInboundChannel", {
        from: deployer,
        args: [config.basicChannelSourceID, parachainClient.address],
        libraries: {
            MerkleProof: merkleProof.address
        },
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    await deployments.deploy("BasicOutboundChannel", {
        from: deployer,
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })
}
