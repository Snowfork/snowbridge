import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ ethers, deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    const feeData = await ethers.provider.getFeeData()

    console.log("Configuring BasicOutboundChannel")
    await deployments.execute(
        "BasicOutboundChannel",
        {
            from: deployer,
            autoMine: true,
            maxFeePerGas: feeData.maxFeePerGas,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        },
        "initialize",
        deployer,
        []
    )

    // Mark deployment to run only once
    return true
}

module.exports.id = "configure-channels"
