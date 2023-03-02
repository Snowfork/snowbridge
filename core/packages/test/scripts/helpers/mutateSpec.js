const readline = require('readline');
const fs = require('fs');

function run() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
  });

  function getTimestampInSeconds () {
    return Math.floor(Date.now() / 1000)
  }

  let buffer = "";
  rl.on('line', function(line) {
    buffer += line;
  });

  rl.on('close', function() {
    data = JSON.parse(buffer);

    let contracts = JSON.parse(fs.readFileSync(process.argv[2]));
    let initialSync = JSON.parse(fs.readFileSync(process.argv[3]));

    data['genesis']['runtime']['ethereumBeaconClient']['initialSync'] = initialSync;
    data['genesis']['runtime']['ethereumBeaconClient']['initialSync']['import_time'] = getTimestampInSeconds()
    data['genesis']['runtime']['parachainInfo']['parachainId'] = 1000;
    data['para_id'] = 1000;

    data['genesis']['runtime']['basicInboundChannel']['sourceChannel'] = contracts['contracts']['BasicOutboundChannel']['address'];

    console.log(JSON.stringify(
      data,
      null, // replacer
      4, // spaces
    ));
  });
}

run();
