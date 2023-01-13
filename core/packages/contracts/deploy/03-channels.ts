import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ ethers, deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let parachainClient = await deployments.get("ParachainClient")
    let merkleProof = await deployments.get("MerkleProof")

    const feeData = await ethers.provider.getFeeData()

    await deployments.deploy("BasicInboundChannel", {
        from: deployer,
        args: [parachainClient.address],
        libraries: {
            MerkleProof: merkleProof.address
        },
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    await deployments.deploy("BasicOutboundChannel", {
        from: deployer,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })
}
