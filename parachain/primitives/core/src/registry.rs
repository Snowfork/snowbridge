
use crate::types::AppID;

#[derive(Copy, Clone)]
pub enum AppName {
	ETH,
	ERC20,
}

#[derive(Copy, Clone)]
pub struct Entry {
	pub name: AppName,
	pub id: AppID,
}

pub static REGISTRY: &[Entry] = &[
	Entry {
		name: AppName::ETH,
		id: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
	},
	Entry {
		name: AppName::ERC20,
		id: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
	}
];
