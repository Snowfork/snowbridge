import { config as minimalConfig } from "@chainsafe/lodestar-config/default";

function startBeaconNetwork() {
  console.log("Hello");
  console.log(minimalConfig);
}

const main = async () => {
  startBeaconNetwork();
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});

