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
