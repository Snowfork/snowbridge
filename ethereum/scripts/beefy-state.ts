import hre from "hardhat"

async function beefyState() {
    let [signer] = await hre.ethers.getSigners()

    let beefyDeployment = await hre.deployments.get("BeefyClient")
    let beefyClientContract = new hre.ethers.Contract(beefyDeployment.address, beefyDeployment.abi)
    let beefyClient = beefyClientContract.connect(signer)

    let [cur, next, latestMMRRoot, latestBeefyBlock] = await Promise.all([
        beefyClient.currentValidatorSet(),
        beefyClient.nextValidatorSet(),
        beefyClient.latestMMRRoot(),
        beefyClient.latestBeefyBlock(),
    ])

    console.log({
        current: {
            id: cur.id.toString(),
        },
        next: {
            id: next.id.toString(),
        },
        latestMMRRoot,
        latestBeefyBlock: latestBeefyBlock.toString(),
    })

    return
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
beefyState()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error)
        process.exit(1)
    })
