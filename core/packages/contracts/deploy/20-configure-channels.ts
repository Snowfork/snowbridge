import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    if (!("INCENTIVIZED_CHANNEL_FEE" in process.env)) {
        throw "Missing INCENTIVIZED_CHANNEL_FEE in environment config"
    }
    const fee = process.env.INCENTIVIZED_CHANNEL_FEE

    let dotApp = await deployments.get("DOTApp")
    let ethApp = await deployments.get("ETHApp")
    let erc20App = await deployments.get("ERC20App")

    console.log("Configuring BasicOutboundChannel")
    await deployments.execute(
        "BasicOutboundChannel",
        {
            from: deployer,
            autoMine: true,
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
        },
        "initialize",
        deployer,
        ethApp.address
    )

    // Mark deployment to run only once
    return true
}

module.exports.id = "configure-channels"
