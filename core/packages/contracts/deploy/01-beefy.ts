import { HardhatRuntimeEnvironment } from "hardhat/types"

module.exports = async ({ deployments, getUnnamedAccounts }: HardhatRuntimeEnvironment) => {
    let [deployer] = await getUnnamedAccounts()

    let bitFieldLibrary = await deployments.get("Bitfield")
    let merkleProofLibrary = await deployments.get("MerkleProof")

    await deployments.deploy("BeefyClient", {
        from: deployer,
        libraries: {
            MerkleProof: merkleProofLibrary.address,
            Bitfield: bitFieldLibrary.address
        },
        log: true,
        autoMine: true
    })
}
