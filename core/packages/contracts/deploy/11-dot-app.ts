import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ ethers, deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let incentivizedOutboundChannel = await deployments.get("IncentivizedOutboundChannel")
    let channelRegistry = await deployments.get("ChannelRegistry")

    const feeData = await ethers.provider.getFeeData()

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
