const BridgeClient = require('./src/client').BridgeClient;

const main = async() => {
    const endpoint = "http://localhost:9545";
    const bridgeClient = new BridgeClient(endpoint);
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
