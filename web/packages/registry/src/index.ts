import { AssetRegistry } from "@snowbridge/base-types";
import polkadot_mainnet from "./polkadot_mainnet.registry.json";
import westend_sepolia from "./westend_sepolia.registry.json";
import paseo_sepolia from "./paseo_sepolia.registry.json";
import local_e2e from "./local_e2e.registry.json";

function transformBigInt(obj: any): any {
  // Regex to match strings like "bigint:123"
  const bigintPattern = /^bigint:(\d+)$/;

  // Handle null or non-object/non-array values
  if (obj === null || typeof obj !== "object") {
    if (typeof obj === "string") {
      const match = obj.match(bigintPattern);
      if (match) {
        return Object.freeze(BigInt(match[1]));
      }
    }
    return Object.freeze(obj);
  }

  // Handle arrays
  if (Array.isArray(obj)) {
    return Object.freeze(obj.map((item) => transformBigInt(item)));
  }

  // Handle objects
  const result: { [key: string]: any } = {};
  for (const key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      result[key] = transformBigInt(obj[key]);
    }
  }
  return Object.freeze(result);
}

const cache: { [env: string]: AssetRegistry } = {};
export function assetRegistryFor(
  env: "polkadot_mainnet" | "westend_sepolia" | "paseo_sepolia" | (string & {})
): AssetRegistry {
  if (env in cache) {
    return cache[env];
  }
  let json;
  switch (env) {
    case "polkadot_mainnet":
      json = polkadot_mainnet;
      break;
    case "westend_sepolia":
      json = westend_sepolia;
      break;
    case "paseo_sepolia":
      json = paseo_sepolia;
      break;
    case "local_e2e":
      json = local_e2e;
      break;
    default:
      throw Error(`Unknown env '${env}'`);
  }
  cache[env] = transformBigInt(json);
  return cache[env];
}
