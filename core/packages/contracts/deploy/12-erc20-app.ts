import { HardhatRuntimeEnvironment } from "hardhat/types"
import hre from "hardhat";

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let channelRegistry = await deployments.get("ChannelRegistry")
    let scaleCodecLibrary = await deployments.get("ScaleCodec")

    const feeData = await hre.ethers.provider.getFeeData()

    let vault = await deployments.deploy('ERC20Vault', {
        from: deployer,
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    let app = await deployments.deploy("ERC20App", {
        from: deployer,
        args: [vault.address, channelRegistry.address],
        libraries: {
            ScaleCodec: scaleCodecLibrary.address,
        },
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
