import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let incentivizedInboundChannel = await deployments.get("IncentivizedInboundChannel")
    let channelRegistry = await deployments.get("ChannelRegistry")
    let scaleCodecLibrary = await deployments.get("ScaleCodec")

    await deployments.deploy("ETHApp", {
        from: deployer,
        args: [incentivizedInboundChannel.address, channelRegistry.address],
        libraries: {
            ScaleCodec: scaleCodecLibrary.address,
        },
        log: true,
        autoMine: true,
    })
}
