// Sleep is a wait function
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

// fetchDeployedAddrs fetches the ETHApp, ERC20App, and TEST token contract addresses as JSON
async function fetchDeployedAddrs() {
    const cmd = "cd ../ethereum && truffle exec scripts/fetchDeployedAddrs.js"
    const appsStr = await execShellCommand(cmd);

    var regExp = /\{([^)]+)\}/;
    var matches = regExp.exec(appsStr);
    return JSON.parse("{" + matches[1] + "}");
}

// execShellCommand executes a shell command
function execShellCommand(cmd) {
    const exec = require('child_process').exec;
    return new Promise((resolve, reject) => {
        exec(cmd, (error, stdout, stderr) => {
        if (error) {
        console.warn(error);
        }
        resolve(stdout? stdout : stderr);
        });
    });
}

module.exports = {
    sleep,
    fetchDeployedAddrs
};
