import { HardhatRuntimeEnvironment } from "hardhat/types"
const hre = require("hardhat")

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let scaleCodecLibrary = await deployments.get("ScaleCodec")
    let bitFieldLibrary = await deployments.get("Bitfield")
    let merkleProofLibrary = await deployments.get("MerkleProof")
    let mmrProofVerificationLibrary = await deployments.get("MMRProofVerification")

    const feeData = await hre.ethers.provider.getFeeData()

    await deployments.deploy("BeefyClient", {
        from: deployer,
        libraries: {
            MerkleProof: merkleProofLibrary.address,
            MMRProofVerification: mmrProofVerificationLibrary.address,
            Bitfield: bitFieldLibrary.address,
            ScaleCodec: scaleCodecLibrary.address,
        },
        log: true,
        autoMine: true,
        maxFeePerGas: feeData.maxFeePerGas,
        maxPriorityFeePerGas: feeData.maxPriorityFeePerGas,
    })
}
