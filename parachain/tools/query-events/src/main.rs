#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod runtime {}

use codec::Encode;
use serde::Serialize;
use serde_hex::{SerHexSeq, StrictPfx};
use serde_json;
use std::{
	io::{self, Write},
	str::FromStr,
};
use subxt::{ext::sp_core::H256, OnlineClient, PolkadotConfig};

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

	let query = runtime::storage().system().events();
	let events = api.storage().fetch(&query, Some(block_hash.into())).await?;

	let mut items: Vec<Item> = Vec::new();

	for event in events.expect("Failed to get events for block.").into_iter() {
		if let runtime::runtime_types::snowbase_runtime::Event::BasicOutboundChannel(ev) =
			&event.event
		{
			if let runtime::runtime_types::snowbridge_basic_channel::outbound::pallet::Event::Committed { hash, data } = ev {
					items.push(Item {
						id: 0,
						hash: hash.encode(),
						data: data.encode(),
					});
				}
		}
		if let runtime::runtime_types::snowbase_runtime::Event::IncentivizedOutboundChannel(ev) =
			&event.event
		{
			if let runtime::runtime_types::snowbridge_incentivized_channel::outbound::pallet::Event::Committed { hash, data } = ev {
					items.push(Item {
						id: 1,
						hash: hash.encode(),
						data: data.encode(),
					});
				}
		}
	}

	let output = &serde_json::to_string(&Items { items })?;
	io::stdout().write_all(output.as_bytes())?;
	io::stdout().write(b"\n")?;

	Ok(())
}
