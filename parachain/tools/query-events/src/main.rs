use codec::Encode;
use serde::Serialize;
use serde_hex::{SerHexSeq, StrictPfx};
use serde_json;
use std::{
	io::{self, Write},
	str::FromStr,
};
use subxt::{ext::sp_core::H256, OnlineClient, PolkadotConfig};

#[cfg_attr(feature = "parachain-snowbase",
  subxt::subxt(runtime_metadata_path = "snowbase-metadata.scale")
)]
#[cfg_attr(feature = "parachain-snowblink",
  subxt::subxt(runtime_metadata_path = "snowblink-metadata.scale")
)]
pub mod runtime {}


#[derive(Debug, Serialize)]
struct Items {
	items: Vec<Item>,
}

#[derive(Debug, Serialize)]
struct Item {
	id: u8,
	#[serde(with = "SerHexSeq::<StrictPfx>")]
	hash: Vec<u8>,
	#[serde(with = "SerHexSeq::<StrictPfx>")]
	data: Vec<u8>,
}

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
	/// Name of the person to greet
	#[clap(short, long)]
	api: String,

	/// Number of times to greet
	#[clap(short, long)]
	block: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();
	let block_hash = H256::from_str(args.block.trim_start_matches("0x"))?;
	let api = OnlineClient::<PolkadotConfig>::from_url(args.api).await?;
	let events = api.events().at(Some(block_hash.into())).await?;
	let mut items: Vec<Item> = Vec::new();

	for ev in events.find::<runtime::basic_outbound_channel::events::Committed>() {
		if let Ok(runtime::basic_outbound_channel::events::Committed { hash , data }) = ev {
			items.push(Item {
				id: 0,
				hash: hash.encode(),
				data: data.encode(),
			});
		}
	}

	for ev in events.find::<runtime::incentivized_outbound_channel::events::Committed>() {
		if let Ok(runtime::incentivized_outbound_channel::events::Committed { hash , data }) = ev {
			items.push(Item {
				id: 1,
				hash: hash.encode(),
				data: data.encode(),
			});
		}
	}

	let output = &serde_json::to_string(&Items { items })?;
	io::stdout().write_all(output.as_bytes())?;
	io::stdout().write(b"\n")?;

	Ok(())
}
