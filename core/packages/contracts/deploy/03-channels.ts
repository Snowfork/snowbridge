import { HardhatRuntimeEnvironment } from "hardhat/types"
import { getConfigForNetwork } from "../config"

module.exports = async ({ ethers, deployments, getUnnamedAccounts, network }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    const config = getConfigForNetwork(network.name)

    let parachainClient = await deployments.get("ParachainClient")
    let merkleProof = await deployments.get("MerkleProof")

    const feeData = await ethers.provider.getFeeData()

    let basicInboundChannel = await deployments.deploy("BasicInboundChannel", {
        from: deployer,
        args: [config.basicChannelSourceID, parachainClient.address],
        libraries: {
            MerkleProof: merkleProof.address
        },
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    let incentivizedInboundChannel = await deployments.deploy("IncentivizedInboundChannel", {
        from: deployer,
        args: [config.incentivizedChannelSourceID, parachainClient.address],
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    let basicOutboundChannel = await deployments.deploy("BasicOutboundChannel", {
        from: deployer,
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    let incentivizedOutboundChannel = await deployments.deploy("IncentivizedOutboundChannel", {
        from: deployer,
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    await deployments.deploy("ChannelRegistry", {
        from: deployer,
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })

    console.log("Configuring ChannelRegistry")
    await deployments.execute(
        "ChannelRegistry",
        {
            from: deployer,
            autoMine: true,
            maxFeePerGas: feeData.maxFeePerGas,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        },
        "updateChannel",
        0,
        basicInboundChannel.address,
        basicOutboundChannel.address
    )
    await deployments.execute(
        "ChannelRegistry",
        {
            from: deployer,
            autoMine: true,
            maxFeePerGas: feeData.maxFeePerGas,
            maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
        },
        "updateChannel",
        1,
        incentivizedInboundChannel.address,
        incentivizedOutboundChannel.address
    )
}
