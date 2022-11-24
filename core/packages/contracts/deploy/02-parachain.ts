import { HardhatRuntimeEnvironment } from "hardhat/types"
import { getConfigForNetwork } from "../config"

module.exports = async ({ deployments, getUnnamedAccounts, network }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    const paraID = getConfigForNetwork(network.name).parachainID

    let merkleProofLibrary = await deployments.get("MerkleProof")
    let beefyClient = await deployments.get("BeefyClient")

    await deployments.deploy("ParachainClient", {
        from: deployer,
        args: [beefyClient.address, paraID],
        libraries: {
            MerkleProof: merkleProofLibrary.address
        },
        log: true,
        autoMine: true
    })
}
