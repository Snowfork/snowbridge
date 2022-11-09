import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let channelRegistry = await deployments.get("ChannelRegistry")
    let scaleCodecLibrary = await deployments.get("ScaleCodec")

    await deployments.deploy("ERC20App", {
        from: deployer,
        args: [channelRegistry.address],
        libraries: {
            ScaleCodec: scaleCodecLibrary.address,
        },
        log: true,
        autoMine: true,
    })

    await deployments.deploy("TestToken", {
        from: deployer,
        args: ["Test Token", "TEST"],
        log: true,
    })
}
