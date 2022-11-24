import { HardhatRuntimeEnvironment } from "hardhat/types"
import hre from "hardhat";

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let incentivizedInboundChannel = await deployments.get("IncentivizedInboundChannel")
    let channelRegistry = await deployments.get("ChannelRegistry")
    let scaleCodecLibrary = await deployments.get("ScaleCodec")

    const feeData = await hre.ethers.provider.getFeeData()

    let vault = await deployments.deploy('EtherVault', {
        from: deployer,
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    let app = await deployments.deploy("ETHApp", {
        from: deployer,
        args: [incentivizedInboundChannel.address, vault.address, channelRegistry.address],
        libraries: {
            ScaleCodec: scaleCodecLibrary.address,
        },
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
