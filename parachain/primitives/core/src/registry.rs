
use crate::types::AppId;

/// Short identifer for an application.
#[derive(Copy, Clone)]
pub enum App {
	ETH,
	ERC20,
}

#[derive(Copy, Clone)]
struct Entry {
	pub app: App,
	pub id: AppId,
}

static APP_REGISTRY: &[Entry] = &[
	Entry {
		app: App::ETH,
		id: include!(concat!(env!("OUT_DIR"), "/eth_app_id.rs")),
	},
	Entry {
		app: App::ERC20,
		id: include!(concat!(env!("OUT_DIR"), "/erc20_app_id.rs")),
	}
];

/// Looks up an application in the registry identified by `app_id`.
pub fn lookup_app(app_id: AppId) -> Option<App> {
	for entry in APP_REGISTRY.iter() {
		if app_id == entry.id {
			return Some(entry.app)
		}
	}
	None
}
