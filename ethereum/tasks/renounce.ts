import { task } from "hardhat/config";
import { TaskArguments } from "hardhat/types";

task("renounce")
    .addParam("appaddr", "The app you want to renounce ownership of")
    .setAction(async ({appaddr}: TaskArguments, { ethers }) => {
  const accounts = await ethers.getSigners();
  const app = await ethers.getContractAt(["function renounceRole(bytes32 role, address account)"], appaddr)
  const tx = await app.renounceRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes("CHANNEL_UPGRADE_ROLE")), accounts[0].address);
  console.log(`tx submitted... https://etherscan.io/tx/${tx.hash}`);
  const receipt = await tx.wait();
  console.log(`tx mined, receipt:`);
  console.log(JSON.stringify(receipt, undefined, 2))
});
