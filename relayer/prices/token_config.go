package prices

import (
	"encoding/hex"
	"strings"
)

const EthLlamaKey = "ethereum:0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"

type TokenPriceConfig struct {
	LlamaKey string
	Decimals uint8
	Symbol   string
	// FixedPriceUSD is a hardcoded USD price for tokens with no live feed. When
	// set it is used as a last-resort fallback (the indexer already bakes it into
	// its stored prices); update manually if the price drifts materially.
	FixedPriceUSD float64
}

// tokenPriceByAddress maps Snowbridge ERC-20 contract addresses to DefiLlama price keys.
// Ported from snowbridge-subsquid/src/pricefetcher/tokenConfig.ts.
var tokenPriceByAddress = map[string]TokenPriceConfig{
	"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48": {LlamaKey: "ethereum:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", Decimals: 6, Symbol: "USDC"},
	"0xdac17f958d2ee523a2206206994597c13d831ec7": {LlamaKey: "ethereum:0xdac17f958d2ee523a2206206994597c13d831ec7", Decimals: 6, Symbol: "USDT"},
	"0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2": {LlamaKey: "ethereum:0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", Decimals: 18, Symbol: "WETH"},
	"0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0": {LlamaKey: "ethereum:0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0", Decimals: 18, Symbol: "wstETH"},
	"0x514910771af9ca656af840dff83e8264ecf986ca": {LlamaKey: "ethereum:0x514910771af9ca656af840dff83e8264ecf986ca", Decimals: 18, Symbol: "LINK"},
	"0x2260fac5e5542a773aa44fbcfedf7c193bc2c599": {LlamaKey: "ethereum:0x2260fac5e5542a773aa44fbcfedf7c193bc2c599", Decimals: 8, Symbol: "WBTC"},
	"0x57e114b691db790c35207b2e685d4a43181e6061": {LlamaKey: "ethereum:0x57e114b691db790c35207b2e685d4a43181e6061", Decimals: 18, Symbol: "ENA"},
	"0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9": {LlamaKey: "ethereum:0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9", Decimals: 18, Symbol: "AAVE"},
	"0x5a98fcbea516cf06857215779fd812ca3bef1b32": {LlamaKey: "ethereum:0x5a98fcbea516cf06857215779fd812ca3bef1b32", Decimals: 18, Symbol: "LDO"},
	"0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003": {LlamaKey: "ethereum:0xba41ddf06b7ffd89d1267b5a93bfef2424eb2003", Decimals: 18, Symbol: "MYTH"},
	"0x45804880de22913dafe09f4980848ece6ecbaf78": {LlamaKey: "ethereum:0x45804880de22913dafe09f4980848ece6ecbaf78", Decimals: 18, Symbol: "PAXG"},
	"0x9d39a5de30e57443bff2a8307a4256c8797a3497": {LlamaKey: "ethereum:0x9d39a5de30e57443bff2a8307a4256c8797a3497", Decimals: 18, Symbol: "sUSDe"},
	"0x56072c95faa701256059aa122697b133aded9279": {LlamaKey: "ethereum:0x56072c95faa701256059aa122697b133aded9279", Decimals: 18, Symbol: "SKY"},
	"0xcccccccccc33d538dbc2ee4feab0a7a1ff4e8a94": {LlamaKey: "ethereum:0xcccccccccc33d538dbc2ee4feab0a7a1ff4e8a94", Decimals: 18, Symbol: "CFG"},
	"0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f": {LlamaKey: "ethereum:0xaa7a9ca87d3694b5755f213b5d04094b8d0f0a6f", Decimals: 18, Symbol: "TRAC"},
	"0x18084fba666a33d37592fa2633fd49a74dd93a88": {LlamaKey: "ethereum:0x18084fba666a33d37592fa2633fd49a74dd93a88", Decimals: 18, Symbol: "tBTC"},
	"0x5d3d01fd6d2ad1169b17918eb4f153c6616288eb": {LlamaKey: "ethereum:0x5d3d01fd6d2ad1169b17918eb4f153c6616288eb", Decimals: 15, Symbol: "KILT", FixedPriceUSD: 0.0177}, // as of June 2026
	"0x38eeb52f0771140d10c4e9a9a72349a329fe8a6a": {LlamaKey: "ethereum:0x38eeb52f0771140d10c4e9a9a72349a329fe8a6a", Decimals: 18, Symbol: "apyUSD"},
	"0x1f752250f518e419f2f03804e25728c5c6dff73f": {LlamaKey: "ethereum:0x1f752250f518e419f2f03804e25728c5c6dff73f", Decimals: 9, Symbol: "XRT"},
	"0x6b175474e89094c44da98b954eedeac495271d0f": {LlamaKey: "ethereum:0x6b175474e89094c44da98b954eedeac495271d0f", Decimals: 18, Symbol: "DAI"},
	"0x6982508145454ce325ddbe47a25d4ec3d2311933": {LlamaKey: "ethereum:0x6982508145454ce325ddbe47a25d4ec3d2311933", Decimals: 18, Symbol: "PEPE"},
	"0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce": {LlamaKey: "ethereum:0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce", Decimals: 18, Symbol: "SHIB"},
	"0x163f8c2467924be0ae7b5347228cabf260318753": {LlamaKey: "ethereum:0x163f8c2467924be0ae7b5347228cabf260318753", Decimals: 18, Symbol: "WLD"},
	"0x1abaea1f7c830bd89acc67ec4af516284b1bc33c": {LlamaKey: "ethereum:0x1abaea1f7c830bd89acc67ec4af516284b1bc33c", Decimals: 6, Symbol: "EURC"},
	"0x8236a87084f8b84306f72007f36f2618a5634494": {LlamaKey: "ethereum:0x8236a87084f8b84306f72007f36f2618a5634494", Decimals: 8, Symbol: "LBTC"},
	"0xb62132e35a6c13ee1ee0f84dc5d40bad8d815206": {LlamaKey: "ethereum:0xb62132e35a6c13ee1ee0f84dc5d40bad8d815206", Decimals: 18, Symbol: "NEXO"},
	"0x582d872a1b094fc48f5de31d3b73f2d9be47def1": {LlamaKey: "ethereum:0x582d872a1b094fc48f5de31d3b73f2d9be47def1", Decimals: 9, Symbol: "TONCOIN"},
	"0xa3931d71877c0e7a3148cb7eb4463524fec27fbd": {LlamaKey: "ethereum:0xa3931d71877c0e7a3148cb7eb4463524fec27fbd", Decimals: 18, Symbol: "sUSDS"},
	"0xd166337499e176bbc38a1fbd113ab144e5bd2df7": {LlamaKey: "ethereum:0xd166337499e176bbc38a1fbd113ab144e5bd2df7", Decimals: 18, Symbol: "sUSDat"},
	"0x8daebade922df735c38c80c7ebd708af50815faa": {LlamaKey: "ethereum:0x8daebade922df735c38c80c7ebd708af50815faa", Decimals: 18, Symbol: "TBTC"},
	"0x0000000000000000000000000000000000000000": {LlamaKey: EthLlamaKey, Decimals: 18, Symbol: "ETH"},
	"0x196c20da81fbc324ecdf55501e95ce9f0bd84d14": {LlamaKey: "coingecko:polkadot", Decimals: 10, Symbol: "DOT"},
	"0x12bbfdc9e813614eef8dc8a2560b0efbeaf7c2ab": {LlamaKey: "coingecko:kusama", Decimals: 12, Symbol: "KSM"},
	"0x769916a66fdac0e3d57363129caac59386ea622b": {LlamaKey: "coingecko:integritee", Decimals: 12, Symbol: "TEER", FixedPriceUSD: 0.0038}, // as of June 2026
	// vTAO has no direct price feed; proxied to Bittensor TAO (live via coingecko).
	"0xe9f6d9898f9269b519e1435e6ebaff766c7f46bf": {LlamaKey: "coingecko:bittensor", Decimals: 18, Symbol: "vTAO"},
	// CGT2.0 (Curio Gas Token) has no public price source; placeholder until one exists.
	"0x0e186357c323c806c1efdad36d217f7a54b63d18": {LlamaKey: "ethereum:0x0e186357c323c806c1efdad36d217f7a54b63d18", Decimals: 18, Symbol: "CGT2.0", FixedPriceUSD: 0.01}, // placeholder, no market price as of June 2026
}

// foreignTokenPriceByID maps v2 MintForeignToken foreignTokenID hashes to price config.
var foreignTokenPriceByID = map[string]TokenPriceConfig{
	"4e241583d94b5d48a27a22064cd49b2ed6f5231d2d950e432f9b7c2e0ade52b2": {LlamaKey: "coingecko:polkadot", Decimals: 10, Symbol: "DOT"},
	"03b6054d0c576dd8391e34e1609cf398f68050c23009d19ce93c000922bcd852": {LlamaKey: "coingecko:kusama", Decimals: 12, Symbol: "KSM"},
}

func TokenPriceConfigByAddress(address string) (TokenPriceConfig, bool) {
	config, ok := tokenPriceByAddress[strings.ToLower(address)]
	return config, ok
}

func TokenPriceConfigByForeignID(foreignTokenID [32]byte) (TokenPriceConfig, bool) {
	key := hex.EncodeToString(foreignTokenID[:])
	config, ok := foreignTokenPriceByID[key]
	return config, ok
}

// fixedPriceByLlamaKey indexes the configured fixed prices by DefiLlama key so a
// price lookup can fall back to them when no live source has a price.
var fixedPriceByLlamaKey = buildFixedPriceByLlamaKey()

func buildFixedPriceByLlamaKey() map[string]float64 {
	out := make(map[string]float64)
	for _, config := range tokenPriceByAddress {
		if config.FixedPriceUSD > 0 {
			out[config.LlamaKey] = config.FixedPriceUSD
		}
	}
	for _, config := range foreignTokenPriceByID {
		if config.FixedPriceUSD > 0 {
			out[config.LlamaKey] = config.FixedPriceUSD
		}
	}
	return out
}

// FixedPriceByLlamaKey returns the configured fixed USD price for a llama key, if any.
func FixedPriceByLlamaKey(llamaKey string) (float64, bool) {
	price, ok := fixedPriceByLlamaKey[llamaKey]
	return price, ok
}
