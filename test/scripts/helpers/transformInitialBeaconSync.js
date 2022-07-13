const readline = require('readline');

function parseBeaconHeader(input) {
  let data = JSON.parse(input)['result'];
  if (!data) {
    throw Error("Failed to parse header from input. Expected HTTP response data as input");
  }
  return data;
}

function transformBeaconForParachain(input) {
  let output = input["data"];
  output["validators_root"] = "";
  return output;
}

function run() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
  });

  let buffer = "";
  rl.on('line', function(line){
    buffer += line;
  });

  rl.on('close', function() {
    console.log(JSON.stringify(
      transformBeaconForParachain(parseBeaconHeader(buffer)),
      null, // replacer
      4, // spaces
    ));
  });
}

run();
