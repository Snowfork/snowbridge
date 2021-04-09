const readline = require('readline');

const PATTERNS = {
    'block_execution_weight': /^Block import \(Noop\/empty/,
    'extrinsic_base_weight': /^Block import \(Noop\/custom/,
    'rocksdb_read_weight': /^Trie read benchmark/,
    'rocksdb_write_weight': /^Trie write benchmark/,
};

function extractWeights(benchOutput) {
    return benchOutput
        .flat()
        .filter(output => output.name.includes("RocksDb"))
        .map(output => {
            const [key, _] = Object.entries(PATTERNS).find(([_, p]) => p.test(output.name));
            return [key, output.average];
        })
        .reduce(
            (obj, [key, weight]) => key === undefined ? obj : { ...obj, [key]: weight },
            {},
        );
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
      extractWeights(JSON.parse(buffer)),
      null, // replacer
      4, // spaces
    ));
  });
}

run();