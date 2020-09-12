
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
		id: [13, 39, 176, 6, 146, 65, 192, 53, 117, 102, 159, 237, 27, 173, 203, 204, 220, 13, 212, 209],
	},
	Entry {
		name: AppName::ERC20,
		id: [143, 225, 177, 35, 63, 112, 50, 206, 248, 207, 197, 234, 175, 65, 29, 255, 170, 119, 160, 124],
	}
];


