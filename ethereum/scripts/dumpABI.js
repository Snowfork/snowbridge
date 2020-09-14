#!/usr/bin/env node

const fs = require('fs');

const args = process.argv.slice(2);
const contract = JSON.parse(fs.readFileSync(args[0], 'utf8'));

console.log(JSON.stringify(contract.abi, null, 2));
