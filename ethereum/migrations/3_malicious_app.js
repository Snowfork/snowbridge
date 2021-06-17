require("dotenv").config();

const MaliciousApp = artifacts.require("MaliciousApp");

module.exports = function (deployer, network, accounts) {
  deployer.then(async () => {

    if (network === 'development') {
      return
    }

    await deployer.deploy(MaliciousApp);

  })
};
