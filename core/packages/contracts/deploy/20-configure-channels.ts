import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ ethers, deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let dotApp = await deployments.get("DOTApp")
    let ethApp = await deployments.get("ETHApp")
    let erc20App = await deployments.get("ERC20App")

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
        [dotApp.address, ethApp.address, erc20App.address]
    )

    // Mark deployment to run only once
    return true
}

module.exports.id = "configure-channels"
