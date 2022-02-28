import { task } from "hardhat/config";

task("contractAddressList", "Print the list of deployed contract addresses.")
    .setAction(async (TaskArguments, hre) => {
        
        let { name, chainId } = await hre.ethers.provider.getNetwork()
        name = name === 'unknown' ? 'localhost' : name

        console.log("name", name)
        console.log("chainId", chainId)

        const BasicInboundChannel = await hre.deployments.get("BasicInboundChannel")
        const BasicOutboundChannel = await hre.deployments.get("BasicOutboundChannel")
        const IncentivizedInboundChannel = await hre.deployments.get("IncentivizedInboundChannel")
        const IncentivizedOutboundChannel = await hre.deployments.get("IncentivizedOutboundChannel")
        const ETHApp = await hre.deployments.get("ETHApp")
        const ERC20App = await hre.deployments.get("ERC20App")
        const DOTApp = await hre.deployments.get("DOTApp")
        const ERC721App = await hre.deployments.get("ERC721App")
        const DOTAppContract = await hre.ethers.getContractAt("DOTApp", DOTApp.address);
        const SnowDOTAddress = await DOTAppContract.token();

        console.log("BasicInboundChannel", BasicInboundChannel.address)
        console.log("BasicOutboundChannel", BasicOutboundChannel.address)
        console.log("IncentivizedInboundChannel", IncentivizedInboundChannel.address)
        console.log("IncentivizedOutboundChannel", IncentivizedOutboundChannel.address)
        console.log("ETHApp", ETHApp.address)
        console.log("ERC20App", ERC20App.address)
        console.log("DOTApp", DOTApp.address)
        console.log("ERC721App", ERC721App.address)
        console.log("SnowDOTAddress", SnowDOTAddress)

    });
