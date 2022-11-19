import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let channelRegistry = await deployments.get("ChannelRegistry")

    let vault = await deployments.deploy("ERC20Vault", {
        from: deployer,
        log: true,
        autoMine: true
    })

    let app = await deployments.deploy("ERC20App", {
        from: deployer,
        args: [vault.address, channelRegistry.address],
        log: true,
        autoMine: true
    })

    await deployments.execute(
        "ERC20Vault",
        {
            from: deployer,
            log: true,
            autoMine: true
        },
        "transferOwnership",
        app.address
    )

    await deployments.deploy("TestToken", {
        from: deployer,
        args: ["Test Token", "TEST"],
        log: true
    })
}
