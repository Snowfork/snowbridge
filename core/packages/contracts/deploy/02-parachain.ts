import { HardhatRuntimeEnvironment } from "hardhat/types"
import hre from "hardhat";

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    if (!("PARACHAIN_ID" in process.env)) {
        throw "Missing PARACHAIN_ID in environment config"
    }
    const paraID = process.env.PARACHAIN_ID

    let merkleProofLibrary = await deployments.get("MerkleProof")
    let scaleCodecLibrary = await deployments.get("ScaleCodec")

    let beefyClient = await deployments.get("BeefyClient")

    const feeData = await hre.ethers.provider.getFeeData()

    await deployments.deploy("ParachainClient", {
        from: deployer,
        args: [beefyClient.address, paraID],
        libraries: {
            MerkleProof: merkleProofLibrary.address,
            ScaleCodec: scaleCodecLibrary.address,
        },
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })
}
