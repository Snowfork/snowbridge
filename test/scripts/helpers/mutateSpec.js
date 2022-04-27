const readline = require('readline');
const fs = require('fs');

function run() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
  });

  let buffer = "";
  rl.on('line', function(line) {
    buffer += line;
  });

  rl.on('close', function() {
    data = JSON.parse(buffer);

    let header = JSON.parse(fs.readFileSync(process.argv[2]));
    let contracts = JSON.parse(fs.readFileSync(process.argv[3]));

    data['genesis']['runtime']['ethereumLightClient']['initialHeader'] = header;
    data['genesis']['runtime']['ethereumLightClient']['initialDifficulty'] = "0x0";
    data['genesis']['runtime']['parachainInfo']['parachainId'] = 1000;
    data['para_id'] = 1000;

    data['genesis']['runtime']['dotApp']['address'] = contracts['contracts']['DOTApp']['address'];
    data['genesis']['runtime']['ethApp']['address'] = contracts['contracts']['ETHApp']['address'];
    data['genesis']['runtime']['erc20App']['address'] = contracts['contracts']['ERC20App']['address'];
    data['genesis']['runtime']['incentivizedInboundChannel']['sourceChannel'] = contracts['contracts']['IncentivizedOutboundChannel']['address'];
    data['genesis']['runtime']['basicInboundChannel']['sourceChannel'] = contracts['contracts']['BasicOutboundChannel']['address'];

    console.log(JSON.stringify(
      data,
      null, // replacer
      4, // spaces
    ));
  });
}

run();
