import { assetsV2 } from "@snowbridge/api";
import polkadot_mainnet from "./polkadot_mainnet.registry.json";
import westend_sepolia from "./westend_sepolia.registry.json";
import paseo_sepolia from "./paseo_sepolia.registry.json";

function transformBigInt(obj: any): any {
  // Regex to match strings like "bigint:123"
  const bigintPattern = /^bigint:(\d+)$/;

  // Handle null or non-object/non-array values
  if (obj === null || typeof obj !== "object") {
    if (typeof obj === "string") {
      const match = obj.match(bigintPattern);
      if (match) {
        return BigInt(match[1]);
      }
    }
    return obj;
  }

  // Handle arrays
  if (Array.isArray(obj)) {
    return obj.map((item) => transformBigInt(item));
  }

  // Handle objects
  const result: { [key: string]: any } = {};
  for (const key in obj) {
    if (Object.prototype.hasOwnProperty.call(obj, key)) {
      result[key] = transformBigInt(obj[key]);
    }
  }
  return result;
}

const cache: { [env: string]: assetsV2.AssetRegistry } = {};
export function assetRegistryFor(
  env: "polkadot_mainnet" | "westend_sepolia" | "paseo_sepolia"
): assetsV2.AssetRegistry {
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
    default:
      throw Error(`Unkown env '${env}'`);
  }
  cache[env] = transformBigInt(json);
  return cache[env];
}
