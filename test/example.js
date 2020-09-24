const BridgeClient = require('./src/client').BridgeClient;

// BridgeClient instantiation variables
const endpoint = "http://localhost:9545";
const ethAppAddress = "0x0823eFE0D0c6bd134a48cBd562fE4460aBE6e92c";
const erc20AppAddress = "0x5040BA3Cf968de7273201d7C119bB8D8F03BDcBc";

const main = async() => {
    const bridgeClient = new BridgeClient(endpoint, ethAppAddress, erc20AppAddress);
    await bridgeClient.initWallet();

    const testTokenContractAddress = "0xB69271c677cFeC73bD61b0AA39158E9262397a38";
    const polkadotRecipient = "38j4dG5GzsL1bw2U2AVgeyAk6QTxq43V7zPbdXAmbVLjvDCK";
    const txHash = "0x50f5e90f9683edf7dee65302de787e90b8f72d661dfb7ab0904137549fd0116e";

    const res = await bridgeClient.getEthBalance();
    // const res = await bridgeClient.sendEth("5", polkadotRecipient);
    // const res = await bridgeClient.getErc20Balance(testTokenContractAddress);
    // const res = await bridgeClient.getErc20Allowance(testTokenContractAddress);
    // const res = await bridgeClient.approveERC20(500, testTokenContractAddress);
    // const res = await bridgeClient.sendERC20(100, testTokenContractAddress, polkadotRecipient);
    // const res = await bridgeClient.getTx(txHash);

    console.log("res:", res);
}

main();
