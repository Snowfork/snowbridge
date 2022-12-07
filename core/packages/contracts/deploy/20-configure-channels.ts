import { HardhatRuntimeEnvironment } from "hardhat/types"
import { getConfigForNetwork } from "../config"

module.exports = async ({ ethers, deployments, getUnnamedAccounts, network }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    const fee = getConfigForNetwork(network.name).incentivizedChannelFee

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

    console.log("Configuring IncentivizedOutboundChannel")
    await deployments.execute(
        "IncentivizedOutboundChannel",
        {
            from: deployer,
            autoMine: true,
            maxFeePerGas: feeData.maxFeePerGas,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        },
        "initialize",
        deployer,
        dotApp.address,
        [dotApp.address, ethApp.address, erc20App.address]
    )
    await deployments.execute(
        "IncentivizedOutboundChannel",
        {
            from: deployer,
            autoMine: true,
            maxFeePerGas: feeData.maxFeePerGas,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        },
        "setFee",
        fee
    )

    console.log("Configuring IncentivizedInboundChannel")
    await deployments.execute(
        "IncentivizedInboundChannel",
        {
            from: deployer,
            autoMine: true,
            maxFeePerGas: feeData.maxFeePerGas,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        },
        "initialize",
        deployer,
        ethApp.address
    )

    // Mark deployment to run only once
    return true
}

module.exports.id = "configure-channels"
