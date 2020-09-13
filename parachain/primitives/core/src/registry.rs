
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
		id: [13, 39, 176, 6, 146, 65, 192, 53, 117, 102, 159, 237, 27, 173, 203, 204, 220, 13, 212, 209],
	},
	Entry {
		app: App::ERC20,
		id: [143, 225, 177, 35, 63, 112, 50, 206, 248, 207, 197, 234, 175, 65, 29, 255, 170, 119, 160, 124],
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
