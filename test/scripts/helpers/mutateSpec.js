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
    data['genesis']['runtime']['ethereumLightClient']['initialHeader'] = header;
    data['genesis']['runtime']['ethereumLightClient']['initialDifficulty'] = "0x0";
    data['genesis']['runtime']['parachainInfo']['parachainId'] = 1000;
    data['para_id'] = 1000;

    console.log(JSON.stringify(
      data,
      null, // replacer
      4, // spaces
    ));
  });
}

run();
