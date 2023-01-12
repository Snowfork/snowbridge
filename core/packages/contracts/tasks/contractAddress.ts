import { task } from "hardhat/config";

task("contractAddressList", "Print the list of deployed contract addresses.")
    .setAction(async (TaskArguments, hre) => {
        let { name, chainId } = await hre.ethers.provider.getNetwork()
        name = name === 'unknown' ? 'localhost' : name

        const BasicInboundChannel = await hre.deployments.get("BasicInboundChannel")
        const BasicOutboundChannel = await hre.deployments.get("BasicOutboundChannel")
        const ETHApp = await hre.deployments.get("ETHApp")
        const ERC20App = await hre.deployments.get("ERC20App")
        const DOTApp = await hre.deployments.get("DOTApp")
        const DOTAppContract = await hre.ethers.getContractAt("DOTApp", DOTApp.address);
        const BeefyClient = await await hre.deployments.get("BeefyClient")
        const SnowDOTAddress = await DOTAppContract.token();

        const addresses = {
            "BasicInboundChannel": BasicInboundChannel.address,
            "BasicOutboundChannel": BasicOutboundChannel.address,
            "ETHApp": ETHApp.address,
            "ERC20App": ERC20App.address,
            "DOTApp": DOTApp.address,
            "SnowDOTAddress": SnowDOTAddress,
            "BeefyClient": BeefyClient.address,
            "name": name,
            "chainId": chainId,
        }

        console.log(JSON.stringify(addresses, undefined, 2))
    });
