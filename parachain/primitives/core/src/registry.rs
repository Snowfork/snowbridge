
use crate::types::AppID;

pub enum AppName {
	PolkaETH,
	PolkaERC20,
}

pub struct Entry {
	pub symbol: AppName,
	pub id: AppID,
}

pub static REGISTRY: &[Entry] = &[
	Entry {
		symbol: AppName::PolkaETH,
		// AppID is currently unused, so set it to zeroes.
		id: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
	},
	Entry {
		symbol: AppName::PolkaERC20,
		// AppID is currently unused, so set it to zeroes.
		id: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
	}
];
