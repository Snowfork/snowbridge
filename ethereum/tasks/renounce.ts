import { task } from "hardhat/config";
import { TaskArguments } from "hardhat/types";

task("renounce")
    .addParam("addr", "The address of the app or channel you want to renounce ownership of")
    .addParam("role", "The contract role to renounce (e.g. CHANNEL_UPGRADE_ROLE or BEEFY_UPGRADE_ROLE)")
    .setAction(async ({addr, role}: TaskArguments, { ethers }) => {
  const accounts = await ethers.getSigners();
  const app = await ethers.getContractAt(["function renounceRole(bytes32 role, address account)"], addr)
  const tx = await app.renounceRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes(role)), accounts[0].address);
  console.log(`tx submitted... https://etherscan.io/tx/${tx.hash}`);
  await tx.wait();
  console.log(`tx mined!`);
});
