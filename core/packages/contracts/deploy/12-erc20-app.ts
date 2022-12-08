import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ ethers, deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let channelRegistry = await deployments.get("ChannelRegistry")

    const feeData = await ethers.provider.getFeeData()

    let vault = await deployments.deploy("ERC20Vault", {
        from: deployer,
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    let app = await deployments.deploy("ERC20App", {
        from: deployer,
        args: [vault.address, channelRegistry.address],
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    await deployments.execute(
        "ERC20Vault",
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

    await deployments.deploy("TestToken", {
        from: deployer,
        args: ["Test Token", "TEST"],
        log: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })
}
