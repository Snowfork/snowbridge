#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::parameter_types;

parameter_types! {
	pub const MaxMessagePayloadSize: u32 = 256;
	pub const MaxMessagesPerCommit: u32 = 20;
}
