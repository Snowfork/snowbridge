import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({
    deployments,
    getUnnamedAccounts,
    network,
}: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let scaleCodecLibrary = await deployments.get("ScaleCodec")
    let incentivizedOutboundChannel = await deployments.get("IncentivizedOutboundChannel")
    let channelRegistry = await deployments.get("IncentivizedOutboundChannel")

    let tokenContract = await deployments.deploy("WrappedToken", {
        from: deployer,
        args: ["Wrapped DOT", "WDOT"],
        log: true,
        autoMine: true,
    })

    let dotAppContract = await deployments.deploy("DOTApp", {
        from: deployer,
        args: [
            tokenContract.address,
            incentivizedOutboundChannel.address,
            channelRegistry.address
        ],
        libraries: {
            ScaleCodec: scaleCodecLibrary.address,
        },
        log: true,
        autoMine: true,
    })

    console.log("Configuring WrappedToken")
    await deployments.execute(
        "WrappedToken",
        {
            from: deployer,
            autoMine: true,
        },
        "transferOwnership",
        dotAppContract.address
    )
}
