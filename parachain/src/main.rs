// Copyright 2019-2021 Snowfork
// This file is part of Snowbridge

//! Artemis parachain collator

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod rpc;

fn main() -> sc_cli::Result<()> {
	command::run()
}
