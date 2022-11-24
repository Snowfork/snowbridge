import { HardhatRuntimeEnvironment } from "hardhat/types"
const hre = require("hardhat")

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let scaleCodecLibrary = await deployments.get("ScaleCodec")
    let incentivizedOutboundChannel = await deployments.get("IncentivizedOutboundChannel")
    let channelRegistry = await deployments.get("ChannelRegistry")

    const feeData = await hre.ethers.provider.getFeeData()

    let tokenContract = await deployments.deploy("WrappedToken", {
        from: deployer,
        args: ["Wrapped DOT", "WDOT"],
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    let dotAppContract = await deployments.deploy("DOTApp", {
        from: deployer,
        args: [tokenContract.address, incentivizedOutboundChannel.address, channelRegistry.address],
        libraries: {
            ScaleCodec: scaleCodecLibrary.address,
        },
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    console.log("Configuring WrappedToken")
    await deployments.execute(
        "WrappedToken",
        {
            from: deployer,
            autoMine: true,
            maxFeePerGas: feeData.maxFeePerGas,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        },
        "transferOwnership",
        dotAppContract.address
    )
}
