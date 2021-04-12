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

function calculateUnits(weights, transactionsInBlock) {
  return Object.entries(weights)
    .reduce(
      (obj, [key, value]) => {
        if (key === 'extrinsic_base_weight') {
          value = value / transactionsInBlock;
        }

        return {
          ...obj,
          [key + '_in_nanos']: value,
          [key + '_in_micros']: Math.ceil(value / 1000),
          [key + '_in_millis']: Math.ceil(value / 1000000),
        };
      },
      {},
    );
}

function run() {
  const transactionsInBlock = parseInt(process.argv[2]);
  if (!transactionsInBlock) {
      throw Error("Expected number of transactions in block as argument");
  }

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
    const weights = extractWeights(JSON.parse(buffer));

    console.log(JSON.stringify(
      calculateUnits(weights, transactionsInBlock),
      null, // replacer
      4, // spaces
    ));
  });
}

run();
