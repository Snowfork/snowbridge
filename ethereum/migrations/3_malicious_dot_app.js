require("dotenv").config();

const MaliciousDOTApp = artifacts.require("MaliciousDOTApp");

module.exports = function (deployer, network, accounts) {
  deployer.then(async () => {

    if (network === 'development') {
      return
    }

    await deployer.deploy(MaliciousDOTApp);

  })
};
