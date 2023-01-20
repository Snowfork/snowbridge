import { task } from "hardhat/config";

task("contractAddressList", "Print the list of deployed contract addresses.")
    .setAction(async (TaskArguments, hre) => {
        let { name, chainId } = await hre.ethers.provider.getNetwork()
        name = name === 'unknown' ? 'localhost' : name

        const BasicInboundChannel = await hre.deployments.get("BasicInboundChannel")
        const BasicOutboundChannel = await hre.deployments.get("BasicOutboundChannel")
        const BeefyClient = await hre.deployments.get("BeefyClient")

        const addresses = {
            "BasicInboundChannel": BasicInboundChannel.address,
            "BasicOutboundChannel": BasicOutboundChannel.address,
            "BeefyClient": BeefyClient.address,
            "name": name,
            "chainId": chainId,
        }

        console.log(JSON.stringify(addresses, undefined, 2))
    });
