import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ ethers, deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let incentivizedInboundChannel = await deployments.get("IncentivizedInboundChannel")
    let channelRegistry = await deployments.get("ChannelRegistry")

    const feeData = await ethers.provider.getFeeData()

    let vault = await deployments.deploy("EtherVault", {
        from: deployer,
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    let app = await deployments.deploy("ETHApp", {
        from: deployer,
        args: [incentivizedInboundChannel.address, vault.address, channelRegistry.address],
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    await deployments.execute(
        "EtherVault",
        {
          from: deployer,
          log: true,
          autoMine: true,
            maxFeePerGas: feeData.maxFeePerGas,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        },
        "transferOwnership",
        app.address
    )
}
