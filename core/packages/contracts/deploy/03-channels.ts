import { HardhatRuntimeEnvironment } from "hardhat/types"
import { getConfigForNetwork } from "../config"

module.exports = async ({ deployments, getUnnamedAccounts, network }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    const config = getConfigForNetwork(network.name)

    let parachainClient = await deployments.get("ParachainClient")
    let merkleProof = await deployments.get("MerkleProof")

    let basicInboundChannel = await deployments.deploy("BasicInboundChannel", {
        from: deployer,
        args: [config.basicChannelSourceID, parachainClient.address],
        libraries: {
            MerkleProof: merkleProof.address
        },
        log: true,
        autoMine: true
    })

    let incentivizedInboundChannel = await deployments.deploy("IncentivizedInboundChannel", {
        from: deployer,
        args: [config.incentivizedChannelSourceID, parachainClient.address],
        log: true,
        autoMine: true
    })

    let basicOutboundChannel = await deployments.deploy("BasicOutboundChannel", {
        from: deployer,
        log: true,
        autoMine: true
    })

    let incentivizedOutboundChannel = await deployments.deploy("IncentivizedOutboundChannel", {
        from: deployer,
        log: true,
        autoMine: true
    })

    await deployments.deploy("ChannelRegistry", {
        from: deployer,
        log: true,
        autoMine: true
    })

    console.log("Configuring ChannelRegistry")
    await deployments.execute(
        "ChannelRegistry",
        {
            from: deployer,
            autoMine: true
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
            autoMine: true
        },
        "updateChannel",
        1,
        incentivizedInboundChannel.address,
        incentivizedOutboundChannel.address
    )
}
