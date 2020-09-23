#!/bin/bash

cd ../ethereum

truffle exec scripts/dumpRelayerConfig.js | sed '/^Using/d;/^$/d'
