import { Signer } from "@ethersproject/abstract-signer";
import { task } from "hardhat/config";
import { TaskArguments } from "hardhat/types";
task("upgrade")
    .addParam("appaddr", "The app you want to upgrade")
    .addParam("basicinbound")
    .addParam("basicoutbound")
    .addParam("incinbound")
    .addParam("incoutbound")
    .setAction(async ({appaddr, basicinbound, basicoutbound, incinbound, incoutbound}: TaskArguments, { ethers }) => {
  const accounts: Signer[] = await ethers.getSigners();
  const app = await ethers.getContractAt(["function upgrade(tuple(address inbound, address outbound) _basic, tuple(address inbound, address outbound) _incentivized)"], appaddr)
  const tx = await app.upgrade(
    [basicinbound, basicoutbound],
    [incinbound, incoutbound]
  );
  console.log(`tx submitted... https://etherscan.io/tx/${tx.hash}`);
  const receipt = await tx.wait();
  console.log(`tx mined, receipt:`);
  console.log(JSON.stringify(receipt, undefined, 2))
});
