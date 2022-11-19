import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let incentivizedInboundChannel = await deployments.get("IncentivizedInboundChannel")
    let channelRegistry = await deployments.get("ChannelRegistry")

    let vault = await deployments.deploy("EtherVault", {
        from: deployer,
        log: true,
        autoMine: true
    })

    let app = await deployments.deploy("ETHApp", {
        from: deployer,
        args: [incentivizedInboundChannel.address, vault.address, channelRegistry.address],
        log: true,
        autoMine: true
    })

    await deployments.execute(
        "EtherVault",
        {
            from: deployer,
            log: true,
            autoMine: true
        },
        "transferOwnership",
        app.address
    )
}
