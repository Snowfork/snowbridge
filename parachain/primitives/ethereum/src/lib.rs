#![cfg_attr(not(feature = "std"), no_std)]

mod log;
mod event;

pub use crate::{
	log::Log,
	event::Event,
};
